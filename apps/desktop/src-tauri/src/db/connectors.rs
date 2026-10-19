use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{log_activity, new_id, now_iso, upsert_fts, DbError, DbResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorInstance {
    pub id: String,
    pub workspace_id: String,
    pub connector_id: String,
    pub status: String,
    pub config_json: String,
    pub last_sync_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalRef {
    pub id: String,
    pub workspace_id: String,
    pub provider: String,
    pub external_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub source_url: Option<String>,
    pub metadata_json: Option<String>,
    pub last_synced_at: String,
    pub sync_conflict: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorSyncSummary {
    pub provider: String,
    pub created: i64,
    pub updated: i64,
    pub conflicts: i64,
    pub skipped: i64,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorDef {
    pub id: String,
    pub label: String,
    pub description: String,
    pub modules: Vec<String>,
    pub capabilities: Vec<String>,
}

pub fn connector_catalog() -> Vec<ConnectorDef> {
    vec![
        ConnectorDef {
            id: "zendesk".into(),
            label: "Zendesk".into(),
            description: "Sync support tickets. Stay on Zendesk, or adopt into Confluity Support."
                .into(),
            modules: vec!["support".into()],
            capabilities: vec!["sync".into(), "import".into()],
        },
        ConnectorDef {
            id: "jira".into(),
            label: "Jira".into(),
            description: "Import engineering issues. Mirror boards, then adopt into Projects."
                .into(),
            modules: vec!["projects".into()],
            capabilities: vec!["sync".into(), "import".into()],
        },
        ConnectorDef {
            id: "datadog".into(),
            label: "Datadog".into(),
            description: "Observability alerts → Support tickets with deep links.".into(),
            modules: vec!["support".into(), "operations".into()],
            capabilities: vec!["import".into(), "webhook".into()],
        },
        ConnectorDef {
            id: "grafana".into(),
            label: "Grafana".into(),
            description: "Import Grafana alert payloads into Support.".into(),
            modules: vec!["support".into(), "operations".into()],
            capabilities: vec!["import".into(), "webhook".into()],
        },
        ConnectorDef {
            id: "cloudwatch".into(),
            label: "CloudWatch".into(),
            description: "Import AWS CloudWatch alarms into Support.".into(),
            modules: vec!["support".into(), "operations".into()],
            capabilities: vec!["import".into(), "webhook".into()],
        },
        ConnectorDef {
            id: "csv".into(),
            label: "CSV Import".into(),
            description: "Bulk import companies and contacts.".into(),
            modules: vec!["crm".into()],
            capabilities: vec!["import".into()],
        },
        ConnectorDef {
            id: "email".into(),
            label: "Email to ticket".into(),
            description: "Turn inbound email into Support tickets.".into(),
            modules: vec!["support".into()],
            capabilities: vec!["import".into(), "webhook".into()],
        },
        ConnectorDef {
            id: "slack".into(),
            label: "Slack".into(),
            description: "Coming soon — notify channels from workspace events.".into(),
            modules: vec!["teams".into(), "support".into()],
            capabilities: vec!["embed".into(), "webhook".into()],
        },
        ConnectorDef {
            id: "quickbooks".into(),
            label: "QuickBooks".into(),
            description: "Coming soon — sync invoices and payments into Finance.".into(),
            modules: vec!["finance".into()],
            capabilities: vec!["sync".into(), "import".into()],
        },
    ]
}

fn map_connector(row: &rusqlite::Row<'_>) -> rusqlite::Result<ConnectorInstance> {
    Ok(ConnectorInstance {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        connector_id: row.get(2)?,
        status: row.get(3)?,
        config_json: row.get(4)?,
        last_sync_at: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        deleted_at: row.get(8)?,
        version: row.get(9)?,
    })
}

fn map_ref(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExternalRef> {
    let conflict: i64 = row.get(9)?;
    Ok(ExternalRef {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        provider: row.get(2)?,
        external_id: row.get(3)?,
        entity_type: row.get(4)?,
        entity_id: row.get(5)?,
        source_url: row.get(6)?,
        metadata_json: row.get(7)?,
        last_synced_at: row.get(8)?,
        sync_conflict: conflict != 0,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

const C_COLS: &str =
    "id, workspace_id, connector_id, status, config_json, last_sync_at, created_at, updated_at, deleted_at, version";
const R_COLS: &str = "id, workspace_id, provider, external_id, entity_type, entity_id, source_url, metadata_json, last_synced_at, sync_conflict, created_at, updated_at";

pub fn list_connectors(conn: &Connection, workspace_id: &str) -> DbResult<Vec<ConnectorInstance>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {C_COLS} FROM connectors
         WHERE workspace_id = ?1 AND deleted_at IS NULL
         ORDER BY connector_id COLLATE NOCASE"
    ))?;
    let rows = stmt.query_map(params![workspace_id], map_connector)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_connector(conn: &Connection, id: &str) -> DbResult<Option<ConnectorInstance>> {
    conn.query_row(
        &format!("SELECT {C_COLS} FROM connectors WHERE id = ?1 AND deleted_at IS NULL"),
        params![id],
        map_connector,
    )
    .optional()
    .map_err(Into::into)
}

pub fn get_connector_by_type(
    conn: &Connection,
    workspace_id: &str,
    connector_id: &str,
) -> DbResult<Option<ConnectorInstance>> {
    conn.query_row(
        &format!(
            "SELECT {C_COLS} FROM connectors
             WHERE workspace_id = ?1 AND connector_id = ?2 AND deleted_at IS NULL
             ORDER BY updated_at DESC LIMIT 1"
        ),
        params![workspace_id, connector_id],
        map_connector,
    )
    .optional()
    .map_err(Into::into)
}

pub fn upsert_connector(
    conn: &Connection,
    workspace_id: &str,
    connector_id: &str,
    status: &str,
    config_json: &str,
) -> DbResult<ConnectorInstance> {
    if connector_catalog()
        .iter()
        .all(|c| c.id != connector_id)
    {
        return Err(DbError::Sqlite(rusqlite::Error::InvalidParameterName(
            format!("unknown connector: {connector_id}"),
        )));
    }
    let ts = now_iso();
    if let Some(existing) = get_connector_by_type(conn, workspace_id, connector_id)? {
        conn.execute(
            "UPDATE connectors SET status = ?1, config_json = ?2, updated_at = ?3, version = version + 1
             WHERE id = ?4",
            params![status, config_json, ts, existing.id],
        )?;
        return get_connector(conn, &existing.id)?
            .ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows));
    }
    let id = new_id();
    conn.execute(
        "INSERT INTO connectors
         (id, workspace_id, connector_id, status, config_json, last_sync_at, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?6, NULL, 1)",
        params![id, workspace_id, connector_id, status, config_json, ts],
    )?;
    get_connector(conn, &id)?.ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}

pub fn disconnect_connector(conn: &Connection, id: &str) -> DbResult<ConnectorInstance> {
    let ts = now_iso();
    conn.execute(
        "UPDATE connectors SET status = 'disconnected', updated_at = ?1, version = version + 1
         WHERE id = ?2 AND deleted_at IS NULL",
        params![ts, id],
    )?;
    get_connector(conn, id)?.ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}

pub fn find_external_ref(
    conn: &Connection,
    workspace_id: &str,
    provider: &str,
    external_id: &str,
) -> DbResult<Option<ExternalRef>> {
    conn.query_row(
        &format!(
            "SELECT {R_COLS} FROM external_refs
             WHERE workspace_id = ?1 AND provider = ?2 AND external_id = ?3"
        ),
        params![workspace_id, provider, external_id],
        map_ref,
    )
    .optional()
    .map_err(Into::into)
}

pub fn list_external_refs_for_entity(
    conn: &Connection,
    workspace_id: &str,
    entity_type: &str,
    entity_id: &str,
) -> DbResult<Vec<ExternalRef>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {R_COLS} FROM external_refs
         WHERE workspace_id = ?1 AND entity_type = ?2 AND entity_id = ?3
         ORDER BY provider COLLATE NOCASE"
    ))?;
    let rows = stmt.query_map(params![workspace_id, entity_type, entity_id], map_ref)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn upsert_external_ref(
    conn: &Connection,
    workspace_id: &str,
    provider: &str,
    external_id: &str,
    entity_type: &str,
    entity_id: &str,
    source_url: Option<&str>,
    metadata_json: Option<&str>,
) -> DbResult<ExternalRef> {
    let ts = now_iso();
    if let Some(existing) = find_external_ref(conn, workspace_id, provider, external_id)? {
        conn.execute(
            "UPDATE external_refs SET entity_type = ?1, entity_id = ?2, source_url = ?3,
             metadata_json = ?4, last_synced_at = ?5, sync_conflict = 0, updated_at = ?5
             WHERE id = ?6",
            params![
                entity_type,
                entity_id,
                source_url,
                metadata_json,
                ts,
                existing.id
            ],
        )?;
        return find_external_ref(conn, workspace_id, provider, external_id)?
            .ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows));
    }
    let id = new_id();
    conn.execute(
        "INSERT INTO external_refs
         (id, workspace_id, provider, external_id, entity_type, entity_id, source_url, metadata_json,
          last_synced_at, sync_conflict, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?9, ?9)",
        params![
            id,
            workspace_id,
            provider,
            external_id,
            entity_type,
            entity_id,
            source_url,
            metadata_json,
            ts
        ],
    )?;
    find_external_ref(conn, workspace_id, provider, external_id)?
        .ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}

