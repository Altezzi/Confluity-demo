use rusqlite::{params, types::ValueRef, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::{new_id, now_iso, DbError, DbResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveQuery {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub sql_text: String,
    pub created_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveQueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub truncated: bool,
    pub ran_at: String,
}

const ROW_CAP: usize = 200;
const TABLE_ROW_CAP: usize = 2000;

const MENU_TABLES: &[&str] = &[
    "tickets",
    "issues",
    "companies",
    "contacts",
    "members",
    "rmas",
    "orders",
    "invoices",
    "payments",
    "assets",
    "products",
    "documents",
    "projects",
    "vendors",
    "purchase_orders",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightTable {
    pub name: String,
    pub row_count: i64,
}

fn assert_ident(name: &str) -> Result<&str, String> {
    if name.is_empty() || name.len() > 64 {
        return Err("invalid table".into());
    }
    if !name
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err("invalid table".into());
    }
    Ok(name)
}

fn map_query(row: &rusqlite::Row<'_>) -> rusqlite::Result<LiveQuery> {
    Ok(LiveQuery {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        sql_text: row.get(4)?,
        created_by: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        deleted_at: row.get(8)?,
        version: row.get(9)?,
    })
}

const Q_COLS: &str =
    "id, workspace_id, name, description, sql_text, created_by, created_at, updated_at, deleted_at, version";

pub fn list_live_queries(conn: &Connection, workspace_id: &str) -> DbResult<Vec<LiveQuery>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {Q_COLS} FROM live_queries
         WHERE workspace_id = ?1 AND deleted_at IS NULL
         ORDER BY name COLLATE NOCASE"
    ))?;
    let rows = stmt.query_map(params![workspace_id], map_query)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_live_query(conn: &Connection, id: &str) -> DbResult<Option<LiveQuery>> {
    conn.query_row(
        &format!("SELECT {Q_COLS} FROM live_queries WHERE id = ?1 AND deleted_at IS NULL"),
        params![id],
        map_query,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn upsert_live_query(
    conn: &Connection,
    workspace_id: &str,
    id: Option<&str>,
    name: &str,
    description: Option<&str>,
    sql_text: &str,
    created_by: Option<&str>,
) -> DbResult<LiveQuery> {
    let sql = assert_readonly_sql(sql_text).map_err(|e| {
        DbError::Sqlite(rusqlite::Error::InvalidParameterName(e))
    })?;
    let ts = now_iso();
    if let Some(existing_id) = id {
        conn.execute(
            "UPDATE live_queries SET name = ?1, description = ?2, sql_text = ?3, updated_at = ?4, version = version + 1
             WHERE id = ?5 AND deleted_at IS NULL",
            params![name.trim(), description, sql, ts, existing_id],
        )?;
        return get_live_query(conn, existing_id)?
            .ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows));
    }
    let new = new_id();
    conn.execute(
        "INSERT INTO live_queries (id, workspace_id, name, description, sql_text, created_by, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, NULL, 1)",
        params![new, workspace_id, name.trim(), description, sql, created_by, ts],
    )?;
    get_live_query(conn, &new)?.ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}

pub fn delete_live_query(conn: &Connection, id: &str) -> DbResult<()> {
    let ts = now_iso();
    conn.execute(
        "UPDATE live_queries SET deleted_at = ?1, updated_at = ?1, version = version + 1 WHERE id = ?2",
        params![ts, id],
    )?;
    Ok(())
}

fn table_exists(conn: &Connection, name: &str) -> DbResult<bool> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        params![name],
        |row| row.get(0),
    )?;
    Ok(n > 0)
}

fn count_sql(name: &str) -> String {
    match name {
        "invoice_lines" => format!("SELECT COUNT(*) FROM \"{name}\""),
        _ => format!("SELECT COUNT(*) FROM \"{name}\" WHERE deleted_at IS NULL"),
    }
}

