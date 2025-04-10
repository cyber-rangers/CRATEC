use tauri::Manager;
use std::process::Command;

#[tauri::command]
pub fn shutdown_system() -> Result<(), String> {
  // Windows: vypnutí, Linux/macOS: vypnutí
  #[cfg(target_os = "windows")]
  {
    Command::new("shutdown")
      .args(&["/s", "/t", "0"])
      .spawn()
      .map_err(|e| e.to_string())?;
  }
  #[cfg(any(target_os = "linux", target_os = "macos"))]
  {
    Command::new("shutdown")
      .args(&["-h", "now"])
      .spawn()
      .map_err(|e| e.to_string())?;
  }
  Ok(())
}

#[tauri::command]
pub fn restart_system() -> Result<(), String> {
  // Windows: restart, Linux/macOS: restart
  #[cfg(target_os = "windows")]
  {
    Command::new("shutdown")
      .args(&["/r", "/t", "0"])
      .spawn()
      .map_err(|e| e.to_string())?;
  }
  #[cfg(any(target_os = "linux", target_os = "macos"))]
  {
    Command::new("shutdown")
      .args(&["-r", "now"])
      .spawn()
      .map_err(|e| e.to_string())?;
  }
  Ok(())
}
