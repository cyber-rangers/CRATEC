use crate::logger::{log_debug, log_error, log_warn};
use serde::Serialize;
use std::process::Command;
use std::str;
use tauri::command;
use udev::{Device, Enumerator};

#[derive(Serialize, Debug)]
pub struct HddDetails {
    pub model_family: String,
    pub device_model: String,
    pub serial_number: String,
    pub firmware_version: String,
    pub user_capacity: String,
    pub sector_sizes: String,
    pub rotation_rate: String,
    pub form_factor: String,
    pub smart_status: String,
    pub smart_attributes: Vec<SmartAttribute>,
    pub full_output: String, // Obsahuje celý výstup smartctl
}

#[derive(Serialize, Debug)]
pub struct SmartAttribute {
    pub id: u8,
    pub name: String,
    pub flag: String,
    pub value: u16,
    pub worst: u16,
    pub thresh: u16,
    pub type_field: String,
    pub updated: String,
    pub when_failed: String,
    pub raw_value: String,
}

#[derive(Serialize, Debug)]
pub struct UsbDeviceDetails {
    pub current_tags: String,
    pub devlinks: String,
    pub devname: String,
    pub devpath: String,
    pub devtype: String,
    pub diskseq: String,
    pub id_bus: String,
    pub id_fs_boot_system_id: String,
    pub id_fs_data_preparer_id: String,
    pub id_fs_label: String,
    pub id_fs_label_enc: String,
    pub id_fs_type: String,
    pub id_fs_usage: String,
    pub id_fs_uuid: String,
    pub id_fs_uuid_enc: String,
    pub id_fs_version: String,
    pub id_instance: String,
    pub id_model: String,
    pub id_model_enc: String,
    pub id_model_id: String,
    pub id_part_table_type: String,
    pub id_part_table_uuid: String,
    pub id_path: String,
    pub id_path_tag: String,
    pub id_revision: String,
    pub id_serial: String,
    pub id_serial_short: String,
    pub id_type: String,
    pub id_usb_driver: String,
    pub id_usb_interfaces: String,
    pub id_usb_interface_num: String,
    pub id_vendor: String,
    pub id_vendor_enc: String,
    pub id_vendor_id: String,
    pub major: String,
    pub minor: String,
    pub subsystem: String,
    pub tags: String,
    pub usec_initialized: String,
}

