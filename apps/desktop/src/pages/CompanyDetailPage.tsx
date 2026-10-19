import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import type { Company, Contact, Ticket } from "@confluity/shared";
import { api, formatLabel, recordPath } from "../lib/api";
import { CommentsPanel } from "../components/CommentsPanel";
import { LinkedRecordsPanel } from "../components/LinkedRecords";
import { PersonChip } from "../components/PersonChip";
import { useApp } from "../context/AppContext";

export function CompanyDetailPage() {
  const { id = "" } = useParams();
  const { setFocus, can } = useApp();
  const [company, setCompany] = useState<Company | null>(null);
  const [contacts, setContacts] = useState<Contact[]>([]);
  const [tickets, setTickets] = useState<Ticket[]>([]);

  useEffect(() => {
    void (async () => {
      const c = await api.getCompany(id);
      setCompany(c);
      if (c) {
        setFocus({ entityType: "company", entityId: c.id, label: c.name });
        const [people, tix] = await Promise.all([
          api.listContacts(c.id),
          api.listTickets(c.id),
        ]);
        setContacts(people);
        setTickets(tix);
      }
    })();
  }, [id, setFocus]);

  if (!company) return <div className="empty">Company not found</div>;

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <div className="eyebrow">Company</div>
          <h1>{company.name}</h1>
          <p>
            {company.domain || "No domain"} · {company.industry || "No industry"}
          </p>
        </div>
        <Link className="btn" to="/crm">
          Back to CRM
        </Link>
      </div>

      <div className="grid-2">
        <div>
          <div className="panel detail-block">
            <h2>Notes</h2>
            <p>
              {can("crm.field.notes")
                ? company.notes || "No notes yet."
                : "Notes hidden by your role."}
            </p>
          </div>
          <div className="panel detail-block">
            <h2>Contacts</h2>
            <div className="list">
              {contacts.map((c) => (
                <div key={c.id} className="list-item">
                  <div>
                    <PersonChip name={c.name} department={c.title} />
                    <div className="meta">
                      {can("crm.field.contact_pii")
                        ? c.email || "No email"
                        : "Contact details hidden"}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
          <div className="panel">
            <h2 style={{ marginTop: 0 }}>Tickets</h2>
            <div className="list">
              {tickets.map((t) => (
                <Link key={t.id} className="list-item" to={recordPath("ticket", t.id)}>
                  <div>
                    <div className="title">{t.title}</div>
                    <div className="meta">
                      <span className={`badge ${t.status}`}>{formatLabel(t.status)}</span>
                    </div>
                  </div>
                </Link>
              ))}
            </div>
          </div>
        </div>
        <div>
          <LinkedRecordsPanel entityType="company" entityId={company.id} />
          <div style={{ marginTop: 16 }}>
            <CommentsPanel entityType="company" entityId={company.id} />
          </div>
        </div>
      </div>
    </div>
  );
}
