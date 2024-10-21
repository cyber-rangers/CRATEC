use crate::logger::{log_debug, log_error, log_warn};
use once_cell::sync::Lazy;
use serde::Serialize;
use std::process::Command;
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use sysinfo::{CpuExt, CpuRefreshKind, RefreshKind, System, SystemExt};
use tauri::command;
use udev::{Device, Enumerator};

#[derive(Serialize, Debug)]
pub struct SataDevice {
    pub interface: String,
}

#[derive(Serialize, Debug)]
pub struct UsbDevice {
    pub interface: String,
}

#[derive(Serialize, Debug)]
pub struct DeviceStatus {
    pub usb_devices: Vec<UsbDevice>,
    pub sata_devices: Vec<SataDevice>,
    pub cpu_usage: f32,
    pub ram_usage: f32,
}

// Funkce pro kontrolu, zda je zařízení USB Mass Storage
fn is_usb_mass_storage(dev: &Device) -> bool {
    let mut device = dev.clone();
    loop {
        if let Some(subsystem) = device.subsystem() {
            if subsystem == "usb" {
                return true;
            }
        }
        if let Some(parent) = device.parent() {
            device = parent.clone();
        } else {
            break;
        }
    }
    false
}

// Globální proměnné pro sdílení informací mezi vlákny
static SYSTEM_INFO: Lazy<Arc<Mutex<System>>> = Lazy::new(|| {
    let system = Arc::new(Mutex::new(System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::new().with_cpu_usage())
            .with_memory(),
    )));

    let system_clone = Arc::clone(&system);

    // Spuštění samostatného vlákna pro aktualizaci CPU a RAM informací
    thread::spawn(move || {
        loop {
            {
                let mut sys = system_clone.lock().unwrap();
                sys.refresh_cpu();
                sys.refresh_memory();
            }
            thread::sleep(Duration::from_secs(1)); // Aktualizace každou sekundu
        }
    });

    system
});

#[command]
pub fn get_device_status() -> Result<DeviceStatus, String> {
    // Získání zamku pro přístup k systému
    let system = match SYSTEM_INFO.lock() {
        Ok(sys) => sys,
        Err(e) => {
            log_error(&format!("Failed to acquire system lock: {}", e));
            return Err(format!("Failed to acquire system lock: {}", e));
        }
    };

    // Získání využití CPU a RAM
    let cpu_usage = system.global_cpu_info().cpu_usage();

    let used_memory = system.used_memory();
    let total_memory = system.total_memory();
    let ram_usage = if total_memory > 0 {
        (used_memory as f32 / total_memory as f32) * 100.0
    } else {
        0.0
    };

    // Enumerace všech block devices
    let mut enumerator = match Enumerator::new() {
        Ok(e) => e,
        Err(e) => {
            log_error(&format!("Failed to create udev enumerator: {}", e));
            return Err(format!("Failed to create udev enumerator: {}", e));
        }
    };
    if let Err(e) = enumerator.match_subsystem("block") {
        log_error(&format!("Failed to match block subsystem: {}", e));
        return Err(format!("Failed to match block subsystem: {}", e));
    }

    let mut usb_devices = Vec::new();

    for device in enumerator.scan_devices().map_err(|e| {
        log_error(&format!("Failed to scan block devices: {}", e));
        format!("Failed to scan block devices: {}", e)
    })? {
        // Filtrace pouze na block devices s DEVTYPE == "disk"
        let devtype = device
            .property_value("DEVTYPE")
            .and_then(|v| v.to_str())
            .unwrap_or("Unknown");

        if devtype != "disk" {
            continue;
        }

        // Zkontrola, zda je block device připojeno k USB Mass Storage
        if !is_usb_mass_storage(&device) {
            continue;
        }

        // Získání devnode (např. /dev/sda)
        if let Some(devnode) = device.devnode() {
            let interface = devnode.to_string_lossy().to_string();
            usb_devices.push(UsbDevice { interface: interface.clone() });
        }
    }

    // Získání seznamu SATA zařízení pomocí cacheování
    let smartctl_disks = match get_smartctl_disks() {
        Ok(disks) => disks,
        Err(e) => {
            log_error(&format!("Failed to get smartctl disks: {}", e));
            return Err(format!("Failed to get smartctl disks: {}", e));
        }
    };

    let sata_devices = smartctl_disks
        .iter()
        .filter(|disk| disk.starts_with("/dev/sd"))
        .map(|disk| SataDevice {
            interface: disk.clone(),
        })
        .collect::<Vec<SataDevice>>();

    Ok(DeviceStatus {
        usb_devices,
        sata_devices,
        cpu_usage,
        ram_usage,
    })
}

fn get_smartctl_disks() -> Result<Vec<String>, String> {
    let output = Command::new("smartctl")
        .arg("--scan")
        .output()
        .map_err(|e| {
            log_error(&format!("Failed to execute smartctl: {}", e));
            format!("Failed to execute smartctl: {}", e)
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log_error(&format!(
            "smartctl exited with status {}: {}",
            output.status.code().unwrap_or(-1),
            stderr
        ));
        return Err(format!(
            "smartctl exited with status {}: {}",
            output.status.code().unwrap_or(-1),
            stderr
        ));
    }

    let stdout = str::from_utf8(&output.stdout).map_err(|e| {
        log_error(&format!("Invalid UTF-8 output from smartctl: {}", e));
        format!("Invalid UTF-8 output from smartctl: {}", e)
    })?;

    let mut disks = Vec::new();
    for line in stdout.lines() {
        if let Some(device_path) = line.split_whitespace().next() {
            disks.push(device_path.to_string());
        } else {
            log_warn(&format!("Unexpected line format in smartctl output: {}", line));
        }
    }

    Ok(disks)
}
