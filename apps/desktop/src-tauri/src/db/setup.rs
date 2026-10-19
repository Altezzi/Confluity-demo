use rusqlite::{params, Connection};

use super::{get_meta, set_meta, DbResult};

pub fn is_setup_complete(conn: &Connection) -> DbResult<bool> {
    Ok(get_meta(conn, "setup_complete")?.as_deref() == Some("true"))
}

pub fn set_setup_complete(conn: &Connection, complete: bool) -> DbResult<()> {
    set_meta(conn, "setup_complete", if complete { "true" } else { "false" })
}

pub fn demo_data_detected(conn: &Connection, workspace_id: &str) -> DbResult<bool> {
    let northwind: i64 = conn.query_row(
        "SELECT COUNT(*) FROM companies WHERE workspace_id = ?1 AND deleted_at IS NULL AND name = 'Northwind Robotics'",
        params![workspace_id],
        |row| row.get(0),
    )?;
    Ok(northwind > 0)
}

/// Remove demo seed records so release setup starts clean (keeps roles, positions, modules).
pub fn clear_demo_workspace_data(conn: &Connection, workspace_id: &str) -> DbResult<()> {
    let ts = super::now_iso();
    let tables = [
        "attachments",
        "comments",
        "notifications",
        "outbox",
        "activities",
        "workspace_events",
        "issues",
        "tickets",
        "contacts",
        "companies",
        "documents",
        "products",
        "assets",
        "rmas",
        "orders",
        "invoices",
        "payments",
        "vendors",
        "purchase_orders",
        "projects",
    ];
    for table in tables {
        let sql = format!(
            "UPDATE {table} SET deleted_at = ?1, updated_at = ?1, version = version + 1
             WHERE workspace_id = ?2 AND deleted_at IS NULL"
        );
        let _ = conn.execute(&sql, params![ts, workspace_id]);
    }

    conn.execute(
        "DELETE FROM search_fts WHERE entity_type IN (
            'company','contact','ticket','issue','document','project','product','asset','order','invoice','vendor','rma'
        )",
        [],
    )?;

    conn.execute(
        "UPDATE members SET deleted_at = ?1, updated_at = ?1, version = version + 1
         WHERE workspace_id = ?2 AND deleted_at IS NULL AND name IN ('Jordan Hale', 'Riley Okonkwo')",
        params![ts, workspace_id],
    )?;

    set_meta(conn, "current_member_id", "")?;
    set_meta(conn, "setup_admin_id", "")?;
    set_meta(conn, "seeded", "false")?;
    set_setup_complete(conn, false)?;
    Ok(())
}
