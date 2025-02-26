use rusqlite::{Connection, Result};

pub fn initialize_dd_config_scheme(conn: &Connection) -> Result<()> {
    conn.execute(
        r#"CREATE TABLE IF NOT EXISTS dd_config (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            active BOOLEAN NOT NULL DEFAULT true,
            confname TEXT NOT NULL,
            bs INTEGER NOT NULL DEFAULT 32768
                CHECK(bs IN (32768, 65536, 131072, 262144, 524288, 1048576, 1572864, 2097152, 3145728, 4194304, 5242880, 6291456, 7340032, 8388608)),
            count TEXT NOT NULL DEFAULT 'whole'
                CHECK(count IN ('whole', 'ask')),
            ibs INTEGER NOT NULL DEFAULT 32768,
            obs INTEGER NOT NULL DEFAULT 32768,
            seek INTEGER NOT NULL DEFAULT 0,
            skip INTEGER NOT NULL DEFAULT 0,
            hash_types TEXT NOT NULL DEFAULT 'md5',
            hashwindow INTEGER NOT NULL DEFAULT 4096,
            hashlog TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'on'
                CHECK(status IN ('on', 'off')),
            statusinterval INTEGER NOT NULL DEFAULT 256,
            split TEXT NOT NULL DEFAULT 'ask'
                CHECK(split IN ('ask', 'disabled')),
            splitformat TEXT NOT NULL DEFAULT 'nnn',
            vf TEXT NOT NULL DEFAULT 'ask',
            verifylog TEXT NOT NULL DEFAULT ''
        )"#,
        [],
    )?;
    Ok(())
}
