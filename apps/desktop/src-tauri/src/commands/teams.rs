use serde::Deserialize;
use std::sync::Arc;

use crate::db::{
    get_meta,
    teams::{
        create_team, create_team_channel, create_team_ex, ensure_category_teams, get_or_create_dm,
        get_team, list_dm_messages, list_dm_threads, list_team_channels, list_team_messages, list_teams,
        member_can_create_chats, send_dm_message, send_team_message, set_team_members, DmMessage,
        DmThread, Team, TeamChannel, TeamMessage,
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

fn current_member_id(state: &AppState) -> Result<String, String> {
    let conn = state.conn.lock();
    get_meta(&conn, "current_member_id")
        .map_err(map_err)?
        .ok_or_else(|| "no current member".into())
}

#[derive(Debug, Deserialize)]
pub struct CreateTeamInput {
    pub name: String,
    pub description: Option<String>,
    pub department: Option<String>,
    pub color: Option<String>,
    pub member_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct SetTeamMembersInput {
    pub team_id: String,
    pub member_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SendTeamMessageInput {
    pub team_id: String,
    pub channel_id: Option<String>,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTeamChannelInput {
    pub team_id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGroupChatInput {
    pub name: String,
    pub description: Option<String>,
    pub member_ids: Vec<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OpenDmInput {
    pub member_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SendDmInput {
    pub thread_id: String,
    pub body: String,
}

fn require_team_access(conn: &rusqlite::Connection, me: &str, team_id: &str) -> Result<Team, String> {
    let team = get_team(conn, team_id)
        .map_err(map_err)?
        .ok_or_else(|| "team not found".to_string())?;
    if !team.member_ids.iter().any(|id| id == me) {
        return Err("you don't have access to this team's chats".into());
    }
    Ok(team)
}

#[tauri::command]
pub fn list_teams_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<Vec<Team>, String> {
    crate::commands::roles::require_perm(&state, "module.teams.view")?;
    let ws = workspace_id(&state)?;
    let me = current_member_id(&state)?;
    let conn = state.conn.lock();
    ensure_category_teams(&conn, &ws).map_err(map_err)?;
    let teams = list_teams(&conn, &ws).map_err(map_err)?;
    Ok(teams
        .into_iter()
        .filter(|t| t.member_ids.iter().any(|id| id == &me))
        .collect())
}

#[tauri::command]
pub fn get_team_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<Option<Team>, String> {
    crate::commands::roles::require_perm(&state, "module.teams.view")?;
    let me = current_member_id(&state)?;
    let conn = state.conn.lock();
    match require_team_access(&conn, &me, &id) {
        Ok(team) => Ok(Some(team)),
        Err(_) => Ok(None),
    }
}

#[tauri::command]
pub fn create_team_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateTeamInput,
) -> Result<Team, String> {
    crate::commands::roles::require_perm(&state, "teams.manage")?;
    let ws = workspace_id(&state)?;
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err("team name is required".into());
    }
    let color = input.color.unwrap_or_else(|| "#6B7280".into());
    let conn = state.conn.lock();
    let team = create_team(
        &conn,
        &ws,
        &name,
        input.description.as_deref(),
        input.department.as_deref(),
        &color,
    )
    .map_err(map_err)?;
    if let Some(members) = input.member_ids {
        set_team_members(&conn, &team.id, &members).map_err(map_err)?;
    }
    get_team(&conn, &team.id)
        .map_err(map_err)?
        .ok_or_else(|| "failed to load team".into())
}

#[tauri::command]
pub fn set_team_members_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: SetTeamMembersInput,
) -> Result<Team, String> {
    crate::commands::roles::require_perm(&state, "teams.manage")?;
    let conn = state.conn.lock();
    set_team_members(&conn, &input.team_id, &input.member_ids).map_err(map_err)?;
    get_team(&conn, &input.team_id)
        .map_err(map_err)?
        .ok_or_else(|| "team not found".into())
}

fn require_create_chats(state: &AppState) -> Result<String, String> {
    let me = current_member_id(state)?;
    let conn = state.conn.lock();
    let allowed = member_can_create_chats(&conn, &me).map_err(map_err)?;
    if !allowed {
        return Err("only managers and supervisors can create chats".into());
    }
    Ok(me)
}

#[tauri::command]
pub fn list_team_channels_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    team_id: String,
) -> Result<Vec<TeamChannel>, String> {
    crate::commands::roles::require_perm(&state, "module.teams.view")?;
    let me = current_member_id(&state)?;
    let conn = state.conn.lock();
    require_team_access(&conn, &me, &team_id)?;
    list_team_channels(&conn, &team_id).map_err(map_err)
}

#[tauri::command]
pub fn create_team_channel_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateTeamChannelInput,
) -> Result<TeamChannel, String> {
    crate::commands::roles::require_perm(&state, "module.teams.view")?;
    let me = require_create_chats(&state)?;
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    let team = get_team(&conn, &input.team_id)
        .map_err(map_err)?
        .ok_or_else(|| "team not found".to_string())?;
    let is_admin = crate::db::roles::member_has_permission(&conn, &me, "teams.manage")
        .map_err(map_err)?
        || crate::db::roles::member_has_permission(&conn, &me, "settings.workspace_admin")
            .map_err(map_err)?;
    if !is_admin && !team.member_ids.iter().any(|id| id == &me) {
        return Err("join the team before creating a chat in it".into());
    }
    create_team_channel(
        &conn,
        &ws,
        &input.team_id,
        &input.name,
        input.description.as_deref(),
        Some(&me),
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn create_group_chat_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateGroupChatInput,
) -> Result<Team, String> {
    crate::commands::roles::require_perm(&state, "module.teams.view")?;
    let me = require_create_chats(&state)?;
    let ws = workspace_id(&state)?;
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err("chat name is required".into());
    }
    let color = input.color.unwrap_or_else(|| "#8ab4f8".into());
    let conn = state.conn.lock();
    let team = create_team_ex(
        &conn,
        &ws,
        &name,
        input.description.as_deref(),
        None,
        &color,
        "chat",
        Some(&me),
        None,
    )
    .map_err(map_err)?;
    let mut members = input.member_ids;
    if !members.iter().any(|id| id == &me) {
        members.push(me);
    }
    set_team_members(&conn, &team.id, &members).map_err(map_err)?;
    get_team(&conn, &team.id)
        .map_err(map_err)?
        .ok_or_else(|| "failed to load chat".into())
}

#[tauri::command]
pub fn list_team_messages_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    team_id: String,
    channel_id: Option<String>,
) -> Result<Vec<TeamMessage>, String> {
    crate::commands::roles::require_perm(&state, "module.teams.view")?;
    let me = current_member_id(&state)?;
    let conn = state.conn.lock();
    require_team_access(&conn, &me, &team_id)?;
    list_team_messages(&conn, &team_id, channel_id.as_deref()).map_err(map_err)
}

#[tauri::command]
pub fn send_team_message_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: SendTeamMessageInput,
) -> Result<TeamMessage, String> {
    crate::commands::roles::require_perm(&state, "teams.message")?;
    let ws = workspace_id(&state)?;
    let me = current_member_id(&state)?;
    let body = input.body.trim().to_string();
    if body.is_empty() {
        return Err("message body is required".into());
    }
    let conn = state.conn.lock();
    require_team_access(&conn, &me, &input.team_id)?;
    send_team_message(
        &conn,
        &ws,
        &input.team_id,
        input.channel_id.as_deref(),
        &me,
        &body,
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn can_create_team_chats_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    let me = current_member_id(&state)?;
    let conn = state.conn.lock();
    member_can_create_chats(&conn, &me).map_err(map_err)
}

#[tauri::command]
pub fn list_dm_threads_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<DmThread>, String> {
    crate::commands::roles::require_perm(&state, "module.teams.view")?;
    let ws = workspace_id(&state)?;
    let me = current_member_id(&state)?;
    let conn = state.conn.lock();
    list_dm_threads(&conn, &ws, &me).map_err(map_err)
}

#[tauri::command]
pub fn open_dm_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: OpenDmInput,
) -> Result<DmThread, String> {
    crate::commands::roles::require_perm(&state, "teams.message")?;
    let ws = workspace_id(&state)?;
    let me = current_member_id(&state)?;
    let conn = state.conn.lock();
    get_or_create_dm(&conn, &ws, &me, &input.member_id).map_err(map_err)
}

#[tauri::command]
pub fn list_dm_messages_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    thread_id: String,
) -> Result<Vec<DmMessage>, String> {
    crate::commands::roles::require_perm(&state, "module.teams.view")?;
    let conn = state.conn.lock();
    list_dm_messages(&conn, &thread_id).map_err(map_err)
}

#[tauri::command]
pub fn send_dm_message_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: SendDmInput,
) -> Result<DmMessage, String> {
    crate::commands::roles::require_perm(&state, "teams.message")?;
    let ws = workspace_id(&state)?;
    let me = current_member_id(&state)?;
    let body = input.body.trim().to_string();
    if body.is_empty() {
        return Err("message body is required".into());
    }
    let conn = state.conn.lock();
    send_dm_message(&conn, &ws, &input.thread_id, &me, &body).map_err(map_err)
}
