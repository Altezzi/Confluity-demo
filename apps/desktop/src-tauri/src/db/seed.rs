use rusqlite::{params, Connection};

use super::models::Workspace;
use super::{get_meta, log_activity, new_id, notify_workspace, now_iso, set_meta, upsert_fts, DbResult};

pub fn ensure_seeded(conn: &Connection) -> DbResult<Workspace> {
    if let Some(workspace_id) = get_meta(conn, "workspace_id")? {
        if let Some(ws) = super::models::get_workspace(conn, &workspace_id)? {
            // Upgrade legacy demo tenant names so the sidebar shows the customer org.
            let org_name: Option<String> = conn
                .query_row(
                    "SELECT name FROM organizations WHERE id = ?1",
                    params![ws.organization_id],
                    |row| row.get(0),
                )
                .ok();
            if org_name.as_deref() == Some("Local Organization") {
                let ts = now_iso();
                conn.execute(
                    "UPDATE organizations SET name = 'Apex Industrial', updated_at = ?1, version = version + 1 WHERE id = ?2",
                    params![ts, ws.organization_id],
                )?;
                conn.execute(
                    "UPDATE workspaces SET name = 'Operations', updated_at = ?1, version = version + 1 WHERE id = ?2",
                    params![ts, ws.id],
                )?;
            }
            if get_meta(conn, "demo_mode")?.as_deref() == Some("true")
                && get_meta(conn, "seeded")?.as_deref() != Some("true")
            {
                seed_demo_data(conn, &ws)?;
                set_meta(conn, "seeded", "true")?;
            }
            let ws = super::models::get_workspace(conn, &workspace_id)?.unwrap_or(ws);
            if get_meta(conn, "demo_mode")?.as_deref() == Some("true") {
                ensure_phase1_demo(conn, &ws)?;
            }
            if get_meta(conn, "setup_complete")?.is_none() {
                let complete = get_meta(conn, "seeded")?.as_deref() == Some("true")
                    && get_meta(conn, "demo_mode")?.as_deref() == Some("true");
                set_meta(conn, "setup_complete", if complete { "true" } else { "false" })?;
            }
            super::roles::ensure_default_roles(conn, &ws.id)?;
            super::positions::ensure_default_positions(conn, &ws.id)?;
            super::positions::ensure_setup_admin(conn, &ws.id)?;
            super::teams::ensure_default_teams(conn, &ws.id)?;
            super::insights::ensure_default_queries(conn, &ws.id)?;
            super::modules::ensure_workspace_modules(conn, &ws.id)?;
            return Ok(ws);
        }
    }

    let ts = now_iso();
    let org_id = new_id();
    let workspace_id = new_id();

    conn.execute(
        "INSERT INTO organizations (id, name, created_at, updated_at, deleted_at, version)
         VALUES (?1, 'Apex Industrial', ?2, ?2, NULL, 1)",
        params![org_id, ts],
    )?;
    conn.execute(
        "INSERT INTO workspaces (id, organization_id, name, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, 'Operations', ?3, ?3, NULL, 1)",
        params![workspace_id, org_id, ts],
    )?;

    set_meta(conn, "organization_id", &org_id)?;
    set_meta(conn, "workspace_id", &workspace_id)?;
    set_meta(conn, "storage_mode", "local")?;
    set_meta(conn, "ai_api_base_url", "https://api.openai.com/v1")?;
    set_meta(conn, "ai_model", "gpt-4o-mini")?;

    let ws = Workspace {
        id: workspace_id.clone(),
        organization_id: org_id,
        name: "Operations".into(),
        created_at: ts.clone(),
        updated_at: ts,
        deleted_at: None,
        version: 1,
    };

    set_meta(conn, "demo_mode", "true")?;
    set_meta(conn, "setup_complete", "true")?;
    set_meta(conn, "seeded", "false")?;
    seed_demo_data(conn, &ws)?;
    ensure_phase1_demo(conn, &ws)?;
    set_meta(conn, "seeded", "true")?;
    super::roles::ensure_default_roles(conn, &ws.id)?;
    super::positions::ensure_default_positions(conn, &ws.id)?;
    super::positions::ensure_setup_admin(conn, &ws.id)?;
    super::teams::ensure_default_teams(conn, &ws.id)?;
    super::insights::ensure_default_queries(conn, &ws.id)?;
    super::modules::ensure_workspace_modules(conn, &ws.id)?;
    Ok(ws)
}

