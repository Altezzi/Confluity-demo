use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::{new_id, now_iso, DbError, DbResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub color: String,
    pub position: i64,
    pub is_system: bool,
    pub permissions: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

/// Every grantable permission key (mirrors packages/shared/src/permissions.ts).
pub fn all_permission_keys() -> Vec<&'static str> {
    let mut keys = vec![
        "module.dashboard.view",
        "module.dashboard.edit",
        "module.crm.view",
        "module.crm.edit",
        "module.support.view",
        "module.support.edit",
        "module.projects.view",
        "module.projects.edit",
        "module.operations.view",
        "module.operations.edit",
        "module.finance.view",
        "module.finance.edit",
        "module.portal.view",
        "module.portal.edit",
        "module.docs.view",
        "module.docs.edit",
        "module.insights.view",
        "module.insights.edit",
        "module.teams.view",
        "module.teams.edit",
        "module.search.view",
        "module.search.edit",
        "module.settings.view",
        "module.settings.edit",
        "panel.comments",
        "panel.attachments",
        "panel.linked_records",
        "crm.field.notes",
        "crm.field.contact_pii",
        "support.field.internal_comments",
        "support.field.attachments",
        "projects.field.blocked_reason",
        "operations.field.serial_numbers",
        "finance.field.amounts",
        "finance.field.line_items",
        "docs.field.internal",
        "teams.manage",
        "teams.message",
        "teams.see_all_chats",
        "settings.preferences",
        "settings.manage_roles",
        "settings.manage_members",
        "settings.integrations",
        "settings.ai",
        "settings.data_server",
        "settings.workspace_admin",
    ];
    keys.sort_unstable();
    keys.dedup();
    keys
}

fn viewer_keys() -> Vec<&'static str> {
    vec![
        "module.dashboard.view",
        "module.crm.view",
        "module.support.view",
        "module.projects.view",
        "module.operations.view",
        "module.finance.view",
        "module.portal.view",
        "module.docs.view",
        "module.teams.view",
        "module.search.view",
        "module.settings.view",
        "settings.preferences",
        "panel.linked_records",
        "teams.message",
    ]
}

fn support_keys() -> Vec<&'static str> {
    let mut keys = viewer_keys();
    keys.extend([
        "module.crm.edit",
        "module.support.edit",
        "module.docs.edit",
        "module.portal.edit",
        "module.insights.view",
        "module.insights.edit",
        "module.teams.edit",
        "module.search.edit",
        "panel.comments",
        "panel.attachments",
        "crm.field.notes",
        "crm.field.contact_pii",
        "support.field.internal_comments",
        "support.field.attachments",
        "docs.field.internal",
    ]);
    keys
}

fn engineering_keys() -> Vec<&'static str> {
    let mut keys = viewer_keys();
    keys.extend([
        "module.projects.edit",
        "module.operations.edit",
        "module.docs.edit",
        "module.support.view",
        "module.insights.view",
        "module.insights.edit",
        "module.teams.edit",
        "module.search.edit",
        "panel.comments",
        "panel.attachments",
        "projects.field.blocked_reason",
        "operations.field.serial_numbers",
        "docs.field.internal",
    ]);
    keys
}

fn finance_keys() -> Vec<&'static str> {
    let mut keys = viewer_keys();
    keys.extend([
        "module.finance.edit",
        "module.crm.edit",
        "module.portal.edit",
        "module.docs.view",
        "module.insights.view",
        "module.teams.edit",
        "module.search.edit",
        "panel.comments",
        "panel.attachments",
        "finance.field.amounts",
        "finance.field.line_items",
        "crm.field.notes",
        "crm.field.contact_pii",
    ]);
    keys
}

fn management_keys() -> Vec<&'static str> {
    let mut keys = all_permission_keys();
    // Management runs the company chart, not app data plumbing / workspace admin.
    keys.retain(|k| {
        *k != "settings.manage_roles"
            && *k != "settings.data_server"
            && *k != "settings.integrations"
            && *k != "settings.ai"
            && *k != "settings.workspace_admin"
    });
    keys
}

pub fn list_roles(conn: &Connection, workspace_id: &str) -> DbResult<Vec<Role>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, name, color, position, is_system, created_at, updated_at, deleted_at, version
         FROM roles WHERE workspace_id = ?1 AND deleted_at IS NULL
         ORDER BY position DESC, name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map(params![workspace_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, Option<String>>(8)?,
            row.get::<_, i64>(9)?,
        ))
    })?;

    let mut roles = Vec::new();
    for row in rows {
        let (id, ws, name, color, position, is_system, created_at, updated_at, deleted_at, version) =
            row?;
        let permissions = list_role_permission_keys(conn, &id)?;
        roles.push(Role {
            id,
            workspace_id: ws,
            name,
            color,
            position,
            is_system: is_system != 0,
            permissions,
            created_at,
            updated_at,
            deleted_at,
            version,
        });
    }
    Ok(roles)
}

