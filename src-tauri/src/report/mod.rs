//! Modul pro tvorbu PDF reportu (s detailními debug-výpisy)

use crate::{db::DB_POOL, disk_utils};
use crate::system_info::get_report_system_info;
use chrono::{Local, NaiveDateTime, TimeZone, Utc}; // přidej Utc
use once_cell::sync::Lazy;
use rusqlite::{Row, Statement};
use serde_json::{Map, Value};
use std::fs;
use tera::{Context, Tera};

static TEMPLATE_EN_EWF: &str = include_str!("./templates/en_ewf.tex");
static TEMPLATE_EN_DD: &str = include_str!("./templates/en_dd.tex");

/// ----------------- malé pomůcky ------------------------------------------
fn vstr<S: Into<String>>(s: S) -> Value {
    Value::String(s.into())
}
fn vu64(n: u64) -> Value {
    Value::Number(n.into())
}
static EMPTY_MAP: Lazy<Map<String, Value>> = Lazy::new(Map::new);

trait StrExt {
    fn if_empty_then<'a>(&'a self, alt: &'a str) -> &'a str;
}
impl StrExt for str {
    fn if_empty_then<'a>(&'a self, alt: &'a str) -> &'a str {
        if self.is_empty() {
            alt
        } else {
            self
        }
    }
}

/// Vypůjčené &str přímo z JSONu
fn gs<'a>(o: &'a Map<String, Value>, k: &str) -> &'a str {
    o.get(k).and_then(Value::as_str).unwrap_or("")
}
fn gu(o: &Map<String, Value>, k: &str) -> u64 {
    o.get(k)
        .and_then(|v| match v {
            Value::Number(n) => n.as_u64(),
            Value::String(s) => s.parse::<u64>().ok(),
            _ => None,
        })
        .unwrap_or(0)
}

/// Převod řádku SQL → JSON objekt
fn row_to_json_with_cols(row: &Row, cols: &[String]) -> Value {
    let mut obj = Map::new();
    for (i, c) in cols.iter().enumerate() {
        let v = row
            .get::<_, String>(i)
            .map(Value::String)
            .or_else(|_| row.get::<_, i64>(i).map(|n| Value::Number(n.into())))
            .unwrap_or(Value::Null);
        obj.insert(c.clone(), v);
    }
    Value::Object(obj)
}

/// Získá `interface_path` pro daný disk-ID
fn get_interface_path(conn: &rusqlite::Connection, id: i64) -> Option<String> {
    conn.query_row(
        "SELECT interface_path FROM interface WHERE id=?",
        [id],
        |r| r.get(0),
    )
    .ok()
}