pub(crate) fn seed_demo_data(conn: &Connection, ws: &Workspace) -> DbResult<()> {
    let existing: i64 = conn.query_row(
        "SELECT COUNT(*) FROM companies WHERE workspace_id = ?1 AND deleted_at IS NULL",
        params![ws.id],
        |row| row.get(0),
    )?;
    if existing > 0 {
        return Ok(());
    }

    let ts = now_iso();
    let company_id = new_id();
    let contact_id = new_id();
    let ticket_id = new_id();
    let issue_id = new_id();
    let doc_id = new_id();

    conn.execute(
        "INSERT INTO companies (id, workspace_id, name, domain, industry, notes, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, 'Northwind Robotics', 'northwindrobotics.com', 'Industrial Automation',
         'Enterprise customer evaluating Confluity for support + engineering handoff.', ?3, ?3, NULL, 1)",
        params![company_id, ws.id, ts],
    )?;
    upsert_fts(
        conn,
        "company",
        &company_id,
        "Northwind Robotics",
        "northwindrobotics.com Industrial Automation Enterprise customer",
    )?;

    conn.execute(
        "INSERT INTO contacts (id, workspace_id, company_id, name, email, title, phone, notes, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, 'Maya Chen', 'maya.chen@northwindrobotics.com', 'Head of Support', '+1-555-0142',
         'Primary contact for escalations.', ?4, ?4, NULL, 1)",
        params![contact_id, ws.id, company_id, ts],
    )?;
    upsert_fts(
        conn,
        "contact",
        &contact_id,
        "Maya Chen",
        "maya.chen@northwindrobotics.com Head of Support",
    )?;

    let ticket_desc = "Fleet sync intermittently fails after firmware 3.2.1. Customer sees stale telemetry for 8–12 minutes. Blocking pilot expansion.";
    conn.execute(
        "INSERT INTO tickets (id, workspace_id, company_id, contact_id, title, description, status, priority, tags, assignee_id, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, 'Fleet sync fails after firmware 3.2.1', ?5, 'in_progress', 'high', 'firmware,sync,pilot', NULL, ?6, ?6, NULL, 1)",
        params![ticket_id, ws.id, company_id, contact_id, ticket_desc, ts],
    )?;
    upsert_fts(
        conn,
        "ticket",
        &ticket_id,
        "Fleet sync fails after firmware 3.2.1",
        ticket_desc,
    )?;

    let issue_desc = "Reproduce race in SyncWorker when heartbeat ACK arrives after delta batch commit. Likely missing lock around cursor advance.";
    let blocked = "Waiting on firmware team logs from unit NW-4412 and confirmation of ACK ordering change in 3.2.1.";
    conn.execute(
        "INSERT INTO issues (id, workspace_id, ticket_id, company_id, project_id, title, description, status, priority, blocked_reason, assignee_id, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, NULL, 'Fix SyncWorker race after firmware 3.2.1', ?5, 'blocked', 'high', ?6, NULL, ?7, ?7, NULL, 1)",
        params![issue_id, ws.id, ticket_id, company_id, issue_desc, blocked, ts],
    )?;
    upsert_fts(
        conn,
        "issue",
        &issue_id,
        "Fix SyncWorker race after firmware 3.2.1",
        &format!("{issue_desc} {blocked}"),
    )?;

    let doc_body = "# Fleet Sync Troubleshooting\n\n## Symptoms\nStale telemetry lasting 8–12 minutes after firmware 3.2.1.\n\n## Known related issue\nEngineering issue tracks a SyncWorker race around heartbeat ACK ordering.\n\n## Workaround\nForce a full resync from the device console. Do not roll back firmware during the pilot.\n\n## Next steps\nCollect NW-4412 logs and compare ACK timestamps to delta commits.";
    conn.execute(
        "INSERT INTO documents (id, workspace_id, title, body, category, company_id, ticket_id, issue_id, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, 'Fleet Sync Troubleshooting', ?3, 'Support', ?4, ?5, ?6, ?7, ?7, NULL, 1)",
        params![doc_id, ws.id, doc_body, company_id, ticket_id, issue_id, ts],
    )?;
    upsert_fts(
        conn,
        "document",
        &doc_id,
        "Fleet Sync Troubleshooting",
        doc_body,
    )?;

    log_activity(
        conn,
        &ws.id,
        "seed",
        "company",
        &company_id,
        "Seeded demo company Northwind Robotics",
    )?;
    log_activity(
        conn,
        &ws.id,
        "seed",
        "ticket",
        &ticket_id,
        "Seeded support ticket for fleet sync failure",
    )?;
    log_activity(
        conn,
        &ws.id,
        "seed",
        "issue",
        &issue_id,
        "Seeded engineering issue linked from support ticket",
    )?;
    log_activity(
        conn,
        &ws.id,
        "seed",
        "document",
        &doc_id,
        "Seeded troubleshooting document linked to ticket and issue",
    )?;

    Ok(())
}

