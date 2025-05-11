use serde::Serialize;
use std::process::Command;
use rusqlite::{Connection, Result as SqlResult};
use crate::db::DB_POOL; 
use std::fs;
use hex;                                
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;                     
use std::fs::File;
use std::io::Read;


include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

#[derive(Serialize)]
pub struct ProgramVersions {
    pub ewfacquire: String,
    pub dcfldd: String,
    pub build_date: String,
    pub short_hw_id: String,        
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

    // načteme krátké HW UUID
    let short_hw_id = get_short_hw_id().map_err(|e| format!("Failed to get HW ID: {}", e))?;

    Ok(ProgramVersions {
        ewfacquire: ewf_version,
        dcfldd: dcfldd_version,
        build_date: BUILD_DATE.to_string(),
        short_hw_id,                 // vyplníme nové pole
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

#[tauri::command]
pub fn get_short_hw_id() -> Result<String, String> {
    let out = Command::new("sudo")
        .args(&["cat", "/sys/class/dmi/id/product_uuid"])
        .output()
        .map_err(|e| format!("Failed to run sudo cat: {}", e))?;
    if !out.status.success() {
        return Err(format!(
            "sudo cat failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let uuid_raw = String::from_utf8_lossy(&out.stdout).to_string();

    let uuid = uuid_raw.trim().replace('-', "");

    let bytes = hex::decode(&uuid).map_err(|e| format!("Invalid hex in UUID: {}", e))?;

    let short = STANDARD_NO_PAD.encode(&bytes)[..22].to_string();

    Ok(short)
}


#[tauri::command]
pub fn get_cratec_md5() -> Result<String, String> {
    let mut file = File::open("/usr/local/bin/cratec")
        .map_err(|e| format!("open /usr/local/bin/cratec: {e}"))?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|e| format!("read cratec: {e}"))?;

    let digest = md5::compute(buf);
    Ok(format!("{:x}", digest))
}


#[derive(Serialize)]
pub struct ReportSystemInfo {
    pub build_date: String,
    pub cratec_hash: String,
    pub short_hw_id: String,
}

#[tauri::command]
pub fn get_report_system_info() -> Result<ReportSystemInfo, String> {
    Ok(ReportSystemInfo {
        build_date: BUILD_DATE.to_string(),
        cratec_hash: get_cratec_md5()?,
        short_hw_id: get_short_hw_id()?,
    })
}