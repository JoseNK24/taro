use rusqlite::{params, Connection};
use thiserror::Error;

use crate::models::{
    ClientTargetRecord, CommunityInstallMeta, DiscoveredMcpEntry, DiscoverySearchResult,
    DiscoverySyncStats, HarnessInstanceRecord, HealthCheckRecord, InstallationRecord,
    McpServer, PluginClientTargetRecord, PluginInstallationRecord,
};

#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Not found: {0}")]
    NotFound(String),
}

pub type DbResult<T> = Result<T, DbError>;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &std::path::Path) -> DbResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DbError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })?;
        }
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> DbResult<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS installations (
                id TEXT PRIMARY KEY,
                integration_id TEXT NOT NULL UNIQUE,
                enabled INTEGER NOT NULL DEFAULT 1,
                status TEXT NOT NULL DEFAULT 'connected',
                installed_at TEXT NOT NULL,
                error_message TEXT
            );

            CREATE TABLE IF NOT EXISTS client_targets (
                installation_id TEXT NOT NULL,
                client_id TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                PRIMARY KEY (installation_id, client_id),
                FOREIGN KEY (installation_id) REFERENCES installations(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS health_checks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                installation_id TEXT NOT NULL,
                latency_ms INTEGER,
                ok INTEGER NOT NULL,
                checked_at TEXT NOT NULL,
                detail TEXT,
                FOREIGN KEY (installation_id) REFERENCES installations(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS discovered_mcps (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                tags TEXT NOT NULL DEFAULT '[]',
                github_url TEXT,
                homepage_url TEXT,
                registry_url TEXT,
                github_stars INTEGER NOT NULL DEFAULT 0,
                github_forks INTEGER NOT NULL DEFAULT 0,
                github_updated_at TEXT,
                discovered_at TEXT NOT NULL,
                sources TEXT NOT NULL DEFAULT '[]',
                popularity_score REAL NOT NULL DEFAULT 0,
                install_hint TEXT
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS discovered_mcps_fts USING fts5(
                name, description, tags,
                content='discovered_mcps', content_rowid='rowid'
            );

            CREATE TRIGGER IF NOT EXISTS discovered_mcps_ai AFTER INSERT ON discovered_mcps BEGIN
                INSERT INTO discovered_mcps_fts(rowid, name, description, tags)
                VALUES (new.rowid, new.name, new.description, new.tags);
            END;

            CREATE TRIGGER IF NOT EXISTS discovered_mcps_ad AFTER DELETE ON discovered_mcps BEGIN
                INSERT INTO discovered_mcps_fts(discovered_mcps_fts, rowid, name, description, tags)
                VALUES ('delete', old.rowid, old.name, old.description, old.tags);
            END;

            CREATE TRIGGER IF NOT EXISTS discovered_mcps_au AFTER UPDATE ON discovered_mcps BEGIN
                INSERT INTO discovered_mcps_fts(discovered_mcps_fts, rowid, name, description, tags)
                VALUES ('delete', old.rowid, old.name, old.description, old.tags);
                INSERT INTO discovered_mcps_fts(rowid, name, description, tags)
                VALUES (new.rowid, new.name, new.description, new.tags);
            END;

            CREATE TABLE IF NOT EXISTS plugin_installations (
                id TEXT PRIMARY KEY,
                plugin_id TEXT NOT NULL UNIQUE,
                enabled INTEGER NOT NULL DEFAULT 1,
                status TEXT NOT NULL DEFAULT 'installed',
                installed_at TEXT NOT NULL,
                error_message TEXT
            );

            CREATE TABLE IF NOT EXISTS plugin_client_targets (
                installation_id TEXT NOT NULL,
                client_id TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                status TEXT NOT NULL DEFAULT 'installed',
                error_message TEXT,
                PRIMARY KEY (installation_id, client_id),
                FOREIGN KEY (installation_id) REFERENCES plugin_installations(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS harness_instances (
                id TEXT PRIMARY KEY,
                driver_kind TEXT NOT NULL,
                display_name TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                is_default_install_agent INTEGER NOT NULL DEFAULT 0,
                config_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS community_install_meta (
                installation_id TEXT PRIMARY KEY,
                discovered_mcp_id TEXT NOT NULL,
                harness_instance_id TEXT NOT NULL,
                resolved_server_json TEXT NOT NULL,
                env_keys_json TEXT NOT NULL DEFAULT '[]',
                agent_log TEXT,
                FOREIGN KEY (installation_id) REFERENCES installations(id) ON DELETE CASCADE
            );

            -- Ownership annotation: the exact (client, server_id) entries Taro
            -- wrote. Source of truth for uninstall, so we never re-derive a name
            -- and never touch a server we did not install.
            CREATE TABLE IF NOT EXISTS managed_servers (
                client_id TEXT NOT NULL,
                server_id TEXT NOT NULL,
                installation_id TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (client_id, server_id)
            );
            ",
        )?;
        self.migrate_schema()?;
        self.backfill_managed_servers()?;
        Ok(())
    }

    /// Seed `managed_servers` from installs that predate the table, using the
    /// id scheme each install actually wrote, so existing integrations stay
    /// removable after we stop prefixing new names.
    fn backfill_managed_servers(&self) -> DbResult<()> {
        if self.get_setting("managed_servers_backfilled")?.as_deref() == Some("true") {
            return Ok(());
        }
        for inst in self.list_installations()? {
            let server_id = if inst.source == "community" {
                match self.get_community_install_meta(&inst.id)? {
                    Some(meta) => meta.resolved_server.id,
                    None => continue,
                }
            } else {
                format!("taro-{}", inst.integration_id)
            };
            for target in self.list_client_targets(&inst.id)? {
                self.mark_managed_server(&target.client_id, &server_id, Some(&inst.id))?;
            }
        }
        self.set_setting("managed_servers_backfilled", "true")
    }

    pub fn mark_managed_server(
        &self,
        client_id: &str,
        server_id: &str,
        installation_id: Option<&str>,
    ) -> DbResult<()> {
        self.conn.execute(
            "INSERT INTO managed_servers (client_id, server_id, installation_id)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(client_id, server_id) DO UPDATE SET installation_id = excluded.installation_id",
            params![client_id, server_id, installation_id],
        )?;
        Ok(())
    }

    pub fn unmark_managed_server(&self, client_id: &str, server_id: &str) -> DbResult<()> {
        self.conn.execute(
            "DELETE FROM managed_servers WHERE client_id = ?1 AND server_id = ?2",
            params![client_id, server_id],
        )?;
        Ok(())
    }

    pub fn is_managed_server(&self, client_id: &str, server_id: &str) -> DbResult<bool> {
        Ok(self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM managed_servers WHERE client_id = ?1 AND server_id = ?2",
            params![client_id, server_id],
            |row| row.get(0),
        )?)
    }

    /// (client_id, server_id) pairs Taro wrote for a given installation.
    pub fn managed_servers_for_installation(
        &self,
        installation_id: &str,
    ) -> DbResult<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT client_id, server_id FROM managed_servers WHERE installation_id = ?1",
        )?;
        let rows = stmt.query_map(params![installation_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    fn migrate_schema(&self) -> DbResult<()> {
        let has_source: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('installations') WHERE name = 'source'",
            [],
            |row| row.get(0),
        )?;
        if !has_source {
            self.conn.execute(
                "ALTER TABLE installations ADD COLUMN source TEXT NOT NULL DEFAULT 'curated'",
                [],
            )?;
        }
        let has_env_keys: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('community_install_meta') WHERE name = 'env_keys_json'",
            [],
            |row| row.get(0),
        )?;
        if !has_env_keys {
            self.conn.execute(
                "ALTER TABLE community_install_meta ADD COLUMN env_keys_json TEXT NOT NULL DEFAULT '[]'",
                [],
            )?;
        }
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> DbResult<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> DbResult<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn is_first_run(&self) -> DbResult<bool> {
        Ok(self
            .get_setting("first_run_completed")?
            .map(|v| v != "true")
            .unwrap_or(true))
    }

    pub fn complete_first_run(&self) -> DbResult<()> {
        self.set_setting("first_run_completed", "true")
    }

    fn row_to_installation(row: &rusqlite::Row<'_>) -> Result<InstallationRecord, rusqlite::Error> {
        Ok(InstallationRecord {
            id: row.get(0)?,
            integration_id: row.get(1)?,
            enabled: row.get::<_, i64>(2)? != 0,
            status: row.get(3)?,
            installed_at: row.get(4)?,
            error_message: row.get(5)?,
            source: row.get::<_, String>(6).unwrap_or_else(|_| "curated".to_string()),
        })
    }

    pub fn list_installations(&self) -> DbResult<Vec<InstallationRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, integration_id, enabled, status, installed_at, error_message, source
             FROM installations ORDER BY installed_at DESC",
        )?;
        let rows = stmt.query_map([], Self::row_to_installation)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    pub fn get_installation_by_integration(
        &self,
        integration_id: &str,
    ) -> DbResult<Option<InstallationRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, integration_id, enabled, status, installed_at, error_message, source
             FROM installations WHERE integration_id = ?1",
        )?;
        let mut rows = stmt.query(params![integration_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_installation(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_installation(&self, id: &str) -> DbResult<InstallationRecord> {
        let mut stmt = self.conn.prepare(
            "SELECT id, integration_id, enabled, status, installed_at, error_message, source
             FROM installations WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Self::row_to_installation(row)?)
        } else {
            Err(DbError::NotFound(format!("installation {id}")))
        }
    }

    pub fn upsert_installation(
        &self,
        id: &str,
        integration_id: &str,
        enabled: bool,
        status: &str,
        error_message: Option<&str>,
    ) -> DbResult<()> {
        self.upsert_installation_with_source(
            id,
            integration_id,
            enabled,
            status,
            error_message,
            "curated",
        )
    }

    pub fn upsert_installation_with_source(
        &self,
        id: &str,
        integration_id: &str,
        enabled: bool,
        status: &str,
        error_message: Option<&str>,
        source: &str,
    ) -> DbResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO installations (id, integration_id, enabled, status, installed_at, error_message, source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(integration_id) DO UPDATE SET
               enabled = excluded.enabled,
               status = excluded.status,
               error_message = excluded.error_message,
               source = excluded.source",
            params![
                id,
                integration_id,
                enabled as i64,
                status,
                now,
                error_message,
                source
            ],
        )?;
        Ok(())
    }

    pub fn update_installation_status(
        &self,
        id: &str,
        status: &str,
        error_message: Option<&str>,
    ) -> DbResult<()> {
        self.conn.execute(
            "UPDATE installations SET status = ?2, error_message = ?3 WHERE id = ?1",
            params![id, status, error_message],
        )?;
        Ok(())
    }

    pub fn set_installation_enabled(&self, id: &str, enabled: bool) -> DbResult<()> {
        self.conn.execute(
            "UPDATE installations SET enabled = ?2 WHERE id = ?1",
            params![id, enabled as i64],
        )?;
        Ok(())
    }

    pub fn delete_installation(&self, id: &str) -> DbResult<()> {
        self.conn
            .execute("DELETE FROM installations WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_client_targets(&self, installation_id: &str) -> DbResult<Vec<ClientTargetRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT installation_id, client_id, enabled FROM client_targets
             WHERE installation_id = ?1",
        )?;
        let rows = stmt.query_map(params![installation_id], |row| {
            Ok(ClientTargetRecord {
                installation_id: row.get(0)?,
                client_id: row.get(1)?,
                enabled: row.get::<_, i64>(2)? != 0,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    pub fn set_client_target(
        &self,
        installation_id: &str,
        client_id: &str,
        enabled: bool,
    ) -> DbResult<()> {
        self.conn.execute(
            "INSERT INTO client_targets (installation_id, client_id, enabled)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(installation_id, client_id) DO UPDATE SET enabled = excluded.enabled",
            params![installation_id, client_id, enabled as i64],
        )?;
        Ok(())
    }

    pub fn list_enabled_client_targets(
        &self,
        installation_id: &str,
    ) -> DbResult<Vec<ClientTargetRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT installation_id, client_id, enabled FROM client_targets
             WHERE installation_id = ?1 AND enabled = 1",
        )?;
        let rows = stmt.query_map(params![installation_id], |row| {
            Ok(ClientTargetRecord {
                installation_id: row.get(0)?,
                client_id: row.get(1)?,
                enabled: row.get::<_, i64>(2)? != 0,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    pub fn insert_health_check(
        &self,
        installation_id: &str,
        latency_ms: Option<i64>,
        ok: bool,
        detail: Option<&str>,
    ) -> DbResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO health_checks (installation_id, latency_ms, ok, checked_at, detail)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![installation_id, latency_ms, ok as i64, now, detail],
        )?;
        Ok(())
    }

    pub fn latest_health_checks(&self) -> DbResult<Vec<HealthCheckRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT h.installation_id, i.integration_id, '', h.latency_ms, h.ok, h.checked_at, h.detail
             FROM health_checks h
             INNER JOIN installations i ON i.id = h.installation_id
             WHERE h.id IN (
               SELECT MAX(id) FROM health_checks GROUP BY installation_id
             )
             ORDER BY h.checked_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(HealthCheckRecord {
                installation_id: row.get(0)?,
                integration_id: row.get(1)?,
                integration_name: row.get(2)?,
                latency_ms: row.get(3)?,
                ok: row.get::<_, i64>(4)? != 0,
                checked_at: row.get(5)?,
                detail: row.get(6)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    pub fn discovered_mcp_count(&self) -> DbResult<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM discovered_mcps", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn import_discovered_mcps(&self, entries: &[DiscoveredMcpEntry]) -> DbResult<()> {
        for entry in entries {
            self.upsert_discovered_mcp(entry)?;
        }
        Ok(())
    }

    pub fn upsert_discovered_mcps(&self, entries: &[DiscoveredMcpEntry]) -> DbResult<DiscoverySyncStats> {
        let mut added = 0i64;
        let mut updated = 0i64;
        for entry in entries {
            let exists: bool = self
                .conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM discovered_mcps WHERE id = ?1",
                    params![entry.id],
                    |row| row.get(0),
                )
                .unwrap_or(false);
            self.upsert_discovered_mcp(entry)?;
            if exists {
                updated += 1;
            } else {
                added += 1;
            }
        }
        Ok(DiscoverySyncStats {
            added,
            updated,
            errors: 0,
        })
    }

    fn upsert_discovered_mcp(&self, entry: &DiscoveredMcpEntry) -> DbResult<()> {
        let tags = serde_json::to_string(&entry.tags).map_err(|e| {
            DbError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
        })?;
        let sources = serde_json::to_string(&entry.sources).map_err(|e| {
            DbError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
        })?;
        self.conn.execute(
            "INSERT INTO discovered_mcps (
                id, name, description, tags, github_url, homepage_url, registry_url,
                github_stars, github_forks, github_updated_at, discovered_at, sources,
                popularity_score, install_hint
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                tags = excluded.tags,
                github_url = excluded.github_url,
                homepage_url = excluded.homepage_url,
                registry_url = excluded.registry_url,
                github_stars = excluded.github_stars,
                github_forks = excluded.github_forks,
                github_updated_at = excluded.github_updated_at,
                sources = excluded.sources,
                popularity_score = excluded.popularity_score,
                install_hint = excluded.install_hint",
            params![
                entry.id,
                entry.name,
                entry.description,
                tags,
                entry.github_url,
                entry.homepage_url,
                entry.registry_url,
                entry.github_stars,
                entry.github_forks,
                entry.github_updated_at,
                entry.discovered_at,
                sources,
                entry.popularity_score,
                entry.install_hint,
            ],
        )?;
        Ok(())
    }

    pub fn get_discovered_mcp(&self, id: &str) -> DbResult<Option<DiscoveredMcpEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, tags, github_url, homepage_url, registry_url,
                    github_stars, github_forks, github_updated_at, discovered_at, sources,
                    popularity_score, install_hint
             FROM discovered_mcps WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_discovered_mcp(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn search_discovered_mcps(
        &self,
        query: &str,
        sort: &str,
        limit: i64,
        offset: i64,
    ) -> DbResult<DiscoverySearchResult> {
        let order = match sort {
            "stars" => "d.github_stars DESC",
            "recent" => "d.github_updated_at DESC NULLS LAST",
            _ => "d.popularity_score DESC",
        };

        let trimmed = query.trim();
        if trimmed.is_empty() {
            let count: i64 = self.discovered_mcp_count()?;
            let sql = format!(
                "SELECT id, name, description, tags, github_url, homepage_url, registry_url,
                        github_stars, github_forks, github_updated_at, discovered_at, sources,
                        popularity_score, install_hint
                 FROM discovered_mcps d
                 ORDER BY {order}
                 LIMIT ?1 OFFSET ?2"
            );
            let mut stmt = self.conn.prepare(&sql)?;
            let rows = stmt.query_map(params![limit, offset], row_to_discovered_mcp)?;
            let entries = rows.collect::<Result<Vec<_>, _>>()?;
            return Ok(DiscoverySearchResult { entries, total: count });
        }

        let fts_query = build_fts_query(trimmed);
        let count_sql = "SELECT COUNT(*) FROM discovered_mcps d
                         INNER JOIN discovered_mcps_fts fts ON d.rowid = fts.rowid
                         WHERE discovered_mcps_fts MATCH ?1";
        let count: i64 = self
            .conn
            .query_row(count_sql, params![fts_query], |row| row.get(0))?;

        let sql = format!(
            "SELECT d.id, d.name, d.description, d.tags, d.github_url, d.homepage_url,
                    d.registry_url, d.github_stars, d.github_forks, d.github_updated_at,
                    d.discovered_at, d.sources, d.popularity_score, d.install_hint
             FROM discovered_mcps d
             INNER JOIN discovered_mcps_fts fts ON d.rowid = fts.rowid
             WHERE discovered_mcps_fts MATCH ?1
             ORDER BY {order}
             LIMIT ?2 OFFSET ?3"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt
            .query_map(params![fts_query, limit, offset], row_to_discovered_mcp)?;
        let entries = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(DiscoverySearchResult { entries, total: count })
    }

    pub fn list_plugin_installations(&self) -> DbResult<Vec<PluginInstallationRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, plugin_id, enabled, status, installed_at, error_message
             FROM plugin_installations ORDER BY installed_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(PluginInstallationRecord {
                id: row.get(0)?,
                plugin_id: row.get(1)?,
                enabled: row.get::<_, i64>(2)? != 0,
                status: row.get(3)?,
                installed_at: row.get(4)?,
                error_message: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    pub fn get_plugin_installation_by_plugin(
        &self,
        plugin_id: &str,
    ) -> DbResult<Option<PluginInstallationRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, plugin_id, enabled, status, installed_at, error_message
             FROM plugin_installations WHERE plugin_id = ?1",
        )?;
        let mut rows = stmt.query(params![plugin_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(PluginInstallationRecord {
                id: row.get(0)?,
                plugin_id: row.get(1)?,
                enabled: row.get::<_, i64>(2)? != 0,
                status: row.get(3)?,
                installed_at: row.get(4)?,
                error_message: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_plugin_installation(&self, id: &str) -> DbResult<PluginInstallationRecord> {
        let mut stmt = self.conn.prepare(
            "SELECT id, plugin_id, enabled, status, installed_at, error_message
             FROM plugin_installations WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(PluginInstallationRecord {
                id: row.get(0)?,
                plugin_id: row.get(1)?,
                enabled: row.get::<_, i64>(2)? != 0,
                status: row.get(3)?,
                installed_at: row.get(4)?,
                error_message: row.get(5)?,
            })
        } else {
            Err(DbError::NotFound(format!("plugin installation {id}")))
        }
    }

    pub fn upsert_plugin_installation(
        &self,
        id: &str,
        plugin_id: &str,
        enabled: bool,
        status: &str,
        error_message: Option<&str>,
    ) -> DbResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO plugin_installations (id, plugin_id, enabled, status, installed_at, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(plugin_id) DO UPDATE SET
               enabled = excluded.enabled,
               status = excluded.status,
               error_message = excluded.error_message",
            params![id, plugin_id, enabled as i64, status, now, error_message],
        )?;
        Ok(())
    }

    pub fn update_plugin_installation_status(
        &self,
        id: &str,
        status: &str,
        error_message: Option<&str>,
    ) -> DbResult<()> {
        self.conn.execute(
            "UPDATE plugin_installations SET status = ?2, error_message = ?3 WHERE id = ?1",
            params![id, status, error_message],
        )?;
        Ok(())
    }

    pub fn set_plugin_installation_enabled(&self, id: &str, enabled: bool) -> DbResult<()> {
        self.conn.execute(
            "UPDATE plugin_installations SET enabled = ?2 WHERE id = ?1",
            params![id, enabled as i64],
        )?;
        Ok(())
    }

    pub fn delete_plugin_installation(&self, id: &str) -> DbResult<()> {
        self.conn.execute(
            "DELETE FROM plugin_installations WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn list_plugin_client_targets(
        &self,
        installation_id: &str,
    ) -> DbResult<Vec<PluginClientTargetRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT installation_id, client_id, enabled, status, error_message
             FROM plugin_client_targets WHERE installation_id = ?1",
        )?;
        let rows = stmt.query_map(params![installation_id], |row| {
            Ok(PluginClientTargetRecord {
                installation_id: row.get(0)?,
                client_id: row.get(1)?,
                enabled: row.get::<_, i64>(2)? != 0,
                status: row.get(3)?,
                error_message: row.get(4)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    pub fn set_plugin_client_target(
        &self,
        installation_id: &str,
        client_id: &str,
        enabled: bool,
        status: &str,
        error_message: Option<&str>,
    ) -> DbResult<()> {
        self.conn.execute(
            "INSERT INTO plugin_client_targets (installation_id, client_id, enabled, status, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(installation_id, client_id) DO UPDATE SET
               enabled = excluded.enabled,
               status = excluded.status,
               error_message = excluded.error_message",
            params![installation_id, client_id, enabled as i64, status, error_message],
        )?;
        Ok(())
    }

    pub fn list_harness_instances(&self) -> DbResult<Vec<HarnessInstanceRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, driver_kind, display_name, enabled, is_default_install_agent, config_json, created_at
             FROM harness_instances ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(HarnessInstanceRecord {
                id: row.get(0)?,
                driver_kind: row.get(1)?,
                display_name: row.get(2)?,
                enabled: row.get::<_, i64>(3)? != 0,
                is_default_install_agent: row.get::<_, i64>(4)? != 0,
                config_json: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    pub fn get_harness_instance(&self, id: &str) -> DbResult<HarnessInstanceRecord> {
        let mut stmt = self.conn.prepare(
            "SELECT id, driver_kind, display_name, enabled, is_default_install_agent, config_json, created_at
             FROM harness_instances WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(HarnessInstanceRecord {
                id: row.get(0)?,
                driver_kind: row.get(1)?,
                display_name: row.get(2)?,
                enabled: row.get::<_, i64>(3)? != 0,
                is_default_install_agent: row.get::<_, i64>(4)? != 0,
                config_json: row.get(5)?,
                created_at: row.get(6)?,
            })
        } else {
            Err(DbError::NotFound(format!("harness instance {id}")))
        }
    }

    pub fn create_harness_instance(&self, record: &HarnessInstanceRecord) -> DbResult<()> {
        self.conn.execute(
            "INSERT INTO harness_instances (id, driver_kind, display_name, enabled, is_default_install_agent, config_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                record.id,
                record.driver_kind,
                record.display_name,
                record.enabled as i64,
                record.is_default_install_agent as i64,
                record.config_json,
                record.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn update_harness_instance(
        &self,
        id: &str,
        display_name: &str,
        enabled: bool,
        config_json: &str,
    ) -> DbResult<()> {
        self.conn.execute(
            "UPDATE harness_instances SET display_name = ?2, enabled = ?3, config_json = ?4 WHERE id = ?1",
            params![id, display_name, enabled as i64, config_json],
        )?;
        Ok(())
    }

    pub fn delete_harness_instance(&self, id: &str) -> DbResult<()> {
        self.conn
            .execute("DELETE FROM harness_instances WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn set_default_install_agent(&self, instance_id: &str) -> DbResult<()> {
        self.conn
            .execute("UPDATE harness_instances SET is_default_install_agent = 0", [])?;
        self.conn.execute(
            "UPDATE harness_instances SET is_default_install_agent = 1 WHERE id = ?1",
            params![instance_id],
        )?;
        Ok(())
    }

    pub fn upsert_community_install_meta(&self, meta: &CommunityInstallMeta) -> DbResult<()> {
        let server_json = serde_json::to_string(&meta.resolved_server).map_err(|e| {
            DbError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
        })?;
        let env_keys_json = serde_json::to_string(&meta.env_keys).map_err(|e| {
            DbError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
        })?;
        self.conn.execute(
            "INSERT INTO community_install_meta (installation_id, discovered_mcp_id, harness_instance_id, resolved_server_json, env_keys_json, agent_log)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(installation_id) DO UPDATE SET
               discovered_mcp_id = excluded.discovered_mcp_id,
               harness_instance_id = excluded.harness_instance_id,
               resolved_server_json = excluded.resolved_server_json,
               env_keys_json = excluded.env_keys_json,
               agent_log = excluded.agent_log",
            params![
                meta.installation_id,
                meta.discovered_mcp_id,
                meta.harness_instance_id,
                server_json,
                env_keys_json,
                meta.agent_log,
            ],
        )?;
        Ok(())
    }

    pub fn get_community_install_meta(
        &self,
        installation_id: &str,
    ) -> DbResult<Option<CommunityInstallMeta>> {
        let mut stmt = self.conn.prepare(
            "SELECT installation_id, discovered_mcp_id, harness_instance_id, resolved_server_json, env_keys_json, agent_log
             FROM community_install_meta WHERE installation_id = ?1",
        )?;
        let mut rows = stmt.query(params![installation_id])?;
        if let Some(row) = rows.next()? {
            let server_json: String = row.get(3)?;
            let env_keys_json: String = row.get(4)?;
            let resolved_server: McpServer = serde_json::from_str(&server_json).map_err(|e| {
                DbError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })?;
            let env_keys: Vec<String> = serde_json::from_str(&env_keys_json).unwrap_or_default();
            Ok(Some(CommunityInstallMeta {
                installation_id: row.get(0)?,
                discovered_mcp_id: row.get(1)?,
                harness_instance_id: row.get(2)?,
                resolved_server,
                env_keys,
                agent_log: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_community_meta_by_discovered_id(
        &self,
        discovered_mcp_id: &str,
    ) -> DbResult<Option<CommunityInstallMeta>> {
        let mut stmt = self.conn.prepare(
            "SELECT installation_id, discovered_mcp_id, harness_instance_id, resolved_server_json, env_keys_json, agent_log
             FROM community_install_meta WHERE discovered_mcp_id = ?1",
        )?;
        let mut rows = stmt.query(params![discovered_mcp_id])?;
        if let Some(row) = rows.next()? {
            let server_json: String = row.get(3)?;
            let env_keys_json: String = row.get(4)?;
            let resolved_server: McpServer = serde_json::from_str(&server_json).map_err(|e| {
                DbError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })?;
            let env_keys: Vec<String> = serde_json::from_str(&env_keys_json).unwrap_or_default();
            Ok(Some(CommunityInstallMeta {
                installation_id: row.get(0)?,
                discovered_mcp_id: row.get(1)?,
                harness_instance_id: row.get(2)?,
                resolved_server,
                env_keys,
                agent_log: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_enabled_plugin_client_targets(
        &self,
        installation_id: &str,
    ) -> DbResult<Vec<PluginClientTargetRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT installation_id, client_id, enabled, status, error_message
             FROM plugin_client_targets
             WHERE installation_id = ?1 AND enabled = 1",
        )?;
        let rows = stmt.query_map(params![installation_id], |row| {
            Ok(PluginClientTargetRecord {
                installation_id: row.get(0)?,
                client_id: row.get(1)?,
                enabled: row.get::<_, i64>(2)? != 0,
                status: row.get(3)?,
                error_message: row.get(4)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }
}

fn row_to_discovered_mcp(row: &rusqlite::Row<'_>) -> Result<DiscoveredMcpEntry, rusqlite::Error> {
    let tags_str: String = row.get(3)?;
    let sources_str: String = row.get(11)?;
    let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
    let sources: Vec<String> = serde_json::from_str(&sources_str).unwrap_or_default();
    Ok(DiscoveredMcpEntry {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        tags,
        github_url: row.get(4)?,
        homepage_url: row.get(5)?,
        registry_url: row.get(6)?,
        github_stars: row.get(7)?,
        github_forks: row.get(8)?,
        github_updated_at: row.get(9)?,
        discovered_at: row.get(10)?,
        sources,
        popularity_score: row.get(12)?,
        install_hint: row.get(13)?,
    })
}

fn build_fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .map(|token| {
            let escaped = token.replace('"', "\"\"");
            format!("\"{escaped}\"*")
        })
        .collect::<Vec<_>>()
        .join(" OR ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_servers_roundtrip() {
        let path = std::env::temp_dir().join(format!("taro-managed-{}.db", uuid::Uuid::new_v4()));
        let db = Database::open(&path).unwrap();

        assert!(!db.is_managed_server("claude-code", "filesystem").unwrap());

        db.mark_managed_server("claude-code", "filesystem", Some("inst-1"))
            .unwrap();
        db.mark_managed_server("cursor", "filesystem", Some("inst-1"))
            .unwrap();

        assert!(db.is_managed_server("claude-code", "filesystem").unwrap());
        let pairs = db.managed_servers_for_installation("inst-1").unwrap();
        assert_eq!(pairs.len(), 2);

        db.unmark_managed_server("claude-code", "filesystem").unwrap();
        assert!(!db.is_managed_server("claude-code", "filesystem").unwrap());
        assert_eq!(db.managed_servers_for_installation("inst-1").unwrap().len(), 1);

        let _ = std::fs::remove_file(path);
    }
}
