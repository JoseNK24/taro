use std::process::Command;
use std::time::Instant;

use thiserror::Error;

use crate::catalog::Catalog;
use crate::models::McpServer;

#[derive(Debug, Error)]
pub enum HealthError {
    #[error("Probe failed: {0}")]
    Probe(String),
    #[error("Integration not found: {0}")]
    NotFound(String),
}

pub type HealthResult<T> = Result<T, HealthError>;

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub ok: bool,
    pub latency_ms: Option<i64>,
    pub detail: Option<String>,
}

pub fn probe_server(server: &McpServer) -> ProbeResult {
    let start = Instant::now();
    let output = Command::new(&server.command)
        .args(&server.args)
        .envs(&server.env)
        .arg("--version")
        .output();

    let latency_ms = start.elapsed().as_millis() as i64;

    match output {
        Ok(out) if out.status.success() => ProbeResult {
            ok: true,
            latency_ms: Some(latency_ms),
            detail: Some("Proceso iniciado correctamente".to_string()),
        },
        Ok(out) => {
            // Some MCP servers don't support --version; try without it
            let fallback = Command::new(&server.command)
                .args(&server.args)
                .envs(&server.env)
                .output();
            match fallback {
                Ok(fb) if fb.status.success() || fb.status.code() == Some(0) => ProbeResult {
                    ok: true,
                    latency_ms: Some(latency_ms),
                    detail: Some("Servidor responde".to_string()),
                },
                Ok(fb) => {
                    let stderr = String::from_utf8_lossy(&fb.stderr);
                    ProbeResult {
                        ok: false,
                        latency_ms: Some(latency_ms),
                        detail: Some(if stderr.is_empty() {
                            format!("Código de salida: {:?}", fb.status.code())
                        } else {
                            stderr.chars().take(200).collect()
                        }),
                    }
                }
                Err(e) => ProbeResult {
                    ok: false,
                    latency_ms: Some(latency_ms),
                    detail: Some(e.to_string()),
                },
            }
        }
        Err(e) => ProbeResult {
            ok: false,
            latency_ms: Some(latency_ms),
            detail: Some(e.to_string()),
        },
    }
}

pub fn build_mcp_server(
    catalog: &Catalog,
    integration_id: &str,
    env: std::collections::HashMap<String, String>,
) -> HealthResult<McpServer> {
    let entry = catalog
        .get(integration_id)
        .ok_or_else(|| HealthError::NotFound(integration_id.to_string()))?;

    let mut args = entry.server.args.clone();
    if integration_id == "filesystem" {
        if let Some(home) = dirs::home_dir() {
            args.push(home.display().to_string());
        }
    }

    Ok(McpServer {
        id: format!("taro-{integration_id}"),
        command: entry.server.command.clone(),
        args,
        env,
    })
}
