#!/usr/bin/env node
/**
 * Sync discovered MCP servers from GitHub Search API and MCP Registry.
 * Usage: node scripts/sync-discovered-mcps.mjs [--out path] [--min-entries N]
 */

import { mkdir, readFile, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");
const CACHE_DIR = join(ROOT, ".cache/mcp-discovery");
const DEFAULT_OUT = join(ROOT, "src-tauri/resources/discovered_catalog.json");
const MAX_ENTRIES = 2000;
const MIN_ENTRIES_DEFAULT = 10;

const GITHUB_SEARCH = "https://api.github.com/search/repositories";
const REGISTRY_URL = "https://registry.modelcontextprotocol.io/v0/servers";

function parseArgs() {
  const args = process.argv.slice(2);
  let out = DEFAULT_OUT;
  let minEntries = MIN_ENTRIES_DEFAULT;
  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--out" && args[i + 1]) out = args[++i];
    if (args[i] === "--min-entries" && args[i + 1]) minEntries = Number(args[++i]);
  }
  return { out, minEntries };
}

function cacheKey(url) {
  return createHash("sha256").update(url).digest("hex") + ".json";
}

async function cachedFetch(url, options = {}) {
  await mkdir(CACHE_DIR, { recursive: true });
  const key = cacheKey(url);
  const cachePath = join(CACHE_DIR, key);

  try {
    const cached = await readFile(cachePath, "utf8");
    return JSON.parse(cached);
  } catch {
    // miss
  }

  const headers = {
    Accept: "application/vnd.github+json",
    "User-Agent": "taro-mcp-discovery/0.1",
    ...(options.headers ?? {}),
  };

  const resp = await fetch(url, { ...options, headers });
  if (!resp.ok) {
    throw new Error(`HTTP ${resp.status} for ${url}`);
  }
  const data = await resp.json();
  await writeFile(cachePath, JSON.stringify(data));
  return data;
}

function computePopularityScore(stars, updatedAt) {
  const starScore = Math.log10(stars + 1) * 10;
  let recencyBonus = 0;
  if (updatedAt) {
    const days = (Date.now() - new Date(updatedAt).getTime()) / (1000 * 60 * 60 * 24);
    if (days < 30) recencyBonus = 30;
    else if (days < 90) recencyBonus = 15;
  }
  return starScore + recencyBonus;
}

function parseGithubOwnerRepo(url) {
  if (!url) return null;
  const normalized = url.trim().replace(/\.git$/, "").replace("git@github.com:", "https://github.com/");
  const match = normalized.match(/github\.com\/([^/]+)\/([^/]+)/);
  if (!match) return null;
  return `${match[1]}/${match[2]}`;
}

function parseGithubUrl(url) {
  if (!url) return null;
  const normalized = url.trim().replace(/\.git$/, "").replace("git@github.com:", "https://github.com/");
  return normalized.includes("github.com") ? normalized : null;
}

function mergeEntry(map, incoming) {
  const existing = map.get(incoming.id);
  if (!existing) {
    map.set(incoming.id, incoming);
    return;
  }
  if (incoming.description.length > existing.description.length) {
    existing.description = incoming.description;
  }
  if (incoming.name.length > existing.name.length) {
    existing.name = incoming.name;
  }
  for (const tag of incoming.tags) {
    if (!existing.tags.includes(tag)) existing.tags.push(tag);
  }
  for (const src of incoming.sources) {
    if (!existing.sources.includes(src)) existing.sources.push(src);
  }
  if (incoming.github_stars > existing.github_stars) {
    existing.github_stars = incoming.github_stars;
    existing.github_forks = incoming.github_forks;
    existing.github_updated_at = incoming.github_updated_at;
  }
  existing.github_url ??= incoming.github_url;
  existing.homepage_url ??= incoming.homepage_url;
  existing.registry_url ??= incoming.registry_url;
  existing.install_hint ??= incoming.install_hint;
  existing.popularity_score = computePopularityScore(
    existing.github_stars,
    existing.github_updated_at,
  );
}

