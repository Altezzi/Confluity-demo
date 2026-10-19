use std::collections::{BTreeMap, BTreeSet};

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::roles::{list_role_permission_keys, set_member_roles, Role};
use super::{new_id, now_iso, DbError, DbResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgCategory {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub color: String,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: String,
    pub workspace_id: String,
    pub category_id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub sort_order: i64,
    #[serde(default)]
    pub role_ids: Vec<String>,
    #[serde(default)]
    pub role_permissions: BTreeMap<String, Vec<String>>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

fn map_category(row: &rusqlite::Row<'_>) -> rusqlite::Result<OrgCategory> {
    Ok(OrgCategory {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        name: row.get(2)?,
        color: row.get(3)?,
        sort_order: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
        deleted_at: row.get(7)?,
        version: row.get(8)?,
    })
}

fn map_position(row: &rusqlite::Row<'_>) -> rusqlite::Result<Position> {
    Ok(Position {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        category_id: row.get(2)?,
        name: row.get(3)?,
        parent_id: row.get(4)?,
        sort_order: row.get(5)?,
        role_ids: Vec::new(),
        role_permissions: BTreeMap::new(),
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        deleted_at: row.get(8)?,
        version: row.get(9)?,
    })
}

const CAT_COLS: &str =
    "id, workspace_id, name, color, sort_order, created_at, updated_at, deleted_at, version";
const POS_COLS: &str = "id, workspace_id, category_id, name, parent_id, sort_order, created_at, updated_at, deleted_at, version";

pub fn list_categories(conn: &Connection, workspace_id: &str) -> DbResult<Vec<OrgCategory>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {CAT_COLS} FROM org_categories
         WHERE workspace_id = ?1 AND deleted_at IS NULL
         ORDER BY sort_order ASC, name COLLATE NOCASE"
    ))?;
    let rows = stmt.query_map(params![workspace_id], map_category)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_category(conn: &Connection, id: &str) -> DbResult<Option<OrgCategory>> {
    conn.query_row(
        &format!("SELECT {CAT_COLS} FROM org_categories WHERE id = ?1 AND deleted_at IS NULL"),
        params![id],
        map_category,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn create_category(
    conn: &Connection,
    workspace_id: &str,
    name: &str,
    color: &str,
) -> DbResult<OrgCategory> {
    let id = new_id();
    let ts = now_iso();
    let max: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM org_categories WHERE workspace_id = ?1 AND deleted_at IS NULL",
            params![workspace_id],
            |row| row.get(0),
        )
        .unwrap_or(-1);
    conn.execute(
        "INSERT INTO org_categories (id, workspace_id, name, color, sort_order, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, NULL, 1)",
        params![id, workspace_id, name.trim(), color, max + 1, ts],
    )?;
    let cat = get_category(conn, &id)?.ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))?;
    super::teams::ensure_category_teams(conn, workspace_id)?;
    Ok(cat)
}

pub fn update_category(conn: &Connection, id: &str, name: &str, color: &str) -> DbResult<OrgCategory> {
    let ts = now_iso();
    conn.execute(
        "UPDATE org_categories SET name = ?1, color = ?2, updated_at = ?3, version = version + 1
         WHERE id = ?4 AND deleted_at IS NULL",
        params![name.trim(), color, ts, id],
    )?;
    let cat = get_category(conn, id)?.ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))?;
    super::teams::ensure_category_teams(conn, &cat.workspace_id)?;
    Ok(cat)
}

pub fn delete_category(conn: &Connection, id: &str) -> DbResult<()> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM positions WHERE category_id = ?1 AND deleted_at IS NULL",
        params![id],
        |row| row.get(0),
    )?;
    if count > 0 {
        return Err(DbError::Sqlite(rusqlite::Error::InvalidParameterName(
            "category still has positions".into(),
        )));
    }
    let ts = now_iso();
    conn.execute(
        "UPDATE org_categories SET deleted_at = ?1, updated_at = ?1, version = version + 1 WHERE id = ?2",
        params![ts, id],
    )?;
    Ok(())
}

