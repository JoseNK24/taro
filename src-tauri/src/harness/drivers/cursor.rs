use std::process::{Command, Stdio};

use crate::harness::driver::{
    CommunityInstallContext, HarnessDriver, HarnessError, HarnessProbeResult, HarnessResult,
};
use crate::harness::util::{app_exists, extract_json_object, heuristic_from_hint, run_version_probe, which};
use crate::models::{HarnessInstanceConfig, ResolvedMcpConfig};

pub struct CursorDriver;

impl HarnessDriver for CursorDriver {
    fn kind(&self) -> &str {
        "cursor"
    }

    fn display_name(&self) -> &str {
        "Cursor"
    }

    fn install_hint(&self) -> Option<&str> {
        Some("Install Cursor from https://cursor.com and ensure cursor-agent is on PATH")
    }

    fn detect(&self) -> bool {
        app_exists(&["/Applications/Cursor.app"]) && which("cursor-agent").is_some()
    }

    fn probe(&self, _config: &HarnessInstanceConfig) -> HarnessProbeResult {
        let detected = self.detect();
        let version = if detected {
            run_version_probe("cursor-agent", &["--version"])
        } else {
            None
        };
        HarnessProbeResult {
            detected,
            version,
            auth_status: if detected {
                "unknown".to_string()
            } else {
                "unauthenticated".to_string()
            },
            agent_capable: self.agent_capable(),
            detail: None,
        }
    }

    fn agent_capable(&self) -> bool {
        self.detect()
    }

    fn run_install_agent(
        &self,
        ctx: &CommunityInstallContext,
        _config: &HarnessInstanceConfig,
        log: &mut String,
    ) -> HarnessResult<ResolvedMcpConfig> {
        if !self.agent_capable() {
            return Err(HarnessError::NotAgentCapable);
        }

        let prompt = build_agent_prompt(ctx);
        log.push_str("Running cursor-agent for MCP analysis…\n");

        let output = Command::new("cursor-agent")
            .args(["-p", &prompt])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                log.push_str(&stdout);
                if !stderr.is_empty() {
                    log.push_str(&stderr);
                }
                if let Some(json) = extract_json_object(&stdout) {
                    if let Ok(config) = serde_json::from_str::<ResolvedMcpConfig>(&json) {
                        return Ok(config);
                    }
                }
            }
            Err(e) => {
                log.push_str(&format!("cursor-agent failed: {e}\n"));
            }
        }

        log.push_str("Falling back to install_hint heuristic…\n");
        heuristic_from_hint(ctx).ok_or_else(|| {
            HarnessError::AgentFailed("Could not resolve MCP config".to_string())
        })
    }
}

fn build_agent_prompt(ctx: &CommunityInstallContext) -> String {
    let readme = ctx
        .readme_excerpt
        .as_deref()
        .unwrap_or("(no README available)");
    format!(
        "Analyze this MCP server and return ONLY a JSON object with fields: command, args (array), env_keys (array), requires (array), confidence (high|medium|low), notes (string).\n\
         Allowed commands: npx, uvx, node, python3.\n\
         MCP name: {}\nDescription: {}\nInstall hint: {}\nREADME excerpt:\n{}\n",
        ctx.entry.name,
        ctx.entry.description,
        ctx.entry.install_hint.as_deref().unwrap_or(""),
        readme,
    )
}
