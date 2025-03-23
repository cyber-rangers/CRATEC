use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::Number;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use crate::db::connect_db; // Import funkce pro spojení

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EwfParams {
    pub case_number: String,
    pub description: String,
    pub investigator_name: String,
    pub evidence_number: String,
    pub notes: String,
    pub offset: i32,
    pub bytes_to_read: i32
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

#[tauri::command(rename_all = "snake_case")]
pub async fn run_ewfacquire(
    config_id: i32,
    ewf_params: EwfParams,
    input_interface: String,
    output_interface: String,
) -> Result<(), String> {
    let ewf_params_db = ewf_params.clone();
    let input_interface_db = input_interface.clone();
    let output_interface_db = output_interface.clone();

    // Na samostatném vlákně získáme config z DB a provedeme zápis do logovacích tabulek.
    let (config, process_id) = tauri::async_runtime::spawn_blocking(move || -> Result<(EwfConfig, i64), String> {
        let conn = connect_db().map_err(|e| e.to_string())?;
        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

        let config = {
            let mut stmt = tx.prepare(
                "SELECT confname, codepage, sectors_per_read, bytes_to_read,
                        compression_method, compression_level, hash_types, ewf_format,
                        granularity_sectors, notes, offset, process_buffer_size,
                        bytes_per_sector, read_retry_count, swap_byte_pairs,
                        segment_size, zero_on_read_error, use_chunk_data
                 FROM ewf_config
                 WHERE id = ?1 AND active = 1",
            ).map_err(|e| e.to_string())?;
            stmt.query_row(params![config_id], |row| {
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
            }).map_err(|e| e.to_string())?
        };

        // Vložíme základní informace o případu do tabulky copy_log_ewf.
        tx.execute(
            "INSERT INTO copy_log_ewf (
                config_id, case_number, description, investigator_name, evidence_number,
                source_disk_id, dest_disk_id, notes, offset, bytes_to_read,  start_datetime
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, DATETIME('now'))",
            params![
                config_id,
                ewf_params_db.case_number,
                ewf_params_db.description,
                ewf_params_db.investigator_name,
                ewf_params_db.evidence_number,
                input_interface_db,
                output_interface_db,
                ewf_params_db.notes,
                ewf_params_db.offset,
                ewf_params_db.bytes_to_read
            ],
        ).map_err(|e| e.to_string())?;

        // Zaznamenáme spuštění procesu
        tx.execute(
            "INSERT INTO copy_process (
                triggered_by_ewf
            ) VALUES (?1)",
            params![config_id],
        ).map_err(|e| e.to_string())?;

        let process_id = tx.last_insert_rowid();
        tx.commit().map_err(|e| e.to_string())?;
        Ok((config, process_id))
    }).await.map_err(|e| e.to_string())??;

    // Budeme sestavovat dva vektory argumentů – jeden pro samotné předání do Command a druhý pro tisk.
    let mut args_exec: Vec<String> = Vec::new();
    let mut args_print: Vec<String> = Vec::new();

    // Pomocná funkce pro vložení páru (flag, hodnota)
    fn push_pair(exec: &mut Vec<String>, print: &mut Vec<String>, flag: &str, value: &str) {
        exec.push(flag.to_string());
        exec.push(value.to_string());
        print.push(flag.to_string());
        print.push(format!("\"{}\"", value));
    }

   /*  -A:     codepage of header section, options: ascii (default),
    windows-874, windows-932, windows-936, windows-949,
    windows-950, windows-1250, windows-1251, windows-1252,
    windows-1253, windows-1254, windows-1255, windows-1256,
    windows-1257 or windows-1258 */
    push_pair(&mut args_exec, &mut args_print, "-A", &config.codepage);

    /* -b:     specify the number of sectors to read at once (per chunk),
    options: 16, 32, 64 (default), 128, 256, 512, 1024, 2048, 4096,
    8192, 16384 or 32768 */
    push_pair(&mut args_exec, &mut args_print, "-b", &config.sectors_per_read);
    
    /* -B:     specify the number of bytes to acquire (default is all bytes)
    přidáváme pouze pokud je něco jiného než whole, jinak se bere celý disk */
    if config.bytes_to_read != "whole" {
        push_pair(&mut args_exec, &mut args_print, "-B", &ewf_params.bytes_to_read.to_string());
    }

    /* -c:     specify the compression values as: level or method:level
    compression method options: deflate (default)
    compression level options: none (default), empty-block,
    fast or best */
    let compression_settings = format!("{}:{}", config.compression_method, config.compression_level);
    push_pair(&mut args_exec, &mut args_print, "-c", &compression_settings);
           
    /* -C:     specify the case number (default is case_number). */
    push_pair(&mut args_exec, &mut args_print, "-C", &ewf_params.case_number);

    /* -d:     calculate additional digest (hash) types besides md5, options:
    sha1, sha256 */
    if config.hash_types != "[]" && !config.hash_types.is_empty() {
        push_pair(&mut args_exec, &mut args_print, "-d", &config.hash_types);
    }

    /* -D:     specify the description (default is description). */
    push_pair(&mut args_exec, &mut args_print, "-D", &ewf_params.description);

    /* -e:     specify the examiner name (default is examiner_name). */
    push_pair(&mut args_exec, &mut args_print, "-e", &ewf_params.investigator_name);

    /* -E:     specify the evidence number (default is evidence_number). */
    push_pair(&mut args_exec, &mut args_print, "-E", &ewf_params.evidence_number);

    /* -f:     specify the EWF file format to write to, options: ewf, smart,
	       ftk, encase2, encase3, encase4, encase5, encase6 (default),
	       encase7, encase7-v2, linen5, linen6, linen7, ewfx */
    push_pair(&mut args_exec, &mut args_print, "-f", &config.ewf_format);

    /* -g      specify the number of sectors to be used as error granularity */
    push_pair(&mut args_exec, &mut args_print, "-g", &config.granularity_sectors);

    /* -l:     logs acquiry errors and the digest (hash) to the log_filename */
    push_pair(&mut args_exec, &mut args_print, "-l", "test.log");


    /* -m:     specify the media type, options: fixed (default), removable,
        optical, memory */
    push_pair(&mut args_exec, &mut args_print, "-m", "fixed");

    /* -M:     specify the media flags, options: logical, physical (default) */
    push_pair(&mut args_exec, &mut args_print, "-M", "physical");

    /* -N:     specify the notes (default is notes). */
    if config.notes == "ask" {
        push_pair(&mut args_exec, &mut args_print, "-N", &ewf_params.notes);
    }
    
    /* -o:     specify the offset to start to acquire (default is 0) */
    if config.offset == "ask" {
        push_pair(&mut args_exec, &mut args_print, "-o", &ewf_params.offset.to_string());
    }else {
        push_pair(&mut args_exec, &mut args_print, "-o", &config.offset);
    }
    

    /* -p:     specify the process buffer size (default is the chunk size) */
    if config.process_buffer_size != "auto" {
        push_pair(&mut args_exec, &mut args_print, "-p", &config.process_buffer_size);
    }

    /* -P:     specify the number of bytes per sector (default is 512)
        (use this to override the automatic bytes per sector detection) 
        pokud bude v bytes_per_serctor auto tak nevkládáme vůbec */

    if config.bytes_per_sector != "auto" {
        push_pair(&mut args_exec, &mut args_print, "-P", &config.bytes_per_sector);
    }
        

    /* -q:     quiet shows minimal status information 
        nikdy nespouštíme chceme vždy plný výstup*/


    /* -r:     specify the number of retries when a read error occurs (default
        is 2) */
    push_pair(&mut args_exec, &mut args_print, "-r", &config.read_retry_count);

    /* -R:     resume acquiry at a safe point 
        provádíme automaticky pokud byl proces kopírování pozastavený*/


    /* -s:     swap byte pairs of the media data (from AB to BA)
        (use this for big to little endian conversion and vice versa) */
    if config.swap_byte_pairs {
        args_exec.push("-s".to_string());
        args_print.push("-s".to_string());
    }

    /* -S:     specify the segment file size in bytes (default is 1.4 GiB)
        (minimum is 1.0 MiB, maximum is 7.9 EiB for encase6
        and encase7 format and 1.9 GiB for other formats) */
    push_pair(&mut args_exec, &mut args_print, "-S", &config.segment_size);

    /* -t:     specify the target file (without extension) to write to */
    push_pair(&mut args_exec, &mut args_print, "-t", &output_interface);

    /* -T:     specify the file containing the table of contents (TOC) of
        an optical disc. The TOC file must be in the CUE for mat.
        nebudeme používat*/


    /* -u:     unattended mode (disables user interaction) 
        vždy spouštíme bez user interaction*/
    args_exec.push("-u".to_string());
    args_print.push("-u".to_string());

    /* -v:     verbose output to stderr 
        vždy necháme verbose výstup*/
    args_exec.push("-v".to_string());
    args_print.push("-v".to_string());

    /* -w:     zero sectors on read error (mimic EnCase like behavior) */
    if config.zero_on_read_error {
        args_exec.push("-w".to_string());
        args_print.push("-w".to_string());
    }

    /* -x:     use the chunk data instead of the buffered read and write
        functions. */
    if config.use_chunk_data {
        args_exec.push("-x".to_string());
        args_print.push("-x".to_string());
    }

    /* -2:     specify the secondary target file (without extension) to write
        to */

    

    // Posledním argumentem (bez vlajky) je vstupní rozhraní:
    args_exec.push(input_interface.clone());
    args_print.push(format!("\"{}\"", input_interface));


    // Vytvoříme tiskový řetězec – před spuštěním si ověříme, jak bude příkaz vypadat
    let full_command_print = format!("sudo ewfacquire {}", args_print.join(" "));
    println!("Spouštím příkaz: {}", full_command_print);

    // Spouštíme příkaz přes sudo – první argument je "ewfacquire" následovaný předanými argumenty (bez uvozovek)
    let mut child = Command::new("sudo")
        .arg("ewfacquire")
        .args(&args_exec)
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    // Pro zápis logu získáme nové DB instance (protože Connection není bezpečná pro sdílení mezi vlákny)
    let conn_stdout = connect_db().map_err(|e| e.to_string())?;
    let conn_stderr = connect_db().map_err(|e| e.to_string())?;

    // Asynchroní úloha pro čtení stdout
    let stdout_handle = if let Some(stdout) = child.stdout.take() {
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            let mut line_counter = 1;
            while let Some(line) = reader.next_line().await.map_err(|e| e.to_string())? {
                conn_stdout.execute(
                    "INSERT INTO process_log_lines (
                        process_id, line_content, line_number
                    ) VALUES (?1, ?2, ?3)",
                    params![process_id, line, line_counter],
                ).map_err(|e| e.to_string())?;
                line_counter += 1;
            }
            Ok::<(), String>(())
        })
    } else {
        tokio::spawn(async { Ok(()) })
    };

    // Asynchroní úloha pro čtení stderr (řádky označíme prefixem "STDERR:")
    let stderr_handle = if let Some(stderr) = child.stderr.take() {
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            let mut line_counter = 1;
            while let Some(line) = reader.next_line().await.map_err(|e| e.to_string())? {
                let log_line = format!("STDERR: {}", line);
                conn_stderr.execute(
                    "INSERT INTO process_log_lines (
                        process_id, line_content, line_number
                    ) VALUES (?1, ?2, ?3)",
                    params![process_id, log_line, line_counter],
                ).map_err(|e| e.to_string())?;
                line_counter += 1;
            }
            Ok::<(), String>(())
        })
    } else {
        tokio::spawn(async { Ok(()) })
    };

    stdout_handle.await.map_err(|e| e.to_string())??;
    stderr_handle.await.map_err(|e| e.to_string())??;

    let status = child.wait().await.map_err(|e| e.to_string())?;
    let final_status = if status.success() { "done" } else { "error" };

    // Aktualizace stavu v databázi
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        let conn2 = connect_db().map_err(|e| e.to_string())?;
        conn2.execute(
            "UPDATE copy_process
             SET status = ?1,
                 end_datetime = DATETIME('now')
             WHERE id = ?2",
            params![final_status, process_id],
        ).map_err(|e| e.to_string())?;
        conn2.execute(
            "UPDATE copy_log_ewf
             SET status = ?1,
                 end_datetime = DATETIME('now')
             WHERE config_id = ?2",
            params![final_status, config_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())??;

    Ok(())
}
