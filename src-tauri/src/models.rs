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
    #[serde(default = "default_source_curated")]
    pub source: String,
}

fn default_source_curated() -> String {
    "curated".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessInstanceRecord {
    pub id: String,
    pub driver_kind: String,
    pub display_name: String,
    pub enabled: bool,
    pub is_default_install_agent: bool,
    pub config_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessSnapshot {
    pub instance_id: String,
    pub driver_kind: String,
    pub display_name: String,
    pub enabled: bool,
    pub detected: bool,
    pub version: Option<String>,
    pub auth_status: String,
    pub agent_capable: bool,
    pub probe_detail: Option<String>,
    pub probed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HarnessInstanceConfig {
    #[serde(default)]
    pub env_overrides: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessDriverInfo {
    pub kind: String,
    pub display_name: String,
    pub detected: bool,
    pub agent_capable: bool,
    pub install_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedMcpConfig {
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env_keys: Vec<String>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub confidence: String,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityInstallMeta {
    pub installation_id: String,
    pub discovered_mcp_id: String,
    pub harness_instance_id: String,
    pub resolved_server: McpServer,
    #[serde(default)]
    pub env_keys: Vec<String>,
    pub agent_log: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityInstallJob {
    pub id: String,
    pub discovered_mcp_id: String,
    pub harness_instance_id: String,
    pub status: String,
    pub resolved_config: Option<ResolvedMcpConfig>,
    pub agent_log: String,
    pub error: Option<String>,
    pub discovered_name: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredMcpEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub github_url: Option<String>,
    pub homepage_url: Option<String>,
    pub registry_url: Option<String>,
    #[serde(default)]
    pub github_stars: i64,
    #[serde(default)]
    pub github_forks: i64,
    pub github_updated_at: Option<String>,
    pub discovered_at: String,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub popularity_score: f64,
    pub install_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverySearchResult {
    pub entries: Vec<DiscoveredMcpEntry>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryStatus {
    pub last_synced_at: Option<String>,
    pub total_count: i64,
    pub sync_in_progress: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverySyncStats {
    pub added: i64,
    pub updated: i64,
    pub errors: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMarketplace {
    pub source: String,
    pub plugin_id: String,
    pub marketplace_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum PluginInstallStrategy {
    MarketplaceCli,
    RepoFiles { files: Vec<String> },
    ExtensionCli { package: String },
    ComingSoon,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCatalogEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub icon: String,
    #[serde(default)]
    pub coming_soon: bool,
    pub github_url: String,
    #[serde(default)]
    pub github_stars: i64,
    pub marketplace: PluginMarketplace,
    #[serde(default)]
    pub client_install: std::collections::HashMap<String, PluginInstallStrategy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstallationRecord {
    pub id: String,
    pub plugin_id: String,
    pub enabled: bool,
    pub status: String,
    pub installed_at: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginClientTargetRecord {
    pub installation_id: String,
    pub client_id: String,
    pub enabled: bool,
    pub status: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginClientInstallResult {
    pub client_id: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstallResult {
    pub installation_id: String,
    pub success: bool,
    pub message: String,
    pub client_results: Vec<PluginClientInstallResult>,
}
