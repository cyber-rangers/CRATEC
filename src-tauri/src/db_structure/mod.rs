pub mod logging_scheme;
pub mod config_scheme;

use rusqlite::Connection;
use rusqlite::Result;

// Funkce pro inicializaci všech schémat
pub fn initialize_db(conn: &Connection) -> Result<()> {
    logging_scheme::initialize_logging_scheme(conn)?;
    config_scheme::initialize_config_scheme(conn)?;
    Ok(())
}
