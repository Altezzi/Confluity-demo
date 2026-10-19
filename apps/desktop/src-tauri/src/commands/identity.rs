use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::{
    get_meta, identity::{
        get_cloud_identity_settings, get_identity_session_public, stored_session,
        update_cloud_identity_settings, write_session, CloudIdentitySettings,
        IdentitySessionPublic, IdentitySyncResult, StoredIdentitySession, UpdateCloudIdentitySettings,
        PROVIDER_CUSTOM, PROVIDER_SUPABASE,
    },
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

fn normalize_supabase_url(url: &str) -> String {
    url.trim().trim_end_matches('/').to_string()
}

#[derive(Debug, Deserialize)]
pub struct IdentitySignInInput {
    pub email: String,
    pub password: String,
}

#[tauri::command]
pub fn get_cloud_identity_settings_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<CloudIdentitySettings, String> {
    crate::commands::roles::require_perm(&state, "settings.data_server")?;
    let conn = state.conn.lock();
    get_cloud_identity_settings(&conn).map_err(map_err)
}

#[tauri::command]
pub fn get_identity_session_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<IdentitySessionPublic, String> {
    let conn = state.conn.lock();
    get_identity_session_public(&conn).map_err(map_err)
}

#[tauri::command]
pub fn update_cloud_identity_settings_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: UpdateCloudIdentitySettings,
) -> Result<CloudIdentitySettings, String> {
    crate::commands::roles::require_perm(&state, "settings.data_server")?;
    {
        let conn = state.conn.lock();
        update_cloud_identity_settings(&conn, &input).map_err(map_err)?;

        if input.provider == PROVIDER_SUPABASE {
            if let Some(url) = input.supabase_url.as_ref().filter(|u| !u.trim().is_empty()) {
                let url = normalize_supabase_url(url);
                set_meta(&conn, "server_url", &url).map_err(map_err)?;
                set_meta(&conn, "server_status", "connected").map_err(map_err)?;
                set_meta(&conn, "storage_mode", "hybrid").map_err(map_err)?;
            }
        } else if input.provider == PROVIDER_CUSTOM {
            if let Some(url) = input.custom_api_url.as_ref().filter(|u| !u.trim().is_empty()) {
                set_meta(&conn, "server_url", url.trim()).map_err(map_err)?;
                set_meta(&conn, "server_status", "connected").map_err(map_err)?;
                set_meta(&conn, "storage_mode", "hybrid").map_err(map_err)?;
            }
        }
    }
    get_cloud_identity_settings_cmd(state)
}

#[tauri::command]
pub async fn test_cloud_identity_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<String, String> {
    crate::commands::roles::require_perm(&state, "settings.data_server")?;
    let settings = {
        let conn = state.conn.lock();
        get_cloud_identity_settings(&conn).map_err(map_err)?
    };

    match settings.provider.as_str() {
        PROVIDER_SUPABASE => {
            let url = settings
                .supabase_url
                .filter(|u| !u.trim().is_empty())
                .ok_or_else(|| "Supabase project URL is required.".to_string())?;
            let anon = {
                let conn = state.conn.lock();
                crate::db::identity::supabase_anon_key(&conn).map_err(map_err)?
            }
            .ok_or_else(|| "Supabase anon key is required.".to_string())?;
            let base = normalize_supabase_url(&url);
            let health_url = format!("{base}/auth/v1/health");
            let client = reqwest::Client::new();
            let resp = client
                .get(health_url)
                .header("apikey", &anon)
                .send()
                .await
                .map_err(|e| format!("Could not reach Supabase: {e}"))?;
            if resp.status().is_success() {
                Ok("Supabase auth endpoint responded OK.".into())
            } else {
                Err(format!("Supabase health check failed ({})", resp.status()))
            }
        }
        PROVIDER_CUSTOM => {
            let url = settings
                .custom_api_url
                .filter(|u| !u.trim().is_empty())
                .ok_or_else(|| "Custom API base URL is required.".to_string())?;
            let client = reqwest::Client::new();
            let resp = client
                .get(url.trim().trim_end_matches('/'))
                .send()
                .await
                .map_err(|e| format!("Could not reach custom API: {e}"))?;
            Ok(format!(
                "Custom API reachable (HTTP {}).",
                resp.status().as_u16()
            ))
        }
        _ => Err("Choose Supabase or Custom API before testing.".into()),
    }
}

