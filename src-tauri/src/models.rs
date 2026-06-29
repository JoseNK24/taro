use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationCatalogEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub icon: String,
    #[serde(default)]
    pub coming_soon: bool,
    pub server: ServerDef,
    #[serde(default)]
    pub secrets: Vec<SecretDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerDef {
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretDef {
    pub key: String,
    pub label: String,
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub id: String,
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub client_id: String,
    pub display_name: String,
    pub detected: bool,
    pub config_path: Option<String>,
    pub config_exists: bool,
    pub sync_supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationRecord {
    pub id: String,
    pub integration_id: String,
    pub enabled: bool,
    pub status: String,
    pub installed_at: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientTargetRecord {
    pub installation_id: String,
    pub client_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckRecord {
    pub installation_id: String,
    pub integration_id: String,
    pub integration_name: String,
    pub latency_ms: Option<i64>,
    pub ok: bool,
    pub checked_at: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretStatus {
    pub integration_id: String,
    pub integration_name: String,
    pub secret_key: String,
    pub label: String,
    pub required: bool,
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStatus {
    pub name: String,
    pub available: bool,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirstRunStatus {
    pub completed: bool,
    pub detected_clients: Vec<DetectionResult>,
    pub existing_servers: Vec<ExistingServerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExistingServerInfo {
    pub client_id: String,
    pub client_name: String,
    pub server_id: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    pub installation_id: String,
    pub success: bool,
    pub message: String,
}
