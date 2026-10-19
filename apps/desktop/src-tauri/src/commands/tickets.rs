use serde::Deserialize;
use std::sync::Arc;

use crate::db::{
    models::{
        get_issue, get_ticket, list_ticket_ownership, list_tickets, log_ticket_ownership, Issue,
        Ticket, TicketOwnership,
    },
    new_id, now_iso, notify_workspace, record_change, upsert_fts, DbError,
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
pub struct CreateTicketInput {
    pub company_id: String,
    pub contact_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub tags: Option<String>,
    pub assignee_id: Option<String>,
    pub form_id: Option<String>,
    #[serde(default)]
    pub custom_fields: Option<std::collections::BTreeMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTicketInput {
    pub id: String,
    pub company_id: String,
    pub contact_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub tags: Option<String>,
    pub assignee_id: Option<String>,
    pub form_id: Option<String>,
    #[serde(default)]
    pub custom_fields: Option<std::collections::BTreeMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct EscalateTicketInput {
    pub ticket_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub blocked_reason: Option<String>,
}

#[tauri::command]
pub fn list_tickets_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    company_id: Option<String>,
) -> Result<Vec<Ticket>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_tickets(&conn, &ws, company_id.as_deref()).map_err(map_err)
}

#[tauri::command]
pub fn get_ticket_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<Option<Ticket>, String> {
    let conn = state.conn.lock();
    get_ticket(&conn, &id).map_err(map_err)
}

#[tauri::command]
pub fn create_ticket_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateTicketInput,
) -> Result<Ticket, String> {
    crate::commands::roles::require_perm(&state, "module.support.edit")?;
    let ws = workspace_id(&state)?;
    let id = new_id();
    let ts = now_iso();
    let status = input.status.unwrap_or_else(|| "open".into());
    let priority = input.priority.unwrap_or_else(|| "medium".into());
    let custom = input.custom_fields.clone().unwrap_or_default();
    let custom_json = serde_json::to_string(&custom).unwrap_or_else(|_| "{}".into());
    let conn = state.conn.lock();
    crate::db::ticket_forms::validate_ticket_submission(
        &conn,
        &ws,
        input.form_id.as_deref(),
        &input.title,
        &input.company_id,
        input.contact_id.as_deref(),
        input.description.as_deref(),
        input.tags.as_deref(),
        input.assignee_id.as_deref(),
        &custom,
    )?;
    let form_id = input.form_id.clone().filter(|id| !id.is_empty()).or_else(|| {
        crate::db::ticket_forms::list_ticket_forms(&conn, &ws)
            .ok()
            .and_then(|forms| {
                forms
                    .iter()
                    .find(|form| form.is_default)
                    .or_else(|| forms.first())
                    .map(|form| form.id.clone())
            })
    });
    conn.execute(
        "INSERT INTO tickets (id, workspace_id, company_id, contact_id, title, description, status, priority, tags, assignee_id, form_id, custom_fields, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13, NULL, 1)",
        rusqlite::params![
            id,
            ws,
            input.company_id,
            input.contact_id,
            input.title,
            input.description,
            status,
            priority,
            input.tags,
            input.assignee_id,
            form_id,
            custom_json,
            ts
        ],
    )
    .map_err(|e| e.to_string())?;
    let search_body = format!(
        "{} {}",
        input.description.as_deref().unwrap_or(""),
        custom.values().cloned().collect::<Vec<_>>().join(" ")
    );
    upsert_fts(&conn, "ticket", &id, &input.title, &search_body)
    .map_err(map_err)?;
    record_change(
        &conn,
        &ws,
        "create",
        "ticket",
        &id,
        &format!("Created ticket {}", input.title),
    )
    .map_err(map_err)?;
    log_ticket_ownership(
        &conn,
        &ws,
        &id,
        input.assignee_id.as_deref(),
        None,
        Some(&ts),
        false,
    )
    .map_err(map_err)?;
    get_ticket(&conn, &id)
        .map_err(map_err)?
        .ok_or_else(|| "failed to load ticket".into())
}