pub fn get_role(conn: &Connection, id: &str) -> DbResult<Option<Role>> {
    let row = conn
        .query_row(
            "SELECT id, workspace_id, name, color, position, is_system, created_at, updated_at, deleted_at, version
             FROM roles WHERE id = ?1 AND deleted_at IS NULL",
            params![id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, i64>(9)?,
                ))
            },
        )
        .optional()?;
    let Some((id, ws, name, color, position, is_system, created_at, updated_at, deleted_at, version)) =
        row
    else {
        return Ok(None);
    };
    Ok(Some(Role {
        id: id.clone(),
        workspace_id: ws,
        name,
        color,
        position,
        is_system: is_system != 0,
        permissions: list_role_permission_keys(conn, &id)?,
        created_at,
        updated_at,
        deleted_at,
        version,
    }))
}

pub fn list_role_permission_keys(conn: &Connection, role_id: &str) -> DbResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT key FROM role_permissions WHERE role_id = ?1 AND granted = 1 ORDER BY key",
    )?;
    let rows = stmt.query_map(params![role_id], |row| row.get::<_, String>(0))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn set_role_permissions(conn: &Connection, role_id: &str, keys: &[String]) -> DbResult<()> {
    conn.execute(
        "DELETE FROM role_permissions WHERE role_id = ?1",
        params![role_id],
    )?;
    let mut unique = keys.to_vec();
    unique.sort_unstable();
    unique.dedup();
    for key in unique {
        conn.execute(
            "INSERT INTO role_permissions (role_id, key, granted) VALUES (?1, ?2, 1)",
            params![role_id, key],
        )?;
    }
    Ok(())
}

pub fn member_role_ids(conn: &Connection, member_id: &str) -> DbResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT mr.role_id FROM member_roles mr
         INNER JOIN roles r ON r.id = mr.role_id AND r.deleted_at IS NULL
         WHERE mr.member_id = ?1
         ORDER BY r.position DESC",
    )?;
    let rows = stmt.query_map(params![member_id], |row| row.get::<_, String>(0))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn set_member_roles(conn: &Connection, member_id: &str, role_ids: &[String]) -> DbResult<()> {
    conn.execute(
        "DELETE FROM member_roles WHERE member_id = ?1",
        params![member_id],
    )?;
    let ts = now_iso();
    for role_id in role_ids {
        conn.execute(
            "INSERT INTO member_roles (member_id, role_id, created_at) VALUES (?1, ?2, ?3)",
            params![member_id, role_id, ts],
        )?;
    }
    refresh_member_display_role(conn, member_id)?;
    Ok(())
}

pub fn refresh_member_display_role(conn: &Connection, member_id: &str) -> DbResult<()> {
    let name: Option<String> = conn
        .query_row(
            "SELECT r.name FROM member_roles mr
             INNER JOIN roles r ON r.id = mr.role_id AND r.deleted_at IS NULL
             WHERE mr.member_id = ?1
             ORDER BY r.position DESC, r.name COLLATE NOCASE
             LIMIT 1",
            params![member_id],
            |row| row.get(0),
        )
        .optional()?;
    let label = name.unwrap_or_else(|| "viewer".into());
    let ts = now_iso();
    conn.execute(
        "UPDATE members SET role = ?1, updated_at = ?2, version = version + 1 WHERE id = ?3",
        params![label.to_lowercase(), ts, member_id],
    )?;
    Ok(())
}

