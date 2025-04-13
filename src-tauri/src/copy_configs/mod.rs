use rusqlite::{Connection, Result, ToSql};

/// **Struktura pro vkládání nového záznamu (bez ID) pro EWF konfiguraci**
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
    pub granularity_sectors: String,
    pub notes: String,
    pub offset: String,
    pub process_buffer_size: String,
    pub bytes_per_sector: String,
    pub read_retry_count: String,
    pub swap_byte_pairs: bool,
    pub segment_size: String,
    pub zero_on_read_error: bool,
    pub use_chunk_data: bool,
}

/// **Struktura pro načítání záznamu z databáze (s ID) pro EWF konfiguraci**
#[derive(Debug, serde::Serialize)]
pub struct StoredEWFConfig {
    pub id: i32, // Přidáno ID
    pub confname: String,
    pub codepage: String,
    pub sectors_per_read: String,
    pub bytes_to_read: String,
    pub compression_method: String,
    pub compression_level: String,
    pub hash_types: String,
    pub ewf_format: String,
    pub granularity_sectors: String,
    pub notes: String,
    pub offset: String,
    pub process_buffer_size: String,
    pub bytes_per_sector: String,
    pub read_retry_count: String,
    pub swap_byte_pairs: bool,
    pub segment_size: String,
    pub zero_on_read_error: bool,
    pub use_chunk_data: bool,
    pub created: String,
}

/// **Uložení nové EWF konfigurace do databáze**
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
        &config.read_retry_count,
        &config.swap_byte_pairs,
        &config.segment_size,
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
            read_retry_count,
            swap_byte_pairs,
            segment_size,
            zero_on_read_error,
            use_chunk_data
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)"#,
        params.as_slice(),
    )?;
    Ok(())
}

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
    granularity_sectors: String,
    notes: String,
    offset: String,
    process_buffer_size: String,
    bytes_per_sector: String,
    read_retry_count: String,
    swap_byte_pairs: bool,
    segment_size: String,
    zero_on_read_error: bool,
    use_chunk_data: bool,
) -> Result<(), String> {
    let config = NewConfig {
        confname,
        codepage,
        sectors_per_read,
        bytes_to_read,
        compression_method,
        compression_level,
        hash_types: hash_types.join(","),
        ewf_format,
        granularity_sectors,
        notes,
        offset,
        process_buffer_size,
        bytes_per_sector,
        read_retry_count,
        swap_byte_pairs,
        segment_size,
        zero_on_read_error,
        use_chunk_data,
    };

    let db_conn = crate::db::DB_CONN.clone();

    // Použijeme asynchronní zámek
    let conn = db_conn.lock().await;

    // Spustíme synchronní operaci v rámci asynchronního kontextu
    save_ewf_config(&conn, config)
        .map_err(|e| format!("Error saving config: {}", e))?;

    Ok(())
}

/// **Upravená struktura pro vkládání nové DCFLDD konfigurace**
#[derive(Debug, serde::Serialize)]
pub struct NewDDConfig {
    pub confname: String,
    pub format: String,
    pub limit_mode: String,
    pub offset: String, // TEXT místo i32
    pub hash_types: String,
    pub hashwindow: String,
    pub split: String,
    pub vf: i32,
    pub diffwr: i32,
    pub notes: String,
}

/// **Synchronní funkce pro uložení DCFLDD konfigurace do databáze**
pub fn save_dd_config(conn: &Connection, config: NewDDConfig) -> Result<()> {
    let params: Vec<&dyn ToSql> = vec![
        &config.confname,
        &config.format,
        &config.limit_mode,
        &config.offset,
        &config.hash_types,
        &config.hashwindow,
        &config.split,
        &config.vf,
        &config.diffwr,
        &config.notes,
    ];
    conn.execute(
        r#"INSERT INTO dd_config (
            confname,
            format,
            limit_mode,
            offset,
            hash_types,
            hashwindow,
            split,
            vf,
            diffwr,
            notes
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
        params.as_slice(),
    )?;
    Ok(())
}

/// **Asynchronní Tauri command pro uložení nové DCFLDD konfigurace**
#[tauri::command(rename_all = "snake_case")]
pub async fn save_new_dd_config(
    confname: String,
    format: String,
    limit_mode: String,
    offset: String, // akceptuje i "ask"
    hash_types: Vec<String>,
    hashwindow_value: String,
    hashwindow_unit: String,
    split_value: String,
    split_unit: String,
    vf: String,
    diffwr: String,
    notes: String,
) -> Result<(), String> {
    let vf_parsed = if vf == "on" { 1 } else { 0 };
    let diffwr_parsed = if diffwr == "on" { 1 } else { 0 };

    let hashwindow = if hashwindow_value.to_lowercase() == "whole" {
        "whole".to_string()
    } else {
        format!("{}{}", hashwindow_value, hashwindow_unit)
    };

    let split = if split_value.to_lowercase() == "whole" {
        "whole".to_string()
    } else {
        format!("{}{}", split_value, split_unit)
    };

    let config = NewDDConfig {
        confname,
        format,
        limit_mode,
        offset, // uloží se jako TEXT
        hash_types: hash_types.join(","),
        hashwindow,
        split,
        vf: vf_parsed,
        diffwr: diffwr_parsed,
        notes,
    };

    let db_conn = crate::db::DB_CONN.clone();
    let conn = db_conn.lock().await;
    save_dd_config(&conn, config)
        .map_err(|e| format!("Error saving config: {e}"))?;
    Ok(())
}

