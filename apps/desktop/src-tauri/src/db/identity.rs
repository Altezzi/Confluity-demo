use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use super::{get_meta, set_meta, DbResult};

pub const PROVIDER_NONE: &str = "none";
pub const PROVIDER_SUPABASE: &str = "supabase";
pub const PROVIDER_CUSTOM: &str = "custom_api";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CloudIdentitySettings {
    pub provider: String,
    pub supabase_url: Option<String>,
    pub supabase_anon_key_configured: bool,
    pub supabase_service_role_configured: bool,
    pub custom_api_url: Option<String>,
    pub sync_members_enabled: bool,
    pub members_table: String,
    pub sync_data_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredIdentitySession {
    pub provider: String,
    pub user_id: String,
    pub email: Option<String>,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentitySessionPublic {
    pub signed_in: bool,
    pub provider: Option<String>,
    pub user_id: Option<String>,
    pub email: Option<String>,
    #[serde(default)]
    pub must_set_password: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentitySyncResult {
    pub ok: bool,
    pub message: String,
    pub members_upserted: i64,
}

pub fn get_cloud_identity_settings(conn: &Connection) -> DbResult<CloudIdentitySettings> {
    let provider = get_meta(conn, "identity_provider")?
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| PROVIDER_NONE.into());
    Ok(CloudIdentitySettings {
        provider,
        supabase_url: get_meta(conn, "identity_supabase_url")?,
        supabase_anon_key_configured: get_meta(conn, "identity_supabase_anon_key")?
            .is_some_and(|k| !k.trim().is_empty()),
        supabase_service_role_configured: get_meta(conn, "identity_supabase_service_role")?
            .is_some_and(|k| !k.trim().is_empty()),
        custom_api_url: get_meta(conn, "identity_custom_api_url")?,
        sync_members_enabled: get_meta(conn, "identity_sync_members")?.as_deref() == Some("true"),
        members_table: get_meta(conn, "identity_members_table")?
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| "workspace_members".into()),
        sync_data_enabled: get_meta(conn, "identity_sync_data")?.as_deref() == Some("true"),
    })
}

pub fn update_cloud_identity_settings(
    conn: &Connection,
    input: &UpdateCloudIdentitySettings,
) -> DbResult<()> {
    set_meta(conn, "identity_provider", &input.provider)?;
    if let Some(url) = &input.supabase_url {
        set_meta(conn, "identity_supabase_url", url.trim())?;
    }
    if let Some(key) = &input.supabase_anon_key {
        if !key.trim().is_empty() {
            set_meta(conn, "identity_supabase_anon_key", key.trim())?;
        }
    }
    if input.clear_supabase_anon_key == Some(true) {
        set_meta(conn, "identity_supabase_anon_key", "")?;
    }
    if let Some(key) = &input.supabase_service_role {
        if !key.trim().is_empty() {
            set_meta(conn, "identity_supabase_service_role", key.trim())?;
        }
    }
    if input.clear_supabase_service_role == Some(true) {
        set_meta(conn, "identity_supabase_service_role", "")?;
    }
    if let Some(url) = &input.custom_api_url {
        set_meta(conn, "identity_custom_api_url", url.trim())?;
    }
    if let Some(v) = input.sync_members_enabled {
        set_meta(conn, "identity_sync_members", if v { "true" } else { "false" })?;
    }
    if let Some(table) = &input.members_table {
        set_meta(conn, "identity_members_table", table.trim())?;
    }
    if let Some(v) = input.sync_data_enabled {
        set_meta(conn, "identity_sync_data", if v { "true" } else { "false" })?;
    }
    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCloudIdentitySettings {
    pub provider: String,
    pub supabase_url: Option<String>,
    pub supabase_anon_key: Option<String>,
    pub clear_supabase_anon_key: Option<bool>,
    pub supabase_service_role: Option<String>,
    pub clear_supabase_service_role: Option<bool>,
    pub custom_api_url: Option<String>,
    pub sync_members_enabled: Option<bool>,
    pub members_table: Option<String>,
    pub sync_data_enabled: Option<bool>,
}

fn read_session(conn: &Connection) -> DbResult<Option<StoredIdentitySession>> {
    let raw = get_meta(conn, "identity_session")?;
    let Some(json) = raw.filter(|s| !s.trim().is_empty()) else {
        return Ok(None);
    };
    serde_json::from_str(&json).map_err(|e| {
        super::DbError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
    })
}

pub fn write_session(conn: &Connection, session: Option<&StoredIdentitySession>) -> DbResult<()> {
    match session {
        Some(s) => {
            let json = serde_json::to_string(s).map_err(|e| {
                super::DbError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })?;
            set_meta(conn, "identity_session", &json)
        }
        None => set_meta(conn, "identity_session", ""),
    }
}

pub fn get_identity_session_public(conn: &Connection) -> DbResult<IdentitySessionPublic> {
    let session = read_session(conn)?;
    Ok(match session {
        Some(s) => {
            let must_set_password = member_must_set_password(conn, &s.user_id, s.email.as_deref());
            IdentitySessionPublic {
                signed_in: true,
                provider: Some(s.provider),
                user_id: Some(s.user_id),
                email: s.email,
                must_set_password,
            }
        }
        None => IdentitySessionPublic {
            signed_in: false,
            provider: None,
            user_id: None,
            email: None,
            must_set_password: false,
        },
    })
}

fn member_must_set_password(conn: &Connection, user_id: &str, email: Option<&str>) -> bool {
    let email = email.unwrap_or("");
    conn.query_row(
        "SELECT must_set_password FROM members
         WHERE deleted_at IS NULL AND (id = ?1 OR (?2 != '' AND lower(coalesce(email, '')) = lower(?2)))
         ORDER BY CASE WHEN id = ?1 THEN 0 ELSE 1 END
         LIMIT 1",
        rusqlite::params![user_id, email],
        |row| row.get::<_, i64>(0),
    )
    .unwrap_or(0)
        != 0
}

pub fn supabase_anon_key(conn: &Connection) -> DbResult<Option<String>> {
    Ok(get_meta(conn, "identity_supabase_anon_key")?.filter(|k| !k.trim().is_empty()))
}

pub fn supabase_service_role(conn: &Connection) -> DbResult<Option<String>> {
    Ok(get_meta(conn, "identity_supabase_service_role")?.filter(|k| !k.trim().is_empty()))
}

pub fn stored_session(conn: &Connection) -> DbResult<Option<StoredIdentitySession>> {
    read_session(conn)
}
