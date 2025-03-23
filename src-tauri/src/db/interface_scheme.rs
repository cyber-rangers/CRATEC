use rusqlite::{Connection, Result};

pub fn initialize_interface_scheme(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS interfaces (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            interface_path TEXT NOT NULL UNIQUE,
            side TEXT NOT NULL CHECK (side IN ('input','output'))
        )",
        [],
    )?;
    Ok(())
}
