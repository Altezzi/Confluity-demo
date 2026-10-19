use serde::{Deserialize, Serialize};

use super::{get_meta, set_meta, DbError};

const API_META_KEY: &str = "api_connections";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredApiConnection {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub enabled: bool,
    pub is_default: bool,
}

fn read_raw(conn: &rusqlite::Connection) -> Result<Vec<StoredApiConnection>, DbError> {
    let raw = get_meta(conn, API_META_KEY)?;
    let Some(json) = raw.filter(|s| !s.trim().is_empty()) else {
        return Ok(Vec::new());
    };
    serde_json::from_str(&json).map_err(|e| {
        DbError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
    })
}

pub fn write_api_connections(
    conn: &rusqlite::Connection,
    items: &[StoredApiConnection],
) -> Result<(), DbError> {
    let json = serde_json::to_string(items).map_err(|e| {
        DbError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
    })?;
    set_meta(conn, API_META_KEY, &json)?;
    sync_legacy_ai_meta(conn, items)?;
    Ok(())
}

pub fn sync_legacy_ai_meta(conn: &rusqlite::Connection, items: &[StoredApiConnection]) -> Result<(), DbError> {
    let pick = items
        .iter()
        .find(|c| c.is_default && c.enabled)
        .or_else(|| items.iter().find(|c| c.enabled))
        .or_else(|| items.first());

    if let Some(c) = pick {
        set_meta(conn, "ai_api_base_url", &c.base_url)?;
        set_meta(conn, "ai_model", &c.model)?;
        set_meta(conn, "ai_api_key", c.api_key.as_deref().unwrap_or(""))?;
    }
    Ok(())
}

pub fn migrate_legacy_api_connections(conn: &rusqlite::Connection) -> Result<Vec<StoredApiConnection>, DbError> {
    let existing = read_raw(conn)?;
    if !existing.is_empty() {
        return Ok(existing);
    }

    let base = get_meta(conn, "ai_api_base_url")?
        .unwrap_or_else(|| "https://api.openai.com/v1".into());
    let model = get_meta(conn, "ai_model")?
        .unwrap_or_else(|| "gpt-4o-mini".into());
    let key = get_meta(conn, "ai_api_key")?.filter(|k| !k.trim().is_empty());
    let has_legacy = key.is_some()
        || get_meta(conn, "ai_api_base_url")?.is_some()
        || get_meta(conn, "ai_model")?.is_some();

    if !has_legacy {
        return Ok(Vec::new());
    }

    Ok(vec![StoredApiConnection {
        id: "api_default".into(),
        name: "Default provider".into(),
        base_url: base,
        model,
        api_key: key,
        enabled: true,
        is_default: true,
    }])
}

pub fn load_api_connections(conn: &rusqlite::Connection) -> Result<Vec<StoredApiConnection>, DbError> {
    let items = migrate_legacy_api_connections(conn)?;
    if read_raw(conn)?.is_empty() && !items.is_empty() {
        write_api_connections(conn, &items)?;
    }
    Ok(items)
}

pub fn active_api_credentials(conn: &rusqlite::Connection) -> Result<(String, String, String), DbError> {
    let items = load_api_connections(conn)?;
    let pick = items
        .iter()
        .find(|c| c.is_default && c.enabled)
        .or_else(|| items.iter().find(|c| c.enabled))
        .or_else(|| items.first());

    if let Some(c) = pick {
        return Ok((
            c.api_key.clone().unwrap_or_default(),
            c.base_url.clone(),
            c.model.clone(),
        ));
    }

    Ok((
        get_meta(conn, "ai_api_key")?.unwrap_or_default(),
        get_meta(conn, "ai_api_base_url")?
            .unwrap_or_else(|| "https://api.openai.com/v1".into()),
        get_meta(conn, "ai_model")?
            .unwrap_or_else(|| "gpt-4o-mini".into()),
    ))
}

pub fn any_api_key_configured(conn: &rusqlite::Connection) -> Result<bool, DbError> {
    let items = load_api_connections(conn)?;
    if items
        .iter()
        .any(|c| c.api_key.as_ref().is_some_and(|k| !k.trim().is_empty()))
    {
        return Ok(true);
    }
    Ok(get_meta(conn, "ai_api_key")?
        .is_some_and(|k| !k.trim().is_empty()))
}
