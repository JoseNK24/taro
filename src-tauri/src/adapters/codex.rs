use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::adapters::json_mcp::command_exists;
use crate::adapters::{
    backup_file, expand_home_path, rollback_from_backup, AdapterError, AdapterResult, ClientAdapter,
};
use crate::models::{DetectionResult, McpServer};

const CONFIG_PATH: &str = "~/.codex/config.toml";

#[derive(Debug, Deserialize, Serialize)]
struct CodexConfig {
    #[serde(default, rename = "mcp_servers")]
    mcp_servers: HashMap<String, CodexServerEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CodexServerEntry {
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    env: HashMap<String, String>,
}

pub struct CodexAdapter {
    path: PathBuf,
}

impl CodexAdapter {
    pub fn new() -> Self {
        Self {
            path: expand_home_path(CONFIG_PATH),
        }
    }

    fn read_config(&self) -> AdapterResult<CodexConfig> {
        if !self.path.exists() {
            return Ok(CodexConfig {
                mcp_servers: HashMap::new(),
            });
        }
        let content = fs::read_to_string(&self.path)?;
        toml::from_str(&content).map_err(|e| AdapterError::Validation(e.to_string()))
    }

    fn write_config(&self, config: &CodexConfig) -> AdapterResult<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content =
            toml::to_string_pretty(config).map_err(|e| AdapterError::Validation(e.to_string()))?;
        fs::write(&self.path, content)?;
        Ok(())
    }
}

impl ClientAdapter for CodexAdapter {
    fn id(&self) -> &str {
        "codex"
    }

    fn display_name(&self) -> &str {
        "Codex"
    }

    fn config_path(&self) -> PathBuf {
        self.path.clone()
    }

    fn detect(&self) -> DetectionResult {
        let detected = PathBuf::from("/Applications/Codex.app").exists() || command_exists("codex");
        DetectionResult {
            client_id: self.id().to_string(),
            display_name: self.display_name().to_string(),
            detected,
            config_path: Some(self.path.display().to_string()),
            config_exists: self.path.exists(),
            sync_supported: true,
        }
    }

    fn read_servers(&self) -> AdapterResult<Vec<McpServer>> {
        let config = self.read_config()?;
        Ok(config
            .mcp_servers
            .into_iter()
            .map(|(id, entry)| McpServer {
                id,
                command: entry.command,
                args: entry.args,
                env: entry.env,
            })
            .collect())
    }

    fn write_server(&self, server: &McpServer) -> AdapterResult<()> {
        let backup = backup_file(&self.path)?;
        let mut config = self.read_config()?;
        config.mcp_servers.insert(
            server.id.clone(),
            CodexServerEntry {
                command: server.command.clone(),
                args: server.args.clone(),
                env: server.env.clone(),
            },
        );
        if let Err(e) = self.write_config(&config) {
            let _ = rollback_from_backup(&self.path, &backup);
            return Err(e);
        }
        Ok(())
    }

    fn remove_server(&self, server_id: &str) -> AdapterResult<bool> {
        let backup = backup_file(&self.path)?;
        let mut config = self.read_config()?;
        let removed = config.mcp_servers.remove(server_id).is_some();
        if let Err(e) = self.write_config(&config) {
            let _ = rollback_from_backup(&self.path, &backup);
            return Err(e);
        }
        Ok(removed)
    }

    fn backup_config(&self) -> AdapterResult<PathBuf> {
        backup_file(&self.path)
    }
}
