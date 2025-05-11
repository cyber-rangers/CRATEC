use once_cell::sync::Lazy;
use std::sync::RwLock;
use crate::logger::{log_debug, log_warn};


/// Globální úložiště pro kód zámku.
static LOCK_CODE: Lazy<RwLock<Option<String>>> = Lazy::new(|| RwLock::new(None));

/// Zamkne systém zadaným kódem (4–6 číslic).
/// Vrací Err pokud kód nevyhovuje formátu.
#[tauri::command(rename_all = "snake_case")]
pub fn lock_system(code: &str) -> Result<(), String> {
    // validace: délka 4–6, jen čísla
    if !(4..=6).contains(&code.len()) || !code.chars().all(|c| c.is_ascii_digit()) {
        return Err("Kód musí obsahovat 4 až 6 číslic.".into());
    }
    {
        let mut guard = LOCK_CODE.write().unwrap();
        *guard = Some(code.to_string());
    }
    log_debug(&format!("systém zamčen kódem {}", code));
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn unlock_system(code: &str) -> bool {
    let mut guard = LOCK_CODE.write().unwrap();
    match guard.as_deref() {
        Some(stored) if stored == code => {
            *guard = None;
            log_debug(&format!("systém odemčen kódem {}", code));
            true
        }
        _ => {
            log_warn(&format!("neplatný odemykací kód: {}", code));
            false
        }
    }
}