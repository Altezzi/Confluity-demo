use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

pub mod ai_config;
pub mod connectors;
pub mod identity;
pub mod extended;
pub mod insights;
pub mod migrations;
pub mod models;
pub mod modules;
pub mod obs;
pub mod positions;
pub mod roles;
pub mod seed;
pub mod setup;
pub mod teams;
pub mod ticket_forms;


#[cfg(test)]
mod tests;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub type DbResult<T> = Result<T, DbError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageMode {
    Local,
    Server,
    Hybrid,
}

impl StorageMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            StorageMode::Local => "local",
            StorageMode::Server => "server",
            StorageMode::Hybrid => "hybrid",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "server" => StorageMode::Server,
            "hybrid" => StorageMode::Hybrid,
            _ => StorageMode::Local,
        }
    }
}

pub struct WorkspacePaths {
    pub root: PathBuf,
    pub db: PathBuf,
    pub attachments: PathBuf,
    pub cache: PathBuf,
    pub logs: PathBuf,
    pub config: PathBuf,
}

impl WorkspacePaths {
    pub fn resolve() -> DbResult<Self> {
        let root = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Confluity Demo");
        let paths = Self {
            db: root.join("confluity.db"),
            attachments: root.join("attachments"),
            cache: root.join("cache"),
            logs: root.join("logs"),
            config: root.join("config"),
            root,
        };
        std::fs::create_dir_all(&paths.root)?;
        std::fs::create_dir_all(&paths.attachments)?;
        std::fs::create_dir_all(&paths.cache)?;
        std::fs::create_dir_all(&paths.logs)?;
        std::fs::create_dir_all(&paths.config)?;
        Ok(paths)
    }
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn open_db(path: &Path) -> DbResult<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        ",
    )?;
    migrations::run(&conn)?;
    Ok(conn)
}

pub fn get_meta(conn: &Connection, key: &str) -> DbResult<Option<String>> {
    let value = conn
        .query_row(
            "SELECT value FROM app_meta WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()?;
    Ok(value)
}

pub fn set_meta(conn: &Connection, key: &str, value: &str) -> DbResult<()> {
    conn.execute(
        "INSERT INTO app_meta (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn log_activity(
    conn: &Connection,
    workspace_id: &str,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    summary: &str,
) -> DbResult<()> {
    let id = new_id();
    let ts = now_iso();
    conn.execute(
        "INSERT INTO activities (
            id, workspace_id, actor, action, entity_type, entity_id, summary, metadata,
            created_at, updated_at, deleted_at, version
         ) VALUES (?1, ?2, 'local-user', ?3, ?4, ?5, ?6, NULL, ?7, ?7, NULL, 1)",
        params![id, workspace_id, action, entity_type, entity_id, summary, ts],
    )?;
    Ok(())
}

pub fn upsert_fts(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
    title: &str,
    body: &str,
) -> DbResult<()> {
    conn.execute(
        "DELETE FROM search_fts WHERE entity_type = ?1 AND entity_id = ?2",
        params![entity_type, entity_id],
    )?;
    conn.execute(
        "INSERT INTO search_fts (entity_type, entity_id, title, body) VALUES (?1, ?2, ?3, ?4)",
        params![entity_type, entity_id, title, body],
    )?;
    Ok(())
}

pub fn log_audit(
    conn: &Connection,
    workspace_id: &str,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    summary: &str,
) -> DbResult<()> {
    conn.execute(
        "INSERT INTO audit_log (id, workspace_id, actor, action, entity_type, entity_id, summary, metadata, created_at)
         VALUES (?1, ?2, 'local-user', ?3, ?4, ?5, ?6, NULL, ?7)",
        params![new_id(), workspace_id, action, entity_type, entity_id, summary, now_iso()],
    )?;
    Ok(())
}

pub fn enqueue_outbox(
    conn: &Connection,
    workspace_id: &str,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    summary: &str,
) -> DbResult<()> {
    conn.execute(
        "INSERT INTO outbox (id, workspace_id, entity_type, entity_id, action, payload, status, created_at, sent_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'queued', ?7, NULL)",
        params![new_id(), workspace_id, entity_type, entity_id, action, summary, now_iso()],
    )?;
    Ok(())
}

pub fn emit_event(
    conn: &Connection,
    workspace_id: &str,
    event_type: &str,
    entity_type: &str,
    entity_id: &str,
    payload: &str,
) -> DbResult<()> {
    conn.execute(
        "INSERT INTO workspace_events (id, workspace_id, event_type, entity_type, entity_id, payload, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![new_id(), workspace_id, event_type, entity_type, entity_id, payload, now_iso()],
    )?;
    Ok(())
}

pub fn notify_workspace(
    conn: &Connection,
    workspace_id: &str,
    member_id: Option<&str>,
    title: &str,
    body: &str,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
) -> DbResult<()> {
    conn.execute(
        "INSERT INTO notifications (id, workspace_id, member_id, title, body, entity_type, entity_id, read_at, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, ?8)",
        params![
            new_id(),
            workspace_id,
            member_id,
            title,
            body,
            entity_type,
            entity_id,
            now_iso()
        ],
    )?;
    Ok(())
}

pub fn record_change(
    conn: &Connection,
    workspace_id: &str,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    summary: &str,
) -> DbResult<()> {
    log_activity(conn, workspace_id, action, entity_type, entity_id, summary)?;
    let _ = log_audit(conn, workspace_id, action, entity_type, entity_id, summary);
    let _ = enqueue_outbox(conn, workspace_id, action, entity_type, entity_id, summary);
    let _ = emit_event(
        conn,
        workspace_id,
        &format!("{entity_type}.{action}"),
        entity_type,
        entity_id,
        summary,
    );
    Ok(())
}
