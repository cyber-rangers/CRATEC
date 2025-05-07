// src/aide_utils.rs
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::{Command, Stdio};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

#[derive(Debug, Deserialize, Serialize)]
pub struct AideReport {
    #[serde(rename = "number_of_entries")]
    pub counts: Counts,
    #[serde(default)]
    pub added: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub removed: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub changed: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Counts {
    pub total: u64,
    #[serde(default)]
    pub added: Option<u64>,
    #[serde(default)]
    pub removed: Option<u64>,
    #[serde(default)]
    pub changed: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct AideSummary {
    pub anything_changed: bool,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<String>,
    pub status_message: String,
    pub raw_json: String, // přidáno
}

#[tauri::command(rename_all = "snake_case")]
pub async fn run_aide_check_json(app_handle: AppHandle) -> Result<AideSummary, String> {
    let aide_cmd = [
        "-A",
        "report_format=json",
        "-A",
        "report_url=stdout",
        "--check",
    ];

    let shell = app_handle.shell();
    let (mut rx, _child) = shell
        .command("sudo")
        .args(["aide"])
        .args(&aide_cmd)
        .spawn()
        .map_err(|e| format!("Cannot execute aide: {}", e))?;

    let mut stdout = String::new();
    let mut stderr = String::new();

    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(line) => {
                stdout.push_str(&String::from_utf8_lossy(&line));
            }
            CommandEvent::Stderr(line) => {
                stderr.push_str(&String::from_utf8_lossy(&line));
            }
            CommandEvent::Terminated(_exit) => {
                break;
            }
            _ => {}
        }
    }

    // Najdi první a poslední složenou závorku a vyřízni JSON blok
    let json_start = stdout.find('{');
    let mut brace_count = 0;
    let mut json_end = None;
    if let Some(start) = json_start {
        for (i, c) in stdout[start..].char_indices() {
            match c {
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        json_end = Some(start + i + 1);
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    let json_str = match (json_start, json_end) {
        (Some(start), Some(end)) if end > start => &stdout[start..end],
        _ => {
            return Err(format!("AIDE output does not contain valid JSON.\nFull output:\n{}", stdout));
        }
    };

    let report: AideReport = serde_json::from_str(json_str)
        .map_err(|e| format!("Invalid JSON from AIDE: {}\nJSON:\n{}", e, json_str))?;

    let added = report
        .added
        .unwrap_or_default()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    let removed = report
        .removed
        .unwrap_or_default()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    let changed = report
        .changed
        .unwrap_or_default()
        .keys()
        .cloned()
        .collect::<Vec<_>>();

    let anything_changed = !added.is_empty() || !removed.is_empty() || !changed.is_empty();

    let status_message = if anything_changed {
        "Systém byl změněn (něco přidáno, odebráno nebo upraveno)!".to_string()
    } else {
        "Systém je beze změn, vše v pořádku.".to_string()
    };

    Ok(AideSummary {
        anything_changed,
        added,
        removed,
        changed,
        status_message,
        raw_json: json_str.to_string(),
    })
}
