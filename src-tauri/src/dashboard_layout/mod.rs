use crate::logger::{log_debug, log_error, log_warn};
use once_cell::sync::Lazy;
use serde::Serialize;
use std::collections::HashMap;
use std::process::Command;
use std::str;
use std::sync::{Arc, Mutex as StdMutex};
use std::thread;
use std::time::Duration;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use tauri::command;
use udev::Device;

// Konstantní cesta pro mount root – tento adresář vytvořte jednou jako root a změňte vlastníka (např. na "master")
const MOUNT_ROOT: &str = "/mnt/CRATEC";

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct SataDevice {
    pub interface: String,
    pub serial: Option<String>,
    pub name: Option<String>,
    pub sector_count: Option<u64>,
    pub sector_size: Option<u64>,
    pub side: Option<String>,
    pub mountpoint: Option<String>, // nový údaj pro mountpoint
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct UsbDevice {
    pub interface: String,
    pub serial: Option<String>,
    pub name: Option<String>,
    pub sector_count: Option<u64>,
    pub sector_size: Option<u64>,
    pub side: Option<String>,
    pub mountpoint: Option<String>, // přidaný mountpoint pro USB zařízení
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct DeviceStatus {
    pub usb_devices: Vec<UsbDevice>,
    pub sata_devices: Vec<SataDevice>,
    pub cpu_usage: f32,
    pub ram_usage: f32,
}

#[derive(Serialize)]
#[serde(tag = "type", content = "data")]
pub enum DeviceUpdate {
    Full(DeviceStatus),
}

// Funkce pro kontrolu, zda je zařízení USB (fallback)
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

static SYSTEM_INFO: Lazy<Arc<StdMutex<System>>> = Lazy::new(|| {
    let system = Arc::new(StdMutex::new(System::new_with_specifics(
        RefreshKind::everything()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    )));
    let system_clone = Arc::clone(&system);
    thread::spawn(move || loop {
        {
            let mut sys = system_clone.lock().unwrap();
            sys.refresh_cpu_all();
            sys.refresh_memory();
        }
        thread::sleep(Duration::from_secs(1));
    });
    system
});

fn get_disk_sector_info(devnode: &str) -> Option<(u64, u64)> {
    let dev_name = devnode.trim_start_matches("/dev/");
    let size_path = format!("/sys/class/block/{}/size", dev_name);
    let sector_size_path = format!("/sys/class/block/{}/queue/hw_sector_size", dev_name);

    let sector_count = std::fs::read_to_string(size_path)
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()?;
    let sector_size = std::fs::read_to_string(sector_size_path)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(512);

    Some((sector_count, sector_size))
}

// Funkce pro získání seznamu disků pomocí smartctl
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
            log_warn(&format!(
                "Unexpected line format in smartctl output: {}",
                line
            ));
        }
    }
    Ok(disks)
}

/// Vrací filesystem type oddílu pomocí příkazu blkid.
fn get_fs_type(device: &str) -> Option<String> {
    let output = Command::new("sudo")
        .arg("blkid")
        .arg("-o")
        .arg("value")
        .arg("-s")
        .arg("TYPE")
        .arg(device)
        .output()
        .ok()?;

    if output.status.success() {
        let fs = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if fs.is_empty() {
            None
        } else {
            Some(fs)
        }
    } else {
        None
    }
}

