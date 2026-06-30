mod adapters;
mod catalog;
mod commands;
mod community_install;
mod debug_log;
mod db;
mod deps;
mod detect;
mod discovery;
mod harness;
mod health;
mod install;
mod models;
mod plugin_adapters;
mod plugin_catalog;
mod plugin_install;
mod secrets;
mod state;

use std::sync::Mutex;

use std::sync::Arc;

use tauri::Manager;

use catalog::Catalog;
use community_install::CommunityInstallManager;
use db::Database;
use discovery::{import_if_empty, maybe_background_sync};
use plugin_catalog::PluginCatalog;
use state::AppState;

fn migrate_github_token_to_keychain(db: &Database) {
    if let Ok(Some(token)) = db.get_setting("github_token") {
        if !token.is_empty() && secrets::get_app_secret("github_token").is_err() {
            let _ = secrets::set_app_secret("github_token", &token);
            let _ = db.delete_setting("github_token");
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Native macOS frosted-glass background so the sidebar can be translucent.
            #[cfg(target_os = "macos")]
            if let Some(window) = app.get_webview_window("main") {
                use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
                let _ = apply_vibrancy(
                    &window,
                    NSVisualEffectMaterial::Sidebar,
                    None,
                    None,
                );
            }

            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            let db_path = data_dir.join("taro.db");
            let db = Database::open(&db_path).expect("failed to open database");

            migrate_github_token_to_keychain(&db);

            let catalog = Catalog::load(app.handle())
                .or_else(|_| Catalog::from_embedded())
                .expect("failed to load catalog");

            let plugin_catalog = PluginCatalog::load(app.handle())
                .or_else(|_| PluginCatalog::from_embedded())
                .expect("failed to load plugin catalog");

            import_if_empty(&db, app.handle()).expect("failed to import discovered catalog");

            app.manage(AppState {
                db: Arc::new(Mutex::new(db)),
                catalog,
                plugin_catalog,
                community_install: CommunityInstallManager::default(),
            });

            maybe_background_sync(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_catalog,
            commands::get_installations,
            commands::install_integration,
            commands::uninstall_integration,
            commands::uninstall_integration_from_clients,
            commands::toggle_installation,
            commands::sync_installation,
            commands::set_client_target,
            commands::detect_clients,
            commands::scan_existing_mcp_servers,
            commands::force_remove_mcp_server,
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
            commands::search_discovered_mcps,
            commands::get_discovered_mcp,
            commands::sync_discovered_catalog,
            commands::get_discovery_status,
            commands::get_plugin_catalog,
            commands::get_plugin_installations,
            commands::install_plugin,
            commands::uninstall_plugin,
            commands::uninstall_plugin_from_clients,
            commands::toggle_plugin_installation,
            commands::get_plugin_client_targets,
            commands::list_harness_instances,
            commands::list_harness_drivers,
            commands::create_harness_instance,
            commands::update_harness_instance,
            commands::delete_harness_instance,
            commands::probe_harnesses,
            commands::set_default_install_agent,
            commands::start_community_install,
            commands::get_community_install_job,
            commands::cancel_community_install,
            commands::confirm_community_install_cmd,
            commands::community_missing_dependencies,
            commands::install_dependencies_cmd,
            commands::get_community_install_meta,
            commands::list_community_install_details,
            commands::get_github_token,
            commands::set_github_token,
            commands::remove_github_token,
            commands::get_setting,
            commands::set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
