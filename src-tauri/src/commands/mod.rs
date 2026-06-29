use tauri::State;

use crate::catalog::Catalog;
use crate::db::Database;
use crate::detect::{
    check_all_dependencies, detect_all_clients, first_run_status, scan_existing_servers,
};
use crate::health::{build_mcp_server, probe_server};
use crate::install::InstallEngine;
use crate::models::{
    DependencyStatus, DetectionResult, ExistingServerInfo, FirstRunStatus,
    HealthCheckRecord, InstallResult, InstallationRecord, IntegrationCatalogEntry,
    SecretStatus,
};
use crate::secrets::{self, delete_secret, set_secret};
use crate::state::AppState;

fn with_db<F, T>(state: &State<AppState>, f: F) -> Result<T, String>
where
    F: FnOnce(&Database, &Catalog) -> Result<T, String>,
{
    let db = state.db.lock().map_err(|e| e.to_string())?;
    f(&db, &state.catalog)
}

#[tauri::command]
pub fn get_catalog(state: State<AppState>) -> Result<Vec<IntegrationCatalogEntry>, String> {
    Ok(state.catalog.all().to_vec())
}

#[tauri::command]
pub fn get_installations(state: State<AppState>) -> Result<Vec<InstallationRecord>, String> {
    with_db(&state, |db, _| {
        db.list_installations().map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn install_integration(
    state: State<AppState>,
    integration_id: String,
    client_ids: Vec<String>,
) -> Result<InstallResult, String> {
    with_db(&state, |db, catalog| {
        InstallEngine { db, catalog }.install(&integration_id, client_ids)
    })
}

#[tauri::command]
pub fn uninstall_integration(state: State<AppState>, installation_id: String) -> Result<(), String> {
    with_db(&state, |db, catalog| {
        InstallEngine { db, catalog }.uninstall(&installation_id)
    })
}

#[tauri::command]
pub fn toggle_installation(
    state: State<AppState>,
    installation_id: String,
    enabled: bool,
) -> Result<(), String> {
    with_db(&state, |db, catalog| {
        InstallEngine { db, catalog }.set_enabled(&installation_id, enabled)
    })
}

#[tauri::command]
pub fn sync_installation(state: State<AppState>, installation_id: String) -> Result<(), String> {
    with_db(&state, |db, catalog| {
        InstallEngine { db, catalog }.sync_installation(&installation_id)
    })
}

#[tauri::command]
pub fn set_client_target(
    state: State<AppState>,
    installation_id: String,
    client_id: String,
    enabled: bool,
) -> Result<(), String> {
    with_db(&state, |db, catalog| {
        InstallEngine { db, catalog }.update_client_targets(
            &installation_id,
            &client_id,
            enabled,
        )
    })
}

#[tauri::command]
pub fn detect_clients() -> Result<Vec<DetectionResult>, String> {
    Ok(detect_all_clients())
}

#[tauri::command]
pub fn scan_existing_mcp_servers() -> Result<Vec<ExistingServerInfo>, String> {
    Ok(scan_existing_servers())
}

#[tauri::command]
pub fn get_dependencies() -> Result<Vec<DependencyStatus>, String> {
    Ok(check_all_dependencies())
}

#[tauri::command]
pub fn get_first_run_status(state: State<AppState>) -> Result<FirstRunStatus, String> {
    with_db(&state, |db, _| {
        let is_first = db.is_first_run().map_err(|e| e.to_string())?;
        Ok(first_run_status(is_first))
    })
}

#[tauri::command]
pub fn complete_first_run(state: State<AppState>) -> Result<(), String> {
    with_db(&state, |db, _| {
        db.complete_first_run().map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn get_secrets_status(state: State<AppState>) -> Result<Vec<SecretStatus>, String> {
    let mut statuses = Vec::new();
    for entry in state.catalog.all() {
        for secret in &entry.secrets {
            statuses.push(SecretStatus {
                integration_id: entry.id.clone(),
                integration_name: entry.name.clone(),
                secret_key: secret.key.clone(),
                label: secret.label.clone(),
                required: secret.required,
                connected: secrets::has_secret(&entry.id, &secret.key),
            });
        }
    }
    Ok(statuses)
}

#[tauri::command]
pub fn save_secret(
    integration_id: String,
    secret_key: String,
    value: String,
) -> Result<(), String> {
    set_secret(&integration_id, &secret_key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_secret(integration_id: String, secret_key: String) -> Result<(), String> {
    delete_secret(&integration_id, &secret_key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn run_health_check(
    state: State<AppState>,
    installation_id: String,
) -> Result<HealthCheckRecord, String> {
    with_db(&state, |db, catalog| {
        let installation = db.get_installation(&installation_id).map_err(|e| e.to_string())?;
        let entry = catalog
            .get(&installation.integration_id)
            .ok_or_else(|| "Integración no encontrada".to_string())?;

        let env = secrets::resolve_env(&installation.integration_id, &entry.secrets)
            .unwrap_or_default();
        let server = build_mcp_server(catalog, &installation.integration_id, env)
            .map_err(|e| e.to_string())?;
        let probe = probe_server(&server);

        db.insert_health_check(
            &installation_id,
            probe.latency_ms,
            probe.ok,
            probe.detail.as_deref(),
        )
        .map_err(|e| e.to_string())?;

        let status = if probe.ok { "connected" } else { "error" };
        db.update_installation_status(&installation_id, status, probe.detail.as_deref())
            .map_err(|e| e.to_string())?;

        Ok(HealthCheckRecord {
            installation_id: installation_id.clone(),
            integration_id: installation.integration_id.clone(),
            integration_name: entry.name.clone(),
            latency_ms: probe.latency_ms,
            ok: probe.ok,
            checked_at: chrono::Utc::now().to_rfc3339(),
            detail: probe.detail,
        })
    })
}

#[tauri::command]
pub fn run_all_health_checks(state: State<AppState>) -> Result<Vec<HealthCheckRecord>, String> {
    with_db(&state, |db, catalog| {
        let installations = db.list_installations().map_err(|e| e.to_string())?;
        let mut results = Vec::new();

        for installation in installations {
            let entry = catalog
                .get(&installation.integration_id)
                .ok_or_else(|| "Integración no encontrada".to_string())?;

            let env = secrets::resolve_env(&installation.integration_id, &entry.secrets)
                .unwrap_or_default();
            let server = build_mcp_server(catalog, &installation.integration_id, env)
                .map_err(|e| e.to_string())?;
            let probe = probe_server(&server);

            db.insert_health_check(
                &installation.id,
                probe.latency_ms,
                probe.ok,
                probe.detail.as_deref(),
            )
            .map_err(|e| e.to_string())?;

            let status = if probe.ok { "connected" } else { "error" };
            db.update_installation_status(&installation.id, status, probe.detail.as_deref())
                .map_err(|e| e.to_string())?;

            results.push(HealthCheckRecord {
                installation_id: installation.id,
                integration_id: installation.integration_id.clone(),
                integration_name: entry.name.clone(),
                latency_ms: probe.latency_ms,
                ok: probe.ok,
                checked_at: chrono::Utc::now().to_rfc3339(),
                detail: probe.detail,
            });
        }

        Ok(results)
    })
}

#[tauri::command]
pub fn get_health_status(state: State<AppState>) -> Result<Vec<HealthCheckRecord>, String> {
    with_db(&state, |db, catalog| {
        let mut checks = db.latest_health_checks().map_err(|e| e.to_string())?;
        for check in &mut checks {
            if let Some(entry) = catalog.get(&check.integration_id) {
                check.integration_name = entry.name.clone();
            }
        }
        Ok(checks)
    })
}

#[tauri::command]
pub fn get_client_targets(
    state: State<AppState>,
    installation_id: String,
) -> Result<Vec<crate::models::ClientTargetRecord>, String> {
    with_db(&state, |db, _| {
        db.list_client_targets(&installation_id)
            .map_err(|e| e.to_string())
    })
}
