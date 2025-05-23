use crate::disk_utils::get_mountpoint_for_interface;
use crate::led::LED_CONTROLLER;
use crate::logger::{log_debug, log_error};
use crate::report::generate_report_ewfacquire;
use crate::websocket;
use chrono::Utc;
use lazy_static::lazy_static;
use regex::Regex;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

fn execute_with_retry<T, F>(operation_name: &str, mut f: F, max_retries: usize) -> Result<T, String>
where
    F: FnMut() -> Result<T, String>,
{
    let mut retries = 0;
    let mut last_error = String::new();

    while retries < max_retries {
        match f() {
            Ok(result) => {
                if retries > 0 {
                    log_debug(&format!(
                        "({}) Succeeded after {} retries",
                        operation_name, retries
                    ));
                }
                return Ok(result);
            }
            Err(e) => {
                last_error = e.to_string();
                retries += 1;
                if last_error.contains("database is locked")
                    || last_error.contains("locked")
                    || last_error.contains("busy")
                    || last_error.contains("cannot start a transaction")
                {
                    let wait_ms = 500 * (1 << retries);
                    log_debug(&format!(
                        "({}) Database access issue, retry {}/{} after {}ms: {}",
                        operation_name, retries, max_retries, wait_ms, last_error
                    ));
                    println!(
                        "({}) Database lock detected, waiting {}ms before retry {}/{}",
                        operation_name, wait_ms, retries, max_retries
                    );
                    thread::sleep(Duration::from_millis(wait_ms));
                } else {
                    log_error(&format!(
                        "({}) Error not related to locking, not retrying: {}",
                        operation_name, last_error
                    ));
                    return Err(last_error);
                }
            }
        }
    }

    log_error(&format!(
        "({}) Failed after {} retries. Last error: {}",
        operation_name, max_retries, last_error
    ));
    Err(last_error)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EwfParams {
    pub case_number: String,
    pub description: String,
    pub investigator_name: String,
    pub evidence_number: String,
    pub notes: String,
    pub offset: u64,
    pub bytes_to_read: u64,
}

#[derive(Debug)]
struct EwfConfig {
    confname: String,
    codepage: String,
    sectors_per_read: String,
    bytes_to_read: String,
    compression_method: String,
    compression_level: String,
    hash_types: String,
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
}

#[derive(Serialize)]
struct WsProcessOutput {
    #[serde(rename = "type")]
    msg_type: String,
    id: i64,
    output: String,
}

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

#[derive(Serialize)]
struct WsProcessProgress {
    #[serde(rename = "type")]
    msg_type: String,
    id: i64,
    progress_perc: Option<u8>,
    progress_time: Option<u64>,
    speed: Option<f64>,
}

#[derive(Serialize)]
struct WsProcessDone {
    #[serde(rename = "type")]
    msg_type: String,
    id: i64,
    status: String,
    end_datetime: String,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn run_ewfacquire(
    app_handle: tauri::AppHandle,
    config_id: i32,
    ewf_params: EwfParams,
    input_interface: String,
    output_interfaces: Vec<String>,
) -> Result<(), String> {
    LED_CONTROLLER.notify_process_start();

    let actual_input_device = format!("/dev/disk/by-path/{}", strip_dev_prefix(&input_interface));

    if output_interfaces.is_empty() {
        log_error("(run_ewfacquire) No output disks provided!");
        return Err("(run_ewfacquire) No output disks provided!".to_string());
    }
    let first_output = format!(
        "/dev/disk/by-path/{}",
        strip_dev_prefix(&output_interfaces[0])
    );
    let actual_output_mount = match get_mountpoint_for_interface(&first_output) {
        Some(mounted) => mounted,
        None => {
            let err = format!(
                "(run_ewfacquire) Nepodařilo se najít mountpoint pro {}",
                first_output
            );
            log_error(&err);
            return Err(err);
        }
    };

    let second_output_mount = if output_interfaces.len() > 1 {
        let second_output = format!(
            "/dev/disk/by-path/{}",
            strip_dev_prefix(&output_interfaces[1])
        );
        match get_mountpoint_for_interface(&second_output) {
            Some(mounted) => Some(mounted),
            None => {
                let err = format!(
                    "(run_ewfacquire) Nepodařilo se najít mountpoint pro {}",
                    second_output
                );
                log_error(&err);
                return Err(err);
            }
        }
    } else {
        None
    };

    let ewf_params_db = ewf_params.clone();

    let input_raw = strip_dev_prefix(&input_interface);
    let first_output_raw = strip_dev_prefix(&output_interfaces[0]);
    let output_interfaces_raw = output_interfaces
        .iter()
        .map(|path| strip_dev_prefix(path))
        .collect::<Vec<_>>();

    let (config, process_id) =
        tauri::async_runtime::spawn_blocking(move || -> Result<(EwfConfig, i64), String> {
            let mut conn = execute_with_retry(
                "DB connection",
                || {
                    crate::db::create_new_connection()
                        .map_err(|e| format!("(DB) Error creating database connection: {}", e))
                },
                10,
            )?;

            let _ = conn.pragma_update(None, "journal_mode", &"DELETE");
            let _ = conn.busy_timeout(Duration::from_secs(120));
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| {
                    log_error(&format!("(DB) Chyba při zahájení transakce: {}", e));
                    format!("(DB) Chyba při zahájení transakce: {}", e)
                })?;

            let config = {
                let mut stmt = tx
                    .prepare(
                        "SELECT confname, codepage, sectors_per_read, bytes_to_read,
                         compression_method, compression_level, hash_types, ewf_format,
                         granularity_sectors, notes, offset, process_buffer_size,
                         bytes_per_sector, read_retry_count, swap_byte_pairs,
                         segment_size, zero_on_read_error, use_chunk_data
                         FROM ewf_config
                         WHERE id = ?1 AND active = 1",
                    )
                    .map_err(|e| {
                        log_error(&format!("(DB) Chyba při přípravě SQL dotazu: {}", e));
                        format!("(DB) Chyba při přípravě SQL dotazu: {}", e)
                    })?;

                stmt.query_row([config_id], |row| {
                    Ok(EwfConfig {
                        confname: row.get(0)?,
                        codepage: row.get(1)?,
                        sectors_per_read: row.get(2)?,
                        bytes_to_read: row.get(3)?,
                        compression_method: row.get(4)?,
                        compression_level: row.get(5)?,
                        hash_types: row.get(6)?,
                        ewf_format: row.get(7)?,
                        granularity_sectors: row.get(8)?,
                        notes: row.get(9)?,
                        offset: row.get(10)?,
                        process_buffer_size: row.get(11)?,
                        bytes_per_sector: row.get(12)?,
                        read_retry_count: row.get(13)?,
                        swap_byte_pairs: row.get(14)?,
                        segment_size: row.get(15)?,
                        zero_on_read_error: row.get(16)?,
                        use_chunk_data: row.get(17)?,
                    })
                })
                .map_err(|e| {
                    log_error(&format!("(DB) Chyba při získávání konfigurace: {}", e));
                    format!("(DB) Chyba při získávání konfigurace: {}", e)
                })?
            };

            let source_disk_id: i64 = tx
                .query_row(
                    "SELECT id FROM interface WHERE interface_path = ?1 LIMIT 1",
                    [input_raw.as_str()],
                    |row| row.get(0),
                )
                .map_err(|_| {
                    let msg = format!(
                        "(DB) Nepodařilo se najít source disk v tabulce interfaces: {}",
                        input_raw
                    );
                    log_error(&msg);
                    msg
                })?;

            let first_output_id: i64 = tx
                .query_row(
                    "SELECT id FROM interface WHERE interface_path = ?1 LIMIT 1",
                    [first_output_raw.as_str()],
                    |row| row.get(0),
                )
                .map_err(|_| {
                    let msg = format!(
                        "(DB) Nepodařilo se najít first_output disk v tabulce interfaces: {}",
                        first_output_raw
                    );
                    log_error(&msg);
                    msg
                })?;

            let second_output_id = if output_interfaces_raw.len() > 1 {
                let second_stripped = output_interfaces_raw[1].clone();
                Some(
                    tx.query_row(
                        "SELECT id FROM interface WHERE interface_path = ?1 LIMIT 1",
                        [second_stripped.as_str()],
                        |row| row.get(0),
                    )
                    .map_err(|_| {
                        let msg = format!(
                            "(DB) Nepodařilo se najít second_output disk v tabulce interfaces: {}",
                            second_stripped
                        );
                        log_error(&msg);
                        msg
                    })?,
                )
            } else {
                None
            };

            tx.execute(
                "INSERT INTO copy_log_ewf (
                    config_id,
                    case_number,
                    description,
                    investigator_name,
                    evidence_number,
                    source_disk_id,
                    dest_disk_id,
                    second_dest_disk_id,
                    notes,
                    offset,
                    bytes_to_read,
                    start_datetime
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, DATETIME('now'))",
                rusqlite::params![
                    config_id,
                    ewf_params_db.case_number.replace("'", "''"),
                    ewf_params_db.description.replace("'", "''"),
                    ewf_params_db.investigator_name.replace("'", "''"),
                    ewf_params_db.evidence_number.replace("'", "''"),
                    source_disk_id,
                    first_output_id,
                    second_output_id.unwrap_or(0),
                    ewf_params_db.notes.replace("'", "''"),
                    ewf_params_db.offset,
                    ewf_params_db.bytes_to_read
                ],
            )
            .map_err(|e| {
                log_error(&format!("(DB) Chyba při zápisu do copy_log_ewf: {}", e));
                format!("(DB) Chyba při zápisu do copy_log_ewf: {}", e)
            })?;

            let copy_log_id = tx.last_insert_rowid();

            let process_result = tx.execute(
                "INSERT INTO copy_process (triggered_by_ewf) VALUES (?1)",
                rusqlite::params![copy_log_id],
            );
            if let Err(e) = &process_result {
                if e.to_string().contains("locked") || e.to_string().contains("busy") {
                    thread::sleep(Duration::from_secs(5));
                    tx.execute(
                        "INSERT INTO copy_process (triggered_by_ewf) VALUES (?1)",
                        rusqlite::params![copy_log_id],
                    )
                    .map_err(|e2| {
                        log_error(&format!("(DB) Chyba při zápisu do copy_process: {}", e2));
                        format!("(DB) Chyba při zápisu do copy_process: {}", e2)
                    })?;
                } else {
                    log_error(&format!("(DB) Chyba při zápisu do copy_process: {}", e));
                    return Err(format!("(DB) Chyba při zápisu do copy_process: {}", e));
                }
            }

            let process_id = tx.last_insert_rowid();
            tx.commit().map_err(|e| {
                log_error(&format!("(DB) Chyba při potvrzení transakce: {}", e));
                format!("(DB) Chyba při potvrzení transakce: {}", e)
            })?;
            Ok((config, process_id))
        })
        .await
        .map_err(|e| format!("(async) Chyba při spawn_blocking: {}", e))??;

    let mut destination_disks = vec![actual_output_mount.clone()];
    if let Some(second_mount) = &second_output_mount {
        destination_disks.push(second_mount.clone());
    }
    {
        let ws_update = WsProcessUpdate {
            msg_type: "ProcessFull".to_string(),
            id: process_id,
            start_datetime: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            end_datetime: None,
            status: "running".to_string(),
            triggered_by_ewf: true,
            triggered_by_dd: false,
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

    let mut args_exec: Vec<String> = Vec::new();
    let mut args_print: Vec<String> = Vec::new();

    fn push_pair(exec: &mut Vec<String>, print: &mut Vec<String>, flag: &str, value: &str) {
        exec.push(flag.to_string());
        exec.push(value.to_string()); // BEZ uvozovek!
        print.push(flag.to_string());
        print.push(format!("\"{}\"", value)); // pro logování s uvozovkami
    }
    
    push_pair(&mut args_exec, &mut args_print, "-A", &config.codepage);
    push_pair(
        &mut args_exec,
        &mut args_print,
        "-b",
        &config.sectors_per_read,
    );
    if config.bytes_to_read != "whole" {
        push_pair(
            &mut args_exec,
            &mut args_print,
            "-B",
            &ewf_params.bytes_to_read.to_string(),
        );
    }
    let compression_settings =
        format!("{}:{}", config.compression_method, config.compression_level);
    push_pair(&mut args_exec, &mut args_print, "-c", &compression_settings);
    push_pair(
        &mut args_exec,
        &mut args_print,
        "-C",
        &ewf_params.case_number,
    );
    push_pair(
        &mut args_exec,
        &mut args_print,
        "-D",
        &ewf_params.description,
    );
    push_pair(
        &mut args_exec,
        &mut args_print,
        "-e",
        &ewf_params.investigator_name,
    );
    push_pair(
        &mut args_exec,
        &mut args_print,
        "-E",
        &ewf_params.evidence_number,
    );
    push_pair(&mut args_exec, &mut args_print, "-f", &config.ewf_format);
    let granularity = config.granularity_sectors.parse::<u32>().unwrap_or(2);
    push_pair(
        &mut args_exec,
        &mut args_print,
        "-g",
        &granularity.to_string(),
    );

    push_pair(&mut args_exec, &mut args_print, "-m", "fixed");
    push_pair(&mut args_exec, &mut args_print, "-M", "physical");
    if config.notes == "ask" {
        push_pair(&mut args_exec, &mut args_print, "-N", &ewf_params.notes);
    }
    if config.offset == "ask" {
        push_pair(
            &mut args_exec,
            &mut args_print,
            "-o",
            &ewf_params.offset.to_string(),
        );
    } else {
        push_pair(&mut args_exec, &mut args_print, "-o", &config.offset);
    }
    if config.process_buffer_size != "auto" {
        push_pair(
            &mut args_exec,
            &mut args_print,
            "-p",
            &config.process_buffer_size,
        );
    }
    if config.bytes_per_sector != "auto" {
        push_pair(
            &mut args_exec,
            &mut args_print,
            "-P",
            &config.bytes_per_sector,
        );
    }

    let mut retry_count = config.read_retry_count.parse::<u8>().unwrap_or(2);
    if retry_count < 1 || retry_count > 5 { // uprav rozsah podle ewfacquire
        retry_count = 2;
    }
    push_pair(&mut args_exec, &mut args_print, "-r", &retry_count.to_string());

    if config.swap_byte_pairs {
        args_exec.push("-s".to_string());
        args_print.push("-s".to_string());
    }
    push_pair(&mut args_exec, &mut args_print, "-S", &config.segment_size);

    if !config.hash_types.is_empty() && config.hash_types != "[]" {
        let hash_types = config
            .hash_types
            .replace(['[', ']', '"', '\''], "")
            .trim()
            .to_string();

        if !hash_types.is_empty() {
            push_pair(&mut args_exec, &mut args_print, "-d", &hash_types);
        }
    }

    fn prepare_evidence_dir(
        base: &str,
        case_number: &str,
        evidence_number: &str,
    ) -> Result<String, String> {
        let case_dir = Path::new(base).join(case_number);
        let evidence_dir = case_dir.join(evidence_number);

        if !case_dir.exists() {
            fs::create_dir(&case_dir).map_err(|e| format!("Failed to create case dir: {}", e))?;
        }
        if !evidence_dir.exists() {
            fs::create_dir(&evidence_dir)
                .map_err(|e| format!("Failed to create evidence dir: {}", e))?;
        }
        Ok(evidence_dir.to_string_lossy().to_string())
    }

    let case_number = ewf_params.case_number.trim();
    let evidence_number = ewf_params.evidence_number.trim();

    let evidence_dir_1 = prepare_evidence_dir(&actual_output_mount, case_number, evidence_number)?;

    let evidence_dir_2 = if let Some(second_mount) = &second_output_mount {
        Some(prepare_evidence_dir(
            second_mount,
            case_number,
            evidence_number,
        )?)
    } else {
        None
    };

    push_pair(
        &mut args_exec,
        &mut args_print,
        "-l",
        &format!("{}/copy", evidence_dir_1),
    );

    push_pair(
        &mut args_exec,
        &mut args_print,
        "-t",
        &format!("{}/{}", evidence_dir_1, evidence_number),
    );

    if let Some(evidence_dir_2) = evidence_dir_2 {
        push_pair(
            &mut args_exec,
            &mut args_print,
            "-2",
            &format!("{}/{}", evidence_dir_2, evidence_number),
        );
    }

    args_exec.push("-u".to_string());
    args_print.push("-u".to_string());
    args_exec.push("-v".to_string());
    args_print.push("-v".to_string());
    if config.zero_on_read_error {
        args_exec.push("-w".to_string());
        args_print.push("-w".to_string());
    }
    if config.use_chunk_data {
        args_exec.push("-x".to_string());
        args_print.push("-x".to_string());
    }
    args_exec.push(actual_input_device.clone());
    args_print.push(format!("\"{}\"", actual_input_device));

    let full_command_print = format!("sudo ewfacquire {}", args_print.join(" "));
    log_debug(&format!("Spouštím příkaz: {}", full_command_print));
    println!("Spouštím příkaz: {}", full_command_print);

    let shell = app_handle.shell();
    let (mut rx, _child) = shell
        .command("sudo")
        .args(["ewfacquire"])
        .args(&args_exec)
        .spawn()
        .map_err(|e| {
            log_error(&format!("(Command) Failed to spawn command: {}", e));
            format!("(Command) Failed to spawn command: {}", e)
        })?;

    let mut md5_hash: Option<String> = None;
    let mut sha1_hash: Option<String> = None;
    let mut sha256_hash: Option<String> = None;

    let copy_log_id: i64 = {
        let conn = crate::db::create_new_connection()
            .map_err(|e| format!("Failed to create connection: {}", e))?;
        conn.query_row(
            "SELECT triggered_by_ewf FROM copy_process WHERE id = ?1",
            params![process_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to get copy_log_id from copy_process: {}", e))?
    };

    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(line) => {
                {
                    let conn = crate::db::create_new_connection()
                        .map_err(|e| format!("(DB stdout) Failed to create connection: {}", e))?;
                    conn.execute(
                        "INSERT INTO process_log_lines (process_id, line_content, line_number)
                         VALUES (?1, ?2, COALESCE((SELECT MAX(line_number) FROM process_log_lines WHERE process_id=?1),0)+1)",
                        params![process_id, line],
                    ).map_err(|e| format!("(DB stdout) Error writing log: {}", e))?;
                }
                let output_msg = WsProcessOutput {
                    msg_type: "ProcessOutput".to_string(),
                    id: process_id,
                    output: String::from_utf8_lossy(&line).to_string(),
                };
                let json_output = serde_json::to_string(&output_msg).unwrap();
                websocket::broadcast_message(&json_output).await;

                lazy_static! {
                    static ref PROGRESS_REGEX: Regex = Regex::new(r"Status: at (\d+)%\.").unwrap();
                    static ref COMPLETION_REGEX: Regex = Regex::new(
                        r"completion in(?: (?:(?P<days>\d+) day\(s\), )?(?:(?P<hours>\d+) hour\(s\), )?(?:(?P<minutes>\d+) minute\(s\) and )?)?(?P<seconds>\d+) second\(s\) with (?P<speed>[\d\.]+) (?P<unit>[KMGT]?iB)/s"
                    ).unwrap();
                    static ref MD5_REGEX: Regex = Regex::new(r"MD5 hash calculated over data:\s+([a-fA-F0-9]{32})").unwrap();
                    static ref SHA1_REGEX: Regex = Regex::new(r"SHA1 hash calculated over data:\s+([a-fA-F0-9]{40})").unwrap();
                    static ref SHA256_REGEX: Regex = Regex::new(r"SHA256 hash calculated over data:\s+([a-fA-F0-9]{64})").unwrap();
                    static ref WRITTEN_REGEX: Regex = Regex::new(r"Written: (.*?) \((.*?) bytes\) in (.*?) with (.*?) \((.*?) bytes/second\)").unwrap();
                }
                if let Some(caps) = PROGRESS_REGEX.captures(&String::from_utf8_lossy(&line)) {
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
                let line_str = String::from_utf8_lossy(&line);
                if let Some(caps) = COMPLETION_REGEX.captures(&line_str) {
                    let days = caps
                        .name("days")
                        .map_or(0, |m| m.as_str().parse().unwrap_or(0));
                    let hours = caps
                        .name("hours")
                        .map_or(0, |m| m.as_str().parse().unwrap_or(0));
                    let minutes = caps
                        .name("minutes")
                        .map_or(0, |m| m.as_str().parse().unwrap_or(0));
                    let seconds = caps["seconds"].parse().unwrap_or(0);
                    let total_time = days * 86400 + hours * 3600 + minutes * 60 + seconds;

                    let speed_raw = caps["speed"].parse::<f64>().unwrap_or(0.0);
                    let unit = caps.name("unit").map(|m| m.as_str()).unwrap_or("MiB");

                    let multiplier = match unit {
                        "KiB" => 1.0 / 1024.0,
                        "GiB" => 1024.0,
                        "TiB" => 1024.0 * 1024.0,
                        _ => 1.0,
                    };

                    let speed_mib = speed_raw * multiplier;

                    let update = WsProcessProgress {
                        msg_type: "ProcessProgress".to_string(),
                        id: process_id,
                        progress_perc: None,
                        progress_time: Some(total_time),
                        speed: Some(speed_mib),
                    };
                    let json = serde_json::to_string(&update).unwrap();
                    websocket::broadcast_message(&json).await;
                }

                if let Some(caps) = MD5_REGEX.captures(&line_str) {
                    md5_hash = Some(caps[1].to_string());
                    println!("Found MD5 hash: {}", caps[1].to_string());
                }
                if let Some(caps) = SHA1_REGEX.captures(&line_str) {
                    sha1_hash = Some(caps[1].to_string());
                    println!("Found SHA1 hash: {}", caps[1].to_string());
                }
                if let Some(caps) = SHA256_REGEX.captures(&line_str) {
                    sha256_hash = Some(caps[1].to_string());
                    println!("Found SHA256 hash: {}", caps[1].to_string());
                }
                if let Some(caps) = WRITTEN_REGEX.captures(&line_str) {
                    println!(
                        "Written: {} ({} bytes) in {} with {} ({} bytes/second)",
                        caps[1].to_string(),
                        caps[2].to_string(),
                        caps[3].to_string(),
                        caps[4].to_string(),
                        caps[5].to_string()
                    );
                }
            }
            CommandEvent::Stderr(line) => {
                let log_line = format!("STDERR: {}", String::from_utf8_lossy(&line));
                let conn = crate::db::create_new_connection()
                    .map_err(|e| format!("(DB stderr) Failed to create connection: {}", e))?;
                conn.execute(
                    "INSERT INTO process_log_lines (process_id, line_content, line_number)
                     VALUES (?1, ?2, COALESCE((SELECT MAX(line_number) FROM process_log_lines WHERE process_id=?1),0)+1)",
                    params![process_id, log_line],
                ).map_err(|e| format!("(DB stderr) Chyba při zápisu stderr: {}", e))?;
            }
            CommandEvent::Terminated(exit_code) => {
                LED_CONTROLLER.notify_process_end();

                let final_status = if exit_code.code.unwrap_or(-1) == 0 {
                    "done"
                } else {
                    "error"
                };

                println!(
                    "ewfacquire process terminated with status: {}",
                    final_status
                );

                let end_time = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
                let end_time_for_db = end_time.clone();

                tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
                    let conn = crate::db::create_new_connection()
                        .map_err(|e| format!("(DB) Error creating final connection: {}", e))?;

                    let mut update_sql = format!(
                        "UPDATE copy_log_ewf SET status = '{}', end_datetime = '{}' ",
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
                        .map_err(|e| format!("Error updating copy_log_ewf: {}", e))?;

                    conn.execute(
                        "UPDATE copy_process SET status = ?, end_datetime = ? WHERE id = ?",
                        params![final_status, end_time_for_db, process_id],
                    )
                    .map_err(|e| format!("Error updating copy_process: {}", e))?;

                    Ok(())
                })
                .await
                .map_err(|e| e.to_string())??;

                if final_status == "done" {
                    let report_result = generate_report_ewfacquire(process_id);
                    if let Err(e) = report_result {
                        eprintln!("Chyba při generování reportu: {}", e);
                    }
                }

                let process_done = WsProcessDone {
                    msg_type: "ProcessDone".to_string(),
                    id: process_id,
                    status: final_status.to_string(),
                    end_datetime: end_time,
                };
                let json_done = serde_json::to_string(&process_done).unwrap_or_default();
                websocket::broadcast_message(&json_done).await;

                break;
            }
            _ => (),
        }
    }

    Ok(())
}

fn strip_dev_prefix(full_path: &str) -> String {
    full_path
        .trim_start_matches("/dev/disk/by-path/")
        .to_string()
}
