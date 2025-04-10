use serde::Serialize;
use std::fs;
use std::path::PathBuf;

/// Struktura, kterou vracíme do frontendu.
#[derive(Serialize)]
pub struct FileItem {
    pub name: String,
    pub file_type: String, // "folder" nebo "file"
}

/// Tato funkce projde daný adresář (mountpoint) a vrátí seznam FileItem
#[tauri::command]
pub fn get_directory_contents(mountpoint: &str) -> Result<Vec<FileItem>, String> {
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

        results.push(FileItem { name, file_type });
    }

    Ok(results)
}