#[derive(Debug, Deserialize)]
struct SupabaseTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    user: SupabaseUser,
}

#[derive(Debug, Deserialize)]
struct SupabaseUser {
    id: String,
    email: Option<String>,
}

fn friendly_auth_error(action: &str, body: &str) -> String {
    let parsed: Option<serde_json::Value> = serde_json::from_str(body).ok();
    let msg = parsed
        .as_ref()
        .and_then(|v| {
            v.get("msg")
                .or_else(|| v.get("message"))
                .or_else(|| v.get("error_description"))
                .and_then(|m| m.as_str())
        })
        .unwrap_or("");
    let lower = msg.to_ascii_lowercase();
    if lower.contains("invalid login") || lower.contains("invalid_credentials") {
        return "Email or password was not accepted.".into();
    }
    if lower.contains("already registered") || lower.contains("already been registered") {
        return "An account with this email already exists.".into();
    }
    if lower.contains("password") && lower.contains("least") {
        return msg.to_string();
    }
    if msg.is_empty() {
        format!("{action} failed. Check the Supabase URL and anon key.")
    } else {
        format!("{action} failed: {msg}")
    }
}

fn store_supabase_session(
    state: &AppState,
    user_id: String,
    email: Option<String>,
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
) -> Result<StoredIdentitySession, String> {
    let expires_at = expires_in.map(|secs| {
        let ts = chrono::Utc::now() + chrono::Duration::seconds(secs);
        ts.to_rfc3339()
    });
    let session = StoredIdentitySession {
        provider: PROVIDER_SUPABASE.into(),
        user_id,
        email,
        access_token,
        refresh_token,
        expires_at,
    };
    let conn = state.conn.lock();
    write_session(&conn, Some(&session)).map_err(map_err)?;
    Ok(session)
}

async fn supabase_credentials(state: &AppState) -> Result<(String, String), String> {
    let settings = {
        let conn = state.conn.lock();
        get_cloud_identity_settings(&conn).map_err(map_err)?
    };
    if settings.provider != PROVIDER_SUPABASE {
        return Err("Save Supabase as the server source before creating the admin login.".into());
    }
    let url = settings
        .supabase_url
        .filter(|u| !u.trim().is_empty())
        .ok_or_else(|| "Supabase project URL is required.".to_string())?;
    let anon = {
        let conn = state.conn.lock();
        crate::db::identity::supabase_anon_key(&conn).map_err(map_err)?
    }
    .ok_or_else(|| "Supabase anon key is required.".to_string())?;
    Ok((normalize_supabase_url(&url), anon))
}