pub fn position_role_ids(conn: &Connection, position_id: &str) -> DbResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT pr.role_id FROM position_roles pr
         INNER JOIN roles r ON r.id = pr.role_id AND r.deleted_at IS NULL
         WHERE pr.position_id = ?1
         ORDER BY r.position DESC",
    )?;
    let rows = stmt.query_map(params![position_id], |row| row.get::<_, String>(0))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn position_role_permissions(
    conn: &Connection,
    position_id: &str,
) -> DbResult<BTreeMap<String, Vec<String>>> {
    let mut stmt = conn.prepare(
        "SELECT role_id, key FROM position_role_permissions
         WHERE position_id = ?1
         ORDER BY role_id, key",
    )?;
    let rows = stmt.query_map(params![position_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in rows {
        let (role_id, key) = row?;
        map.entry(role_id).or_default().push(key);
    }
    Ok(map)
}

fn hydrate_position(conn: &Connection, p: &mut Position) -> DbResult<()> {
    p.role_ids = position_role_ids(conn, &p.id)?;
    p.role_permissions = position_role_permissions(conn, &p.id)?;
    Ok(())
}

fn seed_role_grants(conn: &Connection, position_id: &str, role_id: &str) -> DbResult<()> {
    for key in list_role_permission_keys(conn, role_id)? {
        conn.execute(
            "INSERT OR IGNORE INTO position_role_permissions (position_id, role_id, key)
             VALUES (?1, ?2, ?3)",
            params![position_id, role_id, key],
        )?;
    }
    Ok(())
}

pub fn list_positions(conn: &Connection, workspace_id: &str) -> DbResult<Vec<Position>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {POS_COLS} FROM positions
         WHERE workspace_id = ?1 AND deleted_at IS NULL
         ORDER BY sort_order ASC, name COLLATE NOCASE"
    ))?;
    let rows = stmt.query_map(params![workspace_id], map_position)?;
    let mut positions: Vec<Position> = rows.filter_map(|r| r.ok()).collect();
    for p in &mut positions {
        hydrate_position(conn, p)?;
    }
    Ok(positions)
}

pub fn get_position(conn: &Connection, id: &str) -> DbResult<Option<Position>> {
    let mut pos = conn
        .query_row(
            &format!("SELECT {POS_COLS} FROM positions WHERE id = ?1 AND deleted_at IS NULL"),
            params![id],
            map_position,
        )
        .optional()
        .map_err(DbError::from)?;
    if let Some(ref mut p) = pos {
        hydrate_position(conn, p)?;
    }
    Ok(pos)
}

pub fn create_position(
    conn: &Connection,
    workspace_id: &str,
    category_id: &str,
    name: &str,
    parent_id: Option<&str>,
) -> DbResult<Position> {
    if let Some(pid) = parent_id {
        let parent = get_position(conn, pid)?
            .ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))?;
        if parent.category_id != category_id {
            return Err(DbError::Sqlite(rusqlite::Error::InvalidParameterName(
                "parent must be in the same category".into(),
            )));
        }
    }
    let id = new_id();
    let ts = now_iso();
    let max: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM positions
             WHERE category_id = ?1 AND deleted_at IS NULL
               AND IFNULL(parent_id, '') = IFNULL(?2, '')",
            params![category_id, parent_id],
            |row| row.get(0),
        )
        .unwrap_or(-1);
    conn.execute(
        "INSERT INTO positions (id, workspace_id, category_id, name, parent_id, sort_order, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, NULL, 1)",
        params![id, workspace_id, category_id, name.trim(), parent_id, max + 1, ts],
    )?;
    get_position(conn, &id)?.ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}

