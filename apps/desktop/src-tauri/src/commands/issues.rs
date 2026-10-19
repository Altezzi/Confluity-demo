use serde::Deserialize;
use std::sync::Arc;

use crate::db::{
    models::{get_issue, list_issues, Issue},
    new_id, now_iso, record_change, upsert_fts, DbError,
};
use crate::state::AppState;

fn map_err(e: DbError) -> String {
    e.to_string()
}

fn workspace_id(state: &AppState) -> Result<String, String> {
    let conn = state.conn.lock();
    crate::db::get_meta(&conn, "workspace_id")
        .map_err(map_err)?
        .ok_or_else(|| "workspace not initialized".into())
}

#[derive(Debug, Deserialize)]
pub struct CreateIssueInput {
    pub title: String,
    pub description: Option<String>,
    pub ticket_id: Option<String>,
    pub company_id: Option<String>,
    pub project_id: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub blocked_reason: Option<String>,
    pub assignee_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIssueInput {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub ticket_id: Option<String>,
    pub company_id: Option<String>,
    pub project_id: Option<String>,
    pub status: String,
    pub priority: String,
    pub blocked_reason: Option<String>,
    pub assignee_id: Option<String>,
}

#[tauri::command]
pub fn list_issues_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    project_id: Option<String>,
) -> Result<Vec<Issue>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_issues(&conn, &ws, project_id.as_deref()).map_err(map_err)
}

#[tauri::command]
pub fn get_issue_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<Option<Issue>, String> {
    let conn = state.conn.lock();
    get_issue(&conn, &id).map_err(map_err)
}

#[tauri::command]
pub fn create_issue_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateIssueInput,
) -> Result<Issue, String> {
    crate::commands::roles::require_perm(&state, "module.projects.edit")?;
    let ws = workspace_id(&state)?;
    let id = new_id();
    let ts = now_iso();
    let status = input.status.unwrap_or_else(|| "backlog".into());
    let priority = input.priority.unwrap_or_else(|| "medium".into());
    let conn = state.conn.lock();
    conn.execute(
        "INSERT INTO issues (id, workspace_id, ticket_id, company_id, project_id, title, description, status, priority, blocked_reason, assignee_id, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12, NULL, 1)",
        rusqlite::params![
            id,
            ws,
            input.ticket_id,
            input.company_id,
            input.project_id,
            input.title,
            input.description,
            status,
            priority,
            input.blocked_reason,
            input.assignee_id,
            ts
        ],
    )
    .map_err(|e| e.to_string())?;
    let body = format!(
        "{} {}",
        input.description.clone().unwrap_or_default(),
        input.blocked_reason.clone().unwrap_or_default()
    );
    upsert_fts(&conn, "issue", &id, &input.title, &body).map_err(map_err)?;
    record_change(
        &conn,
        &ws,
        "create",
        "issue",
        &id,
        &format!("Created issue {}", input.title),
    )
    .map_err(map_err)?;
    get_issue(&conn, &id)
        .map_err(map_err)?
        .ok_or_else(|| "failed to load issue".into())
}

#[tauri::command]
pub fn update_issue_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: UpdateIssueInput,
) -> Result<Issue, String> {
    crate::commands::roles::require_perm(&state, "module.projects.edit")?;
    let ws = workspace_id(&state)?;
    let ts = now_iso();
    let conn = state.conn.lock();
    conn.execute(
        "UPDATE issues SET ticket_id = ?1, company_id = ?2, project_id = ?3, title = ?4, description = ?5, status = ?6, priority = ?7, blocked_reason = ?8, assignee_id = ?9, updated_at = ?10, version = version + 1
         WHERE id = ?11 AND deleted_at IS NULL",
        rusqlite::params![
            input.ticket_id,
            input.company_id,
            input.project_id,
            input.title,
            input.description,
            input.status,
            input.priority,
            input.blocked_reason,
            input.assignee_id,
            ts,
            input.id
        ],
    )
    .map_err(|e| e.to_string())?;
    let body = format!(
        "{} {}",
        input.description.clone().unwrap_or_default(),
        input.blocked_reason.clone().unwrap_or_default()
    );
    upsert_fts(&conn, "issue", &input.id, &input.title, &body).map_err(map_err)?;
    record_change(
        &conn,
        &ws,
        "update",
        "issue",
        &input.id,
        &format!("Updated issue {}", input.title),
    )
    .map_err(map_err)?;
    get_issue(&conn, &input.id)
        .map_err(map_err)?
        .ok_or_else(|| "issue not found".into())
}
