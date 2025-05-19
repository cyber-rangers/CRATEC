use crate::db::DB_CONN;
use rusqlite::Result;

/// Vloží záznam logu do databáze.
pub async fn log_to_db(level: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    log::debug!("Attempting to log to DB: [{}] {}", level, message);

    // Použijeme asynchronní zámek
    let conn = DB_CONN.lock().await;

    conn.execute(
        "INSERT INTO logs (level, message) VALUES (?1, ?2)",
        rusqlite::params![level, message],
    )?;
    log::debug!("Log successfully written to DB.");
    Ok(())
}
/// Loguje chybové zprávy do databáze.
pub fn log_error(message: &str) {
    let message = message.to_string(); // Zkopírujeme hodnotu do String
    tauri::async_runtime::spawn(async move {
        if let Err(e) = log_to_db("ERROR", &message).await {
            eprintln!("Failed to log ERROR: {}", e);
        }
    });
}

/// Loguje varovné zprávy do databáze.
pub fn log_warn(message: &str) {
    let message = message.to_string(); // Zkopírujeme hodnotu do String
    tauri::async_runtime::spawn(async move {
        if let Err(e) = log_to_db("WARN", &message).await {
            eprintln!("Failed to log WARN: {}", e);
        }
    });
}

/// Loguje debug zprávy do databáze.
pub fn log_debug(message: &str) {
    let message = message.to_string(); // Zkopírujeme hodnotu do String
    tauri::async_runtime::spawn(async move {
        if let Err(e) = log_to_db("DEBUG", &message).await {
            eprintln!("Failed to log DEBUG: {}", e);
        }
    });
}