/// --------------------------------------------------------------------------
pub fn generate_report_ewfacquire(id: i64) -> Result<(), String> {
    println!("▶️  generate_report({id}) – START");
    println!("Current dir: {:?}", std::env::current_dir());
    println!("USER: {:?}", std::env::var("USER"));
    println!("HOME: {:?}", std::env::var("HOME"));
    println!("PATH: {:?}", std::env::var("PATH"));

    // 1️⃣  Načti agregovaný JSON z DB
    let mut report = get_report_json_data(id)?
        .as_object()
        .cloned()
        .ok_or("Report JSON není objekt")?;
    let log_map = report["log_record"]
        .as_object()
        .cloned()
        .unwrap_or_default();
    println!(
        "✅  data z DB – klíče: {:?}",
        report.keys().collect::<Vec<_>>()
    );

    // --- helper: načti disk podle ID v logu
    let load_disk = |key: &str| -> Result<Value, String> {
        let disk_id = log_map.get(key).and_then(Value::as_i64).unwrap_or(0);
        if disk_id == 0 {
            return Ok(Value::Null);
        }
        let mut pool = DB_POOL.get_connection().map_err(|e| e.to_string())?;
        let path = get_interface_path(pool.connection(), disk_id)
            .map(|p| format!("/dev/disk/by-path/{p}"))
            .ok_or_else(|| format!("interface_path chybí pro disk {disk_id}"))?;
        println!("🔍  {key}: disk_utils::get_disk_info({path})");
        disk_utils::get_disk_info(&path)
            .map_err(|e| e.to_string())
            .and_then(|d| serde_json::to_value(d).map_err(|e| e.to_string()))
    };
    report.insert("source_disk".into(), load_disk("source_disk_id")?);
    report.insert("dest_disk".into(), load_disk("dest_disk_id")?);
    report.insert("second_dest_disk".into(), load_disk("second_dest_disk_id")?);
    println!("✅  disky načteny");

    let root_value = Value::Object(report.clone());
    // debug‐print celého JSON
    println!(
        "DEBUG root JSON:\n{}",
        serde_json::to_string_pretty(&root_value).unwrap()
    );

    // 2️⃣  Sestavení Tera Contextu
    let mut ctx = Context::new();
    let now = Local::now();
    ctx.insert("date", &now.format("%b %d, %Y").to_string());
    ctx.insert("time_local", &now.format("%H:%M:%S (%Z)").to_string());

    let sysinfo = get_report_system_info().map_err(|e| format!("system info: {e}"))?;
    ctx.insert("software_hash", &sysinfo.cratec_hash);
    ctx.insert("build_date", &sysinfo.build_date);
    ctx.insert("serial_number", &sysinfo.short_hw_id);

    let cfg = report["config_record"].as_object().unwrap();
    let proc = report["copy_process"].as_object().unwrap();
    ctx.insert("mode", "DriveToFile");
    ctx.insert(
        "method",
        if proc.get("triggered_by_ewf").is_some() {
            "ewfacquire"
        } else {
            "dcfldd"
        },
    );

    let hash_types = gs(cfg, "hash_types");
    let mut hash_type = String::from("MD5");
    let hash_types_trimmed = hash_types
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(",");
    if !hash_types_trimmed.is_empty() {
        hash_type.push(',');
        hash_type.push_str(&hash_types_trimmed);
    }
    ctx.insert("hash_type", &hash_type);
    ctx.insert("compression_method", gs(cfg, "compression_method"));
    ctx.insert("compression_level", gs(cfg, "compression_level"));
    ctx.insert("ewf_format", gs(cfg, "ewf_format"));
    ctx.insert("segment_size", gs(cfg, "segment_size"));
    ctx.insert("granularity_sectors", gs(cfg, "granularity_sectors"));
    let swap_byte_pairs = cfg
        .get("swap_byte_pairs")
        .and_then(Value::as_i64)
        .unwrap_or(0)
        == 1;
    ctx.insert("swap_byte_pairs", &swap_byte_pairs);
    ctx.insert("hash_enabled", &true);
    ctx.insert("verify_hash", &false);
    ctx.insert("unlock_hpa", &false);
    ctx.insert("unlock_dco", &false);

    // ---------- Segment #1 -------------------------------------------------
    let mut seg_uid = String::new();
    let mut seg_fs = String::new();
    let mut seg_serial = String::new();
    let mut seg_file = String::new();
    let mut seg_path = String::new();

    if let Some(dest) = report["dest_disk"].as_object() {
        if let Some(part) = dest
            .get("partitions")
            .and_then(Value::as_array)
            .and_then(|a| a.iter().find(|p| p.get("mountpoint").is_some()))
        {
            let po = part.as_object().unwrap();
            seg_uid = gs(po, "uuid").into();
            seg_fs = gs(po, "filesystem").into();
            seg_serial = gs(dest, "serial").into();
            seg_file = gs(&log_map, "evidence_number").into();
            let mount = gs(po, "mountpoint");
            seg_path = format!(
                "{mount}/{}/{}/",
                gs(&log_map, "case_number"),
                gs(&log_map, "evidence_number")
            );
        }
    }
    ctx.insert("segment_uid", &seg_uid);
    ctx.insert("segment_fs", &seg_fs);
    ctx.insert("segment_serial", &seg_serial);
    ctx.insert("segment_file", &seg_file);
    ctx.insert("segment_path", &seg_path);
    ctx.insert("image_path", &seg_path);

    // ---------- Segment #2 (volitelný) -------------------------------------
    let mut seg2_uid = String::new();
    let mut seg2_fs = String::new();
    let mut seg2_serial = String::new();
    let mut seg2_file = String::new();
    let mut seg2_path = String::new();

    if let Some(dest2) = report["second_dest_disk"].as_object() {
        if !dest2.is_empty() {
            if let Some(part) = dest2
                .get("partitions")
                .and_then(Value::as_array)
                .and_then(|a| a.iter().find(|p| p.get("mountpoint").is_some()))
            {
                let po = part.as_object().unwrap();
                seg2_uid = gs(po, "uuid").into();
                seg2_fs = gs(po, "filesystem").into();
                seg2_serial = gs(dest2, "serial").into();
                seg2_file = gs(&log_map, "evidence_number").into();
                let mount = gs(po, "mountpoint");
                seg2_path = format!(
                    "{mount}/{}/{}/",
                    gs(&log_map, "case_number"),
                    gs(&log_map, "evidence_number")
                );
            }
        }
    }
    ctx.insert("segment2_uid", &seg2_uid);
    ctx.insert("segment2_fs", &seg2_fs);
    ctx.insert("segment2_serial", &seg2_serial);
    ctx.insert("segment2_file", &seg2_file);
    ctx.insert("segment2_path", &seg2_path);

    // ---------- další položky ----------------------------------------------
    let result = match proc.get("status").and_then(Value::as_str) {
        Some("done") => "SUCCESS",
        Some("error") => "ERROR",
        _ => "N/A",
    };
    ctx.insert("result", result);
    ctx.insert("case_file", "CaseFile_001");

    // --- časy
    let fmt = "%Y-%m-%d %H:%M:%S";
    let t_start_str = gs(proc, "start_datetime");
    let t_start = NaiveDateTime::parse_from_str(&t_start_str, fmt)
        .ok()
        .and_then(|dt| Utc.from_local_datetime(&dt).single())
        .map(|dt| dt.with_timezone(&Local));
    if let Some(s) = t_start {
        ctx.insert("time_started", &s.format("%H:%M:%S %-d.%-m.%Y").to_string());
    }
    let t_end = NaiveDateTime::parse_from_str(gs(&log_map, "end_datetime"), fmt)
        .ok()
        .and_then(|dt| Utc.from_local_datetime(&dt).single())
        .map(|dt| dt.with_timezone(&Local));
    if let Some(c) = t_end {
        ctx.insert(
            "time_complete",
            &c.format("%H:%M:%S %-d.%-m.%Y").to_string(),
        );
    }
    let duration = t_start
        .zip(t_end)
        .map(|(s, e)| e - s)
        .map(|d| {
            format!(
                "{:02}:{:02}:{:02}",
                d.num_hours(),
                d.num_minutes() % 60,
                d.num_seconds() % 60
            )
        })
        .unwrap_or_else(|| "N/A".into());
    ctx.insert("duration", &duration);

    let sdisk = report["source_disk"].as_object().unwrap_or(&EMPTY_MAP);

    // Získání hodnot z log_record (nebo odkud je máš v JSONu)
    let log = report["log_record"].as_object().unwrap_or(&EMPTY_MAP);
    let offset = gu(log, "offset");
    let bytes_to_read = gu(log, "bytes_to_read");
    let sector_size = gu(sdisk, "logical_sector_size");
    let cap = gu(sdisk, "capacity_bytes");

    let lba_count = if offset == 0 && bytes_to_read == 0 {
        if sector_size > 0 {
            cap / sector_size
        } else {
            0
        }
    } else if bytes_to_read > 0 && sector_size > 0 {
        bytes_to_read / sector_size
    } else if offset > 0 && bytes_to_read == 0 && sector_size > 0 {
        (cap - offset) / sector_size
    } else {
        0
    };
    ctx.insert("lba_count", &lba_count);

    ctx.insert("offset", &offset);
    if offset == 0 && bytes_to_read == 0 {
        ctx.insert("bytes_to_read", "Whole Disk");
    } else if offset > 0 && bytes_to_read == 0 {
        ctx.insert("bytes_to_read", &(cap - offset));
    } else {
        ctx.insert("bytes_to_read", &bytes_to_read);
    }

    ctx.insert("sector_size", &sector_size);

    // hashes
    let mut hashes = Vec::<(String, String)>::new();
    for (t, k) in [
        ("MD5", "md5_hash"),
        ("SHA1", "sha1_hash"),
        ("SHA256", "sha256_hash"),
    ] {
        if let Some(h) = log_map
            .get(k)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
        {
            hashes.push((t.into(), h.into()));
        }
    }
    ctx.insert("hashes", &hashes);

    // case info
    ctx.insert("case_number", gs(&log_map, "case_number"));
    ctx.insert("evidence_number", gs(&log_map, "evidence_number"));
    ctx.insert("examiner", gs(&log_map, "investigator_name"));
    ctx.insert("notes", gs(&log_map, "notes"));

    println!(
        "✅  Context naplněn – {} klíčů",
        ctx.clone().into_json().as_object().unwrap().len()
    );

    // ---------- tabulky drives / capacities / encryption -------------------
    let mut drives = Vec::<Map<String, Value>>::new();
    let mut caps = Vec::<Map<String, Value>>::new();
    let mut encs = Vec::<Map<String, Value>>::new();

    let push_disk = |vd: &mut Vec<Map<String, Value>>,
                     vc: &mut Vec<Map<String, Value>>,
                     ve: &mut Vec<Map<String, Value>>,
                     disk: &Value,
                     bay: &str,
                     role: &str| {
        let o = match disk.as_object() {
            Some(m) if !m.is_empty() => m,
            _ => return,
        };
        // drives
        let mut d = Map::new();
        d.insert("bay".into(), vstr(bay));
        d.insert("role".into(), vstr(role));
        d.insert("serial".into(), vstr(gs(o, "serial")));
        d.insert("model".into(), vstr(gs(o, "model")));
        let fs = o
            .get("partitions")
            .and_then(Value::as_array)
            .and_then(|a| a.get(0))
            .and_then(|p| p.get("filesystem").and_then(Value::as_str))
            .unwrap_or("");
        d.insert("fs".into(), vstr(fs));
        d.insert(
            "cipher".into(),
            vstr(gs(o, "disk_encryption").if_empty_then("None")),
        );
        vd.push(d);

        // caps
        let mut c = Map::new();
        let bytes = gu(o, "capacity_bytes");
        c.insert("bay".into(), vstr(bay));
        c.insert("serial".into(), vstr(gs(o, "serial")));
        c.insert("model".into(), vstr(gs(o, "model")));
        c.insert("capacity_bytes".into(), vu64(bytes));
        c.insert(
            "capacity_gb".into(),
            vstr(format!("{:.1}", bytes as f64 / 1e9)),
        );
        vc.push(c);

        // encryption
        let mut e = Map::new();
        let yes = |b| if b { "Yes" } else { "No" };
        e.insert("bay".into(), vstr(bay));
        e.insert("role".into(), vstr(role));
        e.insert(
            "ata_encryption".into(),
            vstr(yes(o
                .get("ata_encryption")
                .and_then(Value::as_bool)
                .unwrap_or(false))),
        );
        e.insert(
            "sed_encryption".into(),
            vstr(yes(o
                .get("sed_encryption")
                .and_then(Value::as_bool)
                .unwrap_or(false))),
        );
        e.insert(
            "locked".into(),
            vstr(yes(!o
                .get("readable")
                .and_then(Value::as_bool)
                .unwrap_or(true))),
        );
        ve.push(e);
    };

    push_disk(
        &mut drives,
        &mut caps,
        &mut encs,
        &report["source_disk"],
        "1",
        "Source",
    );
    push_disk(
        &mut drives,
        &mut caps,
        &mut encs,
        &report["dest_disk"],
        "2",
        "Destination",
    );
    push_disk(
        &mut drives,
        &mut caps,
        &mut encs,
        &report["second_dest_disk"],
        "3",
        "Secondary Destination",
    );

    ctx.insert("drives", &drives);
    ctx.insert("capacities", &caps);
    ctx.insert("encryption", &encs);
    println!(
        "✅  Tabulky vloženy – drives={}, caps={}, enc={}",
        drives.len(),
        caps.len(),
        encs.len()
    );

    // ---------- partitions --------------------------------------------------
    let mut parts = Vec::<Map<String, Value>>::new();
    if let Some(arr) = sdisk.get("partitions").and_then(Value::as_array) {
        for p in arr {
            let o = p.as_object().unwrap();
            let start = gu(o, "start_sector");
            let end = gu(o, "end_sector");
            let size_mb = ((end + 1).saturating_sub(start)) as f64 * 512.0 / 1_048_576.0;
            let mut m = Map::new();
            m.insert(
                "index".into(),
                vu64(o.get("index").and_then(Value::as_u64).unwrap_or(0)),
            );
            m.insert("fs".into(), vstr(gs(o, "filesystem")));
            m.insert("start".into(), vu64(start));
            m.insert("end".into(), vu64(end));
            m.insert("size".into(), vstr(format!("{:.1} MB", size_mb)));
            parts.push(m);
        }
    }
    ctx.insert("source_partitions", &parts);
    println!("✅  Source partitions – {}", parts.len());

    println!("🚧  Renderuji Tera …");

    let latex = match Tera::one_off(TEMPLATE_EN_EWF, &ctx, false) {
        Ok(l) => l,
        Err(err) => {
            eprintln!("❌  Tera render error: {:#?}", err);
            eprintln!("└── {}", err);
            return Err(format!("Render selhal: {err}"));
        }
    };

    // Unikátní dočasné cesty podle id
    let tex_path = format!("/tmp/report_{id}.tex");
    let pdf_path = format!("/tmp/report_{id}.pdf");

    // Zápis .tex
    fs::write(&tex_path, &latex).map_err(|e| e.to_string())?;

    // Spuštění tectonic
    let tectonic_status = std::process::Command::new("tectonic")
        .arg("--outdir")
        .arg("/tmp")
        .arg(&tex_path)
        .status()
        .map_err(|e| format!("Failed to run tectonic: {e}"))?;

    if !tectonic_status.success() {
        // Úklid
        let _ = fs::remove_file(&tex_path);
        return Err(format!(
            "Tectonic failed with exit code: {:?}",
            tectonic_status.code()
        ));
    }

    // Načtení PDF
    let pdf = fs::read(&pdf_path).map_err(|e| e.to_string())?;

    // Uložení na cílové zařízení
    let save_pdf = |key: &str, pdf: &[u8]| -> std::io::Result<()> {
        let disk = report[key].as_object().unwrap_or(&EMPTY_MAP);
        let base = disk
            .get("partitions")
            .and_then(Value::as_array)
            .and_then(|a| a.iter().find(|p| p.get("mountpoint").is_some()))
            .and_then(|p| p.get("mountpoint").and_then(Value::as_str))
            .map(|mp| {
                format!(
                    "{mp}/{}/{}/",
                    gs(&log_map, "case_number"),
                    gs(&log_map, "evidence_number")
                )
            });
        if let Some(mut dir) = base {
            fs::create_dir_all(&dir)?;
            dir.push_str("audit-report.pdf");
            fs::write(dir, pdf)?;
        }
        Ok(())
    };
    save_pdf("dest_disk", &pdf)
        .and(save_pdf("second_dest_disk", &pdf))
        .map_err(|e| e.to_string())?;

    let _ = fs::remove_file(&tex_path);
    let _ = fs::remove_file(&pdf_path);

    println!("🏁  generate_report({id}) – DONE");
    Ok(())
}