/// **Upravená struktura pro načítání DD konfigurace z databáze (s ID)**
#[derive(Debug, serde::Serialize)]
pub struct StoredDDConfig {
    pub id: i32,
    pub created: String,
    pub active: bool,
    pub confname: String,
    pub format: String,
    pub limit_mode: String,
    pub offset: String, // místo seek/skip
    pub hash_types: String,
    pub hashwindow: String,
    pub split: String,
    pub vf: bool,
    pub diffwr: bool,
    pub notes: String,
}

/// **Struktura pro vrácení kombinovaných konfigurací**
#[derive(Debug, serde::Serialize)]
pub struct CombinedConfigs {
    pub ewf: Vec<StoredEWFConfig>,
    pub dd: Vec<StoredDDConfig>,
}

/// Načtení všech aktivních konfigurací pro EWF a DD
pub fn get_all_configs(conn: &Connection) -> Result<CombinedConfigs> {
    let mut stmt_ewf = conn.prepare(
        r#"SELECT 
            id,
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
            read_retry_count,
            swap_byte_pairs,
            segment_size,
            zero_on_read_error,
            use_chunk_data,
            created
         FROM ewf_config
         WHERE active = true"#,
    )?;
    let ewf_iter = stmt_ewf.query_map([], |row| {
        Ok(StoredEWFConfig {
            id: row.get(0)?,
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
            read_retry_count: row.get(14)?,
            swap_byte_pairs: row.get(15)?,
            segment_size: row.get(16)?,
            zero_on_read_error: row.get(17)?,
            use_chunk_data: row.get(18)?,
            created: row.get(19)?,
        })
    })?;
    let mut ewf_configs = Vec::new();
    for config in ewf_iter {
        ewf_configs.push(config?);
    }

    let mut stmt_dd = conn.prepare(
        r#"SELECT 
            id,
            created,
            active,
            confname,
            format,
            limit_mode,
            offset,
            hash_types,
            hashwindow,
            split,
            vf,
            diffwr,
            notes
         FROM dd_config
         WHERE active = true"#, // odstraněno seek, skip
    )?;
    let dd_iter = stmt_dd.query_map([], |row| {
        Ok(StoredDDConfig {
            id: row.get(0)?,
            created: row.get(1)?,
            active: row.get(2)?,
            confname: row.get(3)?,
            format: row.get(4)?,
            limit_mode: row.get(5)?,
            offset: row.get(6)?, // místo seek, skip
            hash_types: row.get(7)?,
            hashwindow: row.get(8)?,
            split: row.get(9)?,
            vf: row.get(10)?,
            diffwr: row.get(11)?,
            notes: row.get(12)?,
        })
    })?;
    let mut dd_configs = Vec::new();
    for config in dd_iter {
        dd_configs.push(config?);
    }

    Ok(CombinedConfigs {
        ewf: ewf_configs,
        dd: dd_configs,
    })
}

/// **Asynchronní příkaz pro získání všech aktivních konfigurací**
#[tauri::command(rename_all = "snake_case")]
pub async fn get_all_active_configs() -> Result<CombinedConfigs, String> {
    let conn = Connection::open("/var/lib/cratec/database.db")
        .map_err(|e| format!("Error opening DB: {}", e))?;
    let configs = tauri::async_runtime::spawn_blocking(move || get_all_configs(&conn))
        .await
        .map_err(|e| format!("Async error: {}", e))?
        .map_err(|e| format!("Error querying configs: {}", e))?;
    Ok(configs)
}

/// **Funkce pro smazání nebo deaktivaci konfigurace podle ID a typu**
pub fn delete_or_deactivate(conn: &Connection, config_id: i32, config_type: &str) -> Result<()> {
    // Určíme příslušné tabulky na základě typu konfigurace
    let (log_table, config_table) = match config_type {
        "ewf" => ("copy_log_ewf", "ewf_config"),
        "dd" => ("copy_log_dd", "dd_config"),
        _ => return Err(rusqlite::Error::InvalidQuery),
    };

    // Zjistíme, zda existují záznamy v logovací tabulce
    let query = format!("SELECT COUNT(*) FROM {} WHERE config_id = ?1", log_table);
    let mut stmt = conn.prepare(&query)?;
    let count: i32 = stmt.query_row([&config_id], |row| row.get(0))?;

    if count > 0 {
        // Pokud existují odkazy, deaktivujeme konfiguraci
        let update_query = format!("UPDATE {} SET active = false WHERE id = ?1", config_table);
        conn.execute(&update_query, [&config_id])?;
        println!("Konfigurace s ID {} byla deaktivována.", config_id);
    } else {
        // Pokud neexistují, smažeme konfiguraci
        let delete_query = format!("DELETE FROM {} WHERE id = ?1", config_table);
        conn.execute(&delete_query, [&config_id])?;
        println!("Konfigurace s ID {} byla odstraněna.", config_id);
    }

    Ok(())
}

/// **Asynchroní Tauri command pro smazání nebo deaktivaci konfigurace podle ID a typu**
#[tauri::command(rename_all = "snake_case")]
pub async fn delete_or_deactivate_config(
    config_id: i32,
    config_type: String,
) -> Result<(), String> {
    let conn = Connection::open("/var/lib/cratec/database.db")
        .map_err(|e| format!("Chyba při otevírání DB: {}", e))?;
    tauri::async_runtime::spawn_blocking(move || {
        delete_or_deactivate(&conn, config_id, &config_type)
    })
    .await
    .map_err(|e| format!("Asynchronní chyba: {}", e))?
    .map_err(|e| format!("Chyba při mazání/deaktivaci konfigurace: {}", e))?;
    Ok(())
}