pub(crate) fn ensure_phase1_demo(conn: &Connection, ws: &Workspace) -> DbResult<()> {
    let member_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM members WHERE workspace_id = ?1 AND deleted_at IS NULL",
        params![ws.id],
        |row| row.get(0),
    )?;

    let ts = now_iso();
    let jordan_id = new_id();
    let riley_id = new_id();
    let project_id = new_id();

    if member_count == 0 {
        conn.execute(
            "INSERT INTO members (id, workspace_id, name, role, title, created_at, updated_at, deleted_at, version)
             VALUES (?1, ?2, 'Jordan Hale', 'admin', 'Support', ?3, ?3, NULL, 1)",
            params![jordan_id, ws.id, ts],
        )?;
        conn.execute(
            "INSERT INTO members (id, workspace_id, name, role, title, created_at, updated_at, deleted_at, version)
             VALUES (?1, ?2, 'Riley Okonkwo', 'engineering', 'Engineering', ?3, ?3, NULL, 1)",
            params![riley_id, ws.id, ts],
        )?;
        set_meta(conn, "current_member_id", &jordan_id)?;
    }

    conn.execute(
        "UPDATE members SET title = 'Support' WHERE name = 'Jordan Hale' AND (title IS NULL OR title = '')",
        [],
    )?;
    conn.execute(
        "UPDATE members SET title = 'Engineering' WHERE name = 'Riley Okonkwo' AND (title IS NULL OR title = '')",
        [],
    )?;

    let project_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM projects WHERE workspace_id = ?1 AND deleted_at IS NULL",
        params![ws.id],
        |row| row.get(0),
    )?;
    if project_count == 0 {
        conn.execute(
            "INSERT INTO projects (id, workspace_id, name, description, status, created_at, updated_at, deleted_at, version)
             VALUES (?1, ?2, 'Fleet Platform', 'Firmware, sync, and device telemetry workstream.', 'active', ?3, ?3, NULL, 1)",
            params![project_id, ws.id, ts],
        )?;
        upsert_fts(
            conn,
            "project",
            &project_id,
            "Fleet Platform",
            "Firmware sync device telemetry",
        )?;
        log_activity(
            conn,
            &ws.id,
            "create",
            "project",
            &project_id,
            "Created project Fleet Platform",
        )?;

        let jordan = if member_count == 0 {
            jordan_id.clone()
        } else {
            conn.query_row(
                "SELECT id FROM members WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY name LIMIT 1",
                params![ws.id],
                |row| row.get::<_, String>(0),
            )
            .unwrap_or(jordan_id.clone())
        };
        let riley = if member_count == 0 {
            riley_id.clone()
        } else {
            conn.query_row(
                "SELECT id FROM members WHERE workspace_id = ?1 AND role = 'engineering' AND deleted_at IS NULL LIMIT 1",
                params![ws.id],
                |row| row.get::<_, String>(0),
            )
            .unwrap_or(jordan.clone())
        };

        conn.execute(
            "UPDATE tickets SET assignee_id = ?1 WHERE workspace_id = ?2 AND assignee_id IS NULL AND deleted_at IS NULL",
            params![jordan, ws.id],
        )?;
        conn.execute(
            "UPDATE issues SET project_id = ?1, assignee_id = COALESCE(assignee_id, ?2)
             WHERE workspace_id = ?3 AND deleted_at IS NULL",
            params![project_id, riley, ws.id],
        )?;
    }

    if get_meta(conn, "current_member_id")?.is_none() {
        if let Ok(id) = conn.query_row(
            "SELECT id FROM members WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY created_at LIMIT 1",
            params![ws.id],
            |row| row.get::<_, String>(0),
        ) {
            set_meta(conn, "current_member_id", &id)?;
        }
    }

    let comment_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM comments WHERE workspace_id = ?1 AND deleted_at IS NULL",
        params![ws.id],
        |row| row.get(0),
    )?;
    if comment_count == 0 {
        if let Ok(ticket_id) = conn.query_row(
            "SELECT id FROM tickets WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY created_at LIMIT 1",
            params![ws.id],
            |row| row.get::<_, String>(0),
        ) {
            let cid = new_id();
            conn.execute(
                "INSERT INTO comments (id, workspace_id, entity_type, entity_id, author, body, visibility, created_at, updated_at, deleted_at, version)
                 VALUES (?1, ?2, 'ticket', ?3, 'Maya Chen', 'Customer confirmed NW-4412 still shows a 10 minute telemetry gap after resync.', 'internal', ?4, ?4, NULL, 1)",
                params![cid, ws.id, ticket_id, ts],
            )?;
        }
        if let Ok(issue_id) = conn.query_row(
            "SELECT id FROM issues WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY created_at LIMIT 1",
            params![ws.id],
            |row| row.get::<_, String>(0),
        ) {
            let cid = new_id();
            conn.execute(
                "INSERT INTO comments (id, workspace_id, entity_type, entity_id, author, body, visibility, created_at, updated_at, deleted_at, version)
                 VALUES (?1, ?2, 'issue', ?3, 'Riley Okonkwo', 'Internal note: ACK race looks real on the 3.2.1 branch. Need the unit log before changing the lock.', 'internal', ?4, ?4, NULL, 1)",
                params![cid, ws.id, issue_id, ts],
            )?;
        }
    }

    conn.execute(
        "UPDATE comments SET visibility = 'internal'
         WHERE author = 'Maya Chen'
           AND body LIKE 'Customer confirmed NW-4412%'
           AND visibility = 'public'",
        [],
    )?;

    super::models::ensure_ticket_ownership(conn, &ws.id)?;
    ensure_ops_demo(conn, ws)?;
    Ok(())
}

