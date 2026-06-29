use tauri::State;

use crate::catalog::Catalog;
use crate::community_install::{confirm_community_install as run_confirm_install, validate_resolved_config};
use crate::harness::{create_instance, list_driver_infos, probe_all_instances};
use crate::db::Database;
use crate::detect::{
    check_all_dependencies, detect_all_clients, first_run_status, scan_existing_servers,
};
use crate::health::{build_mcp_server, probe_server};
use crate::install::InstallEngine;
use crate::models::{
    CommunityInstallJob, CommunityInstallMeta, DependencyStatus, DetectionResult,
    DiscoveredMcpEntry, DiscoverySearchResult, DiscoveryStatus, DiscoverySyncStats,
    ExistingServerInfo, FirstRunStatus, HarnessDriverInfo, HarnessInstanceRecord,
    HarnessSnapshot, HealthCheckRecord, InstallResult, InstallationRecord,
    IntegrationCatalogEntry, PluginCatalogEntry, PluginClientTargetRecord,
    PluginInstallationRecord, PluginInstallResult, ResolvedMcpConfig, SecretStatus,
};
use crate::plugin_install::PluginInstallEngine;
use crate::discovery::{get_discovered_mcp as discovery_get, get_discovery_status as discovery_status, run_sync, search_discovered_mcps as discovery_search};
use crate::secrets::{self, delete_secret, set_secret};
use crate::state::AppState;

fn with_db<F, T>(state: &State<AppState>, f: F) -> Result<T, String>
where
    F: FnOnce(&Database, &Catalog) -> Result<T, String>,
{
    let db = state.db.lock().map_err(|e| e.to_string())?;
    f(&db, &state.catalog)
}

fn with_plugin_db<F, T>(state: &State<AppState>, f: F) -> Result<T, String>
where
    F: FnOnce(&Database, &crate::plugin_catalog::PluginCatalog) -> Result<T, String>,
{
    let db = state.db.lock().map_err(|e| e.to_string())?;
    f(&db, &state.plugin_catalog)
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

        let server = if installation.source == "community" {
            let meta = db
                .get_community_install_meta(&installation_id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "Community install metadata not found".to_string())?;
            let mut server = meta.resolved_server.clone();
            let env = secrets::resolve_env_keys(&installation.integration_id, &meta.env_keys);
            server.env = env;
            server
        } else {
            let entry = catalog
                .get(&installation.integration_id)
                .ok_or_else(|| "Integration not found".to_string())?;
            let env = secrets::resolve_env(&installation.integration_id, &entry.secrets)
                .unwrap_or_default();
            build_mcp_server(catalog, &installation.integration_id, env)
                .map_err(|e| e.to_string())?
        };

        let integration_name = if installation.source == "community" {
            meta_name_from_community_id(&installation.integration_id, db)?
        } else {
            catalog
                .get(&installation.integration_id)
                .map(|e| e.name.clone())
                .unwrap_or_else(|| installation.integration_id.clone())
        };
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
            integration_name,
            latency_ms: probe.latency_ms,
            ok: probe.ok,
            checked_at: chrono::Utc::now().to_rfc3339(),
            detail: probe.detail,
        })
    })
}

fn meta_name_from_community_id(integration_id: &str, db: &Database) -> Result<String, String> {
    let discovered_id = integration_id
        .strip_prefix("community-")
        .unwrap_or(integration_id);
    if let Ok(Some(entry)) = db.get_discovered_mcp(discovered_id) {
        return Ok(entry.name);
    }
    Ok(discovered_id.to_string())
}