/// Create the Supabase user when they are new, otherwise sign in.
pub async fn ensure_supabase_account(
    state: &AppState,
    email: &str,
    password: &str,
    full_name: &str,
) -> Result<String, String> {
    if password.len() < 6 {
        return Err("Use a password of at least 6 characters.".into());
    }
    let (base, anon) = supabase_credentials(state).await?;
    let client = reqwest::Client::new();
    let signup = client
        .post(format!("{base}/auth/v1/signup"))
        .header("apikey", &anon)
        .header("Authorization", format!("Bearer {anon}"))
        .json(&serde_json::json!({
            "email": email.trim(),
            "password": password,
            "data": { "full_name": full_name }
        }))
        .send()
        .await
        .map_err(|e| format!("Could not reach Supabase: {e}"))?;

    let status = signup.status();
    let body = signup.text().await.unwrap_or_default();
    if status.as_u16() == 429 {
        return Err("Supabase is rate limiting signups. Wait a moment and try again.".into());
    }

    let already = {
        let lower = body.to_ascii_lowercase();
        status.as_u16() == 422
            || lower.contains("already registered")
            || lower.contains("user_already_exists")
    };
    if already {
        let session = sign_in_confirming(state, email, password).await?;
        return Ok(session.user_id);
    }
    if !status.is_success() {
        return Err(friendly_auth_error("Account setup", &body));
    }

    let parsed: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("Unexpected Supabase response: {e}"))?;
    let user_id = parsed
        .pointer("/user/id")
        .or_else(|| parsed.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let user_email = parsed
        .pointer("/user/email")
        .or_else(|| parsed.get("email"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let access_token = parsed
        .get("access_token")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if user_id.is_empty() {
        return Err("Supabase did not return an account id.".into());
    }
    let already_exists = parsed
        .pointer("/user/identities")
        .and_then(|v| v.as_array())
        .is_some_and(|list| list.is_empty());
    if already_exists || access_token.is_empty() {
        let session = sign_in_confirming(state, email, password).await?;
        return Ok(session.user_id);
    }

    let refresh = parsed
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let expires_in = parsed.get("expires_in").and_then(|v| v.as_i64());
    store_supabase_session(
        state,
        user_id.clone(),
        user_email,
        access_token,
        refresh,
        expires_in,
    )?;
    Ok(user_id)
}

async fn sign_in_confirming(
    state: &AppState,
    email: &str,
    password: &str,
) -> Result<StoredIdentitySession, String> {
    if let Ok(session) = perform_supabase_sign_in(state, email, password).await {
        return Ok(session);
    }
    crate::commands::cloud_sync::confirm_auth_email(state, email).await?;
    perform_supabase_sign_in(state, email, password).await
}

pub async fn perform_supabase_sign_in(
    state: &AppState,
    email: &str,
    password: &str,
) -> Result<StoredIdentitySession, String> {
    let settings = {
        let conn = state.conn.lock();
        get_cloud_identity_settings(&conn).map_err(map_err)?
    };

    if settings.provider != PROVIDER_SUPABASE {
        return Err("Sign-in is available for Supabase provider. Save Supabase settings first.".into());
    }

    let (base, anon) = supabase_credentials(state).await?;
    let token_url = format!("{base}/auth/v1/token?grant_type=password");
    let client = reqwest::Client::new();
    let resp = client
        .post(token_url)
        .header("apikey", &anon)
        .header("Authorization", format!("Bearer {anon}"))
        .json(&serde_json::json!({
            "email": email.trim(),
            "password": password,
        }))
        .send()
        .await
        .map_err(|e| format!("Sign-in request failed: {e}"))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(friendly_auth_error("Sign-in", &body));
    }

    let parsed: SupabaseTokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("Unexpected sign-in response: {e}"))?;

    store_supabase_session(
        state,
        parsed.user.id,
        parsed.user.email,
        parsed.access_token,
        parsed.refresh_token,
        parsed.expires_in,
    )
}

#[derive(Debug, Deserialize)]
pub struct WorkspaceSignInInput {
    pub email: String,
    pub password: String,
}

#[tauri::command]
pub async fn workspace_sign_in_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: WorkspaceSignInInput,
) -> Result<IdentitySessionPublic, String> {
    let (complete, provider, admin_email, admin_password, admin_id) = {
        let conn = state.conn.lock();
        (
            crate::db::setup::is_setup_complete(&conn).map_err(map_err)?,
            get_meta(&conn, "identity_provider")
                .map_err(map_err)?
                .unwrap_or_else(|| "none".into()),
            get_meta(&conn, "setup_admin_email")
                .map_err(map_err)?
                .unwrap_or_default(),
            get_meta(&conn, "setup_admin_password")
                .map_err(map_err)?
                .unwrap_or_default(),
            get_meta(&conn, "setup_admin_id").map_err(map_err)?,
        )
    };

    if !complete {
        return Err("This workspace is not set up yet. Choose Set up.".into());
    }

    if provider == PROVIDER_SUPABASE {
        if let Err(err) = perform_supabase_sign_in(&state, &input.email, &input.password).await {
            let upper = input.password.to_ascii_uppercase();
            if upper == input.password {
                return Err(err);
            }
            perform_supabase_sign_in(&state, &input.email, &upper).await?;
        }
        crate::commands::cloud_sync::require_confluity_account(&state).await?;
        {
            let conn = state.conn.lock();
            let session = stored_session(&conn).map_err(map_err)?;
            if let Some(session) = session {
                bind_signed_in_member(&conn, &session.user_id, session.email.as_deref())?;
            }
        }
        return get_identity_session_cmd(state);
    }

    let email_ok = admin_email.eq_ignore_ascii_case(input.email.trim());
    let password_ok = !input.password.is_empty() && input.password == admin_password;
    if !email_ok || !password_ok {
        return Err("Email or password is incorrect.".into());
    }
    if let Some(id) = admin_id.as_deref().filter(|id| !id.is_empty()) {
        let conn = state.conn.lock();
        set_meta(&conn, "current_member_id", id).map_err(map_err)?;
    }

    Ok(IdentitySessionPublic {
        signed_in: true,
        provider: Some("local".into()),
        user_id: admin_id,
        email: Some(admin_email),
        must_set_password: false,
    })
}

