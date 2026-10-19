use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::{
    ai_config::{load_api_connections, write_api_connections, StoredApiConnection, any_api_key_configured},
    get_meta,
    models::{ApiConnection, AppSettings, McpConnection, Workspace},
    seed::ensure_seeded,
    set_meta, StorageMode, DbError,
};
use crate::state::AppState;

fn map_err(e: DbError) -> String {
    e.to_string()
}

const MCP_META_KEY: &str = "mcp_connections";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredMcpConnection {
    id: String,
    name: String,
    transport: String,
    command: Option<String>,
    args: Vec<String>,
    url: Option<String>,
    auth_token: Option<String>,
    enabled: bool,
}

fn read_stored_mcp(conn: &rusqlite::Connection) -> Result<Vec<StoredMcpConnection>, String> {
    let raw = get_meta(conn, MCP_META_KEY).map_err(map_err)?;
    let Some(json) = raw.filter(|s| !s.trim().is_empty()) else {
        return Ok(Vec::new());
    };
    serde_json::from_str(&json).map_err(|e| format!("Invalid MCP config: {e}"))
}

fn write_stored_mcp(conn: &rusqlite::Connection, items: &[StoredMcpConnection]) -> Result<(), String> {
    let json = serde_json::to_string(items).map_err(|e| format!("MCP config encode: {e}"))?;
    set_meta(conn, MCP_META_KEY, &json).map_err(map_err)
}

fn to_public_mcp(stored: &StoredMcpConnection) -> McpConnection {
    McpConnection {
        id: stored.id.clone(),
        name: stored.name.clone(),
        transport: stored.transport.clone(),
        command: stored.command.clone(),
        args: stored.args.clone(),
        url: stored.url.clone(),
        auth_token_configured: stored
            .auth_token
            .as_ref()
            .is_some_and(|t| !t.trim().is_empty()),
        enabled: stored.enabled,
    }
}

fn to_public_api(stored: &StoredApiConnection) -> ApiConnection {
    ApiConnection {
        id: stored.id.clone(),
        name: stored.name.clone(),
        base_url: stored.base_url.clone(),
        model: stored.model.clone(),
        api_key_configured: stored
            .api_key
            .as_ref()
            .is_some_and(|k| !k.trim().is_empty()),
        enabled: stored.enabled,
        is_default: stored.is_default,
    }
}

