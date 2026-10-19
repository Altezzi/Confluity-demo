use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{new_id, now_iso, record_change, upsert_fts, DbError, DbResult};
use super::{get_meta, notify_workspace, set_meta};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsAlert {
    pub id: String,
    pub workspace_id: String,
    pub source: String,
    pub fingerprint: String,
    pub title: String,
    pub severity: Option<String>,
    pub external_url: Option<String>,
    pub raw_json: String,
    pub ticket_id: Option<String>,
    pub asset_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsConnectorSettings {
    pub datadog_api_key: Option<String>,
    pub datadog_site: Option<String>,
    pub grafana_url: Option<String>,
    pub cloudwatch_region: Option<String>,
    pub outbound_enabled: bool,
    pub default_company_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NormalizedAlert {
    pub source: String,
    pub fingerprint: String,
    pub title: String,
    pub severity: Option<String>,
    pub external_url: Option<String>,
    pub summary: String,
}

fn map_alert(row: &rusqlite::Row<'_>) -> rusqlite::Result<ObsAlert> {
    Ok(ObsAlert {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        source: row.get(2)?,
        fingerprint: row.get(3)?,
        title: row.get(4)?,
        severity: row.get(5)?,
        external_url: row.get(6)?,
        raw_json: row.get(7)?,
        ticket_id: row.get(8)?,
        asset_id: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
        deleted_at: row.get(12)?,
        version: row.get(13)?,
    })
}

const ALERT_COLS: &str = "id, workspace_id, source, fingerprint, title, severity, external_url, raw_json, ticket_id, asset_id, created_at, updated_at, deleted_at, version";

pub fn get_obs_settings(conn: &Connection) -> DbResult<ObsConnectorSettings> {
    Ok(ObsConnectorSettings {
        datadog_api_key: get_meta(conn, "obs_datadog_api_key")?,
        datadog_site: get_meta(conn, "obs_datadog_site")?,
        grafana_url: get_meta(conn, "obs_grafana_url")?,
        cloudwatch_region: get_meta(conn, "obs_cloudwatch_region")?,
        outbound_enabled: get_meta(conn, "obs_outbound_enabled")?.as_deref() == Some("true"),
        default_company_id: get_meta(conn, "obs_default_company_id")?,
    })
}

pub fn update_obs_settings(conn: &Connection, settings: &ObsConnectorSettings) -> DbResult<()> {
    set_meta(
        conn,
        "obs_datadog_api_key",
        settings.datadog_api_key.as_deref().unwrap_or(""),
    )?;
    set_meta(
        conn,
        "obs_datadog_site",
        settings.datadog_site.as_deref().unwrap_or("datadoghq.com"),
    )?;
    set_meta(
        conn,
        "obs_grafana_url",
        settings.grafana_url.as_deref().unwrap_or(""),
    )?;
    set_meta(
        conn,
        "obs_cloudwatch_region",
        settings.cloudwatch_region.as_deref().unwrap_or("us-east-1"),
    )?;
    set_meta(
        conn,
        "obs_outbound_enabled",
        if settings.outbound_enabled {
            "true"
        } else {
            "false"
        },
    )?;
    set_meta(
        conn,
        "obs_default_company_id",
        settings.default_company_id.as_deref().unwrap_or(""),
    )?;
    Ok(())
}

fn simple_hash(s: &str) -> String {
    let mut h: u64 = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    format!("{h:x}")
}

fn str_field(v: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(s) = v.get(*key).and_then(|x| x.as_str()) {
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }
    None
}

/// Best-effort normalize Datadog / Grafana / CloudWatch alert JSON.
pub fn normalize_alert(source: &str, raw: &str) -> Result<NormalizedAlert, String> {
    let source = source.to_lowercase();
    let mut root: Value =
        serde_json::from_str(raw).map_err(|e| format!("invalid JSON: {e}"))?;

    // CloudWatch SNS often wraps the alarm in Message
    if source == "cloudwatch" {
        if let Some(msg) = root.get("Message").and_then(|m| m.as_str()) {
            if let Ok(inner) = serde_json::from_str::<Value>(msg) {
                root = inner;
            }
        }
    }

    match source.as_str() {
        "datadog" => {
            let title = str_field(&root, &["title", "alert_title", "monitor_name", "event_title"])
                .or_else(|| {
                    root.get("msg_title")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "Datadog alert".into());
            let severity = str_field(&root, &["alert_type", "priority", "severity", "status"])
                .or_else(|| {
                    root.pointer("/monitor/priority")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                });
            let external_url = str_field(&root, &["link", "url", "permalink", "monitor_url"])
                .or_else(|| {
                    root.pointer("/monitor/url")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                });
            let id_part = str_field(&root, &["id", "alert_id", "monitor_id", "aggregation_key"])
                .unwrap_or_else(|| simple_hash(&title));
            let fingerprint = format!("datadog:{id_part}");
            let summary = str_field(&root, &["body", "text", "message", "alert_transition"])
                .unwrap_or_else(|| title.clone());
            Ok(NormalizedAlert {
                source: "datadog".into(),
                fingerprint,
                title,
                severity,
                external_url,
                summary,
            })
        }
        "grafana" => {
            let title = str_field(&root, &["title", "ruleName", "alertname"])
                .or_else(|| {
                    root.pointer("/alerts/0/labels/alertname")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "Grafana alert".into());
            let severity = str_field(&root, &["status", "state"])
                .or_else(|| {
                    root.pointer("/alerts/0/labels/severity")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                });
            let external_url = str_field(&root, &["externalURL", "generatorURL", "ruleUrl"])
                .or_else(|| {
                    root.pointer("/alerts/0/generatorURL")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .or_else(|| {
                    root.pointer("/alerts/0/dashboardURL")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                });
            let fp = str_field(&root, &["fingerprint", "ruleUID", "orgId"])
                .or_else(|| {
                    root.pointer("/alerts/0/fingerprint")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| simple_hash(&title));
            let summary = str_field(&root, &["message"])
                .or_else(|| {
                    root.pointer("/alerts/0/annotations/summary")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| title.clone());
            Ok(NormalizedAlert {
                source: "grafana".into(),
                fingerprint: format!("grafana:{fp}"),
                title,
                severity,
                external_url,
                summary,
            })
        }
        "cloudwatch" => {
            let title = str_field(&root, &["AlarmName", "alarmName"])
                .unwrap_or_else(|| "CloudWatch alarm".into());
            let severity = str_field(&root, &["NewStateValue", "StateValue", "newStateValue"]);
            let region = str_field(&root, &["Region", "region"]).unwrap_or_else(|| "us-east-1".into());
            let account = str_field(&root, &["AWSAccountId", "accountId"]).unwrap_or_default();
            let external_url = Some(format!(
                "https://{region}.console.aws.amazon.com/cloudwatch/home?region={region}#alarmsV2:alarm/{encoded}",
                encoded = urlencoding_encode(&title)
            ));
            let fingerprint = format!(
                "cloudwatch:{}:{}",
                account,
                str_field(&root, &["AlarmArn", "alarmArn"]).unwrap_or_else(|| title.clone())
            );
            let summary = str_field(&root, &["NewStateReason", "StateReason", "AlarmDescription"])
                .unwrap_or_else(|| title.clone());
            let _ = region;
            Ok(NormalizedAlert {
                source: "cloudwatch".into(),
                fingerprint,
                title,
                severity,
                external_url,
                summary,
            })
        }
        other => Err(format!("unsupported obs source: {other}")),
    }
}

fn urlencoding_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

pub fn find_alert_by_fingerprint(
    conn: &Connection,
    workspace_id: &str,
    source: &str,
    fingerprint: &str,
) -> DbResult<Option<ObsAlert>> {
    conn.query_row(
        &format!(
            "SELECT {ALERT_COLS} FROM obs_alerts
             WHERE workspace_id = ?1 AND source = ?2 AND fingerprint = ?3 AND deleted_at IS NULL"
        ),
        params![workspace_id, source, fingerprint],
        map_alert,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn get_alert(conn: &Connection, id: &str) -> DbResult<Option<ObsAlert>> {
    conn.query_row(
        &format!("SELECT {ALERT_COLS} FROM obs_alerts WHERE id = ?1 AND deleted_at IS NULL"),
        params![id],
        map_alert,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn list_alerts(conn: &Connection, workspace_id: &str) -> DbResult<Vec<ObsAlert>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {ALERT_COLS} FROM obs_alerts
         WHERE workspace_id = ?1 AND deleted_at IS NULL
         ORDER BY updated_at DESC LIMIT 100"
    ))?;
    let rows = stmt.query_map(params![workspace_id], map_alert)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn list_alerts_for_ticket(conn: &Connection, ticket_id: &str) -> DbResult<Vec<ObsAlert>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {ALERT_COLS} FROM obs_alerts
         WHERE ticket_id = ?1 AND deleted_at IS NULL
         ORDER BY updated_at DESC"
    ))?;
    let rows = stmt.query_map(params![ticket_id], map_alert)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

fn match_asset_id(conn: &Connection, workspace_id: &str, haystack: &str) -> Option<String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, serial, name FROM assets
             WHERE workspace_id = ?1 AND deleted_at IS NULL AND serial IS NOT NULL",
        )
        .ok()?;
    let rows = stmt
        .query_map(params![workspace_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .ok()?;
    let lower = haystack.to_lowercase();
    for row in rows.flatten() {
        let (id, serial, name) = row;
        if let Some(serial) = serial {
            if !serial.is_empty() && lower.contains(&serial.to_lowercase()) {
                return Some(id);
            }
        }
        if !name.is_empty() && lower.contains(&name.to_lowercase()) {
            return Some(id);
        }
    }
    None
}

fn default_company_id(conn: &Connection, workspace_id: &str, override_id: Option<&str>) -> DbResult<String> {
    if let Some(id) = override_id {
        if !id.is_empty() {
            return Ok(id.to_string());
        }
    }
    if let Some(id) = get_meta(conn, "obs_default_company_id")? {
        if !id.is_empty() {
            return Ok(id);
        }
    }
    conn.query_row(
        "SELECT id FROM companies WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY created_at LIMIT 1",
        params![workspace_id],
        |row| row.get(0),
    )
    .map_err(|_| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}

/// Import (or refresh) an alert and ensure a Support ticket exists.
pub fn import_alert(
    conn: &Connection,
    workspace_id: &str,
    source: &str,
    raw_json: &str,
    company_id: Option<&str>,
) -> DbResult<(ObsAlert, String)> {
    let normalized =
        normalize_alert(source, raw_json).map_err(|e| DbError::Sqlite(rusqlite::Error::InvalidParameterName(e)))?;
    let ts = now_iso();
    let company = default_company_id(conn, workspace_id, company_id)?;
    let asset_id = match_asset_id(
        conn,
        workspace_id,
        &format!("{} {}", normalized.title, normalized.summary),
    );
    let tags = format!("obs,{}", normalized.source);
    let description = format!(
        "Observability alert ({})\nSeverity: {}\n\n{}\n\nExternal: {}\n\n--- raw ---\n{}",
        normalized.source,
        normalized.severity.clone().unwrap_or_else(|| "unknown".into()),
        normalized.summary,
        normalized.external_url.clone().unwrap_or_else(|| "(none)".into()),
        raw_json
    );

    if let Some(existing) = find_alert_by_fingerprint(
        conn,
        workspace_id,
        &normalized.source,
        &normalized.fingerprint,
    )? {
        if let Some(ticket_id) = existing.ticket_id.clone() {
            conn.execute(
                "UPDATE tickets SET title = ?1, description = ?2, status = 'open', priority = ?3, tags = ?4, updated_at = ?5, version = version + 1
                 WHERE id = ?6 AND deleted_at IS NULL",
                params![
                    normalized.title,
                    description,
                    severity_to_priority(normalized.severity.as_deref()),
                    tags,
                    ts,
                    ticket_id
                ],
            )?;
            upsert_fts(conn, "ticket", &ticket_id, &normalized.title, &description)?;
            conn.execute(
                "UPDATE obs_alerts SET title = ?1, severity = ?2, external_url = ?3, raw_json = ?4, asset_id = COALESCE(?5, asset_id), updated_at = ?6, version = version + 1
                 WHERE id = ?7",
                params![
                    normalized.title,
                    normalized.severity,
                    normalized.external_url,
                    raw_json,
                    asset_id,
                    ts,
                    existing.id
                ],
            )?;
            record_change(
                conn,
                workspace_id,
                "update",
                "ticket",
                &ticket_id,
                &format!("Refreshed from {} alert", normalized.source),
            )?;
            let alert = get_alert(conn, &existing.id)?.ok_or_else(|| {
                DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows)
            })?;
            return Ok((alert, ticket_id));
        }
    }

    let ticket_id = new_id();
    conn.execute(
        "INSERT INTO tickets (id, workspace_id, company_id, contact_id, title, description, status, priority, tags, assignee_id, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, NULL, ?4, ?5, 'open', ?6, ?7, NULL, ?8, ?8, NULL, 1)",
        params![
            ticket_id,
            workspace_id,
            company,
            normalized.title,
            description,
            severity_to_priority(normalized.severity.as_deref()),
            tags,
            ts
        ],
    )?;
    upsert_fts(conn, "ticket", &ticket_id, &normalized.title, &description)?;

    let alert_id = new_id();
    conn.execute(
        "INSERT INTO obs_alerts (id, workspace_id, source, fingerprint, title, severity, external_url, raw_json, ticket_id, asset_id, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11, NULL, 1)",
        params![
            alert_id,
            workspace_id,
            normalized.source,
            normalized.fingerprint,
            normalized.title,
            normalized.severity,
            normalized.external_url,
            raw_json,
            ticket_id,
            asset_id,
            ts
        ],
    )?;

    record_change(
        conn,
        workspace_id,
        "create",
        "ticket",
        &ticket_id,
        &format!("Created ticket from {} alert", normalized.source),
    )?;
    let _ = notify_workspace(
        conn,
        workspace_id,
        None,
        &format!("{} alert", normalized.source),
        &normalized.title,
        Some("ticket"),
        Some(&ticket_id),
    );

    let alert = get_alert(conn, &alert_id)?.ok_or_else(|| {
        DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows)
    })?;
    Ok((alert, ticket_id))
}

fn severity_to_priority(sev: Option<&str>) -> &'static str {
    match sev.map(|s| s.to_lowercase()).as_deref() {
        Some("critical" | "p1" | "error" | "alerting" | "alarm") => "urgent",
        Some("warning" | "warn" | "p2" | "high") => "high",
        Some("info" | "ok" | "ok_" | "resolved" | "low") => "low",
        _ => "medium",
    }
}

/// Fire-and-forget Datadog event when outbound is enabled.
pub fn emit_datadog_event_blocking(
    api_key: &str,
    site: &str,
    title: &str,
    text: &str,
) -> Result<(), String> {
    let site = if site.is_empty() {
        "datadoghq.com"
    } else {
        site
    };
    let url = format!("https://api.{site}/api/v1/events");
    let body = serde_json::json!({
        "title": title,
        "text": text,
        "tags": ["source:confluity", "service:confluity"],
        "alert_type": "info",
    });
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(&url)
        .header("DD-API-KEY", api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().unwrap_or_default();
        return Err(format!("datadog event failed: {status} {text}"));
    }
    Ok(())
}
