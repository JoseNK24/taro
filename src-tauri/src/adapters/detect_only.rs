use std::path::{Path, PathBuf};

use crate::adapters::json_mcp::command_exists;
use crate::adapters::{expand_home_path, AdapterError, AdapterResult, ClientAdapter};
use crate::models::{DetectionResult, McpServer};

/// Adapter for clients we can detect but not sync to yet.
pub struct DetectOnlyAdapter {
    id: String,
    display_name: String,
    config_path: PathBuf,
    config_path_alternates: Vec<PathBuf>,
    app_paths: Vec<PathBuf>,
    cli_commands: Vec<String>,
}

fn resolve_config_path(primary: &PathBuf, alternates: &[PathBuf]) -> (PathBuf, bool) {
    if primary.exists() {
        return (primary.clone(), true);
    }
    for alt in alternates {
        if alt.exists() {
            return (alt.clone(), true);
        }
    }
    (primary.clone(), false)
}

impl DetectOnlyAdapter {
    pub fn new(
        id: &str,
        display_name: &str,
        config_path: &str,
        config_alternates: &[&str],
        app_paths: &[&str],
        cli_commands: &[&str],
    ) -> Self {
        Self {
            id: id.to_string(),
            display_name: display_name.to_string(),
            config_path: expand_home_path(config_path),
            config_path_alternates: config_alternates
                .iter()
                .map(|p| expand_home_path(p))
                .collect(),
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
        let (resolved_config, config_exists) =
            resolve_config_path(&self.config_path, &self.config_path_alternates);
        DetectionResult {
            client_id: self.id.clone(),
            display_name: self.display_name.clone(),
            detected: app_exists || cli_exists,
            config_path: Some(resolved_config.display().to_string()),
            config_exists,
            sync_supported: false,
        }
    }

    fn read_servers(&self) -> AdapterResult<Vec<McpServer>> {
        Ok(vec![])
    }

    fn write_server(&self, _server: &McpServer) -> AdapterResult<()> {
        Err(AdapterError::Validation(format!(
            "{} does not support sync from Taro yet",
            self.display_name
        )))
    }

    fn remove_server(&self, _server_id: &str) -> AdapterResult<bool> {
        Err(AdapterError::Validation(format!(
            "{} does not support sync from Taro yet",
            self.display_name
        )))
    }

    fn backup_config(&self) -> AdapterResult<PathBuf> {
        Err(AdapterError::Validation("Sync not available".to_string()))
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
    let cli_exists = cli_commands.iter().any(|cmd| command_exists(cmd));
    DetectionResult {
        client_id: id.to_string(),
        display_name: display_name.to_string(),
        detected: app_exists || cli_exists,
        config_path: Some(config_path.display().to_string()),
        config_exists: config_path.exists(),
        sync_supported,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::get_adapter;

    #[test]
    fn resolve_config_path_falls_back_to_alternate() {
        let dir = std::env::temp_dir().join("taro-detect-test-alt");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let primary = dir.join("opencode.json");
        let alternate = dir.join("opencode.jsonc");
        std::fs::write(&alternate, "{}").unwrap();

        let (resolved, exists) = resolve_config_path(&primary, &[alternate.clone()]);
        assert!(exists);
        assert_eq!(resolved, alternate);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn opencode_detects_when_desktop_app_installed() {
        if !PathBuf::from("/Applications/OpenCode.app").exists() {
            return;
        }
        let adapter = get_adapter("opencode").expect("opencode adapter registered");
        let result = adapter.detect();
        assert!(
            result.detected,
            "OpenCode.app exists but adapter did not detect it"
        );
        if dirs::home_dir()
            .map(|h| h.join(".config/opencode/opencode.jsonc").exists())
            .unwrap_or(false)
        {
            assert!(result.config_exists);
            assert!(
                result
                    .config_path
                    .as_ref()
                    .is_some_and(|p| p.ends_with("opencode.jsonc")),
                "expected opencode.jsonc path, got {:?}",
                result.config_path
            );
        }
    }
}