fn normalize_api_defaults(items: &mut [StoredApiConnection]) {
    if items.is_empty() {
        return;
    }
    let has_default = items.iter().any(|c| c.is_default);
    if !has_default {
        if let Some(first) = items.iter_mut().find(|c| c.enabled) {
            first.is_default = true;
        } else {
            items[0].is_default = true;
        }
    }
    let default_id = items
        .iter()
        .find(|c| c.is_default)
        .map(|c| c.id.clone())
        .unwrap_or_else(|| items[0].id.clone());
    for c in items.iter_mut() {
        c.is_default = c.id == default_id;
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateAiSettingsInput {
    pub ai_api_base_url: Option<String>,
    pub ai_model: Option<String>,
    pub ai_api_key: Option<String>,
    pub clear_ai_api_key: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SaveMcpConnectionInput {
    pub id: String,
    pub name: String,
    pub transport: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub url: Option<String>,
    pub auth_token: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct SaveApiConnectionInput {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub clear_api_key: Option<bool>,
    pub enabled: bool,
    pub is_default: bool,
}

#[tauri::command]
pub fn get_settings_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<AppSettings, String> {
    let conn = state.conn.lock();
    let workspace_id = get_meta(&conn, "workspace_id")
        .map_err(map_err)?
        .unwrap_or_default();
    let organization_id = get_meta(&conn, "organization_id")
        .map_err(map_err)?
        .unwrap_or_default();

    let workspace_name = conn
        .query_row(
            "SELECT name FROM workspaces WHERE id = ?1",
            rusqlite::params![workspace_id],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "Workspace".into());

    let organization_name = conn
        .query_row(
            "SELECT name FROM organizations WHERE id = ?1",
            rusqlite::params![organization_id],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "Organization".into());

    Ok(AppSettings {
        storage_mode: get_meta(&conn, "storage_mode")
            .map_err(map_err)?
            .unwrap_or_else(|| "local".into()),
        server_url: get_meta(&conn, "server_url").map_err(map_err)?,
        workspace_id: workspace_id.clone(),
        workspace_name,
        organization_id,
        organization_name,
        ai_api_base_url: get_meta(&conn, "ai_api_base_url").map_err(map_err)?,
        ai_model: get_meta(&conn, "ai_model").map_err(map_err)?,
        ai_api_key_configured: any_api_key_configured(&conn).map_err(map_err)?,
        seeded: get_meta(&conn, "seeded").map_err(map_err)?.as_deref() == Some("true"),
        current_member_id: get_meta(&conn, "current_member_id").map_err(map_err)?,
        server_status: get_meta(&conn, "server_status")
            .map_err(map_err)?
            .unwrap_or_else(|| "disconnected".into()),
        last_sync: get_meta(&conn, "last_sync").map_err(map_err)?,
        queued_changes: conn
            .query_row(
                "SELECT COUNT(*) FROM outbox WHERE workspace_id = ?1 AND status = 'queued'",
                rusqlite::params![workspace_id],
                |row| row.get(0),
            )
            .unwrap_or(0),
        setup_complete: crate::db::setup::is_setup_complete(&conn).map_err(map_err)?,
    })
}

#[tauri::command]
pub fn update_storage_mode_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    mode: String,
) -> Result<AppSettings, String> {
    crate::commands::roles::require_perm(&state, "settings.data_server")?;
    let parsed = StorageMode::parse(&mode);
    {
        let conn = state.conn.lock();
        set_meta(&conn, "storage_mode", parsed.as_str()).map_err(map_err)?;
        if matches!(parsed, StorageMode::Local) {
            set_meta(&conn, "server_status", "disconnected").map_err(map_err)?;
        }
    }
    get_settings_cmd(state)
}

#[tauri::command]
pub fn update_ai_settings_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: UpdateAiSettingsInput,
) -> Result<AppSettings, String> {
    crate::commands::roles::require_perm(&state, "settings.ai")?;
    {
        let conn = state.conn.lock();
        if let Some(url) = input.ai_api_base_url {
            set_meta(&conn, "ai_api_base_url", &url).map_err(map_err)?;
        }
        if let Some(model) = input.ai_model {
            set_meta(&conn, "ai_model", &model).map_err(map_err)?;
        }
        if input.clear_ai_api_key == Some(true) {
            set_meta(&conn, "ai_api_key", "").map_err(map_err)?;
        } else if let Some(key) = input.ai_api_key {
            if !key.trim().is_empty() {
                set_meta(&conn, "ai_api_key", &key).map_err(map_err)?;
            }
        }
    }
    get_settings_cmd(state)
}

#[tauri::command]
pub fn list_api_connections_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<ApiConnection>, String> {
    crate::commands::roles::require_perm(&state, "settings.ai")?;
    let conn = state.conn.lock();
    let stored = load_api_connections(&conn).map_err(map_err)?;
    Ok(stored.iter().map(to_public_api).collect())
}

#[tauri::command]
pub fn save_api_connections_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    connections: Vec<SaveApiConnectionInput>,
) -> Result<Vec<ApiConnection>, String> {
    crate::commands::roles::require_perm(&state, "settings.ai")?;
    let mut next = Vec::with_capacity(connections.len());
    {
        let conn = state.conn.lock();
        let existing = load_api_connections(&conn).map_err(map_err)?;
        for input in connections {
            let prev = existing.iter().find(|e| e.id == input.id);
            let api_key = if input.clear_api_key == Some(true) {
                None
            } else if let Some(key) = input.api_key.filter(|k| !k.trim().is_empty()) {
                Some(key)
            } else {
                prev.and_then(|p| p.api_key.clone())
            };
            next.push(StoredApiConnection {
                id: input.id,
                name: input.name.trim().to_string(),
                base_url: input.base_url.trim().to_string(),
                model: input.model.trim().to_string(),
                api_key,
                enabled: input.enabled,
                is_default: input.is_default,
            });
        }
        normalize_api_defaults(&mut next);
        write_api_connections(&conn, &next).map_err(map_err)?;
    }
    Ok(next.iter().map(to_public_api).collect())
}

#[tauri::command]
pub fn list_mcp_connections_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<McpConnection>, String> {
    crate::commands::roles::require_perm(&state, "settings.ai")?;
    let conn = state.conn.lock();
    let stored = read_stored_mcp(&conn)?;
    Ok(stored.iter().map(to_public_mcp).collect())
}

#[tauri::command]
pub fn save_mcp_connections_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    connections: Vec<SaveMcpConnectionInput>,
) -> Result<Vec<McpConnection>, String> {
    crate::commands::roles::require_perm(&state, "settings.ai")?;
    let mut next = Vec::with_capacity(connections.len());
    {
        let conn = state.conn.lock();
        let existing = read_stored_mcp(&conn)?;
        for input in connections {
            let transport = if input.transport == "sse" { "sse" } else { "stdio" };
            let prev = existing.iter().find(|e| e.id == input.id);
            let auth_token = match input.auth_token {
                Some(token) if !token.trim().is_empty() => Some(token),
                _ => prev.and_then(|p| p.auth_token.clone()),
            };
            next.push(StoredMcpConnection {
                id: input.id,
                name: input.name.trim().to_string(),
                transport: transport.into(),
                command: input
                    .command
                    .map(|c| c.trim().to_string())
                    .filter(|c| !c.is_empty()),
                args: input
                    .args
                    .into_iter()
                    .map(|a| a.trim().to_string())
                    .filter(|a| !a.is_empty())
                    .collect(),
                url: input
                    .url
                    .map(|u| u.trim().to_string())
                    .filter(|u| !u.is_empty()),
                auth_token,
                enabled: input.enabled,
            });
        }
        write_stored_mcp(&conn, &next)?;
    }
    Ok(next.iter().map(to_public_mcp).collect())
}

#[tauri::command]
pub fn ensure_seeded_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<Workspace, String> {
    let conn = state.conn.lock();
    ensure_seeded(&conn).map_err(map_err)
}

#[tauri::command]
pub fn get_data_dir_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<String, String> {
    Ok(state.paths.root.display().to_string())
}
