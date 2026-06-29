use crate::harness::driver::{
    CommunityInstallContext, HarnessDriver, HarnessError, HarnessProbeResult, HarnessResult,
};
use crate::models::{HarnessInstanceConfig, ResolvedMcpConfig};

pub struct StubDriver {
    kind: String,
    name: String,
    detected: bool,
}

impl StubDriver {
    pub fn new(kind: &str, name: &str, detected: bool) -> Self {
        Self {
            kind: kind.to_string(),
            name: name.to_string(),
            detected,
        }
    }
}

impl HarnessDriver for StubDriver {
    fn kind(&self) -> &str {
        &self.kind
    }

    fn display_name(&self) -> &str {
        &self.name
    }

    fn detect(&self) -> bool {
        self.detected
    }

    fn probe(&self, _config: &HarnessInstanceConfig) -> HarnessProbeResult {
        HarnessProbeResult {
            detected: self.detected,
            version: None,
            auth_status: "unknown".to_string(),
            agent_capable: false,
            detail: Some("Detect only — agent install not supported".to_string()),
        }
    }

    fn agent_capable(&self) -> bool {
        false
    }

    fn run_install_agent(
        &self,
        _ctx: &CommunityInstallContext,
        _config: &HarnessInstanceConfig,
        _log: &mut String,
    ) -> HarnessResult<ResolvedMcpConfig> {
        Err(HarnessError::NotAgentCapable)
    }
}
