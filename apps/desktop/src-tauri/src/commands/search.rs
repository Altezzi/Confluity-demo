use std::sync::Arc;

use crate::db::{models::SearchResult, DbError};
use crate::state::AppState;

fn map_err(e: DbError) -> String {
    e.to_string()
}

#[tauri::command]
pub fn search_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    query: String,
) -> Result<Vec<SearchResult>, String> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(vec![]);
    }

    let conn = state.conn.lock();
    // Escape FTS5 special chars lightly by quoting tokens.
    let fts_query = q
        .split_whitespace()
        .map(|t| {
            let cleaned = t.replace('"', "");
            format!("\"{cleaned}\"*")
        })
        .collect::<Vec<_>>()
        .join(" ");

    let mut stmt = conn
        .prepare(
            "SELECT entity_type, entity_id, title,
                    snippet(search_fts, 3, '', '', '…', 12) as snippet,
                    bm25(search_fts) as rank
             FROM search_fts
             WHERE search_fts MATCH ?1
             ORDER BY rank
             LIMIT 50",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![fts_query], |row| {
            Ok(SearchResult {
                entity_type: row.get(0)?,
                entity_id: row.get(1)?,
                title: row.get(2)?,
                snippet: row.get(3)?,
                rank: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| e.to_string())?);
    }
    let _ = map_err;
    Ok(results)
}
