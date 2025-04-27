use serde::Serialize;
use std::process::Command;
use rusqlite::{Connection, Result as SqlResult};
use crate::db::DB_POOL; // přidej tento import

// Načti build date vygenerované build.rs
include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

#[derive(Serialize)]
pub struct ProgramVersions {
    pub ewfacquire: String,
    pub dcfldd: String,
    pub build_date: String,
}

#[derive(Serialize)]
pub struct SystemLog {
    pub id: i64,
    pub level: String,
    pub message: String,
    pub timestamp: String,
}

#[tauri::command]
pub fn get_program_versions() -> Result<ProgramVersions, String> {
    // ewfacquire -V
    let ewf_output = Command::new("ewfacquire")
        .arg("-V")
        .output()
        .map_err(|e| format!("Failed to run ewfacquire: {}", e))?;
    let ewf_version = String::from_utf8_lossy(&ewf_output.stdout)
        .lines()
        .next()
        .unwrap_or("unknown")
        .to_string();

    // dcfldd --version
    let dcfldd_output = Command::new("dcfldd")
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to run dcfldd: {}", e))?;
    let dcfldd_version = String::from_utf8_lossy(&dcfldd_output.stdout)
        .lines()
        .next()
        .unwrap_or("unknown")
        .to_string();

    Ok(ProgramVersions {
        ewfacquire: ewf_version,
        dcfldd: dcfldd_version,
        build_date: BUILD_DATE.to_string(),
    })
}

#[tauri::command]
pub fn get_system_logs() -> Result<Vec<SystemLog>, String> {
    // Použij connection pool
    let mut pooled = DB_POOL.get_connection().map_err(|e| format!("DB pool error: {}", e))?;
    let conn = pooled.connection();

    let mut stmt = conn
        .prepare("SELECT id, level, message, timestamp FROM logs ORDER BY id DESC")
        .map_err(|e| format!("DB prepare error: {}", e))?;

    let logs_iter = stmt
        .query_map([], |row| {
            Ok(SystemLog {
                id: row.get(0)?,
                level: row.get(1)?,
                message: row.get(2)?,
                timestamp: row.get(3)?,
            })
        })
        .map_err(|e| format!("DB query error: {}", e))?;

    let mut logs = Vec::new();
    for log in logs_iter {
        logs.push(log.map_err(|e| format!("DB row error: {}", e))?);
    }
    Ok(logs)
}