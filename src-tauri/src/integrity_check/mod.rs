// src/aide_utils.rs
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::{Command, Stdio};

/// Struktura odpovídající klíčům v JSON reportu AIDE
#[derive(Debug, Deserialize, Serialize)]
pub struct AideReport {
    #[serde(rename = "number_of_entries")]
    pub counts: Counts,
    /// Změny v mapě <cesta, popis>
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
    pub added: u64,
    pub removed: u64,
    pub changed: u64,
}

/// Shrnutí, které vracíme Tauri front-endu
#[derive(Debug, Serialize)]
pub struct AideSummary {
    pub anything_changed: bool,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<String>,
}

#[tauri::command(rename_all = "snake_case")]
pub fn run_aide_check_json() -> Result<AideSummary, String> {
    // Spustíme AIDE se dvěma -A: umlčíme původní stdout a vytiskneme čistý JSON
    let aide_cmd = [
        "-A",
        "report_format=json",
        "-A",
        "report_url=stdout",
        "--check",
    ];

    let output = Command::new("sudo")
        .arg("aide")
        .args(&aide_cmd)
        .stdout(Stdio::piped())
        .output()
        .map_err(|e| format!("Cannot execute aide: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "AIDE exited with code {}",
            output.status.code().unwrap_or(-1)
        ));
    }

    // Parse JSON
    let report: AideReport = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Invalid JSON from AIDE: {}", e))?;

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

    Ok(AideSummary {
        anything_changed,
        added,
        removed,
        changed,
    })
}
