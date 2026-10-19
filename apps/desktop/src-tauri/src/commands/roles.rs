use serde::Deserialize;
use std::sync::Arc;

use crate::db::{
    get_meta,
    roles::{
        create_role, delete_role, effective_permissions_for_member, get_role, list_roles,
        member_has_permission, reset_default_roles, set_member_roles, set_role_permissions,
        swap_role_positions, update_role, Role,
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

/// Require the current member to hold a permission. If no member is set or no roles
/// exist yet, allow (bootstrap). Otherwise deny with a clear error.
pub fn require_perm(state: &AppState, key: &str) -> Result<(), String> {
    let conn = state.conn.lock();
    let ws = match get_meta(&conn, "workspace_id").map_err(map_err)? {
        Some(ws) => ws,
        None => return Ok(()),
    };
    if !crate::db::setup::is_setup_complete(&conn).unwrap_or(false) {
        return Ok(());
    }
    let role_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM roles WHERE workspace_id = ?1 AND deleted_at IS NULL",
            rusqlite::params![ws],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if role_count == 0 {
        return Ok(());
    }
    let Some(member_id) = get_meta(&conn, "current_member_id").map_err(map_err)? else {
        return Ok(());
    };
    if member_has_permission(&conn, &member_id, key).map_err(map_err)? {
        Ok(())
    } else {
        Err(format!("permission denied: {key}"))
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateRoleInput {
    pub name: String,
    pub color: Option<String>,
    pub permissions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoleInput {
    pub id: String,
    pub name: String,
    pub color: String,
    pub position: i64,
}

#[derive(Debug, Deserialize)]
pub struct SetRolePermissionsInput {
    pub role_id: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetMemberRolesInput {
    pub member_id: String,
    pub role_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderRolesInput {
    pub role_id: String,
    pub swap_with_id: String,
}

#[tauri::command]
pub fn list_roles_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<Vec<Role>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_roles(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn create_role_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateRoleInput,
) -> Result<Role, String> {
    require_perm(&state, "settings.manage_roles")?;
    let ws = workspace_id(&state)?;
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err("role name is required".into());
    }
    let color = input
        .color
        .unwrap_or_else(|| "#6B7280".into())
        .trim()
        .to_string();
    let perms = input.permissions.unwrap_or_default();
    let conn = state.conn.lock();
    create_role(&conn, &ws, &name, &color, &perms).map_err(map_err)
}

#[tauri::command]
pub fn update_role_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: UpdateRoleInput,
) -> Result<Role, String> {
    require_perm(&state, "settings.manage_roles")?;
    let conn = state.conn.lock();
    update_role(
        &conn,
        &input.id,
        input.name.trim(),
        input.color.trim(),
        input.position,
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn delete_role_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    require_perm(&state, "settings.manage_roles")?;
    let conn = state.conn.lock();
    let role = get_role(&conn, &id).map_err(map_err)?;
    if role.as_ref().map(|r| r.is_system).unwrap_or(false) {
        return Err("cannot delete system role".into());
    }
    delete_role(&conn, &id).map_err(map_err)
}

#[tauri::command]
pub fn reset_default_roles_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<Role>, String> {
    require_perm(&state, "settings.manage_roles")?;
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    reset_default_roles(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn set_role_permissions_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: SetRolePermissionsInput,
) -> Result<Role, String> {
    require_perm(&state, "settings.manage_roles")?;
    let conn = state.conn.lock();
    let role = get_role(&conn, &input.role_id)
        .map_err(map_err)?
        .ok_or_else(|| "role not found".to_string())?;
    if role.is_system {
        // Admin always keeps all keys; still allow syncing catalog keys.
    }
    set_role_permissions(&conn, &input.role_id, &input.permissions).map_err(map_err)?;
    let ts = crate::db::now_iso();
    conn.execute(
        "UPDATE roles SET updated_at = ?1, version = version + 1 WHERE id = ?2",
        rusqlite::params![ts, input.role_id],
    )
    .map_err(|e| e.to_string())?;
    get_role(&conn, &input.role_id)
        .map_err(map_err)?
        .ok_or_else(|| "role not found".into())
}

#[tauri::command]
pub fn set_member_roles_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: SetMemberRolesInput,
) -> Result<crate::db::models::Member, String> {
    require_perm(&state, "settings.manage_members")?;
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    set_member_roles(&conn, &input.member_id, &input.role_ids).map_err(map_err)?;
    crate::db::models::list_members(&conn, &ws)
        .map_err(map_err)?
        .into_iter()
        .find(|m| m.id == input.member_id)
        .ok_or_else(|| "member not found".into())
}

#[tauri::command]
pub fn reorder_roles_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: ReorderRolesInput,
) -> Result<Vec<Role>, String> {
    require_perm(&state, "settings.manage_roles")?;
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    swap_role_positions(&conn, &input.role_id, &input.swap_with_id).map_err(map_err)?;
    list_roles(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn get_my_permissions_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<String>, String> {
    let Some(member_id) = current_member_id(&state)? else {
        return Ok(crate::db::roles::all_permission_keys()
            .into_iter()
            .map(|s| s.to_string())
            .collect());
    };
    let conn = state.conn.lock();
    let perms = effective_permissions_for_member(&conn, &member_id).map_err(map_err)?;
    if perms.is_empty() {
        // Bootstrap: no roles assigned yet → full access until seeded
        let role_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM member_roles WHERE member_id = ?1",
                rusqlite::params![member_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if role_count == 0 {
            return Ok(crate::db::roles::all_permission_keys()
                .into_iter()
                .map(|s| s.to_string())
                .collect());
        }
    }
    Ok(perms)
}