async function fetchGitHub(token) {
  const entries = [];
  const queries = [
    ["topic:mcp-server fork:false stars:>10", "stars"],
    ["topic:model-context-protocol fork:false stars:>10", "stars"],
    ["topic:mcp-server fork:false stars:>10", "updated"],
    ["topic:model-context-protocol fork:false stars:>10", "updated"],
  ];

  for (const [q, sort] of queries) {
    for (let page = 1; page <= 10; page++) {
      const url = `${GITHUB_SEARCH}?q=${encodeURIComponent(q)}&sort=${sort}&order=desc&per_page=100&page=${page}`;
      const headers = token ? { Authorization: `Bearer ${token}` } : {};
      let data;
      try {
        data = await cachedFetch(url, { headers });
      } catch (err) {
        console.warn(`GitHub fetch warning: ${err.message}`);
        break;
      }

      if (!data.items?.length) break;

      for (const repo of data.items) {
        const description = (repo.description ?? "").trim();
        if (!description) continue;
        const updated = repo.pushed_at ?? null;
        entries.push({
          id: repo.full_name,
          name: repo.full_name.split("/")[1] ?? repo.full_name,
          description,
          tags: repo.topics ?? [],
          github_url: repo.html_url,
          homepage_url: null,
          registry_url: null,
          github_stars: repo.stargazers_count ?? 0,
          github_forks: repo.forks_count ?? 0,
          github_updated_at: updated,
          discovered_at: new Date().toISOString(),
          sources: ["github"],
          popularity_score: computePopularityScore(repo.stargazers_count ?? 0, updated),
          install_hint: null,
        });
      }

      if (data.items.length < 100) break;
    }
  }
  return entries;
}

async function fetchRegistry() {
  const entries = [];
  let cursor = null;

  do {
    const url = cursor ? `${REGISTRY_URL}?cursor=${encodeURIComponent(cursor)}` : REGISTRY_URL;
    let data;
    try {
      data = await cachedFetch(url);
    } catch (err) {
      console.warn(`Registry fetch warning: ${err.message}`);
      break;
    }

    for (const server of data.servers ?? []) {
      const description = (server.description ?? "").trim();
      if (!description) continue;

      const githubUrl = (server.packages ?? [])
        .map((p) => parseGithubUrl(p.repository?.url))
        .find(Boolean) ?? null;

      const installHint = (server.packages ?? [])
        .map((p) => {
          if (!p.identifier) return null;
          const reg = p.registryType ?? p.registry_type ?? "npm";
          return `${reg}:${p.identifier}`;
        })
        .find(Boolean) ?? null;

      const id = parseGithubOwnerRepo(githubUrl) ?? `registry:${server.name}`;

      entries.push({
        id,
        name: server.title || server.name,
        description,
        tags: ["registry"],
        github_url: githubUrl,
        homepage_url: server.remotes?.[0]?.url ?? null,
        registry_url: `https://registry.modelcontextprotocol.io/v0/servers/${server.name}`,
        github_stars: 0,
        github_forks: 0,
        github_updated_at: null,
        discovered_at: new Date().toISOString(),
        sources: ["registry"],
        popularity_score: computePopularityScore(0, null),
        install_hint: installHint,
      });
    }

    cursor = data.nextCursor ?? null;
  } while (cursor);

  return entries;
}

async function main() {
  const { out, minEntries } = parseArgs();
  const token = process.env.GITHUB_TOKEN ?? "";

  console.log("Fetching GitHub MCP repositories…");
  const githubEntries = await fetchGitHub(token);

  console.log("Fetching MCP Registry servers…");
  const registryEntries = await fetchRegistry();

  const map = new Map();
  for (const entry of [...githubEntries, ...registryEntries]) {
    mergeEntry(map, entry);
  }

  let entries = [...map.values()].filter((e) => e.description.length > 0);
  entries.sort((a, b) => b.popularity_score - a.popularity_score);
  entries = entries.slice(0, MAX_ENTRIES);

  if (entries.length < minEntries) {
    console.error(`Error: only ${entries.length} entries (minimum ${minEntries})`);
    process.exit(1);
  }

  const catalog = {
    version: 1,
    generated_at: new Date().toISOString(),
    entries,
  };

  await mkdir(dirname(out), { recursive: true });
  await writeFile(out, JSON.stringify(catalog, null, 2) + "\n");
  console.log(`Wrote ${entries.length} entries to ${out}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
