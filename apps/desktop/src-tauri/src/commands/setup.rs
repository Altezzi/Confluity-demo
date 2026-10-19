use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::{
    get_meta,
    identity::{
        update_cloud_identity_settings,
        UpdateCloudIdentitySettings, PROVIDER_SUPABASE,
    },
    roles::{ensure_default_roles, find_role_by_name, list_roles, set_member_roles, Role},
    setup::{clear_demo_workspace_data, demo_data_detected, is_setup_complete, set_setup_complete},
    set_meta, DbError,
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

#[derive(Debug, Serialize)]
pub struct SetupStatus {
    pub setup_complete: bool,
    pub demo_data_present: bool,
    pub organization_name: String,
    pub workspace_name: String,
    pub server_provider: String,
    pub admin_configured: bool,
}

#[tauri::command]
pub fn get_setup_status_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<SetupStatus, String> {
    let conn = state.conn.lock();
    let workspace_id = get_meta(&conn, "workspace_id")
        .map_err(map_err)?
        .unwrap_or_default();
    let organization_id = get_meta(&conn, "organization_id")
        .map_err(map_err)?
        .unwrap_or_default();

    let organization_name = conn
        .query_row(
            "SELECT name FROM organizations WHERE id = ?1",
            rusqlite::params![organization_id],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "Your Organization".into());

    let workspace_name = conn
        .query_row(
            "SELECT name FROM workspaces WHERE id = ?1",
            rusqlite::params![workspace_id],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "Main Workspace".into());

    let provider = get_meta(&conn, "identity_provider")
        .map_err(map_err)?
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| "none".into());

    let admin_configured = get_meta(&conn, "setup_admin_id")
        .map_err(map_err)?
        .filter(|id| !id.is_empty())
        .is_some_and(|id| {
            conn.query_row(
                "SELECT 1 FROM members WHERE id = ?1 AND deleted_at IS NULL",
                rusqlite::params![id],
                |_| Ok(true),
            )
            .unwrap_or(false)
        });

    Ok(SetupStatus {
        setup_complete: is_setup_complete(&conn).map_err(map_err)?,
        demo_data_present: demo_data_detected(&conn, &workspace_id).map_err(map_err)?,
        organization_name,
        workspace_name,
        server_provider: provider,
        admin_configured,
    })
}

#[tauri::command]
pub fn clear_demo_for_setup_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<SetupStatus, String> {
    let ws = workspace_id(&state)?;
    {
        let conn = state.conn.lock();
        clear_demo_workspace_data(&conn, &ws).map_err(map_err)?;
        set_meta(&conn, "demo_mode", "false").map_err(map_err)?;
    }
    get_setup_status_cmd(state)
}

#[derive(Debug, Deserialize)]
pub struct SaveSetupWorkspaceInput {
    pub organization_name: String,
    pub workspace_name: String,
}

#[tauri::command]
pub fn save_setup_workspace_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: SaveSetupWorkspaceInput,
) -> Result<SetupStatus, String> {
    let ws = workspace_id(&state)?;
    let org = {
        let conn = state.conn.lock();
        get_meta(&conn, "organization_id").map_err(map_err)?
    }
    .ok_or_else(|| "organization missing".to_string())?;

    let org_name = input.organization_name.trim();
    let ws_name = input.workspace_name.trim();
    if org_name.is_empty() || ws_name.is_empty() {
        return Err("Organization and workspace names are required.".into());
    }

    let ts = chrono::Utc::now().to_rfc3339();
    {
        let conn = state.conn.lock();
        conn.execute(
            "UPDATE organizations SET name = ?1, updated_at = ?2, version = version + 1 WHERE id = ?3",
            rusqlite::params![org_name, ts, org],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE workspaces SET name = ?1, updated_at = ?2, version = version + 1 WHERE id = ?3",
            rusqlite::params![ws_name, ts, ws],
        )
        .map_err(|e| e.to_string())?;
    }
    get_setup_status_cmd(state)
}

