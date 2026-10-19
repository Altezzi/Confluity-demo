use std::sync::Arc;

use crate::db::{ticket_forms::*, DbError};
use crate::state::AppState;

fn map_err(error: DbError) -> String {
    let text = error.to_string();
    text.strip_prefix("sqlite: Invalid parameter name: ")
        .unwrap_or(&text)
        .to_string()
}

fn workspace_id(state: &AppState) -> Result<String, String> {
    let conn = state.conn.lock();
    crate::db::get_meta(&conn, "workspace_id")
        .map_err(map_err)?
        .ok_or_else(|| "workspace not initialized".into())
}

#[tauri::command]
pub fn list_ticket_forms_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<TicketForm>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_ticket_forms(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn list_ticket_fields_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<TicketField>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_ticket_fields(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn save_ticket_form_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: SaveTicketFormInput,
) -> Result<TicketForm, String> {
    crate::commands::roles::require_perm(&state, "settings.workspace_admin")?;
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    save_ticket_form(&conn, &ws, &input).map_err(map_err)
}

#[tauri::command]
pub fn create_ticket_field_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateTicketFieldInput,
) -> Result<TicketField, String> {
    crate::commands::roles::require_perm(&state, "settings.workspace_admin")?;
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    create_ticket_field(&conn, &ws, &input).map_err(map_err)
}

#[tauri::command]
pub fn delete_ticket_field_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    crate::commands::roles::require_perm(&state, "settings.workspace_admin")?;
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    delete_ticket_field(&conn, &ws, &id).map_err(map_err)
}

#[tauri::command]
pub fn delete_ticket_form_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<Vec<TicketForm>, String> {
    crate::commands::roles::require_perm(&state, "settings.workspace_admin")?;
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    delete_ticket_form(&conn, &ws, &id).map_err(map_err)?;
    list_ticket_forms(&conn, &ws).map_err(map_err)
}
