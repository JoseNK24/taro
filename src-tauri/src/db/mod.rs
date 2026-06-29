use rusqlite::{params, Connection};
use thiserror::Error;

use crate::models::{ClientTargetRecord, HealthCheckRecord, InstallationRecord};

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
            ",
        )?;
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

    pub fn list_installations(&self) -> DbResult<Vec<InstallationRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, integration_id, enabled, status, installed_at, error_message
             FROM installations ORDER BY installed_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(InstallationRecord {
                id: row.get(0)?,
                integration_id: row.get(1)?,
                enabled: row.get::<_, i64>(2)? != 0,
                status: row.get(3)?,
                installed_at: row.get(4)?,
                error_message: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    pub fn get_installation_by_integration(
        &self,
        integration_id: &str,
    ) -> DbResult<Option<InstallationRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, integration_id, enabled, status, installed_at, error_message
             FROM installations WHERE integration_id = ?1",
        )?;
        let mut rows = stmt.query(params![integration_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(InstallationRecord {
                id: row.get(0)?,
                integration_id: row.get(1)?,
                enabled: row.get::<_, i64>(2)? != 0,
                status: row.get(3)?,
                installed_at: row.get(4)?,
                error_message: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_installation(&self, id: &str) -> DbResult<InstallationRecord> {
        let mut stmt = self.conn.prepare(
            "SELECT id, integration_id, enabled, status, installed_at, error_message
             FROM installations WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(InstallationRecord {
                id: row.get(0)?,
                integration_id: row.get(1)?,
                enabled: row.get::<_, i64>(2)? != 0,
                status: row.get(3)?,
                installed_at: row.get(4)?,
                error_message: row.get(5)?,
            })
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
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO installations (id, integration_id, enabled, status, installed_at, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(integration_id) DO UPDATE SET
               enabled = excluded.enabled,
               status = excluded.status,
               error_message = excluded.error_message",
            params![
                id,
                integration_id,
                enabled as i64,
                status,
                now,
                error_message
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
}
