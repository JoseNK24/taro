use crate::adapters::{get_adapter, all_adapters};
use crate::catalog::Catalog;
use crate::models::{
    DependencyStatus, ExistingServerInfo, FirstRunStatus, McpServer,
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
        return Err(format!(
            "Missing required secrets: {}",
            missing.join(", ")
        ));
    }

    let env = secrets::resolve_env(integration_id, &entry.secrets)
        .map_err(|e| e.to_string())?;

    let mut args = entry.server.args.clone();
    if integration_id == "filesystem" {
        if let Some(home) = dirs::home_dir() {
            args.push(home.display().to_string());
        }
    }

    Ok(McpServer {
        id: format!("taro-{integration_id}"),
        command: entry.server.command.clone(),
        args,
        env,
    })
}

pub fn sync_to_clients(
    server: &McpServer,
    client_ids: &[String],
    remove: bool,
) -> Result<(), String> {
    for client_id in client_ids {
        let adapter = get_adapter(client_id)
            .ok_or_else(|| format!("Cliente desconocido: {client_id}"))?;
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

pub fn sync_to_enabled_clients(
    server: &McpServer,
    client_ids: &[String],
    remove: bool,
) -> Result<(), String> {
    sync_to_clients(server, client_ids, remove)
}