#[tauri::command]
pub async fn identity_sign_in_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: IdentitySignInInput,
) -> Result<IdentitySessionPublic, String> {
    crate::commands::roles::require_perm(&state, "settings.data_server")?;
    perform_supabase_sign_in(&state, &input.email, &input.password).await?;
    crate::commands::cloud_sync::require_confluity_account(&state).await?;
    get_identity_session_cmd(state)
}

#[tauri::command]
pub fn identity_sign_out_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<IdentitySessionPublic, String> {
    crate::commands::roles::require_perm(&state, "settings.data_server")?;
    {
        let conn = state.conn.lock();
        write_session(&conn, None).map_err(map_err)?;
    }
    get_identity_session_cmd(state)
}

#[tauri::command]
pub async fn sync_cloud_identity_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<IdentitySyncResult, String> {
    crate::commands::roles::require_perm(&state, "settings.data_server")?;
    let ws = workspace_id(&state)?;

    let (settings, session, service_role) = {
        let conn = state.conn.lock();
        (
            get_cloud_identity_settings(&conn).map_err(map_err)?,
            stored_session(&conn).map_err(map_err)?,
            crate::db::identity::supabase_service_role(&conn).map_err(map_err)?,
        )
    };

    let session = session.ok_or_else(|| "Sign in before syncing.".to_string())?;

    if settings.provider != PROVIDER_SUPABASE {
        return Ok(IdentitySyncResult {
            ok: true,
            message: "Custom API sync is not implemented yet; server URL is linked for hybrid mode."
                .into(),
            members_upserted: 0,
        });
    }

    if !settings.sync_members_enabled {
        return Ok(IdentitySyncResult {
            ok: true,
            message: "Member sync is disabled in cloud login settings.".into(),
            members_upserted: 0,
        });
    }

    let url = settings
        .supabase_url
        .filter(|u| !u.trim().is_empty())
        .ok_or_else(|| "Supabase project URL is required.".to_string())?;
    let base = normalize_supabase_url(&url);
    let table = settings.members_table.trim();
    let rest_url = format!("{base}/rest/v1/{table}?select=id,email,name,full_name,role,title");

    let bearer = service_role
        .as_ref()
        .filter(|k| !k.trim().is_empty())
        .map(|k| k.as_str())
        .unwrap_or(session.access_token.as_str());

    let anon = {
        let conn = state.conn.lock();
        crate::db::identity::supabase_anon_key(&conn).map_err(map_err)?
    }
    .ok_or_else(|| "Supabase anon key is required.".to_string())?;

    let client = reqwest::Client::new();
    let resp = client
        .get(rest_url)
        .header("apikey", &anon)
        .header("Authorization", format!("Bearer {bearer}"))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Member sync request failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "Could not read `{table}` from Supabase ({status}). Ensure the table exists and RLS/policies allow access. {body}"
        ));
    }

    let rows: Vec<serde_json::Value> = resp
        .json()
        .await
        .map_err(|e| format!("Invalid member rows: {e}"))?;

    let mut upserted = 0i64;
    let ts = chrono::Utc::now().to_rfc3339();
    {
        let conn = state.conn.lock();
        for row in rows {
            let id = row
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(crate::db::new_id);
            let name = row
                .get("name")
                .or_else(|| row.get("full_name"))
                .and_then(|v| v.as_str())
                .unwrap_or("Workspace user")
                .to_string();
            let role = row
                .get("role")
                .and_then(|v| v.as_str())
                .unwrap_or("support")
                .to_string();
            let title = row.get("title").and_then(|v| v.as_str());

            let exists: bool = conn
                .query_row(
                    "SELECT 1 FROM members WHERE id = ?1 AND workspace_id = ?2 AND deleted_at IS NULL",
                    rusqlite::params![id, ws],
                    |_| Ok(true),
                )
                .unwrap_or(false);

            if exists {
                conn.execute(
                    "UPDATE members SET name = ?1, role = ?2, title = ?3, updated_at = ?4, version = version + 1
                     WHERE id = ?5 AND workspace_id = ?6",
                    rusqlite::params![name, role, title, ts, id, ws],
                )
                .map_err(|e| e.to_string())?;
            } else {
                conn.execute(
                    "INSERT INTO members (id, workspace_id, name, role, title, created_at, updated_at, deleted_at, version)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, NULL, 1)",
                    rusqlite::params![id, ws, name, role, title, ts],
                )
                .map_err(|e| e.to_string())?;
            }
            upserted += 1;
        }
        set_meta(&conn, "last_sync", &ts).map_err(map_err)?;
    }

    Ok(IdentitySyncResult {
        ok: true,
        message: format!("Synced {upserted} member row(s) from `{table}`."),
        members_upserted: upserted,
    })
}

