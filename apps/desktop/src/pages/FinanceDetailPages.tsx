import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import type { Invoice, Order } from "@confluity/shared";
import { api, formatLabel } from "../lib/api";
import { LinkedRecordsPanel } from "../components/LinkedRecords";
import { useApp } from "../context/AppContext";

function money(cents: number) {
  return `$${(cents / 100).toFixed(2)}`;
}

export function InvoiceDetailPage() {
  const { id = "" } = useParams();
  const { setFocus, can } = useApp();
  const [invoice, setInvoice] = useState<Invoice | null>(null);
  const [busy, setBusy] = useState(false);

  async function reload() {
    const inv = await api.getInvoice(id);
    setInvoice(inv);
    if (inv) setFocus({ entityType: "invoice", entityId: inv.id, label: inv.reference || "Lead" });
  }

  useEffect(() => {
    void reload();
  }, [id, setFocus]);

  async function pay() {
    if (!invoice) return;
    setBusy(true);
    try {
      await api.createPayment({
        invoice_id: invoice.id,
        amount_cents: invoice.amount_cents,
        method: "ach",
      });
      await reload();
    } finally {
      setBusy(false);
    }
  }

  if (!invoice) return <div className="empty">Lead not found</div>;

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <div className="eyebrow">Lead</div>
          <h1>{invoice.reference || "Lead"}</h1>
          <p>
            <span className={`badge ${invoice.status}`}>{formatLabel(invoice.status)}</span>{" "}
            {can("finance.field.amounts") ? money(invoice.amount_cents) : "Amount hidden"}
          </p>
        </div>
        <div style={{ display: "flex", gap: 8 }}>
          {invoice.status !== "paid" && can("module.finance.edit") && (
            <button className="btn btn-primary" disabled={busy} onClick={() => void pay()}>
              Record payment
            </button>
          )}
          <Link className="btn" to="/finance/leads">
            Back
          </Link>
        </div>
      </div>
      <div className="grid-2">
        <div className="panel">
          <h2>Lines</h2>
          {can("finance.field.line_items") ? (
            <div className="list">
              {(invoice.lines ?? []).map((line) => (
                <div key={line.id} className="list-item">
                  <div>
                    <div className="title">{line.description}</div>
                    <div className="meta">
                      {can("finance.field.amounts")
                        ? `${line.quantity} × ${money(line.unit_cents)}`
                        : `${line.quantity} × hidden`}
                    </div>
                  </div>
                </div>
              ))}
              {(invoice.lines ?? []).length === 0 && <div className="empty">No lines</div>}
            </div>
          ) : (
            <div className="empty">Line items hidden by your role.</div>
          )}
        </div>
        <LinkedRecordsPanel entityType="invoice" entityId={invoice.id} />
      </div>
    </div>
  );
}

export function OrderDetailPage() {
  const { id = "" } = useParams();
  const { setFocus, can } = useApp();
  const [order, setOrder] = useState<Order | null>(null);

  useEffect(() => {
    void (async () => {
      const o = await api.getOrder(id);
      setOrder(o);
      if (o) setFocus({ entityType: "order", entityId: o.id, label: o.reference || "Order" });
    })();
  }, [id, setFocus]);

  if (!order) return <div className="empty">Order not found</div>;

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <div className="eyebrow">Order</div>
          <h1>{order.reference || "Order"}</h1>
          <p>
            <span className={`badge ${order.status}`}>{formatLabel(order.status)}</span>{" "}
            {can("finance.field.amounts") ? money(order.total_cents) : "Amount hidden"}
          </p>
        </div>
        <Link className="btn" to="/finance/orders">
          Back
        </Link>
      </div>
      <div className="grid-2">
        <div className="panel detail-block">
          <h2>Notes</h2>
          <p>{order.notes || "No notes"}</p>
        </div>
        <LinkedRecordsPanel entityType="order" entityId={order.id} />
      </div>
    </div>
  );
}
