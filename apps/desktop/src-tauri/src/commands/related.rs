use std::sync::Arc;

use crate::db::{
    models::{
        get_company, get_contact, get_document, get_issue, get_ticket, list_contacts, list_documents,
        list_issues, list_tickets, LinkedRecords,
    },
    DbError,
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

#[tauri::command]
pub fn get_related_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    entity_type: String,
    entity_id: String,
) -> Result<LinkedRecords, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();

    let mut linked = LinkedRecords {
        company: None,
        contacts: vec![],
        tickets: vec![],
        issues: vec![],
        documents: vec![],
        products: vec![],
        assets: vec![],
        rmas: vec![],
        orders: vec![],
        invoices: vec![],
    };

    match entity_type.as_str() {
        "company" => {
            linked.company = get_company(&conn, &entity_id).map_err(map_err)?;
            linked.contacts = list_contacts(&conn, &ws, Some(&entity_id)).map_err(map_err)?;
            linked.tickets = list_tickets(&conn, &ws, Some(&entity_id)).map_err(map_err)?;
            linked.issues = list_issues(&conn, &ws, None)
                .map_err(map_err)?
                .into_iter()
                .filter(|i| i.company_id.as_deref() == Some(entity_id.as_str()))
                .collect();
            linked.documents = list_documents(&conn, &ws)
                .map_err(map_err)?
                .into_iter()
                .filter(|d| d.company_id.as_deref() == Some(entity_id.as_str()))
                .collect();
            linked.assets = crate::db::extended::assets_for_company(&conn, &entity_id).map_err(map_err)?;
            linked.rmas = crate::db::extended::list_rmas(&conn, &ws)
                .map_err(map_err)?
                .into_iter()
                .filter(|r| r.company_id.as_deref() == Some(entity_id.as_str()))
                .collect();
            linked.orders = crate::db::extended::list_orders(&conn, &ws)
                .map_err(map_err)?
                .into_iter()
                .filter(|o| o.company_id.as_deref() == Some(entity_id.as_str()))
                .collect();
            linked.invoices = crate::db::extended::list_invoices(&conn, &ws)
                .map_err(map_err)?
                .into_iter()
                .filter(|inv| inv.company_id.as_deref() == Some(entity_id.as_str()))
                .collect();
        }
        "contact" => {
            if let Some(contact) = get_contact(&conn, &entity_id).map_err(map_err)? {
                linked.company = get_company(&conn, &contact.company_id).map_err(map_err)?;
                linked.contacts = vec![contact.clone()];
                linked.tickets = list_tickets(&conn, &ws, Some(&contact.company_id))
                    .map_err(map_err)?
                    .into_iter()
                    .filter(|t| t.contact_id.as_deref() == Some(entity_id.as_str()))
                    .collect();
            }
        }
        "ticket" => {
            if let Some(ticket) = get_ticket(&conn, &entity_id).map_err(map_err)? {
                linked.company = get_company(&conn, &ticket.company_id).map_err(map_err)?;
                if let Some(cid) = &ticket.contact_id {
                    if let Some(c) = get_contact(&conn, cid).map_err(map_err)? {
                        linked.contacts = vec![c];
                    }
                }
                linked.tickets = vec![ticket.clone()];
                linked.issues = list_issues(&conn, &ws, None)
                    .map_err(map_err)?
                    .into_iter()
                    .filter(|i| i.ticket_id.as_deref() == Some(entity_id.as_str()))
                    .collect();
                linked.documents = list_documents(&conn, &ws)
                    .map_err(map_err)?
                    .into_iter()
                    .filter(|d| d.ticket_id.as_deref() == Some(entity_id.as_str()))
                    .collect();
                linked.assets = crate::db::extended::assets_for_ticket(&conn, &entity_id).map_err(map_err)?;
                linked.rmas = crate::db::extended::rmas_for_ticket(&conn, &entity_id).map_err(map_err)?;
                linked.orders = crate::db::extended::list_orders(&conn, &ws)
                    .map_err(map_err)?
                    .into_iter()
                    .filter(|o| o.company_id.as_deref() == Some(ticket.company_id.as_str()))
                    .collect();
                linked.invoices = crate::db::extended::list_invoices(&conn, &ws)
                    .map_err(map_err)?
                    .into_iter()
                    .filter(|inv| inv.company_id.as_deref() == Some(ticket.company_id.as_str()))
                    .collect();
            }
        }
        "issue" => {
            if let Some(issue) = get_issue(&conn, &entity_id).map_err(map_err)? {
                if let Some(cid) = &issue.company_id {
                    linked.company = get_company(&conn, cid).map_err(map_err)?;
                }
                if let Some(tid) = &issue.ticket_id {
                    if let Some(t) = get_ticket(&conn, tid).map_err(map_err)? {
                        linked.tickets = vec![t];
                    }
                }
                linked.issues = vec![issue.clone()];
                linked.documents = list_documents(&conn, &ws)
                    .map_err(map_err)?
                    .into_iter()
                    .filter(|d| d.issue_id.as_deref() == Some(entity_id.as_str()))
                    .collect();
            }
        }
        "document" => {
            if let Some(doc) = get_document(&conn, &entity_id).map_err(map_err)? {
                if let Some(cid) = &doc.company_id {
                    linked.company = get_company(&conn, cid).map_err(map_err)?;
                }
                if let Some(tid) = &doc.ticket_id {
                    if let Some(t) = get_ticket(&conn, tid).map_err(map_err)? {
                        linked.tickets = vec![t];
                    }
                }
                if let Some(iid) = &doc.issue_id {
                    if let Some(i) = get_issue(&conn, iid).map_err(map_err)? {
                        linked.issues = vec![i];
                    }
                }
                linked.documents = vec![doc];
            }
        }
        "asset" => {
            if let Some(asset) = crate::db::extended::get_asset(&conn, &entity_id).map_err(map_err)? {
                if let Some(cid) = &asset.company_id {
                    linked.company = get_company(&conn, cid).map_err(map_err)?;
                }
                if let Some(tid) = &asset.ticket_id {
                    if let Some(t) = get_ticket(&conn, tid).map_err(map_err)? {
                        linked.tickets = vec![t];
                    }
                }
                if let Some(pid) = &asset.product_id {
                    if let Some(p) = crate::db::extended::get_product(&conn, pid).map_err(map_err)? {
                        linked.products = vec![p];
                    }
                }
                linked.assets = vec![asset.clone()];
                linked.rmas = crate::db::extended::list_rmas(&conn, &ws)
                    .map_err(map_err)?
                    .into_iter()
                    .filter(|r| r.asset_id.as_deref() == Some(entity_id.as_str()))
                    .collect();
            }
        }
        "rma" => {
            if let Some(rma) = crate::db::extended::get_rma(&conn, &entity_id).map_err(map_err)? {
                if let Some(cid) = &rma.company_id {
                    linked.company = get_company(&conn, cid).map_err(map_err)?;
                }
                if let Some(tid) = &rma.ticket_id {
                    if let Some(t) = get_ticket(&conn, tid).map_err(map_err)? {
                        linked.tickets = vec![t];
                    }
                }
                if let Some(aid) = &rma.asset_id {
                    if let Some(a) = crate::db::extended::get_asset(&conn, aid).map_err(map_err)? {
                        linked.assets = vec![a];
                    }
                }
                linked.rmas = vec![rma];
            }
        }
        "product" => {
            if let Some(product) = crate::db::extended::get_product(&conn, &entity_id).map_err(map_err)? {
                linked.products = vec![product];
                linked.assets = crate::db::extended::list_assets(&conn, &ws)
                    .map_err(map_err)?
                    .into_iter()
                    .filter(|a| a.product_id.as_deref() == Some(entity_id.as_str()))
                    .collect();
            }
        }
        "order" => {
            if let Some(order) = crate::db::extended::get_order(&conn, &entity_id).map_err(map_err)? {
                if let Some(cid) = &order.company_id {
                    linked.company = get_company(&conn, cid).map_err(map_err)?;
                }
                linked.orders = vec![order.clone()];
                linked.invoices = crate::db::extended::list_invoices(&conn, &ws)
                    .map_err(map_err)?
                    .into_iter()
                    .filter(|inv| inv.order_id.as_deref() == Some(entity_id.as_str()))
                    .collect();
            }
        }
        "invoice" => {
            if let Some(invoice) = crate::db::extended::get_invoice(&conn, &entity_id).map_err(map_err)? {
                if let Some(cid) = &invoice.company_id {
                    linked.company = get_company(&conn, cid).map_err(map_err)?;
                }
                if let Some(oid) = &invoice.order_id {
                    if let Some(o) = crate::db::extended::get_order(&conn, oid).map_err(map_err)? {
                        linked.orders = vec![o];
                    }
                }
                linked.invoices = vec![invoice];
            }
        }
        _ => {}
    }

    Ok(linked)
}

#[tauri::command]
pub fn list_activities_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    limit: Option<i64>,
) -> Result<Vec<crate::db::models::Activity>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    crate::db::models::list_activities(&conn, &ws, limit.unwrap_or(30)).map_err(map_err)
}