pub fn expand_effective(granted: &[String]) -> Vec<String> {
    let mut out: Vec<String> = granted.to_vec();
    for key in granted {
        if let Some(rest) = key.strip_prefix("module.") {
            if let Some(module) = rest.strip_suffix(".edit") {
                let view = format!("module.{module}.view");
                if !out.iter().any(|k| k == &view) {
                    out.push(view);
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

pub fn effective_permissions_for_member(
    conn: &Connection,
    member_id: &str,
) -> DbResult<Vec<String>> {
    let position_id: Option<String> = conn
        .query_row(
            "SELECT position_id FROM members WHERE id = ?1 AND deleted_at IS NULL",
            params![member_id],
            |row| row.get(0),
        )
        .optional()?
        .flatten();

    if let Some(pid) = position_id.filter(|id| !id.is_empty()) {
        let role_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM position_roles WHERE position_id = ?1",
            params![pid],
            |row| row.get(0),
        )?;
        if role_count > 0 {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT prp.key FROM position_role_permissions prp
                 INNER JOIN position_roles pr
                    ON pr.position_id = prp.position_id AND pr.role_id = prp.role_id
                 INNER JOIN roles r ON r.id = prp.role_id AND r.deleted_at IS NULL
                 INNER JOIN role_permissions rp
                    ON rp.role_id = prp.role_id AND rp.key = prp.key AND rp.granted = 1
                 WHERE prp.position_id = ?1",
            )?;
            let rows = stmt.query_map(params![pid], |row| row.get::<_, String>(0))?;
            let granted: Vec<String> = rows.filter_map(|r| r.ok()).collect();
            return Ok(expand_effective(&granted));
        }
    }

    let mut stmt = conn.prepare(
        "SELECT DISTINCT rp.key FROM member_roles mr
         INNER JOIN roles r ON r.id = mr.role_id AND r.deleted_at IS NULL
         INNER JOIN role_permissions rp ON rp.role_id = mr.role_id AND rp.granted = 1
         WHERE mr.member_id = ?1",
    )?;
    let rows = stmt.query_map(params![member_id], |row| row.get::<_, String>(0))?;
    let granted: Vec<String> = rows.filter_map(|r| r.ok()).collect();
    Ok(expand_effective(&granted))
}

pub fn member_has_permission(conn: &Connection, member_id: &str, key: &str) -> DbResult<bool> {
    let perms = effective_permissions_for_member(conn, member_id)?;
    Ok(perms.iter().any(|p| p == key))
}

pub fn find_role_by_name(
    conn: &Connection,
    workspace_id: &str,
    name: &str,
) -> DbResult<Option<Role>> {
    let id: Option<String> = conn
        .query_row(
            "SELECT id FROM roles WHERE workspace_id = ?1 AND lower(name) = lower(?2) AND deleted_at IS NULL",
            params![workspace_id, name],
            |row| row.get(0),
        )
        .optional()?;
    match id {
        Some(id) => get_role(conn, &id),
        None => Ok(None),
    }
}

fn insert_role(
    conn: &Connection,
    workspace_id: &str,
    name: &str,
    color: &str,
    position: i64,
    is_system: bool,
    keys: &[&str],
) -> DbResult<String> {
    if let Some(existing) = find_role_by_name(conn, workspace_id, name)? {
        return Ok(existing.id);
    }
    let id = new_id();
    let ts = now_iso();
    conn.execute(
        "INSERT INTO roles (id, workspace_id, name, color, position, is_system, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, NULL, 1)",
        params![id, workspace_id, name, color, position, if is_system { 1 } else { 0 }, ts],
    )?;
    let key_strings: Vec<String> = keys.iter().map(|k| (*k).to_string()).collect();
    set_role_permissions(conn, &id, &key_strings)?;
    Ok(id)
}

/// Ensure default roles exist and backfill member_roles from legacy members.role strings.
pub fn ensure_default_roles(conn: &Connection, workspace_id: &str) -> DbResult<()> {
    let admin_id = insert_role(
        conn,
        workspace_id,
        "Admin",
        "#E74C3C",
        100,
        true,
        &all_permission_keys(),
    )?;
    // Keep Admin in sync when new catalog keys are added
    if let Ok(Some(admin)) = get_role(conn, &admin_id) {
        let _ = set_role_permissions(
            conn,
            &admin.id,
            &all_permission_keys()
                .into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        );
    }
    let _management_id = insert_role(
        conn,
        workspace_id,
        "Management",
        "#F39C12",
        80,
        false,
        &management_keys(),
    )?;
    let support_id = insert_role(
        conn,
        workspace_id,
        "Support",
        "#3498DB",
        50,
        false,
        &support_keys(),
    )?;
    let engineering_id = insert_role(
        conn,
        workspace_id,
        "Engineering",
        "#9B59B6",
        40,
        false,
        &engineering_keys(),
    )?;
    let finance_id = insert_role(
        conn,
        workspace_id,
        "Finance",
        "#1ABC9C",
        45,
        false,
        &finance_keys(),
    )?;
    let viewer_id = insert_role(
        conn,
        workspace_id,
        "Viewer",
        "#95A5A6",
        10,
        false,
        &viewer_keys(),
    )?;

    // Backfill new catalog keys onto seeded non-system roles (idempotent add-only)
    for (role_id, defaults) in [
        (&support_id, support_keys()),
        (&engineering_id, engineering_keys()),
        (&finance_id, finance_keys()),
        (&viewer_id, viewer_keys()),
        (&_management_id, management_keys()),
    ] {
        let existing = list_role_permission_keys(conn, role_id)?;
        let mut set: std::collections::BTreeSet<String> = existing.into_iter().collect();
        let before = set.len();
        for k in defaults {
            set.insert(k.to_string());
        }
        if set.len() != before {
            let _ = set_role_permissions(conn, role_id, &set.into_iter().collect::<Vec<_>>());
        }
    }

    // Clamp Management away from data-source / admin plumbing (idempotent)
    if let Ok(Some(mgmt)) = find_role_by_name(conn, workspace_id, "Management") {
        let existing = list_role_permission_keys(conn, &mgmt.id)?;
        let clamped: Vec<String> = existing
            .into_iter()
            .filter(|k| {
                k != "settings.manage_roles"
                    && k != "settings.data_server"
                    && k != "settings.integrations"
                    && k != "settings.ai"
                    && k != "settings.workspace_admin"
            })
            .collect();
        let _ = set_role_permissions(conn, &mgmt.id, &clamped);
    }

    let mut stmt = conn.prepare(
        "SELECT id, role FROM members WHERE workspace_id = ?1 AND deleted_at IS NULL",
    )?;
    let members: Vec<(String, String)> = stmt
        .query_map(params![workspace_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    for (member_id, legacy_role) in members {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM member_roles WHERE member_id = ?1",
            params![member_id],
            |row| row.get(0),
        )?;
        if count > 0 {
            continue;
        }
        let role_id = match legacy_role.to_lowercase().as_str() {
            "admin" => admin_id.as_str(),
            "support" => support_id.as_str(),
            "engineering" => engineering_id.as_str(),
            _ => viewer_id.as_str(),
        };
        set_member_roles(conn, &member_id, &[role_id.to_string()])?;
    }

    Ok(())
}

/// Restore factory roles (Admin, Management, Support, Engineering, Finance, Viewer)
/// and remove custom roles. Position permission subsets for kept roles are reset too.
pub fn reset_default_roles(conn: &Connection, workspace_id: &str) -> DbResult<Vec<Role>> {
    let defaults: [(&str, &str, i64, bool, Vec<&str>); 6] = [
        ("Admin", "#E74C3C", 100, true, all_permission_keys()),
        ("Management", "#F39C12", 80, false, management_keys()),
        ("Support", "#3498DB", 50, false, support_keys()),
        ("Engineering", "#9B59B6", 40, false, engineering_keys()),
        ("Finance", "#1ABC9C", 45, false, finance_keys()),
        ("Viewer", "#95A5A6", 10, false, viewer_keys()),
    ];

    let mut keep_ids: Vec<String> = Vec::new();
    for (name, color, position, is_system, keys) in defaults {
        keep_ids.push(upsert_default_role(
            conn,
            workspace_id,
            name,
            color,
            position,
            is_system,
            &keys,
        )?);
    }

    let existing = list_roles(conn, workspace_id)?;
    for role in existing {
        if keep_ids.iter().any(|id| id == &role.id) {
            continue;
        }
        conn.execute(
            "DELETE FROM position_role_permissions WHERE role_id = ?1",
            params![role.id],
        )?;
        conn.execute("DELETE FROM position_roles WHERE role_id = ?1", params![role.id])?;
        delete_role(conn, &role.id)?;
    }

    for id in &keep_ids {
        conn.execute(
            "DELETE FROM position_role_permissions WHERE role_id = ?1",
            params![id],
        )?;
        conn.execute(
            "INSERT INTO position_role_permissions (position_id, role_id, key)
             SELECT pr.position_id, pr.role_id, rp.key
             FROM position_roles pr
             INNER JOIN role_permissions rp ON rp.role_id = pr.role_id AND rp.granted = 1
             WHERE pr.role_id = ?1",
            params![id],
        )?;
    }

    if let Some(admin_id) = keep_ids.first() {
        if let Ok(Some(member_id)) = super::get_meta(conn, "setup_admin_id") {
            let mut ids = member_role_ids(conn, &member_id)?;
            if !ids.iter().any(|id| id == admin_id) {
                ids.push(admin_id.clone());
                set_member_roles(conn, &member_id, &ids)?;
            }
        }
    }

    list_roles(conn, workspace_id)
}

fn upsert_default_role(
    conn: &Connection,
    workspace_id: &str,
    name: &str,
    color: &str,
    position: i64,
    is_system: bool,
    keys: &[&str],
) -> DbResult<String> {
    let ts = now_iso();
    let key_strings: Vec<String> = keys.iter().map(|k| (*k).to_string()).collect();
    let existing_id: Option<String> = conn
        .query_row(
            "SELECT id FROM roles
             WHERE workspace_id = ?1 AND lower(name) = lower(?2)
             ORDER BY CASE WHEN deleted_at IS NULL THEN 0 ELSE 1 END, created_at
             LIMIT 1",
            params![workspace_id, name],
            |row| row.get(0),
        )
        .optional()?;

    if let Some(id) = existing_id {
        conn.execute(
            "UPDATE roles SET name = ?1, color = ?2, position = ?3, is_system = ?4,
             deleted_at = NULL, updated_at = ?5, version = version + 1
             WHERE id = ?6",
            params![
                name,
                color,
                position,
                if is_system { 1 } else { 0 },
                ts,
                id
            ],
        )?;
        set_role_permissions(conn, &id, &key_strings)?;
        return Ok(id);
    }

    insert_role(conn, workspace_id, name, color, position, is_system, keys)
}

pub fn next_role_position(conn: &Connection, workspace_id: &str) -> DbResult<i64> {
    let max: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(position), 0) FROM roles WHERE workspace_id = ?1 AND deleted_at IS NULL",
            params![workspace_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    Ok(max + 10)
}

pub fn create_role(
    conn: &Connection,
    workspace_id: &str,
    name: &str,
    color: &str,
    permissions: &[String],
) -> DbResult<Role> {
    let id = new_id();
    let ts = now_iso();
    let position = next_role_position(conn, workspace_id)?;
    conn.execute(
        "INSERT INTO roles (id, workspace_id, name, color, position, is_system, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?6, NULL, 1)",
        params![id, workspace_id, name, color, position, ts],
    )?;
    set_role_permissions(conn, &id, permissions)?;
    get_role(conn, &id)?.ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}

pub fn update_role(
    conn: &Connection,
    id: &str,
    name: &str,
    color: &str,
    position: i64,
) -> DbResult<Role> {
    let ts = now_iso();
    let existing = get_role(conn, id)?.ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))?;
    let name = if existing.is_system {
        existing.name.clone()
    } else {
        name.to_string()
    };
    conn.execute(
        "UPDATE roles SET name = ?1, color = ?2, position = ?3, updated_at = ?4, version = version + 1
         WHERE id = ?5 AND deleted_at IS NULL",
        params![name, color, position, ts, id],
    )?;
    // Refresh display labels for members who have this role
    let mut stmt = conn.prepare("SELECT member_id FROM member_roles WHERE role_id = ?1")?;
    let member_ids: Vec<String> = stmt
        .query_map(params![id], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();
    for mid in member_ids {
        refresh_member_display_role(conn, &mid)?;
    }
    get_role(conn, id)?.ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}

pub fn delete_role(conn: &Connection, id: &str) -> DbResult<()> {
    let existing = get_role(conn, id)?.ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))?;
    if existing.is_system {
        return Err(DbError::Sqlite(rusqlite::Error::InvalidParameterName(
            "cannot delete system role".into(),
        )));
    }
    let ts = now_iso();
    let member_ids = {
        let mut stmt = conn.prepare("SELECT member_id FROM member_roles WHERE role_id = ?1")?;
        let ids: Vec<String> = stmt
            .query_map(params![id], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        ids
    };
    conn.execute("DELETE FROM role_permissions WHERE role_id = ?1", params![id])?;
    conn.execute("DELETE FROM member_roles WHERE role_id = ?1", params![id])?;
    conn.execute(
        "UPDATE roles SET deleted_at = ?1, updated_at = ?1, version = version + 1 WHERE id = ?2",
        params![ts, id],
    )?;
    for mid in member_ids {
        refresh_member_display_role(conn, &mid)?;
    }
    Ok(())
}

pub fn swap_role_positions(conn: &Connection, a: &str, b: &str) -> DbResult<()> {
    let role_a = get_role(conn, a)?.ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))?;
    let role_b = get_role(conn, b)?.ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))?;
    let ts = now_iso();
    conn.execute(
        "UPDATE roles SET position = ?1, updated_at = ?2, version = version + 1 WHERE id = ?3",
        params![role_b.position, ts, a],
    )?;
    conn.execute(
        "UPDATE roles SET position = ?1, updated_at = ?2, version = version + 1 WHERE id = ?3",
        params![role_a.position, ts, b],
    )?;
    Ok(())
}