fn view_sql(name: &str) -> String {
    match name {
        "tickets" => "
            SELECT t.id, t.title, t.status, t.priority, t.tags,
                   c.name AS company, ct.name AS contact, m.name AS assignee,
                   t.created_at, t.updated_at
            FROM tickets t
            LEFT JOIN companies c ON c.id = t.company_id
            LEFT JOIN contacts ct ON ct.id = t.contact_id
            LEFT JOIN members m ON m.id = t.assignee_id
            WHERE t.deleted_at IS NULL
            ORDER BY t.updated_at DESC
        "
        .into(),
        "issues" => "
            SELECT i.id, i.title, i.status, i.priority, i.blocked_reason,
                   p.name AS project, c.name AS company, m.name AS assignee,
                   tk.title AS ticket, i.created_at, i.updated_at
            FROM issues i
            LEFT JOIN projects p ON p.id = i.project_id
            LEFT JOIN companies c ON c.id = i.company_id
            LEFT JOIN members m ON m.id = i.assignee_id
            LEFT JOIN tickets tk ON tk.id = i.ticket_id
            WHERE i.deleted_at IS NULL
            ORDER BY i.updated_at DESC
        "
        .into(),
        "companies" => "
            SELECT id, name, domain, industry, notes, created_at, updated_at
            FROM companies WHERE deleted_at IS NULL ORDER BY name COLLATE NOCASE
        "
        .into(),
        "contacts" => "
            SELECT ct.id, ct.name, ct.email, ct.title, ct.phone, c.name AS company,
                   ct.created_at, ct.updated_at
            FROM contacts ct
            LEFT JOIN companies c ON c.id = ct.company_id
            WHERE ct.deleted_at IS NULL
            ORDER BY ct.updated_at DESC
        "
        .into(),
        "members" => "
            SELECT m.id, m.name, m.role, m.title, oc.name AS category, p.name AS position,
                   m.created_at, m.updated_at
            FROM members m
            LEFT JOIN positions p ON p.id = m.position_id
            LEFT JOIN org_categories oc ON oc.id = p.category_id
            WHERE m.deleted_at IS NULL
            ORDER BY m.name COLLATE NOCASE
        "
        .into(),
        "rmas" => "
            SELECT r.id, r.status, r.reason, r.resolution, c.name AS company,
                   a.name AS asset, a.serial, tk.title AS ticket, r.created_at, r.updated_at
            FROM rmas r
            LEFT JOIN companies c ON c.id = r.company_id
            LEFT JOIN assets a ON a.id = r.asset_id
            LEFT JOIN tickets tk ON tk.id = r.ticket_id
            WHERE r.deleted_at IS NULL
            ORDER BY r.updated_at DESC
        "
        .into(),
        "orders" => "
            SELECT o.id, o.reference, o.status, c.name AS company, o.total_cents, o.notes,
                   o.created_at, o.updated_at
            FROM orders o
            LEFT JOIN companies c ON c.id = o.company_id
            WHERE o.deleted_at IS NULL
            ORDER BY o.updated_at DESC
        "
        .into(),
        "invoices" => "
            SELECT i.id, i.reference, i.status, c.name AS company, o.reference AS order_ref,
                   i.amount_cents, i.due_at, i.created_at, i.updated_at
            FROM invoices i
            LEFT JOIN companies c ON c.id = i.company_id
            LEFT JOIN orders o ON o.id = i.order_id
            WHERE i.deleted_at IS NULL
            ORDER BY i.updated_at DESC
        "
        .into(),
        "payments" => "
            SELECT p.id, p.amount_cents, p.method, p.received_at, i.reference AS invoice,
                   p.notes, p.created_at, p.updated_at
            FROM payments p
            LEFT JOIN invoices i ON i.id = p.invoice_id
            WHERE p.deleted_at IS NULL
            ORDER BY p.updated_at DESC
        "
        .into(),
        "assets" => "
            SELECT a.id, a.name, a.serial, a.warranty_expires_at, pr.name AS product,
                   c.name AS company, tk.title AS ticket, a.created_at, a.updated_at
            FROM assets a
            LEFT JOIN products pr ON pr.id = a.product_id
            LEFT JOIN companies c ON c.id = a.company_id
            LEFT JOIN tickets tk ON tk.id = a.ticket_id
            WHERE a.deleted_at IS NULL
            ORDER BY a.updated_at DESC
        "
        .into(),
        "products" => "
            SELECT id, name, sku, description, inventory_qty, created_at, updated_at
            FROM products WHERE deleted_at IS NULL ORDER BY name COLLATE NOCASE
        "
        .into(),
        "documents" => "
            SELECT d.id, d.title, d.category, c.name AS company, tk.title AS ticket,
                   i.title AS issue, d.created_at, d.updated_at
            FROM documents d
            LEFT JOIN companies c ON c.id = d.company_id
            LEFT JOIN tickets tk ON tk.id = d.ticket_id
            LEFT JOIN issues i ON i.id = d.issue_id
            WHERE d.deleted_at IS NULL
            ORDER BY d.updated_at DESC
        "
        .into(),
        "projects" => "
            SELECT id, name, status, created_at, updated_at
            FROM projects WHERE deleted_at IS NULL ORDER BY updated_at DESC
        "
        .into(),
        "vendors" => "
            SELECT id, name, email, notes, created_at, updated_at
            FROM vendors WHERE deleted_at IS NULL ORDER BY name COLLATE NOCASE
        "
        .into(),
        "purchase_orders" => "
            SELECT po.id, po.reference, po.status, v.name AS vendor, pr.name AS product,
                   po.quantity, po.unit_cents, po.created_at, po.updated_at
            FROM purchase_orders po
            LEFT JOIN vendors v ON v.id = po.vendor_id
            LEFT JOIN products pr ON pr.id = po.product_id
            WHERE po.deleted_at IS NULL
            ORDER BY po.updated_at DESC
        "
        .into(),
        other => format!("SELECT * FROM \"{other}\""),
    }
}

