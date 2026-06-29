use chrono::Utc;
use serde::Deserialize;

use crate::models::DiscoveredMcpEntry;

use super::super::compute_popularity_score;
use super::super::DiscoveryError;

#[derive(Debug, Deserialize)]
struct GitHubSearchResponse {
    items: Vec<GitHubRepo>,
}

#[derive(Debug, Deserialize)]
struct GitHubRepo {
    full_name: String,
    description: Option<String>,
    html_url: String,
    stargazers_count: i64,
    forks_count: i64,
    pushed_at: Option<String>,
    topics: Option<Vec<String>>,
}

const GITHUB_SEARCH: &str = "https://api.github.com/search/repositories";

pub async fn fetch_github_repos(
    client: &reqwest::Client,
    token: Option<&str>,
) -> Result<Vec<DiscoveredMcpEntry>, DiscoveryError> {
    let mut entries = Vec::new();
    let queries = [
        ("topic:mcp-server fork:false stars:>10", "stars"),
        ("topic:model-context-protocol fork:false stars:>10", "stars"),
        ("topic:mcp-server fork:false stars:>10", "updated"),
        ("topic:model-context-protocol fork:false stars:>10", "updated"),
    ];

    for (q, sort) in queries {
        for page in 1..=10 {
            let url = format!(
                "{GITHUB_SEARCH}?q={}&sort={}&order=desc&per_page=100&page={page}",
                urlencoding::encode(q),
                sort
            );

            let mut req = client.get(&url);
            if let Some(t) = token {
                req = req.header("Authorization", format!("Bearer {t}"));
            }

            let resp = req
                .send()
                .await
                .map_err(|e| DiscoveryError::Http(e.to_string()))?;

            if !resp.status().is_success() {
                if resp.status().as_u16() == 403 {
                    break;
                }
                return Err(DiscoveryError::Http(format!(
                    "GitHub API error: {}",
                    resp.status()
                )));
            }

            let body: GitHubSearchResponse = resp
                .json()
                .await
                .map_err(|e| DiscoveryError::Http(e.to_string()))?;

            if body.items.is_empty() {
                break;
            }

            let item_count = body.items.len();
            for repo in body.items {
                let description = repo.description.unwrap_or_default().trim().to_string();
                if description.is_empty() {
                    continue;
                }
                let tags = repo.topics.unwrap_or_default();
                let updated = repo.pushed_at.clone();
                entries.push(DiscoveredMcpEntry {
                    id: repo.full_name.clone(),
                    name: repo
                        .full_name
                        .split('/')
                        .nth(1)
                        .unwrap_or(&repo.full_name)
                        .to_string(),
                    description,
                    tags,
                    github_url: Some(repo.html_url),
                    homepage_url: None,
                    registry_url: None,
                    github_stars: repo.stargazers_count,
                    github_forks: repo.forks_count,
                    github_updated_at: updated.clone(),
                    discovered_at: Utc::now().to_rfc3339(),
                    sources: vec!["github".to_string()],
                    popularity_score: compute_popularity_score(repo.stargazers_count, updated.as_deref()),
                    install_hint: None,
                });
            }

            if item_count < 100 {
                break;
            }
        }
    }

    Ok(entries)
}
