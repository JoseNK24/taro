use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Manager};
use thiserror::Error;

use crate::models::IntegrationCatalogEntry;

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error("Failed to load catalog: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse catalog: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Catalog not found")]
    NotFound,
}

pub type CatalogResult<T> = Result<T, CatalogError>;

#[derive(Clone)]
pub struct Catalog {
    entries: Vec<IntegrationCatalogEntry>,
}

impl Catalog {
    pub fn load(app: &AppHandle) -> CatalogResult<Self> {
        let path = resolve_catalog_path(app)?;
        let content = fs::read_to_string(&path)?;
        let entries: Vec<IntegrationCatalogEntry> = serde_json::from_str(&content)?;
        Ok(Self { entries })
    }

    pub fn from_embedded() -> CatalogResult<Self> {
        let content = include_str!("../../resources/catalog.json");
        let entries: Vec<IntegrationCatalogEntry> = serde_json::from_str(content)?;
        Ok(Self { entries })
    }

    pub fn all(&self) -> &[IntegrationCatalogEntry] {
        &self.entries
    }

    pub fn get(&self, id: &str) -> Option<&IntegrationCatalogEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    pub fn installable(&self) -> Vec<&IntegrationCatalogEntry> {
        self.entries.iter().filter(|e| !e.coming_soon).collect()
    }
}

fn resolve_catalog_path(app: &AppHandle) -> CatalogResult<PathBuf> {
    if let Ok(path) = app.path().resolve("resources/catalog.json", tauri::path::BaseDirectory::Resource) {
        if path.exists() {
            return Ok(path);
        }
    }
    let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/catalog.json");
    if dev_path.exists() {
        return Ok(dev_path);
    }
    Err(CatalogError::NotFound)
}
