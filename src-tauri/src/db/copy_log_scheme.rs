use rusqlite::{Connection, Result};

pub fn initialize_copy_log_scheme(conn: &Connection) -> Result<()> {
    // Vytvoření tabulky copy_log (proměnlivá nastavení)
    conn.execute(
        r#"CREATE TABLE IF NOT EXISTS copy_log_ewf (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            config_id INTEGER NOT NULL,
            source TEXT NOT NULL DEFAULT '',
            case_number TEXT NOT NULL,
            description TEXT NOT NULL,
            investigator_name TEXT NOT NULL,
            evidence_number TEXT NOT NULL,
            acquisition_restart BOOLEAN NOT NULL DEFAULT false,
            media_type TEXT NOT NULL DEFAULT 'fixed',
            media_characteristics TEXT NOT NULL DEFAULT 'physical',
            notes TEXT,
            offset TEXT DEFAULT NULL,
            bytes_to_read TEXT DEFAULT NULL,
            secondary_target_file TEXT NOT NULL DEFAULT '',
            start_datetime DATETIME NOT NULL,
            end_datetime DATETIME,
            status TEXT NOT NULL DEFAULT 'running'
                CHECK(status IN ('running','done','error')),
            source_disk_id INTEGER,
            dest_disk_id TEXT NOT NULL,
            second_dest_disk_id TEXT,
            md5_hash TEXT DEFAULT NULL, 
            sha1_hash TEXT DEFAULT NULL,
            sha256_hash TEXT DEFAULT NULL,
            FOREIGN KEY(config_id) REFERENCES ewf_config(id) ON DELETE CASCADE
        )"#,
        [],
    )?;

    conn.execute(
        r#"CREATE TABLE IF NOT EXISTS copy_log_dd (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            config_id INTEGER NOT NULL,
            source TEXT NOT NULL DEFAULT '',
            case_number TEXT NOT NULL,
            description TEXT NOT NULL,
            investigator_name TEXT NOT NULL,
            evidence_number TEXT NOT NULL,
            notes TEXT DEFAULT NULL,
            secondary_target_file TEXT NOT NULL DEFAULT '',
            start_datetime DATETIME NOT NULL,
            end_datetime DATETIME,
            status TEXT NOT NULL DEFAULT 'running'
                CHECK(status IN ('running','done','error')),
            source_disk_id INTEGER,
            dest_disk_id TEXT NOT NULL,
            second_dest_disk_id TEXT,
            limit_value TEXT,
            offset TEXT DEFAULT NULL,
            md5_hash TEXT DEFAULT NULL, 
            sha1_hash TEXT DEFAULT NULL,
            sha256_hash TEXT DEFAULT NULL,
            sha384_hash TEXT DEFAULT NULL,
            sha512_hash TEXT DEFAULT NULL,
            FOREIGN KEY(config_id) REFERENCES dd_config(id) ON DELETE CASCADE
        )"#,
        [],
    )?;

    Ok(())
}
