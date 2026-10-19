use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::{now_iso, DbResult};

const CORE_DEFAULTS: &[&str] = &[
    "dashboard",
    "crm",
    "support",
    "projects",
    "docs",
    "insights",
    "teams",
    "search",
    "settings",
];

const ALL_MODULES: &[&str] = &[
    "dashboard",
    "crm",
    "support",
    "projects",
    "operations",
    "finance",
    "portal",
    "docs",
    "insights",
    "teams",
    "search",
    "settings",
];

const ALWAYS_ON: &[&str] = &["dashboard", "search", "settings"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceModule {
    pub workspace_id: String,
    pub module_id: String,
    pub enabled: bool,
    pub mode: String,
    pub connector_id: Option<String>,
    pub updated_at: String,
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<WorkspaceModule> {
    let enabled: i64 = row.get(2)?;
    Ok(WorkspaceModule {
        workspace_id: row.get(0)?,
        module_id: row.get(1)?,
        enabled: enabled != 0,
        mode: row.get(3)?,
        connector_id: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

pub fn ensure_workspace_modules(conn: &Connection, workspace_id: &str) -> DbResult<()> {
    let ts = now_iso();
    for module_id in ALL_MODULES {
        let enabled = CORE_DEFAULTS.contains(module_id) || ALWAYS_ON.contains(module_id);
        conn.execute(
            "INSERT OR IGNORE INTO workspace_modules
             (workspace_id, module_id, enabled, mode, connector_id, updated_at)
             VALUES (?1, ?2, ?3, 'native', NULL, ?4)",
            params![workspace_id, module_id, if enabled { 1 } else { 0 }, ts],
        )?;
    }
    // Always-on modules cannot be disabled.
    for module_id in ALWAYS_ON {
        conn.execute(
            "UPDATE workspace_modules SET enabled = 1, updated_at = ?1
             WHERE workspace_id = ?2 AND module_id = ?3",
            params![ts, workspace_id, module_id],
        )?;
    }
    Ok(())
}

pub fn list_workspace_modules(conn: &Connection, workspace_id: &str) -> DbResult<Vec<WorkspaceModule>> {
    ensure_workspace_modules(conn, workspace_id)?;
    let mut stmt = conn.prepare(
        "SELECT workspace_id, module_id, enabled, mode, connector_id, updated_at
         FROM workspace_modules
         WHERE workspace_id = ?1
         ORDER BY module_id COLLATE NOCASE",
    )?;
    let rows = stmt.query_map(params![workspace_id], map_row)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_workspace_module(
    conn: &Connection,
    workspace_id: &str,
    module_id: &str,
) -> DbResult<Option<WorkspaceModule>> {
    ensure_workspace_modules(conn, workspace_id)?;
    conn.query_row(
        "SELECT workspace_id, module_id, enabled, mode, connector_id, updated_at
         FROM workspace_modules WHERE workspace_id = ?1 AND module_id = ?2",
        params![workspace_id, module_id],
        map_row,
    )
    .optional()
    .map_err(Into::into)
}

pub fn is_module_enabled(conn: &Connection, workspace_id: &str, module_id: &str) -> DbResult<bool> {
    if ALWAYS_ON.contains(&module_id) {
        return Ok(true);
    }
    Ok(get_workspace_module(conn, workspace_id, module_id)?
        .map(|m| m.enabled)
        .unwrap_or(false))
}

pub fn set_workspace_module(
    conn: &Connection,
    workspace_id: &str,
    module_id: &str,
    enabled: bool,
    mode: &str,
    connector_id: Option<&str>,
) -> DbResult<WorkspaceModule> {
    ensure_workspace_modules(conn, workspace_id)?;
    if !ALL_MODULES.contains(&module_id) {
        return Err(super::DbError::Sqlite(rusqlite::Error::InvalidParameterName(
            format!("unknown module: {module_id}"),
        )));
    }
    let always = ALWAYS_ON.contains(&module_id);
    let enabled = if always { true } else { enabled };
    let mode = if mode == "connected" { "connected" } else { "native" };
    let connector_id = if mode == "connected" {
        connector_id
    } else {
        None
    };
    let ts = now_iso();
    conn.execute(
        "UPDATE workspace_modules
         SET enabled = ?1, mode = ?2, connector_id = ?3, updated_at = ?4
         WHERE workspace_id = ?5 AND module_id = ?6",
        params![
            if enabled { 1 } else { 0 },
            mode,
            connector_id,
            ts,
            workspace_id,
            module_id
        ],
    )?;
    get_workspace_module(conn, workspace_id, module_id)?
        .ok_or_else(|| super::DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}

pub fn adopt_module(
    conn: &Connection,
    workspace_id: &str,
    module_id: &str,
) -> DbResult<WorkspaceModule> {
    let current = get_workspace_module(conn, workspace_id, module_id)?
        .ok_or_else(|| super::DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))?;
    // Keep connector_id for deep links but switch source of truth to native.
    let ts = now_iso();
    conn.execute(
        "UPDATE workspace_modules
         SET enabled = 1, mode = 'native', updated_at = ?1
         WHERE workspace_id = ?2 AND module_id = ?3",
        params![ts, workspace_id, module_id],
    )?;
    // Archive active connector instance for this module if present.
    if let Some(cid) = current.connector_id.as_deref() {
        let _ = conn.execute(
            "UPDATE connectors SET status = 'archived', updated_at = ?1, version = version + 1
             WHERE workspace_id = ?2 AND connector_id = ?3 AND deleted_at IS NULL AND status != 'archived'",
            params![ts, workspace_id, cid],
        );
    }
    get_workspace_module(conn, workspace_id, module_id)?
        .ok_or_else(|| super::DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}
