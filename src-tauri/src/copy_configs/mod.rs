use rusqlite::{Connection, Result, ToSql};
use tauri::command;

/// **Struktura pro vkládání nového záznamu (bez ID)**
#[derive(Debug, serde::Serialize)]
pub struct NewConfig {
    pub confname: String,
    pub codepage: String,
    pub sectors_per_read: String,
    pub bytes_to_read: String,
    pub compression_method: String,
    pub compression_level: String,
    pub hash_types: String,
    pub ewf_format: String,
    pub granularity_sectors: i32,
    pub notes: String,
    pub offset: String,
    pub process_buffer_size: String,
    pub bytes_per_sector: String,
    pub quiet_mode: bool,
    pub read_retry_count: String,
    pub swap_byte_pairs: bool,
    pub segment_size: String,
    pub verbose_output: bool,
    pub zero_on_read_error: bool,
    pub use_chunk_data: bool,
}

/// **Struktura pro načítání záznamu z databáze (s ID)**
#[derive(Debug, serde::Serialize)]
pub struct StoredConfig {
    pub id: i32, // Přidáno ID
    pub confname: String,
    pub codepage: String,
    pub sectors_per_read: String,
    pub bytes_to_read: String,
    pub compression_method: String,
    pub compression_level: String,
    pub hash_types: String,
    pub ewf_format: String,
    pub granularity_sectors: i32,
    pub notes: String,
    pub offset: String,
    pub process_buffer_size: String,
    pub bytes_per_sector: String,
    pub quiet_mode: bool,
    pub read_retry_count: String,
    pub swap_byte_pairs: bool,
    pub segment_size: String,
    pub verbose_output: bool,
    pub zero_on_read_error: bool,
    pub use_chunk_data: bool,
}

/// **Uložení nové konfigurace do databáze**
pub fn save_ewf_config(conn: &Connection, config: NewConfig) -> Result<()> {
    let params: Vec<&dyn ToSql> = vec![
        &config.confname,
        &config.codepage,
        &config.sectors_per_read,
        &config.bytes_to_read,
        &config.compression_method,
        &config.compression_level,
        &config.hash_types,
        &config.ewf_format,
        &config.granularity_sectors,
        &config.notes,
        &config.offset,
        &config.process_buffer_size,
        &config.bytes_per_sector,
        &config.quiet_mode,
        &config.read_retry_count,
        &config.swap_byte_pairs,
        &config.segment_size,
        &config.verbose_output,
        &config.zero_on_read_error,
        &config.use_chunk_data,
    ];
    conn.execute(
        r#"INSERT INTO ewf_config (
            confname,
            codepage,
            sectors_per_read,
            bytes_to_read,
            compression_method,
            compression_level,
            hash_types,
            ewf_format,
            granularity_sectors,
            notes,
            offset,
            process_buffer_size,
            bytes_per_sector,
            quiet_mode,
            read_retry_count,
            swap_byte_pairs,
            segment_size,
            verbose_output,
            zero_on_read_error,
            use_chunk_data
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)"#,
        params.as_slice(),
    )?;
    Ok(())
}

/// **Asynchronní funkce pro uložení nové konfigurace**
#[tauri::command(rename_all = "snake_case")]
pub async fn save_new_ewf_config(
    confname: String,
    codepage: String,
    sectors_per_read: String,
    bytes_to_read: String,
    compression_method: String,
    compression_level: String,
    hash_types: Vec<String>,
    ewf_format: String,
    granularity_sectors: i32,
    notes: String,
    offset: String,
    process_buffer_size: String,
    bytes_per_sector: String,
    quiet_mode: bool,
    read_retry_count: String,
    swap_byte_pairs: bool,
    segment_size: String,
    verbose_output: bool,
    zero_on_read_error: bool,
    use_chunk_data: bool,
) -> Result<(), String> {
    // Otevření databáze
    let conn = Connection::open("/var/lib/cratec/database.db")
        .map_err(|e| format!("Error opening DB: {}", e))?;
    
    // Spojíme pole hash_types do jednoho řetězce (např. odděleného čárkou)
    let hash_types_str = hash_types.join(",");

    let config = NewConfig {
        confname,
        codepage,
        sectors_per_read,
        bytes_to_read,
        compression_method,
        compression_level,
        hash_types: hash_types_str,
        ewf_format,
        granularity_sectors,
        notes,
        offset,
        process_buffer_size,
        bytes_per_sector,
        quiet_mode,
        read_retry_count,
        swap_byte_pairs,
        segment_size,
        verbose_output,
        zero_on_read_error,
        use_chunk_data,
    };

    // Spuštění synchronní funkce v bloku spawn_blocking
    tauri::async_runtime::spawn_blocking(move || {
        save_ewf_config(&conn, config)
    })
    .await
    .map_err(|e| format!("Async error: {}", e))?
    .map_err(|e| format!("Error saving config: {}", e))?;

    Ok(())
}

