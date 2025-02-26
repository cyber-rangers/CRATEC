pub mod logging_scheme;
pub mod ewf_config_scheme;
pub mod copy_log_scheme;
pub mod dd_config_scheme;

use rusqlite::Connection;
use std::error::Error;
use std::fs;
use std::path::Path;

/// Inicializace databáze:
/// - Vytvoří adresář /var/lib/cratec, pokud ještě neexistuje.
/// - Otevře (nebo vytvoří) databázi na /var/lib/cratec/database.db.
/// - Vytvoří/ověří tabulky dle schémat definovaných v logging_scheme a config_scheme.
pub fn initialize_db() -> Result<Connection, Box<dyn Error>> {
    let db_path = "/var/lib/cratec/database.db";

    // Zajistíme, že adresář existuje
    if let Some(parent_dir) = Path::new(db_path).parent() {
        fs::create_dir_all(parent_dir)?;
    }

    // Otevření nebo vytvoření databáze
    let conn = Connection::open(db_path)?;

    // Inicializace tabulek podle schémat
    logging_scheme::initialize_logging_scheme(&conn)?;
    ewf_config_scheme::initialize_ewf_config_scheme(&conn)?;
    dd_config_scheme::initialize_dd_config_scheme(&conn)?;
    copy_log_scheme::initialize_copy_log_scheme(&conn)?;

    Ok(conn)
}
