use tauri::Builder;
use tauri_plugin_websocket::init as websocket_init;
use tauri_plugin_shell::init as shell_init;
mod copy_configs;
mod dashboard_layout;
mod db;
mod disk_manager;
mod ewfacquire;
mod dcfldd;
mod initial_setup;
mod logger;
mod power_actions;
mod websocket;
mod report;
mod led;
mod disk_utils;
mod config;
mod lockscreen;
mod history;
mod app_info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the database without passing a connection
    // The function now gets its own connection from the pool
    db::initialize_db()?;

    //TEST
    let device = "/dev/sdc";

    // Test funkce detect_hpa_dco
    
    

    /* // Test funkce get_disk_info
    match disk_utils::get_disk_info(device) {
        Ok(disk_info) => {
            println!("Disk Info:");
            println!("Serial: {}", disk_info.serial);
            println!("Capacity (bytes): {}", disk_info.capacity_bytes);
            println!("Partitions: {:?}", disk_info.partitions);
            println!("ATA Encryption: {:?}", disk_info.ata_encryption);
            println!("HPA: {}", disk_info.has_hpa);
            println!("DCO: {}", disk_info.dco);
        }
        Err(e) => {
            println!("Failed to get disk info: {}", e);
        }
    }
    // Konec TEST */


   report::generate_report_dcfldd(1)?;
    
    

    Builder::default()
        .plugin(tauri_plugin_websocket::init())
        .plugin(shell_init())
        .invoke_handler(tauri::generate_handler![
            initial_setup::find_file,
            initial_setup::fetch_integrity_data,
            initial_setup::verify_compatibility,
            initial_setup::check_integrity,
            dashboard_layout::get_device_status,
            websocket::start_websocket_server,
            copy_configs::save_new_ewf_config,
            copy_configs::get_all_active_configs,
            copy_configs::delete_or_deactivate_config,
            copy_configs::save_new_dd_config,
            ewfacquire::run_ewfacquire,
            dcfldd::run_dcfldd,
            disk_manager::get_directory_contents,
            power_actions::shutdown_system,
            power_actions::restart_system,
            disk_utils::get_lsblk_json,
            disk_utils::get_disk_info,
            lockscreen::lock_system,
            lockscreen::unlock_system,
            history::get_history,
            history::get_config_entry,
            history::get_process_log_lines_texts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}