use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use uuid::Uuid;

use crate::debug_log::agent_debug_log;
use crate::harness::driver::{
    CommunityInstallContext, HarnessDriver, HarnessError, HarnessProbeResult, HarnessResult,
};
use crate::harness::util::{
    extract_json_object, heuristic_from_hint, heuristic_from_readme, login_shell_path,
    run_version_probe, which,
};
use crate::models::{HarnessInstanceConfig, ResolvedMcpConfig};

const RESOLVED_MCP_SCHEMA: &str = r#"{
  "type": "object",
  "properties": {
    "command": { "type": "string" },
    "args": { "type": "array", "items": { "type": "string" } },
    "env_keys": { "type": "array", "items": { "type": "string" } },
    "requires": { "type": "array", "items": { "type": "string" } },
    "confidence": { "type": "string" },
    "notes": { "type": "string" }
  },
  "required": ["command", "args"],
  "additionalProperties": false
}"#;

pub struct CodexDriver;

impl HarnessDriver for CodexDriver {
    fn kind(&self) -> &str {
        "codex"
    }

    fn display_name(&self) -> &str {
        "Codex CLI"
    }

    fn install_hint(&self) -> Option<&str> {
        Some("Install Codex CLI and ensure `codex` is on PATH")
    }

    fn detect(&self) -> bool {
        which("codex").is_some()
    }

    fn probe(&self, _config: &HarnessInstanceConfig) -> HarnessProbeResult {
        let detected = self.detect();
        let codex_path = which("codex");
        let version = if detected {
            codex_path
                .as_deref()
                .and_then(|path| run_version_probe(path, &["--version"]))
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
            detail: codex_path,
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
        log.push_str("Running codex for MCP analysis…\n");

        let modes = ["structured", "json", "last_message", "legacy"];
        let mut last_stderr = String::new();
        for mode in modes {
            let result = match mode {
                "structured" => run_codex_structured(&prompt, log),
                "json" => run_codex_json(&prompt, log),
                "last_message" => run_codex_last_message(&prompt, log),
                _ => run_codex_legacy(&prompt, log),
            };

            let (out, response_text) = match result {
                Some(value) => value,
                None => {
                    // #region agent log
                    agent_debug_log(
                        "B",
                        "harness/drivers/codex.rs:run_install_agent",
                        "codex attempt unavailable",
                        serde_json::json!({ "mode": mode }),
                    );
                    // #endregion
                    continue;
                }
            };

            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !stderr.trim().is_empty() {
                last_stderr = stderr.to_string();
            }
            let parse_ok = parse_config_from_text(&response_text).is_some();
            // #region agent log
            agent_debug_log(
                "B",
                "harness/drivers/codex.rs:run_install_agent",
                "codex exec completed",
                serde_json::json!({
                    "mode": mode,
                    "exit_code": out.status.code(),
                    "stdout_len": stdout.len(),
                    "stderr_len": stderr.len(),
                    "response_len": response_text.len(),
                    "stderr_tail": stderr.chars().rev().take(300).collect::<String>().chars().rev().collect::<String>(),
                    "parse_ok": parse_ok,
                    "response_tail": response_text.chars().rev().take(300).collect::<String>().chars().rev().collect::<String>(),
                }),
            );
            // #endregion

            if let Some(config) = parse_config_from_text(&response_text) {
                return Ok(config);
            }
        }

        log.push_str("Falling back to README/install_hint heuristics…\n");
        if let Some(config) = heuristic_from_readme(ctx).or_else(|| heuristic_from_hint(ctx)) {
            return Ok(config);
        }

            let detail = last_stderr
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("no output from codex");
        Err(HarnessError::AgentFailed(format!(
            "codex produced no usable config (last error: {detail})"
        )))
    }
}

fn codex_program() -> String {
    which("codex").unwrap_or_else(|| "codex".to_string())
}

fn codex_command() -> Command {
    let mut command = Command::new(codex_program());
    // codex is a `#!/usr/bin/env node` shim; it must run under the user's real
    // node, which lives on the login-shell PATH (not launchd's minimal PATH).
    command.env("PATH", login_shell_path());
    command
}

fn temp_paths() -> (PathBuf, PathBuf, PathBuf) {
    let id = Uuid::new_v4();
    let base = std::env::temp_dir();
    (
        base.join(format!("taro-codex-schema-{id}.json")),
        base.join(format!("taro-codex-output-{id}.json")),
        base.join(format!("taro-codex-output-{id}.txt")),
    )
}

fn run_codex_with_stdin(prompt: &str, args: &[&str]) -> Option<Output> {
    let mut child = codex_command()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(prompt.as_bytes());
    }

    child.wait_with_output().ok()
}