/// Pro dané zařízení (např. /dev/sdb) projde kandidáty na oddíly (/dev/sdb1, /dev/sdb2, …)
/// a vrátí ten oddíl, který má mountovatelný filesystem a je největší.
fn choose_partition(device: &str) -> Option<String> {
    let allowed_fs = [
        "ext2", "ext3", "ext4", "btrfs", "xfs", "vfat", "ntfs", "exfat",
    ];
    let mut best_partition: Option<(String, u64)> = None;
    // Kontrolujeme oddíly 1 až 16; lze upravit podle potřeby.
    for i in 1..=16 {
        let partition = format!("{}{}", device, i);
        if !std::path::Path::new(&partition).exists() {
            continue;
        }
        if let Some(fs_type) = get_fs_type(&partition) {
            if allowed_fs.contains(&fs_type.as_str()) {
                if let Some((sectors, _)) = get_disk_sector_info(&partition) {
                    if best_partition.is_none() || sectors > best_partition.as_ref().unwrap().1 {
                        best_partition = Some((partition.clone(), sectors));
                    }
                }
            }
        }
    }
    best_partition.map(|(p, _)| p)
}

/// Funkce, která zkontroluje, zda je zařízení již připojeno, a pokud ne, pokusí se jej připojit.
/// Pokud disk obsahuje oddíly, vybere se ten, který je největší a má podporovaný filesystem.
/// Vrací mountpoint, pokud se připojení podaří, nebo None.
fn auto_mount(device: &str) -> Option<String> {
    // Nejprve vybereme nejlepší oddíl (pokud existuje), abychom kontrolovali reálně připojovaný devnode (např. /dev/sdb1 místo /dev/sdb).
    let device_to_mount = choose_partition(device).unwrap_or_else(|| device.to_string());

    // Zkontrolujeme, zda už není device_to_mount namountovaný.
    if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
        for line in mounts.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[0] == device_to_mount {
                log_debug(&format!(
                    "Zařízení {} je již namountované na {}. Mount se neprovádí.",
                    device_to_mount,
                    parts[1]
                ));
                return Some(parts[1].to_string());
            }
        }
    }

    // Vygenerujeme mountpoint a případně se pokusíme zařízení připojit.
    let mountpoint = format!(
        "{}/{}",
        MOUNT_ROOT,
        device_to_mount.trim_start_matches("/dev/")
    );

    if let Err(e) = std::fs::create_dir_all(&mountpoint) {
        log_error(&format!(
            "Nepodařilo se vytvořit adresář {}: {}",
            mountpoint, e
        ));
        return None;
    }

    log_debug(&format!(
        "Spouštím příkaz: sudo mount -o rw {} {}",
        device_to_mount, mountpoint
    ));

    let output = Command::new("sudo")
        .arg("mount")
        .arg("-o")
        .arg("rw")
        .arg(&device_to_mount)
        .arg(&mountpoint)
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                log_debug(&format!(
                    "Zařízení {} bylo připojeno na {}",
                    device_to_mount, mountpoint
                ));
                Some(mountpoint)
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                if stderr.contains("already exclusively opened") {
                    // Zkusíme najít device_to_mount v /proc/mounts znovu
                    if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
                        for line in mounts.lines() {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 && parts[0] == device_to_mount {
                                return Some(parts[1].to_string());
                            }
                        }
                    }
                }
                log_error(&format!(
                    "Nepodařilo se připojit {}: {:?}",
                    device_to_mount, out
                ));
                None
            }
        }
        Err(e) => {
            log_error(&format!(
                "Chyba při spuštění příkazu mount pro {}: {}",
                device_to_mount, e
            ));
            None
        }
    }
}

