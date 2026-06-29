use uuid::Uuid;

use crate::catalog::Catalog;
use crate::db::Database;
use crate::detect::{build_server_for_integration, check_dependencies, sync_to_clients};
use crate::health::{build_mcp_server, probe_server};
use crate::models::InstallResult;
use crate::secrets;

pub struct InstallEngine<'a> {
    pub db: &'a Database,
    pub catalog: &'a Catalog,
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
            .ok_or_else(|| format!("Integración no encontrada: {integration_id}"))?;

        if entry.coming_soon {
            return Err("Esta integración aún no está disponible".to_string());
        }

        let deps = check_dependencies(&entry.server.requires);
        let missing_deps: Vec<_> = deps.iter().filter(|d| !d.available).collect();
        if !missing_deps.is_empty() {
            let names: Vec<_> = missing_deps.iter().map(|d| d.name.as_str()).collect();
            return Err(format!(
                "Dependencias faltantes: {}. Instálalas antes de continuar.",
                names.join(", ")
            ));
        }

        let missing_secrets = secrets::missing_required_secrets(integration_id, &entry.secrets);
        if !missing_secrets.is_empty() {
            return Err(format!(
                "Configura los secretos requeridos antes de instalar: {}",
                missing_secrets.join(", ")
            ));
        }

        let server = build_server_for_integration(self.catalog, integration_id)?;

        if client_ids.is_empty() {
            return Err("Selecciona al menos un cliente".to_string());
        }

        sync_to_clients(&server, &client_ids, false)?;

        let existing = self
            .db
            .get_installation_by_integration(integration_id)
            .map_err(|e| e.to_string())?;
        let installation_id = existing
            .as_ref()
            .map(|i| i.id.clone())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let env = secrets::resolve_env(integration_id, &entry.secrets)
            .map_err(|e| e.to_string())?;
        let probe_server_def = build_mcp_server(self.catalog, integration_id, env)
            .map_err(|e| e.to_string())?;
        let probe = probe_server(&probe_server_def);

        let status = if probe.ok {
            "connected"
        } else {
            "error"
        };

        self.db
            .upsert_installation(
                &installation_id,
                integration_id,
                true,
                status,
                probe.detail.as_deref(),
            )
            .map_err(|e| e.to_string())?;

        for client_id in &client_ids {
            self.db
                .set_client_target(&installation_id, client_id, true)
                .map_err(|e| e.to_string())?;
        }

        self.db
            .insert_health_check(
                &installation_id,
                probe.latency_ms,
                probe.ok,
                probe.detail.as_deref(),
            )
            .map_err(|e| e.to_string())?;

        let message = if probe.ok {
            "Integración instalada correctamente".to_string()
        } else {
            format!(
                "Instalada con advertencias: {}",
                probe.detail.unwrap_or_default()
            )
        };

        Ok(InstallResult {
            installation_id,
            success: probe.ok,
            message,
        })
    }

    pub fn uninstall(&self, installation_id: &str) -> Result<(), String> {
        let installation = self
            .db
            .get_installation(installation_id)
            .map_err(|e| e.to_string())?;

        let server = build_server_for_integration(self.catalog, &installation.integration_id)
            .unwrap_or(crate::models::McpServer {
                id: format!("taro-{}", installation.integration_id),
                command: String::new(),
                args: vec![],
                env: Default::default(),
            });

        let targets = self
            .db
            .list_client_targets(installation_id)
            .map_err(|e| e.to_string())?;
        let client_ids: Vec<String> = targets
            .iter()
            .filter(|t| t.enabled)
            .map(|t| t.client_id.clone())
            .collect();

        if !client_ids.is_empty() {
            sync_to_clients(&server, &client_ids, true)?;
        }

        self.db
            .delete_installation(installation_id)
            .map_err(|e| e.to_string())?;
        Ok(())
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
