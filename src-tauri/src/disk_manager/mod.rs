use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use crate::config::MOUNT_ROOT;


#[derive(Serialize)]
pub struct FileItem {
    pub name: String,
    pub file_type: String, 
    pub size: u64,         
    pub created: Option<u64>, 
}

/// Tato funkce projde daný adresář (mountpoint) a vrátí seznam FileItem
#[tauri::command]
pub fn get_directory_contents(mountpoint: &str) -> Result<Vec<FileItem>, String> {
    // Bezpečnostní kontrola: povoleno pouze v MOUNT_ROOT
    if !mountpoint.starts_with(MOUNT_ROOT) {
        return Err(format!("Access denied: path {} is outside of MOUNT_ROOT", mountpoint));
    }

    let mut results = Vec::new();

    let entries = fs::read_dir(mountpoint)
        .map_err(|e| format!("Failed to read directory {}: {}", mountpoint, e))?;

    for entry_result in entries {
        let entry = entry_result.map_err(|e| e.to_string())?;
        let path: PathBuf = entry.path();
        let metadata = entry.metadata().map_err(|e| e.to_string())?;

        let file_type = if metadata.is_dir() {
            "folder".to_string()
        } else {
            "file".to_string()
        };

        // Název souboru/složky
        let name = match path.file_name() {
            Some(os_str) => os_str.to_string_lossy().to_string(),
            None => String::from("unknown"),
        };

        // Velikost souboru (složka = 0)
        let size = if metadata.is_file() {
            metadata.len()
        } else {
            0
        };

        // Čas vytvoření (pokud je dostupný), jinak čas poslední změny
        let created = metadata.created()
            .or_else(|_| metadata.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        results.push(FileItem { name, file_type, size, created });
    }

    Ok(results)
}
