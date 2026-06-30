use crate::adapters::{all_adapters, get_adapter};
use crate::catalog::Catalog;
use crate::models::{
    ClientOperationResult, DependencyStatus, ExistingServerInfo, FirstRunStatus, McpServer,
};
use crate::secrets;

pub fn detect_all_clients() -> Vec<crate::models::DetectionResult> {
    crate::adapters::detect_all_clients()
}

pub fn scan_existing_servers() -> Vec<ExistingServerInfo> {
    let mut servers = Vec::new();
    for adapter in all_adapters() {
        let detection = adapter.detect();
        if !detection.detected {
            continue;
        }
        if let Ok(entries) = adapter.read_servers() {
            for server in entries {
                servers.push(ExistingServerInfo {
                    client_id: adapter.id().to_string(),
                    client_name: adapter.display_name().to_string(),
                    server_id: server.id,
                    command: server.command,
                    managed: false,
                });
            }
        }
    }
    servers
}

pub fn first_run_status(db_first_run: bool) -> FirstRunStatus {
    FirstRunStatus {
        completed: !db_first_run,
        detected_clients: detect_all_clients(),
        existing_servers: scan_existing_servers(),
    }
}

pub fn check_dependencies(requires: &[String]) -> Vec<DependencyStatus> {
    let tools: Vec<String> = if requires.is_empty() {
        vec!["node", "uv", "brew"]
            .into_iter()
            .map(str::to_string)
            .collect()
    } else {
        requires.to_vec()
    };

    tools
        .into_iter()
        .map(|name| {
            let path = which_path(&name);
            DependencyStatus {
                name: name.clone(),
                available: path.is_some(),
                path,
            }
        })
        .collect()
}

pub fn check_all_dependencies() -> Vec<DependencyStatus> {
    ["node", "uv", "brew"]
        .iter()
        .map(|name| {
            let path = which_path(name);
            DependencyStatus {
                name: (*name).to_string(),
                available: path.is_some(),
                path,
            }
        })
        .collect()
}

fn which_path(cmd: &str) -> Option<String> {
    std::process::Command::new("which")
        .arg(cmd)
        .env("PATH", crate::harness::util::login_shell_path())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn build_server_for_integration(
    catalog: &Catalog,
    integration_id: &str,
) -> Result<McpServer, String> {
    let entry = catalog
        .get(integration_id)
        .ok_or_else(|| format!("Integration not found: {integration_id}"))?;

    if entry.coming_soon {
        return Err("This integration is not available yet".to_string());
    }

    let missing = secrets::missing_required_secrets(integration_id, &entry.secrets);
    if !missing.is_empty() {
        return Err(format!("Missing required secrets: {}", missing.join(", ")));
    }

    let env = secrets::resolve_env(integration_id, &entry.secrets).map_err(|e| e.to_string())?;

    let mut args = entry.server.args.clone();
    if integration_id == "filesystem" {
        if let Some(home) = dirs::home_dir() {
            args.push(home.display().to_string());
        }
    }

    Ok(McpServer {
        // Use the integration's own name unchanged; ownership is tracked in the
        // managed_servers table, not by branding the id with a prefix.
        id: integration_id.to_string(),
        command: entry.server.command.clone(),
        args,
        env,
    })
}

/// Whether `client_id`'s config currently contains a server with `server_id`.
pub fn client_has_server(client_id: &str, server_id: &str) -> bool {
    get_adapter(client_id)
        .and_then(|a| a.read_servers().ok())
        .map(|servers| servers.iter().any(|s| s.id == server_id))
        .unwrap_or(false)
}

pub fn sync_to_clients(
    server: &McpServer,
    client_ids: &[String],
    remove: bool,
) -> Result<(), String> {
    for client_id in client_ids {
        let adapter =
            get_adapter(client_id).ok_or_else(|| format!("Cliente desconocido: {client_id}"))?;
        if remove {
            adapter
                .remove_server(&server.id)
                .map_err(|e| e.to_string())?;
        } else {
            adapter.write_server(server).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn remove_from_clients(
    server: &McpServer,
    client_ids: &[String],
) -> Vec<ClientOperationResult> {
    client_ids
        .iter()
        .map(|client_id| {
            let Some(adapter) = get_adapter(client_id) else {
                return ClientOperationResult {
                    client_id: client_id.clone(),
                    success: false,
                    message: format!("Cliente desconocido: {client_id}"),
                };
            };

            let existed = match adapter.remove_server(&server.id) {
                Ok(existed) => existed,
                Err(e) => {
                    return ClientOperationResult {
                        client_id: client_id.clone(),
                        success: false,
                        message: e.to_string(),
                    };
                }
            };

            // Re-read the written config and confirm the entry is actually gone,
            // so we never report a successful uninstall while an orphan remains.
            match adapter.read_servers() {
                Ok(servers) if servers.iter().any(|s| s.id == server.id) => ClientOperationResult {
                    client_id: client_id.clone(),
                    success: false,
                    message: format!(
                        "'{}' still present in {} after removal",
                        server.id,
                        adapter.display_name()
                    ),
                },
                Ok(_) if existed => ClientOperationResult {
                    client_id: client_id.clone(),
                    success: true,
                    message: format!("Removed from {} (verified)", adapter.display_name()),
                },
                Ok(_) => ClientOperationResult {
                    client_id: client_id.clone(),
                    success: true,
                    message: format!("Already absent from {}", adapter.display_name()),
                },
                Err(e) => ClientOperationResult {
                    client_id: client_id.clone(),
                    success: false,
                    message: format!(
                        "Could not verify removal from {}: {e}",
                        adapter.display_name()
                    ),
                },
            }
        })
        .collect()
}

pub fn sync_to_enabled_clients(
    server: &McpServer,
    client_ids: &[String],
    remove: bool,
) -> Result<(), String> {
    sync_to_clients(server, client_ids, remove)
}