pub fn update_position(
    conn: &Connection,
    id: &str,
    name: &str,
    parent_id: Option<&str>,
) -> DbResult<Position> {
    let existing = get_position(conn, id)?
        .ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))?;
    if let Some(pid) = parent_id {
        if pid == id {
            return Err(DbError::Sqlite(rusqlite::Error::InvalidParameterName(
                "position cannot be its own parent".into(),
            )));
        }
        let parent = get_position(conn, pid)?
            .ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))?;
        if parent.category_id != existing.category_id {
            return Err(DbError::Sqlite(rusqlite::Error::InvalidParameterName(
                "parent must be in the same category".into(),
            )));
        }
    }
    let ts = now_iso();
    conn.execute(
        "UPDATE positions SET name = ?1, parent_id = ?2, updated_at = ?3, version = version + 1
         WHERE id = ?4 AND deleted_at IS NULL",
        params![name.trim(), parent_id, ts, id],
    )?;
    // Refresh denormalized titles for holders
    conn.execute(
        "UPDATE members SET title = ?1, updated_at = ?2, version = version + 1 WHERE position_id = ?3",
        params![name.trim(), ts, id],
    )?;
    get_position(conn, id)?.ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}

pub fn delete_position(conn: &Connection, id: &str) -> DbResult<()> {
    let children: i64 = conn.query_row(
        "SELECT COUNT(*) FROM positions WHERE parent_id = ?1 AND deleted_at IS NULL",
        params![id],
        |row| row.get(0),
    )?;
    if children > 0 {
        return Err(DbError::Sqlite(rusqlite::Error::InvalidParameterName(
            "position still has child positions".into(),
        )));
    }
    let holders: i64 = conn.query_row(
        "SELECT COUNT(*) FROM members WHERE position_id = ?1 AND deleted_at IS NULL",
        params![id],
        |row| row.get(0),
    )?;
    if holders > 0 {
        return Err(DbError::Sqlite(rusqlite::Error::InvalidParameterName(
            "position still has members assigned".into(),
        )));
    }
    let ts = now_iso();
    conn.execute(
        "UPDATE positions SET deleted_at = ?1, updated_at = ?1, version = version + 1 WHERE id = ?2",
        params![ts, id],
    )?;
    Ok(())
}

fn resync_position_members(conn: &Connection, position_id: &str, role_ids: &[String]) -> DbResult<()> {
    let mut stmt = conn.prepare(
        "SELECT id FROM members WHERE position_id = ?1 AND deleted_at IS NULL",
    )?;
    let member_ids: Vec<String> = stmt
        .query_map(params![position_id], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);
    for mid in member_ids {
        set_member_roles(conn, &mid, role_ids)?;
    }
    let ts = now_iso();
    conn.execute(
        "UPDATE positions SET updated_at = ?1, version = version + 1 WHERE id = ?2",
        params![ts, position_id],
    )?;
    if let Ok(ws) = conn.query_row(
        "SELECT workspace_id FROM positions WHERE id = ?1",
        params![position_id],
        |row| row.get::<_, String>(0),
    ) {
        super::teams::ensure_category_teams(conn, &ws)?;
    }
    Ok(())
}

pub fn set_position_roles(conn: &Connection, position_id: &str, role_ids: &[String]) -> DbResult<()> {
    get_position(conn, position_id)?
        .ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))?;
    let previous: BTreeSet<String> = position_role_ids(conn, position_id)?.into_iter().collect();
    let next: BTreeSet<String> = role_ids.iter().cloned().collect();

    conn.execute(
        "DELETE FROM position_roles WHERE position_id = ?1",
        params![position_id],
    )?;
    let ts = now_iso();
    for role_id in role_ids {
        conn.execute(
            "INSERT INTO position_roles (position_id, role_id, created_at) VALUES (?1, ?2, ?3)",
            params![position_id, role_id, ts],
        )?;
        if !previous.contains(role_id) {
            seed_role_grants(conn, position_id, role_id)?;
        }
    }
    for removed in previous.difference(&next) {
        conn.execute(
            "DELETE FROM position_role_permissions WHERE position_id = ?1 AND role_id = ?2",
            params![position_id, removed],
        )?;
    }
    resync_position_members(conn, position_id, role_ids)
}

