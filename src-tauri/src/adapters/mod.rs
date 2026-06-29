pub mod codex;
pub mod detect_only;
pub mod json_mcp;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::{DetectionResult, McpServer};

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Client not detected: {0}")]
    NotDetected(String),
    #[error("Config validation failed: {0}")]
    Validation(String),
    #[error("Rollback failed: {0}")]
    Rollback(String),
}

pub type AdapterResult<T> = Result<T, AdapterError>;

#[derive(Debug, Deserialize, Serialize)]
struct ClientConfig {
    #[serde(rename = "mcpServers", default)]
    mcp_servers: HashMap<String, McpServerEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct McpServerEntry {
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    env: HashMap<String, String>,
}

pub trait ClientAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn display_name(&self) -> &str;
    fn config_path(&self) -> PathBuf;
    fn detect(&self) -> DetectionResult;
    fn read_servers(&self) -> AdapterResult<Vec<McpServer>>;
    fn write_server(&self, server: &McpServer) -> AdapterResult<()>;
    fn remove_server(&self, server_id: &str) -> AdapterResult<()>;
    fn backup_config(&self) -> AdapterResult<PathBuf>;

    fn is_app_installed(&self) -> bool {
        self.detect().detected
    }
}

pub fn all_adapters() -> Vec<Box<dyn ClientAdapter>> {
    use json_mcp::JsonMcpAdapter;

    vec![
        // Sync supported — JSON mcpServers
        Box::new(JsonMcpAdapter::new(
            "cursor",
            "Cursor",
            "~/.cursor/mcp.json",
            &["/Applications/Cursor.app"],
            &["cursor", "cursor-agent"],
        )),
        Box::new(JsonMcpAdapter::new(
            "claude-desktop",
            "Claude Desktop",
            "~/Library/Application Support/Claude/claude_desktop_config.json",
            &["/Applications/Claude.app"],
            &[],
        )),
        Box::new(JsonMcpAdapter::new(
            "claude-code",
            "Claude Code",
            "~/.claude.json",
            &[],
            &["claude"],
        )),
        Box::new(JsonMcpAdapter::new(
            "windsurf",
            "Windsurf",
            "~/.codeium/windsurf/mcp_config.json",
            &["/Applications/Windsurf.app"],
            &["windsurf"],
        )),
        Box::new(JsonMcpAdapter::new(
            "cline",
            "Cline",
            "~/.cline/data/settings/cline_mcp_settings.json",
            &[],
            &["cline"],
        )),
        Box::new(JsonMcpAdapter::new(
            "gemini-cli",
            "Gemini CLI",
            "~/.gemini/settings.json",
            &[],
            &["gemini"],
        )),
        Box::new(JsonMcpAdapter::new(
            "continue",
            "Continue",
            "~/.continue/config.json",
            &[],
            &["continue"],
        )),
        Box::new(codex::CodexAdapter::new()),
        // Detect only — sync coming later
        Box::new(detect_only::DetectOnlyAdapter::new(
            "opencode",
            "OpenCode",
            "~/.config/opencode/opencode.json",
            &["~/.config/opencode/opencode.jsonc"],
            &["/Applications/OpenCode.app"],
            &["opencode"],
        )),
        Box::new(detect_only::DetectOnlyAdapter::new(
            "openclaw",
            "OpenClaw",
            "~/.openclaw/openclaw.json",
            &[],
            &[],
            &["openclaw"],
        )),
        Box::new(detect_only::DetectOnlyAdapter::new(
            "hermes",
            "Hermes Agent",
            "~/.hermes/config.yaml",
            &[],
            &[],
            &["hermes"],
        )),
        Box::new(detect_only::DetectOnlyAdapter::new(
            "vscode",
            "VS Code",
            "~/Library/Application Support/Code/User/globalStorage/mcp.json",
            &[],
            &[
                "/Applications/Visual Studio Code.app",
                "/Applications/Visual Studio Code - Insiders.app",
            ],
            &["code", "code-insiders"],
        )),
        Box::new(detect_only::DetectOnlyAdapter::new(
            "zed",
            "Zed",
            "~/.config/zed/settings.json",
            &[],
            &["/Applications/Zed.app"],
            &["zed"],
        )),
        Box::new(detect_only::DetectOnlyAdapter::new(
            "roo-code",
            "Roo Code",
            "~/.roo/mcp.json",
            &[],
            &[],
            &[],
        )),
        Box::new(detect_only::DetectOnlyAdapter::new(
            "open-interpreter",
            "Open Interpreter",
            "~/.config/open-interpreter/config.json",
            &[],
            &[],
            &["interpreter", "oi"],
        )),
        Box::new(detect_only::DetectOnlyAdapter::new(
            "goose",
            "Goose",
            "~/.config/goose/config.yaml",
            &[],
            &["/Applications/Goose.app"],
            &["goose"],
        )),
        Box::new(detect_only::DetectOnlyAdapter::new(
            "amazon-q",
            "Amazon Q Developer",
            "~/.aws/amazonq/mcp.json",
            &[],
            &[],
            &["q"],
        )),
        Box::new(detect_only::DetectOnlyAdapter::new(
            "jetbrains",
            "JetBrains AI",
            "~/Library/Application Support/JetBrains/mcp.json",
            &[],
            &["/Applications/IntelliJ IDEA.app"],
            &[],
        )),
    ]
}

