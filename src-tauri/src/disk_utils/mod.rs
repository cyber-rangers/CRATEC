use std::fs;
use std::path::Path;
use std::time::Duration;
use std::thread;

/// Najde mountpoint pro zařízení podle zadaného diskového identifikátoru.
/// Používá se v modulech ewfacquire i dcfldd.
pub fn get_mountpoint_for_interface(id_path: &str) -> Option<String> {
    let mounts_str = fs::read_to_string("/proc/mounts").ok()?;
    let mut best_match: Option<(String, usize)> = None;

    for line in mounts_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let device = parts[0];
        let mountpoint = parts[1];

        if let Ok(real_dev) = Path::new(device).canonicalize() {
            if let Ok(entries) = fs::read_dir("/dev/disk/by-path") {
                for entry in entries.flatten() {
                    if let Ok(filename) = entry.file_name().into_string() {
                        let full_link = format!("/dev/disk/by-path/{}", filename);
                        if let Ok(link_target) = fs::read_link(&full_link) {
                            if let Ok(link_canon) = Path::new("/dev/disk/by-path")
                                .join(&link_target)
                                .canonicalize()
                            {
                                if link_canon == real_dev && filename.contains(id_path) {
                                    // V obou modulech se používá číselné pořadí, takže zde necháme partition_number
                                    let partition_number = filename
                                        .split("-part")
                                        .nth(1)
                                        .and_then(|s| s.parse::<usize>().ok())
                                        .unwrap_or(0);
                                    if let Some((_, best_num)) = best_match {
                                        if partition_number > best_num {
                                            best_match =
                                                Some((mountpoint.to_string(), partition_number));
                                        }
                                    } else {
                                        best_match =
                                            Some((mountpoint.to_string(), partition_number));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    best_match.map(|(mp, _)| mp)
}

/// Získá celkový počet bloků pro zadané zařízení při dané velikosti bloku.
/// Používá se v modulu dcfldd.
pub fn get_total_blocks(input_device: &str, block_size: u64) -> Result<u64, String> {
    let canonical = fs::canonicalize(input_device)
        .map_err(|e| format!("Failed to canonicalize {}: {}", input_device, e))?;
    let dev_name = canonical
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Failed to get device basename")?;

    let partitions = fs::read_to_string("/proc/partitions")
        .map_err(|e| format!("Failed to read /proc/partitions: {}", e))?;
    for line in partitions.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("major") {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        if parts[3] == dev_name {
            let blocks_1k = parts[2]
                .parse::<u64>()
                .map_err(|e| format!("Failed to parse block count for {}: {}", dev_name, e))?;
            let total_bytes = blocks_1k * 1024;
            return Ok(total_bytes / block_size);
        }
    }
    Err(format!("Device {} not found in /proc/partitions", dev_name))
}