fn bind_signed_in_member(
    conn: &rusqlite::Connection,
    user_id: &str,
    email: Option<&str>,
) -> Result<(), String> {
    let ws = get_meta(conn, "workspace_id")
        .map_err(map_err)?
        .unwrap_or_default();
    let email = email.unwrap_or("");
    let found: Option<String> = conn
        .query_row(
            "SELECT id FROM members
             WHERE workspace_id = ?1 AND deleted_at IS NULL
               AND (id = ?2 OR (?3 != '' AND lower(coalesce(email, '')) = lower(?3)))
             ORDER BY CASE WHEN id = ?2 THEN 0 ELSE 1 END
             LIMIT 1",
            rusqlite::params![ws, user_id, email],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let Some(id) = found else {
        return Err("This login is not a member of the workspace.".into());
    };
    set_meta(conn, "current_member_id", &id).map_err(map_err)?;
    Ok(())
}

fn invite_code() -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let raw = uuid::Uuid::new_v4();
    let mut code = String::with_capacity(9);
    for (i, byte) in raw.as_bytes().iter().take(8).enumerate() {
        if i == 4 {
            code.push('-');
        }
        code.push(ALPHABET[(*byte as usize) % ALPHABET.len()] as char);
    }
    code
}

fn normalize_email(email: &str) -> Result<String, String> {
    let email = email.trim().to_ascii_lowercase();
    let Some((name, domain)) = email.split_once('@') else {
        return Err("Enter a valid email.".into());
    };
    if name.is_empty() || !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
        return Err("Enter a valid email.".into());
    }
    Ok(email)
}

enum SignupOutcome {
    Created(String),
    AlreadyRegistered,
}

async fn signup_member_auth(
    state: &AppState,
    email: &str,
    password: &str,
    full_name: &str,
) -> Result<SignupOutcome, String> {
    let (base, anon) = supabase_credentials(state).await?;
    let client = reqwest::Client::new();
    let signup = client
        .post(format!("{base}/auth/v1/signup"))
        .header("apikey", &anon)
        .header("Authorization", format!("Bearer {anon}"))
        .json(&serde_json::json!({
            "email": email,
            "password": password,
            "data": { "full_name": full_name }
        }))
        .send()
        .await
        .map_err(|e| format!("Could not reach Supabase: {e}"))?;
    let status = signup.status();
    let body = signup.text().await.unwrap_or_default();
    if status.as_u16() == 429 {
        return Err("Supabase is rate limiting signups. Wait a moment and try again.".into());
    }
    let lower = body.to_ascii_lowercase();
    if status.as_u16() == 422
        || lower.contains("already registered")
        || lower.contains("user_already_exists")
    {
        return Ok(SignupOutcome::AlreadyRegistered);
    }
    if !status.is_success() {
        return Err(friendly_auth_error("Invite", &body));
    }
    let parsed: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("Unexpected Supabase response: {e}"))?;
    let already_exists = parsed
        .pointer("/user/identities")
        .and_then(|v| v.as_array())
        .is_some_and(|list| list.is_empty());
    if already_exists {
        return Ok(SignupOutcome::AlreadyRegistered);
    }
    let user_id = parsed
        .pointer("/user/id")
        .or_else(|| parsed.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if user_id.is_empty() {
        return Err("Supabase did not return an account id.".into());
    }
    crate::commands::cloud_sync::confirm_auth_email(state, email).await?;
    Ok(SignupOutcome::Created(user_id))
}