pub fn set_position_role_permissions(
    conn: &Connection,
    position_id: &str,
    role_id: &str,
    permissions: &[String],
) -> DbResult<()> {
    let pos = get_position(conn, position_id)?
        .ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))?;
    if !pos.role_ids.iter().any(|id| id == role_id) {
        return Err(DbError::Sqlite(rusqlite::Error::InvalidParameterName(
            "role is not attached to this position".into(),
        )));
    }
    let allowed: BTreeSet<String> = list_role_permission_keys(conn, role_id)?
        .into_iter()
        .collect();
    conn.execute(
        "DELETE FROM position_role_permissions WHERE position_id = ?1 AND role_id = ?2",
        params![position_id, role_id],
    )?;
    let mut unique = permissions.to_vec();
    unique.sort_unstable();
    unique.dedup();
    for key in unique {
        if !allowed.contains(&key) {
            continue;
        }
        conn.execute(
            "INSERT INTO position_role_permissions (position_id, role_id, key) VALUES (?1, ?2, ?3)",
            params![position_id, role_id, key],
        )?;
    }
    resync_position_members(conn, position_id, &pos.role_ids)
}

pub fn set_member_position(
    conn: &Connection,
    member_id: &str,
    position_id: Option<&str>,
) -> DbResult<()> {
    let ts = now_iso();
    let ws: String = conn.query_row(
        "SELECT workspace_id FROM members WHERE id = ?1",
        params![member_id],
        |row| row.get(0),
    )?;
    if let Some(pid) = position_id {
        let pos = get_position(conn, pid)?
            .ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))?;
        conn.execute(
            "UPDATE members SET position_id = ?1, title = ?2, updated_at = ?3, version = version + 1 WHERE id = ?4",
            params![pid, pos.name, ts, member_id],
        )?;
        set_member_roles(conn, member_id, &pos.role_ids)?;
    } else {
        conn.execute(
            "UPDATE members SET position_id = NULL, title = NULL, updated_at = ?1, version = version + 1 WHERE id = ?2",
            params![ts, member_id],
        )?;
        set_member_roles(conn, member_id, &[])?;
    }
    super::teams::ensure_category_teams(conn, &ws)?;
    Ok(())
}

