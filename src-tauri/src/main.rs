mod dashboard_layout;
mod disk_info;
mod initial_setup;
mod logger;
mod db_structure;
use tauri::Builder;

fn main() {
    // Inicializace SQLite databáze pro logy
    if let Err(e) = logger::initialize_logging() {
        eprintln!("Logger initialization failed: {}", e);
    }
    
    Builder::default()
        .invoke_handler(tauri::generate_handler![
            initial_setup::find_file,
            initial_setup::fetch_integrity_data,
            initial_setup::verify_compatibility,
            initial_setup::check_integrity,
            dashboard_layout::get_device_status,
            disk_info::get_usb_device_details,
            disk_info::get_hdd_details,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
