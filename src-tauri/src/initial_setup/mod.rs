use crate::logger::{log_debug, log_error, log_warn}; // Import vlastního loggeru
use std::fs::File;
use std::io::Read;
use ring::signature;
use base64::{engine::general_purpose, Engine};
use tauri::command;
use tokio::time::{sleep, Duration};
use serde_json::Value;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref INTEGRITY_DATA: Mutex<Option<Value>> = Mutex::new(None);
}

#[command]
pub async fn find_file() -> bool {
    log_debug("Starting find_file command.");
    sleep(Duration::from_secs(2)).await;
    log_debug("find_file command completed.");
    true 
}

#[command]
pub async fn fetch_integrity_data() -> Result<Value, String> {
    log_debug("Starting fetch_integrity_data command.");
    sleep(Duration::from_secs(2)).await;
    let data = INTEGRITY_DATA.lock().unwrap();
    if let Some(json_data) = &*data {
        log_debug("Integrity data found, returning data.");
        Ok(json_data.clone())
    } else {
        log_warn("No integrity data available.");
        Err("No data available.".to_string())
    }
}

#[command]
pub async fn verify_compatibility() {
    log_debug("Starting verify_compatibility command.");
    sleep(Duration::from_secs(2)).await;
    let data = INTEGRITY_DATA.lock().unwrap();
    if let Some(json_data) = &*data {
        log_debug(&format!("Compatibility check: {}", json_data));
    } else {
        log_warn("No integrity data available for compatibility check.");
    }
    log_debug("verify_compatibility command completed.");
}

const PUBLIC_KEY_BASE64: &str = "PojGI/Yadm2jtD5uJCvQehy9bU7aWqkRmlfMXxVYkpA="; 

#[command]
pub async fn check_integrity() -> bool {
    log_debug("Starting check_integrity command.");
    sleep(Duration::from_secs(2)).await;

    fn inner() -> Option<()> {
        let file_path = "/etc/CRATEC/integrity.crkcfg";
        let signature_path = "/etc/CRATEC/integrity_signature.sig";

        log_debug("Attempting to read public key.");
        let public_key_bytes = general_purpose::STANDARD.decode(PUBLIC_KEY_BASE64).ok()?;
        let peer_public_key = signature::UnparsedPublicKey::new(&signature::ED25519, &public_key_bytes);

        log_debug("Attempting to read integrity configuration file.");
        let mut file = File::open(file_path).ok()?;
        let mut file_data = vec![];
        file.read_to_end(&mut file_data).ok()?;

        log_debug("Attempting to read signature file.");
        let mut signature_file = File::open(signature_path).ok()?;
        let mut signature_bytes = vec![];
        signature_file.read_to_end(&mut signature_bytes).ok()?;

        log_debug("Verifying signature.");
        peer_public_key.verify(&file_data, &signature_bytes).ok()?;

        log_debug("Parsing integrity configuration data.");
        let json_data: Value = serde_json::from_slice(&file_data).ok()?;
        let mut data = INTEGRITY_DATA.lock().unwrap();
        *data = Some(json_data);

        log_debug("Integrity check passed and data stored.");
        Some(())
    }

    let result = inner().is_some();
    if result {
        log_debug("check_integrity command completed successfully.");
    } else {
        log_error("check_integrity command failed.");
    }
    result
}