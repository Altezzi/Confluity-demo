use crate::db::{
    models::list_members, open_db,
    seed::{ensure_phase1_demo, ensure_seeded, seed_demo_data},
    set_meta, ticket_forms::list_ticket_forms, upsert_fts,
};

fn ensure_demo_seeded(conn: &rusqlite::Connection) -> crate::db::models::Workspace {
    let ws = ensure_seeded(conn).unwrap();
    set_meta(conn, "demo_mode", "true").unwrap();
    seed_demo_data(conn, &ws).unwrap();
    ensure_phase1_demo(conn, &ws).unwrap();
    set_meta(conn, "seeded", "true").unwrap();
    set_meta(conn, "setup_complete", "true").unwrap();
    ws
}
use tempfile::tempdir;

#[test]
fn seeds_and_searches_vertical_demo() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let conn = open_db(&db_path).unwrap();
    let ws = ensure_demo_seeded(&conn);
    assert_eq!(ws.name, "Main Workspace");

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM companies WHERE workspace_id = ?1",
            rusqlite::params![ws.id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(count >= 1);

    let hits: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM search_fts WHERE search_fts MATCH '\"firmware\"*'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(hits >= 1);

    upsert_fts(&conn, "company", "x", "TestCo", "hello world").unwrap();
}

#[test]
fn phase1_projects_members_and_comments() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let conn = open_db(&db_path).unwrap();
    let ws = ensure_demo_seeded(&conn);

    let members: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM members WHERE workspace_id = ?1",
            rusqlite::params![ws.id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(members >= 2);

    let titled: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM members WHERE workspace_id = ?1 AND title IS NOT NULL AND title != ''",
            rusqlite::params![ws.id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(titled >= 2);

    let listed = list_members(&conn, &ws.id).unwrap();
    assert!(listed.len() >= 2);
    assert!(listed.iter().all(|m| !m.must_set_password));
    let sample = &listed[0];
    conn.execute(
        "UPDATE members SET email = ?1, must_set_password = 1, invite_code = ?2 WHERE id = ?3",
        rusqlite::params![format!("invite-{}@example.com", sample.id), "ABCD-EFGH", sample.id],
    )
    .unwrap();
    let again = list_members(&conn, &ws.id).unwrap();
    let updated = again.iter().find(|m| m.id == sample.id).unwrap();
    let expected = format!("invite-{}@example.com", sample.id);
    assert_eq!(updated.email.as_deref(), Some(expected.as_str()));
    assert!(updated.must_set_password);
    assert!(updated.invite_code.is_none());

    let forms = list_ticket_forms(&conn, &ws.id).unwrap();
    assert_eq!(forms.len(), 1);
    assert!(forms[0].is_default);
    assert!(forms[0].fields.iter().any(|field| field.field.key == "title" && field.required));
    assert!(forms[0].fields.iter().any(|field| field.field.key == "company" && field.required));

    let projects: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM projects WHERE workspace_id = ?1",
            rusqlite::params![ws.id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(projects >= 1);

    let comments: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM comments WHERE workspace_id = ?1",
            rusqlite::params![ws.id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(comments >= 1);

    let assigned: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tickets WHERE workspace_id = ?1 AND assignee_id IS NOT NULL",
            rusqlite::params![ws.id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(assigned >= 1);

    let ticket_id: String = conn
        .query_row(
            "SELECT id FROM tickets WHERE workspace_id = ?1 LIMIT 1",
            rusqlite::params![ws.id],
            |row| row.get(0),
        )
        .unwrap();
    let listed = crate::db::models::list_comments(&conn, &ws.id, "ticket", &ticket_id).unwrap();
    assert!(!listed.is_empty());

    let ownership = crate::db::models::list_ticket_ownership(&conn, &ticket_id).unwrap();
    assert!(!ownership.is_empty());

    let products: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM products WHERE workspace_id = ?1",
            rusqlite::params![ws.id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(products >= 1);

    let assets: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM assets WHERE serial = 'NW-4412'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(assets >= 1);

    let rmas: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM rmas WHERE workspace_id = ?1",
            rusqlite::params![ws.id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(rmas >= 1);
}

#[test]
fn ops_finance_and_workspace_paths() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let conn = open_db(&db_path).unwrap();
    let ws = ensure_demo_seeded(&conn);

    let invoices: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM invoices WHERE reference = 'INV-2207'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(invoices >= 1);

    let vendors: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM vendors WHERE name = 'Helio Components'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(vendors >= 1);

    let pos: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM purchase_orders WHERE reference = 'PO-4412'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(pos >= 1);

    let hits: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM search_fts WHERE search_fts MATCH '\"NW-4412\"*'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(hits >= 1);

    let ticket_id: String = conn
        .query_row(
            "SELECT id FROM tickets WHERE workspace_id = ?1 LIMIT 1",
            rusqlite::params![ws.id],
            |row| row.get(0),
        )
        .unwrap();
    let assets = crate::db::extended::assets_for_ticket(&conn, &ticket_id).unwrap();
    assert!(assets.iter().any(|a| a.serial.as_deref() == Some("NW-4412")));
    let rmas = crate::db::extended::rmas_for_ticket(&conn, &ticket_id).unwrap();
    assert!(!rmas.is_empty());

    let public_comments: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM comments WHERE entity_id = ?1 AND visibility = 'public' AND deleted_at IS NULL",
            rusqlite::params![ticket_id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(public_comments >= 1);

    let public_docs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM documents WHERE title = 'Fleet Sync Troubleshooting' AND visibility = 'public'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(public_docs >= 1);

    crate::db::record_change(&conn, &ws.id, "create", "ticket", &ticket_id, "test change").unwrap();
    let queued: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM outbox WHERE workspace_id = ?1 AND status = 'queued'",
            rusqlite::params![ws.id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(queued >= 1);
    let audit: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE workspace_id = ?1",
            rusqlite::params![ws.id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(audit >= 1);
    let events: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM workspace_events WHERE workspace_id = ?1",
            rusqlite::params![ws.id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(events >= 1);

    let notes: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM notifications WHERE workspace_id = ?1",
            rusqlite::params![ws.id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(notes >= 1);
}

#[test]
fn fresh_workspace_starts_empty_until_setup() {
    let dir = tempdir().unwrap();
    let conn = open_db(&dir.path().join("test.db")).unwrap();
    let ws = ensure_seeded(&conn).unwrap();

    assert_eq!(
        crate::db::get_meta(&conn, "setup_complete").unwrap().as_deref(),
        Some("false")
    );
    let companies: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM companies WHERE workspace_id = ?1 AND deleted_at IS NULL",
            rusqlite::params![ws.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(companies, 0);

    seed_demo_data(&conn, &ws).unwrap();
    assert!(crate::db::setup::demo_data_detected(&conn, &ws.id).unwrap());
    crate::db::setup::clear_demo_workspace_data(&conn, &ws.id).unwrap();
    assert!(!crate::db::setup::demo_data_detected(&conn, &ws.id).unwrap());
    assert_eq!(
        crate::db::get_meta(&conn, "setup_complete").unwrap().as_deref(),
        Some("false")
    );
}
