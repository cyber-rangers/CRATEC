use crate::db::DB_CONN;
use crate::disk_utils::{get_mountpoint_for_interface, get_total_blocks, get_block_size};  // Přidáno
use crate::led::LED_CONTROLLER;
use crate::logger::{log_debug, log_error, log_warn};
use crate::websocket;
use chrono::{Local, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;
use tauri_plugin_shell::ShellExt;

use tauri_plugin_shell::process::CommandEvent;

/// Parametry pro dcfldd (obdoba EwfParams).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DdParams {
    pub case_number: String,
    pub description: String,
    pub investigator_name: String,
    pub evidence_number: String,
    pub notes: String,
    pub offset: i64,
    pub limit: i64,
}

/// Jednoduchá konfigurace dcfldd, analogicky k EwfConfig.
#[derive(Debug)]
struct DcflddConfig {
    pub confname: String,
    // Další pole dle potřeby...
}

/// Struktura pro frontendu zasílané výstupy.
#[derive(Serialize)]
struct WsProcessOutput {
    #[serde(rename = "type")]
    msg_type: String,
    id: i64,
    output: String,
}

/// Struktura s počátečními údaji o procesu pro UI.
#[derive(Serialize)]
struct WsProcessUpdate {
    #[serde(rename = "type")]
    msg_type: String,
    id: i64,
    start_datetime: String,
    end_datetime: Option<String>,
    status: String,
    triggered_by_ewf: bool,
    triggered_by_dd: bool,
    speed: f64,
    source_disk: String,
    destination_disks: Vec<String>,
    progress_perc: u8,
    progress_time: u64,
    out_log: Vec<String>,
}

/// Průběžné aktualizace procesu.
#[derive(Serialize)]
struct WsProcessProgress {
    #[serde(rename = "type")]
    msg_type: String,
    id: i64,
    progress_perc: Option<u8>,
    progress_time: Option<u64>,
    speed: Option<f64>,
}

/// Oznámení dokončení procesu.
#[derive(Serialize)]
struct WsProcessDone {
    #[serde(rename = "type")]
    msg_type: String,
    id: i64,
    status: String,
    end_datetime: String,
}

fn execute_with_retry<T, F>(operation_name: &str, mut f: F, max_retries: usize) -> Result<T, String>
where
    F: FnMut() -> Result<T, String>,
{
    let mut retries = 0;
    let mut last_error = String::new();
    while retries < max_retries {
        match f() {
            Ok(result) => {
                return Ok(result);
            }
            Err(e) => {
                retries += 1;
                last_error = e;
            }
        }
    }
    Err(format!(
        "({}) Failed after {} retries. Last error: {}",
        operation_name, max_retries, last_error
    ))
}

// Pomocná funkce pro odstranění prefixu "/dev/disk/by-path/"
fn strip_dev_prefix(full_path: &str) -> String {
    full_path
        .trim_start_matches("/dev/disk/by-path/")
        .to_string()
}

// Pomocná funkce pro vytvoření složky case_number/evidence_number na cílovém disku
fn prepare_evidence_dir(base: &str, case_number: &str, evidence_number: &str) -> Result<String, String> {
    let case_dir = Path::new(base).join(case_number);
    let evidence_dir = case_dir.join(evidence_number);

    if !case_dir.exists() {
        fs::create_dir(&case_dir).map_err(|e| format!("Failed to create case dir: {}", e))?;
    }
    if !evidence_dir.exists() {
        fs::create_dir(&evidence_dir).map_err(|e| format!("Failed to create evidence dir: {}", e))?;
    }
    Ok(evidence_dir.to_string_lossy().to_string())
}

