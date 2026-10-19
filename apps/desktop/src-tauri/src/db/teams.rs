use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::{new_id, now_iso, DbError, DbResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub department: Option<String>,
    pub color: String,
    #[serde(default)]
    pub kind: String,
    pub created_by: Option<String>,
    #[serde(default)]
    pub category_id: Option<String>,
    pub member_ids: Vec<String>,
    #[serde(default)]
    pub channels: Vec<TeamChannel>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamChannel {
    pub id: String,
    pub workspace_id: String,
    pub team_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMessage {
    pub id: String,
    pub workspace_id: String,
    pub team_id: String,
    pub channel_id: Option<String>,
    pub author_id: String,
    pub author_name: String,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmThread {
    pub id: String,
    pub workspace_id: String,
    pub member_a_id: String,
    pub member_b_id: String,
    pub other_member_id: String,
    pub other_member_name: String,
    pub last_body: Option<String>,
    pub updated_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmMessage {
    pub id: String,
    pub workspace_id: String,
    pub thread_id: String,
    pub author_id: String,
    pub author_name: String,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

fn team_member_ids(conn: &Connection, team_id: &str) -> DbResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT member_id FROM team_members WHERE team_id = ?1 ORDER BY created_at",
    )?;
    let rows = stmt.query_map(params![team_id], |row| row.get::<_, String>(0))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

const TEAM_COLS: &str = "id, workspace_id, name, description, department, color,
         COALESCE(kind, 'department'), created_by, category_id, created_at, updated_at, deleted_at, version";

fn map_team_row(
    conn: &Connection,
    id: String,
    ws: String,
    name: String,
    description: Option<String>,
    department: Option<String>,
    color: String,
    kind: String,
    created_by: Option<String>,
    category_id: Option<String>,
    created_at: String,
    updated_at: String,
    deleted_at: Option<String>,
    version: i64,
) -> DbResult<Team> {
    Ok(Team {
        id: id.clone(),
        workspace_id: ws,
        name,
        description,
        department,
        color,
        kind,
        created_by,
        category_id,
        member_ids: team_member_ids(conn, &id)?,
        channels: list_team_channels(conn, &id)?,
        created_at,
        updated_at,
        deleted_at,
        version,
    })
}

pub fn list_teams(conn: &Connection, workspace_id: &str) -> DbResult<Vec<Team>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {TEAM_COLS} FROM teams WHERE workspace_id = ?1 AND deleted_at IS NULL
         ORDER BY name COLLATE NOCASE"
    ))?;
    let rows = stmt.query_map(params![workspace_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, Option<String>>(7)?,
            row.get::<_, Option<String>>(8)?,
            row.get::<_, String>(9)?,
            row.get::<_, String>(10)?,
            row.get::<_, Option<String>>(11)?,
            row.get::<_, i64>(12)?,
        ))
    })?;
    let mut teams = Vec::new();
    for row in rows {
        let (id, ws, name, description, department, color, kind, created_by, category_id, created_at, updated_at, deleted_at, version) =
            row?;
        teams.push(map_team_row(
            conn, id, ws, name, description, department, color, kind, created_by, category_id, created_at,
            updated_at, deleted_at, version,
        )?);
    }
    Ok(teams)
}

pub fn get_team(conn: &Connection, id: &str) -> DbResult<Option<Team>> {
    let row = conn
        .query_row(
            &format!("SELECT {TEAM_COLS} FROM teams WHERE id = ?1 AND deleted_at IS NULL"),
            params![id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, Option<String>>(11)?,
                    row.get::<_, i64>(12)?,
                ))
            },
        )
        .optional()?;
    let Some((id, ws, name, description, department, color, kind, created_by, category_id, created_at, updated_at, deleted_at, version)) =
        row
    else {
        return Ok(None);
    };
    Ok(Some(map_team_row(
        conn, id, ws, name, description, department, color, kind, created_by, category_id, created_at,
        updated_at, deleted_at, version,
    )?))
}