/// Seed default categories + position trees and map members from legacy titles.
pub fn ensure_default_positions(conn: &Connection, workspace_id: &str) -> DbResult<()> {
    // Prefer Admin / Support / Engineering / Viewer / Management / Finance role ids
    let roles: Vec<Role> = super::roles::list_roles(conn, workspace_id)?;
    let find_role = |name: &str| -> Option<String> {
        roles
            .iter()
            .find(|r| r.name.eq_ignore_ascii_case(name))
            .map(|r| r.id.clone())
    };
    let admin = find_role("Admin");
    let support = find_role("Support");
    let engineering = find_role("Engineering");
    let management = find_role("Management");
    let finance = find_role("Finance");
    let viewer = find_role("Viewer");

    let defs: &[(&str, &str, &[(&str, Option<&str>, Option<&str>)])] = &[
        (
            "Support",
            "#2ECC71",
            &[
                ("Support Head", None, management.as_deref().or(admin.as_deref())),
                ("Support Manager", Some("Support Head"), support.as_deref()),
                ("Support Member", Some("Support Manager"), support.as_deref().or(viewer.as_deref())),
            ],
        ),
        (
            "Engineering",
            "#9B59B6",
            &[
                ("Engineering Head", None, management.as_deref().or(admin.as_deref())),
                (
                    "Engineering Manager",
                    Some("Engineering Head"),
                    engineering.as_deref(),
                ),
                (
                    "Engineering Member",
                    Some("Engineering Manager"),
                    engineering.as_deref().or(viewer.as_deref()),
                ),
            ],
        ),
        (
            "Management",
            "#F39C12",
            &[
                ("Director", None, management.as_deref().or(admin.as_deref())),
                ("Manager", Some("Director"), management.as_deref()),
            ],
        ),
        (
            "HR",
            "#3498DB",
            &[
                ("HR Head", None, management.as_deref().or(admin.as_deref())),
                ("HR Manager", Some("HR Head"), management.as_deref()),
                ("HR Specialist", Some("HR Manager"), viewer.as_deref()),
            ],
        ),
        (
            "Sales",
            "#1ABC9C",
            &[
                ("Sales Head", None, management.as_deref().or(admin.as_deref())),
                ("Sales Manager", Some("Sales Head"), finance.as_deref().or(management.as_deref())),
                (
                    "Account Executive",
                    Some("Sales Manager"),
                    finance.as_deref().or(viewer.as_deref()),
                ),
            ],
        ),
    ];

    let existing = list_categories(conn, workspace_id)?;

    for (cat_name, color, positions) in defs {
        if existing
            .iter()
            .any(|c| c.name.eq_ignore_ascii_case(cat_name))
        {
            continue;
        }
        let cat = create_category(conn, workspace_id, cat_name, color)?;
        let mut name_to_id: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for (pos_name, parent_name, role_name) in *positions {
            let parent_id = parent_name.and_then(|n| name_to_id.get(n).cloned());
            let pos = create_position(
                conn,
                workspace_id,
                &cat.id,
                pos_name,
                parent_id.as_deref(),
            )?;
            if let Some(rid) = role_name {
                set_position_roles(conn, &pos.id, &[rid.to_string()])?;
            }
            name_to_id.insert(pos_name.to_string(), pos.id);
        }

        // Map members whose title matches the category onto the leaf/member position
        let leaf = positions
            .iter()
            .rev()
            .find_map(|(n, _, _)| name_to_id.get(*n).cloned());
        if let Some(leaf_id) = leaf {
            let mut stmt = conn.prepare(
                "SELECT id FROM members WHERE workspace_id = ?1 AND deleted_at IS NULL
                 AND position_id IS NULL AND lower(COALESCE(title,'')) = lower(?2)",
            )?;
            let ids: Vec<String> = stmt
                .query_map(params![workspace_id, cat_name], |row| row.get(0))?
                .filter_map(|r| r.ok())
                .collect();
            drop(stmt);
            for mid in ids {
                set_member_position(conn, &mid, Some(&leaf_id))?;
            }
        }
    }

    ensure_hr_manager(conn, workspace_id, management.as_deref())?;

    // Admins without position → Management Director if present
    if let Some(director) = list_positions(conn, workspace_id)?
        .into_iter()
        .find(|p| p.name.eq_ignore_ascii_case("Director"))
    {
        let mut stmt = conn.prepare(
            "SELECT id FROM members WHERE workspace_id = ?1 AND deleted_at IS NULL
             AND position_id IS NULL AND lower(role) = 'admin'",
        )?;
        let ids: Vec<String> = stmt
            .query_map(params![workspace_id], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        drop(stmt);
        for mid in ids {
            set_member_position(conn, &mid, Some(&director.id))?;
        }
    }

    conn.execute(
        "INSERT OR IGNORE INTO position_role_permissions (position_id, role_id, key)
         SELECT pr.position_id, pr.role_id, rp.key
         FROM position_roles pr
         INNER JOIN role_permissions rp ON rp.role_id = pr.role_id AND rp.granted = 1",
        [],
    )?;
    super::teams::ensure_category_teams(conn, workspace_id)?;

    Ok(())
}

/// Backfill HR Manager on workspaces that already had Head → Specialist.
fn ensure_hr_manager(
    conn: &Connection,
    workspace_id: &str,
    management_role_id: Option<&str>,
) -> DbResult<()> {
    let Some(hr_cat) = list_categories(conn, workspace_id)?
        .into_iter()
        .find(|c| c.name.eq_ignore_ascii_case("HR"))
    else {
        return Ok(());
    };
    let positions = list_positions(conn, workspace_id)?;
    let in_hr: Vec<_> = positions
        .iter()
        .filter(|p| p.category_id == hr_cat.id)
        .collect();
    let head = in_hr.iter().find(|p| p.name.eq_ignore_ascii_case("HR Head"));
    let manager = in_hr
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case("HR Manager"));
    let specialist = in_hr
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case("HR Specialist"));

    let manager_id = if let Some(existing) = manager {
        existing.id.clone()
    } else {
        let parent = head.map(|p| p.id.as_str());
        let created = create_position(
            conn,
            workspace_id,
            &hr_cat.id,
            "HR Manager",
            parent,
        )?;
        if let Some(rid) = management_role_id {
            set_position_roles(conn, &created.id, &[rid.to_string()])?;
        }
        created.id
    };

    if let Some(spec) = specialist {
        let should_reparent = spec.parent_id.as_deref() != Some(manager_id.as_str())
            && head
                .map(|h| spec.parent_id.as_deref() == Some(h.id.as_str()))
                .unwrap_or(spec.parent_id.is_none());
        if should_reparent {
            update_position(conn, &spec.id, &spec.name, Some(&manager_id))?;
        }
    }

    Ok(())
}