fn run_codex_structured(prompt: &str, log: &mut String) -> Option<(Output, String)> {
    let (schema_path, output_path, _) = temp_paths();
    if std::fs::write(&schema_path, RESOLVED_MCP_SCHEMA).is_err() {
        return None;
    }

    log.push_str("Trying codex exec with structured output schema…\n");
    let schema = schema_path.to_string_lossy().to_string();
    let output = output_path.to_string_lossy().to_string();
    let args = [
        "exec",
        "--skip-git-repo-check",
        "--dangerously-bypass-approvals-and-sandbox",
        "--output-schema",
        &schema,
        "-o",
        &output,
        "-",
    ];

    let out = run_codex_with_stdin(prompt, &args)?;
    append_process_output(log, &out);

    let response = std::fs::read_to_string(&output_path).ok();
    let _ = std::fs::remove_file(&schema_path);
    let _ = std::fs::remove_file(&output_path);

    let text = response.filter(|value| !value.trim().is_empty())?;
    Some((out, text))
}

fn run_codex_json(prompt: &str, log: &mut String) -> Option<(Output, String)> {
    log.push_str("Trying codex exec with JSONL output…\n");
    let args = [
        "exec",
        "--skip-git-repo-check",
        "--dangerously-bypass-approvals-and-sandbox",
        "--json",
        "-",
    ];

    let out = run_codex_with_stdin(prompt, &args)?;
    append_process_output(log, &out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let response = extract_jsonl_agent_text(&stdout)?;
    Some((out, response))
}

fn run_codex_last_message(prompt: &str, log: &mut String) -> Option<(Output, String)> {
    let (_, _, output_path) = temp_paths();

    log.push_str("Trying codex exec with output-last-message…\n");
    let output = output_path.to_string_lossy().to_string();
    let args = [
        "exec",
        "--skip-git-repo-check",
        "--dangerously-bypass-approvals-and-sandbox",
        "-o",
        &output,
        "-",
    ];

    let out = run_codex_with_stdin(prompt, &args)?;
    append_process_output(log, &out);

    let response = std::fs::read_to_string(&output_path).ok();
    let _ = std::fs::remove_file(&output_path);

    let text = response.filter(|value| !value.trim().is_empty())?;
    Some((out, text))
}

fn run_codex_legacy(prompt: &str, log: &mut String) -> Option<(Output, String)> {
    log.push_str("Trying legacy codex exec prompt argument…\n");
    let out = codex_command()
        .args(["exec", "--skip-git-repo-check", prompt])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .ok()?;

    append_process_output(log, &out);
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    if stdout.trim().is_empty() {
        return None;
    }
    Some((out, stdout))
}

fn append_process_output(log: &mut String, out: &Output) {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    if !stdout.is_empty() {
        log.push_str(&stdout);
    }
    if !stderr.is_empty() {
        log.push_str(&stderr);
    }
}

fn extract_jsonl_agent_text(stdout: &str) -> Option<String> {
    let mut last_message = None;
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        for pointer in [
            "/item/text",
            "/msg/content",
            "/content",
            "/text",
        ] {
            if let Some(text) = event.pointer(pointer).and_then(|value| value.as_str()) {
                if !text.trim().is_empty() {
                    last_message = Some(text.to_string());
                }
            }
        }
    }
    last_message
}

fn parse_config_from_text(text: &str) -> Option<ResolvedMcpConfig> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(config) = serde_json::from_str::<ResolvedMcpConfig>(trimmed) {
        return Some(config);
    }

    extract_json_object(trimmed)
        .and_then(|json| serde_json::from_str::<ResolvedMcpConfig>(&json).ok())
}

fn build_agent_prompt(ctx: &CommunityInstallContext) -> String {
    let readme = ctx
        .readme_excerpt
        .as_deref()
        .unwrap_or("(no README available)");
    format!(
        "You are setting up an MCP server on the user's macOS machine so that it can actually run. You have a real shell with full access — do not just describe steps, perform them.\n\
         1. Work out how this MCP launches: npx (needs node), uvx/uv (needs uv), python3 (needs python), or a native binary on PATH.\n\
         2. Check whether the required runtime is already installed (e.g. `which uv`, `which node`).\n\
         3. If a required runtime is MISSING, install it now and verify it: prefer Homebrew (`brew install node`, `brew install python`) or the official installer for uv (`curl -LsSf https://astral.sh/uv/install.sh | sh`, which installs to ~/.local/bin). Re-check with `which` afterwards. If the MCP is a native binary, install it per its README (GitHub release or one-line installer) so the command resolves on PATH.\n\
         4. When everything needed is installed and on PATH, return ONLY a JSON object with fields: command, args (array), env_keys (array), requires (array), confidence (high|medium|low), notes (string).\n\
         Allowed commands: npx, uvx, node, python3, or a safe binary name on PATH (for example codebase-memory-mcp).\n\
         Do not use placeholder paths like /path/to/...; use the binary name when the README describes a native binary install.\n\
         `requires` must contain ONLY runtime command names that must already be on PATH (for example: node, uv). Use an empty array if none. Never put sentences, instructions, or install steps in `requires` — put those in `notes`. In `notes`, briefly state what you installed.\n\
         MCP name: {}\nDescription: {}\nInstall hint: {}\nREADME excerpt:\n{}\n",
        ctx.entry.name,
        ctx.entry.description,
        ctx.entry.install_hint.as_deref().unwrap_or(""),
        readme,
    )
}
