mod adapters;
mod catalog;
mod commands;
mod db;
mod detect;
mod health;
mod install;
mod models;
mod secrets;
mod state;

use std::sync::Mutex;

use tauri::Manager;

use catalog::Catalog;
use db::Database;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            let db_path = data_dir.join("taro.db");
            let db = Database::open(&db_path).expect("failed to open database");

            let catalog = Catalog::load(app.handle())
                .or_else(|_| Catalog::from_embedded())
                .expect("failed to load catalog");

            app.manage(AppState {
                db: Mutex::new(db),
                catalog,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_catalog,
            commands::get_installations,
            commands::install_integration,
            commands::uninstall_integration,
            commands::toggle_installation,
            commands::sync_installation,
            commands::set_client_target,
            commands::detect_clients,
            commands::scan_existing_mcp_servers,
            commands::get_dependencies,
            commands::get_first_run_status,
            commands::complete_first_run,
            commands::get_secrets_status,
            commands::save_secret,
            commands::remove_secret,
            commands::run_health_check,
            commands::run_all_health_checks,
            commands::get_health_status,
            commands::get_client_targets,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