async fn update_signed_in_password(state: &AppState, password: &str) -> Result<(), String> {
    let (base, anon, access_token) = {
        let conn = state.conn.lock();
        let (base, anon) = {
            let settings = get_cloud_identity_settings(&conn).map_err(map_err)?;
            if settings.provider != PROVIDER_SUPABASE {
                return Err("Password setup is available when Supabase is the workspace server.".into());
            }
            let url = settings
                .supabase_url
                .filter(|u| !u.trim().is_empty())
                .ok_or_else(|| "Supabase project URL is required.".to_string())?;
            let anon = crate::db::identity::supabase_anon_key(&conn)
                .map_err(map_err)?
                .ok_or_else(|| "Supabase anon key is required.".to_string())?;
            (normalize_supabase_url(&url), anon)
        };
        let access_token = stored_session(&conn)
            .map_err(map_err)?
            .map(|s| s.access_token)
            .filter(|t| !t.is_empty())
            .ok_or_else(|| "Sign in with the code before choosing a password.".to_string())?;
        (base, anon, access_token)
    };
    let client = reqwest::Client::new();
    let resp = client
        .put(format!("{base}/auth/v1/user"))
        .header("apikey", &anon)
        .header("Authorization", format!("Bearer {access_token}"))
        .json(&serde_json::json!({ "password": password }))
        .send()
        .await
        .map_err(|e| format!("Could not reach Supabase: {e}"))?;
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(friendly_auth_error("Password update", &body));
    }
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct InvitedMember {
    pub member: crate::db::models::Member,
    pub invite_code: String,
}

#[derive(Debug, Deserialize)]
pub struct InviteMemberInput {
    pub name: String,
    pub email: String,
    pub position_id: Option<String>,
}

struct PendingMember {
    id: String,
    must_set_password: bool,
    invite_code: Option<String>,
}

fn member_by_email(conn: &rusqlite::Connection, workspace_id: &str, email: &str) -> Result<Option<PendingMember>, String> {
    conn.query_row(
        "SELECT id, must_set_password, invite_code FROM members
         WHERE workspace_id = ?1 AND deleted_at IS NULL AND lower(coalesce(email, '')) = lower(?2)",
        rusqlite::params![workspace_id, email],
        |row| {
            Ok(PendingMember {
                id: row.get(0)?,
                must_set_password: row.get::<_, i64>(1)? != 0,
                invite_code: row.get(2)?,
            })
        },
    )
    .optional()
    .map_err(|e| e.to_string())
}

fn load_member(conn: &rusqlite::Connection, workspace_id: &str, id: &str) -> Result<crate::db::models::Member, String> {
    crate::db::models::list_members(conn, workspace_id)
        .map_err(map_err)?
        .into_iter()
        .find(|m| m.id == id)
        .ok_or_else(|| "Could not load the new member.".into())
}

#[tauri::command]
pub async fn invite_member_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: InviteMemberInput,
) -> Result<InvitedMember, String> {
    crate::commands::roles::require_perm(&state, "settings.manage_members")?;
    let name = input.name.trim();
    if name.is_empty() {
        return Err("Name is required.".into());
    }
    let email = normalize_email(&input.email)?;
    let position_id = input
        .position_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(|id| id.to_string());

    let (ws, provider, admin_email) = {
        let conn = state.conn.lock();
        (
            get_meta(&conn, "workspace_id")
                .map_err(map_err)?
                .ok_or_else(|| "workspace not initialized".to_string())?,
            get_meta(&conn, "identity_provider")
                .map_err(map_err)?
                .unwrap_or_else(|| "none".into()),
            get_meta(&conn, "setup_admin_email")
                .map_err(map_err)?
                .unwrap_or_default(),
        )
    };
    if provider != PROVIDER_SUPABASE {
        return Err("Connect Supabase before inviting members who can sign in.".into());
    }
    if admin_email.eq_ignore_ascii_case(&email) {
        return Err("That email already belongs to the administrator account.".into());
    }

    if let Some(existing) = {
        let conn = state.conn.lock();
        member_by_email(&conn, &ws, &email)?
    } {
        if !existing.must_set_password {
            return Err("A member with this email already has a password.".into());
        }
        let code = existing.invite_code.ok_or_else(|| {
            "This member is waiting for a password, but the sign-in code is missing. Remove the member and invite them again.".to_string()
        })?;
        crate::commands::cloud_sync::push_workspace_records(&state).await?;
        let member = {
            let conn = state.conn.lock();
            load_member(&conn, &ws, &existing.id)?
        };
        return Ok(InvitedMember {
            member,
            invite_code: code,
        });
    }

    let code = invite_code();
    let user_id = match signup_member_auth(&state, &email, &code, name).await? {
        SignupOutcome::Created(id) => id,
        SignupOutcome::AlreadyRegistered => {
            crate::commands::cloud_sync::reset_auth_password(&state, &email, &code).await?
        }
    };

    let ts = chrono::Utc::now().to_rfc3339();
    {
        let conn = state.conn.lock();
        conn.execute(
            "INSERT INTO members (id, workspace_id, name, role, title, email, must_set_password, invite_code, created_at, updated_at, deleted_at, version)
             VALUES (?1, ?2, ?3, 'support', NULL, ?4, 1, ?5, ?6, ?6, NULL, 1)",
            rusqlite::params![user_id, ws, name, email, code, ts],
        )
        .map_err(|e| e.to_string())?;
        if let Some(position_id) = position_id.as_deref() {
            crate::db::positions::set_member_position(&conn, &user_id, Some(position_id))
                .map_err(map_err)?;
        } else if let Ok(Some(support)) = crate::db::roles::find_role_by_name(&conn, &ws, "Support") {
            let _ = crate::db::roles::set_member_roles(&conn, &user_id, &[support.id]);
        }
        crate::db::record_change(
            &conn,
            &ws,
            "create",
            "member",
            &user_id,
            &format!("Invited member {name}"),
        )
        .map_err(map_err)?;
    }

    if let Err(err) = crate::commands::cloud_sync::push_workspace_records(&state).await {
        return Err(format!(
            "The member was added, but Supabase did not save the account yet ({err}). Submit this email again to finish."
        ));
    }

    let member = {
        let conn = state.conn.lock();
        load_member(&conn, &ws, &user_id)?
    };
    Ok(InvitedMember {
        member,
        invite_code: code,
    })
}

