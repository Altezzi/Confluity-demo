import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import type { Asset, Comment, Contact, Document, Ticket } from "@confluity/shared";
import { api, formatLabel, recordPath } from "../lib/api";
import { useApp } from "../context/AppContext";

export function PortalPage() {
  const { setFocus } = useApp();
  const [contact, setContact] = useState<Contact | null>(null);
  const [ticket, setTicket] = useState<Ticket | null>(null);
  const [comments, setComments] = useState<Comment[]>([]);
  const [docs, setDocs] = useState<Document[]>([]);
  const [assets, setAssets] = useState<Asset[]>([]);

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "Portal preview" });
    void (async () => {
      const [contacts, tickets, documents, allAssets] = await Promise.all([
        api.listContacts(),
        api.listTickets(),
        api.listDocuments(),
        api.listAssets(),
      ]);
      const person = contacts.find((c) => c.name === "Maya Chen") ?? contacts[0] ?? null;
      setContact(person);
      const open = tickets.find((t) => t.contact_id && t.contact_id === person?.id) ?? tickets[0] ?? null;
      setTicket(open);
      if (open) {
        const rows = await api.listComments("ticket", open.id);
        setComments(rows.filter((c) => c.visibility === "public"));
      }
      setDocs(documents.filter((d) => d.visibility === "public"));
      setAssets(allAssets.filter((a) => !person || a.company_id === person.company_id));
    })();
  }, [setFocus]);

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <div className="eyebrow">Customer portal preview</div>
          <h1>Welcome{contact ? `, ${contact.name}` : ""}</h1>
          <p>Public tickets, knowledge, and account assets. Internal notes stay hidden.</p>
        </div>
      </div>
      <div className="grid-2">
        <div className="panel">
          <div className="panel-head">
            <h2>Your ticket</h2>
            {ticket && (
              <Link className="hint" to={recordPath("ticket", ticket.id)}>
                Open in workspace
              </Link>
            )}
          </div>
          {ticket ? (
            <div className="detail-block">
              <p>
                <strong>{ticket.title}</strong>
              </p>
              <p>
                <span className={`badge ${ticket.status}`}>{formatLabel(ticket.status)}</span>
              </p>
              <p>{ticket.description}</p>
              <h2>Updates</h2>
              {comments.map((c) => (
                <div key={c.id} className="comment-item">
                  <div className="comment-meta">
                    <strong>{c.author}</strong>
                    <span>{c.created_at ? new Date(c.created_at).toLocaleString() : ""}</span>
                  </div>
                  <p>{c.body}</p>
                </div>
              ))}
              {comments.length === 0 && <div className="empty">No public updates yet</div>}
            </div>
          ) : (
            <div className="empty">No customer ticket</div>
          )}
        </div>
        <div>
          <div className="panel">
            <div className="panel-head">
              <h2>Knowledge</h2>
            </div>
            <div className="list">
              {docs.map((d) => (
                <Link key={d.id} className="list-item" to={recordPath("document", d.id)}>
                  <div>
                    <div className="title">{d.title}</div>
                    <div className="meta">{d.category || "Article"}</div>
                  </div>
                </Link>
              ))}
            </div>
          </div>
          <div className="panel" style={{ marginTop: 16 }}>
            <div className="panel-head">
              <h2>Your assets</h2>
            </div>
            <div className="list">
              {assets.map((a) => (
                <div key={a.id} className="list-item">
                  <div>
                    <div className="title">{a.name}</div>
                    <div className="meta">{a.serial || "No serial"}</div>
                  </div>
                </div>
              ))}
              {assets.length === 0 && <div className="empty">No assets on this account</div>}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