fn default_company_id(conn: &Connection, workspace_id: &str) -> DbResult<Option<String>> {
    conn.query_row(
        "SELECT id FROM companies WHERE workspace_id = ?1 AND deleted_at IS NULL
         ORDER BY created_at LIMIT 1",
        params![workspace_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(Into::into)
}

/// Idempotent ticket upsert driven by an external provider id.
pub fn upsert_ticket_from_external(
    conn: &Connection,
    workspace_id: &str,
    provider: &str,
    external_id: &str,
    title: &str,
    description: Option<&str>,
    status: &str,
    priority: &str,
    company_id: Option<&str>,
    source_url: Option<&str>,
    metadata_json: Option<&str>,
) -> DbResult<(String, bool)> {
    let ts = now_iso();
    let company = match company_id {
        Some(id) if !id.is_empty() => id.to_string(),
        _ => default_company_id(conn, workspace_id)?.ok_or_else(|| {
            DbError::Sqlite(rusqlite::Error::InvalidParameterName(
                "no company available for import".into(),
            ))
        })?,
    };

    if let Some(existing) = find_external_ref(conn, workspace_id, provider, external_id)? {
        conn.execute(
            "UPDATE tickets SET title = ?1, description = ?2, status = ?3, priority = ?4,
             updated_at = ?5, version = version + 1
             WHERE id = ?6 AND deleted_at IS NULL",
            params![title, description, status, priority, ts, existing.entity_id],
        )?;
        upsert_external_ref(
            conn,
            workspace_id,
            provider,
            external_id,
            "ticket",
            &existing.entity_id,
            source_url,
            metadata_json,
        )?;
        let _ = upsert_fts(conn, "ticket", &existing.entity_id, title, description.unwrap_or(""));
        return Ok((existing.entity_id, false));
    }

    let ticket_id = new_id();
    conn.execute(
        "INSERT INTO tickets
         (id, workspace_id, company_id, contact_id, title, description, status, priority, tags,
          assignee_id, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, NULL, NULL, ?8, ?8, NULL, 1)",
        params![
            ticket_id,
            workspace_id,
            company,
            title,
            description,
            status,
            priority,
            ts
        ],
    )?;
    upsert_external_ref(
        conn,
        workspace_id,
        provider,
        external_id,
        "ticket",
        &ticket_id,
        source_url,
        metadata_json,
    )?;
    let _ = upsert_fts(conn, "ticket", &ticket_id, title, description.unwrap_or(""));
    let _ = log_activity(
        conn,
        workspace_id,
        "import",
        "ticket",
        &ticket_id,
        &format!("Imported from {provider}"),
    );
    Ok((ticket_id, true))
}

/// Idempotent issue upsert for Jira-style imports.
pub fn upsert_issue_from_external(
    conn: &Connection,
    workspace_id: &str,
    provider: &str,
    external_id: &str,
    title: &str,
    description: Option<&str>,
    status: &str,
    priority: &str,
    source_url: Option<&str>,
    metadata_json: Option<&str>,
) -> DbResult<(String, bool)> {
    let ts = now_iso();
    if let Some(existing) = find_external_ref(conn, workspace_id, provider, external_id)? {
        conn.execute(
            "UPDATE issues SET title = ?1, description = ?2, status = ?3, priority = ?4,
             updated_at = ?5, version = version + 1
             WHERE id = ?6 AND deleted_at IS NULL",
            params![title, description, status, priority, ts, existing.entity_id],
        )?;
        upsert_external_ref(
            conn,
            workspace_id,
            provider,
            external_id,
            "issue",
            &existing.entity_id,
            source_url,
            metadata_json,
        )?;
        let _ = upsert_fts(conn, "issue", &existing.entity_id, title, description.unwrap_or(""));
        return Ok((existing.entity_id, false));
    }

    let issue_id = new_id();
    conn.execute(
        "INSERT INTO issues
         (id, workspace_id, ticket_id, company_id, project_id, title, description, status, priority,
          blocked_reason, assignee_id, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, NULL, NULL, NULL, ?3, ?4, ?5, ?6, NULL, NULL, ?7, ?7, NULL, 1)",
        params![issue_id, workspace_id, title, description, status, priority, ts],
    )?;
    upsert_external_ref(
        conn,
        workspace_id,
        provider,
        external_id,
        "issue",
        &issue_id,
        source_url,
        metadata_json,
    )?;
    let _ = upsert_fts(conn, "issue", &issue_id, title, description.unwrap_or(""));
    Ok((issue_id, true))
}

/// Import a JSON array of ticket-like records for Zendesk/Jira stub connectors.
pub fn import_support_batch(
    conn: &Connection,
    workspace_id: &str,
    provider: &str,
    payload_json: &str,
    company_id: Option<&str>,
) -> DbResult<ConnectorSyncSummary> {
    let value: Value = serde_json::from_str(payload_json).map_err(|e| {
        DbError::Sqlite(rusqlite::Error::InvalidParameterName(format!(
            "invalid JSON: {e}"
        )))
    })?;
    let items = match value {
        Value::Array(arr) => arr,
        Value::Object(obj) => {
            if let Some(Value::Array(arr)) = obj.get("tickets").or_else(|| obj.get("issues")) {
                arr.clone()
            } else {
                vec![Value::Object(obj)]
            }
        }
        _ => {
            return Err(DbError::Sqlite(rusqlite::Error::InvalidParameterName(
                "expected JSON array or object".into(),
            )))
        }
    };

    let mut created = 0i64;
    let mut updated = 0i64;
    let mut skipped = 0i64;
    let as_issues = provider == "jira";

    for item in items {
        let Some(obj) = item.as_object() else {
            skipped += 1;
            continue;
        };
        let external_id = obj
            .get("id")
            .or_else(|| obj.get("external_id"))
            .or_else(|| obj.get("key"))
            .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            .unwrap_or_else(|| new_id());
        let title = obj
            .get("title")
            .or_else(|| obj.get("subject"))
            .or_else(|| obj.get("summary"))
            .and_then(|v| v.as_str())
            .unwrap_or("Imported record")
            .to_string();
        let description = obj
            .get("description")
            .or_else(|| obj.get("body"))
            .and_then(|v| v.as_str());
        let status = obj
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or(if as_issues { "todo" } else { "open" });
        let priority = obj
            .get("priority")
            .and_then(|v| v.as_str())
            .unwrap_or("medium");
        let source_url = obj
            .get("url")
            .or_else(|| obj.get("source_url"))
            .or_else(|| obj.get("html_url"))
            .and_then(|v| v.as_str());
        let meta = serde_json::to_string(obj).ok();

        let result = if as_issues {
            upsert_issue_from_external(
                conn,
                workspace_id,
                provider,
                &external_id,
                &title,
                description,
                status,
                priority,
                source_url,
                meta.as_deref(),
            )
        } else {
            upsert_ticket_from_external(
                conn,
                workspace_id,
                provider,
                &external_id,
                &title,
                description,
                status,
                priority,
                company_id,
                source_url,
                meta.as_deref(),
            )
        };

        match result {
            Ok((_, is_new)) => {
                if is_new {
                    created += 1;
                } else {
                    updated += 1;
                }
            }
            Err(_) => skipped += 1,
        }
    }

    let ts = now_iso();
    if let Some(inst) = get_connector_by_type(conn, workspace_id, provider)? {
        conn.execute(
            "UPDATE connectors SET status = 'connected', last_sync_at = ?1, updated_at = ?1, version = version + 1
             WHERE id = ?2",
            params![ts, inst.id],
        )?;
    }

    Ok(ConnectorSyncSummary {
        provider: provider.to_string(),
        created,
        updated,
        conflicts: 0,
        skipped,
        message: Some(format!(
            "Imported {created} new, updated {updated}, skipped {skipped}"
        )),
    })
}
