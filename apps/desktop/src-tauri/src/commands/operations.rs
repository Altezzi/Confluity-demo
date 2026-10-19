use serde::Deserialize;
use std::sync::Arc;

use crate::db::{
    extended::{
        get_asset, get_product, get_rma, list_assets, list_products, list_rmas, Asset, Product, Rma,
    },
    new_id, now_iso, notify_workspace, record_change, upsert_fts, DbError,
};
use crate::state::AppState;

fn map_err(e: DbError) -> String {
    e.to_string()
}

fn workspace_id(state: &AppState) -> Result<String, String> {
    let conn = state.conn.lock();
    crate::db::get_meta(&conn, "workspace_id")
        .map_err(map_err)?
        .ok_or_else(|| "workspace not initialized".into())
}

#[derive(Debug, Deserialize)]
pub struct CreateProductInput {
    pub name: String,
    pub sku: Option<String>,
    pub description: Option<String>,
    pub inventory_qty: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductInput {
    pub id: String,
    pub name: String,
    pub sku: Option<String>,
    pub description: Option<String>,
    pub inventory_qty: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateAssetInput {
    pub name: String,
    pub product_id: Option<String>,
    pub company_id: Option<String>,
    pub ticket_id: Option<String>,
    pub serial: Option<String>,
    pub warranty_expires_at: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRmaInput {
    pub company_id: Option<String>,
    pub asset_id: Option<String>,
    pub ticket_id: Option<String>,
    pub reason: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRmaInput {
    pub id: String,
    pub status: String,
    pub reason: Option<String>,
    pub resolution: Option<String>,
}

#[tauri::command]
pub fn list_products_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<Vec<Product>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_products(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn get_product_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<Option<Product>, String> {
    let conn = state.conn.lock();
    get_product(&conn, &id).map_err(map_err)
}

#[tauri::command]
pub fn create_product_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateProductInput,
) -> Result<Product, String> {
    crate::commands::roles::require_perm(&state, "module.operations.edit")?;
    let ws = workspace_id(&state)?;
    let id = new_id();
    let ts = now_iso();
    let qty = input.inventory_qty.unwrap_or(0);
    let conn = state.conn.lock();
    conn.execute(
        "INSERT INTO products (id, workspace_id, name, sku, description, inventory_qty, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, NULL, 1)",
        rusqlite::params![id, ws, input.name, input.sku, input.description, qty, ts],
    )
    .map_err(|e| e.to_string())?;
    upsert_fts(
        &conn,
        "product",
        &id,
        &input.name,
        &format!(
            "{} {}",
            input.sku.clone().unwrap_or_default(),
            input.description.clone().unwrap_or_default()
        ),
    )
    .map_err(map_err)?;
    record_change(&conn, &ws, "create", "product", &id, &format!("Created product {}", input.name))
        .map_err(map_err)?;
    get_product(&conn, &id)
        .map_err(map_err)?
        .ok_or_else(|| "failed to load product".into())
}

#[tauri::command]
pub fn update_product_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: UpdateProductInput,
) -> Result<Product, String> {
    crate::commands::roles::require_perm(&state, "module.operations.edit")?;
    let ws = workspace_id(&state)?;
    let ts = now_iso();
    let conn = state.conn.lock();
    conn.execute(
        "UPDATE products SET name = ?1, sku = ?2, description = ?3, inventory_qty = ?4, updated_at = ?5, version = version + 1
         WHERE id = ?6 AND deleted_at IS NULL",
        rusqlite::params![input.name, input.sku, input.description, input.inventory_qty, ts, input.id],
    )
    .map_err(|e| e.to_string())?;
    upsert_fts(
        &conn,
        "product",
        &input.id,
        &input.name,
        &format!(
            "{} {}",
            input.sku.clone().unwrap_or_default(),
            input.description.clone().unwrap_or_default()
        ),
    )
    .map_err(map_err)?;
    record_change(
        &conn,
        &ws,
        "update",
        "product",
        &input.id,
        &format!("Updated product {}", input.name),
    )
    .map_err(map_err)?;
    get_product(&conn, &input.id)
        .map_err(map_err)?
        .ok_or_else(|| "product not found".into())
}

#[tauri::command]
pub fn list_assets_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<Vec<Asset>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_assets(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn get_asset_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<Option<Asset>, String> {
    let conn = state.conn.lock();
    get_asset(&conn, &id).map_err(map_err)
}

#[tauri::command]
pub fn create_asset_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateAssetInput,
) -> Result<Asset, String> {
    crate::commands::roles::require_perm(&state, "module.operations.edit")?;
    let ws = workspace_id(&state)?;
    let id = new_id();
    let ts = now_iso();
    let conn = state.conn.lock();
    conn.execute(
        "INSERT INTO assets (id, workspace_id, product_id, company_id, ticket_id, name, serial, warranty_expires_at, notes, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10, NULL, 1)",
        rusqlite::params![
            id,
            ws,
            input.product_id,
            input.company_id,
            input.ticket_id,
            input.name,
            input.serial,
            input.warranty_expires_at,
            input.notes,
            ts
        ],
    )
    .map_err(|e| e.to_string())?;
    upsert_fts(
        &conn,
        "asset",
        &id,
        &input.name,
        &format!(
            "{} {}",
            input.serial.clone().unwrap_or_default(),
            input.notes.clone().unwrap_or_default()
        ),
    )
    .map_err(map_err)?;
    record_change(&conn, &ws, "create", "asset", &id, &format!("Created asset {}", input.name))
        .map_err(map_err)?;
    get_asset(&conn, &id)
        .map_err(map_err)?
        .ok_or_else(|| "failed to load asset".into())
}

#[tauri::command]
pub fn list_rmas_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<Vec<Rma>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_rmas(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn get_rma_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<Option<Rma>, String> {
    let conn = state.conn.lock();
    get_rma(&conn, &id).map_err(map_err)
}

#[tauri::command]
pub fn create_rma_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateRmaInput,
) -> Result<Rma, String> {
    crate::commands::roles::require_perm(&state, "module.operations.edit")?;
    let ws = workspace_id(&state)?;
    let id = new_id();
    let ts = now_iso();
    let status = input.status.unwrap_or_else(|| "open".into());
    let conn = state.conn.lock();
    conn.execute(
        "INSERT INTO rmas (id, workspace_id, company_id, asset_id, ticket_id, status, reason, resolution, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, ?8, ?8, NULL, 1)",
        rusqlite::params![
            id,
            ws,
            input.company_id,
            input.asset_id,
            input.ticket_id,
            status,
            input.reason,
            ts
        ],
    )
    .map_err(|e| e.to_string())?;
    upsert_fts(
        &conn,
        "rma",
        &id,
        "RMA",
        input.reason.as_deref().unwrap_or(""),
    )
    .map_err(map_err)?;
    record_change(&conn, &ws, "create", "rma", &id, "Opened an RMA").map_err(map_err)?;
    notify_workspace(
        &conn,
        &ws,
        None,
        "RMA opened",
        input.reason.as_deref().unwrap_or("Replacement workflow started"),
        Some("rma"),
        Some(&id),
    )
    .map_err(map_err)?;
    get_rma(&conn, &id)
        .map_err(map_err)?
        .ok_or_else(|| "failed to load rma".into())
}

#[tauri::command]
pub fn update_rma_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: UpdateRmaInput,
) -> Result<Rma, String> {
    crate::commands::roles::require_perm(&state, "module.operations.edit")?;
    let ws = workspace_id(&state)?;
    let ts = now_iso();
    let conn = state.conn.lock();
    conn.execute(
        "UPDATE rmas SET status = ?1, reason = ?2, resolution = ?3, updated_at = ?4, version = version + 1
         WHERE id = ?5 AND deleted_at IS NULL",
        rusqlite::params![input.status, input.reason, input.resolution, ts, input.id],
    )
    .map_err(|e| e.to_string())?;
    record_change(
        &conn,
        &ws,
        "update",
        "rma",
        &input.id,
        &format!("Updated RMA to {}", input.status),
    )
    .map_err(map_err)?;
    get_rma(&conn, &input.id)
        .map_err(map_err)?
        .ok_or_else(|| "rma not found".into())
}
