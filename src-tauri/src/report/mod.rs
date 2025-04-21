use crate::db::DB_POOL;
use crate::disk_utils;
use chrono::{Local, NaiveDateTime};
use rusqlite::{params, Connection, Row, Statement};
use serde_json::{Map, Value};
use std::fs;
use tectonic;
use tera::{Context, Tera};

static TEMPLATE: &str = include_str!("./templates/en.tex");

fn row_to_json(row: &Row, stmt: &Statement) -> Value {
    let mut obj = Map::new();
    for (i, name) in stmt.column_names().iter().enumerate() {
        let value = if let Ok(s) = row.get::<_, String>(i) {
            Value::String(s)
        } else if let Ok(n) = row.get::<_, i64>(i) {
            Value::Number(serde_json::Number::from(n))
        } else {
            Value::Null
        };
        obj.insert(name.to_string(), value);
    }
    Value::Object(obj)
}

fn row_to_json_with_columns(row: &Row, column_names: &[String]) -> Value {
    let mut obj = Map::new();
    for (i, name) in column_names.iter().enumerate() {
        let value = if let Ok(s) = row.get::<_, String>(i) {
            Value::String(s)
        } else if let Ok(n) = row.get::<_, i64>(i) {
            Value::Number(serde_json::Number::from(n))
        } else {
            Value::Null
        };
        obj.insert(name.clone(), value);
    }
    Value::Object(obj)
}

fn get_interface_path(conn: &rusqlite::Connection, interface_id: i64) -> Option<String> {
    conn.query_row(
        "SELECT interface_path FROM interface WHERE id = ?1",
        rusqlite::params![interface_id],
        |row| row.get(0),
    )
    .ok()
}

