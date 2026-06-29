pub mod driver;
pub mod drivers;
pub mod util;

use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::db::Database;
use crate::models::{
    HarnessDriverInfo, HarnessInstanceConfig, HarnessInstanceRecord, HarnessSnapshot,
};

use driver::{HarnessDriver, HarnessProbeResult};
use drivers::{
    claude_code::ClaudeCodeDriver, codex::CodexDriver, cursor::CursorDriver,
    opencode::OpencodeDriver, stub::StubDriver,
};
use util::app_exists;

pub fn all_drivers() -> Vec<Box<dyn HarnessDriver>> {
    vec![
        Box::new(CursorDriver),
        Box::new(ClaudeCodeDriver),
        Box::new(CodexDriver),
        Box::new(OpencodeDriver),
        Box::new(StubDriver::new("zed", "Zed", app_exists(&["/Applications/Zed.app"]))),
        Box::new(StubDriver::new("goose", "Goose", app_exists(&["/Applications/Goose.app"]))),
    ]
}

pub fn get_driver(kind: &str) -> Option<Box<dyn HarnessDriver>> {
    all_drivers()
        .into_iter()
        .find(|d| d.kind() == kind)
}

pub fn list_driver_infos() -> Vec<HarnessDriverInfo> {
    all_drivers()
        .iter()
        .map(|d| HarnessDriverInfo {
            kind: d.kind().to_string(),
            display_name: d.display_name().to_string(),
            detected: d.detect(),
            agent_capable: d.agent_capable(),
            install_hint: d.install_hint().map(str::to_string),
        })
        .collect()
}

pub fn parse_instance_config(json: &str) -> HarnessInstanceConfig {
    serde_json::from_str(json).unwrap_or_default()
}

pub fn probe_instance(
    record: &HarnessInstanceRecord,
) -> HarnessProbeResult {
    let config = parse_instance_config(&record.config_json);
    if let Some(driver) = get_driver(&record.driver_kind) {
        driver.probe(&config)
    } else {
        HarnessProbeResult {
            detected: false,
            version: None,
            auth_status: "unknown".to_string(),
            agent_capable: false,
            detail: Some(format!("Unknown driver: {}", record.driver_kind)),
        }
    }
}

pub fn probe_all_instances(db: &Database) -> Result<Vec<HarnessSnapshot>, String> {
    let instances = db.list_harness_instances().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    Ok(instances
        .into_iter()
        .map(|inst| {
            let probe = probe_instance(&inst);
            HarnessSnapshot {
                instance_id: inst.id.clone(),
                driver_kind: inst.driver_kind.clone(),
                display_name: inst.display_name.clone(),
                enabled: inst.enabled,
                detected: probe.detected,
                version: probe.version,
                auth_status: probe.auth_status,
                agent_capable: probe.agent_capable,
                probe_detail: probe.detail,
                probed_at: now.clone(),
            }
        })
        .collect())
}

pub fn create_instance(
    db: &Database,
    driver_kind: &str,
    display_name: &str,
    config_json: &str,
) -> Result<HarnessInstanceRecord, String> {
    let driver = get_driver(driver_kind).ok_or_else(|| "Unknown driver".to_string())?;
    if !driver.detect() {
        return Err(format!(
            "{} is not detected on this Mac",
            driver.display_name()
        ));
    }

    let id = format!("{}_{}", driver_kind, &Uuid::new_v4().to_string()[..8]);
    let record = HarnessInstanceRecord {
        id: id.clone(),
        driver_kind: driver_kind.to_string(),
        display_name: display_name.to_string(),
        enabled: true,
        is_default_install_agent: false,
        config_json: config_json.to_string(),
        created_at: Utc::now().to_rfc3339(),
    };
    db.create_harness_instance(&record)
        .map_err(|e| e.to_string())?;

    let instances = db.list_harness_instances().map_err(|e| e.to_string())?;
    if instances.len() == 1 {
        db.set_default_install_agent(&id).map_err(|e| e.to_string())?;
    }

    Ok(record)
}

pub fn default_install_agent_id(db: &Database) -> Result<Option<String>, String> {
    let instances = db.list_harness_instances().map_err(|e| e.to_string())?;
    Ok(instances
        .into_iter()
        .find(|i| i.is_default_install_agent)
        .map(|i| i.id))
}

pub fn agent_capable_instances(db: &Database) -> Result<Vec<(HarnessInstanceRecord, HarnessProbeResult)>, String> {
    let instances = db.list_harness_instances().map_err(|e| e.to_string())?;
    Ok(instances
        .into_iter()
        .filter(|i| i.enabled)
        .filter_map(|inst| {
            let probe = probe_instance(&inst);
            if probe.agent_capable && probe.detected {
                Some((inst, probe))
            } else {
                None
            }
        })
        .collect())
}

pub fn snapshots_by_id(snapshots: &[HarnessSnapshot]) -> HashMap<String, HarnessSnapshot> {
    snapshots
        .iter()
        .map(|s| (s.instance_id.clone(), s.clone()))
        .collect()
}
