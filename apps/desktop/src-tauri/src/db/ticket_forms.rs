use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::{new_id, now_iso, DbResult};

const SYSTEM_FIELDS: &[(&str, &str, bool)] = &[
    ("title", "Subject", true),
    ("company", "Company", true),
    ("contact", "Contact", false),
    ("priority", "Priority", false),
    ("status", "Status", false),
    ("assignee", "Assignee", false),
    ("tags", "Tags", false),
    ("description", "Description", false),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketField {
    pub id: String,
    pub key: String,
    pub label: String,
    pub kind: String,
    pub options: Vec<String>,
    pub hint: Option<String>,
    pub active: bool,
    pub system: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketFormField {
    pub field_id: String,
    pub position: i64,
    pub required: bool,
    pub field: TicketField,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketForm {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub active: bool,
    pub is_default: bool,
    pub position: i64,
    pub fields: Vec<TicketFormField>,
}

#[derive(Debug, Deserialize)]
pub struct SaveFormFieldInput {
    pub field_id: String,
    pub required: bool,
    pub label: String,
    pub hint: Option<String>,
    pub options: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SaveTicketFormInput {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub active: bool,
    pub is_default: bool,
    pub fields: Vec<SaveFormFieldInput>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTicketFieldInput {
    pub label: String,
    pub kind: String,
    pub options: Vec<String>,
    pub hint: Option<String>,
    pub form_id: Option<String>,
}

pub fn ensure_ticket_forms(conn: &Connection, workspace_id: &str) -> DbResult<()> {
    let ts = now_iso();
    for (key, label, _) in SYSTEM_FIELDS {
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM ticket_fields WHERE workspace_id = ?1 AND key = ?2 AND deleted_at IS NULL",
                params![workspace_id, key],
                |_| Ok(true),
            )
            .optional()?
            .unwrap_or(false);
        if exists {
            continue;
        }
        conn.execute(
            "INSERT INTO ticket_fields (id, workspace_id, key, label, kind, options_json, hint, active, created_at, updated_at, deleted_at)
             VALUES (?1, ?2, ?3, ?4, 'system', '[]', NULL, 1, ?5, ?5, NULL)",
            params![new_id(), workspace_id, key, label, ts],
        )?;
    }

    let form_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ticket_forms WHERE workspace_id = ?1 AND deleted_at IS NULL",
        params![workspace_id],
        |row| row.get(0),
    )?;
    if form_count > 0 {
        return Ok(());
    }

    let form_id = new_id();
    conn.execute(
        "INSERT INTO ticket_forms (id, workspace_id, name, description, active, is_default, position, created_at, updated_at, deleted_at)
         VALUES (?1, ?2, 'General', 'Default ticket format', 1, 1, 0, ?3, ?3, NULL)",
        params![form_id, workspace_id, ts],
    )?;
    let fields = list_fields(conn, workspace_id)?;
    for (index, (key, _, required)) in SYSTEM_FIELDS.iter().enumerate() {
        let Some(field) = fields.iter().find(|field| field.key == *key) else {
            continue;
        };
        conn.execute(
            "INSERT INTO ticket_form_fields (form_id, field_id, position, required) VALUES (?1, ?2, ?3, ?4)",
            params![form_id, field.id, index as i64, if *required { 1 } else { 0 }],
        )?;
    }
    Ok(())
}

pub fn list_ticket_forms(conn: &Connection, workspace_id: &str) -> DbResult<Vec<TicketForm>> {
    ensure_ticket_forms(conn, workspace_id)?;
    let mut stmt = conn.prepare(
        "SELECT id, name, description, active, is_default, position
         FROM ticket_forms WHERE workspace_id = ?1 AND deleted_at IS NULL
         ORDER BY position, name COLLATE NOCASE",
    )?;
    let forms = stmt.query_map(params![workspace_id], |row| {
        Ok(TicketForm {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            active: row.get::<_, i64>(3)? != 0,
            is_default: row.get::<_, i64>(4)? != 0,
            position: row.get(5)?,
            fields: Vec::new(),
        })
    })?;
    let mut forms: Vec<TicketForm> = forms.collect::<Result<Vec<_>, _>>()?;
    for form in &mut forms {
        form.fields = form_fields(conn, &form.id)?;
    }
    Ok(forms)
}

pub fn list_ticket_fields(conn: &Connection, workspace_id: &str) -> DbResult<Vec<TicketField>> {
    ensure_ticket_forms(conn, workspace_id)?;
    list_fields(conn, workspace_id)
}

fn list_fields(conn: &Connection, workspace_id: &str) -> DbResult<Vec<TicketField>> {
    let mut stmt = conn.prepare(
        "SELECT id, key, label, kind, options_json, hint, active
         FROM ticket_fields WHERE workspace_id = ?1 AND deleted_at IS NULL
         ORDER BY label COLLATE NOCASE",
    )?;
    let rows = stmt.query_map(params![workspace_id], map_field)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn form_fields(conn: &Connection, form_id: &str) -> DbResult<Vec<TicketFormField>> {
    let mut stmt = conn.prepare(
        "SELECT f.id, f.key, f.label, f.kind, f.options_json, f.hint, f.active, ff.position, ff.required
         FROM ticket_form_fields ff
         JOIN ticket_fields f ON f.id = ff.field_id
         WHERE ff.form_id = ?1 AND f.deleted_at IS NULL
         ORDER BY ff.position, f.label COLLATE NOCASE",
    )?;
    let rows = stmt.query_map(params![form_id], |row| {
        let field = map_field_at(row, 0)?;
        Ok(TicketFormField {
            field_id: field.id.clone(),
            position: row.get(7)?,
            required: row.get::<_, i64>(8)? != 0,
            field,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn map_field(row: &rusqlite::Row<'_>) -> rusqlite::Result<TicketField> {
    map_field_at(row, 0)
}

fn map_field_at(row: &rusqlite::Row<'_>, offset: usize) -> rusqlite::Result<TicketField> {
    let options_json: Option<String> = row.get(offset + 4)?;
    let options = options_json
        .as_deref()
        .and_then(|raw| serde_json::from_str::<Vec<String>>(raw).ok())
        .unwrap_or_default();
    let kind: String = row.get(offset + 3)?;
    Ok(TicketField {
        id: row.get(offset)?,
        key: row.get(offset + 1)?,
        label: row.get(offset + 2)?,
        system: kind == "system",
        kind,
        options,
        hint: row.get(offset + 5)?,
        active: row.get::<_, i64>(offset + 6)? != 0,
    })
}

pub fn save_ticket_form(
    conn: &Connection,
    workspace_id: &str,
    input: &SaveTicketFormInput,
) -> DbResult<TicketForm> {
    ensure_ticket_forms(conn, workspace_id)?;
    let name = input.name.trim();
    if name.is_empty() {
        return Err(rusqlite::Error::InvalidParameterName("Form name is required".into()).into());
    }
    if !input.fields.iter().any(|field| field_key(conn, &field.field_id).as_deref() == Some("title"))
        || !input
            .fields
            .iter()
            .any(|field| field_key(conn, &field.field_id).as_deref() == Some("company"))
    {
        return Err(
            rusqlite::Error::InvalidParameterName("Subject and Company stay on every form".into())
                .into(),
        );
    }

    let ts = now_iso();
    let form_id = if let Some(id) = input.id.as_deref().filter(|id| !id.is_empty()) {
        conn.execute(
            "UPDATE ticket_forms SET name = ?1, description = ?2, active = ?3, updated_at = ?4
             WHERE id = ?5 AND workspace_id = ?6 AND deleted_at IS NULL",
            params![
                name,
                blank_to_none(input.description.as_deref()),
                if input.active { 1 } else { 0 },
                ts,
                id,
                workspace_id
            ],
        )?;
        id.to_string()
    } else {
        let id = new_id();
        let position: i64 = conn.query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM ticket_forms WHERE workspace_id = ?1 AND deleted_at IS NULL",
            params![workspace_id],
            |row| row.get(0),
        )?;
        conn.execute(
            "INSERT INTO ticket_forms (id, workspace_id, name, description, active, is_default, position, created_at, updated_at, deleted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7, ?7, NULL)",
            params![
                id,
                workspace_id,
                name,
                blank_to_none(input.description.as_deref()),
                if input.active { 1 } else { 0 },
                position,
                ts
            ],
        )?;
        id
    };

    let make_default = input.is_default || !has_other_default(conn, workspace_id, &form_id)?;
    if make_default {
        conn.execute(
            "UPDATE ticket_forms SET is_default = 0 WHERE workspace_id = ?1 AND deleted_at IS NULL",
            params![workspace_id],
        )?;
        conn.execute(
            "UPDATE ticket_forms SET is_default = 1, active = 1 WHERE id = ?1",
            params![form_id],
        )?;
    } else {
        conn.execute(
            "UPDATE ticket_forms SET is_default = 0 WHERE id = ?1",
            params![form_id],
        )?;
    }

    conn.execute(
        "DELETE FROM ticket_form_fields WHERE form_id = ?1",
        params![form_id],
    )?;
    for (index, field) in input.fields.iter().enumerate() {
        let key = field_key(conn, &field.field_id);
        let required = key.as_deref() == Some("title") || key.as_deref() == Some("company") || field.required;
        let label = field.label.trim();
        if !label.is_empty() {
            let options = serde_json::to_string(&field.options).unwrap_or_else(|_| "[]".into());
            conn.execute(
                "UPDATE ticket_fields SET label = ?1, hint = ?2, options_json = ?3, updated_at = ?4 WHERE id = ?5",
                params![
                    label,
                    blank_to_none(field.hint.as_deref()),
                    options,
                    ts,
                    field.field_id
                ],
            )?;
        }
        conn.execute(
            "INSERT INTO ticket_form_fields (form_id, field_id, position, required) VALUES (?1, ?2, ?3, ?4)",
            params![form_id, field.field_id, index as i64, if required { 1 } else { 0 }],
        )?;
    }

    list_ticket_forms(conn, workspace_id)?
        .into_iter()
        .find(|form| form.id == form_id)
        .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows.into())
}

pub fn create_ticket_field(
    conn: &Connection,
    workspace_id: &str,
    input: &CreateTicketFieldInput,
) -> DbResult<TicketField> {
    ensure_ticket_forms(conn, workspace_id)?;
    let label = input.label.trim();
    if label.is_empty() {
        return Err(rusqlite::Error::InvalidParameterName("Field name is required".into()).into());
    }
    let kind = input.kind.trim();
    if !matches!(kind, "text" | "textarea" | "dropdown" | "checkbox" | "number") {
        return Err(rusqlite::Error::InvalidParameterName("Choose a field type".into()).into());
    }
    let options: Vec<String> = input
        .options
        .iter()
        .map(|option| option.trim().to_string())
        .filter(|option| !option.is_empty())
        .collect();
    if kind == "dropdown" && options.is_empty() {
        return Err(
            rusqlite::Error::InvalidParameterName("Add at least one dropdown option".into()).into(),
        );
    }
    let id = new_id();
    let ts = now_iso();
    let options_json = serde_json::to_string(&options).unwrap_or_else(|_| "[]".into());
    conn.execute(
        "INSERT INTO ticket_fields (id, workspace_id, key, label, kind, options_json, hint, active, created_at, updated_at, deleted_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?8, NULL)",
        params![
            id,
            workspace_id,
            format!("custom_{id}"),
            label,
            kind,
            options_json,
            blank_to_none(input.hint.as_deref()),
            ts
        ],
    )?;
    if let Some(form_id) = input.form_id.as_deref().filter(|id| !id.is_empty()) {
        let position: i64 = conn.query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM ticket_form_fields WHERE form_id = ?1",
            params![form_id],
            |row| row.get(0),
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO ticket_form_fields (form_id, field_id, position, required) VALUES (?1, ?2, ?3, 0)",
            params![form_id, id, position],
        )?;
    }
    list_fields(conn, workspace_id)?
        .into_iter()
        .find(|field| field.id == id)
        .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows.into())
}

pub fn delete_ticket_field(conn: &Connection, workspace_id: &str, field_id: &str) -> DbResult<()> {
    let kind: Option<String> = conn
        .query_row(
            "SELECT kind FROM ticket_fields WHERE id = ?1 AND workspace_id = ?2 AND deleted_at IS NULL",
            params![field_id, workspace_id],
            |row| row.get(0),
        )
        .optional()?;
    if kind.as_deref() == Some("system") || kind.is_none() {
        return Err(
            rusqlite::Error::InvalidParameterName("Built-in fields stay in the catalog".into())
                .into(),
        );
    }
    let ts = now_iso();
    conn.execute(
        "UPDATE ticket_fields SET deleted_at = ?1, active = 0, updated_at = ?1 WHERE id = ?2",
        params![ts, field_id],
    )?;
    conn.execute(
        "DELETE FROM ticket_form_fields WHERE field_id = ?1",
        params![field_id],
    )?;
    Ok(())
}

pub fn delete_ticket_form(conn: &Connection, workspace_id: &str, form_id: &str) -> DbResult<()> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ticket_forms WHERE workspace_id = ?1 AND deleted_at IS NULL",
        params![workspace_id],
        |row| row.get(0),
    )?;
    if count <= 1 {
        return Err(rusqlite::Error::InvalidParameterName("Keep at least one ticket form".into()).into());
    }
    let was_default: i64 = conn.query_row(
        "SELECT is_default FROM ticket_forms WHERE id = ?1 AND workspace_id = ?2",
        params![form_id, workspace_id],
        |row| row.get(0),
    )?;
    let ts = now_iso();
    conn.execute(
        "UPDATE ticket_forms SET deleted_at = ?1, is_default = 0, updated_at = ?1 WHERE id = ?2 AND workspace_id = ?3",
        params![ts, form_id, workspace_id],
    )?;
    conn.execute(
        "DELETE FROM ticket_form_fields WHERE form_id = ?1",
        params![form_id],
    )?;
    if was_default != 0 {
        conn.execute(
            "UPDATE ticket_forms SET is_default = 1, active = 1
             WHERE id = (
               SELECT id FROM ticket_forms WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY position LIMIT 1
             )",
            params![workspace_id],
        )?;
    }
    Ok(())
}

pub fn validate_ticket_submission(
    conn: &Connection,
    workspace_id: &str,
    form_id: Option<&str>,
    title: &str,
    company_id: &str,
    contact_id: Option<&str>,
    description: Option<&str>,
    tags: Option<&str>,
    assignee_id: Option<&str>,
    custom: &std::collections::BTreeMap<String, String>,
) -> Result<(), String> {
    let forms = list_ticket_forms(conn, workspace_id).map_err(|e| e.to_string())?;
    let form = form_id
        .and_then(|id| forms.iter().find(|form| form.id == id))
        .or_else(|| forms.iter().find(|form| form.is_default))
        .or_else(|| forms.first());
    let Some(form) = form else {
        return Ok(());
    };
    for placement in &form.fields {
        if !placement.required {
            continue;
        }
        let missing = match placement.field.key.as_str() {
            "title" => title.trim().is_empty(),
            "company" => company_id.trim().is_empty(),
            "contact" => contact_id.map(|v| v.trim().is_empty()).unwrap_or(true),
            "description" => description.map(|v| v.trim().is_empty()).unwrap_or(true),
            "tags" => tags.map(|v| v.trim().is_empty()).unwrap_or(true),
            "assignee" => assignee_id.map(|v| v.trim().is_empty()).unwrap_or(true),
            "status" | "priority" => false,
            _ => {
                let value = custom
                    .get(&placement.field.id)
                    .map(|item| item.trim())
                    .unwrap_or("");
                if placement.field.kind == "checkbox" {
                    value != "true"
                } else {
                    value.is_empty()
                }
            }
        };
        if missing {
            return Err(format!("{} is required.", placement.field.label));
        }
    }
    Ok(())
}

fn field_key(conn: &Connection, field_id: &str) -> Option<String> {
    conn.query_row(
        "SELECT key FROM ticket_fields WHERE id = ?1",
        params![field_id],
        |row| row.get(0),
    )
    .optional()
    .ok()
    .flatten()
}

fn has_other_default(conn: &Connection, workspace_id: &str, form_id: &str) -> DbResult<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ticket_forms
         WHERE workspace_id = ?1 AND deleted_at IS NULL AND is_default = 1 AND id != ?2",
        params![workspace_id, form_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

fn blank_to_none(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}