#[tauri::command]
pub fn update_ticket_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: UpdateTicketInput,
) -> Result<Ticket, String> {
    crate::commands::roles::require_perm(&state, "module.support.edit")?;
    let ws = workspace_id(&state)?;
    let ts = now_iso();
    let conn = state.conn.lock();
    let previous = get_ticket(&conn, &input.id)
        .map_err(map_err)?
        .ok_or_else(|| "ticket not found".to_string())?;
    let custom_json = input
        .custom_fields
        .as_ref()
        .map(|fields| serde_json::to_string(fields).unwrap_or_else(|_| "{}".into()));
    if let Some(custom) = input.custom_fields.as_ref() {
        crate::db::ticket_forms::validate_ticket_submission(
            &conn,
            &ws,
            input.form_id.as_deref().or(previous.form_id.as_deref()),
            &input.title,
            &input.company_id,
            input.contact_id.as_deref(),
            input.description.as_deref(),
            input.tags.as_deref(),
            input.assignee_id.as_deref(),
            custom,
        )?;
    }
    conn.execute(
        "UPDATE tickets SET company_id = ?1, contact_id = ?2, title = ?3, description = ?4, status = ?5, priority = ?6, tags = ?7, assignee_id = ?8, form_id = COALESCE(?9, form_id), custom_fields = COALESCE(?10, custom_fields), updated_at = ?11, version = version + 1
         WHERE id = ?12 AND deleted_at IS NULL",
        rusqlite::params![
            input.company_id,
            input.contact_id,
            input.title,
            input.description,
            input.status,
            input.priority,
            input.tags,
            input.assignee_id,
            input.form_id,
            custom_json,
            ts,
            input.id
        ],
    )
    .map_err(|e| e.to_string())?;
    let search_extra = input
        .custom_fields
        .as_ref()
        .map(|fields| fields.values().cloned().collect::<Vec<_>>().join(" "))
        .unwrap_or_default();
    let search_body = format!(
        "{} {}",
        input.description.as_deref().unwrap_or(""),
        search_extra
    );
    upsert_fts(&conn, "ticket", &input.id, &input.title, &search_body)
    .map_err(map_err)?;
    record_change(
        &conn,
        &ws,
        "update",
        "ticket",
        &input.id,
        &format!("Updated ticket {}", input.title),
    )
    .map_err(map_err)?;
    if previous.assignee_id != input.assignee_id {
        log_ticket_ownership(
            &conn,
            &ws,
            &input.id,
            input.assignee_id.as_deref(),
            previous.assignee_id.as_deref(),
            Some(&ts),
            true,
        )
        .map_err(map_err)?;
        if let Some(assignee_id) = input.assignee_id.as_deref() {
            let _ = notify_workspace(
                &conn,
                &ws,
                Some(assignee_id),
                "Ticket assigned to you",
                &input.title,
                Some("ticket"),
                Some(&input.id),
            );
        }
    }
    let ticket = get_ticket(&conn, &input.id)
        .map_err(map_err)?
        .ok_or_else(|| "ticket not found".to_string())?;
    drop(conn);
    if previous.status != input.status {
        crate::commands::obs::maybe_emit_outbound_for_ticket(&state, &input.id, &input.status);
    }
    Ok(ticket)
}

#[tauri::command]
pub fn list_ticket_ownership_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    ticket_id: String,
) -> Result<Vec<TicketOwnership>, String> {
    let conn = state.conn.lock();
    list_ticket_ownership(&conn, &ticket_id).map_err(map_err)
}

#[tauri::command]
pub fn escalate_ticket_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: EscalateTicketInput,
) -> Result<Issue, String> {
    crate::commands::roles::require_perm(&state, "module.support.edit")?;
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    let ticket = get_ticket(&conn, &input.ticket_id)
        .map_err(map_err)?
        .ok_or_else(|| "ticket not found".to_string())?;

    // If an issue already exists for this ticket, return it.
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM issues WHERE ticket_id = ?1 AND deleted_at IS NULL LIMIT 1",
            rusqlite::params![ticket.id],
            |row| row.get(0),
        )
        .ok();
    if let Some(issue_id) = existing {
        return get_issue(&conn, &issue_id)
            .map_err(map_err)?
            .ok_or_else(|| "issue not found".to_string());
    }

    let id = new_id();
    let ts = now_iso();
    let title = input
        .title
        .unwrap_or_else(|| format!("Engineering: {}", ticket.title));
    let description = input.description.or(ticket.description.clone());
    let priority = input.priority.unwrap_or_else(|| match ticket.priority.as_str() {
        "urgent" => "critical".into(),
        "high" => "high".into(),
        "low" => "low".into(),
        _ => "medium".into(),
    });
    let blocked_reason = input.blocked_reason;

    conn.execute(
        "INSERT INTO issues (id, workspace_id, ticket_id, company_id, project_id, title, description, status, priority, blocked_reason, assignee_id, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6, 'todo', ?7, ?8, NULL, ?9, ?9, NULL, 1)",
        rusqlite::params![
            id,
            ws,
            ticket.id,
            ticket.company_id,
            title,
            description,
            priority,
            blocked_reason,
            ts
        ],
    )
    .map_err(|e| e.to_string())?;

    let fts_body = format!(
        "{} {}",
        description.clone().unwrap_or_default(),
        blocked_reason.clone().unwrap_or_default()
    );
    upsert_fts(&conn, "issue", &id, &title, &fts_body).map_err(map_err)?;
    record_change(
        &conn,
        &ws,
        "escalate",
        "issue",
        &id,
        &format!("Escalated ticket {} into issue {}", ticket.title, title),
    )
    .map_err(map_err)?;

    get_issue(&conn, &id)
        .map_err(map_err)?
        .ok_or_else(|| "failed to load issue".into())
}
