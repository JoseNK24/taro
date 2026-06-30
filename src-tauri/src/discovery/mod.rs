mod sources;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::{DateTime, Utc};
use serde::Deserialize;
use tauri::{AppHandle, Manager};
use thiserror::Error;

use crate::db::{Database, DbResult};
use crate::models::{
    DiscoveredMcpEntry, DiscoverySearchResult, DiscoveryStatus, DiscoverySyncStats,
};

pub static SYNC_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

const SETTING_LAST_SYNCED: &str = "discovered_catalog_synced_at";
const MAX_ENTRIES: usize = 2000;
const UPSERT_BATCH_SIZE: usize = 100;

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("Database error: {0}")]
    Db(#[from] crate::db::DbError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Sync already in progress")]
    SyncInProgress,
}

pub type DiscoveryResult<T> = Result<T, DiscoveryError>;

#[derive(Debug, Deserialize)]
struct BundledCatalog {
    entries: Vec<DiscoveredMcpEntry>,
}

pub fn import_if_empty(db: &Database, app: &AppHandle) -> DiscoveryResult<()> {
    if db.discovered_mcp_count()? > 0 {
        return Ok(());
    }
    let content = load_bundled_catalog(app)?;
    let catalog: BundledCatalog = serde_json::from_str(&content)?;
    db.import_discovered_mcps(&catalog.entries)?;
    Ok(())
}

pub fn maybe_background_sync(app: AppHandle) {
    let should_sync = {
        let state = app.state::<crate::state::AppState>();
        let db = match state.db.lock() {
            Ok(db) => db,
            Err(_) => return,
        };
        match needs_sync(&db) {
            Ok(true) => true,
            _ => false,
        }
    };

    if !should_sync {
        return;
    }

    tauri::async_runtime::spawn(async move {
        let _ = run_sync(&app).await;
    });
}

fn needs_sync(db: &Database) -> DbResult<bool> {
    let last = db.get_setting(SETTING_LAST_SYNCED)?;
    match last {
        None => Ok(true),
        Some(ts) => {
            let parsed = DateTime::parse_from_rfc3339(&ts)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now() - chrono::Duration::days(8));
            Ok(Utc::now().signed_duration_since(parsed).num_days() > 7)
        }
    }
}

pub async fn run_sync(app: &AppHandle) -> DiscoveryResult<DiscoverySyncStats> {
    if SYNC_IN_PROGRESS
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err(DiscoveryError::SyncInProgress);
    }

    let result = sync_discovered_catalog_inner(app).await;

    SYNC_IN_PROGRESS.store(false, Ordering::SeqCst);
    result
}

async fn sync_discovered_catalog_inner(app: &AppHandle) -> DiscoveryResult<DiscoverySyncStats> {
    let github_token = {
        let state = app.state::<crate::state::AppState>();
        let db = state.db.lock().map_err(|e| DiscoveryError::Http(e.to_string()))?;
        db.get_setting("github_token")?.unwrap_or_default()
    };

    let token_opt = if github_token.is_empty() {
        None
    } else {
        Some(github_token.as_str())
    };

    let mut entries = sources::fetch_all(token_opt).await?;
    entries.sort_by(|a, b| {
        b.popularity_score
            .partial_cmp(&a.popularity_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    entries.truncate(MAX_ENTRIES);

    let mut combined = DiscoverySyncStats {
        added: 0,
        updated: 0,
        errors: 0,
    };

    for chunk in entries.chunks(UPSERT_BATCH_SIZE) {
        let stats = {
            let state = app.state::<crate::state::AppState>();
            let db = state.db.lock().map_err(|e| DiscoveryError::Http(e.to_string()))?;
            db.upsert_discovered_mcps(chunk)?
        };
        combined.added += stats.added;
        combined.updated += stats.updated;
        combined.errors += stats.errors;
        tokio::task::yield_now().await;
    }

    {
        let state = app.state::<crate::state::AppState>();
        let db = state.db.lock().map_err(|e| DiscoveryError::Http(e.to_string()))?;
        db.set_setting(SETTING_LAST_SYNCED, &Utc::now().to_rfc3339())?;
    }

    Ok(combined)
}

pub fn search_discovered_mcps(
    db: &Database,
    query: &str,
    sort: &str,
    limit: i64,
    offset: i64,
) -> DbResult<DiscoverySearchResult> {
    db.search_discovered_mcps(query, sort, limit, offset)
}

pub fn get_discovered_mcp(db: &Database, id: &str) -> DbResult<Option<DiscoveredMcpEntry>> {
    db.get_discovered_mcp(id)
}

pub fn get_discovery_status(db: &Database) -> DbResult<DiscoveryStatus> {
    Ok(DiscoveryStatus {
        last_synced_at: db.get_setting(SETTING_LAST_SYNCED)?,
        total_count: db.discovered_mcp_count()?,
        sync_in_progress: SYNC_IN_PROGRESS.load(Ordering::SeqCst),
    })
}

fn load_bundled_catalog(app: &AppHandle) -> DiscoveryResult<String> {
    if let Ok(path) = app.path().resolve(
        "resources/discovered_catalog.json",
        tauri::path::BaseDirectory::Resource,
    ) {
        if path.exists() {
            return Ok(std::fs::read_to_string(path)?);
        }
    }
    let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/discovered_catalog.json");
    if dev_path.exists() {
        return Ok(std::fs::read_to_string(dev_path)?);
    }
    Ok(include_str!("../../resources/discovered_catalog.json").to_string())
}

pub fn compute_popularity_score(stars: i64, updated_at: Option<&str>) -> f64 {
    let star_score = ((stars + 1) as f64).log10() * 10.0;
    let recency_bonus = match updated_at {
        Some(ts) => DateTime::parse_from_rfc3339(ts)
            .map(|d| {
                let days = Utc::now()
                    .signed_duration_since(d.with_timezone(&Utc))
                    .num_days();
                if days < 30 {
                    30.0
                } else if days < 90 {
                    15.0
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0),
        None => 0.0,
    };
    star_score + recency_bonus
}