pub fn get_adapter(client_id: &str) -> Option<Box<dyn ClientAdapter>> {
    all_adapters()
        .into_iter()
        .find(|a| a.id() == client_id)
}

pub fn detect_all_clients() -> Vec<DetectionResult> {
    all_adapters().iter().map(|a| a.detect()).collect()
}

pub fn syncable_adapters() -> Vec<Box<dyn ClientAdapter>> {
    all_adapters()
        .into_iter()
        .filter(|a| a.detect().sync_supported)
        .collect()
}

fn expand_home(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    PathBuf::from(path)
}

pub(crate) fn expand_home_path(path: &str) -> PathBuf {
    expand_home(path)
}

fn read_config(path: &Path) -> AdapterResult<ClientConfig> {
    if !path.exists() {
        return Ok(ClientConfig {
            mcp_servers: HashMap::new(),
        });
    }
    let content = fs::read_to_string(path)?;
    let config: ClientConfig = serde_json::from_str(&content)?;
    Ok(config)
}

fn write_config(path: &Path, config: &ClientConfig) -> AdapterResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    let _: ClientConfig = serde_json::from_str(&content)
        .map_err(|e| AdapterError::Validation(e.to_string()))?;
    fs::write(path, content)?;
    Ok(())
}

fn backup_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "config.json".to_string());
    path.with_file_name(format!("{file_name}.taro-backup"))
}

pub(crate) fn backup_file(path: &Path) -> AdapterResult<PathBuf> {
    let backup = backup_path(path);
    if path.exists() {
        fs::copy(path, &backup)?;
    }
    Ok(backup)
}

pub(crate) fn rollback_from_backup(path: &Path, backup: &Path) -> AdapterResult<()> {
    if backup.exists() {
        fs::copy(backup, path).map_err(|e| AdapterError::Rollback(e.to_string()))?;
    } else if path.exists() {
        fs::remove_file(path).map_err(|e| AdapterError::Rollback(e.to_string()))?;
    }
    Ok(())
}

pub(crate) fn read_servers_from_path(path: &Path) -> AdapterResult<Vec<McpServer>> {
    let config = read_config(path)?;
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

pub(crate) fn write_server_to_path(path: &Path, server: &McpServer) -> AdapterResult<()> {
    let backup = backup_file(path)?;
    let mut config = read_config(path)?;
    config.mcp_servers.insert(
        server.id.clone(),
        McpServerEntry {
            command: server.command.clone(),
            args: server.args.clone(),
            env: server.env.clone(),
        },
    );
    if let Err(e) = write_config(path, &config) {
        let _ = rollback_from_backup(path, &backup);
        return Err(e);
    }
    Ok(())
}

pub(crate) fn remove_server_from_path(path: &Path, server_id: &str) -> AdapterResult<()> {
    let backup = backup_file(path)?;
    let mut config = read_config(path)?;
    config.mcp_servers.remove(server_id);
    if let Err(e) = write_config(path, &config) {
        let _ = rollback_from_backup(path, &backup);
        return Err(e);
    }
    Ok(())
}