/// Najde v systému block zařízení, jehož vlastnost "ID_PATH" se rovná zadanému fyzickému identifikátoru,
/// a vrátí jeho devnode (např. "/dev/sda").
fn find_devnode_by_physical_path(physical_path: &str) -> Result<String, String> {
    let mut enumerator = Enumerator::new().map_err(|e| {
        log_error(&format!("Failed to create udev enumerator: {}", e));
        format!("Failed to create udev enumerator: {}", e)
    })?;
    enumerator.match_subsystem("block").map_err(|e| {
        log_error(&format!("Failed to match block subsystem: {}", e));
        format!("Failed to match block subsystem: {}", e)
    })?;
    let devnode = enumerator
        .scan_devices()
        .map_err(|e| {
            log_error(&format!("Failed to scan block devices: {}", e));
            format!("Failed to scan block devices: {}", e)
        })?
        .find(|device| {
            device
                .property_value("ID_PATH")
                .and_then(|v| v.to_str())
                // Místo přísné rovnosti ověříme, zda hodnota začíná na zadaný řetězec.
                .map(|s| s.starts_with(physical_path))
                .unwrap_or(false)
        })
        .and_then(|device| device.devnode().map(|n| n.to_string_lossy().to_string()))
        .ok_or_else(|| {
            let msg = format!("No block device found with physical path: {}", physical_path);
            log_error(&msg);
            msg
        })?;
    log_debug(&format!("Found devnode {} for physical path {}", devnode, physical_path));
    Ok(devnode)
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_hdd_details(physical_path: &str) -> Result<HddDetails, String> {
    log_debug(&format!("Fetching HDD details for physical path: {}", physical_path));
    
    // Najdeme odpovídající devnode na základě fyzického identifikátoru
    let devnode = find_devnode_by_physical_path(physical_path)?;

    let output = Command::new("smartctl")
        .arg("--all")
        .arg(&devnode)
        .output()
        .map_err(|e| {
            log_error(&format!("Failed to execute smartctl: {}", e));
            format!("Failed to execute smartctl: {}", e)
        })?;

    let stdout = str::from_utf8(&output.stdout).map_err(|e| {
        log_error(&format!("Invalid UTF-8 output from smartctl: {}", e));
        format!("Invalid UTF-8 output from smartctl: {}", e)
    })?;

    let exit_code = output.status.code().unwrap_or(-1);
    if exit_code != 0 && exit_code != 4 {
        log_warn(&format!(
            "smartctl exited with code {}: Drive may have issues.",
            exit_code
        ));
    }

    let model_family = extract_value(stdout, "Model Family:");
    let device_model = extract_value(stdout, "Device Model:");
    let serial_number = extract_value(stdout, "Serial Number:");
    let firmware_version = extract_value(stdout, "Firmware Version:");
    let user_capacity = extract_value(stdout, "User Capacity:");
    let sector_sizes = extract_value(stdout, "Sector Size(s):");
    let rotation_rate = extract_value(stdout, "Rotation Rate:");
    let form_factor = extract_value(stdout, "Form Factor:");
    let smart_status = extract_value(stdout, "SMART overall-health self-assessment test result:");
    let smart_attributes = parse_smart_attributes(stdout);

    log_debug(&format!("Successfully fetched HDD details for device: {}", physical_path));

    Ok(HddDetails {
        model_family,
        device_model,
        serial_number,
        firmware_version,
        user_capacity,
        sector_sizes,
        rotation_rate,
        form_factor,
        smart_status,
        smart_attributes,
        full_output: stdout.to_string(),
    })
}

fn extract_value(output: &str, key: &str) -> String {
    output
        .lines()
        .find(|line| line.contains(key))
        .and_then(|line| line.splitn(2, ':').nth(1))
        .unwrap_or("Unknown")
        .trim()
        .to_string()
}

fn parse_smart_attributes(output: &str) -> Vec<SmartAttribute> {
    let mut attributes = Vec::new();
    let mut parsing_attributes = false;

    for line in output.lines() {
        if line.contains("ID# ATTRIBUTE_NAME") {
            parsing_attributes = true;
            continue;
        }

        if parsing_attributes {
            if line.is_empty()
                || line.starts_with("SMART Error Log")
                || line.contains("Vendor Specific SMART Attributes")
            {
                break;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                let id = parts[0].parse().unwrap_or(0);
                let name = parts[1].to_string();
                let flag = parts[2].to_string();
                let value = parts[3].parse().unwrap_or(0);
                let worst = parts[4].parse().unwrap_or(0);
                let thresh = parts[5].parse().unwrap_or(0);
                let type_field = parts[6].to_string();
                let updated = parts[7].to_string();
                let when_failed = parts[8].to_string();
                let raw_value = parts[9..].join(" ");

                attributes.push(SmartAttribute {
                    id,
                    name,
                    flag,
                    value,
                    worst,
                    thresh,
                    type_field,
                    updated,
                    when_failed,
                    raw_value,
                });
            } else {
                log_warn(&format!("Unexpected SMART attribute line format: {}", line));
            }
        }
    }

    log_debug(&format!("Parsed {} SMART attributes", attributes.len()));
    attributes
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_usb_device_details(physical_path: &str) -> Result<UsbDeviceDetails, String> {
    log_debug(&format!("Fetching USB device details for physical path: {}", physical_path));

    let mut enumerator = Enumerator::new().map_err(|e| {
        log_error(&format!("Failed to create udev enumerator: {}", e));
        format!("Failed to create udev enumerator: {}", e)
    })?;

    enumerator.match_subsystem("block").map_err(|e| {
        log_error(&format!("Failed to match subsystem: {}", e));
        format!("Failed to match subsystem: {}", e)
    })?;

    // Vyhledáme zařízení, jehož ID_PATH odpovídá zadanému fyzickému identifikátoru
    for device in enumerator.scan_devices().map_err(|e| {
        log_error(&format!("Failed to scan devices: {}", e));
        format!("Failed to scan devices: {}", e)
    })? {
        if let Some(id_path) = device.property_value("ID_PATH").and_then(|v| v.to_str()) {
            if id_path == physical_path {
                // Pokud nalezneme, zkontrolujeme devnode
                if let Some(_devnode) = device.devnode() {
                    let mut details = UsbDeviceDetails {
                        current_tags: String::new(),
                        devlinks: String::new(),
                        devname: String::new(),
                        devpath: String::new(),
                        devtype: String::new(),
                        diskseq: String::new(),
                        id_bus: String::new(),
                        id_fs_boot_system_id: String::new(),
                        id_fs_data_preparer_id: String::new(),
                        id_fs_label: String::new(),
                        id_fs_label_enc: String::new(),
                        id_fs_type: String::new(),
                        id_fs_usage: String::new(),
                        id_fs_uuid: String::new(),
                        id_fs_uuid_enc: String::new(),
                        id_fs_version: String::new(),
                        id_instance: String::new(),
                        id_model: String::new(),
                        id_model_enc: String::new(),
                        id_model_id: String::new(),
                        id_part_table_type: String::new(),
                        id_part_table_uuid: String::new(),
                        id_path: String::new(),
                        id_path_tag: String::new(),
                        id_revision: String::new(),
                        id_serial: String::new(),
                        id_serial_short: String::new(),
                        id_type: String::new(),
                        id_usb_driver: String::new(),
                        id_usb_interfaces: String::new(),
                        id_usb_interface_num: String::new(),
                        id_vendor: String::new(),
                        id_vendor_enc: String::new(),
                        id_vendor_id: String::new(),
                        major: String::new(),
                        minor: String::new(),
                        subsystem: String::new(),
                        tags: String::new(),
                        usec_initialized: String::new(),
                    };

                    for property in device.properties() {
                        match property.name().to_string_lossy().as_ref() {
                            "CURRENT_TAGS" => {
                                details.current_tags = property.value().to_string_lossy().to_string()
                            }
                            "DEVLINKS" => {
                                details.devlinks = property.value().to_string_lossy().to_string()
                            }
                            "DEVNAME" => {
                                details.devname = property.value().to_string_lossy().to_string()
                            }
                            "DEVPATH" => {
                                details.devpath = property.value().to_string_lossy().to_string()
                            }
                            "DEVTYPE" => {
                                details.devtype = property.value().to_string_lossy().to_string()
                            }
                            "DISKSEQ" => {
                                details.diskseq = property.value().to_string_lossy().to_string()
                            }
                            "ID_BUS" => details.id_bus = property.value().to_string_lossy().to_string(),
                            "ID_FS_BOOT_SYSTEM_ID" => {
                                details.id_fs_boot_system_id =
                                    property.value().to_string_lossy().to_string()
                            }
                            "ID_FS_DATA_PREPARER_ID" => {
                                details.id_fs_data_preparer_id =
                                    property.value().to_string_lossy().to_string()
                            }
                            "ID_FS_LABEL" => {
                                details.id_fs_label = property.value().to_string_lossy().to_string()
                            }
                            "ID_FS_LABEL_ENC" => {
                                details.id_fs_label_enc = property.value().to_string_lossy().to_string()
                            }
                            "ID_FS_TYPE" => {
                                details.id_fs_type = property.value().to_string_lossy().to_string()
                            }
                            "ID_FS_USAGE" => {
                                details.id_fs_usage = property.value().to_string_lossy().to_string()
                            }
                            "ID_FS_UUID" => {
                                details.id_fs_uuid = property.value().to_string_lossy().to_string()
                            }
                            "ID_FS_UUID_ENC" => {
                                details.id_fs_uuid_enc = property.value().to_string_lossy().to_string()
                            }
                            "ID_FS_VERSION" => {
                                details.id_fs_version = property.value().to_string_lossy().to_string()
                            }
                            "ID_INSTANCE" => {
                                details.id_instance = property.value().to_string_lossy().to_string()
                            }
                            "ID_MODEL" => {
                                details.id_model = property.value().to_string_lossy().to_string()
                            }
                            "ID_MODEL_ENC" => {
                                details.id_model_enc = property.value().to_string_lossy().to_string()
                            }
                            "ID_MODEL_ID" => {
                                details.id_model_id = property.value().to_string_lossy().to_string()
                            }
                            "ID_PART_TABLE_TYPE" => {
                                details.id_part_table_type =
                                    property.value().to_string_lossy().to_string()
                            }
                            "ID_PART_TABLE_UUID" => {
                                details.id_part_table_uuid =
                                    property.value().to_string_lossy().to_string()
                            }
                            "ID_PATH" => {
                                details.id_path = property.value().to_string_lossy().to_string()
                            }
                            "ID_PATH_TAG" => {
                                details.id_path_tag = property.value().to_string_lossy().to_string()
                            }
                            "ID_REVISION" => {
                                details.id_revision = property.value().to_string_lossy().to_string()
                            }
                            "ID_SERIAL" => {
                                details.id_serial = property.value().to_string_lossy().to_string()
                            }
                            "ID_SERIAL_SHORT" => {
                                details.id_serial_short = property.value().to_string_lossy().to_string()
                            }
                            "ID_TYPE" => {
                                details.id_type = property.value().to_string_lossy().to_string()
                            }
                            "ID_USB_DRIVER" => {
                                details.id_usb_driver = property.value().to_string_lossy().to_string()
                            }
                            "ID_USB_INTERFACES" => {
                                details.id_usb_interfaces =
                                    property.value().to_string_lossy().to_string()
                            }
                            "ID_USB_INTERFACE_NUM" => {
                                details.id_usb_interface_num =
                                    property.value().to_string_lossy().to_string()
                            }
                            "ID_VENDOR" => {
                                details.id_vendor = property.value().to_string_lossy().to_string()
                            }
                            "ID_VENDOR_ENC" => {
                                details.id_vendor_enc = property.value().to_string_lossy().to_string()
                            }
                            "ID_VENDOR_ID" => {
                                details.id_vendor_id = property.value().to_string_lossy().to_string()
                            }
                            "MAJOR" => details.major = property.value().to_string_lossy().to_string(),
                            "MINOR" => details.minor = property.value().to_string_lossy().to_string(),
                            "SUBSYSTEM" => {
                                details.subsystem = property.value().to_string_lossy().to_string()
                            }
                            "TAGS" => details.tags = property.value().to_string_lossy().to_string(),
                            "USEC_INITIALIZED" => {
                                details.usec_initialized =
                                    property.value().to_string_lossy().to_string()
                            }
                            _ => {}
                        }
                    }
                    return Ok(details);
                }
            }
        }
    }

    let err_msg = format!("Device with physical path {} not found", physical_path);
    log_error(&err_msg);
    Err(err_msg)
}
