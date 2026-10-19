import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import type { Invoice, Order, PurchaseOrder, Vendor } from "@confluity/shared";
import { api, formatLabel, recordPath } from "../lib/api";
import { useApp } from "../context/AppContext";

function money(cents: number) {
  return `$${(cents / 100).toFixed(2)}`;
}

export function FinancePage() {
  const { setFocus } = useApp();
  const [orders, setOrders] = useState<Order[]>([]);
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [vendors, setVendors] = useState<Vendor[]>([]);
  const [pos, setPos] = useState<PurchaseOrder[]>([]);

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "Finance" });
    void (async () => {
      const [o, inv, v, p] = await Promise.all([
        api.listOrders(),
        api.listInvoices(),
        api.listVendors(),
        api.listPurchaseOrders(),
      ]);
      setOrders(o);
      setInvoices(inv);
      setVendors(v);
      setPos(p);
    })();
  }, [setFocus]);

  const openInvoices = invoices.filter((inv) => inv.status !== "paid");

  return (
    <div className="page">
      <div className="home-hero">
        <div className="home-hero-copy">
          <h1>Finance</h1>
          <p>Orders, leads, vendors, and purchase orders.</p>
        </div>
        <div className="home-hero-actions">
          <Link className="btn btn-primary" to="/finance/leads">
            Leads
          </Link>
          <Link className="btn" to="/finance/orders">
            Orders
          </Link>
          <Link className="btn" to="/finance/vendors">
            Vendors
          </Link>
          <Link className="btn" to="/finance/purchase-orders">
            POs
          </Link>
        </div>
      </div>

      {openInvoices.length > 0 && (
        <Link className="alert-banner" to="/finance/leads">
          {openInvoices.length} open lead{openInvoices.length === 1 ? "" : "s"} need attention
        </Link>
      )}

      <div className="stat-row">
        <Link className="stat" to="/finance/leads">
          <div className="value">{invoices.length}</div>
          <div className="label">Leads</div>
        </Link>
        <Link className="stat" to="/finance/orders">
          <div className="value">{orders.length}</div>
          <div className="label">Orders</div>
        </Link>
        <Link className="stat" to="/finance/vendors">
          <div className="value">{vendors.length}</div>
          <div className="label">Vendors</div>
        </Link>
        <Link className="stat" to="/finance/purchase-orders">
          <div className="value">{pos.length}</div>
          <div className="label">Purchase orders</div>
        </Link>
      </div>

      <div className="grid-2">
        <div className="panel">
          <div className="panel-head">
            <h2>Open leads</h2>
            <Link className="hint" to="/finance/leads">
              View all
            </Link>
          </div>
          <div className="list">
            {openInvoices.slice(0, 5).map((inv) => (
              <Link key={inv.id} className="list-item" to={recordPath("invoice", inv.id)}>
                <div>
                  <div className="title">{inv.reference || "Lead"}</div>
                  <div className="meta">
                    <span className={`badge ${inv.status}`}>{formatLabel(inv.status)}</span>{" "}
                    {money(inv.amount_cents)}
                  </div>
                </div>
              </Link>
            ))}
            {openInvoices.length === 0 && <div className="empty">No open leads</div>}
          </div>
        </div>
        <div className="panel">
          <div className="panel-head">
            <h2>Recent orders</h2>
            <Link className="hint" to="/finance/orders">
              View all
            </Link>
          </div>
          <div className="list">
            {orders.slice(0, 5).map((o) => (
              <Link key={o.id} className="list-item" to={recordPath("order", o.id)}>
                <div>
                  <div className="title">{o.reference || "Order"}</div>
                  <div className="meta">
                    <span className={`badge ${o.status}`}>{formatLabel(o.status)}</span>{" "}
                    {money(o.total_cents)}
                  </div>
                </div>
              </Link>
            ))}
            {orders.length === 0 && <div className="empty">No orders yet</div>}
          </div>
        </div>
      </div>
    </div>
  );
}
