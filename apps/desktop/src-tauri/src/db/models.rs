use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use super::{DbError, DbResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub domain: Option<String>,
    pub industry: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    pub workspace_id: String,
    pub company_id: String,
    pub name: String,
    pub email: Option<String>,
    pub title: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: String,
    pub workspace_id: String,
    pub company_id: String,
    pub contact_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub tags: Option<String>,
    pub assignee_id: Option<String>,
    #[serde(default)]
    pub form_id: Option<String>,
    #[serde(default)]
    pub custom_fields: std::collections::BTreeMap<String, String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub workspace_id: String,
    pub ticket_id: Option<String>,
    pub company_id: Option<String>,
    pub project_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub blocked_reason: Option<String>,
    pub assignee_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub role: String,
    pub title: Option<String>,
    pub position_id: Option<String>,
    #[serde(default)]
    pub role_ids: Vec<String>,
    pub email: Option<String>,
    pub must_set_password: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invite_code: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketOwnership {
    pub id: String,
    pub workspace_id: String,
    pub ticket_id: String,
    pub assignee_id: Option<String>,
    pub assignee_name: String,
    pub previous_assignee_id: Option<String>,
    pub previous_assignee_name: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub workspace_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub author: String,
    pub body: String,
    pub visibility: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub workspace_id: String,
    pub title: String,
    pub body: String,
    pub category: Option<String>,
    pub company_id: Option<String>,
    pub ticket_id: Option<String>,
    pub issue_id: Option<String>,
    pub visibility: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub workspace_id: String,
    pub actor: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub summary: String,
    pub metadata: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConnection {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub api_key_configured: bool,
    pub enabled: bool,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConnection {
    pub id: String,
    pub name: String,
    pub transport: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub url: Option<String>,
    pub auth_token_configured: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub storage_mode: String,
    pub server_url: Option<String>,
    pub workspace_id: String,
    pub workspace_name: String,
    pub organization_id: String,
    pub organization_name: String,
    pub ai_api_base_url: Option<String>,
    pub ai_model: Option<String>,
    pub ai_api_key_configured: bool,
    pub seeded: bool,
    pub current_member_id: Option<String>,
    pub server_status: String,
    pub last_sync: Option<String>,
    pub queued_changes: i64,
    pub setup_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub entity_type: String,
    pub entity_id: String,
    pub title: String,
    pub snippet: String,
    pub rank: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedRecords {
    pub company: Option<Company>,
    pub contacts: Vec<Contact>,
    pub tickets: Vec<Ticket>,
    pub issues: Vec<Issue>,
    pub documents: Vec<Document>,
    pub products: Vec<crate::db::extended::Product>,
    pub assets: Vec<crate::db::extended::Asset>,
    pub rmas: Vec<crate::db::extended::Rma>,
    pub orders: Vec<crate::db::extended::Order>,
    pub invoices: Vec<crate::db::extended::Invoice>,
}

fn map_company(row: &Row<'_>) -> rusqlite::Result<Company> {
    Ok(Company {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        name: row.get(2)?,
        domain: row.get(3)?,
        industry: row.get(4)?,
        notes: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        deleted_at: row.get(8)?,
        version: row.get(9)?,
    })
}

fn map_contact(row: &Row<'_>) -> rusqlite::Result<Contact> {
    Ok(Contact {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        company_id: row.get(2)?,
        name: row.get(3)?,
        email: row.get(4)?,
        title: row.get(5)?,
        phone: row.get(6)?,
        notes: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        deleted_at: row.get(10)?,
        version: row.get(11)?,
    })
}

fn map_ticket(row: &Row<'_>) -> rusqlite::Result<Ticket> {
    Ok(Ticket {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        company_id: row.get(2)?,
        contact_id: row.get(3)?,
        title: row.get(4)?,
        description: row.get(5)?,
        status: row.get(6)?,
        priority: row.get(7)?,
        tags: row.get(8)?,
        assignee_id: row.get(9)?,
        form_id: row.get(14)?,
        custom_fields: parse_custom_fields(row.get(15)?),
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
        deleted_at: row.get(12)?,
        version: row.get(13)?,
    })
}

fn parse_custom_fields(raw: Option<String>) -> std::collections::BTreeMap<String, String> {
    let Some(raw) = raw.filter(|value| !value.trim().is_empty()) else {
        return std::collections::BTreeMap::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return std::collections::BTreeMap::new();
    };
    let Some(object) = value.as_object() else {
        return std::collections::BTreeMap::new();
    };
    object
        .iter()
        .map(|(key, item)| {
            let text = match item {
                serde_json::Value::String(text) => text.clone(),
                serde_json::Value::Bool(flag) => {
                    if *flag {
                        "true".into()
                    } else {
                        "false".into()
                    }
                }
                serde_json::Value::Number(number) => number.to_string(),
                serde_json::Value::Null => String::new(),
                other => other.to_string(),
            };
            (key.clone(), text)
        })
        .collect()
}

fn map_issue(row: &Row<'_>) -> rusqlite::Result<Issue> {
    Ok(Issue {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        ticket_id: row.get(2)?,
        company_id: row.get(3)?,
        project_id: row.get(4)?,
        title: row.get(5)?,
        description: row.get(6)?,
        status: row.get(7)?,
        priority: row.get(8)?,
        blocked_reason: row.get(9)?,
        assignee_id: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
        deleted_at: row.get(13)?,
        version: row.get(14)?,
    })
}

fn map_member(row: &Row<'_>) -> rusqlite::Result<Member> {
    Ok(Member {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        name: row.get(2)?,
        role: row.get(3)?,
        title: row.get(4)?,
        position_id: row.get(5)?,
        role_ids: Vec::new(),
        email: row.get(10)?,
        must_set_password: row.get::<_, i64>(11)? != 0,
        invite_code: None,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        deleted_at: row.get(8)?,
        version: row.get(9)?,
    })
}

fn map_project(row: &Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        status: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
        deleted_at: row.get(7)?,
        version: row.get(8)?,
    })
}

fn map_comment(row: &Row<'_>) -> rusqlite::Result<Comment> {
    Ok(Comment {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        entity_type: row.get(2)?,
        entity_id: row.get(3)?,
        author: row.get(4)?,
        body: row.get(5)?,
        visibility: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
        deleted_at: row.get(9)?,
        version: row.get(10)?,
    })
}

fn map_document(row: &Row<'_>) -> rusqlite::Result<Document> {
    Ok(Document {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        title: row.get(2)?,
        body: row.get(3)?,
        category: row.get(4)?,
        company_id: row.get(5)?,
        ticket_id: row.get(6)?,
        issue_id: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        deleted_at: row.get(10)?,
        version: row.get(11)?,
        visibility: row.get(12).ok(),
    })
}

fn map_activity(row: &Row<'_>) -> rusqlite::Result<Activity> {
    Ok(Activity {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        actor: row.get(2)?,
        action: row.get(3)?,
        entity_type: row.get(4)?,
        entity_id: row.get(5)?,
        summary: row.get(6)?,
        metadata: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        deleted_at: row.get(10)?,
        version: row.get(11)?,
    })
}

fn map_workspace(row: &Row<'_>) -> rusqlite::Result<Workspace> {
    Ok(Workspace {
        id: row.get(0)?,
        organization_id: row.get(1)?,
        name: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
        deleted_at: row.get(5)?,
        version: row.get(6)?,
    })
}

pub fn get_company(conn: &Connection, id: &str) -> DbResult<Option<Company>> {
    conn.query_row(
        "SELECT id, workspace_id, name, domain, industry, notes, created_at, updated_at, deleted_at, version
         FROM companies WHERE id = ?1 AND deleted_at IS NULL",
        params![id],
        map_company,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn list_companies(conn: &Connection, workspace_id: &str) -> DbResult<Vec<Company>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, name, domain, industry, notes, created_at, updated_at, deleted_at, version
         FROM companies WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map(params![workspace_id], map_company)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_contact(conn: &Connection, id: &str) -> DbResult<Option<Contact>> {
    conn.query_row(
        "SELECT id, workspace_id, company_id, name, email, title, phone, notes, created_at, updated_at, deleted_at, version
         FROM contacts WHERE id = ?1 AND deleted_at IS NULL",
        params![id],
        map_contact,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn list_contacts(
    conn: &Connection,
    workspace_id: &str,
    company_id: Option<&str>,
) -> DbResult<Vec<Contact>> {
    if let Some(company_id) = company_id {
        let mut stmt = conn.prepare(
            "SELECT id, workspace_id, company_id, name, email, title, phone, notes, created_at, updated_at, deleted_at, version
             FROM contacts WHERE workspace_id = ?1 AND company_id = ?2 AND deleted_at IS NULL ORDER BY name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map(params![workspace_id, company_id], map_contact)?;
        return Ok(rows.filter_map(|r| r.ok()).collect());
    }
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, company_id, name, email, title, phone, notes, created_at, updated_at, deleted_at, version
         FROM contacts WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map(params![workspace_id], map_contact)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_ticket(conn: &Connection, id: &str) -> DbResult<Option<Ticket>> {
    conn.query_row(
        "SELECT id, workspace_id, company_id, contact_id, title, description, status, priority, tags, assignee_id, created_at, updated_at, deleted_at, version, form_id, custom_fields
         FROM tickets WHERE id = ?1 AND deleted_at IS NULL",
        params![id],
        map_ticket,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn list_tickets(
    conn: &Connection,
    workspace_id: &str,
    company_id: Option<&str>,
) -> DbResult<Vec<Ticket>> {
    if let Some(company_id) = company_id {
        let mut stmt = conn.prepare(
            "SELECT id, workspace_id, company_id, contact_id, title, description, status, priority, tags, assignee_id, created_at, updated_at, deleted_at, version, form_id, custom_fields
             FROM tickets WHERE workspace_id = ?1 AND company_id = ?2 AND deleted_at IS NULL ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map(params![workspace_id, company_id], map_ticket)?;
        return Ok(rows.filter_map(|r| r.ok()).collect());
    }
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, company_id, contact_id, title, description, status, priority, tags, assignee_id, created_at, updated_at, deleted_at, version, form_id, custom_fields
         FROM tickets WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map(params![workspace_id], map_ticket)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_issue(conn: &Connection, id: &str) -> DbResult<Option<Issue>> {
    conn.query_row(
        "SELECT id, workspace_id, ticket_id, company_id, project_id, title, description, status, priority, blocked_reason, assignee_id, created_at, updated_at, deleted_at, version
         FROM issues WHERE id = ?1 AND deleted_at IS NULL",
        params![id],
        map_issue,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn list_issues(
    conn: &Connection,
    workspace_id: &str,
    project_id: Option<&str>,
) -> DbResult<Vec<Issue>> {
    if let Some(project_id) = project_id {
        let mut stmt = conn.prepare(
            "SELECT id, workspace_id, ticket_id, company_id, project_id, title, description, status, priority, blocked_reason, assignee_id, created_at, updated_at, deleted_at, version
             FROM issues WHERE workspace_id = ?1 AND project_id = ?2 AND deleted_at IS NULL ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map(params![workspace_id, project_id], map_issue)?;
        return Ok(rows.filter_map(|r| r.ok()).collect());
    }
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, ticket_id, company_id, project_id, title, description, status, priority, blocked_reason, assignee_id, created_at, updated_at, deleted_at, version
         FROM issues WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map(params![workspace_id], map_issue)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn list_members(conn: &Connection, workspace_id: &str) -> DbResult<Vec<Member>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, name, role, title, position_id, created_at, updated_at, deleted_at, version, email, must_set_password
         FROM members WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map(params![workspace_id], map_member)?;
    let mut members: Vec<Member> = rows.filter_map(|r| r.ok()).collect();
    for member in &mut members {
        member.role_ids = super::roles::member_role_ids(conn, &member.id).unwrap_or_default();
    }
    Ok(members)
}

pub fn get_project(conn: &Connection, id: &str) -> DbResult<Option<Project>> {
    conn.query_row(
        "SELECT id, workspace_id, name, description, status, created_at, updated_at, deleted_at, version
         FROM projects WHERE id = ?1 AND deleted_at IS NULL",
        params![id],
        map_project,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn list_projects(conn: &Connection, workspace_id: &str) -> DbResult<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, name, description, status, created_at, updated_at, deleted_at, version
         FROM projects WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map(params![workspace_id], map_project)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn list_comments(
    conn: &Connection,
    workspace_id: &str,
    entity_type: &str,
    entity_id: &str,
) -> DbResult<Vec<Comment>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, entity_type, entity_id, author, body, visibility, created_at, updated_at, deleted_at, version
         FROM comments WHERE workspace_id = ?1 AND entity_type = ?2 AND entity_id = ?3 AND deleted_at IS NULL ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map(params![workspace_id, entity_type, entity_id], map_comment)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_comment(conn: &Connection, id: &str) -> DbResult<Option<Comment>> {
    conn.query_row(
        "SELECT id, workspace_id, entity_type, entity_id, author, body, visibility, created_at, updated_at, deleted_at, version
         FROM comments WHERE id = ?1 AND deleted_at IS NULL",
        params![id],
        map_comment,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn get_document(conn: &Connection, id: &str) -> DbResult<Option<Document>> {
    conn.query_row(
        "SELECT id, workspace_id, title, body, category, company_id, ticket_id, issue_id, created_at, updated_at, deleted_at, version, visibility
         FROM documents WHERE id = ?1 AND deleted_at IS NULL",
        params![id],
        map_document,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn list_documents(conn: &Connection, workspace_id: &str) -> DbResult<Vec<Document>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, title, body, category, company_id, ticket_id, issue_id, created_at, updated_at, deleted_at, version, visibility
         FROM documents WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map(params![workspace_id], map_document)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn list_activities(conn: &Connection, workspace_id: &str, limit: i64) -> DbResult<Vec<Activity>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, actor, action, entity_type, entity_id, summary, metadata, created_at, updated_at, deleted_at, version
         FROM activities WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY created_at DESC LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![workspace_id, limit], map_activity)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn assignee_label(conn: &Connection, assignee_id: Option<&str>) -> DbResult<String> {
    let Some(id) = assignee_id.filter(|value| !value.is_empty()) else {
        return Ok("Unassigned".into());
    };
    let name: Option<String> = conn
        .query_row(
            "SELECT name FROM members WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(name.unwrap_or_else(|| "Unknown".into()))
}

pub fn log_ticket_ownership(
    conn: &Connection,
    workspace_id: &str,
    ticket_id: &str,
    assignee_id: Option<&str>,
    previous_assignee_id: Option<&str>,
    created_at: Option<&str>,
    handoff: bool,
) -> DbResult<()> {
    let ts = created_at
        .map(str::to_string)
        .unwrap_or_else(super::now_iso);
    let assignee_name = assignee_label(conn, assignee_id)?;
    let previous_id = previous_assignee_id
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let previous_assignee_name = if handoff {
        Some(assignee_label(conn, previous_id.as_deref())?)
    } else {
        None
    };
    conn.execute(
        "INSERT INTO ticket_ownership (
            id, workspace_id, ticket_id, assignee_id, assignee_name,
            previous_assignee_id, previous_assignee_name, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            super::new_id(),
            workspace_id,
            ticket_id,
            assignee_id.filter(|value| !value.is_empty()),
            assignee_name,
            previous_id,
            previous_assignee_name,
            ts
        ],
    )?;
    Ok(())
}

fn map_ticket_ownership(row: &Row<'_>) -> rusqlite::Result<TicketOwnership> {
    Ok(TicketOwnership {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        ticket_id: row.get(2)?,
        assignee_id: row.get(3)?,
        assignee_name: row.get(4)?,
        previous_assignee_id: row.get(5)?,
        previous_assignee_name: row.get(6)?,
        created_at: row.get(7)?,
    })
}

pub fn list_ticket_ownership(
    conn: &Connection,
    ticket_id: &str,
) -> DbResult<Vec<TicketOwnership>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, ticket_id, assignee_id, assignee_name,
                previous_assignee_id, previous_assignee_name, created_at
         FROM ticket_ownership WHERE ticket_id = ?1 ORDER BY created_at ASC, rowid ASC",
    )?;
    let rows = stmt.query_map(params![ticket_id], map_ticket_ownership)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn ensure_ticket_ownership(conn: &Connection, workspace_id: &str) -> DbResult<()> {
    let mut stmt = conn.prepare(
        "SELECT id, assignee_id, created_at FROM tickets
         WHERE workspace_id = ?1 AND deleted_at IS NULL",
    )?;
    let tickets: Vec<(String, Option<String>, String)> = stmt
        .query_map(params![workspace_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .filter_map(|row| row.ok())
        .collect();
    drop(stmt);

    for (ticket_id, assignee_id, created_at) in tickets {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM ticket_ownership WHERE ticket_id = ?1",
            params![ticket_id],
            |row| row.get(0),
        )?;
        if count > 0 {
            continue;
        }
        if assignee_id.is_some() {
            log_ticket_ownership(
                conn,
                workspace_id,
                &ticket_id,
                None,
                None,
                Some(&created_at),
                false,
            )?;
            log_ticket_ownership(
                conn,
                workspace_id,
                &ticket_id,
                assignee_id.as_deref(),
                None,
                Some(&created_at),
                true,
            )?;
        } else {
            log_ticket_ownership(
                conn,
                workspace_id,
                &ticket_id,
                None,
                None,
                Some(&created_at),
                false,
            )?;
        }
    }
    Ok(())
}

pub fn get_workspace(conn: &Connection, id: &str) -> DbResult<Option<Workspace>> {
    conn.query_row(
        "SELECT id, organization_id, name, created_at, updated_at, deleted_at, version
         FROM workspaces WHERE id = ?1 AND deleted_at IS NULL",
        params![id],
        map_workspace,
    )
    .optional()
    .map_err(DbError::from)
}
