use rusqlite::{Connection, Result};

// Funkce pro inicializaci tabulky konfigurací
pub fn initialize_config_scheme(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS config (
            id INTEGER PRIMARY KEY,
            key TEXT NOT NULL,
            value TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}
