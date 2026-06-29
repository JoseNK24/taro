use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::{DiscoveredMcpEntry, HarnessInstanceConfig, ResolvedMcpConfig};

#[derive(Debug, Error)]
pub enum HarnessError {
    #[error("Driver not detected")]
    NotDetected,
    #[error("Agent not capable for this driver")]
    NotAgentCapable,
    #[error("Agent failed: {0}")]
    AgentFailed(String),
    #[error("Invalid agent output: {0}")]
    InvalidOutput(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type HarnessResult<T> = Result<T, HarnessError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessProbeResult {
    pub detected: bool,
    pub version: Option<String>,
    pub auth_status: String,
    pub agent_capable: bool,
    pub detail: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CommunityInstallContext {
    pub entry: DiscoveredMcpEntry,
    pub readme_excerpt: Option<String>,
    pub dependencies: Vec<String>,
}

pub trait HarnessDriver: Send + Sync {
    fn kind(&self) -> &str;
    fn display_name(&self) -> &str;
    fn install_hint(&self) -> Option<&str> {
        None
    }
    fn detect(&self) -> bool;
    fn probe(&self, config: &HarnessInstanceConfig) -> HarnessProbeResult;
    fn agent_capable(&self) -> bool;
    fn run_install_agent(
        &self,
        ctx: &CommunityInstallContext,
        config: &HarnessInstanceConfig,
        log: &mut String,
    ) -> HarnessResult<ResolvedMcpConfig>;
}