pub fn generate_report_dcfldd(id: i64) -> Result<(), String> {
    println!("▶️  generate_report_dcfldd({id}) – START");

    // 1️⃣  Načti agregovaný JSON z DB
    let mut report = get_report_json_data(id)?
        .as_object()
        .cloned()
        .ok_or("Report JSON není objekt")?;
    let log_map = report["log_record"]
        .as_object()
        .cloned()
        .unwrap_or_default();
    println!(
        "✅  data z DB – klíče: {:?}",
        report.keys().collect::<Vec<_>>()
    );

    // --- helper: načti disk podle ID v logu
    let load_disk = |key: &str| -> Result<Value, String> {
        let disk_id = log_map.get(key).and_then(Value::as_i64).unwrap_or(0);
        if disk_id == 0 {
            return Ok(Value::Null);
        }
        let mut pool = DB_POOL.get_connection().map_err(|e| e.to_string())?;
        let path = get_interface_path(pool.connection(), disk_id)
            .map(|p| format!("/dev/disk/by-path/{p}"))
            .ok_or_else(|| format!("interface_path chybí pro disk {disk_id}"))?;
        println!("🔍  {key}: disk_utils::get_disk_info({path})");
        disk_utils::get_disk_info(&path)
            .map_err(|e| e.to_string())
            .and_then(|d| serde_json::to_value(d).map_err(|e| e.to_string()))
    };
    report.insert("source_disk".into(), load_disk("source_disk_id")?);
    report.insert("dest_disk".into(), load_disk("dest_disk_id")?);
    report.insert("second_dest_disk".into(), load_disk("second_dest_disk_id")?);
    println!("✅  disky načteny");

    let root_value = Value::Object(report.clone());
    println!(
        "DEBUG root JSON:\n{}",
        serde_json::to_string_pretty(&root_value).unwrap()
    );

    // 2️⃣  Sestavení Tera Contextu
    let mut ctx = Context::new();
    let now = Local::now();
    ctx.insert("date", &now.format("%b %d, %Y").to_string());
    ctx.insert("time_local", &now.format("%H:%M:%S (%Z)").to_string());
    
    
    let sysinfo = get_report_system_info().map_err(|e| format!("system info: {e}"))?;
    ctx.insert("software_hash", &sysinfo.cratec_hash);
    ctx.insert("build_date", &sysinfo.build_date);
    ctx.insert("serial_number", &sysinfo.short_hw_id);

    let cfg = report["config_record"].as_object().unwrap();
    let proc = report["copy_process"].as_object().unwrap();
    ctx.insert("mode", "DriveToFile");
    ctx.insert("method", "dcfldd");

    let hash_types = gs(cfg, "hash_types");
    let mut hash_type = String::from("MD5");
    let hash_types_trimmed = hash_types
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty() && s != "MD5")
        .collect::<Vec<_>>()
        .join(", ");
    if !hash_types_trimmed.is_empty() {
        hash_type.push_str(", ");
        hash_type.push_str(&hash_types_trimmed);
    }
    ctx.insert("hash_type", &hash_type);

    let split = gs(cfg, "split");
    if split == "whole" {
        ctx.insert("segment_size", "Whole Disk");
    } else {
        ctx.insert("segment_size", split);
    }
    ctx.insert("granularity_sectors", "");
    ctx.insert("swap_byte_pairs", &false);
    ctx.insert("hash_enabled", &true);

    ctx.insert("verify_hash", &false);
    ctx.insert("unlock_hpa", &false);
    ctx.insert("unlock_dco", &false);

    // ---------- Segment #1 -------------------------------------------------
    let mut seg_uid = String::new();
    let mut seg_fs = String::new();
    let mut seg_serial = String::new();
    let mut seg_file = String::new();
    let mut seg_path = String::new();

    if let Some(dest) = report["dest_disk"].as_object() {
        if let Some(part) = dest
            .get("partitions")
            .and_then(Value::as_array)
            .and_then(|a| a.iter().find(|p| p.get("mountpoint").is_some()))
        {
            let po = part.as_object().unwrap();
            seg_uid = gs(po, "uuid").into();
            seg_fs = gs(po, "filesystem").into();
            seg_serial = gs(dest, "serial").into();
            seg_file = gs(&log_map, "evidence_number").into();
            let mount = gs(po, "mountpoint");
            seg_path = format!(
                "{mount}/{}/{}/",
                gs(&log_map, "case_number"),
                gs(&log_map, "evidence_number")
            );
        }
    }
    ctx.insert("segment_uid", &seg_uid);
    ctx.insert("segment_fs", &seg_fs);
    ctx.insert("segment_serial", &seg_serial);
    ctx.insert("segment_file", &seg_file);
    ctx.insert("segment_path", &seg_path);
    ctx.insert("image_path", &seg_path);

    // ---------- Segment #2 (volitelný) -------------------------------------
    let mut seg2_uid = String::new();
    let mut seg2_fs = String::new();
    let mut seg2_serial = String::new();
    let mut seg2_file = String::new();
    let mut seg2_path = String::new();

    if let Some(dest2) = report["second_dest_disk"].as_object() {
        if !dest2.is_empty() {
            if let Some(part) = dest2
                .get("partitions")
                .and_then(Value::as_array)
                .and_then(|a| a.iter().find(|p| p.get("mountpoint").is_some()))
            {
                let po = part.as_object().unwrap();
                seg2_uid = gs(po, "uuid").into();
                seg2_fs = gs(po, "filesystem").into();
                seg2_serial = gs(dest2, "serial").into();
                seg2_file = gs(&log_map, "evidence_number").into();
                let mount = gs(po, "mountpoint");
                seg2_path = format!(
                    "{mount}/{}/{}/",
                    gs(&log_map, "case_number"),
                    gs(&log_map, "evidence_number")
                );
            }
        }
    }
    ctx.insert("segment2_uid", &seg2_uid);
    ctx.insert("segment2_fs", &seg2_fs);
    ctx.insert("segment2_serial", &seg2_serial);
    ctx.insert("segment2_file", &seg2_file);
    ctx.insert("segment2_path", &seg2_path);

    // ---------- další položky ----------------------------------------------
    let result = match proc.get("status").and_then(Value::as_str) {
        Some("done") => "SUCCESS",
        Some("error") => "ERROR",
        _ => "N/A",
    };
    ctx.insert("result", result);
    ctx.insert("case_file", "CaseFile_001");

    // --- časy
    let fmt = "%Y-%m-%d %H:%M:%S";
    let t_start_str = gs(proc, "start_datetime");
    let t_start = NaiveDateTime::parse_from_str(&t_start_str, fmt)
        .ok()
        .and_then(|dt| Utc.from_local_datetime(&dt).single())
        .map(|dt| dt.with_timezone(&Local));
    if let Some(s) = t_start {
        ctx.insert("time_started", &s.format("%H:%M:%S %-d.%-m.%Y").to_string());
    }
    let t_end = NaiveDateTime::parse_from_str(gs(&log_map, "end_datetime"), fmt)
        .ok()
        .and_then(|dt| Utc.from_local_datetime(&dt).single())
        .map(|dt| dt.with_timezone(&Local));
    if let Some(c) = t_end {
        ctx.insert(
            "time_complete",
            &c.format("%H:%M:%S %-d.%-m.%Y").to_string(),
        );
    }
    let duration = t_start
        .zip(t_end)
        .map(|(s, e)| e - s)
        .map(|d| {
            format!(
                "{:02}:{:02}:{:02}",
                d.num_hours(),
                d.num_minutes() % 60,
                d.num_seconds() % 60
            )
        })
        .unwrap_or_else(|| "N/A".into());
    ctx.insert("duration", &duration);

    let sdisk = report["source_disk"].as_object().unwrap_or(&EMPTY_MAP);

    // Získání hodnot z log_record (nebo odkud je máš v JSONu)
    let log = report["log_record"].as_object().unwrap_or(&EMPTY_MAP);
    let offset = gu(log, "offset");
    let limit = gu(log, "limit_value");
    let sector_size = gu(sdisk, "logical_sector_size");
    let cap = gu(sdisk, "capacity_bytes");

    let lba_count = if offset == 0 && limit == 0 {
        if sector_size > 0 {
            cap / sector_size
        } else {
            0
        }
    } else if limit > 0 {
        limit
    } else if offset > 0 && limit == 0 && sector_size > 0 {
        (cap - offset) / sector_size
    } else {
        0
    };
    ctx.insert("lba_count", &lba_count);

    ctx.insert("offset", &offset);
    if offset == 0 && limit == 0 {
        ctx.insert("bytes_to_read", "Whole Disk");
    } else if offset > 0 && limit == 0 {
        ctx.insert("bytes_to_read", &(cap - offset));
    } else {
        ctx.insert("bytes_to_read", &(limit * sector_size));
    }

    ctx.insert("sector_size", &sector_size);

    // hashes
    let mut hashes = Vec::<(String, String)>::new();
    for (t, k) in [
        ("MD5", "md5_hash"),
        ("SHA1", "sha1_hash"),
        ("SHA256", "sha256_hash"),
    ] {
        if let Some(h) = log_map
            .get(k)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
        {
            hashes.push((t.into(), h.into()));
        }
    }
    ctx.insert("hashes", &hashes);

    // case info
    ctx.insert("case_number", gs(&log_map, "case_number"));
    ctx.insert("evidence_number", gs(&log_map, "evidence_number"));
    ctx.insert("examiner", gs(&log_map, "investigator_name"));
    ctx.insert("notes", gs(&log_map, "notes"));

    // ---------- tabulky drives / capacities / encryption -------------------
    let mut drives = Vec::<Map<String, Value>>::new();
    let mut caps = Vec::<Map<String, Value>>::new();
    let mut encs = Vec::<Map<String, Value>>::new();

    let push_disk = |vd: &mut Vec<Map<String, Value>>,
                     vc: &mut Vec<Map<String, Value>>,
                     ve: &mut Vec<Map<String, Value>>,
                     disk: &Value,
                     bay: &str,
                     role: &str| {
        let o = match disk.as_object() {
            Some(m) if !m.is_empty() => m,
            _ => return,
        };
        // drives
        let mut d = Map::new();
        d.insert("bay".into(), vstr(bay));
        d.insert("role".into(), vstr(role));
        d.insert("serial".into(), vstr(gs(o, "serial")));
        d.insert("model".into(), vstr(gs(o, "model")));
        let fs = o
            .get("partitions")
            .and_then(Value::as_array)
            .and_then(|a| a.get(0))
            .and_then(|p| p.get("filesystem").and_then(Value::as_str))
            .unwrap_or("");
        d.insert("fs".into(), vstr(fs));
        d.insert(
            "cipher".into(),
            vstr(gs(o, "disk_encryption").if_empty_then("None")),
        );
        vd.push(d);

        // caps
        let mut c = Map::new();
        let bytes = gu(o, "capacity_bytes");
        c.insert("bay".into(), vstr(bay));
        c.insert("serial".into(), vstr(gs(o, "serial")));
        c.insert("model".into(), vstr(gs(o, "model")));
        c.insert("capacity_bytes".into(), vu64(bytes));
        c.insert(
            "capacity_gb".into(),
            vstr(format!("{:.1}", bytes as f64 / 1e9)),
        );
        vc.push(c);

        // encryption
        let mut e = Map::new();
        let yes = |b| if b { "Yes" } else { "No" };
        e.insert("bay".into(), vstr(bay));
        e.insert("role".into(), vstr(role));
        e.insert(
            "ata_encryption".into(),
            vstr(yes(o
                .get("ata_encryption")
                .and_then(Value::as_bool)
                .unwrap_or(false))),
        );
        e.insert(
            "sed_encryption".into(),
            vstr(yes(o
                .get("sed_encryption")
                .and_then(Value::as_bool)
                .unwrap_or(false))),
        );
        e.insert(
            "locked".into(),
            vstr(yes(!o
                .get("readable")
                .and_then(Value::as_bool)
                .unwrap_or(true))),
        );
        ve.push(e);
    };

    push_disk(
        &mut drives,
        &mut caps,
        &mut encs,
        &report["source_disk"],
        "1",
        "Source",
    );
    push_disk(
        &mut drives,
        &mut caps,
        &mut encs,
        &report["dest_disk"],
        "2",
        "Destination",
    );
    push_disk(
        &mut drives,
        &mut caps,
        &mut encs,
        &report["second_dest_disk"],
        "3",
        "Secondary Destination",
    );

    ctx.insert("drives", &drives);
    ctx.insert("capacities", &caps);
    ctx.insert("encryption", &encs);

    // ---------- partitions --------------------------------------------------
    let mut parts = Vec::<Map<String, Value>>::new();
    if let Some(arr) = sdisk.get("partitions").and_then(Value::as_array) {
        for p in arr {
            let o = p.as_object().unwrap();
            let start = gu(o, "start_sector");
            let end = gu(o, "end_sector");
            let size_mb = ((end + 1).saturating_sub(start)) as f64 * 512.0 / 1_048_576.0;
            let mut m = Map::new();
            m.insert(
                "index".into(),
                vu64(o.get("index").and_then(Value::as_u64).unwrap_or(0)),
            );
            m.insert("fs".into(), vstr(gs(o, "filesystem")));
            m.insert("start".into(), vu64(start));
            m.insert("end".into(), vu64(end));
            m.insert("size".into(), vstr(format!("{:.1} MB", size_mb)));
            parts.push(m);
        }
    }
    ctx.insert("source_partitions", &parts);

    println!("🚧  Renderuji Tera …");

    let latex = match Tera::one_off(TEMPLATE_EN_DD, &ctx, false) {
        Ok(l) => l,
        Err(err) => {
            eprintln!("❌  Tera render error: {:#?}", err);
            eprintln!("└── {}", err);
            return Err(format!("Render selhal: {err}"));
        }
    };

    // Unikátní dočasné cesty podle id
    let tex_path = format!("/tmp/report_{id}.tex");
    let pdf_path = format!("/tmp/report_{id}.pdf");

    // Zápis .tex
    fs::write(&tex_path, &latex).map_err(|e| e.to_string())?;

    // Spuštění tectonic
    let tectonic_status = std::process::Command::new("tectonic")
        .arg("--outdir")
        .arg("/tmp")
        .arg(&tex_path)
        .status()
        .map_err(|e| format!("Failed to run tectonic: {e}"))?;

    if !tectonic_status.success() {
        // Úklid
        let _ = fs::remove_file(&tex_path);
        return Err(format!(
            "Tectonic failed with exit code: {:?}",
            tectonic_status.code()
        ));
    }

    // Načtení PDF
    let pdf = fs::read(&pdf_path).map_err(|e| e.to_string())?;

    // Uložení na cílové zařízení
    let save_pdf = |key: &str, pdf: &[u8]| -> std::io::Result<()> {
        let disk = report[key].as_object().unwrap_or(&EMPTY_MAP);
        let base = disk
            .get("partitions")
            .and_then(Value::as_array)
            .and_then(|a| a.iter().find(|p| p.get("mountpoint").is_some()))
            .and_then(|p| p.get("mountpoint").and_then(Value::as_str))
            .map(|mp| {
                format!(
                    "{mp}/{}/{}/",
                    gs(&log_map, "case_number"),
                    gs(&log_map, "evidence_number")
                )
            });
        if let Some(mut dir) = base {
            fs::create_dir_all(&dir)?;
            dir.push_str("audit-report.pdf");
            fs::write(dir, pdf)?;
        }
        Ok(())
    };
    save_pdf("dest_disk", &pdf)
        .and(save_pdf("second_dest_disk", &pdf))
        .map_err(|e| e.to_string())?;

    // Úklid dočasných souborů
    let _ = fs::remove_file(&tex_path);
    let _ = fs::remove_file(&pdf_path);

    println!("🏁  generate_report({id}) – DONE");
    Ok(())
}

