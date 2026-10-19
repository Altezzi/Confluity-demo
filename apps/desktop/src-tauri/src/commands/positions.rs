use serde::Deserialize;
use std::sync::Arc;

use crate::db::{
    get_meta,
    models::list_members,
    positions::{
        create_category, create_position, delete_category, delete_position, get_position,
        list_categories, list_positions, set_member_position, set_position_role_permissions,
        set_position_roles, update_category, update_position, OrgCategory, Position,
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

#[derive(Debug, Deserialize)]
pub struct CreateCategoryInput {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryInput {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePositionInput {
    pub category_id: String,
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePositionInput {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetPositionRolesInput {
    pub position_id: String,
    pub role_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetPositionRolePermissionsInput {
    pub position_id: String,
    pub role_id: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetMemberPositionInput {
    pub member_id: String,
    pub position_id: Option<String>,
}

#[tauri::command]
pub fn list_org_categories_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<OrgCategory>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_categories(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn create_org_category_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateCategoryInput,
) -> Result<OrgCategory, String> {
    crate::commands::roles::require_perm(&state, "settings.manage_roles")?;
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    create_category(
        &conn,
        &ws,
        &input.name,
        input.color.as_deref().unwrap_or("#8ab4f8"),
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn update_org_category_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: UpdateCategoryInput,
) -> Result<OrgCategory, String> {
    crate::commands::roles::require_perm(&state, "settings.manage_roles")?;
    let conn = state.conn.lock();
    update_category(&conn, &input.id, &input.name, &input.color).map_err(map_err)
}

#[tauri::command]
pub fn delete_org_category_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    crate::commands::roles::require_perm(&state, "settings.manage_roles")?;
    let conn = state.conn.lock();
    delete_category(&conn, &id).map_err(map_err)
}

#[tauri::command]
pub fn list_positions_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<Position>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_positions(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn create_position_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreatePositionInput,
) -> Result<Position, String> {
    crate::commands::roles::require_perm(&state, "settings.manage_roles")?;
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    create_position(
        &conn,
        &ws,
        &input.category_id,
        &input.name,
        input.parent_id.as_deref(),
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn update_position_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: UpdatePositionInput,
) -> Result<Position, String> {
    crate::commands::roles::require_perm(&state, "settings.manage_roles")?;
    let conn = state.conn.lock();
    update_position(&conn, &input.id, &input.name, input.parent_id.as_deref()).map_err(map_err)
}

#[tauri::command]
pub fn delete_position_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    crate::commands::roles::require_perm(&state, "settings.manage_roles")?;
    let conn = state.conn.lock();
    delete_position(&conn, &id).map_err(map_err)
}

#[tauri::command]
pub fn set_position_roles_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: SetPositionRolesInput,
) -> Result<Position, String> {
    crate::commands::roles::require_perm(&state, "settings.manage_roles")?;
    let conn = state.conn.lock();
    set_position_roles(&conn, &input.position_id, &input.role_ids).map_err(map_err)?;
    get_position(&conn, &input.position_id)
        .map_err(map_err)?
        .ok_or_else(|| "position not found".into())
}

#[tauri::command]
pub fn set_position_role_permissions_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: SetPositionRolePermissionsInput,
) -> Result<Position, String> {
    crate::commands::roles::require_perm(&state, "settings.manage_roles")?;
    let conn = state.conn.lock();
    set_position_role_permissions(
        &conn,
        &input.position_id,
        &input.role_id,
        &input.permissions,
    )
    .map_err(map_err)?;
    get_position(&conn, &input.position_id)
        .map_err(map_err)?
        .ok_or_else(|| "position not found".into())
}

#[tauri::command]
pub fn set_member_position_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: SetMemberPositionInput,
) -> Result<crate::db::models::Member, String> {
    crate::commands::roles::require_perm(&state, "settings.manage_members")?;
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    set_member_position(&conn, &input.member_id, input.position_id.as_deref()).map_err(map_err)?;
    list_members(&conn, &ws)
        .map_err(map_err)?
        .into_iter()
        .find(|m| m.id == input.member_id)
        .ok_or_else(|| "member not found".into())
}
