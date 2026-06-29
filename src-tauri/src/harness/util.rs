use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

pub fn app_exists(paths: &[&str]) -> bool {
    paths.iter().any(|p| Path::new(p).exists())
}

/// PATH as seen by the user's login shell.
///
/// A GUI-launched macOS app inherits launchd's minimal PATH, which omits
/// Homebrew, version-manager shims (`~/.local/bin`, fnm/nvm/mise), etc. Tools
/// like `codex` (a `#!/usr/bin/env node` shim) then resolve `node` to a stale
/// interpreter or fail outright. Sourcing the login shell gives us the same
/// PATH the user gets in their terminal. Cached because spawning a login shell
/// is slow. ponytail: login-shell sourcing covers all version managers; replace
/// with explicit shim probing only if a shell-less environment shows up.
pub fn login_shell_path() -> &'static str {
    static PATH: OnceLock<String> = OnceLock::new();
    PATH.get_or_init(|| {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        let from_login = Command::new(&shell)
            .args(["-lic", "printf %s \"$PATH\""])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let inherited = std::env::var("PATH").unwrap_or_default();
        match from_login {
            // Merge so both the login PATH and anything the app already had are searchable.
            Some(login) if login != inherited => format!("{login}:{inherited}"),
            _ => format!("/opt/homebrew/bin:/usr/local/bin:{inherited}:/usr/bin:/bin"),
        }
    })
}

pub fn which(cmd: &str) -> Option<String> {
    Command::new("which")
        .arg(cmd)
        .env("PATH", login_shell_path())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn run_version_probe(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd).args(args).output().ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.lines().next().unwrap_or("").trim();
        if line.is_empty() {
            None
        } else {
            Some(line.to_string())
        }
    } else {
        None
    }
}

pub fn extract_json_object(text: &str) -> Option<String> {
    if let Some(start) = text.find('{') {
        let mut depth = 0i32;
        for (i, ch) in text[start..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(text[start..start + i + 1].to_string());
                    }
                }
                _ => {}
            }
        }
    }
    None
}

pub fn heuristic_from_hint(ctx: &super::driver::CommunityInstallContext) -> Option<crate::models::ResolvedMcpConfig> {
    let hint = ctx.entry.install_hint.as_deref()?;
    let hint = hint.trim();
    if hint.is_empty() {
        return None;
    }

    let parts: Vec<&str> = hint.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let command = parts[0].to_string();
    let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
    let requires = if command == "npx" || command == "uvx" {
        vec!["node".to_string()]
    } else {
        vec![]
    };

    Some(crate::models::ResolvedMcpConfig {
        command,
        args,
        env_keys: vec![],
        requires,
        confidence: "low".to_string(),
        notes: Some("Parsed from install_hint".to_string()),
    })
}

pub fn heuristic_from_readme(
    ctx: &super::driver::CommunityInstallContext,
) -> Option<crate::models::ResolvedMcpConfig> {
    let readme = ctx.readme_excerpt.as_deref()?;

    if let Some(start) = readme.find("\"mcpServers\"") {
        if let Some(json) = extract_json_object(&readme[start..]) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json) {
                if let Some(servers) = value.get("mcpServers").and_then(|s| s.as_object()) {
                    for (name, server) in servers {
                        if let Some(cmd) = server.get("command").and_then(|c| c.as_str()) {
                            let command = normalize_mcp_command(cmd);
                            if command.is_empty() || command.contains("path/to") {
                                continue;
                            }
                            let args = server
                                .get("args")
                                .and_then(|a| a.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(str::to_string))
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_default();
                            return Some(crate::models::ResolvedMcpConfig {
                                command,
                                args,
                                env_keys: vec![],
                                requires: vec![],
                                confidence: "low".to_string(),
                                notes: Some(format!(
                                    "Parsed mcpServers.{name} from README; install the binary first if needed"
                                )),
                            });
                        }
                    }
                }
            }
        }
    }

    for line in readme.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("npx ") || trimmed.starts_with("uvx ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                return Some(crate::models::ResolvedMcpConfig {
                    command: parts[0].to_string(),
                    args: parts[1..].iter().map(|s| s.to_string()).collect(),
                    env_keys: vec![],
                    requires: vec!["node".to_string()],
                    confidence: "low".to_string(),
                    notes: Some("Parsed install command from README".to_string()),
                });
            }
        }
    }

    None
}

fn normalize_mcp_command(cmd: &str) -> String {
    let cmd = cmd.trim();
    if cmd.contains("/path/to/") {
        return cmd
            .rsplit('/')
            .next()
            .unwrap_or(cmd)
            .to_string();
    }
    Path::new(cmd)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(cmd)
        .to_string()
}
