use tauri::Builder;

mod dashboard_layout;
mod disk_info;
mod initial_setup;
mod logger;
mod db;
mod copy_configs;

fn main() {
    db::initialize_db();
    logger::initialize_logging();

    Builder::default()
        .invoke_handler(tauri::generate_handler![
            initial_setup::find_file,
            initial_setup::fetch_integrity_data,
            initial_setup::verify_compatibility,
            initial_setup::check_integrity,
            dashboard_layout::get_device_status,
            disk_info::get_usb_device_details,
            disk_info::get_hdd_details,
            copy_configs::save_new_ewf_config,
            copy_configs::get_all_ewf_configs,
            copy_configs::delete_or_deactivate_ewf_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
