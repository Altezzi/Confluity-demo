use serde::Deserialize;
use std::sync::Arc;

use crate::db::{
    log_activity,
    models::{
        get_comment, get_project, list_comments, list_members, list_projects, Comment, Member,
        Project,
    },
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
pub struct CreateProjectInput {
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentInput {
    #[serde(alias = "entityType")]
    pub entity_type: String,
    #[serde(alias = "entityId")]
    pub entity_id: String,
    pub body: String,
    pub visibility: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListCommentsInput {
    #[serde(alias = "entityType")]
    pub entity_type: String,
    #[serde(alias = "entityId")]
    pub entity_id: String,
}

#[tauri::command]
pub fn list_members_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<Vec<Member>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_members(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn list_projects_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<Vec<Project>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_projects(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn create_project_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateProjectInput,
) -> Result<Project, String> {
    crate::commands::roles::require_perm(&state, "module.projects.edit")?;
    let ws = workspace_id(&state)?;
    let id = new_id();
    let ts = now_iso();
    let status = input.status.unwrap_or_else(|| "active".into());
    let conn = state.conn.lock();
    conn.execute(
        "INSERT INTO projects (id, workspace_id, name, description, status, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, NULL, 1)",
        rusqlite::params![id, ws, input.name, input.description, status, ts],
    )
    .map_err(|e| e.to_string())?;
    upsert_fts(
        &conn,
        "project",
        &id,
        &input.name,
        input.description.as_deref().unwrap_or(""),
    )
    .map_err(map_err)?;
    log_activity(
        &conn,
        &ws,
        "create",
        "project",
        &id,
        &format!("Created project {}", input.name),
    )
    .map_err(map_err)?;
    get_project(&conn, &id)
        .map_err(map_err)?
        .ok_or_else(|| "failed to load project".into())
}

#[tauri::command]
pub fn list_comments_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: Option<ListCommentsInput>,
    entity_type: Option<String>,
    entity_id: Option<String>,
) -> Result<Vec<Comment>, String> {
    let etype = input
        .as_ref()
        .map(|i| i.entity_type.clone())
        .or(entity_type)
        .ok_or_else(|| "entity_type is required".to_string())?;
    let eid = input
        .as_ref()
        .map(|i| i.entity_id.clone())
        .or(entity_id)
        .ok_or_else(|| "entity_id is required".to_string())?;
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_comments(&conn, &ws, &etype, &eid).map_err(map_err)
}

#[tauri::command]
pub fn create_comment_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateCommentInput,
) -> Result<Comment, String> {
    crate::commands::roles::require_perm(&state, "panel.comments")?;
    let ws = workspace_id(&state)?;
    let id = new_id();
    let ts = now_iso();
    let visibility = input.visibility.unwrap_or_else(|| "internal".into());
    let author = input.author.unwrap_or_else(|| "Jordan Hale".into());
    let conn = state.conn.lock();
    conn.execute(
        "INSERT INTO comments (id, workspace_id, entity_type, entity_id, author, body, visibility, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8, NULL, 1)",
        rusqlite::params![
            id,
            ws,
            input.entity_type,
            input.entity_id,
            author,
            input.body,
            visibility,
            ts
        ],
    )
    .map_err(|e| e.to_string())?;
    log_activity(
        &conn,
        &ws,
        "comment",
        &input.entity_type,
        &input.entity_id,
        &format!("Commented on {}", input.entity_type),
    )
    .map_err(map_err)?;
    if let Ok(members) = list_members(&conn, &ws) {
        for member in members {
            let needle = format!("@{}", member.name.replace(' ', ""));
            let alt = format!("@{}", member.name);
            if input.body.contains(&needle) || input.body.to_lowercase().contains(&alt.to_lowercase()) {
                let _ = crate::db::notify_workspace(
                    &conn,
                    &ws,
                    Some(&member.id),
                    &format!("{} mentioned you", author),
                    &input.body.chars().take(120).collect::<String>(),
                    Some(&input.entity_type),
                    Some(&input.entity_id),
                );
            }
        }
    }
    get_comment(&conn, &id)
        .map_err(map_err)?
        .ok_or_else(|| "failed to load comment".into())
}
