use rusqlite::{Connection, Result};

pub fn initialize_process_log_scheme(conn: &Connection) -> Result<()> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS copy_process (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            start_datetime DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            end_datetime DATETIME,
            status TEXT NOT NULL DEFAULT 'running'
                CHECK(status IN ('running','done','error')),
            triggered_by_ewf INTEGER,
            triggered_by_dd INTEGER,
            FOREIGN KEY(triggered_by_ewf) REFERENCES copy_log_ewf(config_id),
            FOREIGN KEY(triggered_by_dd) REFERENCES copy_log_dd(config_id)
        )
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS process_log_lines (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            process_id INTEGER NOT NULL,
            line_content TEXT NOT NULL,
            timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            line_number INTEGER NOT NULL,
            FOREIGN KEY(process_id) REFERENCES copy_process(id) ON DELETE CASCADE
        )
        "#,
        [],
    )?;

    Ok(())
}