#[derive(Debug, Deserialize)]
pub struct SaveSetupServerInput {
    pub provider: String,
    pub supabase_url: Option<String>,
    pub supabase_anon_key: Option<String>,
    pub supabase_db_password: Option<String>,
    pub custom_api_url: Option<String>,
}

#[tauri::command]
pub async fn save_setup_server_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: SaveSetupServerInput,
) -> Result<SetupStatus, String> {
    let provider = if input.provider == "local" {
        "none".to_string()
    } else {
        input.provider.clone()
    };
    if provider == PROVIDER_SUPABASE {
        let url = input.supabase_url.as_deref().unwrap_or("").trim();
        let key = input.supabase_anon_key.as_deref().unwrap_or("").trim();
        if url.is_empty() || key.is_empty() {
            return Err("Project URL and anon key are required.".into());
        }
    }
    {
        let conn = state.conn.lock();
        update_cloud_identity_settings(
            &conn,
            &UpdateCloudIdentitySettings {
                provider: provider.clone(),
                supabase_url: input.supabase_url.clone(),
                supabase_anon_key: input.supabase_anon_key.clone(),
                clear_supabase_anon_key: None,
                supabase_service_role: None,
                clear_supabase_service_role: None,
                custom_api_url: input.custom_api_url.clone(),
                sync_members_enabled: Some(provider != "none"),
                members_table: Some(
                    if provider == PROVIDER_SUPABASE {
                        "confluity_accounts".into()
                    } else {
                        "workspace_members".into()
                    },
                ),
                sync_data_enabled: Some(provider == PROVIDER_SUPABASE),
            },
        )
        .map_err(map_err)?;

        if input.provider == "local" || input.provider == "none" {
            set_meta(&conn, "storage_mode", "local").map_err(map_err)?;
            set_meta(&conn, "server_status", "disconnected").map_err(map_err)?;
            set_meta(&conn, "server_url", "").map_err(map_err)?;
        } else if provider == PROVIDER_SUPABASE {
            let url = input
                .supabase_url
                .as_deref()
                .unwrap_or("")
                .trim()
                .trim_end_matches('/')
                .to_string();
            set_meta(&conn, "server_url", &url).map_err(map_err)?;
            set_meta(&conn, "server_status", "connected").map_err(map_err)?;
            set_meta(&conn, "storage_mode", "hybrid").map_err(map_err)?;
            if let Some(password) = input
                .supabase_db_password
                .as_deref()
                .map(str::trim)
                .filter(|p| !p.is_empty())
            {
                set_meta(&conn, "identity_supabase_db_password", password).map_err(map_err)?;
            }
        } else if let Some(url) = input.custom_api_url.as_deref().map(str::trim).filter(|u| !u.is_empty())
        {
            set_meta(&conn, "server_url", url).map_err(map_err)?;
            set_meta(&conn, "server_status", "connected").map_err(map_err)?;
            set_meta(&conn, "storage_mode", "hybrid").map_err(map_err)?;
        }
    }
    if input.provider == PROVIDER_SUPABASE {
        crate::commands::cloud_sync::ensure_schema_present(&state).await?;
    }
    get_setup_status_cmd(state)
}

#[derive(Debug, Deserialize)]
pub struct SaveSetupAdminInput {
    pub name: String,
    pub email: String,
    pub password: Option<String>,
}