#[command]
pub fn get_device_status() -> Result<DeviceStatus, String> {
    // Získání DB připojení pomocí async mutexu
    let conn_guard = match crate::db::DB_CONN.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            log_error(&format!("Failed to acquire DB lock: {}", e));
            return Err(format!("Failed to acquire DB lock: {}", e));
        }
    };

    fn normalize_interface_path(path: &str) -> String {
        let path = path.replace("usbv3", "usb");
        if path.ends_with(".0") {
            path.trim_end_matches(".0").to_string()
        } else {
            path
        }
    }
    
    let interface_map: HashMap<String, (String, String)> = {
        let mut stmt = conn_guard
            .prepare("SELECT interface_path, name, side FROM interfaces")
            .map_err(|e| {
                log_error(&format!("Failed to prepare statement: {}", e));
                format!("Failed to prepare statement: {}", e)
            })?;

        let rows = stmt
            .query_map([], |row| {
                let path: String = row.get(0)?;
                let name: String = row.get(1)?;
                let side: String = row.get(2)?;
                Ok((path, name, side))
            })
            .map_err(|e| {
                log_error(&format!("Failed to query allowed interfaces: {}", e));
                format!("Failed to query allowed interfaces: {}", e)
            })?;

        let mut map = HashMap::new();
        for row in rows {
            if let Ok((path, name, side)) = row {
                map.insert(normalize_interface_path(&path), (name, side));
            }
        }
        map
    };

    // Systémové informace – CPU a RAM.
    let system = SYSTEM_INFO.lock().map_err(|e| {
        log_error(&format!("Failed to acquire system lock: {}", e));
        format!("Failed to acquire system lock: {}", e)
    })?;
    let cpu_usage = system.global_cpu_usage();
    let used_memory = system.used_memory();
    let total_memory = system.total_memory();
    let ram_usage = if total_memory > 0 {
        (used_memory as f32 / total_memory as f32) * 100.0
    } else {
        0.0
    };

    // Připrava enumerátoru pro bloková zařízení.
    let mut enumerator = udev::Enumerator::new().map_err(|e| {
        log_error(&format!("Failed to create udev enumerator: {}", e));
        format!("Failed to create udev enumerator: {}", e)
    })?;
    enumerator.match_subsystem("block").map_err(|e| {
        log_error(&format!("Failed to match block subsystem: {}", e));
        format!("Failed to match block subsystem: {}", e)
    })?;

    let mut usb_devices = Vec::new();
    // Použijeme HashSet pro sledování serialů, abychom deduplikovali zařízení.
    let mut seen_serials: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Projdeme všechna bloková zařízení a detekujeme USB zařízení.
    for device in enumerator.scan_devices().map_err(|e| {
        log_error(&format!("Failed to scan block devices: {}", e));
        format!("Failed to scan block devices: {}", e)
    })? {
        let devtype = device
            .property_value("DEVTYPE")
            .and_then(|v| v.to_str())
            .unwrap_or("");
        if devtype != "disk" {
            continue;
        }
        // Získáme fyzický interface pomocí vlastnosti "ID_PATH".
        let interface_id =
            if let Some(id) = device.property_value("ID_PATH").and_then(|v| v.to_str()) {
                id.to_string()
            } else {
                continue;
            };

        // Zařízení zahrneme pouze, pokud je jeho fyzický interface v interface_map.
        if !interface_map.contains_key(&interface_id) {
            continue;
        }

        // Získáme unikátní identifikátor (serial) – preferujeme "ID_SERIAL_SHORT", případně "ID_SERIAL".
        let serial = device
            .property_value("ID_SERIAL_SHORT")
            .or_else(|| device.property_value("ID_SERIAL"))
            .and_then(|v| v.to_str())
            .map(|s| s.to_string());

        // Pokud je serial známý a již jsme ho viděli, přeskočíme zařízení.
        if let Some(ref s) = serial {
            if seen_serials.contains(s) {
                continue;
            } else {
                seen_serials.insert(s.clone());
            }
        }

        let id_bus = device
            .property_value("ID_BUS")
            .and_then(|v| v.to_str())
            .unwrap_or("");
        if id_bus == "usb" || is_usb_mass_storage(&device) {
            if let Some(devnode_str) = device.devnode().map(|d| d.to_string_lossy().to_string()) {
                let (sector_count, sector_size) =
                    get_disk_sector_info(&devnode_str).unwrap_or((0, 512));
                // Získáme hodnotu "side" z databáze.
                let side_val = interface_map
                    .get(&interface_id)
                    .map(|(_, side)| side.clone());
                // Pokud je "side" rovno "output", pokusíme se zařízení automaticky připojit.
                let mountpoint = if let Some(ref side) = side_val {
                    if side == "output" {
                        auto_mount(&devnode_str)
                    } else {
                        None
                    }
                } else {
                    None
                };

                usb_devices.push(UsbDevice {
                    interface: interface_id.clone(),
                    serial: serial.clone(),
                    name: interface_map
                        .get(&interface_id)
                        .map(|(name, _)| name.clone()),
                    sector_count: Some(sector_count),
                    sector_size: Some(sector_size),
                    side: side_val,
                    mountpoint, // připojení USB zařízení
                });
            } else {
                log_warn(&format!(
                    "USB zařízení s interface {} nemá dostupný devnode, přeskočeno.",
                    interface_id
                ));
            }
        }
    }

    // Pomocná funkce: získá ID_PATH a serial pro zařízení podle jeho devnode.
    fn get_id_info_for_devnode(devnode: &str) -> Option<(String, Option<String>)> {
        let mut enumerator = udev::Enumerator::new().ok()?;
        enumerator.match_subsystem("block").ok()?;
        for device in enumerator.scan_devices().ok()? {
            if let Some(node) = device.devnode() {
                if node.to_string_lossy() == devnode {
                    let id_path = device
                        .property_value("ID_PATH")
                        .and_then(|v| v.to_str())
                        .map(|s| s.to_string())?;
                    let serial = device
                        .property_value("ID_SERIAL_SHORT")
                        .or_else(|| device.property_value("ID_SERIAL"))
                        .and_then(|v| v.to_str())
                        .map(|s| s.to_string());
                    return Some((id_path, serial));
                }
            }
        }
        None
    }

    // Pro SATA zařízení využijeme smartctl, které vrací devnode (např. "/dev/sda").
    let smartctl_disks = get_smartctl_disks()?;
    let sata_devices: Vec<SataDevice> = smartctl_disks
        .into_iter()
        .filter(|disk| disk.starts_with("/dev/sd"))
        .filter_map(|disk| {
            if let Some((id_path, serial)) = get_id_info_for_devnode(&disk) {
                // Normalizace ID_PATH pro konzistenci.
                let normalized_id_path = normalize_interface_path(&id_path);
                if interface_map.contains_key(&normalized_id_path) {
                    // Pokud je serial znám a už jsme jej viděli jako USB, přeskočíme.
                    if let Some(ref s) = serial {
                        if seen_serials.contains(s) {
                            return None;
                        }
                    }
                    // Zvolíme nejvhodnější oddíl – pokud existuje, jinak celý disk.
                    let disk_to_mount = choose_partition(&disk).unwrap_or_else(|| disk.clone());
                    // Získání sektorových informací z vybraného oddílu nebo disku.
                    let (sector_count, sector_size) =
                        get_disk_sector_info(&disk_to_mount).unwrap_or((0, 512));
                    // Použijeme normalizovanou hodnotu při získávání "side" z databáze.
                    let side_val = interface_map
                        .get(&normalized_id_path)
                        .map(|(_name, side)| side.clone());
                    // Pokud je "side" rovno "output", pokusíme se disk automaticky připojit.
                    let mountpoint = if let Some(ref side) = side_val {
                        if side == "output" {
                            auto_mount(&disk_to_mount)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    Some(SataDevice {
                        interface: normalized_id_path.clone(),
                        serial,
                        name: interface_map.get(&normalized_id_path).map(|(name, _)| name.clone()),
                        sector_count: Some(sector_count),
                        sector_size: Some(sector_size),
                        side: side_val,
                        mountpoint,
                    })                    
                } else {
                    None
                }
            } else {
                log_debug(&format!("ID_PATH pro disk {} nebyl nalezen.", disk));
                None
            }
        })
        .collect();
    Ok(DeviceStatus {
        usb_devices,
        sata_devices,
        cpu_usage,
        ram_usage,
    })
}
