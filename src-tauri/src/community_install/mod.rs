use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::db::Database;
use crate::debug_log::agent_debug_log;
use crate::detect::{check_dependencies, sync_to_clients};
use crate::harness::{get_driver, parse_instance_config};
use crate::health::probe_server;
use crate::models::{
    CommunityInstallJob, CommunityInstallMeta, DiscoveredMcpEntry, McpServer,
    ResolvedMcpConfig,
};
use crate::secrets;

const ALLOWED_COMMANDS: &[&str] = &["npx", "uvx", "node", "python3", "python"];
// Generous: the agent may install runtimes (brew/uv) before reporting.
const AGENT_TIMEOUT_SECS: u64 = 300;

#[derive(Debug, Clone)]
struct JobState {
    job: CommunityInstallJob,
    cancelled: bool,
}

pub struct CommunityInstallManager {
    jobs: Arc<Mutex<HashMap<String, JobState>>>,
}

impl Default for CommunityInstallManager {
    fn default() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl CommunityInstallManager {
    pub fn get_job(&self, job_id: &str) -> Option<CommunityInstallJob> {
        self.jobs
            .lock()
            .ok()
            .and_then(|jobs| jobs.get(job_id).map(|s| s.job.clone()))
    }

    pub fn cancel_job(&self, job_id: &str) -> Result<(), String> {
        let mut jobs = self.jobs.lock().map_err(|e| e.to_string())?;
        if let Some(state) = jobs.get_mut(job_id) {
            state.cancelled = true;
            state.job.status = "failed".to_string();
            state.job.error = Some("Cancelled by user".to_string());
            Ok(())
        } else {
            Err("Job not found".to_string())
        }
    }

    pub fn start_job(
        &self,
        app: AppHandle,
        db: Arc<Mutex<Database>>,
        discovered_mcp_id: String,
        harness_instance_id: String,
    ) -> Result<CommunityInstallJob, String> {
        let entry = {
            let db = db.lock().map_err(|e| e.to_string())?;
            db.get_discovered_mcp(&discovered_mcp_id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "Discovered MCP not found".to_string())?
        };

        let harness = {
            let db = db.lock().map_err(|e| e.to_string())?;
            db.get_harness_instance(&harness_instance_id)
                .map_err(|e| e.to_string())?
        };

        if !harness.enabled {
            return Err("Harness instance is disabled".to_string());
        }

        let driver = get_driver(&harness.driver_kind)
            .ok_or_else(|| "Unknown harness driver".to_string())?;
        if !driver.agent_capable() {
            return Err("Selected harness cannot run agent installs".to_string());
        }

        let job_id = Uuid::new_v4().to_string();
        let job = CommunityInstallJob {
            id: job_id.clone(),
            discovered_mcp_id: discovered_mcp_id.clone(),
            harness_instance_id: harness_instance_id.clone(),
            status: "pending".to_string(),
            resolved_config: None,
            agent_log: String::new(),
            error: None,
            discovered_name: Some(entry.name.clone()),
        };

        {
            let mut jobs = self.jobs.lock().map_err(|e| e.to_string())?;
            jobs.insert(
                job_id.clone(),
                JobState {
                    job: job.clone(),
                    cancelled: false,
                },
            );
        }

        let jobs_ref = self.jobs.clone();
        let app_clone = app.clone();
        let db_clone = db.clone();

        tauri::async_runtime::spawn(async move {
            run_agent_job(
                app_clone,
                db_clone,
                jobs_ref,
                job_id,
                entry,
                harness.driver_kind,
                harness.config_json,
            )
            .await;
        });

        Ok(job)
    }
}

async fn run_agent_job(
    app: AppHandle,
    _db: Arc<Mutex<Database>>,
    jobs: Arc<Mutex<HashMap<String, JobState>>>,
    job_id: String,
    entry: DiscoveredMcpEntry,
    driver_kind: String,
    config_json: String,
) {
    update_job(&jobs, &job_id, |job| {
        job.status = "analyzing".to_string();
    });
    emit_progress(&app, &job_id, "Starting agent analysis…");

    let readme = fetch_readme_excerpt(&entry).await;
    // #region agent log
    agent_debug_log(
        "E",
        "community_install/mod.rs:readme",
        "readme fetched",
        serde_json::json!({
            "mcp_id": entry.id,
            "mcp_name": entry.name,
            "install_hint": entry.install_hint,
            "readme_len": readme.as_ref().map(|r| r.len()),
            "driver_kind": driver_kind,
        }),
    );
    // #endregion
    let deps = check_dependencies(&[]);
    let dep_names: Vec<String> = deps
        .iter()
        .filter(|d| d.available)
        .map(|d| d.name.clone())
        .collect();

    let ctx = crate::harness::driver::CommunityInstallContext {
        entry: entry.clone(),
        readme_excerpt: readme.clone(),
        dependencies: dep_names,
    };

    let config = parse_instance_config(&config_json);
    let driver = match get_driver(&driver_kind) {
        Some(d) => d,
        None => {
            fail_job(&jobs, &job_id, "Unknown driver");
            return;
        }
    };

    let result = tokio::time::timeout(
        Duration::from_secs(AGENT_TIMEOUT_SECS),
        tokio::task::spawn_blocking(move || {
            let mut log = String::new();
            let result = driver.run_install_agent(&ctx, &config, &mut log);
            (result, log)
        }),
    )
    .await;

    if is_cancelled(&jobs, &job_id) {
        return;
    }

    match result {
        Ok(Ok((Ok(resolved), log))) => {
            // #region agent log
            agent_debug_log(
                "D",
                "community_install/mod.rs:agent_ok",
                "agent returned config",
                serde_json::json!({
                    "command": resolved.command,
                    "args": resolved.args,
                    "confidence": resolved.confidence,
                    "log_len": log.len(),
                }),
            );
            // #endregion
            append_log(&jobs, &job_id, &log);
            emit_progress(&app, &job_id, &log);
            match validate_resolved_config(&resolved) {
                Ok(validated) => {
                    update_job(&jobs, &job_id, |job| {
                        job.status = "awaiting_review".to_string();
                        job.resolved_config = Some(validated);
                        job.agent_log = log.clone();
                    });
                    emit_progress(&app, &job_id, &log);
                }
                Err(e) => {
                    // #region agent log
                    agent_debug_log(
                        "D",
                        "community_install/mod.rs:validation_failed",
                        "validate_resolved_config rejected agent output",
                        serde_json::json!({ "error": e }),
                    );
                    // #endregion
                    append_log(&jobs, &job_id, &format!("Validation failed: {e}\n"));
                    try_fallback(&jobs, &job_id, &entry, readme.as_deref(), &app);
                }
            }
        }
        Ok(Ok((Err(e), log))) => {
            // #region agent log
            agent_debug_log(
                "B",
                "community_install/mod.rs:agent_err",
                "agent returned error",
                serde_json::json!({
                    "error": e.to_string(),
                    "log_tail": log.chars().rev().take(500).collect::<String>().chars().rev().collect::<String>(),
                }),
            );
            // #endregion
            append_log(&jobs, &job_id, &format!("{log}\nAgent error: {e}\n"));
            try_fallback(&jobs, &job_id, &entry, readme.as_deref(), &app);
        }
        Ok(Err(e)) => {
            // #region agent log
            agent_debug_log(
                "A",
                "community_install/mod.rs:thread_err",
                "spawn_blocking failed",
                serde_json::json!({ "error": e.to_string() }),
            );
            // #endregion
            fail_job(&jobs, &job_id, &format!("Agent thread failed: {e}"));
        }
        Err(_) => {
            // #region agent log
            agent_debug_log(
                "A",
                "community_install/mod.rs:timeout",
                "agent timed out",
                serde_json::json!({ "timeout_secs": AGENT_TIMEOUT_SECS }),
            );
            // #endregion
            fail_job(&jobs, &job_id, "Agent timed out after 120s");
        }
    }
}

fn try_fallback(
    jobs: &Arc<Mutex<HashMap<String, JobState>>>,
    job_id: &str,
    entry: &DiscoveredMcpEntry,
    readme_excerpt: Option<&str>,
    app: &AppHandle,
) {
    let ctx = crate::harness::driver::CommunityInstallContext {
        entry: entry.clone(),
        readme_excerpt: readme_excerpt.map(str::to_string),
        dependencies: vec![],
    };
    let hint_heuristic = crate::harness::util::heuristic_from_hint(&ctx);
    let readme_heuristic = crate::harness::util::heuristic_from_readme(&ctx);
    // #region agent log
    agent_debug_log(
        "C",
        "community_install/mod.rs:try_fallback",
        "fallback heuristics attempt",
        serde_json::json!({
            "install_hint": entry.install_hint,
            "hint_heuristic_found": hint_heuristic.is_some(),
            "readme_heuristic_found": readme_heuristic.is_some(),
            "readme_len": readme_excerpt.map(|r| r.len()),
        }),
    );
    // #endregion
    if let Some(config) = readme_heuristic.or(hint_heuristic) {
        match validate_resolved_config(&config) {
            Ok(validated) => {
                update_job(jobs, job_id, |job| {
                    job.status = "awaiting_review".to_string();
                    job.resolved_config = Some(validated);
                    job.agent_log
                        .push_str("Using README/install_hint fallback (review carefully).\n");
                });
                emit_progress(app, job_id, "Using README/install_hint fallback");
                return;
            }
            Err(e) => {
                // #region agent log
                agent_debug_log(
                    "D",
                    "community_install/mod.rs:fallback_validation_failed",
                    "fallback config rejected",
                    serde_json::json!({ "error": e }),
                );
                // #endregion
            }
        }
    }
    fail_job(
        jobs,
        job_id,
        "Agent failed. Check install_hint or install manually.",
    );
}

fn is_safe_binary_command(base_cmd: &str) -> bool {
    !base_cmd.is_empty()
        && !base_cmd.starts_with('-')
        && base_cmd
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub fn validate_resolved_config(config: &ResolvedMcpConfig) -> Result<ResolvedMcpConfig, String> {
    let cmd = config.command.trim();
    if cmd.is_empty() {
        return Err("Command is required".to_string());
    }

    let base_cmd = std::path::Path::new(cmd)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(cmd);

    if !ALLOWED_COMMANDS.contains(&base_cmd) && !is_safe_binary_command(base_cmd) {
        return Err(format!(
            "Command '{cmd}' not in allowlist (npx, uvx, node, python3, or safe binary name)"
        ));
    }

    if cmd.contains("/path/to/") {
        return Err("Placeholder paths are not allowed".to_string());
    }

    if cmd.starts_with('/') && !cmd.starts_with("/usr/") && !cmd.starts_with("/opt/") {
        return Err("Absolute paths outside system dirs are not allowed".to_string());
    }

    for arg in &config.args {
        if arg.contains("..") {
            return Err("Path traversal in args is not allowed".to_string());
        }
    }

    // The agent sometimes fills `requires` with prose ("Install the native
    // binary…") instead of command names; those can never resolve on PATH and
    // would block the install with a nonsense "Missing dependencies" error.
    // Keep only plausible command tokens.
    let mut validated = config.clone();
    validated.requires.retain(|r| is_safe_binary_command(r.trim()));

    Ok(validated)
}

pub fn resolved_to_mcp_server(
    discovered_mcp_id: &str,
    config: &ResolvedMcpConfig,
    secrets_map: &HashMap<String, String>,
) -> McpServer {
    let id = format!(
        "taro-community-{}",
        discovered_mcp_id
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
            .collect::<String>()
    );
    McpServer {
        id,
        command: config.command.clone(),
        args: config.args.clone(),
        env: secrets_map.clone(),
    }
}

pub fn confirm_community_install(
    db: &Database,
    job_manager: &CommunityInstallManager,
    job_id: &str,
    client_ids: Vec<String>,
    secrets_map: HashMap<String, String>,
    resolved_override: Option<ResolvedMcpConfig>,
) -> Result<crate::models::InstallResult, String> {
    let job = job_manager
        .get_job(job_id)
        .ok_or_else(|| "Job not found".to_string())?;

    if job.status != "awaiting_review" && resolved_override.is_none() {
        return Err("Job is not ready for confirmation".to_string());
    }

    let config = resolved_override
        .or(job.resolved_config.clone())
        .ok_or_else(|| "No resolved config".to_string())?;
    let validated = validate_resolved_config(&config)?;

    if client_ids.is_empty() {
        return Err("Select at least one client".to_string());
    }

    for key in &validated.env_keys {
        if validated.env_keys.iter().any(|k| k == key) && !secrets_map.contains_key(key) {
            // only required if in env_keys list
        }
    }
    let missing: Vec<_> = validated
        .env_keys
        .iter()
        .filter(|k| !secrets_map.contains_key(*k))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "Missing secrets: {}",
            missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
        ));
    }

    let integration_id = format!("community-{}", job.discovered_mcp_id);
    let mut server = resolved_to_mcp_server(&job.discovered_mcp_id, &validated, &secrets_map);
    server.env.clear();

    // Hands-off backstop: if the agent didn't already install a required runtime,
    // install it here (deterministic recipes) instead of blocking the user.
    let missing = crate::deps::missing_dependencies(&validated);
    let installable: Vec<String> = missing
        .iter()
        .filter(|d| d.installable)
        .map(|d| d.name.clone())
        .collect();
    if !installable.is_empty() {
        crate::deps::install_dependencies(&installable)?;
    }
    let unresolvable: Vec<String> = crate::deps::missing_dependencies(&validated)
        .into_iter()
        .filter(|d| !d.installable)
        .map(|d| d.install_label)
        .collect();
    if !unresolvable.is_empty() {
        return Err(format!(
            "Missing dependencies (install manually): {}",
            unresolvable.join(", ")
        ));
    }

    update_job_status(job_manager, job_id, "installing");

    sync_to_clients(
        &resolved_to_mcp_server(&job.discovered_mcp_id, &validated, &secrets_map),
        &client_ids,
        false,
    )?;

    for (key, value) in &secrets_map {
        secrets::set_secret(&integration_id, key, value).map_err(|e| e.to_string())?;
    }

    let probe = probe_server(&resolved_to_mcp_server(
        &job.discovered_mcp_id,
        &validated,
        &secrets_map,
    ));
    let status = if probe.ok { "connected" } else { "error" };

    let existing = db
        .get_installation_by_integration(&integration_id)
        .map_err(|e| e.to_string())?;
    let installation_id = existing
        .as_ref()
        .map(|i| i.id.clone())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    db.upsert_installation_with_source(
        &installation_id,
        &integration_id,
        true,
        status,
        probe.detail.as_deref(),
        "community",
    )
    .map_err(|e| e.to_string())?;

    for client_id in &client_ids {
        db.set_client_target(&installation_id, client_id, true)
            .map_err(|e| e.to_string())?;
    }

    db.insert_health_check(
        &installation_id,
        probe.latency_ms,
        probe.ok,
        probe.detail.as_deref(),
    )
    .map_err(|e| e.to_string())?;

    let meta = CommunityInstallMeta {
        installation_id: installation_id.clone(),
        discovered_mcp_id: job.discovered_mcp_id.clone(),
        harness_instance_id: job.harness_instance_id.clone(),
        resolved_server: server,
        env_keys: validated.env_keys.clone(),
        agent_log: Some(job.agent_log.clone()),
    };
    db.upsert_community_install_meta(&meta)
        .map_err(|e| e.to_string())?;

    update_job_status(job_manager, job_id, "done");

    let message = if probe.ok {
        "Community MCP installed successfully".to_string()
    } else {
        format!(
            "Installed with warnings: {}",
            probe.detail.unwrap_or_default()
        )
    };

    Ok(crate::models::InstallResult {
        installation_id,
        success: probe.ok,
        message,
    })
}