pub fn list_insight_tables(conn: &Connection) -> DbResult<Vec<InsightTable>> {
    let mut tables = Vec::new();
    for name in MENU_TABLES {
        if !table_exists(conn, name)? {
            continue;
        }
        let count: i64 = conn.query_row(&count_sql(name), [], |row| row.get(0))?;
        tables.push(InsightTable {
            name: (*name).to_string(),
            row_count: count,
        });
    }
    Ok(tables)
}

pub fn load_insight_table(conn: &Connection, name: &str) -> DbResult<LiveQueryResult> {
    let ident = assert_ident(name).map_err(|e| {
        DbError::Sqlite(rusqlite::Error::InvalidParameterName(e))
    })?;
    if !MENU_TABLES.contains(&ident) {
        return Err(DbError::Sqlite(rusqlite::Error::InvalidParameterName(
            "table is not available".into(),
        )));
    }
    if !table_exists(conn, ident)? {
        return Err(DbError::Sqlite(rusqlite::Error::InvalidParameterName(
            "table not found".into(),
        )));
    }
    run_sql(conn, &view_sql(ident), TABLE_ROW_CAP)
}

pub fn run_live_sql(conn: &Connection, sql_text: &str) -> DbResult<LiveQueryResult> {
    let sql = assert_readonly_sql(sql_text).map_err(|e| {
        DbError::Sqlite(rusqlite::Error::InvalidParameterName(e))
    })?;
    run_sql(conn, &sql, ROW_CAP)
}

