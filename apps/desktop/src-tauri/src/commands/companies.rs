use serde::Deserialize;
use std::sync::Arc;

use crate::db::{
    log_activity, models::{get_company, list_companies, Company}, new_id, now_iso, upsert_fts, DbError,
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
pub struct CreateCompanyInput {
    pub name: String,
    pub domain: Option<String>,
    pub industry: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCompanyInput {
    pub id: String,
    pub name: String,
    pub domain: Option<String>,
    pub industry: Option<String>,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn list_companies_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<Vec<Company>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_companies(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn get_company_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<Option<Company>, String> {
    let conn = state.conn.lock();
    get_company(&conn, &id).map_err(map_err)
}

#[tauri::command]
pub fn create_company_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateCompanyInput,
) -> Result<Company, String> {
    crate::commands::roles::require_perm(&state, "module.crm.edit")?;
    let ws = workspace_id(&state)?;
    let id = new_id();
    let ts = now_iso();
    let conn = state.conn.lock();
    conn.execute(
        "INSERT INTO companies (id, workspace_id, name, domain, industry, notes, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, NULL, 1)",
        rusqlite::params![id, ws, input.name, input.domain, input.industry, input.notes, ts],
    )
    .map_err(|e| e.to_string())?;
    let body = format!(
        "{} {} {}",
        input.domain.clone().unwrap_or_default(),
        input.industry.clone().unwrap_or_default(),
        input.notes.clone().unwrap_or_default()
    );
    upsert_fts(&conn, "company", &id, &input.name, &body).map_err(map_err)?;
    log_activity(&conn, &ws, "create", "company", &id, &format!("Created company {}", input.name))
        .map_err(map_err)?;
    get_company(&conn, &id)
        .map_err(map_err)?
        .ok_or_else(|| "failed to load company".into())
}

#[tauri::command]
pub fn update_company_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: UpdateCompanyInput,
) -> Result<Company, String> {
    crate::commands::roles::require_perm(&state, "module.crm.edit")?;
    let ws = workspace_id(&state)?;
    let ts = now_iso();
    let conn = state.conn.lock();
    conn.execute(
        "UPDATE companies SET name = ?1, domain = ?2, industry = ?3, notes = ?4, updated_at = ?5, version = version + 1
         WHERE id = ?6 AND deleted_at IS NULL",
        rusqlite::params![input.name, input.domain, input.industry, input.notes, ts, input.id],
    )
    .map_err(|e| e.to_string())?;
    let body = format!(
        "{} {} {}",
        input.domain.clone().unwrap_or_default(),
        input.industry.clone().unwrap_or_default(),
        input.notes.clone().unwrap_or_default()
    );
    upsert_fts(&conn, "company", &input.id, &input.name, &body).map_err(map_err)?;
    log_activity(
        &conn,
        &ws,
        "update",
        "company",
        &input.id,
        &format!("Updated company {}", input.name),
    )
    .map_err(map_err)?;
    get_company(&conn, &input.id)
        .map_err(map_err)?
        .ok_or_else(|| "company not found".into())
}