#[derive(Debug, Deserialize)]
pub struct RevealInviteInput {
    pub member_id: String,
}

#[tauri::command]
pub fn reveal_member_invite_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: RevealInviteInput,
) -> Result<String, String> {
    crate::commands::roles::require_perm(&state, "settings.manage_members")?;
    let conn = state.conn.lock();
    let code: Option<String> = conn
        .query_row(
            "SELECT invite_code FROM members
             WHERE id = ?1 AND deleted_at IS NULL AND must_set_password = 1",
            rusqlite::params![input.member_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    code.filter(|c| !c.is_empty())
        .ok_or_else(|| "This member has already chosen a password.".into())
}

#[derive(Debug, Deserialize)]
pub struct CompleteMemberPasswordInput {
    pub password: String,
    pub confirm: String,
}

#[tauri::command]
pub async fn complete_member_password_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CompleteMemberPasswordInput,
) -> Result<IdentitySessionPublic, String> {
    if input.password != input.confirm {
        return Err("Re-enter the same password in both fields.".into());
    }
    if input.password.len() < 6 {
        return Err("Use a password of at least 6 characters.".into());
    }
    let (user_id, email) = {
        let conn = state.conn.lock();
        let session = stored_session(&conn)
            .map_err(map_err)?
            .ok_or_else(|| "Sign in with the code before choosing a password.".to_string())?;
        (session.user_id, session.email.unwrap_or_default())
    };
    let member_id = {
        let conn = state.conn.lock();
        let ws = get_meta(&conn, "workspace_id")
            .map_err(map_err)?
            .ok_or_else(|| "workspace not initialized".to_string())?;
        let row: Option<(String, i64, Option<String>)> = conn
            .query_row(
                "SELECT id, must_set_password, invite_code FROM members
                 WHERE workspace_id = ?1 AND deleted_at IS NULL
                   AND (id = ?2 OR (?3 != '' AND lower(coalesce(email, '')) = lower(?3)))
                 ORDER BY CASE WHEN id = ?2 THEN 0 ELSE 1 END
                 LIMIT 1",
                rusqlite::params![ws, user_id, email],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let Some((id, pending, code)) = row else {
            return Err("This login is not a member of the workspace.".into());
        };
        if pending == 0 {
            return Err("This account already has a password. Sign in with it.".into());
        }
        if code.as_deref() == Some(input.password.as_str()) {
            return Err("Choose a password that is different from the sign-in code.".into());
        }
        id
    };

    update_signed_in_password(&state, &input.password).await?;

    let ts = chrono::Utc::now().to_rfc3339();
    {
        let conn = state.conn.lock();
        conn.execute(
            "UPDATE members SET must_set_password = 0, invite_code = NULL, updated_at = ?1, version = version + 1
             WHERE id = ?2",
            rusqlite::params![ts, member_id],
        )
        .map_err(|e| e.to_string())?;
        set_meta(&conn, "current_member_id", &member_id).map_err(map_err)?;
    }
    get_identity_session_cmd(state)
}