fn run_sql(conn: &Connection, sql: &str, cap: usize) -> DbResult<LiveQueryResult> {
    let mut stmt = conn.prepare(sql)?;
    let columns: Vec<String> = stmt.column_names().into_iter().map(|s| s.to_string()).collect();
    let col_count = columns.len();
    let mut rows: Vec<Vec<serde_json::Value>> = Vec::new();
    let mut truncated = false;
    {
        let mut query = stmt.query([])?;
        while let Some(row) = query.next()? {
            if rows.len() >= cap {
                truncated = true;
                break;
            }
            let mut cells = Vec::with_capacity(col_count);
            for i in 0..col_count {
                cells.push(value_to_json(row.get_ref(i)?));
            }
            rows.push(cells);
        }
    }
    Ok(LiveQueryResult {
        columns,
        rows,
        truncated,
        ran_at: now_iso(),
    })
}

fn value_to_json(v: ValueRef<'_>) -> serde_json::Value {
    match v {
        ValueRef::Null => serde_json::Value::Null,
        ValueRef::Integer(i) => serde_json::json!(i),
        ValueRef::Real(f) => serde_json::json!(f),
        ValueRef::Text(t) => serde_json::Value::String(String::from_utf8_lossy(t).into_owned()),
        ValueRef::Blob(_) => serde_json::Value::String("[blob]".into()),
    }
}

fn assert_readonly_sql(sql: &str) -> Result<String, String> {
    let mut s = sql.trim().to_string();
    if s.ends_with(';') {
        s.pop();
        s = s.trim().to_string();
    }
    if s.is_empty() {
        return Err("query is empty".into());
    }
    if s.contains(';') {
        return Err("one statement only".into());
    }
    let collapsed = s
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let padded = format!(" {collapsed} ");
    let first = collapsed.split_whitespace().next().unwrap_or("");
    if first != "select" && first != "with" {
        return Err("only SELECT queries are allowed".into());
    }
    for word in [
        " insert ",
        " update ",
        " delete ",
        " drop ",
        " alter ",
        " attach ",
        " detach ",
        " pragma ",
        " vacuum ",
        " create ",
        " replace into ",
    ] {
        if padded.contains(word) {
            return Err("that statement is not allowed on live queries".into());
        }
    }
    Ok(s)
}

pub fn ensure_default_queries(conn: &Connection, workspace_id: &str) -> DbResult<()> {
    let existing = list_live_queries(conn, workspace_id)?;
    let defs: &[(&str, &str, &str)] = &[
        (
            "Open tickets by status",
            "Live ticket queue — updates as tickets move",
            "SELECT status, COUNT(*) AS tickets\nFROM tickets\nWHERE deleted_at IS NULL\nGROUP BY status\nORDER BY tickets DESC",
        ),
        (
            "Open tickets by owner",
            "Who is holding the live support queue",
            "SELECT COALESCE(m.name, 'Unassigned') AS owner, COUNT(*) AS open_tickets\nFROM tickets t\nLEFT JOIN members m ON m.id = t.assignee_id\nWHERE t.deleted_at IS NULL AND t.status NOT IN ('resolved', 'closed')\nGROUP BY owner\nORDER BY open_tickets DESC",
        ),
        (
            "Issues by status",
            "Engineering board, live",
            "SELECT status, COUNT(*) AS issues\nFROM issues\nWHERE deleted_at IS NULL\nGROUP BY status\nORDER BY issues DESC",
        ),
        (
            "Blocked issues",
            "What is stuck right now",
            "SELECT title, blocked_reason, status, priority\nFROM issues\nWHERE deleted_at IS NULL AND status = 'blocked'\nORDER BY updated_at DESC",
        ),
        (
            "Open RMAs",
            "Operations returns in flight",
            "SELECT status, COUNT(*) AS rmas\nFROM rmas\nWHERE deleted_at IS NULL AND lower(status) NOT IN ('closed', 'complete', 'completed')\nGROUP BY status\nORDER BY rmas DESC",
        ),
    ];
    for (name, desc, sql) in defs {
        if existing.iter().any(|q| q.name.eq_ignore_ascii_case(name)) {
            continue;
        }
        let _ = upsert_live_query(conn, workspace_id, None, name, Some(desc), sql, None)?;
    }
    Ok(())
}