/// ------------ Načtení všech potřebných dat z DB --------------------------
pub fn get_report_json_data(copy_id: i64) -> Result<Value, String> {
    let mut pool = DB_POOL.get_connection().map_err(|e| e.to_string())?;
    let conn = pool.connection();

    // copy_process
    let cp_cols: Vec<String> = conn
        .prepare("SELECT * FROM copy_process LIMIT 1")
        .map_err(|e| e.to_string())?
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let copy_process = conn
        .query_row("SELECT * FROM copy_process WHERE id=?", [copy_id], |r| {
            Ok(row_to_json_with_cols(r, &cp_cols))
        })
        .map_err(|e| e.to_string())?;

    // trig id
    let (tr_ewf, tr_dd): (Option<i64>, Option<i64>) = conn
        .query_row(
            "SELECT triggered_by_ewf,triggered_by_dd FROM copy_process WHERE id=?",
            [copy_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    let (log_table, log_id, cfg_table) = if let Some(l) = tr_ewf {
        print!("VYBÍRÁM EWF");
        ("copy_log_ewf", l, "ewf_config")
    } else if let Some(l) = tr_dd {
        print!("VYBÍRÁM DCFldd");
        ("copy_log_dd", l, "dd_config")
    } else {
        return Err("copy_process nemá trigger".into());
    };

    // log_record
    let log_cols: Vec<String> = conn
        .prepare(&format!("SELECT * FROM {log_table} LIMIT 1"))
        .map_err(|e| e.to_string())?
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let log_record = conn
        .query_row(
            &format!("SELECT * FROM {log_table} WHERE id=?"),
            [log_id],
            |r| Ok(row_to_json_with_cols(r, &log_cols)),
        )
        .map_err(|e| e.to_string())?;

    // config_record
    let cfg_cols: Vec<String> = conn
        .prepare(&format!("SELECT * FROM {cfg_table} LIMIT 1"))
        .map_err(|e| e.to_string())?
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let cfg_id: i64 = conn
        .query_row(
            &format!("SELECT config_id FROM {log_table} WHERE id=?"),
            [log_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let cfg_record = conn
        .query_row(
            &format!("SELECT * FROM {cfg_table} WHERE id=?"),
            [cfg_id],
            |r| Ok(row_to_json_with_cols(r, &cfg_cols)),
        )
        .map_err(|e| e.to_string())?;

    let mut root = Map::new();
    root.insert("copy_process".into(), copy_process);
    root.insert("log_record".into(), log_record);
    root.insert("config_record".into(), cfg_record);

    Ok(Value::Object(root))
}
