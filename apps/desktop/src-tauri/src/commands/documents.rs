use serde::Deserialize;
use std::sync::Arc;

use crate::db::{
    log_activity,
    models::{get_document, list_documents, Document},
    new_id, now_iso, upsert_fts, DbError,
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
pub struct CreateDocumentInput {
    pub title: String,
    pub body: String,
    pub category: Option<String>,
    pub company_id: Option<String>,
    pub ticket_id: Option<String>,
    pub issue_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDocumentInput {
    pub id: String,
    pub title: String,
    pub body: String,
    pub category: Option<String>,
    pub company_id: Option<String>,
    pub ticket_id: Option<String>,
    pub issue_id: Option<String>,
}

#[tauri::command]
pub fn list_documents_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<Document>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_documents(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn get_document_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<Option<Document>, String> {
    let conn = state.conn.lock();
    get_document(&conn, &id).map_err(map_err)
}

#[tauri::command]
pub fn create_document_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateDocumentInput,
) -> Result<Document, String> {
    crate::commands::roles::require_perm(&state, "module.docs.edit")?;
    let ws = workspace_id(&state)?;
    let id = new_id();
    let ts = now_iso();
    let conn = state.conn.lock();
    conn.execute(
        "INSERT INTO documents (id, workspace_id, title, body, category, company_id, ticket_id, issue_id, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9, NULL, 1)",
        rusqlite::params![
            id,
            ws,
            input.title,
            input.body,
            input.category,
            input.company_id,
            input.ticket_id,
            input.issue_id,
            ts
        ],
    )
    .map_err(|e| e.to_string())?;
    upsert_fts(&conn, "document", &id, &input.title, &input.body).map_err(map_err)?;
    log_activity(
        &conn,
        &ws,
        "create",
        "document",
        &id,
        &format!("Created document {}", input.title),
    )
    .map_err(map_err)?;
    get_document(&conn, &id)
        .map_err(map_err)?
        .ok_or_else(|| "failed to load document".into())
}

#[tauri::command]
pub fn update_document_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: UpdateDocumentInput,
) -> Result<Document, String> {
    crate::commands::roles::require_perm(&state, "module.docs.edit")?;
    let ws = workspace_id(&state)?;
    let ts = now_iso();
    let conn = state.conn.lock();
    conn.execute(
        "UPDATE documents SET title = ?1, body = ?2, category = ?3, company_id = ?4, ticket_id = ?5, issue_id = ?6, updated_at = ?7, version = version + 1
         WHERE id = ?8 AND deleted_at IS NULL",
        rusqlite::params![
            input.title,
            input.body,
            input.category,
            input.company_id,
            input.ticket_id,
            input.issue_id,
            ts,
            input.id
        ],
    )
    .map_err(|e| e.to_string())?;
    upsert_fts(&conn, "document", &input.id, &input.title, &input.body).map_err(map_err)?;
    log_activity(
        &conn,
        &ws,
        "update",
        "document",
        &input.id,
        &format!("Updated document {}", input.title),
    )
    .map_err(map_err)?;
    get_document(&conn, &input.id)
        .map_err(map_err)?
        .ok_or_else(|| "document not found".into())
}
