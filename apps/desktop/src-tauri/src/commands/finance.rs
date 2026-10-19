use serde::Deserialize;
use std::sync::Arc;

use crate::db::{
    extended::{
        get_invoice, get_order, get_purchase_order, get_vendor, list_invoices, list_orders,
        list_payments, list_purchase_orders, list_vendors, Invoice, Order, Payment, PurchaseOrder,
        Vendor,
    },
    new_id, now_iso, record_change, upsert_fts, DbError,
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
pub struct CreateOrderInput {
    pub company_id: Option<String>,
    pub status: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub total_cents: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInvoiceInput {
    pub company_id: Option<String>,
    pub order_id: Option<String>,
    pub status: Option<String>,
    pub reference: Option<String>,
    pub amount_cents: Option<i64>,
    pub due_at: Option<String>,
    pub notes: Option<String>,
    pub lines: Option<Vec<CreateInvoiceLineInput>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInvoiceLineInput {
    pub description: String,
    pub quantity: Option<i64>,
    pub unit_cents: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePaymentInput {
    pub invoice_id: String,
    pub amount_cents: i64,
    pub method: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVendorInput {
    pub name: String,
    pub email: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePurchaseOrderInput {
    pub vendor_id: Option<String>,
    pub product_id: Option<String>,
    pub status: Option<String>,
    pub quantity: Option<i64>,
    pub unit_cents: Option<i64>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn list_orders_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<Vec<Order>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_orders(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn get_order_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<Option<Order>, String> {
    let conn = state.conn.lock();
    get_order(&conn, &id).map_err(map_err)
}

#[tauri::command]
pub fn create_order_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateOrderInput,
) -> Result<Order, String> {
    crate::commands::roles::require_perm(&state, "module.finance.edit")?;
    let ws = workspace_id(&state)?;
    let id = new_id();
    let ts = now_iso();
    let status = input.status.unwrap_or_else(|| "open".into());
    let total = input.total_cents.unwrap_or(0);
    let conn = state.conn.lock();
    conn.execute(
        "INSERT INTO orders (id, workspace_id, company_id, status, reference, notes, total_cents, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8, NULL, 1)",
        rusqlite::params![id, ws, input.company_id, status, input.reference, input.notes, total, ts],
    )
    .map_err(|e| e.to_string())?;
    upsert_fts(
        &conn,
        "order",
        &id,
        input.reference.as_deref().unwrap_or("Order"),
        input.notes.as_deref().unwrap_or(""),
    )
    .map_err(map_err)?;
    record_change(&conn, &ws, "create", "order", &id, "Created order").map_err(map_err)?;
    get_order(&conn, &id)
        .map_err(map_err)?
        .ok_or_else(|| "failed to load order".into())
}

#[tauri::command]
pub fn list_invoices_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<Vec<Invoice>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_invoices(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn get_invoice_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<Option<Invoice>, String> {
    let conn = state.conn.lock();
    get_invoice(&conn, &id).map_err(map_err)
}

#[tauri::command]
pub fn create_invoice_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateInvoiceInput,
) -> Result<Invoice, String> {
    crate::commands::roles::require_perm(&state, "module.finance.edit")?;
    let ws = workspace_id(&state)?;
    let id = new_id();
    let ts = now_iso();
    let status = input.status.unwrap_or_else(|| "open".into());
    let amount = input.amount_cents.unwrap_or(0);
    let conn = state.conn.lock();
    conn.execute(
        "INSERT INTO invoices (id, workspace_id, company_id, order_id, status, reference, amount_cents, due_at, notes, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10, NULL, 1)",
        rusqlite::params![
            id,
            ws,
            input.company_id,
            input.order_id,
            status,
            input.reference,
            amount,
            input.due_at,
            input.notes,
            ts
        ],
    )
    .map_err(|e| e.to_string())?;
    if let Some(lines) = input.lines {
        for line in lines {
            conn.execute(
                "INSERT INTO invoice_lines (id, invoice_id, description, quantity, unit_cents)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    new_id(),
                    id,
                    line.description,
                    line.quantity.unwrap_or(1),
                    line.unit_cents.unwrap_or(0)
                ],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    upsert_fts(
        &conn,
        "invoice",
        &id,
        input.reference.as_deref().unwrap_or("Invoice"),
        input.notes.as_deref().unwrap_or(""),
    )
    .map_err(map_err)?;
    record_change(&conn, &ws, "create", "invoice", &id, "Created invoice").map_err(map_err)?;
    get_invoice(&conn, &id)
        .map_err(map_err)?
        .ok_or_else(|| "failed to load invoice".into())
}

#[tauri::command]
pub fn list_payments_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<Vec<Payment>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_payments(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn create_payment_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreatePaymentInput,
) -> Result<Payment, String> {
    crate::commands::roles::require_perm(&state, "module.finance.edit")?;
    let ws = workspace_id(&state)?;
    let id = new_id();
    let ts = now_iso();
    let conn = state.conn.lock();
    conn.execute(
        "INSERT INTO payments (id, workspace_id, invoice_id, amount_cents, method, received_at, notes, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8, NULL, 1)",
        rusqlite::params![
            id,
            ws,
            input.invoice_id,
            input.amount_cents,
            input.method,
            ts,
            input.notes,
            ts
        ],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE invoices SET status = 'paid', updated_at = ?1, version = version + 1 WHERE id = ?2",
        rusqlite::params![ts, input.invoice_id],
    )
    .map_err(|e| e.to_string())?;
    record_change(
        &conn,
        &ws,
        "create",
        "payment",
        &id,
        "Recorded payment",
    )
    .map_err(map_err)?;
    crate::db::extended::get_payment(&conn, &id)
        .map_err(map_err)?
        .ok_or_else(|| "failed to load payment".into())
}

#[tauri::command]
pub fn list_vendors_cmd(state: tauri::State<'_, Arc<AppState>>) -> Result<Vec<Vendor>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_vendors(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn create_vendor_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreateVendorInput,
) -> Result<Vendor, String> {
    crate::commands::roles::require_perm(&state, "module.finance.edit")?;
    let ws = workspace_id(&state)?;
    let id = new_id();
    let ts = now_iso();
    let conn = state.conn.lock();
    conn.execute(
        "INSERT INTO vendors (id, workspace_id, name, email, notes, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, NULL, 1)",
        rusqlite::params![id, ws, input.name, input.email, input.notes, ts],
    )
    .map_err(|e| e.to_string())?;
    upsert_fts(
        &conn,
        "vendor",
        &id,
        &input.name,
        input.email.as_deref().unwrap_or(""),
    )
    .map_err(map_err)?;
    record_change(&conn, &ws, "create", "vendor", &id, &format!("Created vendor {}", input.name))
        .map_err(map_err)?;
    get_vendor(&conn, &id)
        .map_err(map_err)?
        .ok_or_else(|| "failed to load vendor".into())
}

#[tauri::command]
pub fn list_purchase_orders_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<PurchaseOrder>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_purchase_orders(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn create_purchase_order_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: CreatePurchaseOrderInput,
) -> Result<PurchaseOrder, String> {
    crate::commands::roles::require_perm(&state, "module.finance.edit")?;
    let ws = workspace_id(&state)?;
    let id = new_id();
    let ts = now_iso();
    let status = input.status.unwrap_or_else(|| "open".into());
    let qty = input.quantity.unwrap_or(1);
    let unit = input.unit_cents.unwrap_or(0);
    let conn = state.conn.lock();
    conn.execute(
        "INSERT INTO purchase_orders (id, workspace_id, vendor_id, product_id, status, quantity, unit_cents, reference, notes, created_at, updated_at, deleted_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10, NULL, 1)",
        rusqlite::params![
            id,
            ws,
            input.vendor_id,
            input.product_id,
            status,
            qty,
            unit,
            input.reference,
            input.notes,
            ts
        ],
    )
    .map_err(|e| e.to_string())?;
    if let Some(product_id) = &input.product_id {
        conn.execute(
            "UPDATE products SET inventory_qty = inventory_qty + ?1, updated_at = ?2, version = version + 1 WHERE id = ?3",
            rusqlite::params![qty, ts, product_id],
        )
        .map_err(|e| e.to_string())?;
    }
    upsert_fts(
        &conn,
        "purchase_order",
        &id,
        input.reference.as_deref().unwrap_or("PO"),
        input.notes.as_deref().unwrap_or(""),
    )
    .map_err(map_err)?;
    record_change(&conn, &ws, "create", "purchase_order", &id, "Created purchase order")
        .map_err(map_err)?;
    get_purchase_order(&conn, &id)
        .map_err(map_err)?
        .ok_or_else(|| "failed to load purchase order".into())
}
