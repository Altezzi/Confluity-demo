use rusqlite::Connection;

use super::DbResult;

pub fn run(conn: &Connection) -> DbResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS app_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS organizations (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            version INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS workspaces (
            id TEXT PRIMARY KEY,
            organization_id TEXT NOT NULL REFERENCES organizations(id),
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            version INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS companies (
            id TEXT PRIMARY KEY,
            workspace_id TEXT NOT NULL REFERENCES workspaces(id),
            name TEXT NOT NULL,
            domain TEXT,
            industry TEXT,
            notes TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            version INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS contacts (
            id TEXT PRIMARY KEY,
            workspace_id TEXT NOT NULL REFERENCES workspaces(id),
            company_id TEXT NOT NULL REFERENCES companies(id),
            name TEXT NOT NULL,
            email TEXT,
            title TEXT,
            phone TEXT,
            notes TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            version INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS tickets (
            id TEXT PRIMARY KEY,
            workspace_id TEXT NOT NULL REFERENCES workspaces(id),
            company_id TEXT NOT NULL REFERENCES companies(id),
            contact_id TEXT REFERENCES contacts(id),
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL,
            priority TEXT NOT NULL,
            tags TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            version INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS issues (
            id TEXT PRIMARY KEY,
            workspace_id TEXT NOT NULL REFERENCES workspaces(id),
            ticket_id TEXT REFERENCES tickets(id),
            company_id TEXT REFERENCES companies(id),
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL,
            priority TEXT NOT NULL,
            blocked_reason TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            version INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS documents (
            id TEXT PRIMARY KEY,
            workspace_id TEXT NOT NULL REFERENCES workspaces(id),
            title TEXT NOT NULL,
            body TEXT NOT NULL,
            category TEXT,
            company_id TEXT REFERENCES companies(id),
            ticket_id TEXT REFERENCES tickets(id),
            issue_id TEXT REFERENCES issues(id),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            version INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS activities (
            id TEXT PRIMARY KEY,
            workspace_id TEXT NOT NULL REFERENCES workspaces(id),
            actor TEXT NOT NULL,
            action TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            summary TEXT NOT NULL,
            metadata TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            version INTEGER NOT NULL DEFAULT 1
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS search_fts USING fts5(
            entity_type,
            entity_id UNINDEXED,
            title,
            body,
            tokenize = 'porter'
        );
        "#,
    )?;

    let mut version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if version < 1 {
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (1, datetime('now'))",
            [],
        )?;
        version = 1;
    }

    if version < 2 {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS members (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                name TEXT NOT NULL,
                role TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                name TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS comments (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                author TEXT NOT NULL,
                body TEXT NOT NULL,
                visibility TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );

            CREATE INDEX IF NOT EXISTS idx_comments_entity ON comments(entity_type, entity_id);
            "#,
        )?;
        let _ = conn.execute("ALTER TABLE tickets ADD COLUMN assignee_id TEXT", []);
        let _ = conn.execute("ALTER TABLE issues ADD COLUMN assignee_id TEXT", []);
        let _ = conn.execute("ALTER TABLE issues ADD COLUMN project_id TEXT", []);
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (2, datetime('now'))",
            [],
        )?;
        version = 2;
    }

    if version < 3 {
        let _ = conn.execute("ALTER TABLE members ADD COLUMN title TEXT", []);
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (3, datetime('now'))",
            [],
        )?;
        version = 3;
    }

    if version < 4 {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS ticket_ownership (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                ticket_id TEXT NOT NULL REFERENCES tickets(id),
                assignee_id TEXT,
                assignee_name TEXT NOT NULL,
                previous_assignee_id TEXT,
                previous_assignee_name TEXT,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_ticket_ownership_ticket
                ON ticket_ownership(ticket_id, created_at);
            "#,
        )?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (4, datetime('now'))",
            [],
        )?;
        version = 4;
    }

    if version < 5 {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS products (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                name TEXT NOT NULL,
                sku TEXT,
                description TEXT,
                inventory_qty INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS assets (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                product_id TEXT REFERENCES products(id),
                company_id TEXT REFERENCES companies(id),
                ticket_id TEXT REFERENCES tickets(id),
                name TEXT NOT NULL,
                serial TEXT,
                warranty_expires_at TEXT,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS rmas (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                company_id TEXT REFERENCES companies(id),
                asset_id TEXT REFERENCES assets(id),
                ticket_id TEXT REFERENCES tickets(id),
                status TEXT NOT NULL,
                reason TEXT,
                resolution TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS orders (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                company_id TEXT REFERENCES companies(id),
                status TEXT NOT NULL,
                reference TEXT,
                notes TEXT,
                total_cents INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS invoices (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                company_id TEXT REFERENCES companies(id),
                order_id TEXT REFERENCES orders(id),
                status TEXT NOT NULL,
                reference TEXT,
                amount_cents INTEGER NOT NULL DEFAULT 0,
                due_at TEXT,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS invoice_lines (
                id TEXT PRIMARY KEY,
                invoice_id TEXT NOT NULL REFERENCES invoices(id),
                description TEXT NOT NULL,
                quantity INTEGER NOT NULL DEFAULT 1,
                unit_cents INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS payments (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                invoice_id TEXT NOT NULL REFERENCES invoices(id),
                amount_cents INTEGER NOT NULL DEFAULT 0,
                method TEXT,
                received_at TEXT,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS vendors (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                name TEXT NOT NULL,
                email TEXT,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS purchase_orders (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                vendor_id TEXT REFERENCES vendors(id),
                product_id TEXT REFERENCES products(id),
                status TEXT NOT NULL,
                quantity INTEGER NOT NULL DEFAULT 1,
                unit_cents INTEGER NOT NULL DEFAULT 0,
                reference TEXT,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS attachments (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                filename TEXT NOT NULL,
                stored_name TEXT NOT NULL,
                mime TEXT,
                size_bytes INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL,
                actor TEXT NOT NULL,
                action TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                summary TEXT NOT NULL,
                metadata TEXT,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS outbox (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                action TEXT NOT NULL,
                payload TEXT,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                sent_at TEXT
            );
            CREATE TABLE IF NOT EXISTS notifications (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL,
                member_id TEXT,
                title TEXT NOT NULL,
                body TEXT,
                entity_type TEXT,
                entity_id TEXT,
                read_at TEXT,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS workspace_events (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                payload TEXT,
                created_at TEXT NOT NULL
            );
            "#,
        )?;
        let _ = conn.execute(
            "ALTER TABLE documents ADD COLUMN visibility TEXT NOT NULL DEFAULT 'internal'",
            [],
        );
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (5, datetime('now'))",
            [],
        )?;
        version = 5;
    }

    if version < 6 {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS roles (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                name TEXT NOT NULL,
                color TEXT NOT NULL DEFAULT '#6B7280',
                position INTEGER NOT NULL DEFAULT 0,
                is_system INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS role_permissions (
                role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
                key TEXT NOT NULL,
                granted INTEGER NOT NULL DEFAULT 1,
                PRIMARY KEY (role_id, key)
            );
            CREATE TABLE IF NOT EXISTS member_roles (
                member_id TEXT NOT NULL REFERENCES members(id) ON DELETE CASCADE,
                role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
                created_at TEXT NOT NULL,
                PRIMARY KEY (member_id, role_id)
            );
            CREATE INDEX IF NOT EXISTS idx_roles_workspace
                ON roles(workspace_id, position DESC);
            CREATE INDEX IF NOT EXISTS idx_member_roles_role
                ON member_roles(role_id);
            "#,
        )?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (6, datetime('now'))",
            [],
        )?;
        version = 6;
    }

    if version < 7 {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS teams (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                name TEXT NOT NULL,
                description TEXT,
                department TEXT,
                color TEXT NOT NULL DEFAULT '#6B7280',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS team_members (
                team_id TEXT NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
                member_id TEXT NOT NULL REFERENCES members(id) ON DELETE CASCADE,
                created_at TEXT NOT NULL,
                PRIMARY KEY (team_id, member_id)
            );
            CREATE TABLE IF NOT EXISTS team_messages (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                team_id TEXT NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
                author_id TEXT NOT NULL REFERENCES members(id),
                body TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS dm_threads (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                member_a_id TEXT NOT NULL REFERENCES members(id),
                member_b_id TEXT NOT NULL REFERENCES members(id),
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE (workspace_id, member_a_id, member_b_id)
            );
            CREATE TABLE IF NOT EXISTS dm_messages (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                thread_id TEXT NOT NULL REFERENCES dm_threads(id) ON DELETE CASCADE,
                author_id TEXT NOT NULL REFERENCES members(id),
                body TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE INDEX IF NOT EXISTS idx_teams_workspace ON teams(workspace_id);
            CREATE INDEX IF NOT EXISTS idx_team_members_member ON team_members(member_id);
            CREATE INDEX IF NOT EXISTS idx_team_messages_team ON team_messages(team_id, created_at);
            CREATE INDEX IF NOT EXISTS idx_dm_messages_thread ON dm_messages(thread_id, created_at);
            "#,
        )?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (7, datetime('now'))",
            [],
        )?;
        version = 7;
    }

    if version < 8 {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS obs_alerts (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                source TEXT NOT NULL,
                fingerprint TEXT NOT NULL,
                title TEXT NOT NULL,
                severity TEXT,
                external_url TEXT,
                raw_json TEXT NOT NULL,
                ticket_id TEXT REFERENCES tickets(id),
                asset_id TEXT REFERENCES assets(id),
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_obs_alerts_fingerprint
                ON obs_alerts(workspace_id, source, fingerprint)
                WHERE deleted_at IS NULL;
            CREATE INDEX IF NOT EXISTS idx_obs_alerts_ticket
                ON obs_alerts(ticket_id);
            "#,
        )?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (8, datetime('now'))",
            [],
        )?;
        version = 8;
    }

    if version < 9 {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS org_categories (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                name TEXT NOT NULL,
                color TEXT NOT NULL DEFAULT '#8ab4f8',
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS positions (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                category_id TEXT NOT NULL REFERENCES org_categories(id),
                name TEXT NOT NULL,
                parent_id TEXT REFERENCES positions(id),
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS position_roles (
                position_id TEXT NOT NULL REFERENCES positions(id) ON DELETE CASCADE,
                role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
                created_at TEXT NOT NULL,
                PRIMARY KEY (position_id, role_id)
            );
            CREATE INDEX IF NOT EXISTS idx_org_categories_workspace ON org_categories(workspace_id);
            CREATE INDEX IF NOT EXISTS idx_positions_category ON positions(category_id);
            CREATE INDEX IF NOT EXISTS idx_positions_parent ON positions(parent_id);
            "#,
        )?;
        // SQLite: add column if missing
        let has_position: bool = conn
            .prepare("PRAGMA table_info(members)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(|r| r.ok())
            .any(|name| name == "position_id");
        if !has_position {
            conn.execute("ALTER TABLE members ADD COLUMN position_id TEXT", [])?;
        }
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (9, datetime('now'))",
            [],
        )?;
        version = 9;
    }

    if version < 10 {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS position_role_permissions (
                position_id TEXT NOT NULL REFERENCES positions(id) ON DELETE CASCADE,
                role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
                key TEXT NOT NULL,
                PRIMARY KEY (position_id, role_id, key)
            );
            CREATE INDEX IF NOT EXISTS idx_position_role_permissions_position
                ON position_role_permissions(position_id);
            "#,
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO position_role_permissions (position_id, role_id, key)
             SELECT pr.position_id, pr.role_id, rp.key
             FROM position_roles pr
             INNER JOIN role_permissions rp ON rp.role_id = pr.role_id AND rp.granted = 1",
            [],
        )?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (10, datetime('now'))",
            [],
        )?;
        version = 10;
    }

    if version < 11 {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS team_channels (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                team_id TEXT NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                description TEXT,
                created_by TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE INDEX IF NOT EXISTS idx_team_channels_team ON team_channels(team_id);
            "#,
        )?;
        let team_cols: Vec<String> = conn
            .prepare("PRAGMA table_info(teams)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(|r| r.ok())
            .collect();
        if !team_cols.iter().any(|c| c == "kind") {
            conn.execute(
                "ALTER TABLE teams ADD COLUMN kind TEXT NOT NULL DEFAULT 'department'",
                [],
            )?;
        }
        if !team_cols.iter().any(|c| c == "created_by") {
            conn.execute("ALTER TABLE teams ADD COLUMN created_by TEXT", [])?;
        }
        let msg_cols: Vec<String> = conn
            .prepare("PRAGMA table_info(team_messages)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(|r| r.ok())
            .collect();
        if !msg_cols.iter().any(|c| c == "channel_id") {
            conn.execute("ALTER TABLE team_messages ADD COLUMN channel_id TEXT", [])?;
        }
        conn.execute(
            "UPDATE teams SET kind = CASE WHEN department IS NULL OR department = '' THEN 'chat' ELSE 'department' END
             WHERE kind IS NULL OR kind = ''",
            [],
        )?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (11, datetime('now'))",
            [],
        )?;
        version = 11;
    }

    if version < 12 {
        let team_cols: Vec<String> = conn
            .prepare("PRAGMA table_info(teams)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(|r| r.ok())
            .collect();
        if !team_cols.iter().any(|c| c == "category_id") {
            conn.execute("ALTER TABLE teams ADD COLUMN category_id TEXT", [])?;
        }
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (12, datetime('now'))",
            [],
        )?;
        version = 12;
    }

    if version < 13 {
        let pos_cols: Vec<String> = conn
            .prepare("PRAGMA table_info(positions)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(|r| r.ok())
            .collect();
        if !pos_cols.iter().any(|c| c == "see_all_chats") {
            conn.execute(
                "ALTER TABLE positions ADD COLUMN see_all_chats INTEGER NOT NULL DEFAULT 0",
                [],
            )?;
        }
        conn.execute(
            "UPDATE positions SET see_all_chats = 1
             WHERE deleted_at IS NULL AND (
               category_id IN (
                 SELECT id FROM org_categories
                 WHERE lower(name) = 'management' AND deleted_at IS NULL
               )
               OR lower(name) IN ('hr head', 'hr manager')
             )",
            [],
        )?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (13, datetime('now'))",
            [],
        )?;
        version = 13;
    }

    if version < 14 {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS live_queries (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                name TEXT NOT NULL,
                description TEXT,
                sql_text TEXT NOT NULL,
                created_by TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE INDEX IF NOT EXISTS idx_live_queries_workspace ON live_queries(workspace_id);
            "#,
        )?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (14, datetime('now'))",
            [],
        )?;
        version = 14;
    }

    if version < 15 {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS workspace_modules (
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                module_id TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                mode TEXT NOT NULL DEFAULT 'native',
                connector_id TEXT,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (workspace_id, module_id)
            );
            CREATE TABLE IF NOT EXISTS connectors (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                connector_id TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'disconnected',
                config_json TEXT NOT NULL DEFAULT '{}',
                last_sync_at TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE INDEX IF NOT EXISTS idx_connectors_workspace ON connectors(workspace_id);
            CREATE TABLE IF NOT EXISTS external_refs (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                provider TEXT NOT NULL,
                external_id TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                source_url TEXT,
                metadata_json TEXT,
                last_synced_at TEXT NOT NULL,
                sync_conflict INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE (workspace_id, provider, external_id)
            );
            CREATE INDEX IF NOT EXISTS idx_external_refs_entity
                ON external_refs(workspace_id, entity_type, entity_id);
            "#,
        )?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (15, datetime('now'))",
            [],
        )?;
        version = 15;
    }

    if version < 16 {
        conn.execute_batch(
            r#"
            ALTER TABLE members ADD COLUMN email TEXT;
            ALTER TABLE members ADD COLUMN must_set_password INTEGER NOT NULL DEFAULT 0;
            ALTER TABLE members ADD COLUMN invite_code TEXT;
            CREATE UNIQUE INDEX IF NOT EXISTS idx_members_email
                ON members(workspace_id, email)
                WHERE email IS NOT NULL AND deleted_at IS NULL;
            "#,
        )?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (16, datetime('now'))",
            [],
        )?;
        version = 16;
    }

    if version < 17 {
        conn.execute_batch(
            r#"
            ALTER TABLE tickets ADD COLUMN form_id TEXT;
            ALTER TABLE tickets ADD COLUMN custom_fields TEXT;
            CREATE TABLE IF NOT EXISTS ticket_fields (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                key TEXT NOT NULL,
                label TEXT NOT NULL,
                kind TEXT NOT NULL,
                options_json TEXT,
                hint TEXT,
                active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                UNIQUE (workspace_id, key)
            );
            CREATE TABLE IF NOT EXISTS ticket_forms (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id),
                name TEXT NOT NULL,
                description TEXT,
                active INTEGER NOT NULL DEFAULT 1,
                is_default INTEGER NOT NULL DEFAULT 0,
                position INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT
            );
            CREATE TABLE IF NOT EXISTS ticket_form_fields (
                form_id TEXT NOT NULL REFERENCES ticket_forms(id),
                field_id TEXT NOT NULL REFERENCES ticket_fields(id),
                position INTEGER NOT NULL,
                required INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (form_id, field_id)
            );
            "#,
        )?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (17, datetime('now'))",
            [],
        )?;
    }

    Ok(())
}
