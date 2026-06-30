use uuid::Uuid;

use crate::catalog::Catalog;
use crate::db::Database;
use crate::detect::{
    build_server_for_integration, check_dependencies, remove_from_clients, sync_to_clients,
};
use crate::health::{build_mcp_server, probe_server};
use crate::models::{InstallResult, McpServer, UninstallResult};
use crate::secrets;

pub struct InstallEngine<'a> {
    pub db: &'a Database,
    pub catalog: &'a Catalog,
}

fn server_for_uninstall(
    db: &Database,
    catalog: &Catalog,
    installation_id: &str,
    integration_id: &str,
    source: &str,
) -> Result<McpServer, String> {
    if source == "community" {
        return Ok(db
            .get_community_install_meta(installation_id)
            .map_err(|e| e.to_string())?
            .map(|meta| meta.resolved_server)
            .unwrap_or(crate::models::McpServer {
                id: format!("taro-{integration_id}"),
                command: String::new(),
                args: vec![],
                env: Default::default(),
            }));
    }

    Ok(
        build_server_for_integration(catalog, integration_id).unwrap_or(crate::models::McpServer {
            id: format!("taro-{integration_id}"),
            command: String::new(),
            args: vec![],
            env: Default::default(),
        }),
    )
}

pub fn install_resolved_server(
    db: &Database,
    server: &McpServer,
    integration_id: &str,
    source: &str,
    client_ids: &[String],
) -> Result<InstallResult, String> {
    if client_ids.is_empty() {
        return Err("Select at least one client".to_string());
    }

    sync_to_clients(server, client_ids, false)?;

    let probe = probe_server(server);
    let status = if probe.ok { "connected" } else { "error" };

    let existing = db
        .get_installation_by_integration(integration_id)
        .map_err(|e| e.to_string())?;
    let installation_id = existing
        .as_ref()
        .map(|i| i.id.clone())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    db.upsert_installation_with_source(
        &installation_id,
        integration_id,
        true,
        status,
        probe.detail.as_deref(),
        source,
    )
    .map_err(|e| e.to_string())?;

    for client_id in client_ids {
        db.set_client_target(&installation_id, client_id, true)
            .map_err(|e| e.to_string())?;
    }

    db.insert_health_check(
        &installation_id,
        probe.latency_ms,
        probe.ok,
        probe.detail.as_deref(),
    )
    .map_err(|e| e.to_string())?;

    let message = if probe.ok {
        "Integration installed successfully".to_string()
    } else {
        format!(
            "Installed with warnings: {}",
            probe.detail.unwrap_or_default()
        )
    };

    Ok(InstallResult {
        installation_id,
        success: probe.ok,
        message,
    })
}

impl<'a> InstallEngine<'a> {
    pub fn install(
        &self,
        integration_id: &str,
        client_ids: Vec<String>,
    ) -> Result<InstallResult, String> {
        let entry = self
            .catalog
            .get(integration_id)
            .ok_or_else(|| format!("Integration not found: {integration_id}"))?;

        if entry.coming_soon {
            return Err("This integration is not available yet".to_string());
        }

        let deps = check_dependencies(&entry.server.requires);
        let missing_deps: Vec<_> = deps.iter().filter(|d| !d.available).collect();
        if !missing_deps.is_empty() {
            let names: Vec<_> = missing_deps.iter().map(|d| d.name.as_str()).collect();
            return Err(format!(
                "Missing dependencies: {}. Install them before continuing.",
                names.join(", ")
            ));
        }

        let missing_secrets = secrets::missing_required_secrets(integration_id, &entry.secrets);
        if !missing_secrets.is_empty() {
            return Err(format!(
                "Configure required secrets before installing: {}",
                missing_secrets.join(", ")
            ));
        }

        let server = build_server_for_integration(self.catalog, integration_id)?;

        if client_ids.is_empty() {
            return Err("Select at least one client".to_string());
        }

        let env =
            secrets::resolve_env(integration_id, &entry.secrets).map_err(|e| e.to_string())?;
        let probe_server_def =
            build_mcp_server(self.catalog, integration_id, env).map_err(|e| e.to_string())?;

        let result =
            install_resolved_server(self.db, &server, integration_id, "curated", &client_ids)?;

        let probe = probe_server(&probe_server_def);

        let message = if probe.ok {
            result.message
        } else {
            format!(
                "Installed with warnings: {}",
                probe.detail.unwrap_or_default()
            )
        };

        Ok(InstallResult {
            installation_id: result.installation_id,
            success: probe.ok,
            message,
        })
    }

