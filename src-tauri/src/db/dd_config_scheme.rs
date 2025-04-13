use rusqlite::{Connection, Result};

pub fn initialize_dd_config_scheme(conn: &Connection) -> Result<()> {
    conn.execute(
        r#"CREATE TABLE IF NOT EXISTS dd_config (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            active BOOLEAN NOT NULL DEFAULT true,
            confname TEXT NOT NULL,
            format TEXT NOT NULL DEFAULT 'auto'
                CHECK(format IN ('auto','512','1024','2048')),
            limit_mode TEXT NOT NULL DEFAULT 'whole'
                CHECK(limit_mode IN ('whole','ask')),
            offset TEXT NOT NULL DEFAULT '0',
            hash_types TEXT NOT NULL DEFAULT 'md5',
            hashwindow TEXT NOT NULL DEFAULT '1MB',
            split TEXT NOT NULL DEFAULT 'whole',
            vf BOOLEAN NOT NULL DEFAULT 0
                CHECK(vf IN (0,1)),
            diffwr BOOLEAN NOT NULL DEFAULT 0
                CHECK(diffwr IN (0,1)),
            notes TEXT NOT NULL DEFAULT 'ask'
                CHECK(notes IN ('ask','none'))
        )"#,
        [],
    )?;
    Ok(())
}