/// Ensure a default company setup admin exists (configures the workspace for everyone).
pub fn ensure_setup_admin(conn: &Connection, workspace_id: &str) -> DbResult<()> {
    use super::roles::{find_role_by_name, set_member_roles};
    use super::{get_meta, set_meta};

    let admin_role = find_role_by_name(conn, workspace_id, "Admin")?;
    let Some(admin_role) = admin_role else {
        return Ok(());
    };

    let existing_meta = get_meta(conn, "setup_admin_id")?;
    let setup_id = if let Some(id) = existing_meta {
        let ok: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM members WHERE id = ?1 AND workspace_id = ?2 AND deleted_at IS NULL",
                params![id, workspace_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if ok > 0 {
            Some(id)
        } else {
            None
        }
    } else {
        None
    };

    let member_id = if let Some(id) = setup_id {
        id
    } else if let Ok(id) = conn.query_row(
        "SELECT id FROM members WHERE workspace_id = ?1 AND deleted_at IS NULL AND name = 'Jordan Hale' LIMIT 1",
        params![workspace_id],
        |row| row.get::<_, String>(0),
    ) {
        id
    } else if let Ok(id) = conn.query_row(
        "SELECT m.id FROM members m
         INNER JOIN member_roles mr ON mr.member_id = m.id AND mr.role_id = ?2
         WHERE m.workspace_id = ?1 AND m.deleted_at IS NULL
         ORDER BY m.created_at LIMIT 1",
        params![workspace_id, admin_role.id],
        |row| row.get::<_, String>(0),
    ) {
        id
    } else if let Ok(id) = conn.query_row(
        "SELECT id FROM members WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY created_at LIMIT 1",
        params![workspace_id],
        |row| row.get::<_, String>(0),
    ) {
        id
    } else {
        return Ok(());
    };

    set_meta(conn, "setup_admin_id", &member_id)?;
    set_meta(conn, "current_member_id", &member_id)?;
    set_member_roles(conn, &member_id, &[admin_role.id.clone()])?;

    let management = list_categories(conn, workspace_id)?
        .into_iter()
        .find(|c| c.name.eq_ignore_ascii_case("Management"));
    let Some(mgmt_cat) = management else {
        return Ok(());
    };

    let mut company_admin = list_positions(conn, workspace_id)?
        .into_iter()
        .find(|p| p.category_id == mgmt_cat.id && p.name.eq_ignore_ascii_case("Company Admin"));

    if company_admin.is_none() {
        let created = create_position(conn, workspace_id, &mgmt_cat.id, "Company Admin", None)?;
        let ts = now_iso();
        conn.execute(
            "UPDATE positions SET parent_id = NULL, sort_order = -1, updated_at = ?1, version = version + 1 WHERE id = ?2",
            params![ts, created.id],
        )?;
        company_admin = get_position(conn, &created.id)?;
    }

    if let Some(pos) = company_admin {
        set_position_roles(conn, &pos.id, &[admin_role.id])?;
        set_member_position(conn, &member_id, Some(&pos.id))?;
        let ts = now_iso();
        conn.execute(
            "UPDATE members SET title = 'Company Admin', updated_at = ?1, version = version + 1 WHERE id = ?2",
            params![ts, member_id],
        )?;
    }

    Ok(())
}
