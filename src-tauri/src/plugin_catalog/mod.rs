use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Manager};
use thiserror::Error;

use crate::models::PluginCatalogEntry;

#[derive(Debug, Error)]
pub enum PluginCatalogError {
    #[error("Failed to load plugin catalog: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse plugin catalog: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Plugin catalog not found")]
    NotFound,
}

pub type PluginCatalogResult<T> = Result<T, PluginCatalogError>;

#[derive(Clone)]
pub struct PluginCatalog {
    entries: Vec<PluginCatalogEntry>,
}

impl PluginCatalog {
    pub fn load(app: &AppHandle) -> PluginCatalogResult<Self> {
        let path = resolve_catalog_path(app)?;
        let content = fs::read_to_string(&path)?;
        let entries: Vec<PluginCatalogEntry> = serde_json::from_str(&content)?;
        Ok(Self { entries })
    }

    pub fn from_embedded() -> PluginCatalogResult<Self> {
        let content = include_str!("../../resources/plugin_catalog.json");
        let entries: Vec<PluginCatalogEntry> = serde_json::from_str(content)?;
        Ok(Self { entries })
    }

    #[cfg(test)]
    pub fn from_entries(entries: Vec<PluginCatalogEntry>) -> Self {
        Self { entries }
    }

    pub fn all(&self) -> &[PluginCatalogEntry] {
        &self.entries
    }

    pub fn get(&self, id: &str) -> Option<&PluginCatalogEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    pub fn installable(&self) -> Vec<&PluginCatalogEntry> {
        self.entries.iter().filter(|e| !e.coming_soon).collect()
    }
}

fn resolve_catalog_path(app: &AppHandle) -> PluginCatalogResult<PathBuf> {
    if let Ok(path) = app.path().resolve(
        "resources/plugin_catalog.json",
        tauri::path::BaseDirectory::Resource,
    ) {
        if path.exists() {
            return Ok(path);
        }
    }
    let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/plugin_catalog.json");
    if dev_path.exists() {
        return Ok(dev_path);
    }
    Err(PluginCatalogError::NotFound)
}