/// Tauri příkaz pro spuštění dcfldd, analogicky k run_ewfacquire.
#[tauri::command(rename_all = "snake_case")]
pub async fn run_dcfldd(
    app_handle: tauri::AppHandle,
    config_id: i32,
    dd_params: DdParams,
    input_interface: String,
    output_interfaces: Vec<String>,
) -> Result<(), String> {
    LED_CONTROLLER.notify_process_start();

    // Připravíme stripped cesty pro DB lookup
    let input_raw = strip_dev_prefix(&input_interface);
    let output_raws: Vec<String> = output_interfaces.iter().map(|p| strip_dev_prefix(p)).collect();

    // Vstupní zařízení
    let actual_input_device = format!("/dev/disk/by-path/{}", input_interface);

    if output_interfaces.is_empty() {
        return Err("(run_dcfldd) No output disks provided!".to_string());
    }

    // První výstupní disk (přidáváme by-path)
    let first_output = format!("/dev/disk/by-path/{}", output_interfaces[0]);
    let actual_output_mount = match get_mountpoint_for_interface(&first_output) {
        Some(m) => m,
        None => {
            let err = format!(
                "(run_dcfldd) Nelze najít mountpoint pro {}",
                first_output
            );
            log_error(&err);
            return Err(err);
        }
    };

    // Druhý výstupní disk, pokud existuje (přidáváme by-path)
    let second_output_mount = if output_interfaces.len() > 1 {
        let second_output = format!("/dev/disk/by-path/{}", output_interfaces[1]);
        match get_mountpoint_for_interface(&second_output) {
            Some(m) => Some(m),
            None => {
                let err = format!(
                    "(run_dcfldd) Nelze najít mountpoint pro {}",
                    second_output
                );
                log_error(&err);
                return Err(err);
            }
        }
    } else {
        None
    };

    let dd_params_db = dd_params.clone();

    // DB lookup na ID disků podle interface_path a INSERT do copy_log_dd
    #[derive(Debug)]
    struct DdConfigComplete {
        confname: String,
        format: String,
        limit_mode: String,
        offset: String,
        hash_types: String,
        hashwindow: String,
        split: String,
        vf: bool,
        diffwr: bool,
        notes: String,
    }

    let (config, process_id, copy_log_id) = tauri::async_runtime::spawn_blocking(
        move || -> Result<(DdConfigComplete, i64, i64), String> {
            let mut pooled_conn = execute_with_retry(
                "Get connection from DB_POOL",
                || {
                    crate::db::DB_POOL
                        .get_connection()
                        .map_err(|e| format!("(DB_POOL) Nelze získat spojení: {}", e))
                },
                10,
            )?;

            let conn = pooled_conn.connection();
            let _ = conn.pragma_update(None, "journal_mode", &"DELETE");
            let _ = conn.busy_timeout(std::time::Duration::from_secs(120));

            // Načtení konfigurace
            let config = {
                let mut stmt = conn
                    .prepare(
                        "SELECT confname, format, limit_mode, offset, hash_types, 
                         hashwindow, split, vf, diffwr, notes
                         FROM dd_config
                         WHERE id = ?1 AND active = 1",
                    )
                    .map_err(|e| format!("(DB) Chyba při přípravě SQL dotazu: {}", e))?;

                stmt.query_row([config_id], |row| {
                    Ok(DdConfigComplete {
                        confname: row.get(0)?,
                        format: row.get(1)?,
                        limit_mode: row.get(2)?,
                        offset: row.get(3)?,
                        hash_types: row.get(4)?,
                        hashwindow: row.get(5)?,
                        split: row.get(6)?,
                        vf: row.get::<_, i32>(7)? != 0,
                        diffwr: row.get::<_, i32>(8)? != 0,
                        notes: row.get(9)?,
                    })
                })
                .map_err(|e| format!("(DB) Chyba při získávání konfigurace: {}", e))?
            };

            // Najdi ID source disku v tabulce `interfaces`, ve sloupci `interface_path`
            let source_disk_id: i64 = conn
                .query_row(
                    "SELECT id FROM interface WHERE interface_path = ?1 LIMIT 1",
                    [input_raw.as_str()],
                    |row| row.get(0),
                )
                .map_err(|_| {
                    format!(
                        "(DB) Nepodařilo se najít source disk v tabulce interfaces: {}",
                        input_raw
                    )
                })?;

            // První output
            let first_output_id: i64 = conn
                .query_row(
                    "SELECT id FROM interface WHERE interface_path = ?1 LIMIT 1",
                    [output_raws[0].as_str()],
                    |row| row.get(0),
                )
                .map_err(|_| {
                    format!(
                        "(DB) Nepodařilo se najít first_output disk v tabulce interfaces: {}",
                        output_raws[0]
                    )
                })?;

            // (pokud máte druhý výstup)
            let second_output_id: Option<i64> = if output_raws.len() > 1 {
                Some(
                    conn.query_row(
                        "SELECT id FROM interface WHERE interface_path = ?1 LIMIT 1",
                        [output_raws[1].as_str()],
                        // explicitně načteme i64
                        |row| row.get::<_, i64>(0),
                    )
                    .map_err(|_| format!(
                        "(DB) Nepodařilo se najít second_output disk v tabulce interfaces: {}",
                        output_raws[1]
                    ))?,
                )
            } else {
                None
            };

            // Vlož do copy_log_dd se správnými ID disků
            let tx = conn
                .transaction()
                .map_err(|e| format!("(DB) Nelze zahájit transakci: {}", e))?;

            tx.execute(
                "INSERT INTO copy_log_dd (
                    config_id, source, case_number, description, investigator_name, 
                    evidence_number, notes, offset, limit_value, source_disk_id, 
                    dest_disk_id, second_dest_disk_id, start_datetime
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, DATETIME('now'))",
                rusqlite::params![
                    config_id,
                    "dcfldd",
                    dd_params_db.case_number.replace("'", "''"),
                    dd_params_db.description.replace("'", "''"),
                    dd_params_db.investigator_name.replace("'", "''"),
                    dd_params_db.evidence_number.replace("'", "''"),
                    dd_params_db.notes.replace("'", "''"),
                    dd_params_db.offset,
                    dd_params_db.limit,
                    source_disk_id,
                    first_output_id,
                    second_output_id  
                ],
            )
            .map_err(|e| format!("(DB) Chyba při insertu copy_log_dd: {}", e))?;

            let copy_log_id = tx.last_insert_rowid();

            // Ulož do copy_process
            tx.execute(
                "INSERT INTO copy_process (triggered_by_dd) VALUES (?1)",
                params![copy_log_id],
            )
            .map_err(|e| format!("(DB) Chyba při zápisu do copy_process: {}", e))?;

            let pid = tx.last_insert_rowid();
            tx.commit()
                .map_err(|e| format!("(DB) Chyba při potvrzení transakce: {}", e))?;

            Ok((config, pid, copy_log_id))
        },
    )
    .await
    .map_err(|e| format!("(async) DB thread error: {}", e))??;

    // Prepare destination disks for frontend notification
    let mut destination_disks = vec![actual_output_mount.clone()];
    if let Some(second_mount) = &second_output_mount {
        destination_disks.push(second_mount.clone());
    }

    // Notify frontend about process start
    {
        let ws_update = WsProcessUpdate {
            msg_type: "ProcessFull".to_string(),
            id: process_id,
            start_datetime: Utc::now().to_rfc3339(),
            end_datetime: None,
            status: "running".to_string(),
            triggered_by_ewf: false,
            triggered_by_dd: true,
            speed: 0.0,
            source_disk: actual_input_device.clone(),
            destination_disks,
            progress_perc: 0,
            progress_time: 0,
            out_log: Vec::new(),
        };
        let msg = serde_json::to_string(&ws_update).map_err(|e| e.to_string())?;
        websocket::broadcast_message(&msg).await;
    }

    // Build dcfldd command arguments based on config and params
    let mut args_exec: Vec<String> = Vec::new();
    let mut args_print: Vec<String> = Vec::new();

    fn push_key_val(exec: &mut Vec<String>, print: &mut Vec<String>, key: &str, val: &str) {
        let kv = format!("{}={}", key, val);
        exec.push(kv.clone());
        print.push(kv);
    }

    // Input device
    push_key_val(&mut args_exec, &mut args_print, "if", &actual_input_device);

    let case_number = dd_params.case_number.trim();
    let evidence_number = dd_params.evidence_number.trim();

    // První destination disk
    let evidence_dir_1 = prepare_evidence_dir(&actual_output_mount, case_number, evidence_number)?;

    // Pokud je druhý disk, připrav i pro něj
    let evidence_dir_2 = if let Some(ref mount2) = second_output_mount {
        Some(prepare_evidence_dir(mount2, case_number, evidence_number)?)
    } else {
        None
    };

    // First output file
    let primary_out = format!("{}/{}.img", evidence_dir_1, evidence_number);
    push_key_val(&mut args_exec, &mut args_print, "of", &primary_out);

    // Second output if available
    if let Some(evidence_dir_2) = evidence_dir_2 {
        let second_out = format!("{}/{}.img", evidence_dir_2, evidence_number);
        push_key_val(&mut args_exec, &mut args_print, "of2", &second_out);
    }

    // Block size (format from config)
    let block_size = if config.format != "auto" {
        config.format.parse::<u64>().unwrap_or(512)
    } else {
        match get_block_size(&actual_input_device) {
            Ok(size) => size,
            Err(e) => {
                log_warn(&format!("Could not auto-detect block size, defaulting to 512: {}", e));
                512
            }
        }
    };

    push_key_val(&mut args_exec, &mut args_print, "bs", &block_size.to_string());

    // Offset handling (skip parameter)
    let offset_value = if config.offset == "ask" {
        dd_params.offset.to_string()
    } else {
        config.offset.to_string()
    };

    if offset_value != "0" && !offset_value.is_empty() {
        push_key_val(&mut args_exec, &mut args_print, "skip", &offset_value);
    }

    // Limit handling (limit parameter)
    if config.limit_mode == "ask" && dd_params.limit > 0 {
        push_key_val(
            &mut args_exec,
            &mut args_print,
            "limit",
            &dd_params.limit.to_string(),
        );
    } else if config.limit_mode != "whole" {
        // Apply some default or config-specific limit if not "whole"
    }

    // Hash calculation
    if !config.hash_types.is_empty() {
        // Clean up hash types string (remove brackets if present)
        let hash_types = config
            .hash_types
            .replace(['[', ']', '"', '\''], "")
            .trim()
            .to_string();

        if !hash_types.is_empty() {
            // Set hash type
            push_key_val(&mut args_exec, &mut args_print, "hash", &hash_types);

            // Set hash window size if specified
            if config.hashwindow != "whole" {
                push_key_val(
                    &mut args_exec,
                    &mut args_print,
                    "hashwindow",
                    &config.hashwindow,
                );
            }

            // Create hash log file
            let hash_log_path = format!("{}/hash.log", evidence_dir_1);
            push_key_val(&mut args_exec, &mut args_print, "hashlog", &hash_log_path);
        }
    }

    // Split file option if specified
    if config.split != "whole" {
        push_key_val(&mut args_exec, &mut args_print, "split", &config.split);
    }

    // Verify input option
    if config.vf {
        // This would need verification file path logic
        // push_key_val(&mut args_exec, &mut args_print, "vf", &verification_file_path);
    }

    // Diff write option
    if config.diffwr {
        push_key_val(&mut args_exec, &mut args_print, "diffwr", "on");
    }

    // Status output for progress tracking
    push_key_val(&mut args_exec, &mut args_print, "status", "on");
    push_key_val(&mut args_exec, &mut args_print, "statusinterval", "150000");

    // Create error log
    let error_log_path = format!("{}/error.log", evidence_dir_1);
    push_key_val(&mut args_exec, &mut args_print, "errlog", &error_log_path);

    // Print the command with more detail
    println!("\n=== SPOUŠTÍM DCFLDD PŘÍKAZ ===");
    let cmd_print = format!("sudo dcfldd {}", args_print.join(" "));
    println!("{}", cmd_print);
    println!("================================\n");

    let total_blocks = match get_total_blocks(&actual_input_device) {
        Ok(tb) => tb,
        Err(e) => {
            log_warn(&format!("Cannot get total blocks, using fallback: {}", e));
            5_000_000 // fallback if lsblk fails
        }
    };
    println!("Test: Total blocks calculated: {}", total_blocks);

    // Execute the command
    let shell = app_handle.shell();
    let (mut rx, _child) = shell
        .command("sudo")
        .args(["dcfldd"])
        .args(&args_exec)
        .spawn()
        .map_err(|e| format!("(Command) Failed to spawn dcfldd: {}", e))?;

    // Variables to store hash values
    let mut md5_hash: Option<String> = None;
    let mut sha1_hash: Option<String> = None;
    let mut sha256_hash: Option<String> = None;

    // Process command output
    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(line) | CommandEvent::Stderr(line) => {
                let line_str = String::from_utf8_lossy(&line).to_string();
                let trimmed = line_str.trim(); // Odstraní bílé znaky na začátku a na konci

                // Odesíláme zprávu ProcessOutput s původním (nebo trimovaným) řetězcem
                let output_msg = WsProcessOutput {
                    msg_type: "ProcessOutput".to_string(),
                    id: process_id,
                    output: line_str.clone(),
                };
                let json_output = serde_json::to_string(&output_msg).unwrap();
                websocket::broadcast_message(&json_output).await;

                // Log do databáze
                {
                    let conn = crate::db::create_new_connection()
                        .map_err(|e| format!("(DB output) Failed to create connection: {}", e))?;
                    conn.execute(
                        "INSERT INTO process_log_lines (process_id, line_content, line_number)
                         VALUES (?1, ?2, COALESCE((SELECT MAX(line_number) FROM process_log_lines WHERE process_id=?1),0)+1)",
                        params![process_id, line_str],
                    ).map_err(|e| format!("(DB output) Error writing log: {}", e))?;
                }

                // Použijeme trimovanou verzi výsledku pro regulární výrazy
                lazy_static! {
                    static ref PROGRESS_REGEX: Regex = Regex::new(r"(\d+)% done, .*").unwrap();
                    static ref MD5_REGEX: Regex = Regex::new(r"MD5: ([a-fA-F0-9]{32})").unwrap();
                    static ref SHA1_REGEX: Regex = Regex::new(r"SHA1: ([a-fA-F0-9]{40})").unwrap();
                    static ref SHA256_REGEX: Regex = Regex::new(r"SHA256: ([a-fA-F0-9]{64})").unwrap();
                    static ref BLOCKS_REGEX: Regex = Regex::new(r"^(\d+)\ blocks\s+\((\d+)Mb\)\ written\.?$").unwrap();
                }

                // Extrakce procentuálního postupu pomocí trimované verze
                if let Some(caps) = PROGRESS_REGEX.captures(trimmed) {
                    if let Ok(perc) = caps[1].parse::<u8>() {
                        let update = WsProcessProgress {
                            msg_type: "ProcessProgress".to_string(),
                            id: process_id,
                            progress_perc: Some(perc),
                            progress_time: None,
                            speed: None,
                        };
                        let json = serde_json::to_string(&update).unwrap();
                        websocket::broadcast_message(&json).await;
                    }
                }

                // Extrakce hash hodnot
                if let Some(caps) = MD5_REGEX.captures(trimmed) {
                    md5_hash = Some(caps[1].to_string());
                    println!("Found MD5 hash: {}", caps[1].to_string());
                }
                if let Some(caps) = SHA1_REGEX.captures(trimmed) {
                    sha1_hash = Some(caps[1].to_string());
                    println!("Found SHA1 hash: {}", caps[1].to_string());
                }
                if let Some(caps) = SHA256_REGEX.captures(trimmed) {
                    sha256_hash = Some(caps[1].to_string());
                    println!("Found SHA256 hash: {}", caps[1].to_string());
                }

                // Zpracování progresního výstupu pomocí trimované verze
                if let Some(caps) = BLOCKS_REGEX.captures(trimmed) {
                    let blocks_written: u64 = caps[1].parse().unwrap_or(0);
                    let perc_complete = if total_blocks > 0 {
                        (blocks_written as f64 / total_blocks as f64) * 100.0
                    } else {
                        0.0
                    };

                    // Výpočet rychlosti v MiB/s
                    static mut LAST_BLOCKS: u64 = 0;
                    static mut LAST_TIME: Option<std::time::Instant> = None;
                    let speed_estimate = unsafe {
                        let now = std::time::Instant::now();
                        let spd = if let Some(prev_time) = LAST_TIME {
                            let elapsed = now.duration_since(prev_time).as_secs_f64();
                            if elapsed > 0.0 {
                                (blocks_written.saturating_sub(LAST_BLOCKS) as f64 * block_size as f64)
                                    / (elapsed * 1024.0 * 1024.0)
                            } else {
                                0.0
                            }
                        } else {
                            0.0
                        };
                        LAST_BLOCKS = blocks_written;
                        LAST_TIME = Some(now);
                        spd
                    };

                    // Výpočet ETA (zbývající čas) v sekundách
                    let remaining_bytes = if blocks_written < total_blocks {
                        (total_blocks - blocks_written) as f64 * block_size as f64
                    } else {
                        0.0
                    };
                    let remaining_mib = remaining_bytes / (1024.0 * 1024.0);
                    let eta_seconds = if speed_estimate > 0.0 {
                        remaining_mib / speed_estimate
                    } else {
                        0.0
                    };

                    let progress_update = WsProcessProgress {
                        msg_type: "ProcessProgress".to_string(),
                        id: process_id,
                        progress_perc: Some(perc_complete as u8),
                        progress_time: Some(eta_seconds as u64), // zbývající čas v sekundách
                        speed: Some(speed_estimate), // rychlost v MiB/s
                    };
                    let json = serde_json::to_string(&progress_update).unwrap();
                    websocket::broadcast_message(&json).await;
                }
            }
            CommandEvent::Terminated(exit_code) => {
                // ... (nezměněný kód ukončení procesu)
                LED_CONTROLLER.notify_process_end();

                let final_status = if exit_code.code.unwrap_or(-1) == 0 {
                    "done"
                } else {
                    "error"
                };

                let end_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                let end_time_for_db = end_time.clone();

                tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
                    let conn = crate::db::create_new_connection()
                        .map_err(|e| format!("(DB) Error creating final connection: {}", e))?;
                    let mut update_sql = format!(
                        "UPDATE copy_log_dd SET status = '{}', end_datetime = '{}' ",
                        final_status, end_time_for_db
                    );
                    if let Some(hash) = &md5_hash {
                        update_sql.push_str(&format!(", md5_hash = '{}' ", hash));
                    }
                    if let Some(hash) = &sha1_hash {
                        update_sql.push_str(&format!(", sha1_hash = '{}' ", hash));
                    }
                    if let Some(hash) = &sha256_hash {
                        update_sql.push_str(&format!(", sha256_hash = '{}' ", hash));
                    }
                    update_sql.push_str(&format!("WHERE id = {}", copy_log_id));

                    conn.execute(&update_sql, [])
                        .map_err(|e| format!("Error updating copy_log_dd: {}", e))?;

                    conn.execute(
                        "UPDATE copy_process SET status = ?, end_datetime = ? WHERE id = ?",
                        params![final_status, end_time_for_db, process_id],
                    )
                    .map_err(|e| format!("Error updating copy_process: {}", e))?;

                    Ok(())
                })
                .await
                .map_err(|e| e.to_string())??;

                let ws_done = WsProcessDone {
                    msg_type: "ProcessDone".to_string(),
                    id: process_id,
                    status: final_status.to_string(),
                    end_datetime: end_time,
                };
                let done_msg = serde_json::to_string(&ws_done).unwrap();
                websocket::broadcast_message(&done_msg).await;

                break;
            }
            _ => (),
        }
    }

    Ok(())
}