fn update_job_status(manager: &CommunityInstallManager, job_id: &str, status: &str) {
    if let Ok(mut jobs) = manager.jobs.lock() {
        if let Some(state) = jobs.get_mut(job_id) {
            state.job.status = status.to_string();
        }
    }
}

fn update_job<F>(jobs: &Arc<Mutex<HashMap<String, JobState>>>, job_id: &str, f: F)
where
    F: FnOnce(&mut CommunityInstallJob),
{
    if let Ok(mut map) = jobs.lock() {
        if let Some(state) = map.get_mut(job_id) {
            f(&mut state.job);
        }
    }
}

fn append_log(jobs: &Arc<Mutex<HashMap<String, JobState>>>, job_id: &str, text: &str) {
    update_job(jobs, job_id, |job| job.agent_log.push_str(text));
}

fn fail_job(jobs: &Arc<Mutex<HashMap<String, JobState>>>, job_id: &str, error: &str) {
    update_job(jobs, job_id, |job| {
        job.status = "failed".to_string();
        job.error = Some(error.to_string());
    });
}

fn is_cancelled(jobs: &Arc<Mutex<HashMap<String, JobState>>>, job_id: &str) -> bool {
    jobs.lock()
        .ok()
        .and_then(|m| m.get(job_id).map(|s| s.cancelled))
        .unwrap_or(false)
}

fn emit_progress(app: &AppHandle, job_id: &str, message: &str) {
    let _ = app.emit(
        "community_install_progress",
        serde_json::json!({
            "job_id": job_id,
            "message": message,
        }),
    );
}

async fn fetch_readme_excerpt(entry: &DiscoveredMcpEntry) -> Option<String> {
    let github_url = entry.github_url.as_deref()?;
    let raw_url = github_readme_raw_url(github_url)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .ok()?;
    let resp = client.get(&raw_url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let text = resp.text().await.ok()?;
    Some(text.chars().take(8192).collect())
}

fn github_readme_raw_url(github_url: &str) -> Option<String> {
    let trimmed = github_url.trim_end_matches('/');
    let parts: Vec<&str> = trimmed.split('/').collect();
    if parts.len() < 5 {
        return None;
    }
    let user = parts[parts.len() - 2];
    let repo = parts[parts.len() - 1];
    Some(format!(
        "https://raw.githubusercontent.com/{user}/{repo}/HEAD/README.md"
    ))
}