/// **Načtení všech konfigurací včetně ID**
pub fn get_all_configs(conn: &Connection) -> Result<Vec<StoredConfig>> {
    let mut stmt = conn.prepare(
        r#"SELECT 
            id,  -- Přidáno ID
            confname,
            codepage,
            sectors_per_read,
            bytes_to_read,
            compression_method,
            compression_level,
            hash_types,
            ewf_format,
            granularity_sectors,
            notes,
            offset,
            process_buffer_size,
            bytes_per_sector,
            quiet_mode,
            read_retry_count,
            swap_byte_pairs,
            segment_size,
            verbose_output,
            zero_on_read_error,
            use_chunk_data
        FROM ewf_config"#
    )?;

    let config_iter = stmt.query_map([], |row| {
        Ok(StoredConfig {
            id: row.get(0)?, // ID přidáno
            confname: row.get(1)?,
            codepage: row.get(2)?,
            sectors_per_read: row.get(3)?,
            bytes_to_read: row.get(4)?,
            compression_method: row.get(5)?,
            compression_level: row.get(6)?,
            hash_types: row.get(7)?,
            ewf_format: row.get(8)?,
            granularity_sectors: row.get(9)?,
            notes: row.get(10)?,
            offset: row.get(11)?,
            process_buffer_size: row.get(12)?,
            bytes_per_sector: row.get(13)?,
            quiet_mode: row.get(14)?,
            read_retry_count: row.get(15)?,
            swap_byte_pairs: row.get(16)?,
            segment_size: row.get(17)?,
            verbose_output: row.get(18)?,
            zero_on_read_error: row.get(19)?,
            use_chunk_data: row.get(20)?,
        })
    })?;
    
    let mut configs = Vec::new();
    for config in config_iter {
        configs.push(config?);
    }
    Ok(configs)
}

/// **Asynchronní příkaz pro získání všech konfigurací včetně ID**
#[tauri::command(rename_all = "snake_case")]
pub async fn get_all_ewf_configs() -> Result<Vec<StoredConfig>, String> {
    let conn = Connection::open("/var/lib/cratec/database.db")
        .map_err(|e| format!("Error opening DB: {}", e))?;
    let configs = tauri::async_runtime::spawn_blocking(move || {
        get_all_configs(&conn)
    })
    .await
    .map_err(|e| format!("Async error: {}", e))?
    .map_err(|e| format!("Error querying configs: {}", e))?;
    Ok(configs)
}


/// **Funkce pro smazání nebo deaktivaci konfigurace podle ID**
pub fn delete_or_deactivate_config(conn: &Connection, config_id: i32) -> Result<()> {
    // Zjistíme, zda existují záznamy v tabulce `copy_log`, které odkazují na daný `config_id`
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM copy_log_ewf WHERE config_id = ?1")?;
    let count: i32 = stmt.query_row([&config_id], |row| row.get(0))?;

    if count > 0 {
        // Pokud existují odkazy, pouze deaktivujeme konfiguraci
        conn.execute(
            "UPDATE config SET active = false WHERE id = ?1",
            [&config_id],
        )?;
        println!("Konfigurace s ID {} byla deaktivována.", config_id);
    } else {
        // Pokud žádné odkazy neexistují, smažeme konfiguraci
        conn.execute(
            "DELETE FROM ewf_config WHERE id = ?1",
            [&config_id],
        )?;
        println!("Konfigurace s ID {} byla odstraněna.", config_id);
    }
    
    Ok(())
}

/// **Asynchronní Tauri command pro smazání nebo deaktivaci konfigurace**
#[tauri::command(rename_all = "snake_case")]
pub async fn delete_or_deactivate_ewf_config(config_id: i32) -> Result<(), String> {
    // Otevření databáze
    let conn = Connection::open("/var/lib/cratec/database.db")
        .map_err(|e| format!("Chyba při otevírání DB: {}", e))?;

    // Spustíme operaci v bloku spawn_blocking
    tauri::async_runtime::spawn_blocking(move || {
        delete_or_deactivate_config(&conn, config_id)
    })
    .await
    .map_err(|e| format!("Asynchronní chyba: {}", e))?
    .map_err(|e| format!("Chyba při mazání/deaktivaci konfigurace: {}", e))?;

    Ok(())
}