#[tauri::command]
pub fn run_all_health_checks(state: State<AppState>) -> Result<Vec<HealthCheckRecord>, String> {
    with_db(&state, |db, catalog| {
        let installations = db.list_installations().map_err(|e| e.to_string())?;
        let mut results = Vec::new();

        for installation in installations {
            let (server, integration_name) = if installation.source == "community" {
                let meta = db
                    .get_community_install_meta(&installation.id)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| "Community install metadata not found".to_string())?;
                let mut server = meta.resolved_server.clone();
                server.env =
                    secrets::resolve_env_keys(&installation.integration_id, &meta.env_keys);
                let name = meta_name_from_community_id(&installation.integration_id, db)?;
                (server, name)
            } else {
                let entry = catalog
                    .get(&installation.integration_id)
                    .ok_or_else(|| "Integration not found".to_string())?;
                let env = secrets::resolve_env(&installation.integration_id, &entry.secrets)
                    .unwrap_or_default();
                let server = build_mcp_server(catalog, &installation.integration_id, env)
                    .map_err(|e| e.to_string())?;
                (server, entry.name.clone())
            };
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
                integration_name,
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

#[tauri::command]
pub fn search_discovered_mcps(
    state: State<AppState>,
    query: String,
    sort: String,
    limit: i64,
    offset: i64,
) -> Result<DiscoverySearchResult, String> {
    with_db(&state, |db, _| {
        discovery_search(db, &query, &sort, limit, offset).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn get_discovered_mcp(
    state: State<AppState>,
    id: String,
) -> Result<Option<DiscoveredMcpEntry>, String> {
    with_db(&state, |db, _| {
        discovery_get(db, &id).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub async fn sync_discovered_catalog(
    app: tauri::AppHandle,
) -> Result<DiscoverySyncStats, String> {
    run_sync(&app).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_discovery_status(state: State<AppState>) -> Result<DiscoveryStatus, String> {
    with_db(&state, |db, _| discovery_status(db).map_err(|e| e.to_string()))
}

#[tauri::command]
pub fn get_plugin_catalog(state: State<AppState>) -> Result<Vec<PluginCatalogEntry>, String> {
    Ok(state.plugin_catalog.all().to_vec())
}

#[tauri::command]
pub fn get_plugin_installations(
    state: State<AppState>,
) -> Result<Vec<PluginInstallationRecord>, String> {
    with_plugin_db(&state, |db, _| {
        db.list_plugin_installations().map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn install_plugin(
    state: State<AppState>,
    plugin_id: String,
    client_ids: Vec<String>,
) -> Result<PluginInstallResult, String> {
    with_plugin_db(&state, |db, catalog| {
        PluginInstallEngine { db, catalog }.install(&plugin_id, client_ids)
    })
}

#[tauri::command]
pub fn uninstall_plugin(state: State<AppState>, installation_id: String) -> Result<(), String> {
    with_plugin_db(&state, |db, catalog| {
        PluginInstallEngine { db, catalog }.uninstall(&installation_id)
    })
}

#[tauri::command]
pub fn toggle_plugin_installation(
    state: State<AppState>,
    installation_id: String,
    enabled: bool,
) -> Result<(), String> {
    with_plugin_db(&state, |db, catalog| {
        PluginInstallEngine { db, catalog }.set_enabled(&installation_id, enabled)
    })
}

#[tauri::command]
pub fn get_plugin_client_targets(
    state: State<AppState>,
    installation_id: String,
) -> Result<Vec<PluginClientTargetRecord>, String> {
    with_plugin_db(&state, |db, _| {
        db.list_plugin_client_targets(&installation_id)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn list_harness_instances(state: State<AppState>) -> Result<Vec<HarnessInstanceRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.list_harness_instances().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_harness_drivers() -> Result<Vec<HarnessDriverInfo>, String> {
    Ok(list_driver_infos())
}

#[tauri::command]
pub fn create_harness_instance(
    state: State<AppState>,
    driver_kind: String,
    display_name: String,
    config_json: String,
) -> Result<HarnessInstanceRecord, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    create_instance(&db, &driver_kind, &display_name, &config_json)
}

#[tauri::command]
pub fn update_harness_instance(
    state: State<AppState>,
    instance_id: String,
    display_name: String,
    enabled: bool,
    config_json: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_harness_instance(&instance_id, &display_name, enabled, &config_json)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_harness_instance(state: State<AppState>, instance_id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_harness_instance(&instance_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn probe_harnesses(state: State<AppState>) -> Result<Vec<HarnessSnapshot>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    probe_all_instances(&db)
}

#[tauri::command]
pub fn set_default_install_agent(
    state: State<AppState>,
    instance_id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_default_install_agent(&instance_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_community_install(
    app: tauri::AppHandle,
    state: State<AppState>,
    discovered_mcp_id: String,
    harness_instance_id: String,
) -> Result<CommunityInstallJob, String> {
    state.community_install.start_job(
        app,
        state.db.clone(),
        discovered_mcp_id,
        harness_instance_id,
    )
}

#[tauri::command]
pub fn get_community_install_job(
    state: State<AppState>,
    job_id: String,
) -> Result<Option<CommunityInstallJob>, String> {
    Ok(state.community_install.get_job(&job_id))
}

#[tauri::command]
pub fn cancel_community_install(state: State<AppState>, job_id: String) -> Result<(), String> {
    state.community_install.cancel_job(&job_id)
}

#[tauri::command]
pub fn confirm_community_install_cmd(
    state: State<AppState>,
    job_id: String,
    client_ids: Vec<String>,
    secrets: std::collections::HashMap<String, String>,
    resolved_override: Option<ResolvedMcpConfig>,
) -> Result<InstallResult, String> {
    if let Some(ref config) = resolved_override {
        validate_resolved_config(config)?;
    }
    let db = state.db.lock().map_err(|e| e.to_string())?;
    run_confirm_install(
        &db,
        &state.community_install,
        &job_id,
        client_ids,
        secrets,
        resolved_override,
    )
}

#[tauri::command]
pub fn community_missing_dependencies(
    config: ResolvedMcpConfig,
) -> Vec<crate::deps::MissingDependency> {
    crate::deps::missing_dependencies(&config)
}

#[tauri::command]
pub fn install_dependencies_cmd(names: Vec<String>) -> Result<String, String> {
    crate::deps::install_dependencies(&names)
}

#[tauri::command]
pub fn get_community_install_meta(
    state: State<AppState>,
    installation_id: String,
) -> Result<Option<CommunityInstallMeta>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_community_install_meta(&installation_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_setting(state: State<AppState>, key: String) -> Result<Option<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(state: State<AppState>, key: String, value: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_setting(&key, &value).map_err(|e| e.to_string())
}
