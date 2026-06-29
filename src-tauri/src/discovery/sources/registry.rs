use chrono::Utc;
use serde::Deserialize;

use crate::models::DiscoveredMcpEntry;

use super::super::compute_popularity_score;
use super::super::DiscoveryError;

const REGISTRY_URL: &str = "https://registry.modelcontextprotocol.io/v0/servers";

#[derive(Debug, Deserialize)]
struct RegistryResponse {
    servers: Vec<RegistryServer>,
    #[serde(rename = "nextCursor")]
    next_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RegistryServer {
    name: String,
    title: Option<String>,
    description: Option<String>,
    #[serde(default)]
    packages: Vec<RegistryPackage>,
    #[serde(default)]
    remotes: Vec<RegistryRemote>,
}

#[derive(Debug, Deserialize)]
struct RegistryPackage {
    #[serde(alias = "registryType")]
    registry_type: Option<String>,
    identifier: Option<String>,
    #[serde(default)]
    repository: Option<RegistryRepo>,
}

#[derive(Debug, Deserialize)]
struct RegistryRepo {
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RegistryRemote {
    url: Option<String>,
}

pub async fn fetch_registry_servers(
    client: &reqwest::Client,
) -> Result<Vec<DiscoveredMcpEntry>, DiscoveryError> {
    let mut entries = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let url = match &cursor {
            Some(c) => format!("{REGISTRY_URL}?cursor={c}"),
            None => REGISTRY_URL.to_string(),
        };

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| DiscoveryError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(DiscoveryError::Http(format!(
                "Registry API error: {}",
                resp.status()
            )));
        }

        let body: RegistryResponse = resp
            .json()
            .await
            .map_err(|e| DiscoveryError::Http(e.to_string()))?;

        for server in body.servers {
            let description = server
                .description
                .unwrap_or_default()
                .trim()
                .to_string();
            if description.is_empty() {
                continue;
            }

            let name = server
                .title
                .filter(|t| !t.is_empty())
                .unwrap_or_else(|| server.name.clone());

            let github_url = server
                .packages
                .iter()
                .filter_map(|p| p.repository.as_ref()?.url.as_ref())
                .find_map(|url| parse_github_url(url));

            let install_hint = server
                .packages
                .iter()
                .find_map(|p| {
                    let id = p.identifier.as_ref()?;
                    let reg = p.registry_type.as_deref().unwrap_or("npm");
                    Some(format!("{reg}:{id}"))
                });

            let id = github_url
                .as_ref()
                .and_then(|u| parse_github_owner_repo(u))
                .unwrap_or_else(|| format!("registry:{}", server.name));

            entries.push(DiscoveredMcpEntry {
                id,
                name,
                description,
                tags: vec!["registry".to_string()],
                github_url,
                homepage_url: server.remotes.first().and_then(|r| r.url.clone()),
                registry_url: Some(format!(
                    "https://registry.modelcontextprotocol.io/v0/servers/{}",
                    server.name
                )),
                github_stars: 0,
                github_forks: 0,
                github_updated_at: None,
                discovered_at: Utc::now().to_rfc3339(),
                sources: vec!["registry".to_string()],
                popularity_score: compute_popularity_score(0, None),
                install_hint,
            });
        }

        cursor = body.next_cursor;
        if cursor.is_none() {
            break;
        }
    }

    Ok(entries)
}

fn parse_github_url(url: &str) -> Option<String> {
    let normalized = url
        .trim()
        .trim_end_matches(".git")
        .replace("git@github.com:", "https://github.com/");
    if normalized.contains("github.com") {
        Some(normalized)
    } else {
        None
    }
}

fn parse_github_owner_repo(url: &str) -> Option<String> {
    let path = url
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .split("github.com/")
        .nth(1)?;
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 {
        Some(format!("{}/{}", parts[0], parts[1]))
    } else {
        None
    }
}
