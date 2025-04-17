use crate::db::DB_POOL;
use crate::disk_utils;
use chrono::Local;
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

/// Vygeneruje report pomocí mock dat – původní implementace.
pub fn mock_data_report() -> Result<(), String> {
    println!("📄 Vkládám šablonu přímo z kódu...");

    let mut context = Context::new();

    // Nastavení proměnných pro šablonu
    context.insert("software_hash", "75f1c14d734ea09147330fae210faa54");
    context.insert("build_date", "Jul 08, 2024 13:38:46 PDT");
    context.insert("serial_number", "117495");
    context.insert("time_local", "12:17:32 (CEST +0200)");
    context.insert("date", "Jul 08, 2024");
    context.insert("mode", "DriveToFile");
    context.insert("method", "DDCapture");
    context.insert("hash_type", "SHA-1");
    context.insert("image_path", "/var/reports/.../Kingston480GB\\_SATA");
    context.insert("lba_count", &937703088u64);
    context.insert("sector_size", &512u64);
    context.insert("segment_size", "WholeDisk");
    context.insert("compression", "None");
    context.insert("hash_enabled", &true);
    context.insert("verify_hash", &false);
    context.insert("unlock_hpa", &true);
    context.insert("unlock_dco", &true);
    context.insert("granularity", "SUCCESS");
    context.insert("result", "SUCCESS");
    context.insert("duration", "00:47:20");
    context.insert("time_complete", "12:14:52");
    context.insert("hash_lba_count", &937703088u64);
    context.insert("hash_sector_size", &512u64);
    context.insert("hash_primary_type", "SHA-1");
    context.insert("hash_primary", "75f1c14d734ea09147330fae210faa54");
    context.insert("hash_secondary_type", "MD5");
    context.insert("hash_secondary", "abcdef1234567890abcdef1234567890");
    context.insert("case_file", "CaseFile\\_001");
    context.insert("case_id", "CASE-2024-001");
    context.insert("examiner", "John Doe");
    context.insert("notes", "This case involves encrypted drives.");
    context.insert("segment_uid", "SEG123456");
    context.insert("segment_path", "/segments/seg123456");
    context.insert("segment_fs", "NTFS");
    context.insert("segment_serial", "SEG-SERIAL-001");
    context.insert("segment_file", "segment\\_file.img");
    context.insert("segment_hash", "seg\\_hash\\_abc123");

    // Ukázka disků
    let drives = vec![
        {
            let mut map = std::collections::HashMap::new();
            map.insert("bay", "1");
            map.insert("role", "Primary");
            map.insert("serial", "DRIVE123456");
            map.insert("model", "WD Blue");
            map.insert("raid", "RAID0");
            map.insert("fs", "EXT4");
            map.insert("cipher", "None");
            map
        },
        {
            let mut map = std::collections::HashMap::new();
            map.insert("bay", "2");
            map.insert("role", "Secondary");
            map.insert("serial", "DRIVE654321");
            map.insert("model", "Seagate Barracuda");
            map.insert("raid", "RAID1");
            map.insert("fs", "EXT4");
            map.insert("cipher", "AES-256");
            map
        },
    ];
    context.insert("drives", &drives);

    // Kapacity
    let capacities = vec![
        {
            let mut map = std::collections::HashMap::new();
            map.insert("bay", "1");
            map.insert("serial", "DRIVE123456");
            map.insert("capacity", "500");
            map
        },
        {
            let mut map = std::collections::HashMap::new();
            map.insert("bay", "2");
            map.insert("serial", "DRIVE654321");
            map.insert("capacity", "1000");
            map
        },
    ];
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

    // Partitions
    let partitions = vec![
        {
            let mut map = std::collections::HashMap::new();
            map.insert("index", "1");
            map.insert("fs", "NTFS");
            map.insert("start", "2048");
            map.insert("end", "409600");
            map.insert("size", "200MB");
            map.insert("encryption", "Yes");
            map.insert("decrypted", "No");
            map
        },
        {
            let mut map = std::collections::HashMap::new();
            map.insert("index", "2");
            map.insert("fs", "FAT32");
            map.insert("start", "409601");
            map.insert("end", "819200");
            map.insert("size", "200MB");
            map.insert("encryption", "No");
            map.insert("decrypted", "Yes");
            map
        },
    ];
    context.insert("partitions", &partitions);

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
            // Funkce pro doplnění paths
            fn full_disk_path(disk_id: &str) -> String {
                if disk_id.starts_with("/dev/disk/by-path/") {
                    disk_id.to_string()
                } else {
                    format!("/dev/disk/by-path/{}", disk_id)
                }
            }

            // Získání source_disk
            let source_disk = if let Some(source_disk_id) =
                log_obj.get("source_disk_id").and_then(|v| v.as_str())
            {
                if !source_disk_id.trim().is_empty() {
                    let path = full_disk_path(source_disk_id);
                    let disk_info = disk_utils::get_disk_info(&path)?;
                    serde_json::to_value(disk_info)
                        .map_err(|e| format!("Chyba při serializaci disk info (source): {}", e))?
                } else {
                    serde_json::Value::String(String::new())
                }
            } else {
                serde_json::Value::String(String::new())
            };
            report.insert("source_disk".into(), source_disk);

            // Získání dest_disk
            let dest_disk =
                if let Some(dest_disk_id) = log_obj.get("dest_disk_id").and_then(|v| v.as_str()) {
                    if !dest_disk_id.trim().is_empty() {
                        let path = full_disk_path(dest_disk_id);
                        let disk_info = disk_utils::get_disk_info(&path)?;
                        serde_json::to_value(disk_info)
                            .map_err(|e| format!("Chyba při serializaci disk info (dest): {}", e))?
                    } else {
                        serde_json::Value::String(String::new())
                    }
                } else {
                    serde_json::Value::String(String::new())
                };
            report.insert("dest_disk".into(), dest_disk);

            // Získání second_dest_disk (vždy vloženo, může být prázdné)
            let second_dest_disk = if let Some(second_dest_disk_id) =
                log_obj.get("second_dest_disk_id").and_then(|v| v.as_str())
            {
                if !second_dest_disk_id.trim().is_empty() {
                    let path = full_disk_path(second_dest_disk_id);
                    let disk_info = disk_utils::get_disk_info(&path)?;
                    serde_json::to_value(disk_info)
                        .map_err(|e| format!("Chyba při serializaci disk info (second): {}", e))?
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

    context.insert("segment_size", "WholeDisk");

    // místo pevného "compression"
    let compression = report
        .get("config_record")
        .and_then(|cfg| cfg.get("compression_level"))
        .and_then(Value::as_str)
        .unwrap_or("None");
    context.insert("compression", compression);

    context.insert("hash_enabled", &true);
    context.insert("verify_hash", &false);
    context.insert("unlock_hpa", &true);
    context.insert("unlock_dco", &true);
    context.insert("granularity", "SUCCESS");

    // místo pevného "result"
    let result = report
        .get("copy_process")
        .and_then(|cp| cp.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("UNKNOWN");
    context.insert("result", result);

    context.insert("duration", "00:47:20");

    // místo pevného "time_complete"
    let time_complete = report
        .get("log_record")
        .and_then(|lr| lr.get("end_datetime"))
        .and_then(Value::as_str)
        .unwrap_or("");
    context.insert("time_complete", time_complete);

    context.insert("hash_lba_count", &937703088u64);
    context.insert("hash_sector_size", &512u64);
    context.insert("hash_primary_type", "SHA-1");
    context.insert("hash_primary", "75f1c14d734ea09147330fae210faa54");
    context.insert("hash_secondary_type", "MD5");
    context.insert("hash_secondary", "abcdef1234567890abcdef1234567890");
    context.insert("case_file", "CaseFile\\_001");

    // místo pevného "case_id"
    let case_id = report
        .get("log_record")
        .and_then(|lr| lr.get("case_number"))
        .and_then(Value::as_str)
        .unwrap_or("");
    context.insert("case_id", case_id);

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

    // Ukázka disků
    let drives = vec![
        {
            let mut map = std::collections::HashMap::new();
            map.insert("bay", "1");
            map.insert("role", "Primary");
            map.insert("serial", "DRIVE123456");
            map.insert("model", "WD Blue");
            map.insert("raid", "RAID0");
            map.insert("fs", "EXT4");
            map.insert("cipher", "None");
            map
        },
        {
            let mut map = std::collections::HashMap::new();
            map.insert("bay", "2");
            map.insert("role", "Secondary");
            map.insert("serial", "DRIVE654321");
            map.insert("model", "Seagate Barracuda");
            map.insert("raid", "RAID1");
            map.insert("fs", "EXT4");
            map.insert("cipher", "AES-256");
            map
        },
    ];
    context.insert("drives", &drives);

    // Kapacity
    let capacities = vec![
        {
            let mut map = std::collections::HashMap::new();
            map.insert("bay", "1");
            map.insert("serial", "DRIVE123456");
            map.insert("capacity", "500");
            map
        },
        {
            let mut map = std::collections::HashMap::new();
            map.insert("bay", "2");
            map.insert("serial", "DRIVE654321");
            map.insert("capacity", "1000");
            map
        },
    ];
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

    // Partitions
    let partitions = vec![
        {
            let mut map = std::collections::HashMap::new();
            map.insert("index", "1");
            map.insert("fs", "NTFS");
            map.insert("start", "2048");
            map.insert("end", "409600");
            map.insert("size", "200MB");
            map.insert("encryption", "Yes");
            map.insert("decrypted", "No");
            map
        },
        {
            let mut map = std::collections::HashMap::new();
            map.insert("index", "2");
            map.insert("fs", "FAT32");
            map.insert("start", "409601");
            map.insert("end", "819200");
            map.insert("size", "200MB");
            map.insert("encryption", "No");
            map.insert("decrypted", "Yes");
            map
        },
    ];
    context.insert("partitions", &partitions);

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
