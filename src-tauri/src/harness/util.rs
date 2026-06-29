use std::path::Path;
use std::process::Command;

pub fn app_exists(paths: &[&str]) -> bool {
    paths.iter().any(|p| Path::new(p).exists())
}

pub fn which(cmd: &str) -> Option<String> {
    Command::new("which")
        .arg(cmd)
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
