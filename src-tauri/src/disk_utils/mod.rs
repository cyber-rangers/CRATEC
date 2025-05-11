use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::str;
use std::thread;
use std::time::Duration;

#[tauri::command(rename_all = "snake_case")]
pub fn get_lsblk_json(device: &str) -> Result<Value, String> {
    println!("[DEBUG] Spouštím lsblk pro device: {}", device);
    let output = Command::new("sudo")
        .arg("lsblk")
        .arg("-J")
        .arg("-O")
        .arg("-b")
        .arg(device)
        .output()
        .map_err(|e| format!("Failed to run lsblk: {}", e))?;

    let json: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse lsblk JSON: {}", e))?;
    Ok(json)
}

/// Získá logickou velikost bloku ("log-sec") z lsblk JSON.
pub fn get_block_size(device: &str) -> Result<u64, String> {
    let lsblk_data = get_lsblk_json(device)?;
    let devices = lsblk_data["blockdevices"]
        .as_array()
        .ok_or("No blockdevices array found")?;
    if devices.is_empty() {
        return Err("No device info found in lsblk output".to_string());
    }
    devices[0]["log-sec"]
        .as_u64()
        .ok_or("Failed to parse logical block size".to_string())
}

/// Získá celkový počet bloků (počítaných podle block size) z lsblk JSON.
/// (Celková kapacita v bajtech dělená logickou velikostí bloku.)
pub fn get_total_blocks(device: &str) -> Result<u64, String> {
    let lsblk_data = get_lsblk_json(device)?;
    let devices = lsblk_data["blockdevices"]
        .as_array()
        .ok_or("No blockdevices array found")?;
    if devices.is_empty() {
        return Err("No device info found in lsblk output".to_string());
    }
    let capacity_bytes = devices[0]["size"]
        .as_u64()
        .ok_or("Failed to parse disk size")?;
    let block_size = get_block_size(device)?;
    Ok(capacity_bytes / block_size)
}

/// Vrátí mountpoint hlavního zařízení nebo některého z jeho oddílů z lsblk JSON (pokud existuje).
pub fn get_mountpoint_for_interface(device: &str) -> Option<String> {
    println!("[DEBUG] Hledám mountpoint pro device: {}", device);
    let lsblk_data = match get_lsblk_json(device) {
        Ok(data) => data,
        Err(e) => {
            println!("[DEBUG] Chyba při získávání lsblk JSON: {}", e);
            return None;
        }
    };
    let devices = lsblk_data["blockdevices"].as_array()?;
    if devices.is_empty() {
        println!("[DEBUG] Pole blockdevices je prázdné.");
        return None;
    }
    // Zkusíme zjistit mountpoint přímo u hlavního zařízení
    if let Some(mp) = devices[0]["mountpoint"].as_str() {
        println!("[DEBUG] Mountpoint na hlavním zařízení: {}", mp);
        if !mp.is_empty() {
            return Some(mp.to_string());
        }
    } else {
        println!("[DEBUG] Pole mountpoint na hlavním zařízení není k dispozici.");
    }
    // Pokud není, projdeme oddíly ("children")
    if let Some(children) = devices[0]["children"].as_array() {
        println!(
            "[DEBUG] Nalezeno {} oddílů, prohledávám je...",
            children.len()
        );
        for child in children {
            if let Some(mp) = child["mountpoint"].as_str() {
                println!("[DEBUG] Oddíl má mountpoint: {}", mp);
                if !mp.is_empty() {
                    return Some(mp.to_string());
                }
            }
            if let Some(mps) = child["mountpoints"].as_array() {
                println!("[DEBUG] Oddíl obsahuje mountpoints: {:?}", mps);
                for m in mps {
                    if let Some(mp) = m.as_str() {
                        println!("[DEBUG] nalezený mountpoint v mountpoints: {}", mp);
                        if !mp.is_empty() {
                            return Some(mp.to_string());
                        }
                    }
                }
            }
        }
    } else {
        println!("[DEBUG] Pole children není k dispozici.");
    }
    println!("[DEBUG] Nebyl nalezen žádný mountpoint.");
    None
}

/// Informace o oddílu.
#[derive(Debug, Serialize)]
pub struct PartitionInfo {
    pub index: usize,
    pub start_sector: u64,
    pub end_sector: u64,
    pub filesystem: Option<String>,
    pub fssize: Option<u64>,
    pub fsused: Option<u64>,
    pub fsuse_percent: Option<String>,
    pub uuid: Option<String>,
    pub mountpoint: Option<String>,
}

/// Struktura s informacemi o disku.
#[derive(Debug, Serialize)]
pub struct DiskInfo {
    pub serial: String,
    pub capacity_bytes: u64,
    pub logical_sector_size: u64,
    pub partitions: Vec<PartitionInfo>,
    pub ata_encryption: bool,
    pub sed_encryption: bool,
    pub readable: bool,
    pub has_hpa: bool,
    pub dco: u64,
    pub model: Option<String>,
}

