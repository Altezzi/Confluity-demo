use serde::Deserialize;
use std::sync::Arc;

use crate::db::{
    log_activity,
    models::{get_contact, list_contacts, Contact},
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
pub struct CreateContactInput {
    pub company_id: String,
    pub name: String,
    pub email: Option<String>,
    pub title: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContactInput {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub email: Option<String>,
    pub title: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn list_contacts_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    company_id: Option<String>,
) -> Result<Vec<Contact>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_contacts(&conn, &ws, company_id.as_deref()).map_err(map_err)
}

#[tauri::command]
pub fn get_contact_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<Option<Contact>, String> {
    let conn = state.conn.lock();
    get_contact(&conn, &id).map_err(map_err)
}

#[tauri::command]
pub fn create_contact_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateContactInput,
) -> Result<Contact, String> {
    crate::commands::roles::require_perm(&state, "module.crm.edit")?;
    let ws = workspace_id(&state)?;
    let id = new_id();
    let ts = now_iso();
    let conn = state.conn.lock();
    conn.execute(
        "INSERT INTO contacts (id, workspace_id, company_id, name, email, title, phone, notes, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9, NULL, 1)",
        rusqlite::params![
            id,
            ws,
            input.company_id,
            input.name,
            input.email,
            input.title,
            input.phone,
            input.notes,
            ts
        ],
    )
    .map_err(|e| e.to_string())?;
    let body = format!(
        "{} {} {}",
        input.email.clone().unwrap_or_default(),
        input.title.clone().unwrap_or_default(),
        input.notes.clone().unwrap_or_default()
    );
    upsert_fts(&conn, "contact", &id, &input.name, &body).map_err(map_err)?;
    log_activity(
        &conn,
        &ws,
        "create",
        "contact",
        &id,
        &format!("Created contact {}", input.name),
    )
    .map_err(map_err)?;
    get_contact(&conn, &id)
        .map_err(map_err)?
        .ok_or_else(|| "failed to load contact".into())
}

#[tauri::command]
pub fn update_contact_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: UpdateContactInput,
) -> Result<Contact, String> {
    crate::commands::roles::require_perm(&state, "module.crm.edit")?;
    let ws = workspace_id(&state)?;
    let ts = now_iso();
    let conn = state.conn.lock();
    conn.execute(
        "UPDATE contacts SET company_id = ?1, name = ?2, email = ?3, title = ?4, phone = ?5, notes = ?6, updated_at = ?7, version = version + 1
         WHERE id = ?8 AND deleted_at IS NULL",
        rusqlite::params![
            input.company_id,
            input.name,
            input.email,
            input.title,
            input.phone,
            input.notes,
            ts,
            input.id
        ],
    )
    .map_err(|e| e.to_string())?;
    let body = format!(
        "{} {} {}",
        input.email.clone().unwrap_or_default(),
        input.title.clone().unwrap_or_default(),
        input.notes.clone().unwrap_or_default()
    );
    upsert_fts(&conn, "contact", &input.id, &input.name, &body).map_err(map_err)?;
    log_activity(
        &conn,
        &ws,
        "update",
        "contact",
        &input.id,
        &format!("Updated contact {}", input.name),
    )
    .map_err(map_err)?;
    get_contact(&conn, &input.id)
        .map_err(map_err)?
        .ok_or_else(|| "contact not found".into())
}
