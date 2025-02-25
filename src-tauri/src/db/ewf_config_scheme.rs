use rusqlite::{Connection, Result};

/// Inicializace databáze: vytvoří tabulky config a copy_log.
/// Tabulka config obsahuje fixní pole a tabulka copy_log obsahuje proměnlivá pole s FK na config.
pub fn initialize_ewf_config_scheme(conn: &Connection) -> Result<()> {
    // Vytvoření tabulky config (fixní nastavení)
    conn.execute(
        r#"CREATE TABLE IF NOT EXISTS ewf_config (
            id INTEGER PRIMARY KEY,
            created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            active BOOLEAN NOT NULL DEFAULT true,
            confname TEXT NOT NULL,
            codepage TEXT NOT NULL DEFAULT 'ascii'
                CHECK(codepage IN ('ascii','windows-874','windows-932','windows-936','windows-949','windows-950','windows-1250','windows-1251','windows-1252','windows-1253','windows-1254','windows-1255','windows-1256','windows-1257','windows-1258')),
            sectors_per_read TEXT NOT NULL DEFAULT '64'
                CHECK(sectors_per_read IN ('16','32','64','128','256','512','1024','2048','4096','8192','16384','32768')),
            bytes_to_read TEXT NOT NULL DEFAULT 'whole'
                CHECK(bytes_to_read IN ('whole','ask')),
            compression_method TEXT NOT NULL DEFAULT 'deflate'
                CHECK(compression_method = 'deflate'),
            compression_level TEXT NOT NULL DEFAULT 'none'
                CHECK(compression_level IN ('none','empty-block','fast','best')),
            hash_types TEXT NOT NULL DEFAULT '[]',
            ewf_format TEXT NOT NULL DEFAULT 'encase6'
                CHECK(ewf_format IN ('ewf','smart','ftk','encase2','encase3','encase4','encase5','encase6','linen5','linen6','ewfx')),
            granularity_sectors INTEGER NOT NULL DEFAULT 0,
            notes TEXT NOT NULL DEFAULT '',
            offset TEXT NOT NULL DEFAULT '0'
                CHECK(offset IN ('0','ask')),
            process_buffer_size TEXT NOT NULL DEFAULT '',
            bytes_per_sector TEXT NOT NULL DEFAULT 'auto',
            quiet_mode BOOLEAN NOT NULL DEFAULT false,
            read_retry_count TEXT NOT NULL DEFAULT '2',
            swap_byte_pairs BOOLEAN NOT NULL DEFAULT false,
            segment_size TEXT NOT NULL DEFAULT '1.4 GiB',
            verbose_output BOOLEAN NOT NULL DEFAULT false,
            zero_on_read_error BOOLEAN NOT NULL DEFAULT false,
            use_chunk_data BOOLEAN NOT NULL DEFAULT false
        )"#,
        [],
    )?;

    Ok(())
}
