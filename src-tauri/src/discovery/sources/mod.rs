mod github;
mod registry;

use crate::models::DiscoveredMcpEntry;

use super::DiscoveryError;

pub async fn fetch_all(github_token: Option<&str>) -> Result<Vec<DiscoveredMcpEntry>, DiscoveryError> {
    let client = reqwest::Client::builder()
        .user_agent("taro-mcp-discovery/0.1")
        .build()
        .map_err(|e| DiscoveryError::Http(e.to_string()))?;

    let mut map: std::collections::HashMap<String, DiscoveredMcpEntry> =
        std::collections::HashMap::new();

    let github_entries = github::fetch_github_repos(&client, github_token).await?;
    for entry in github_entries {
        merge_entry(&mut map, entry);
    }

    let registry_entries = registry::fetch_registry_servers(&client).await?;
    for entry in registry_entries {
        merge_entry(&mut map, entry);
    }

    let mut entries: Vec<DiscoveredMcpEntry> = map.into_values().collect();
    entries.retain(|e| !e.description.is_empty());
    Ok(entries)
}

fn merge_entry(map: &mut std::collections::HashMap<String, DiscoveredMcpEntry>, incoming: DiscoveredMcpEntry) {
    match map.get_mut(&incoming.id) {
        Some(existing) => {
            if incoming.description.len() > existing.description.len() {
                existing.description = incoming.description.clone();
            }
            if incoming.name.len() > existing.name.len() {
                existing.name = incoming.name.clone();
            }
            for tag in &incoming.tags {
                if !existing.tags.contains(tag) {
                    existing.tags.push(tag.clone());
                }
            }
            for src in &incoming.sources {
                if !existing.sources.contains(src) {
                    existing.sources.push(src.clone());
                }
            }
            if incoming.github_stars > existing.github_stars {
                existing.github_stars = incoming.github_stars;
                existing.github_forks = incoming.github_forks;
                existing.github_updated_at = incoming.github_updated_at.clone();
            }
            if existing.github_url.is_none() {
                existing.github_url = incoming.github_url.clone();
            }
            if existing.homepage_url.is_none() {
                existing.homepage_url = incoming.homepage_url.clone();
            }
            if existing.registry_url.is_none() {
                existing.registry_url = incoming.registry_url.clone();
            }
            if existing.install_hint.is_none() {
                existing.install_hint = incoming.install_hint.clone();
            }
            existing.popularity_score = super::compute_popularity_score(
                existing.github_stars,
                existing.github_updated_at.as_deref(),
            );
        }
        None => {
            map.insert(incoming.id.clone(), incoming);
        }
    }
}