fn detect_ata_encryption(device: &str) -> bool {
    if let Ok(output) = std::process::Command::new("sudo")
        .arg("hdparm")
        .arg("-I")
        .arg(device)
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("Security:") {
            for line in stdout.lines() {
                if line.trim().starts_with("enabled") {
                    return true;
                }
            }
        }
    }
    false
}

/// Detekce HPA a DCO pomocí příkazů hdparm a blockdev.
/// Vrací dvojici: (Option<bool> pro HPA, Option<bool> pro DCO).
/// Pokud výstup obsahuje "missing sense data", "bad sense data" nebo "Real max sectors: 1",
/// vrátí se None (N/A); jinak se porovná Real max sectors s adresovatelnými sektory.
pub fn detect_hpa_dco(device: &str) -> (bool, u64) {
    // Detekce HPA
    let mut has_hpa = false;
    if let Ok(output) = Command::new("sudo")
        .arg("hdparm")
        .arg("-N")
        .arg(device)
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);

        let has_bad_sense = stdout.contains("bad/missing sense data");
        let has_invalid_max = stdout.lines().any(|l| l.trim() == "max sectors   = 0/1, HPA is enabled");

        if !has_bad_sense && !has_invalid_max {
            if stdout.contains("HPA is enabled") {
                has_hpa = true;
            } else if stdout.contains("HPA is disabled") {
                has_hpa = false;
            }
        }
    }

    // Detekce DCO - získání real max sectors
    let mut real_max_sectors = 0u64;
    if let Ok(output) = Command::new("sudo")
        .arg("hdparm")
        .arg("--dco-identify")
        .arg(device)
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.contains("missing sense data")
            && !stdout.contains("bad sense data")
            && !stdout.contains("Real max sectors: 1")
        {
            if let Some(line) = stdout.lines().find(|l| l.contains("Real max sectors:")) {
                if let Some(sectors_str) = line.split(':').nth(1) {
                    let parsed = sectors_str.trim().parse::<u64>().unwrap_or(0);
                    if parsed > 1 {
                        real_max_sectors = parsed;
                    }
                }
            }
        }
    }
    (has_hpa, real_max_sectors)
}

fn detect_encryption_status(device: &str) -> (bool, bool, bool) {
    let mut ata_encryption = false;
    let mut sed_encryption = false;
    let mut readable = true;

    if let Ok(output) = std::process::Command::new("sudo")
        .arg("hdparm")
        .arg("-I")
        .arg(device)
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);

        // ATA encryption detection
        if let Some(security_section) = stdout.split("Security:").nth(1) {
            for line in security_section.lines() {
                let l = line.trim();
                if l.starts_with("enabled") {
                    ata_encryption = true;
                }
                if l.starts_with("locked") {
                    readable = false;
                }
            }
        }

        // SED detection (search for 'encrypted' or 'encryption' in the whole output)
        if stdout.contains("encrypted") || stdout.contains("encryption") {
            sed_encryption = true;
        }
    }

    (ata_encryption, sed_encryption, readable)
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_disk_info(device: &str) -> Result<DiskInfo, String> {
    let lsblk_data = get_lsblk_json(device)?;
    let devices = lsblk_data["blockdevices"]
        .as_array()
        .ok_or("No blockdevices array found")?;
    if devices.is_empty() {
        return Err("No disk info found in lsblk output".to_string());
    }
    let disk = &devices[0];

    let serial = disk["serial"]
        .as_str()
        .unwrap_or("UnknownSerial")
        .to_string();

    let capacity_bytes = disk["size"]
        .as_u64()
        .ok_or("Failed to parse disk size")?;

    let logical_sector_size = disk["log-sec"]
        .as_u64()
        .ok_or("Failed to parse logical sector size")?;

    let model = disk["model"].as_str().map(|s| s.to_string());

    let mut partitions = Vec::new();
    if let Some(children) = disk["children"].as_array() {
        for child in children {
            let part_size = child["size"].as_u64().unwrap_or(0);
            let start_sector = child["start"].as_u64().unwrap_or(0);
            let index = child["partn"].as_u64().unwrap_or(0) as usize;
            let filesystem = child["fstype"]
                .as_str()
                .map(|s| s.to_string())
                .or(Some("unknown".to_string()));
            let end_sector = if part_size > 0 {
                start_sector + (part_size / 512) - 1
            } else {
                start_sector
            };

            let fssize = child["fssize"].as_u64();
            let fsused = child["fsused"].as_u64();
            let fsuse_percent = child["fsuse%"].as_str().map(|s| s.to_string());
            let uuid = child["uuid"].as_str().map(|s| s.to_string());
            let mountpoint = child["mountpoint"].as_str().map(|s| s.to_string());

            partitions.push(PartitionInfo {
                index,
                start_sector,
                end_sector,
                filesystem,
                fssize,
                fsused,
                fsuse_percent,
                uuid,
                mountpoint,
            });
        }
    }

    let (ata_encryption, sed_encryption, readable) = detect_encryption_status(device);
    let (has_hpa, dco) = detect_hpa_dco(device);

    Ok(DiskInfo {
        serial,
        capacity_bytes,
        logical_sector_size,
        partitions,
        ata_encryption,
        sed_encryption,
        readable,
        has_hpa,
        dco,
        model,
    })
}
