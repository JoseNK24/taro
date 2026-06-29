//! Auto-installing the runtimes an MCP needs (uv, node, …) so the user doesn't
//! have to set anything up first.
//!
//! Recipes are HARDCODED here, never taken from the agent: the model only ever
//! produces a runtime *name* (`uv`, `node`), and we map that to a fixed,
//! auditable install command. The agent can never make us run arbitrary shell.

use std::collections::HashSet;
use std::process::Command;

use serde::Serialize;

use crate::harness::util::{login_shell_path, which};
use crate::models::ResolvedMcpConfig;

#[derive(Debug, Clone, Serialize)]
pub struct MissingDependency {
    /// Binary that must end up on PATH (e.g. "uv").
    pub name: String,
    /// Human label shown before installing (e.g. "uv (Astral installer)").
    pub install_label: String,
    /// Whether Taro has a recipe to install it automatically.
    pub installable: bool,
}

struct Recipe {
    /// Binary to look for on PATH to decide if it's already installed.
    check: &'static str,
    label: &'static str,
    /// Shell command that installs it.
    command: &'static str,
}

fn recipe(dep: &str) -> Option<Recipe> {
    match dep {
        "uv" | "uvx" => Some(Recipe {
            check: "uv",
            label: "uv (Astral installer)",
            command: "curl -LsSf https://astral.sh/uv/install.sh | sh",
        }),
        "node" | "npx" | "npm" => Some(Recipe {
            check: "node",
            label: "Node.js (Homebrew)",
            command: "brew install node",
        }),
        "python" | "python3" => Some(Recipe {
            check: "python3",
            label: "Python (Homebrew)",
            command: "brew install python",
        }),
        _ => None,
    }
}

/// Runtime tools an MCP config implies: the agent-declared `requires` plus the
/// runtime behind the base command (e.g. `uvx …` ⇒ uv).
fn required_tools(config: &ResolvedMcpConfig) -> Vec<String> {
    let mut tools = config.requires.clone();
    let cmd = config.command.trim();
    let base = std::path::Path::new(cmd)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(cmd);
    tools.push(base.to_string());
    tools
}

/// Dependencies that aren't on PATH yet. Only runtimes we have a recipe for are
/// reported — the MCP's own native binary (no recipe) is left to the probe.
pub fn missing_dependencies(config: &ResolvedMcpConfig) -> Vec<MissingDependency> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for tool in required_tools(config) {
        let tool = tool.trim();
        let Some(r) = recipe(tool) else { continue };
        if !seen.insert(r.check) {
            continue;
        }
        if which(r.check).is_none() {
            out.push(MissingDependency {
                name: r.check.to_string(),
                install_label: r.label.to_string(),
                installable: true,
            });
        }
    }
    out
}

/// Run the install recipe for each named dependency. Returns the combined
/// installer output on success, or the failing installer's error.
pub fn install_dependencies(names: &[String]) -> Result<String, String> {
    let mut log = String::new();
    for name in names {
        let r = recipe(name).ok_or_else(|| format!("No installer recipe for '{name}'"))?;
        log.push_str(&format!("$ {}\n", r.command));
        let out = Command::new("sh")
            .arg("-c")
            .arg(r.command)
            .env("PATH", login_shell_path())
            .output()
            .map_err(|e| format!("Failed to run installer for {name}: {e}"))?;
        log.push_str(&String::from_utf8_lossy(&out.stdout));
        log.push_str(&String::from_utf8_lossy(&out.stderr));
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let detail = stderr.lines().rev().find(|l| !l.trim().is_empty());
            return Err(format!(
                "Could not install {name}: {}",
                detail.unwrap_or("installer failed")
            ));
        }
    }
    Ok(log)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(command: &str, requires: &[&str]) -> ResolvedMcpConfig {
        ResolvedMcpConfig {
            command: command.to_string(),
            args: vec![],
            env_keys: vec![],
            requires: requires.iter().map(|s| s.to_string()).collect(),
            confidence: "low".to_string(),
            notes: None,
        }
    }

    #[test]
    fn uvx_command_and_requires_collapse_to_uv() {
        // `uvx` base command + a "uvx" require both map to the single `uv` recipe.
        let tools = required_tools(&cfg("uvx", &["uvx"]));
        assert!(tools.iter().any(|t| t == "uvx"));
        // recipe() normalizes both spellings to the same check binary.
        assert_eq!(recipe("uvx").unwrap().check, "uv");
        assert_eq!(recipe("uv").unwrap().check, "uv");
    }

    #[test]
    fn unknown_runtime_has_no_recipe() {
        assert!(recipe("codebase-memory-mcp").is_none());
    }
}