/// Nová funkce generate_report, která zatím přijímá copy_process id a nedělá nic dalšího.
pub fn generate_report(copy_process_id: i64) -> Result<(), String> {
    // 1) Získání dat z DB pomocí get_report_json_data
    let report_json = get_report_json_data(copy_process_id)?;

    // Aktualizujeme report_json o diskové informace získané z log_record
    let mut report = match report_json {
        serde_json::Value::Object(map) => map,
        _ => return Err("Report JSON není objekt.".into()),
    };

    // Hledáme v reportu log_record a vždy vložíme pole source_disk, dest_disk a second_dest_disk
    if let Some(log_value) = report.get("log_record").cloned() {
        if let Some(log_obj) = log_value.as_object() {
            let mut pooled_conn = DB_POOL.get_connection().unwrap();
            let conn = pooled_conn.connection();

            // Získání source_disk
            let source_disk = if let Some(source_disk_id) =
                log_obj.get("source_disk_id").and_then(|v| v.as_i64())
            {
                if source_disk_id != 0 {
                    if let Some(interface_path) = get_interface_path(conn, source_disk_id) {
                        let path = format!("/dev/disk/by-path/{}", interface_path);
                        println!("SOURCE DISK PAH :{} ", path);
                        let disk_info = disk_utils::get_disk_info(&path)?;
                        serde_json::to_value(disk_info).map_err(|e| {
                            format!("Chyba při serializaci disk info (source): {}", e)
                        })?
                    } else {
                        serde_json::Value::String(String::new())
                    }
                } else {
                    serde_json::Value::String(String::new())
                }
            } else {
                serde_json::Value::String(String::new())
            };
            report.insert("source_disk".into(), source_disk);

            // Získání dest_disk
            let dest_disk = if let Some(dest_disk_id) =
                log_obj.get("dest_disk_id").and_then(|v| v.as_i64())
            {
                if dest_disk_id != 0 {
                    if let Some(interface_path) = get_interface_path(conn, dest_disk_id) {
                        let path = format!("/dev/disk/by-path/{}", interface_path);
                        println!("DEST DISK PAH :{} ", path);
                        let disk_info = disk_utils::get_disk_info(&path)?;
                        serde_json::to_value(disk_info)
                            .map_err(|e| format!("Chyba při serializaci disk info (dest): {}", e))?
                    } else {
                        serde_json::Value::String(String::new())
                    }
                } else {
                    serde_json::Value::String(String::new())
                }
            } else {
                serde_json::Value::String(String::new())
            };
            report.insert("dest_disk".into(), dest_disk);

            // Získání second_dest_disk (vždy vloženo, může být prázdné)
            let second_dest_disk = if let Some(second_dest_disk_id) =
                log_obj.get("second_dest_disk_id").and_then(|v| v.as_i64())
            {
                if second_dest_disk_id != 0 {
                    if let Some(interface_path) = get_interface_path(conn, second_dest_disk_id) {
                        let path = format!("/dev/disk/by-path/{}", interface_path);
                        println!("SECOND DISK PAH :{} ", path);
                        let disk_info = disk_utils::get_disk_info(&path)?;
                        serde_json::to_value(disk_info).map_err(|e| {
                            format!("Chyba při serializaci disk info (second): {}", e)
                        })?
                    } else {
                        serde_json::Value::String(String::new())
                    }
                } else {
                    serde_json::Value::String(String::new())
                }
            } else {
                serde_json::Value::String(String::new())
            };
            report.insert("second_dest_disk".into(), second_dest_disk);
        }
    }

    // Vypsání modifikovaného reportu do konzole ve formátu "pretty printed" JSON.
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::Value::Object(report.clone()))
            .map_err(|e| format!("Chyba při serializaci JSON: {}", e))?
    );

    // 2) Nyní vytvoříme Tera context, kam vložíme mock data.
    let mut context = Context::new();

    // Aktuální datum a čas
    let now = Local::now();
    context.insert("date", &now.format("%b %d, %Y").to_string());
    context.insert("time_local", &now.format("%H:%M:%S (%Z)").to_string());
    context.insert("software_hash", "75f1c14d734ea09147330fae210faa54");
    context.insert("build_date", "Jul 08, 2024 13:38:46 PDT");
    context.insert("serial_number", "117495");
    context.insert("mode", "DriveToFile");

    // Nastavení metody podle toho, co bylo využito (triggered_by_ewf či triggered_by_dd)
    let method = if report
        .get("copy_process")
        .and_then(|cp| cp.get("triggered_by_ewf"))
        .and_then(|v| v.as_i64())
        .is_some()
    {
        "ewfacquire"
    } else {
        "dcfldd"
    };
    context.insert("method", method);

    // Pole "hash_type" nastavíme dle hodnoty "hash_types" z config_record
    let hash_types = report
        .get("config_record")
        .and_then(|config| config.get("hash_types"))
        .and_then(|v| v.as_str())
        .unwrap_or("N/A");
    context.insert("hash_type", hash_types);

    context.insert("image_path", "/var/reports/.../Kingston480GB\\_SATA");

    // lba_count nastavíme na hodnotu capacity_bytes ze source_disk
    let lba_count_val = report
        .get("source_disk")
        .and_then(|sd| sd.get("capacity_bytes"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    context.insert("lba_count", &lba_count_val);

    let sector_size_val = report
        .get("source_disk")
        .and_then(|sd| sd.get("logical_sector_size"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    context.insert("sector_size", &sector_size_val);

    let segment_size = report
        .get("config_record")
        .and_then(|config| config.get("segment_size"))
        .and_then(|v| v.as_str())
        .unwrap_or("N/A");
    context.insert("segment_size", segment_size);

    // místo pevného "compression"
    let compression_method = report
        .get("config_record")
        .and_then(|cfg| cfg.get("compression_method"))
        .and_then(Value::as_str)
        .unwrap_or("None");
    context.insert("compression_method", compression_method);

    let compression_level = report
        .get("config_record")
        .and_then(|cfg| cfg.get("compression_level"))
        .and_then(Value::as_str)
        .unwrap_or("None");
    context.insert("compression_level", compression_level);

    let ewf_format = report
        .get("config_record")
        .and_then(|cfg| cfg.get("ewf_format"))
        .and_then(Value::as_str)
        .unwrap_or("N/A");
    context.insert("ewf_format", ewf_format);

    let hash_enabled = if hash_types != "N/A" && !hash_types.trim().is_empty() {
        true
    } else {
        false
    };
    context.insert("hash_enabled", &hash_enabled);

    context.insert("verify_hash", &false);
    context.insert("unlock_hpa", &false);
    context.insert("unlock_dco", &false);

    let granularity_sectors = report
        .get("config_record")
        .and_then(|cfg| cfg.get("granularity_sectors"))
        .and_then(Value::as_str)
        .unwrap_or("N/A");
    context.insert("granularity_sectors", granularity_sectors);

    let swap_byte_pairs = match report
        .get("config_record")
        .and_then(|cfg| cfg.get("swap_byte_pairs"))
        .and_then(Value::as_i64)
    {
        Some(0) => serde_json::Value::Bool(false),
        Some(1) => serde_json::Value::Bool(true),
        _ => serde_json::Value::String("N/A".to_string()),
    };
    context.insert("swap_byte_pairs", &swap_byte_pairs);

    // místo pevného "result"
    let result = match report
        .get("copy_process")
        .and_then(|cp| cp.get("status"))
        .and_then(Value::as_str)
    {
        Some("done") => "SUCCESS",
        Some("error") => "ERROR",
        _ => "N/A",
    };
    context.insert("result", result);

    // Získání a formátování time_started z copy_process.start_datetime
    let time_started_raw = report
        .get("copy_process")
        .and_then(|cp| cp.get("start_datetime"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let time_started = NaiveDateTime::parse_from_str(time_started_raw, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.format("%H:%M:%S %-d.%-m.%Y").to_string())
        .unwrap_or_else(|_| "".to_string());
    context.insert("time_started", &time_started);

    // Výpočet duration jako rozdíl mezi copy_process.end_datetime a copy_process.start_datetime
    let duration_str = {
        let fmt = "%Y-%m-%d %H:%M:%S";
        let start_opt = NaiveDateTime::parse_from_str(time_started_raw, fmt).ok();
        let end_opt = report
            .get("copy_process")
            .and_then(|cp| cp.get("end_datetime"))
            .and_then(Value::as_str)
            .and_then(|end_str| NaiveDateTime::parse_from_str(end_str, fmt).ok());
        if let (Some(start_dt), Some(end_dt)) = (start_opt, end_opt) {
            let diff = end_dt - start_dt;
            let total_secs = diff.num_seconds();
            let hours = total_secs / 3600;
            let minutes = (total_secs % 3600) / 60;
            let seconds = total_secs % 60;
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            "N/A".to_string()
        }
    };
    context.insert("duration", &duration_str);

    // Získání a formátování time_complete z log_record.end_datetime
    let time_complete_raw = report
        .get("log_record")
        .and_then(|lr| lr.get("end_datetime"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let time_complete = NaiveDateTime::parse_from_str(time_complete_raw, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.format("%H:%M:%S %-d.%-m.%Y").to_string())
        .unwrap_or_else(|_| "".to_string());
    context.insert("time_complete", &time_complete);

    let log_record = report
        .get("log_record")
        .and_then(|v| v.as_object())
        .unwrap();

    let mut hashes = Vec::new();

    if let Some(md5) = log_record.get("md5_hash").and_then(|v| v.as_str()) {
        if !md5.is_empty() {
            hashes.push(("MD5", md5));
        }
    }
    if let Some(sha1) = log_record.get("sha1_hash").and_then(|v| v.as_str()) {
        if !sha1.is_empty() {
            hashes.push(("SHA1", sha1));
        }
    }
    if let Some(sha256) = log_record.get("sha256_hash").and_then(|v| v.as_str()) {
        if !sha256.is_empty() {
            hashes.push(("SHA256", sha256));
        }
    }
    context.insert("hashes", &hashes);
    context.insert("case_file", "CaseFile\\_001");

    // místo pevného "case_id"
    let case_number = report
        .get("log_record")
        .and_then(|lr| lr.get("case_number"))
        .and_then(Value::as_str)
        .unwrap_or("");
    context.insert("case_number", case_number);

    let evidence_number = report
        .get("log_record")
        .and_then(|lr| lr.get("evidence_number"))
        .and_then(Value::as_str)
        .unwrap_or("");
    context.insert("evidence_number", evidence_number);

    // místo pevného "examiner"
    let examiner = report
        .get("log_record")
        .and_then(|lr| lr.get("investigator_name"))
        .and_then(Value::as_str)
        .unwrap_or("");
    context.insert("examiner", examiner);

    // místo pevného "notes"
    let notes = report
        .get("log_record")
        .and_then(|lr| lr.get("notes"))
        .and_then(Value::as_str)
        .unwrap_or("");
    context.insert("notes", notes);

    context.insert("segment_uid", "SEG123456");
    context.insert("segment_path", "/segments/seg123456");
    context.insert("segment_fs", "NTFS");
    context.insert("segment_serial", "SEG-SERIAL-001");
    context.insert("segment_file", "segment\\_file.img");
    context.insert("segment_hash", "seg\\_hash\\_abc123");

    let mut drives = Vec::new();

    if let Some(source_disk) = report.get("source_disk").and_then(|v| v.as_object()) {
        let mut map = std::collections::HashMap::new();
        map.insert("bay", "1");
        map.insert("role", "Source");
        map.insert(
            "serial",
            source_disk
                .get("serial")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
        );
        map.insert(
            "model",
            source_disk
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
        );
        map.insert(
            "fs",
            source_disk
                .get("partitions")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.get(0))
                .and_then(|p| p.get("filesystem"))
                .and_then(|v| v.as_str())
                .unwrap_or(""),
        );
        map.insert(
            "cipher",
            source_disk
                .get("disk_encryption")
                .and_then(|v| v.as_str())
                .unwrap_or("None"),
        );
        drives.push(map);
    }

    if let Some(dest_disk) = report.get("dest_disk").and_then(|v| v.as_object()) {
        let mut map = std::collections::HashMap::new();
        map.insert("bay", "2");
        map.insert("role", "Destination");
        map.insert(
            "serial",
            dest_disk
                .get("serial")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
        );
        map.insert(
            "model",
            dest_disk
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
        );
        map.insert(
            "fs",
            dest_disk
                .get("partitions")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.get(0))
                .and_then(|p| p.get("filesystem"))
                .and_then(|v| v.as_str())
                .unwrap_or(""),
        );
        map.insert(
            "cipher",
            dest_disk
                .get("disk_encryption")
                .and_then(|v| v.as_str())
                .unwrap_or("None"),
        );
        drives.push(map);
    }

    if let Some(second_dest_disk) = report.get("second_dest_disk").and_then(|v| v.as_object()) {
        // Pokud není prázdný objekt
        if !second_dest_disk.is_empty() {
            let mut map = std::collections::HashMap::new();
            map.insert("bay", "3");
            map.insert("role", "Secondary Destination");
            map.insert(
                "serial",
                second_dest_disk
                    .get("serial")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
            );
            map.insert(
                "model",
                second_dest_disk
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
            );
            map.insert(
                "fs",
                second_dest_disk
                    .get("partitions")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| arr.get(0))
                    .and_then(|p| p.get("filesystem"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
            );
            map.insert(
                "cipher",
                second_dest_disk
                    .get("disk_encryption")
                    .and_then(|v| v.as_str())
                    .unwrap_or("None"),
            );
            drives.push(map);
        }
    }

    context.insert("drives", &drives);

    let mut capacities = Vec::new();

    if let Some(source_disk) = report.get("source_disk").and_then(|v| v.as_object()) {
        let mut map = std::collections::HashMap::new();
        map.insert("bay".to_string(), "1".to_string());
        map.insert(
            "serial".to_string(),
            source_disk
                .get("serial")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        );
        map.insert(
            "model".to_string(),
            source_disk
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        );
        let cap_bytes = source_disk
            .get("capacity_bytes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let cap_gb = (cap_bytes as f64) / 1024.0 / 1024.0 / 1024.0;
        map.insert("capacity_bytes".to_string(), cap_bytes.to_string());
        map.insert("capacity_gb".to_string(), format!("{:.1}", cap_gb));
        capacities.push(map);
    }

    if let Some(dest_disk) = report.get("dest_disk").and_then(|v| v.as_object()) {
        let mut map = std::collections::HashMap::new();
        map.insert("bay".to_string(), "2".to_string());
        map.insert(
            "serial".to_string(),
            dest_disk
                .get("serial")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        );
        map.insert(
            "model".to_string(),
            dest_disk
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        );
        let cap_bytes = dest_disk
            .get("capacity_bytes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let cap_gb = (cap_bytes as f64) / 1024.0 / 1024.0 / 1024.0;
        map.insert("capacity_bytes".to_string(), cap_bytes.to_string());
        map.insert("capacity_gb".to_string(), format!("{:.1}", cap_gb));
        capacities.push(map);
    }

    if let Some(second_dest_disk) = report.get("second_dest_disk").and_then(|v| v.as_object()) {
        if !second_dest_disk.is_empty() {
            let mut map = std::collections::HashMap::new();
            map.insert("bay".to_string(), "3".to_string());
            map.insert(
                "serial".to_string(),
                second_dest_disk
                    .get("serial")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            );
            map.insert(
                "model".to_string(),
                second_dest_disk
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            );
            let cap_bytes = second_dest_disk
                .get("capacity_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let cap_gb = (cap_bytes as f64) / 1024.0 / 1024.0 / 1024.0;
            map.insert("capacity_bytes".to_string(), cap_bytes.to_string());
            map.insert("capacity_gb".to_string(), format!("{:.1}", cap_gb));
            capacities.push(map);
        }
    }
    context.insert("capacities", &capacities);

    // ATA security
    let ata_security = vec![{
        let mut map = std::collections::HashMap::new();
        map.insert("bay", "1");
        map.insert("role", "Primary");
        map.insert("enabled", "Yes");
        map.insert("locked", "No");
        map
    }];
    context.insert("ata_security", &ata_security);

    // Encryption
    let encryption = vec![{
        let mut map = std::collections::HashMap::new();
        map.insert("bay", "2");
        map.insert("role", "Secondary");
        map.insert("encrypted", "Yes");
        map.insert("locked", "Yes");
        map
    }];
    context.insert("encryption", &encryption);

    let mut source_partitions = Vec::new();
    if let Some(source_disk) = report.get("source_disk").and_then(|v| v.as_object()) {
        if let Some(parts) = source_disk.get("partitions").and_then(|v| v.as_array()) {
            for part in parts {
                let mut map = std::collections::HashMap::new();
                map.insert(
                    "index".to_string(),
                    part.get("index")
                        .and_then(|v| v.as_u64())
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                );
                map.insert(
                    "fs".to_string(),
                    part.get("filesystem")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                );
                map.insert(
                    "start".to_string(),
                    part.get("start_sector")
                        .and_then(|v| v.as_u64())
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                );
                map.insert(
                    "end".to_string(),
                    part.get("end_sector")
                        .and_then(|v| v.as_u64())
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                );
                // Výpočet velikosti v sektorech a v MB
                let start = part
                    .get("start_sector")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let end = part.get("end_sector").and_then(|v| v.as_u64()).unwrap_or(0);
                let size_mb = ((end + 1).saturating_sub(start)) as f64 * 512.0 / 1024.0 / 1024.0;
                map.insert("size".to_string(), format!("{:.1} MB", size_mb));
                let is_encrypted = part
                    .get("is_encrypted")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                map.insert(
                    "encryption".to_string(),
                    if is_encrypted {
                        "Yes".to_string()
                    } else {
                        "No".to_string()
                    },
                );
                // Pokud máš info o dešifrování, doplň, jinak nech prázdné
                map.insert("decrypted".to_string(), "".to_string());
                source_partitions.push(map);
            }
        }
    }
    context.insert("source_partitions", &source_partitions);

    
    println!("🧾 Vyrenderuji šablonu z paměti...");
    let latex_code = Tera::one_off(TEMPLATE, &context, false)
        .map_err(|e| format!("Chyba při renderování šablony: {}", e))?;

    fs::write("/home/master/Dokumenty/debug_output.tex", &latex_code)
        .map_err(|e| format!("Nelze uložit debug_output.tex: {}", e))?;

    println!("🧾 Kompiluji PDF pomocí Tectonic...");
    match tectonic::latex_to_pdf(&latex_code) {
        Ok(pdf_data) => {
            fs::write("/home/master/Dokumenty/output.pdf", pdf_data)
                .map_err(|e| format!("Nelze uložit output.pdf: {}", e))?;
            println!("✅ PDF úspěšně vytvořeno: output.pdf");
        }
        Err(e) => {
            eprintln!("❌ Chyba při kompilaci: {:#?}", e);
            println!("🧪 LaTeX byl uložen do debug_output.tex.");
            return Err(format!("Chyba při kompilaci: {:?}", e));
        }
    }

    Ok(())
}

pub fn get_report_json_data(copy_process_id: i64) -> Result<serde_json::Value, String> {
    println!("Načítám data pro copy_process id: {}", copy_process_id);

    // Získání sdíleného připojení z DB_POOL
    let mut pooled_conn = DB_POOL
        .get_connection()
        .map_err(|e| format!("Chyba při získání připojení: {}", e))?;
    let conn = pooled_conn.connection();

    // 1) Načíst záznam z tabulky copy_process jako serde_json::Value
    let copy_process: serde_json::Value = conn
        .query_row(
            "SELECT * FROM copy_process WHERE id = ?1",
            rusqlite::params![copy_process_id],
            |row| {
                let column_names: Vec<String> = row
                    .as_ref()
                    .column_names()
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                Ok(row_to_json_with_columns(row, &column_names))
            },
        )
        .map_err(|e| format!("Chyba načítání copy_process: {}", e))?;

    // 2) Získat hodnoty triggered_by_ewf a triggered_by_dd z copy_process
    let (triggered_by_ewf, triggered_by_dd): (Option<i64>, Option<i64>) = conn
        .query_row(
            "SELECT triggered_by_ewf, triggered_by_dd FROM copy_process WHERE id = ?1",
            rusqlite::params![copy_process_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("Chyba při načítání triggerů z copy_process: {}", e))?;
    println!(
        "triggered_by_ewf={:?}, triggered_by_dd={:?}",
        triggered_by_ewf, triggered_by_dd
    );

    // 3) Podle triggerů určit logovací a konfigurační tabulku
    let (log_table, log_id, config_table) = if let Some(ewf_log_id) = triggered_by_ewf {
        println!("Proces spuštěn přes EWF, log id: {}", ewf_log_id);
        ("copy_log_ewf", ewf_log_id, "ewf_config")
    } else if let Some(dd_log_id) = triggered_by_dd {
        println!("Proces spuštěn přes DD, log id: {}", dd_log_id);
        ("copy_log_dd", dd_log_id, "dd_config")
    } else {
        return Err("copy_process neobsahuje ani triggered_by_ewf ani triggered_by_dd".into());
    };

    // 4) Načíst záznam z logovací tabulky
    let log_query = format!("SELECT * FROM {} WHERE id = ?1", log_table);
    let mut stmt_log = conn
        .prepare(&log_query)
        .map_err(|e| format!("Chyba přípravy dotazu na {}: {}", log_table, e))?;
    let log_columns: Vec<String> = stmt_log
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let log_record: serde_json::Value = stmt_log
        .query_row(rusqlite::params![log_id], |row| {
            Ok(row_to_json_with_columns(row, &log_columns))
        })
        .map_err(|e| format!("Chyba načítání z tabulky {}: {}", log_table, e))?;

    // 5) Z logovacího záznamu získat config_id (předpokládáme sloupec "config_id")
    let config_id: i64 = {
        let mut stmt = conn
            .prepare(&format!(
                "SELECT config_id FROM {} WHERE id = ?1",
                log_table
            ))
            .map_err(|e| format!("Chyba přípravy dotazu pro config_id: {}", e))?;
        stmt.query_row(rusqlite::params![log_id], |row| row.get(0))
            .map_err(|e| format!("Chyba načítání config_id z tabulky {}: {}", log_table, e))?
    };
    println!("Z logu získaný config_id: {}", config_id);

    // 6) Načíst celý konfigurační záznam
    let config_query = format!("SELECT * FROM {} WHERE id = ?1", config_table);
    let mut stmt_config = conn
        .prepare(&config_query)
        .map_err(|e| format!("Chyba přípravy dotazu na {}: {}", config_table, e))?;
    let config_columns: Vec<String> = stmt_config
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let config: serde_json::Value = stmt_config
        .query_row(rusqlite::params![config_id], |row| {
            Ok(row_to_json_with_columns(row, &config_columns))
        })
        .map_err(|e| format!("Chyba načítání z tabulky {}: {}", config_table, e))?;

    // Sestavit výsledný JSON objekt se všemi získanými daty.
    let mut result = serde_json::Map::new();
    result.insert("copy_process".into(), copy_process);
    result.insert("log_record".into(), log_record);
    result.insert("config_record".into(), config);

    Ok(serde_json::Value::Object(result))
}