#[tauri::command]
pub async fn save_setup_admin_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: SaveSetupAdminInput,
) -> Result<SetupStatus, String> {
    let ws = workspace_id(&state)?;
    let name = input.name.trim();
    let email = input.email.trim();
    if name.is_empty() || email.is_empty() {
        return Err("Name and email are required.".into());
    }

    let provider = {
        let conn = state.conn.lock();
        get_meta(&conn, "identity_provider").map_err(map_err)?
    }
    .unwrap_or_else(|| "none".into());

    let mut remote_user_id: Option<String> = None;
    let local_password = if provider == PROVIDER_SUPABASE {
        None
    } else {
        Some(
            input
                .password
                .as_deref()
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .ok_or_else(|| "Password is required.".to_string())?
                .to_string(),
        )
    };

    if provider == PROVIDER_SUPABASE {
        let password = input
            .password
            .filter(|p| !p.is_empty())
            .ok_or_else(|| "Password is required for Supabase sign-in.".to_string())?;
        remote_user_id = Some(
            crate::commands::identity::ensure_supabase_account(&state, email, &password, name)
                .await?,
        );
    }

    let member_id = remote_user_id.unwrap_or_else(crate::db::new_id);
    let ts = chrono::Utc::now().to_rfc3339();
    {
        let conn = state.conn.lock();
        let admin_role = find_role_by_name(&conn, &ws, "Admin").map_err(map_err)?;
        let Some(admin_role) = admin_role else {
            return Err("Admin role missing — complete workspace bootstrap first.".into());
        };

        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM members WHERE id = ?1",
                rusqlite::params![member_id],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if exists {
            conn.execute(
                "UPDATE members SET workspace_id = ?2, name = ?3, role = 'admin', title = 'Administrator', updated_at = ?4, deleted_at = NULL, version = version + 1 WHERE id = ?1",
                rusqlite::params![member_id, ws, name, ts],
            )
            .map_err(|e| e.to_string())?;
        } else {
            conn.execute(
                "INSERT INTO members (id, workspace_id, name, role, title, created_at, updated_at, deleted_at, version)
                 VALUES (?1, ?2, ?3, 'admin', 'Administrator', ?4, ?4, NULL, 1)",
                rusqlite::params![member_id, ws, name, ts],
            )
            .map_err(|e| e.to_string())?;
        }

        set_member_roles(&conn, &member_id, &[admin_role.id]).map_err(map_err)?;
        set_meta(&conn, "setup_admin_id", &member_id).map_err(map_err)?;
        set_meta(&conn, "current_member_id", &member_id).map_err(map_err)?;
        set_meta(&conn, "setup_admin_email", email).map_err(map_err)?;
        if let Some(password) = &local_password {
            set_meta(&conn, "setup_admin_password", password).map_err(map_err)?;
        }
    }

    if provider == PROVIDER_SUPABASE {
        crate::commands::cloud_sync::push_workspace_records(&state).await?;
    }

    get_setup_status_cmd(state)
}

#[derive(Debug, Deserialize)]
pub struct CompleteSetupRolesInput {
    pub use_defaults: bool,
}

#[tauri::command]
pub fn complete_setup_roles_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CompleteSetupRolesInput,
) -> Result<Vec<Role>, String> {
    let ws = workspace_id(&state)?;
    {
        let conn = state.conn.lock();
        if input.use_defaults {
            ensure_default_roles(&conn, &ws).map_err(map_err)?;
        }
    }
    let conn = state.conn.lock();
    list_roles(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub async fn complete_setup_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<SetupStatus, String> {
    let ws = workspace_id(&state)?;
    {
        let conn = state.conn.lock();
        let admin_ok = get_meta(&conn, "setup_admin_id")
            .map_err(map_err)?
            .filter(|id| !id.is_empty())
            .is_some_and(|id| {
                conn.query_row(
                    "SELECT 1 FROM members WHERE id = ?1 AND deleted_at IS NULL",
                    rusqlite::params![id],
                    |_| Ok(true),
                )
                .unwrap_or(false)
            });
        if !admin_ok {
            return Err("Create your administrator login before finishing setup.".into());
        }
    }
    crate::commands::cloud_sync::push_workspace_records(&state).await?;
    {
        let conn = state.conn.lock();
        set_setup_complete(&conn, true).map_err(map_err)?;
        set_meta(&conn, "demo_mode", "false").map_err(map_err)?;
        crate::db::positions::ensure_setup_admin(&conn, &ws).map_err(map_err)?;
    }
    get_setup_status_cmd(state)
}
