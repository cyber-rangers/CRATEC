use crate::db_structure;

use rusqlite::{params, Connection, Result};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

static DB_CONNECTION: Lazy<Arc<Mutex<Connection>>> = Lazy::new(|| {
    let conn = Connection::open("database.db").expect("Failed to open database");
    Arc::new(Mutex::new(conn))
});

// Funkce pro inicializaci logování včetně struktury databáze
pub fn initialize_logging() -> Result<()> {
    let conn = DB_CONNECTION.lock().unwrap();
    // Zavolejte funkci pro inicializaci struktury databáze z modulu db_structure
    db_structure::initialize_db(&conn)?;
    Ok(())
}

// Funkce pro vložení logu do databáze
pub fn log_to_db(level: &str, message: &str) -> Result<()> {
    let conn = DB_CONNECTION.lock().unwrap();
    conn.execute(
        "INSERT INTO logs (level, message) VALUES (?1, ?2)",
        params![level, message],
    )?;
    Ok(())
}

// Pomocné funkce pro různé úrovně logování
pub fn log_error(message: &str) {
    if let Err(e) = log_to_db("ERROR", message) {
        eprintln!("Failed to log ERROR: {}", e);
    }
}

pub fn log_warn(message: &str) {
    if let Err(e) = log_to_db("WARN", message) {
        eprintln!("Failed to log WARN: {}", e);
    }
}

pub fn log_debug(message: &str) {
    if let Err(e) = log_to_db("DEBUG", message) {
        eprintln!("Failed to log DEBUG: {}", e);
    }
}