pub fn create_team(
    conn: &Connection,
    workspace_id: &str,
    name: &str,
    description: Option<&str>,
    department: Option<&str>,
    color: &str,
) -> DbResult<Team> {
    create_team_ex(
        conn,
        workspace_id,
        name,
        description,
        department,
        color,
        "department",
        None,
        None,
    )
}

pub fn create_team_ex(
    conn: &Connection,
    workspace_id: &str,
    name: &str,
    description: Option<&str>,
    department: Option<&str>,
    color: &str,
    kind: &str,
    created_by: Option<&str>,
    category_id: Option<&str>,
) -> DbResult<Team> {
    let id = new_id();
    let ts = now_iso();
    conn.execute(
        "INSERT INTO teams (id, workspace_id, name, description, department, color, kind, created_by, category_id, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10, NULL, 1)",
        params![id, workspace_id, name, description, department, color, kind, created_by, category_id, ts],
    )?;
    ensure_general_channel(conn, workspace_id, &id, created_by)?;
    get_team(conn, &id)?.ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}

pub fn list_team_channels(conn: &Connection, team_id: &str) -> DbResult<Vec<TeamChannel>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, team_id, name, description, created_by, created_at, updated_at, deleted_at, version
         FROM team_channels WHERE team_id = ?1 AND deleted_at IS NULL
         ORDER BY CASE WHEN lower(name) = 'general' THEN 0 ELSE 1 END, name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map(params![team_id], |row| {
        Ok(TeamChannel {
            id: row.get(0)?,
            workspace_id: row.get(1)?,
            team_id: row.get(2)?,
            name: row.get(3)?,
            description: row.get(4)?,
            created_by: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
            deleted_at: row.get(8)?,
            version: row.get(9)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn create_team_channel(
    conn: &Connection,
    workspace_id: &str,
    team_id: &str,
    name: &str,
    description: Option<&str>,
    created_by: Option<&str>,
) -> DbResult<TeamChannel> {
    let id = new_id();
    let ts = now_iso();
    let slug = name.trim().trim_start_matches('#').to_string();
    if slug.is_empty() {
        return Err(DbError::Sqlite(rusqlite::Error::InvalidParameterName(
            "chat name is required".into(),
        )));
    }
    conn.execute(
        "INSERT INTO team_channels (id, workspace_id, team_id, name, description, created_by, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, NULL, 1)",
        params![id, workspace_id, team_id, slug, description, created_by, ts],
    )?;
    list_team_channels(conn, team_id)?
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}

pub fn ensure_general_channel(
    conn: &Connection,
    workspace_id: &str,
    team_id: &str,
    created_by: Option<&str>,
) -> DbResult<TeamChannel> {
    let existing = list_team_channels(conn, team_id)?;
    if let Some(general) = existing
        .iter()
        .find(|c| c.name.eq_ignore_ascii_case("general"))
    {
        conn.execute(
            "UPDATE team_messages SET channel_id = ?1 WHERE team_id = ?2 AND channel_id IS NULL",
            params![general.id, team_id],
        )?;
        return Ok(general.clone());
    }
    let created = create_team_channel(
        conn,
        workspace_id,
        team_id,
        "general",
        Some("Team chat"),
        created_by,
    )?;
    conn.execute(
        "UPDATE team_messages SET channel_id = ?1 WHERE team_id = ?2 AND channel_id IS NULL",
        params![created.id, team_id],
    )?;
    Ok(created)
}

pub fn ensure_all_team_channels(conn: &Connection, workspace_id: &str) -> DbResult<()> {
    let teams = list_teams(conn, workspace_id)?;
    for team in teams {
        ensure_general_channel(conn, workspace_id, &team.id, team.created_by.as_deref())?;
    }
    Ok(())
}

pub fn is_lead_title(value: &str) -> bool {
    let l = value.to_lowercase();
    ["manager", "supervisor", "head", "director", "lead", "admin"]
        .iter()
        .any(|k| l.contains(k))
}

pub fn member_can_create_chats(conn: &Connection, member_id: &str) -> DbResult<bool> {
    if super::roles::member_has_permission(conn, member_id, "teams.manage")?
        || super::roles::member_has_permission(conn, member_id, "settings.workspace_admin")?
    {
        return Ok(true);
    }
    let row: Option<(Option<String>, Option<String>)> = conn
        .query_row(
            "SELECT m.title, p.name FROM members m
             LEFT JOIN positions p ON p.id = m.position_id AND p.deleted_at IS NULL
             WHERE m.id = ?1 AND m.deleted_at IS NULL",
            params![member_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let Some((title, position)) = row else {
        return Ok(false);
    };
    Ok(title.as_deref().map(is_lead_title).unwrap_or(false)
        || position.as_deref().map(is_lead_title).unwrap_or(false))
}

pub fn set_team_members(conn: &Connection, team_id: &str, member_ids: &[String]) -> DbResult<()> {
    conn.execute("DELETE FROM team_members WHERE team_id = ?1", params![team_id])?;
    let ts = now_iso();
    for mid in member_ids {
        conn.execute(
            "INSERT INTO team_members (team_id, member_id, created_at) VALUES (?1, ?2, ?3)",
            params![team_id, mid, ts],
        )?;
    }
    let ts2 = now_iso();
    conn.execute(
        "UPDATE teams SET updated_at = ?1, version = version + 1 WHERE id = ?2",
        params![ts2, team_id],
    )?;
    Ok(())
}

pub fn list_team_messages(
    conn: &Connection,
    team_id: &str,
    channel_id: Option<&str>,
) -> DbResult<Vec<TeamMessage>> {
    let sql = if channel_id.is_some() {
        "SELECT tm.id, tm.workspace_id, tm.team_id, tm.channel_id, tm.author_id, COALESCE(m.name, 'Unknown'), tm.body,
                tm.created_at, tm.updated_at, tm.deleted_at, tm.version
         FROM team_messages tm
         LEFT JOIN members m ON m.id = tm.author_id
         WHERE tm.team_id = ?1 AND tm.deleted_at IS NULL AND tm.channel_id = ?2
         ORDER BY tm.created_at ASC
         LIMIT 200"
    } else {
        "SELECT tm.id, tm.workspace_id, tm.team_id, tm.channel_id, tm.author_id, COALESCE(m.name, 'Unknown'), tm.body,
                tm.created_at, tm.updated_at, tm.deleted_at, tm.version
         FROM team_messages tm
         LEFT JOIN members m ON m.id = tm.author_id
         WHERE tm.team_id = ?1 AND tm.deleted_at IS NULL
         ORDER BY tm.created_at ASC
         LIMIT 200"
    };
    let mut stmt = conn.prepare(sql)?;
    let map_row = |row: &rusqlite::Row<'_>| {
        Ok(TeamMessage {
            id: row.get(0)?,
            workspace_id: row.get(1)?,
            team_id: row.get(2)?,
            channel_id: row.get(3)?,
            author_id: row.get(4)?,
            author_name: row.get(5)?,
            body: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            deleted_at: row.get(9)?,
            version: row.get(10)?,
        })
    };
    let rows = if let Some(cid) = channel_id {
        stmt.query_map(params![team_id, cid], map_row)?
    } else {
        stmt.query_map(params![team_id], map_row)?
    };
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn send_team_message(
    conn: &Connection,
    workspace_id: &str,
    team_id: &str,
    channel_id: Option<&str>,
    author_id: &str,
    body: &str,
) -> DbResult<TeamMessage> {
    let channel = match channel_id {
        Some(id) => id.to_string(),
        None => ensure_general_channel(conn, workspace_id, team_id, Some(author_id))?.id,
    };
    let id = new_id();
    let ts = now_iso();
    conn.execute(
        "INSERT INTO team_messages (id, workspace_id, team_id, channel_id, author_id, body, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, NULL, 1)",
        params![id, workspace_id, team_id, channel, author_id, body, ts],
    )?;
    list_team_messages(conn, team_id, Some(&channel))?
        .into_iter()
        .find(|m| m.id == id)
        .ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}

fn ordered_pair(a: &str, b: &str) -> (String, String) {
    if a <= b {
        (a.to_string(), b.to_string())
    } else {
        (b.to_string(), a.to_string())
    }
}

pub fn get_or_create_dm(
    conn: &Connection,
    workspace_id: &str,
    me: &str,
    other: &str,
) -> DbResult<DmThread> {
    if me == other {
        return Err(DbError::Sqlite(rusqlite::Error::InvalidParameterName(
            "cannot DM yourself".into(),
        )));
    }
    let (a, b) = ordered_pair(me, other);
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM dm_threads WHERE workspace_id = ?1 AND member_a_id = ?2 AND member_b_id = ?3",
            params![workspace_id, a, b],
            |row| row.get(0),
        )
        .optional()?;
    let id = if let Some(id) = existing {
        id
    } else {
        let id = new_id();
        let ts = now_iso();
        conn.execute(
            "INSERT INTO dm_threads (id, workspace_id, member_a_id, member_b_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            params![id, workspace_id, a, b, ts],
        )?;
        id
    };
    list_dm_threads(conn, workspace_id, me)?
        .into_iter()
        .find(|t| t.id == id)
        .ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}

pub fn list_dm_threads(conn: &Connection, workspace_id: &str, me: &str) -> DbResult<Vec<DmThread>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.workspace_id, t.member_a_id, t.member_b_id, t.created_at, t.updated_at
         FROM dm_threads t
         WHERE t.workspace_id = ?1 AND (t.member_a_id = ?2 OR t.member_b_id = ?2)
         ORDER BY t.updated_at DESC",
    )?;
    let rows = stmt.query_map(params![workspace_id, me], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (id, ws, a, b, created_at, updated_at) = row?;
        let other = if a == me { b.clone() } else { a.clone() };
        let other_name: String = conn
            .query_row(
                "SELECT name FROM members WHERE id = ?1",
                params![other],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "Unknown".into());
        let last_body: Option<String> = conn
            .query_row(
                "SELECT body FROM dm_messages WHERE thread_id = ?1 AND deleted_at IS NULL
                 ORDER BY created_at DESC LIMIT 1",
                params![id],
                |r| r.get(0),
            )
            .optional()?;
        out.push(DmThread {
            id,
            workspace_id: ws,
            member_a_id: a,
            member_b_id: b,
            other_member_id: other,
            other_member_name: other_name,
            last_body,
            updated_at,
            created_at,
        });
    }
    Ok(out)
}

pub fn list_dm_messages(conn: &Connection, thread_id: &str) -> DbResult<Vec<DmMessage>> {
    let mut stmt = conn.prepare(
        "SELECT d.id, d.workspace_id, d.thread_id, d.author_id, COALESCE(m.name, 'Unknown'), d.body,
                d.created_at, d.updated_at, d.deleted_at, d.version
         FROM dm_messages d
         LEFT JOIN members m ON m.id = d.author_id
         WHERE d.thread_id = ?1 AND d.deleted_at IS NULL
         ORDER BY d.created_at ASC
         LIMIT 200",
    )?;
    let rows = stmt.query_map(params![thread_id], |row| {
        Ok(DmMessage {
            id: row.get(0)?,
            workspace_id: row.get(1)?,
            thread_id: row.get(2)?,
            author_id: row.get(3)?,
            author_name: row.get(4)?,
            body: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
            deleted_at: row.get(8)?,
            version: row.get(9)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn send_dm_message(
    conn: &Connection,
    workspace_id: &str,
    thread_id: &str,
    author_id: &str,
    body: &str,
) -> DbResult<DmMessage> {
    let id = new_id();
    let ts = now_iso();
    conn.execute(
        "INSERT INTO dm_messages (id, workspace_id, thread_id, author_id, body, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, NULL, 1)",
        params![id, workspace_id, thread_id, author_id, body, ts],
    )?;
    conn.execute(
        "UPDATE dm_threads SET updated_at = ?1 WHERE id = ?2",
        params![ts, thread_id],
    )?;
    list_dm_messages(conn, thread_id)?
        .into_iter()
        .find(|m| m.id == id)
        .ok_or_else(|| DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}

/// One department team per position category; membership follows assigned positions.
pub fn ensure_category_teams(conn: &Connection, workspace_id: &str) -> DbResult<()> {
    let mut cat_stmt = conn.prepare(
        "SELECT id, name, color FROM org_categories
         WHERE workspace_id = ?1 AND deleted_at IS NULL
         ORDER BY sort_order ASC, name COLLATE NOCASE",
    )?;
    let categories: Vec<(String, String, String)> = cat_stmt
        .query_map(params![workspace_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .filter_map(|r| r.ok())
        .collect();
    drop(cat_stmt);

    let existing = list_teams(conn, workspace_id)?;
    let mut claimed: Vec<String> = Vec::new();

    for (cat_id, cat_name, cat_color) in categories {
        let found = existing.iter().find(|t| {
            t.kind != "chat"
                && !claimed.iter().any(|id| id == &t.id)
                && (t.category_id.as_deref() == Some(cat_id.as_str())
                    || t.department.as_deref().is_some_and(|d| d.eq_ignore_ascii_case(&cat_name))
                    || t.name.eq_ignore_ascii_case(&cat_name))
        });

        let team_id = if let Some(team) = found {
            let ts = now_iso();
            conn.execute(
                "UPDATE teams SET name = ?1, department = ?1, color = ?2, category_id = ?3, kind = 'department',
                 updated_at = ?4, version = version + 1 WHERE id = ?5",
                params![cat_name, cat_color, cat_id, ts, team.id],
            )?;
            team.id.clone()
        } else {
            let created = create_team_ex(
                conn,
                workspace_id,
                &cat_name,
                Some("Chats for this position category"),
                Some(&cat_name),
                &cat_color,
                "department",
                None,
                Some(&cat_id),
            )?;
            created.id
        };
        claimed.push(team_id.clone());

        let mut mem_stmt = conn.prepare(
            "SELECT DISTINCT m.id FROM members m
             INNER JOIN positions p ON p.id = m.position_id AND p.deleted_at IS NULL
             WHERE m.workspace_id = ?1 AND m.deleted_at IS NULL
             AND (
               p.category_id = ?2
               OR EXISTS (
                 SELECT 1 FROM position_role_permissions prp
                 INNER JOIN position_roles pr
                    ON pr.position_id = prp.position_id AND pr.role_id = prp.role_id
                 INNER JOIN role_permissions rp
                    ON rp.role_id = prp.role_id AND rp.key = prp.key AND rp.granted = 1
                 INNER JOIN roles r ON r.id = prp.role_id AND r.deleted_at IS NULL
                 WHERE prp.position_id = p.id AND prp.key = 'teams.see_all_chats'
               )
             )",
        )?;
        let member_ids: Vec<String> = mem_stmt
            .query_map(params![workspace_id, cat_id], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        drop(mem_stmt);
        set_team_members(conn, &team_id, &member_ids)?;
        ensure_general_channel(
            conn,
            workspace_id,
            &team_id,
            member_ids.first().map(String::as_str),
        )?;
    }

    ensure_all_team_channels(conn, workspace_id)?;
    Ok(())
}

pub fn ensure_default_teams(conn: &Connection, workspace_id: &str) -> DbResult<()> {
    ensure_category_teams(conn, workspace_id)
}
