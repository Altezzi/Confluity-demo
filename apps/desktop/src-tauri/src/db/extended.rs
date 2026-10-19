use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use super::{DbError, DbResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub sku: Option<String>,
    pub description: Option<String>,
    pub inventory_qty: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub workspace_id: String,
    pub product_id: Option<String>,
    pub company_id: Option<String>,
    pub ticket_id: Option<String>,
    pub name: String,
    pub serial: Option<String>,
    pub warranty_expires_at: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rma {
    pub id: String,
    pub workspace_id: String,
    pub company_id: Option<String>,
    pub asset_id: Option<String>,
    pub ticket_id: Option<String>,
    pub status: String,
    pub reason: Option<String>,
    pub resolution: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub workspace_id: String,
    pub company_id: Option<String>,
    pub status: String,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub total_cents: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLine {
    pub id: String,
    pub invoice_id: String,
    pub description: String,
    pub quantity: i64,
    pub unit_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub workspace_id: String,
    pub company_id: Option<String>,
    pub order_id: Option<String>,
    pub status: String,
    pub reference: Option<String>,
    pub amount_cents: i64,
    pub due_at: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
    pub lines: Vec<InvoiceLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub workspace_id: String,
    pub invoice_id: String,
    pub amount_cents: i64,
    pub method: Option<String>,
    pub received_at: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vendor {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub email: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrder {
    pub id: String,
    pub workspace_id: String,
    pub vendor_id: Option<String>,
    pub product_id: Option<String>,
    pub status: String,
    pub quantity: i64,
    pub unit_cents: i64,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub workspace_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub filename: String,
    pub stored_name: String,
    pub mime: Option<String>,
    pub size_bytes: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub workspace_id: String,
    pub actor: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub summary: String,
    pub metadata: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxItem {
    pub id: String,
    pub workspace_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub action: String,
    pub payload: Option<String>,
    pub status: String,
    pub created_at: String,
    pub sent_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub workspace_id: String,
    pub member_id: Option<String>,
    pub title: String,
    pub body: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub read_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceEvent {
    pub id: String,
    pub workspace_id: String,
    pub event_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub payload: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub server_url: Option<String>,
    pub server_status: String,
    pub last_sync: Option<String>,
    pub queued_changes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePreview {
    pub companies: i64,
    pub contacts: i64,
    pub tickets: i64,
    pub issues: i64,
    pub documents: i64,
    pub products: i64,
    pub assets: i64,
    pub rmas: i64,
    pub orders: i64,
    pub invoices: i64,
}

fn map_product(row: &Row<'_>) -> rusqlite::Result<Product> {
    Ok(Product {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        name: row.get(2)?,
        sku: row.get(3)?,
        description: row.get(4)?,
        inventory_qty: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        deleted_at: row.get(8)?,
        version: row.get(9)?,
    })
}

fn map_asset(row: &Row<'_>) -> rusqlite::Result<Asset> {
    Ok(Asset {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        product_id: row.get(2)?,
        company_id: row.get(3)?,
        ticket_id: row.get(4)?,
        name: row.get(5)?,
        serial: row.get(6)?,
        warranty_expires_at: row.get(7)?,
        notes: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        deleted_at: row.get(11)?,
        version: row.get(12)?,
    })
}

fn map_rma(row: &Row<'_>) -> rusqlite::Result<Rma> {
    Ok(Rma {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        company_id: row.get(2)?,
        asset_id: row.get(3)?,
        ticket_id: row.get(4)?,
        status: row.get(5)?,
        reason: row.get(6)?,
        resolution: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        deleted_at: row.get(10)?,
        version: row.get(11)?,
    })
}

fn map_order(row: &Row<'_>) -> rusqlite::Result<Order> {
    Ok(Order {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        company_id: row.get(2)?,
        status: row.get(3)?,
        reference: row.get(4)?,
        notes: row.get(5)?,
        total_cents: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
        deleted_at: row.get(9)?,
        version: row.get(10)?,
    })
}

fn map_invoice_row(row: &Row<'_>) -> rusqlite::Result<Invoice> {
    Ok(Invoice {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        company_id: row.get(2)?,
        order_id: row.get(3)?,
        status: row.get(4)?,
        reference: row.get(5)?,
        amount_cents: row.get(6)?,
        due_at: row.get(7)?,
        notes: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        deleted_at: row.get(11)?,
        version: row.get(12)?,
        lines: vec![],
    })
}

fn map_payment(row: &Row<'_>) -> rusqlite::Result<Payment> {
    Ok(Payment {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        invoice_id: row.get(2)?,
        amount_cents: row.get(3)?,
        method: row.get(4)?,
        received_at: row.get(5)?,
        notes: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
        deleted_at: row.get(9)?,
        version: row.get(10)?,
    })
}

fn map_vendor(row: &Row<'_>) -> rusqlite::Result<Vendor> {
    Ok(Vendor {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        name: row.get(2)?,
        email: row.get(3)?,
        notes: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
        deleted_at: row.get(7)?,
        version: row.get(8)?,
    })
}

fn map_po(row: &Row<'_>) -> rusqlite::Result<PurchaseOrder> {
    Ok(PurchaseOrder {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        vendor_id: row.get(2)?,
        product_id: row.get(3)?,
        status: row.get(4)?,
        quantity: row.get(5)?,
        unit_cents: row.get(6)?,
        reference: row.get(7)?,
        notes: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        deleted_at: row.get(11)?,
        version: row.get(12)?,
    })
}

fn map_attachment(row: &Row<'_>) -> rusqlite::Result<Attachment> {
    Ok(Attachment {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        entity_type: row.get(2)?,
        entity_id: row.get(3)?,
        filename: row.get(4)?,
        stored_name: row.get(5)?,
        mime: row.get(6)?,
        size_bytes: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        deleted_at: row.get(10)?,
        version: row.get(11)?,
    })
}

const PRODUCT_COLS: &str = "id, workspace_id, name, sku, description, inventory_qty, created_at, updated_at, deleted_at, version";
const ASSET_COLS: &str = "id, workspace_id, product_id, company_id, ticket_id, name, serial, warranty_expires_at, notes, created_at, updated_at, deleted_at, version";
const RMA_COLS: &str = "id, workspace_id, company_id, asset_id, ticket_id, status, reason, resolution, created_at, updated_at, deleted_at, version";
const ORDER_COLS: &str = "id, workspace_id, company_id, status, reference, notes, total_cents, created_at, updated_at, deleted_at, version";
const INVOICE_COLS: &str = "id, workspace_id, company_id, order_id, status, reference, amount_cents, due_at, notes, created_at, updated_at, deleted_at, version";
const PAYMENT_COLS: &str = "id, workspace_id, invoice_id, amount_cents, method, received_at, notes, created_at, updated_at, deleted_at, version";
const VENDOR_COLS: &str = "id, workspace_id, name, email, notes, created_at, updated_at, deleted_at, version";
const PO_COLS: &str = "id, workspace_id, vendor_id, product_id, status, quantity, unit_cents, reference, notes, created_at, updated_at, deleted_at, version";
const ATT_COLS: &str = "id, workspace_id, entity_type, entity_id, filename, stored_name, mime, size_bytes, created_at, updated_at, deleted_at, version";

fn load_invoice_lines(conn: &Connection, invoice_id: &str) -> DbResult<Vec<InvoiceLine>> {
    let mut stmt = conn.prepare(
        "SELECT id, invoice_id, description, quantity, unit_cents FROM invoice_lines WHERE invoice_id = ?1",
    )?;
    let rows = stmt.query_map(params![invoice_id], |row| {
        Ok(InvoiceLine {
            id: row.get(0)?,
            invoice_id: row.get(1)?,
            description: row.get(2)?,
            quantity: row.get(3)?,
            unit_cents: row.get(4)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

fn with_lines(conn: &Connection, mut invoice: Invoice) -> DbResult<Invoice> {
    invoice.lines = load_invoice_lines(conn, &invoice.id)?;
    Ok(invoice)
}

macro_rules! list_by_ws {
    ($fn:ident, $table:expr, $cols:expr, $map:ident, $ty:ty) => {
        pub fn $fn(conn: &Connection, workspace_id: &str) -> DbResult<Vec<$ty>> {
            let mut stmt = conn.prepare(&format!(
                "SELECT {} FROM {} WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY updated_at DESC",
                $cols, $table
            ))?;
            let rows = stmt.query_map(params![workspace_id], $map)?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        }
    };
}

macro_rules! get_by_id {
    ($fn:ident, $table:expr, $cols:expr, $map:ident, $ty:ty) => {
        pub fn $fn(conn: &Connection, id: &str) -> DbResult<Option<$ty>> {
            conn.query_row(
                &format!("SELECT {} FROM {} WHERE id = ?1 AND deleted_at IS NULL", $cols, $table),
                params![id],
                $map,
            )
            .optional()
            .map_err(DbError::from)
        }
    };
}

list_by_ws!(list_products, "products", PRODUCT_COLS, map_product, Product);
get_by_id!(get_product, "products", PRODUCT_COLS, map_product, Product);
list_by_ws!(list_assets, "assets", ASSET_COLS, map_asset, Asset);
get_by_id!(get_asset, "assets", ASSET_COLS, map_asset, Asset);
list_by_ws!(list_rmas, "rmas", RMA_COLS, map_rma, Rma);
get_by_id!(get_rma, "rmas", RMA_COLS, map_rma, Rma);
list_by_ws!(list_orders, "orders", ORDER_COLS, map_order, Order);
get_by_id!(get_order, "orders", ORDER_COLS, map_order, Order);
list_by_ws!(list_payments, "payments", PAYMENT_COLS, map_payment, Payment);
get_by_id!(get_payment, "payments", PAYMENT_COLS, map_payment, Payment);
list_by_ws!(list_vendors, "vendors", VENDOR_COLS, map_vendor, Vendor);
get_by_id!(get_vendor, "vendors", VENDOR_COLS, map_vendor, Vendor);
list_by_ws!(list_purchase_orders, "purchase_orders", PO_COLS, map_po, PurchaseOrder);
get_by_id!(get_purchase_order, "purchase_orders", PO_COLS, map_po, PurchaseOrder);

pub fn list_invoices(conn: &Connection, workspace_id: &str) -> DbResult<Vec<Invoice>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {INVOICE_COLS} FROM invoices WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY updated_at DESC"
    ))?;
    let rows = stmt.query_map(params![workspace_id], map_invoice_row)?;
    let mut out = Vec::new();
    for row in rows.flatten() {
        out.push(with_lines(conn, row)?);
    }
    Ok(out)
}

pub fn get_invoice(conn: &Connection, id: &str) -> DbResult<Option<Invoice>> {
    let invoice = conn
        .query_row(
            &format!("SELECT {INVOICE_COLS} FROM invoices WHERE id = ?1 AND deleted_at IS NULL"),
            params![id],
            map_invoice_row,
        )
        .optional()?;
    match invoice {
        Some(inv) => Ok(Some(with_lines(conn, inv)?)),
        None => Ok(None),
    }
}

pub fn list_attachments(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
) -> DbResult<Vec<Attachment>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {ATT_COLS} FROM attachments WHERE entity_type = ?1 AND entity_id = ?2 AND deleted_at IS NULL ORDER BY created_at DESC"
    ))?;
    let rows = stmt.query_map(params![entity_type, entity_id], map_attachment)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_attachment(conn: &Connection, id: &str) -> DbResult<Option<Attachment>> {
    conn.query_row(
        &format!("SELECT {ATT_COLS} FROM attachments WHERE id = ?1 AND deleted_at IS NULL"),
        params![id],
        map_attachment,
    )
    .optional()
    .map_err(DbError::from)
}

pub fn list_audit(conn: &Connection, workspace_id: &str, limit: i64) -> DbResult<Vec<AuditEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, actor, action, entity_type, entity_id, summary, metadata, created_at
         FROM audit_log WHERE workspace_id = ?1 ORDER BY created_at DESC LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![workspace_id, limit], |row| {
        Ok(AuditEntry {
            id: row.get(0)?,
            workspace_id: row.get(1)?,
            actor: row.get(2)?,
            action: row.get(3)?,
            entity_type: row.get(4)?,
            entity_id: row.get(5)?,
            summary: row.get(6)?,
            metadata: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn list_outbox(conn: &Connection, workspace_id: &str) -> DbResult<Vec<OutboxItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, entity_type, entity_id, action, payload, status, created_at, sent_at
         FROM outbox WHERE workspace_id = ?1 ORDER BY created_at DESC LIMIT 100",
    )?;
    let rows = stmt.query_map(params![workspace_id], |row| {
        Ok(OutboxItem {
            id: row.get(0)?,
            workspace_id: row.get(1)?,
            entity_type: row.get(2)?,
            entity_id: row.get(3)?,
            action: row.get(4)?,
            payload: row.get(5)?,
            status: row.get(6)?,
            created_at: row.get(7)?,
            sent_at: row.get(8)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn list_notifications(conn: &Connection, workspace_id: &str) -> DbResult<Vec<Notification>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, member_id, title, body, entity_type, entity_id, read_at, created_at
         FROM notifications WHERE workspace_id = ?1 ORDER BY created_at DESC LIMIT 200",
    )?;
    let rows = stmt.query_map(params![workspace_id], |row| {
        Ok(Notification {
            id: row.get(0)?,
            workspace_id: row.get(1)?,
            member_id: row.get(2)?,
            title: row.get(3)?,
            body: row.get(4)?,
            entity_type: row.get(5)?,
            entity_id: row.get(6)?,
            read_at: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn list_events(conn: &Connection, workspace_id: &str) -> DbResult<Vec<WorkspaceEvent>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, event_type, entity_type, entity_id, payload, created_at
         FROM workspace_events WHERE workspace_id = ?1 ORDER BY created_at DESC LIMIT 80",
    )?;
    let rows = stmt.query_map(params![workspace_id], |row| {
        Ok(WorkspaceEvent {
            id: row.get(0)?,
            workspace_id: row.get(1)?,
            event_type: row.get(2)?,
            entity_type: row.get(3)?,
            entity_id: row.get(4)?,
            payload: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn count_table(conn: &Connection, table: &str, workspace_id: &str) -> DbResult<i64> {
    conn.query_row(
        &format!("SELECT COUNT(*) FROM {table} WHERE workspace_id = ?1 AND deleted_at IS NULL"),
        params![workspace_id],
        |row| row.get(0),
    )
    .map_err(DbError::from)
}

pub fn workspace_preview(conn: &Connection, workspace_id: &str) -> DbResult<WorkspacePreview> {
    Ok(WorkspacePreview {
        companies: count_table(conn, "companies", workspace_id)?,
        contacts: count_table(conn, "contacts", workspace_id)?,
        tickets: count_table(conn, "tickets", workspace_id)?,
        issues: count_table(conn, "issues", workspace_id)?,
        documents: count_table(conn, "documents", workspace_id)?,
        products: count_table(conn, "products", workspace_id)?,
        assets: count_table(conn, "assets", workspace_id)?,
        rmas: count_table(conn, "rmas", workspace_id)?,
        orders: count_table(conn, "orders", workspace_id)?,
        invoices: count_table(conn, "invoices", workspace_id)?,
    })
}

pub fn assets_for_ticket(conn: &Connection, ticket_id: &str) -> DbResult<Vec<Asset>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {ASSET_COLS} FROM assets WHERE ticket_id = ?1 AND deleted_at IS NULL"
    ))?;
    let rows = stmt.query_map(params![ticket_id], map_asset)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn rmas_for_ticket(conn: &Connection, ticket_id: &str) -> DbResult<Vec<Rma>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {RMA_COLS} FROM rmas WHERE ticket_id = ?1 AND deleted_at IS NULL"
    ))?;
    let rows = stmt.query_map(params![ticket_id], map_rma)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn assets_for_company(conn: &Connection, company_id: &str) -> DbResult<Vec<Asset>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {ASSET_COLS} FROM assets WHERE company_id = ?1 AND deleted_at IS NULL"
    ))?;
    let rows = stmt.query_map(params![company_id], map_asset)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}