    pub fn uninstall(&self, installation_id: &str) -> Result<UninstallResult, String> {
        let installation = self
            .db
            .get_installation(installation_id)
            .map_err(|e| e.to_string())?;

        let server = server_for_uninstall(
            self.db,
            self.catalog,
            installation_id,
            &installation.integration_id,
            &installation.source,
        )?;

        let targets = self
            .db
            .list_client_targets(installation_id)
            .map_err(|e| e.to_string())?;
        let client_ids: Vec<String> = targets
            .iter()
            .filter(|t| t.enabled)
            .map(|t| t.client_id.clone())
            .collect();

        let client_results = remove_from_clients(&server, &client_ids);
        let failures: Vec<_> = client_results.iter().filter(|r| !r.success).collect();

        if !failures.is_empty() {
            let message = failures
                .iter()
                .map(|r| format!("{}: {}", r.client_id, r.message))
                .collect::<Vec<_>>()
                .join("; ");
            self.db
                .update_installation_status(installation_id, "error", Some(&message))
                .map_err(|e| e.to_string())?;
            return Ok(UninstallResult {
                success: false,
                message: format!("Could not remove integration from all clients: {message}"),
                client_results,
            });
        }

        self.db
            .delete_installation(installation_id)
            .map_err(|e| e.to_string())?;
        let already_absent = client_results
            .iter()
            .any(|r| r.message.starts_with("Already absent"));
        let message = if client_ids.is_empty() {
            "Integration removed from Taro".to_string()
        } else if already_absent {
            "Integration removed; some clients were already clean".to_string()
        } else {
            "Integration removed successfully".to_string()
        };
        Ok(UninstallResult {
            success: true,
            message,
            client_results,
        })
    }

    pub fn set_enabled(&self, installation_id: &str, enabled: bool) -> Result<(), String> {
        let installation = self
            .db
            .get_installation(installation_id)
            .map_err(|e| e.to_string())?;

        let server = build_server_for_integration(self.catalog, &installation.integration_id)?;

        let targets = self
            .db
            .list_client_targets(installation_id)
            .map_err(|e| e.to_string())?;
        let client_ids: Vec<String> = targets
            .iter()
            .filter(|t| t.enabled)
            .map(|t| t.client_id.clone())
            .collect();

        sync_to_clients(&server, &client_ids, !enabled)?;

        self.db
            .set_installation_enabled(installation_id, enabled)
            .map_err(|e| e.to_string())?;

        let status = if enabled { "connected" } else { "disabled" };
        self.db
            .update_installation_status(installation_id, status, None)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn sync_installation(&self, installation_id: &str) -> Result<(), String> {
        let installation = self
            .db
            .get_installation(installation_id)
            .map_err(|e| e.to_string())?;

        if !installation.enabled {
            return Ok(());
        }

        let server = build_server_for_integration(self.catalog, &installation.integration_id)?;
        let targets = self
            .db
            .list_enabled_client_targets(installation_id)
            .map_err(|e| e.to_string())?;
        let client_ids: Vec<String> = targets.iter().map(|t| t.client_id.clone()).collect();

        sync_to_clients(&server, &client_ids, false)?;
        Ok(())
    }

    pub fn update_client_targets(
        &self,
        installation_id: &str,
        client_id: &str,
        enabled: bool,
    ) -> Result<(), String> {
        let installation = self
            .db
            .get_installation(installation_id)
            .map_err(|e| e.to_string())?;

        self.db
            .set_client_target(installation_id, client_id, enabled)
            .map_err(|e| e.to_string())?;

        let server = build_server_for_integration(self.catalog, &installation.integration_id)?;

        if installation.enabled {
            sync_to_clients(&server, &[client_id.to_string()], !enabled)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CommunityInstallMeta;

    #[test]
    fn community_uninstall_uses_saved_resolved_server_id() {
        let path =
            std::env::temp_dir().join(format!("taro-community-uninstall-{}.db", Uuid::new_v4()));
        let db = Database::open(&path).unwrap();
        db.upsert_installation_with_source(
            "installation-1",
            "community-example",
            true,
            "connected",
            None,
            "community",
        )
        .unwrap();
        db.upsert_community_install_meta(&CommunityInstallMeta {
            installation_id: "installation-1".to_string(),
            discovered_mcp_id: "example".to_string(),
            harness_instance_id: "harness-1".to_string(),
            resolved_server: McpServer {
                id: "taro-community-example".to_string(),
                command: "node".to_string(),
                args: vec!["server.js".to_string()],
                env: Default::default(),
            },
            env_keys: vec![],
            agent_log: None,
        })
        .unwrap();

        let catalog = Catalog::from_entries(vec![]);
        let server = server_for_uninstall(
            &db,
            &catalog,
            "installation-1",
            "community-example",
            "community",
        )
        .unwrap();

        assert_eq!(server.id, "taro-community-example");
        let _ = std::fs::remove_file(path);
    }
}
