use std::path::{Path, PathBuf};

use crate::adapters::json_mcp::command_exists;
use crate::adapters::{AdapterError, AdapterResult, ClientAdapter, expand_home_path};
use crate::models::{DetectionResult, McpServer};

/// Adapter for clients we can detect but not sync to yet.
pub struct DetectOnlyAdapter {
    id: String,
    display_name: String,
    config_path: PathBuf,
    app_paths: Vec<PathBuf>,
    cli_commands: Vec<String>,
}

impl DetectOnlyAdapter {
    pub fn new(
        id: &str,
        display_name: &str,
        config_path: &str,
        app_paths: &[&str],
        cli_commands: &[&str],
    ) -> Self {
        Self {
            id: id.to_string(),
            display_name: display_name.to_string(),
            config_path: expand_home_path(config_path),
            app_paths: app_paths.iter().map(PathBuf::from).collect(),
            cli_commands: cli_commands.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl ClientAdapter for DetectOnlyAdapter {
    fn id(&self) -> &str {
        &self.id
    }

    fn display_name(&self) -> &str {
        &self.display_name
    }

    fn config_path(&self) -> PathBuf {
        self.config_path.clone()
    }

    fn detect(&self) -> DetectionResult {
        let app_exists = self.app_paths.iter().any(|p| p.exists());
        let cli_exists = self.cli_commands.iter().any(|cmd| command_exists(cmd));
        DetectionResult {
            client_id: self.id.clone(),
            display_name: self.display_name.clone(),
            detected: app_exists || cli_exists,
            config_path: Some(self.config_path.display().to_string()),
            config_exists: self.config_path.exists(),
            sync_supported: false,
        }
    }

    fn read_servers(&self) -> AdapterResult<Vec<McpServer>> {
        Ok(vec![])
    }

    fn write_server(&self, _server: &McpServer) -> AdapterResult<()> {
        Err(AdapterError::Validation(format!(
            "{} aún no admite sincronización desde Taro",
            self.display_name
        )))
    }

    fn remove_server(&self, _server_id: &str) -> AdapterResult<()> {
        Err(AdapterError::Validation(format!(
            "{} aún no admite sincronización desde Taro",
            self.display_name
        )))
    }

    fn backup_config(&self) -> AdapterResult<PathBuf> {
        Err(AdapterError::Validation(
            "Sincronización no disponible".to_string(),
        ))
    }
}

pub fn detect_from_spec(
    id: &str,
    display_name: &str,
    config_path: &Path,
    app_paths: &[PathBuf],
    cli_commands: &[String],
    sync_supported: bool,
) -> DetectionResult {
    let app_exists = app_paths.iter().any(|p| p.exists());
    let cli_exists = cli_commands
        .iter()
        .any(|cmd| command_exists(cmd));
    DetectionResult {
        client_id: id.to_string(),
        display_name: display_name.to_string(),
        detected: app_exists || cli_exists,
        config_path: Some(config_path.display().to_string()),
        config_exists: config_path.exists(),
        sync_supported,
    }
}
