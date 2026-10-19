use serde::Deserialize;
use std::sync::Arc;

use crate::db::{
    get_meta,
    insights::{
        delete_live_query, get_live_query, list_insight_tables, list_live_queries,
        load_insight_table, run_live_sql, upsert_live_query, InsightTable, LiveQuery,
        LiveQueryResult,
    },
    DbError,
};
use crate::state::AppState;

fn map_err(e: DbError) -> String {
    e.to_string()
}

fn workspace_id(state: &AppState) -> Result<String, String> {
    let conn = state.conn.lock();
    get_meta(&conn, "workspace_id")
        .map_err(map_err)?
        .ok_or_else(|| "workspace not initialized".into())
}

fn current_member_id(state: &AppState) -> Result<Option<String>, String> {
    let conn = state.conn.lock();
    get_meta(&conn, "current_member_id").map_err(map_err)
}

#[derive(Debug, Deserialize)]
pub struct UpsertLiveQueryInput {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub sql_text: String,
}

#[tauri::command]
pub fn list_insight_tables_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<InsightTable>, String> {
    crate::commands::roles::require_perm(&state, "module.insights.view")?;
    let conn = state.conn.lock();
    list_insight_tables(&conn).map_err(map_err)
}

#[tauri::command]
pub fn load_insight_table_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    name: String,
) -> Result<LiveQueryResult, String> {
    crate::commands::roles::require_perm(&state, "module.insights.view")?;
    let conn = state.conn.lock();
    load_insight_table(&conn, &name).map_err(map_err)
}

#[tauri::command]
pub fn list_live_queries_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<LiveQuery>, String> {
    crate::commands::roles::require_perm(&state, "module.insights.view")?;
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    crate::db::insights::ensure_default_queries(&conn, &ws).map_err(map_err)?;
    list_live_queries(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn run_live_query_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    sql_text: String,
) -> Result<LiveQueryResult, String> {
    crate::commands::roles::require_perm(&state, "module.insights.view")?;
    let conn = state.conn.lock();
    run_live_sql(&conn, &sql_text).map_err(map_err)
}

#[tauri::command]
pub fn upsert_live_query_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: UpsertLiveQueryInput,
) -> Result<LiveQuery, String> {
    crate::commands::roles::require_perm(&state, "module.insights.edit")?;
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err("query name is required".into());
    }
    let ws = workspace_id(&state)?;
    let me = current_member_id(&state)?;
    let conn = state.conn.lock();
    upsert_live_query(
        &conn,
        &ws,
        input.id.as_deref(),
        &name,
        input.description.as_deref(),
        &input.sql_text,
        me.as_deref(),
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn delete_live_query_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    crate::commands::roles::require_perm(&state, "module.insights.edit")?;
    let conn = state.conn.lock();
    let _ = get_live_query(&conn, &id)
        .map_err(map_err)?
        .ok_or_else(|| "query not found".to_string())?;
    delete_live_query(&conn, &id).map_err(map_err)
}
