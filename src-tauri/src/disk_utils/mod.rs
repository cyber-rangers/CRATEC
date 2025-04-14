use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::str;
use std::time::Duration;
use std::thread;

/// Spustí příkaz `sudo lsblk -J -O -b <device>` a vrátí výsledný JSON.
fn get_lsblk_json(device: &str) -> Result<Value, String> {
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
    let lsblk_data = get_lsblk_json(device).ok()?;
    let devices = lsblk_data["blockdevices"].as_array()?;
    if devices.is_empty() {
        return None;
    }

    // Zkusíme zjistit mountpoint přímo u hlavního zařízení
    if let Some(mp) = devices[0]["mountpoint"].as_str() {
        if !mp.is_empty() {
            return Some(mp.to_string());
        }
    }
    // Pokud není, projdeme oddíly ("children")
    if let Some(children) = devices[0]["children"].as_array() {
        for child in children {
            if let Some(mp) = child["mountpoint"].as_str() {
                if !mp.is_empty() {
                    return Some(mp.to_string());
                }
            }
            if let Some(mps) = child["mountpoints"].as_array() {
                for m in mps {
                    if let Some(mp) = m.as_str() {
                        if !mp.is_empty() {
                            return Some(mp.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Informace o oddílu.
#[derive(Debug)]
pub struct PartitionInfo {
    pub index: usize,
    pub start_sector: u64,
    pub end_sector: u64,
    pub filesystem: Option<String>,
    pub is_encrypted: bool,
}

/// Struktura s informacemi o disku.
#[derive(Debug)]
pub struct DiskInfo {
    pub serial: String,
    pub capacity_bytes: u64,
    pub partitions: Vec<PartitionInfo>,
    pub disk_encryption: Option<String>,
    pub has_hpa: bool,
    pub has_dco: bool,
}

/// Detekce šifrování pomocí lsblk JSON.
/// Prochází oddíly a pokud u některého najde fstype obsahující indikátor šifrování,
/// vrací ho (např. "BitLocker", "crypto_LUKS", "dm-crypt").
fn detect_encryption(device: &str) -> Option<String> {
    let lsblk_data = get_lsblk_json(device).ok()?;
    let devices = lsblk_data["blockdevices"].as_array()?;
    if devices.is_empty() {
        return None;
    }
    let disk = &devices[0];
    if let Some(children) = disk["children"].as_array() {
        for child in children {
            if let Some(fstype) = child["fstype"].as_str() {
                let fstype_lc = fstype.to_lowercase();
                if fstype_lc.contains("bitlocker") || fstype_lc.contains("luks") || fstype_lc.contains("dm-crypt") {
                    return Some(fstype.to_string());
                }
            }
        }
    }
    None
}

/// Detekce HPA a DCO pomocí příkazů hdparm a blockdev.
/// Vrací dvojici: (Option<bool> pro HPA, Option<bool> pro DCO).
/// Pokud výstup obsahuje "missing sense data", "bad sense data" nebo "Real max sectors: 1",
/// vrátí se None (N/A); jinak se porovná Real max sectors s adresovatelnými sektory.
pub fn detect_hpa_dco(device: &str) -> (Option<bool>, Option<bool>) {
    let mut has_hpa: Option<bool> = None;
    let mut has_dco: Option<bool> = None;

    // Detekce HPA
    if let Ok(output) = Command::new("sudo").arg("hdparm").arg("-N").arg(device).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("missing sense data") || stdout.contains("bad sense data") {
            has_hpa = None;
        } else if stdout.contains("HPA is enabled") {
            has_hpa = Some(true);
        } else if stdout.contains("HPA is disabled") {
            has_hpa = Some(false);
        }
    }

    // Detekce DCO
    if let Ok(output) = Command::new("sudo")
        .arg("hdparm")
        .arg("--dco-identify")
        .arg(device)
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("missing sense data")
            || stdout.contains("bad sense data")
            || stdout.contains("Real max sectors: 1")
        {
            has_dco = None;
        } else if let Some(line) = stdout.lines().find(|l| l.contains("Real max sectors:"))
        {
            if let Some(real_max_sectors_str) = line.split(':').nth(1) {
                let real_max_sectors = real_max_sectors_str.trim().parse::<u64>().unwrap_or(0);
                if let Ok(blockdev_output) = Command::new("sudo")
                    .arg("blockdev")
                    .arg("--getsz")
                    .arg(device)
                    .output()
                {
                    let blockdev_stdout = String::from_utf8_lossy(&blockdev_output.stdout);
                    if let Ok(addressable_sectors) = blockdev_stdout.trim().parse::<u64>() {
                        if real_max_sectors == addressable_sectors {
                            has_dco = Some(false);
                        } else {
                            has_dco = Some(true);
                        }
                    }
                }
            }
        }
    }
    (has_hpa, has_dco)
}

/// Získá rozšířené informace o disku z lsblk JSON a detekci HPA/DCO.
pub fn get_disk_info(device: &str) -> Result<DiskInfo, String> {
    let lsblk_data = get_lsblk_json(device)?;
    let devices = lsblk_data["blockdevices"]
        .as_array()
        .ok_or("No blockdevices array found")?;
    if devices.is_empty() {
        return Err("No disk info found in lsblk output".to_string());
    }
    let disk = &devices[0];

    let serial = disk["serial"].as_str().unwrap_or("UnknownSerial").to_string();
    let capacity_bytes = disk["size"]
        .as_u64()
        .ok_or("Failed to parse disk size")?;

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
            let is_encrypted = filesystem
                .as_ref()
                .map(|fs| {
                    let fs_lc = fs.to_lowercase();
                    fs_lc.contains("bitlocker") || fs_lc.contains("luks") || fs_lc.contains("dm-crypt")
                })
                .unwrap_or(false);
            let end_sector = if part_size > 0 {
                start_sector + (part_size / 512) - 1
            } else {
                start_sector
            };

            partitions.push(PartitionInfo {
                index,
                start_sector,
                end_sector,
                filesystem,
                is_encrypted,
            });
        }
    }
    let disk_encryption = detect_encryption(device);
    let (has_hpa, has_dco) = detect_hpa_dco(device);

    Ok(DiskInfo {
        serial,
        capacity_bytes,
        partitions,
        disk_encryption,
        has_hpa: has_hpa.unwrap_or(false),
        has_dco: has_dco.unwrap_or(false),
    })
}