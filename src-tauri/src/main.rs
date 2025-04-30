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
mod system_info;
mod integrity_check;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    db::initialize_db()?;
 
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
            system_info::get_program_versions,
            system_info::get_system_logs,
            integrity_check::run_aide_check_json,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}