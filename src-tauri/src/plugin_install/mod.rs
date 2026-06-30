use uuid::Uuid;

use crate::adapters::detect_all_clients;
use crate::db::Database;
use crate::models::{
    ClientOperationResult, PluginClientInstallResult, PluginInstallResult, PluginInstallStrategy,
    UninstallResult,
};
use crate::plugin_adapters::{
    check_cli_for_client, check_plugin_dependencies, effective_strategy, install_plugin_for_client,
    is_strategy_supported, strategy_for_client, uninstall_plugin_for_client,
};
use crate::plugin_catalog::PluginCatalog;

pub struct PluginInstallEngine<'a> {
    pub db: &'a Database,
    pub catalog: &'a PluginCatalog,
}

impl<'a> PluginInstallEngine<'a> {
    pub fn install(
        &self,
        plugin_id: &str,
        client_ids: Vec<String>,
    ) -> Result<PluginInstallResult, String> {
        let entry = self
            .catalog
            .get(plugin_id)
            .ok_or_else(|| format!("Plugin no encontrado: {plugin_id}"))?;

        if entry.coming_soon {
            return Err("Este plugin aún no está disponible".to_string());
        }

        if client_ids.is_empty() {
            return Err("Selecciona al menos un cliente".to_string());
        }

        let detected = detect_all_clients();
        let mut client_results = Vec::new();

        for client_id in &client_ids {
            let detection = detected.iter().find(|d| &d.client_id == client_id);
            let sync_supported = detection.map(|d| d.sync_supported).unwrap_or(false);
            let strategy = effective_strategy(entry, client_id, sync_supported);

            if !is_strategy_supported(&strategy) {
                let message = match &strategy {
                    PluginInstallStrategy::ComingSoon => {
                        "Próximamente para este cliente".to_string()
                    }
                    _ => "Sincronización no disponible para este cliente".to_string(),
                };
                client_results.push(PluginClientInstallResult {
                    client_id: client_id.clone(),
                    success: false,
                    message,
                });
                continue;
            }

            let Some(det) = detection else {
                client_results.push(PluginClientInstallResult {
                    client_id: client_id.clone(),
                    success: false,
                    message: "Cliente no reconocido".to_string(),
                });
                continue;
            };

            if !det.detected {
                client_results.push(PluginClientInstallResult {
                    client_id: client_id.clone(),
                    success: false,
                    message: format!("{} no está instalado en este sistema", det.display_name),
                });
                continue;
            }

            if let Err(e) = check_plugin_dependencies(&strategy) {
                return Err(e);
            }
            if let Err(e) = check_cli_for_client(client_id, &strategy) {
                client_results.push(PluginClientInstallResult {
                    client_id: client_id.clone(),
                    success: false,
                    message: e,
                });
                continue;
            }

            match install_plugin_for_client(entry, client_id, &strategy) {
                Ok(()) => {
                    client_results.push(PluginClientInstallResult {
                        client_id: client_id.clone(),
                        success: true,
                        message: format!("Instalado en {}", det.display_name),
                    });
                }
                Err(e) => {
                    client_results.push(PluginClientInstallResult {
                        client_id: client_id.clone(),
                        success: false,
                        message: e.to_string(),
                    });
                }
            }
        }

        let successful: Vec<_> = client_results.iter().filter(|r| r.success).collect();
        if successful.is_empty() {
            let messages: Vec<_> = client_results
                .iter()
                .map(|r| format!("{}: {}", r.client_id, r.message))
                .collect();
            return Err(messages.join("; "));
        }

        let existing = self
            .db
            .get_plugin_installation_by_plugin(plugin_id)
            .map_err(|e| e.to_string())?;
        let installation_id = existing
            .as_ref()
            .map(|i| i.id.clone())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let has_errors = client_results.iter().any(|r| !r.success);
        let status = if has_errors { "error" } else { "installed" };
        let error_message = if has_errors {
            Some(
                client_results
                    .iter()
                    .filter(|r| !r.success)
                    .map(|r| format!("{}: {}", r.client_id, r.message))
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        } else {
            None
        };

        self.db
            .upsert_plugin_installation(
                &installation_id,
                plugin_id,
                true,
                status,
                error_message.as_deref(),
            )
            .map_err(|e| e.to_string())?;

        for result in &client_results {
            if result.success {
                self.db
                    .set_plugin_client_target(
                        &installation_id,
                        &result.client_id,
                        true,
                        "installed",
                        None,
                    )
                    .map_err(|e| e.to_string())?;
            }
        }

        let message = if has_errors {
            format!(
                "Plugin instalado con advertencias en {} cliente(s)",
                successful.len()
            )
        } else {
            "Plugin instalado correctamente".to_string()
        };

        Ok(PluginInstallResult {
            installation_id,
            success: !has_errors,
            message,
            client_results,
        })
    }

    pub fn uninstall(&self, installation_id: &str) -> Result<UninstallResult, String> {
        let targets = self
            .db
            .list_plugin_client_targets(installation_id)
            .map_err(|e| e.to_string())?;
        let client_ids = targets
            .iter()
            .filter(|t| t.enabled)
            .map(|t| t.client_id.clone())
            .collect();
        self.uninstall_from_clients(installation_id, client_ids)
    }

    pub fn uninstall_from_clients(
        &self,
        installation_id: &str,
        client_ids: Vec<String>,
    ) -> Result<UninstallResult, String> {
        if client_ids.is_empty() {
            return Err("Select at least one client".to_string());
        }

        let installation = self
            .db
            .get_plugin_installation(installation_id)
            .map_err(|e| e.to_string())?;

        let entry = self
            .catalog
            .get(&installation.plugin_id)
            .ok_or_else(|| format!("Plugin no encontrado: {}", installation.plugin_id))?;

        let mut client_results = Vec::new();
        for client_id in &client_ids {
            let Some(strategy) = strategy_for_client(entry, client_id) else {
                client_results.push(ClientOperationResult {
                    client_id: client_id.clone(),
                    success: false,
                    message: "No uninstall strategy is configured for this client".to_string(),
                });
                continue;
            };

            match uninstall_plugin_for_client(entry, client_id, strategy) {
                Ok(()) => {
                    self.db
                        .set_plugin_client_target(
                            installation_id,
                            client_id,
                            false,
                            "removed",
                            None,
                        )
                        .map_err(|e| e.to_string())?;
                    client_results.push(ClientOperationResult {
                        client_id: client_id.clone(),
                        success: true,
                        message: "Plugin uninstalled".to_string(),
                    });
                }
                Err(e) => client_results.push(ClientOperationResult {
                    client_id: client_id.clone(),
                    success: false,
                    message: e.to_string(),
                }),
            }
        }

        let failures: Vec<_> = client_results.iter().filter(|r| !r.success).collect();
        if !failures.is_empty() {
            let message = failures
                .iter()
                .map(|r| format!("{}: {}", r.client_id, r.message))
                .collect::<Vec<_>>()
                .join("; ");
            self.db
                .update_plugin_installation_status(installation_id, "error", Some(&message))
                .map_err(|e| e.to_string())?;
            return Ok(UninstallResult {
                success: false,
                message: format!("Could not uninstall plugin from all clients: {message}"),
                client_results,
            });
        }

        let remaining_targets = self
            .db
            .list_plugin_client_targets(installation_id)
            .map_err(|e| e.to_string())?
            .into_iter()
            .filter(|t| t.enabled)
            .collect::<Vec<_>>();
        if remaining_targets.is_empty() {
            self.db
                .delete_plugin_installation(installation_id)
                .map_err(|e| e.to_string())?;
        }
        let message = if remaining_targets.is_empty() {
            "Plugin uninstalled successfully".to_string()
        } else {
            "Plugin uninstalled from selected clients".to_string()
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
            .get_plugin_installation(installation_id)
            .map_err(|e| e.to_string())?;

        let entry = self
            .catalog
            .get(&installation.plugin_id)
            .ok_or_else(|| format!("Plugin no encontrado: {}", installation.plugin_id))?;

        let targets = self
            .db
            .list_enabled_plugin_client_targets(installation_id)
            .map_err(|e| e.to_string())?;

        if enabled {
            for target in &targets {
                if let Some(strategy) = strategy_for_client(entry, &target.client_id) {
                    install_plugin_for_client(entry, &target.client_id, strategy)
                        .map_err(|e| e.to_string())?;
                }
            }
        } else {
            for target in &targets {
                if let Some(strategy) = strategy_for_client(entry, &target.client_id) {
                    let _ = uninstall_plugin_for_client(entry, &target.client_id, strategy);
                }
            }
        }

        self.db
            .set_plugin_installation_enabled(installation_id, enabled)
            .map_err(|e| e.to_string())?;

        let status = if enabled { "installed" } else { "disabled" };
        self.db
            .update_plugin_installation_status(installation_id, status, None)
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{PluginCatalogEntry, PluginMarketplace};
    use std::collections::HashMap;

    fn test_db(name: &str) -> (Database, std::path::PathBuf) {
        let path = std::env::temp_dir().join(format!("{name}-{}.db", Uuid::new_v4()));
        (Database::open(&path).unwrap(), path)
    }

    fn plugin_entry(strategy: Option<PluginInstallStrategy>) -> PluginCatalogEntry {
        let mut client_install = HashMap::new();
        if let Some(strategy) = strategy {
            client_install.insert("codex".to_string(), strategy);
        }
        PluginCatalogEntry {
            id: "plugin-a".to_string(),
            name: "Plugin A".to_string(),
            description: "Test plugin".to_string(),
            tags: vec![],
            icon: "plug".to_string(),
            coming_soon: false,
            github_url: "https://example.com/plugin-a.git".to_string(),
            github_stars: 0,
            marketplace: PluginMarketplace {
                source: "https://example.com".to_string(),
                plugin_id: "plugin-a".to_string(),
                marketplace_name: "test".to_string(),
            },
            client_install,
        }
    }

    #[test]
    fn plugin_uninstall_keeps_db_record_when_client_fails() {
        let (db, path) = test_db("taro-plugin-uninstall-fails");
        db.upsert_plugin_installation("installation-1", "plugin-a", true, "installed", None)
            .unwrap();
        db.set_plugin_client_target("installation-1", "codex", true, "installed", None)
            .unwrap();
        let catalog = PluginCatalog::from_entries(vec![plugin_entry(None)]);

        let result = PluginInstallEngine {
            db: &db,
            catalog: &catalog,
        }
        .uninstall("installation-1")
        .unwrap();

        assert!(!result.success);
        assert!(db.get_plugin_installation("installation-1").is_ok());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn plugin_uninstall_deletes_db_record_when_all_clients_succeed() {
        let (db, path) = test_db("taro-plugin-uninstall-succeeds");
        let plugin_file = std::env::temp_dir().join(format!("taro-plugin-file-{}", Uuid::new_v4()));
        std::fs::write(&plugin_file, "plugin").unwrap();
        db.upsert_plugin_installation("installation-1", "plugin-a", true, "installed", None)
            .unwrap();
        db.set_plugin_client_target("installation-1", "codex", true, "installed", None)
            .unwrap();
        let catalog = PluginCatalog::from_entries(vec![plugin_entry(Some(
            PluginInstallStrategy::RepoFiles {
                files: vec![plugin_file.display().to_string()],
            },
        ))]);

        let result = PluginInstallEngine {
            db: &db,
            catalog: &catalog,
        }
        .uninstall("installation-1")
        .unwrap();

        assert!(result.success);
        assert!(db.get_plugin_installation("installation-1").is_err());
        assert!(!plugin_file.exists());
        let _ = std::fs::remove_file(path);
    }
}
