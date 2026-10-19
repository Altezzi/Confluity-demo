use serde::Deserialize;
use std::sync::Arc;

use crate::db::{
    extended::{
        get_attachment, list_attachments, list_audit, list_events, list_notifications, list_outbox,
        workspace_preview, Attachment, AuditEntry, Notification, OutboxItem, SyncStatus,
        WorkspaceEvent, WorkspacePreview,
    },
    get_meta, models::list_members, new_id, now_iso, notify_workspace, record_change, set_meta,
    upsert_fts, DbError,
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

#[derive(Debug, Deserialize)]
pub struct AddAttachmentInput {
    pub entity_type: String,
    pub entity_id: String,
    pub filename: String,
    pub mime: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberInput {
    pub id: String,
    pub name: String,
    pub role: String,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMemberInput {
    pub name: String,
    pub role: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectServerInput {
    pub server_url: String,
    pub mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EmailToTicketInput {
    pub company_id: String,
    pub contact_id: Option<String>,
    pub from_email: Option<String>,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct CsvImportInput {
    pub csv: String,
}

#[derive(Debug, Deserialize)]
pub struct ContactCsvImportInput {
    pub company_id: String,
    pub csv: String,
}

#[tauri::command]
pub fn list_attachments_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    entity_type: String,
    entity_id: String,
) -> Result<Vec<Attachment>, String> {
    let conn = state.conn.lock();
    list_attachments(&conn, &entity_type, &entity_id).map_err(map_err)
}

#[tauri::command]
pub async fn add_attachment_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: AddAttachmentInput,
) -> Result<Attachment, String> {
    crate::commands::roles::require_perm(&state, "panel.attachments")?;
    let ws = workspace_id(&state)?;
    let id = new_id();
    let ts = now_iso();
    let stored = format!("{id}-{}", input.filename.replace(['/', '\\'], "_"));
    let dest = state.paths.attachments.join(&stored);
    std::fs::write(&dest, &input.bytes).map_err(|e| e.to_string())?;
    let attachment = {
        let conn = state.conn.lock();
        conn.execute(
            "INSERT INTO attachments (id, workspace_id, entity_type, entity_id, filename, stored_name, mime, size_bytes, created_at, updated_at, deleted_at, version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9, NULL, 1)",
            rusqlite::params![
                id,
                ws,
                input.entity_type,
                input.entity_id,
                input.filename,
                stored,
                input.mime,
                input.bytes.len() as i64,
                ts
            ],
        )
        .map_err(|e| e.to_string())?;
        record_change(
            &conn,
            &ws,
            "create",
            "attachment",
            &id,
            &format!("Attached {}", input.filename),
        )
        .map_err(map_err)?;
        get_attachment(&conn, &id)
            .map_err(map_err)?
            .ok_or_else(|| "failed to load attachment".to_string())?
    };
    if let Err(err) = crate::commands::cloud_sync::upload_attachment_if_remote(
        &state,
        &ws,
        &stored,
        input.mime.as_deref(),
        &input.bytes,
    )
    .await
    {
        let conn = state.conn.lock();
        let _ = conn.execute("DELETE FROM attachments WHERE id = ?1", rusqlite::params![id]);
        drop(conn);
        let _ = std::fs::remove_file(&dest);
        return Err(err);
    }
    Ok(attachment)
}

#[tauri::command]
pub fn list_notifications_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<Notification>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_notifications(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn mark_notification_read_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    let ts = now_iso();
    let conn = state.conn.lock();
    conn.execute(
        "UPDATE notifications SET read_at = ?1 WHERE id = ?2",
        rusqlite::params![ts, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn mark_all_notifications_read_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let ws = workspace_id(&state)?;
    let ts = now_iso();
    let conn = state.conn.lock();
    conn.execute(
        "UPDATE notifications SET read_at = ?1 WHERE workspace_id = ?2 AND read_at IS NULL",
        rusqlite::params![ts, ws],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_audit_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    limit: Option<i64>,
) -> Result<Vec<AuditEntry>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_audit(&conn, &ws, limit.unwrap_or(40)).map_err(map_err)
}

#[tauri::command]
pub fn list_outbox_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<Vec<OutboxItem>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_outbox(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn list_events_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<WorkspaceEvent>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_events(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn get_sync_status_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<SyncStatus, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    let queued: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM outbox WHERE workspace_id = ?1 AND status = 'queued'",
            rusqlite::params![ws],
            |row| row.get(0),
        )
        .unwrap_or(0);
    Ok(SyncStatus {
        server_url: get_meta(&conn, "server_url").map_err(map_err)?,
        server_status: get_meta(&conn, "server_status")
            .map_err(map_err)?
            .unwrap_or_else(|| "disconnected".into()),
        last_sync: get_meta(&conn, "last_sync").map_err(map_err)?,
        queued_changes: queued,
    })
}

#[tauri::command]
pub fn connect_server_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: ConnectServerInput,
) -> Result<SyncStatus, String> {
    crate::commands::roles::require_perm(&state, "settings.data_server")?;
    let url = input.server_url.trim();
    if url.is_empty() || (!url.starts_with("http://") && !url.starts_with("https://")) {
        return Err("Enter a valid http or https server URL.".into());
    }
    let mode = input.mode.unwrap_or_else(|| "hybrid".into());
    {
        let conn = state.conn.lock();
        set_meta(&conn, "server_url", url).map_err(map_err)?;
        set_meta(&conn, "server_status", "connected").map_err(map_err)?;
        set_meta(&conn, "storage_mode", if mode == "server" { "server" } else { "hybrid" })
            .map_err(map_err)?;
    }
    get_sync_status_cmd(state)
}

#[tauri::command]
pub fn disconnect_server_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<SyncStatus, String> {
    {
        let conn = state.conn.lock();
        set_meta(&conn, "server_status", "disconnected").map_err(map_err)?;
        set_meta(&conn, "storage_mode", "local").map_err(map_err)?;
    }
    get_sync_status_cmd(state)
}

#[tauri::command]
pub async fn sync_now_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<SyncStatus, String> {
    let ws = workspace_id(&state)?;
    let supabase = {
        let conn = state.conn.lock();
        let status = get_meta(&conn, "server_status")
            .map_err(map_err)?
            .unwrap_or_else(|| "disconnected".into());
        if status != "connected" {
            return Err("Connect a server before syncing.".into());
        }
        get_meta(&conn, "identity_provider")
            .map_err(map_err)?
            .as_deref()
            == Some("supabase")
    };
    let ts = now_iso();
    if supabase {
        crate::commands::cloud_sync::push_workspace_records(&state).await?;
    }
    {
        let conn = state.conn.lock();
        conn.execute(
            "UPDATE outbox SET status = 'sent', sent_at = ?1 WHERE workspace_id = ?2 AND status = 'queued'",
            rusqlite::params![ts, ws],
        )
        .map_err(|e| e.to_string())?;
        set_meta(&conn, "last_sync", &ts).map_err(map_err)?;
        set_meta(&conn, "last_sync_error", "").map_err(map_err)?;
    }
    get_sync_status_cmd(state)
}

#[tauri::command]
pub fn workspace_preview_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<WorkspacePreview, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    workspace_preview(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn create_member_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateMemberInput,
) -> Result<crate::db::models::Member, String> {
    crate::commands::roles::require_perm(&state, "settings.manage_members")?;
    let ws = workspace_id(&state)?;
    let id = new_id();
    let ts = now_iso();
    let role = input.role.unwrap_or_else(|| "support".into());
    let conn = state.conn.lock();
    conn.execute(
        "INSERT INTO members (id, workspace_id, name, role, title, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, NULL, 1)",
        rusqlite::params![id, ws, input.name, role, input.title, ts],
    )
    .map_err(|e| e.to_string())?;
    // Assign matching catalog role when present
    if let Ok(Some(catalog_role)) = crate::db::roles::find_role_by_name(&conn, &ws, &role) {
        let _ = crate::db::roles::set_member_roles(&conn, &id, &[catalog_role.id]);
    } else if let Ok(Some(support)) = crate::db::roles::find_role_by_name(&conn, &ws, "Support") {
        let _ = crate::db::roles::set_member_roles(&conn, &id, &[support.id]);
    }
    record_change(&conn, &ws, "create", "member", &id, &format!("Added member {}", input.name))
        .map_err(map_err)?;
    list_members(&conn, &ws)
        .map_err(map_err)?
        .into_iter()
        .find(|m| m.id == id)
        .ok_or_else(|| "failed to load member".into())
}

#[tauri::command]
pub fn update_member_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: UpdateMemberInput,
) -> Result<crate::db::models::Member, String> {
    crate::commands::roles::require_perm(&state, "settings.manage_members")?;
    let ws = workspace_id(&state)?;
    let ts = now_iso();
    let conn = state.conn.lock();
    conn.execute(
        "UPDATE members SET name = ?1, role = ?2, title = ?3, updated_at = ?4, version = version + 1
         WHERE id = ?5 AND deleted_at IS NULL",
        rusqlite::params![input.name, input.role, input.title, ts, input.id],
    )
    .map_err(|e| e.to_string())?;
    record_change(&conn, &ws, "update", "member", &input.id, &format!("Updated member {}", input.name))
        .map_err(map_err)?;
    list_members(&conn, &ws)
        .map_err(map_err)?
        .into_iter()
        .find(|m| m.id == input.id)
        .ok_or_else(|| "member not found".into())
}

#[tauri::command]
pub fn email_to_ticket_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: EmailToTicketInput,
) -> Result<crate::db::models::Ticket, String> {
    crate::commands::roles::require_perm(&state, "settings.integrations")?;
    let ws = workspace_id(&state)?;
    let id = new_id();
    let ts = now_iso();
    let title = if input.subject.trim().is_empty() {
        "Email ticket".into()
    } else {
        input.subject.trim().to_string()
    };
    let body = format!(
        "From: {}\n\n{}",
        input.from_email.clone().unwrap_or_else(|| "unknown".into()),
        input.body
    );
    let conn = state.conn.lock();
    conn.execute(
        "INSERT INTO tickets (id, workspace_id, company_id, contact_id, title, description, status, priority, tags, assignee_id, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'open', 'medium', 'email', NULL, ?7, ?7, NULL, 1)",
        rusqlite::params![id, ws, input.company_id, input.contact_id, title, body, ts],
    )
    .map_err(|e| e.to_string())?;
    upsert_fts(&conn, "ticket", &id, &title, &body).map_err(map_err)?;
    record_change(&conn, &ws, "create", "ticket", &id, "Created ticket from email").map_err(map_err)?;
    notify_workspace(
        &conn,
        &ws,
        None,
        "Email became a ticket",
        &title,
        Some("ticket"),
        Some(&id),
    )
    .map_err(map_err)?;
    crate::db::models::get_ticket(&conn, &id)
        .map_err(map_err)?
        .ok_or_else(|| "failed to load ticket".into())
}

#[tauri::command]
pub fn import_companies_csv_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CsvImportInput,
) -> Result<i64, String> {
    crate::commands::roles::require_perm(&state, "settings.integrations")?;
    let ws = workspace_id(&state)?;
    let ts = now_iso();
    let mut created = 0i64;
    let conn = state.conn.lock();
    for (i, line) in input.csv.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || (i == 0 && line.to_lowercase().starts_with("name")) {
            continue;
        }
        let mut cols = line.split(',').map(|s| s.trim().trim_matches('"'));
        let Some(name) = cols.next().filter(|s| !s.is_empty()) else {
            continue;
        };
        let domain = cols.next().filter(|s| !s.is_empty());
        let industry = cols.next().filter(|s| !s.is_empty());
        let id = new_id();
        conn.execute(
            "INSERT INTO companies (id, workspace_id, name, domain, industry, notes, created_at, updated_at, deleted_at, version)
             VALUES (?1, ?2, ?3, ?4, ?5, 'Imported from CSV', ?6, ?6, NULL, 1)",
            rusqlite::params![id, ws, name, domain, industry, ts],
        )
        .map_err(|e| e.to_string())?;
        upsert_fts(&conn, "company", &id, name, &format!("{} {}", domain.unwrap_or(""), industry.unwrap_or("")))
            .map_err(map_err)?;
        created += 1;
    }
    Ok(created)
}

#[tauri::command]
pub fn import_contacts_csv_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: ContactCsvImportInput,
) -> Result<i64, String> {
    crate::commands::roles::require_perm(&state, "settings.integrations")?;
    let ws = workspace_id(&state)?;
    let ts = now_iso();
    let mut created = 0i64;
    let conn = state.conn.lock();
    for (i, line) in input.csv.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || (i == 0 && line.to_lowercase().starts_with("name")) {
            continue;
        }
        let mut cols = line.split(',').map(|s| s.trim().trim_matches('"'));
        let Some(name) = cols.next().filter(|s| !s.is_empty()) else {
            continue;
        };
        let email = cols.next().filter(|s| !s.is_empty());
        let title = cols.next().filter(|s| !s.is_empty());
        let id = new_id();
        conn.execute(
            "INSERT INTO contacts (id, workspace_id, company_id, name, email, title, phone, notes, created_at, updated_at, deleted_at, version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, 'Imported from CSV', ?7, ?7, NULL, 1)",
            rusqlite::params![id, ws, input.company_id, name, email, title, ts],
        )
        .map_err(|e| e.to_string())?;
        upsert_fts(
            &conn,
            "contact",
            &id,
            name,
            &format!("{} {}", email.unwrap_or(""), title.unwrap_or("")),
        )
        .map_err(map_err)?;
        created += 1;
    }
    Ok(created)
}

#[tauri::command]
pub fn notify_workspace_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    title: String,
    body: Option<String>,
    entity_type: Option<String>,
    entity_id: Option<String>,
) -> Result<(), String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    notify_workspace(
        &conn,
        &ws,
        None,
        &title,
        body.as_deref().unwrap_or(""),
        entity_type.as_deref(),
        entity_id.as_deref(),
    )
    .map_err(map_err)
}
