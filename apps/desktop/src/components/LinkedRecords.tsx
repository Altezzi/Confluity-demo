import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import type { LinkedRecords, RecordType } from "@confluity/shared";
import { api, formatLabel, recordPath } from "../lib/api";
import { useApp } from "../context/AppContext";

export function LinkedRecordsPanel({
  entityType,
  entityId,
}: {
  entityType: RecordType;
  entityId: string;
}) {
  const { can } = useApp();
  const [linked, setLinked] = useState<LinkedRecords | null>(null);
  const allowed = can("panel.linked_records");

  useEffect(() => {
    if (!allowed) return;
    let cancelled = false;
    (async () => {
      const data = await api.getRelated(entityType, entityId);
      if (!cancelled) setLinked(data);
    })();
    return () => {
      cancelled = true;
    };
  }, [entityType, entityId, allowed]);

  if (!allowed) return null;

  if (!linked) return <div className="panel">Loading linked records…</div>;

  return (
    <div className="panel linked-list">
      <div className="linked-group">
        <h3>Linked records</h3>
        {linked.company && entityType !== "company" && (
          <Link className="linked-link" to={recordPath("company", linked.company.id)}>
            <div className="title">Company · {linked.company.name}</div>
            <div className="meta">{linked.company.domain}</div>
          </Link>
        )}
        {linked.contacts.map((c) => (
          <Link key={c.id} className="linked-link" to="/crm">
            <div className="title">Contact · {c.name}</div>
            <div className="meta">{c.email}</div>
          </Link>
        ))}
        {linked.tickets
          .filter((t) => !(entityType === "ticket" && t.id === entityId))
          .map((t) => (
            <Link key={t.id} className="linked-link" to={recordPath("ticket", t.id)}>
              <div className="title">Ticket · {t.title}</div>
              <div className="meta">
                <span className={`badge ${t.status}`}>{formatLabel(t.status)}</span>{" "}
                <span className={`badge ${t.priority}`}>{formatLabel(t.priority)}</span>
              </div>
            </Link>
          ))}
        {linked.issues
          .filter((i) => !(entityType === "issue" && i.id === entityId))
          .map((i) => (
            <Link key={i.id} className="linked-link" to={recordPath("issue", i.id)}>
              <div className="title">Issue · {i.title}</div>
              <div className="meta">
                <span className={`badge ${i.status}`}>{formatLabel(i.status)}</span>
                {i.blocked_reason ? " · blocked" : ""}
              </div>
            </Link>
          ))}
        {linked.documents
          .filter((d) => !(entityType === "document" && d.id === entityId))
          .map((d) => (
            <Link key={d.id} className="linked-link" to={recordPath("document", d.id)}>
              <div className="title">Doc · {d.title}</div>
              <div className="meta">{d.category}</div>
            </Link>
          ))}
        {(linked.products ?? []).map((p) => (
          <Link key={p.id} className="linked-link" to={recordPath("product", p.id)}>
            <div className="title">Product · {p.name}</div>
            <div className="meta">{p.sku}</div>
          </Link>
        ))}
        {(linked.assets ?? []).map((a) => (
          <Link key={a.id} className="linked-link" to={recordPath("asset", a.id)}>
            <div className="title">Asset · {a.name}</div>
            <div className="meta">{a.serial}</div>
          </Link>
        ))}
        {(linked.rmas ?? []).map((r) => (
          <Link key={r.id} className="linked-link" to={recordPath("rma", r.id)}>
            <div className="title">RMA · {r.reason || r.status}</div>
            <div className="meta">
              <span className={`badge ${r.status}`}>{formatLabel(r.status)}</span>
            </div>
          </Link>
        ))}
        {(linked.orders ?? []).map((o) => (
          <Link key={o.id} className="linked-link" to={recordPath("order", o.id)}>
            <div className="title">Order · {o.reference || o.status}</div>
            <div className="meta">{formatLabel(o.status)}</div>
          </Link>
        ))}
        {(linked.invoices ?? []).map((inv) => (
          <Link key={inv.id} className="linked-link" to={recordPath("invoice", inv.id)}>
            <div className="title">Lead · {inv.reference || inv.status}</div>
            <div className="meta">{formatLabel(inv.status)}</div>
          </Link>
        ))}
        {!linked.company &&
          linked.contacts.length === 0 &&
          linked.tickets.filter((t) => !(entityType === "ticket" && t.id === entityId)).length === 0 &&
          linked.issues.filter((i) => !(entityType === "issue" && i.id === entityId)).length === 0 &&
          linked.documents.filter((d) => !(entityType === "document" && d.id === entityId)).length === 0 &&
          (linked.products ?? []).length === 0 &&
          (linked.assets ?? []).length === 0 &&
          (linked.rmas ?? []).length === 0 &&
          (linked.orders ?? []).length === 0 &&
          (linked.invoices ?? []).length === 0 && (
            <div className="empty">No linked records yet.</div>
          )}
      </div>
    </div>
  );
}