fn ensure_ops_demo(conn: &Connection, ws: &Workspace) -> DbResult<()> {
    let product_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM products WHERE workspace_id = ?1 AND deleted_at IS NULL",
        params![ws.id],
        |row| row.get(0),
    )?;
    if product_count > 0 {
        let _ = conn.execute(
            "UPDATE documents SET visibility = 'public' WHERE title = 'Fleet Sync Troubleshooting' AND (visibility IS NULL OR visibility = 'internal')",
            [],
        );
        if let Ok(ticket_id) = conn.query_row(
            "SELECT id FROM tickets WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY created_at LIMIT 1",
            params![ws.id],
            |row| row.get::<_, String>(0),
        ) {
            let public_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM comments WHERE entity_type = 'ticket' AND entity_id = ?1 AND visibility = 'public' AND deleted_at IS NULL",
                params![ticket_id],
                |row| row.get(0),
            )?;
            if public_count == 0 {
                let ts = now_iso();
                conn.execute(
                    "INSERT INTO comments (id, workspace_id, entity_type, entity_id, author, body, visibility, created_at, updated_at, deleted_at, version)
                     VALUES (?1, ?2, 'ticket', ?3, 'Jordan Hale', 'Public update: we are investigating the telemetry stall on NW-4412 and have a replacement RMA ready if needed.', 'public', ?4, ?4, NULL, 1)",
                    params![new_id(), ws.id, ticket_id, ts],
                )?;
            }
        }
        return Ok(());
    }

    let ts = now_iso();
    let company_id: String = conn.query_row(
        "SELECT id FROM companies WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY created_at LIMIT 1",
        params![ws.id],
        |row| row.get(0),
    )?;
    let ticket_id: String = conn.query_row(
        "SELECT id FROM tickets WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY created_at LIMIT 1",
        params![ws.id],
        |row| row.get(0),
    )?;

    let product_id = new_id();
    conn.execute(
        "INSERT INTO products (id, workspace_id, name, sku, description, inventory_qty, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, 'Fleet Sync Hub', 'FSH-321', 'On-device telemetry hub used in the Northwind pilot.', 18, ?3, ?3, NULL, 1)",
        params![product_id, ws.id, ts],
    )?;
    upsert_fts(conn, "product", &product_id, "Fleet Sync Hub", "FSH-321 telemetry hub firmware")?;

    let asset_id = new_id();
    conn.execute(
        "INSERT INTO assets (id, workspace_id, product_id, company_id, ticket_id, name, serial, warranty_expires_at, notes, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, 'Pilot unit NW-4412', 'NW-4412', '2027-03-01', 'Shows 8-12 minute telemetry gap after firmware 3.2.1.', ?6, ?6, NULL, 1)",
        params![asset_id, ws.id, product_id, company_id, ticket_id, ts],
    )?;
    upsert_fts(conn, "asset", &asset_id, "Pilot unit NW-4412", "NW-4412 warranty telemetry")?;

    let rma_id = new_id();
    conn.execute(
        "INSERT INTO rmas (id, workspace_id, company_id, asset_id, ticket_id, status, reason, resolution, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, 'open', 'Telemetry stall after firmware 3.2.1. Customer needs a replacement hub if logs confirm a hardware race.', NULL, ?6, ?6, NULL, 1)",
        params![rma_id, ws.id, company_id, asset_id, ticket_id, ts],
    )?;
    upsert_fts(conn, "rma", &rma_id, "RMA NW-4412", "replacement hub firmware")?;

    let order_id = new_id();
    conn.execute(
        "INSERT INTO orders (id, workspace_id, company_id, status, reference, notes, total_cents, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, 'fulfilled', 'SO-8841', 'Pilot fleet order including NW-4412.', 249900, ?4, ?4, NULL, 1)",
        params![order_id, ws.id, company_id, ts],
    )?;
    upsert_fts(conn, "order", &order_id, "SO-8841", "pilot fleet order")?;

    let invoice_id = new_id();
    conn.execute(
        "INSERT INTO invoices (id, workspace_id, company_id, order_id, status, reference, amount_cents, due_at, notes, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, 'open', 'INV-2207', 249900, '2026-09-01', 'Net 30 on the Northwind pilot order.', ?5, ?5, NULL, 1)",
        params![invoice_id, ws.id, company_id, order_id, ts],
    )?;
    conn.execute(
        "INSERT INTO invoice_lines (id, invoice_id, description, quantity, unit_cents)
         VALUES (?1, ?2, 'Fleet Sync Hub', 1, 249900)",
        params![new_id(), invoice_id],
    )?;
    upsert_fts(conn, "invoice", &invoice_id, "INV-2207", "Northwind pilot invoice")?;

    let vendor_id = new_id();
    conn.execute(
        "INSERT INTO vendors (id, workspace_id, name, email, notes, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, 'Helio Components', 'buy@heliocomponents.com', 'Board and radio supplier for Fleet Sync Hub.', ?3, ?3, NULL, 1)",
        params![vendor_id, ws.id, ts],
    )?;
    upsert_fts(conn, "vendor", &vendor_id, "Helio Components", "board radio supplier")?;

    let po_id = new_id();
    conn.execute(
        "INSERT INTO purchase_orders (id, workspace_id, vendor_id, product_id, status, quantity, unit_cents, reference, notes, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, 'received', 20, 89000, 'PO-4412', 'Restock after the Northwind pilot.', ?5, ?5, NULL, 1)",
        params![po_id, ws.id, vendor_id, product_id, ts],
    )?;
    upsert_fts(conn, "purchase_order", &po_id, "PO-4412", "restock Fleet Sync Hub")?;

    let _ = conn.execute(
        "UPDATE documents SET visibility = 'public' WHERE title = 'Fleet Sync Troubleshooting'",
        [],
    );

    let public_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM comments WHERE entity_type = 'ticket' AND entity_id = ?1 AND visibility = 'public' AND deleted_at IS NULL",
        params![ticket_id],
        |row| row.get(0),
    )?;
    if public_count == 0 {
        conn.execute(
            "INSERT INTO comments (id, workspace_id, entity_type, entity_id, author, body, visibility, created_at, updated_at, deleted_at, version)
             VALUES (?1, ?2, 'ticket', ?3, 'Jordan Hale', 'Public update: we are investigating the telemetry stall on NW-4412 and have a replacement RMA ready if needed.', 'public', ?4, ?4, NULL, 1)",
            params![new_id(), ws.id, ticket_id, ts],
        )?;
    }

    log_activity(conn, &ws.id, "seed", "asset", &asset_id, "Seeded asset NW-4412")?;
    log_activity(conn, &ws.id, "seed", "rma", &rma_id, "Seeded open RMA for NW-4412")?;
    notify_workspace(
        conn,
        &ws.id,
        None,
        "RMA opened for NW-4412",
        "Replacement workflow is ready on the fleet sync ticket.",
        Some("rma"),
        Some(&rma_id),
    )?;
    Ok(())
}
