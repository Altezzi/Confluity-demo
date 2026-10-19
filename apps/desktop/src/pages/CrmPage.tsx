import { useEffect, useMemo, useState, type FormEvent } from "react";
import { Link, useNavigate } from "react-router-dom";
import type { Company, Contact } from "@confluity/shared";
import { api, recordPath } from "../lib/api";
import { PersonChip } from "../components/PersonChip";
import { useApp } from "../context/AppContext";

type CrmModal = "company" | "contact" | null;

type DraftContact = {
  key: string;
  name: string;
  email: string;
  title: string;
};

function emptyDraftContact(): DraftContact {
  return {
    key: crypto.randomUUID(),
    name: "",
    email: "",
    title: "",
  };
}

export function CrmPage() {
  const navigate = useNavigate();
  const { setFocus } = useApp();
  const [companies, setCompanies] = useState<Company[]>([]);
  const [contacts, setContacts] = useState<Contact[]>([]);
  const [query, setQuery] = useState("");
  const [modal, setModal] = useState<CrmModal>(null);
  const [busy, setBusy] = useState(false);
  const [name, setName] = useState("");
  const [domain, setDomain] = useState("");
  const [draftContacts, setDraftContacts] = useState<DraftContact[]>([]);
  const [contactName, setContactName] = useState("");
  const [contactEmail, setContactEmail] = useState("");
  const [companyId, setCompanyId] = useState("");

  async function reload() {
    const [c, people] = await Promise.all([api.listCompanies(), api.listContacts()]);
    setCompanies(c);
    setContacts(people);
    if (!companyId && c[0]) setCompanyId(c[0].id);
  }

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "CRM" });
    void reload();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [setFocus]);

  const filteredCompanies = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return companies;
    return companies.filter((c) =>
      `${c.name} ${c.domain ?? ""} ${c.industry ?? ""} ${c.notes ?? ""}`.toLowerCase().includes(q),
    );
  }, [companies, query]);

  const filteredContacts = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return contacts;
    return contacts.filter((c) =>
      `${c.name} ${c.email ?? ""} ${c.title ?? ""}`.toLowerCase().includes(q),
    );
  }, [contacts, query]);

  function closeModal() {
    setModal(null);
    setName("");
    setDomain("");
    setDraftContacts([]);
    setContactName("");
    setContactEmail("");
  }

  function openCompanyModal() {
    setDraftContacts([emptyDraftContact()]);
    setModal("company");
  }

  function updateDraftContact(key: string, patch: Partial<DraftContact>) {
    setDraftContacts((rows) => rows.map((row) => (row.key === key ? { ...row, ...patch } : row)));
  }

  function removeDraftContact(key: string) {
    setDraftContacts((rows) => {
      const next = rows.filter((row) => row.key !== key);
      return next.length > 0 ? next : [emptyDraftContact()];
    });
  }

  async function createCompany(e: FormEvent) {
    e.preventDefault();
    if (!name.trim() || busy) return;
    setBusy(true);
    try {
      const company = await api.createCompany({ name: name.trim(), domain: domain || null });
      const people = draftContacts
        .map((row) => ({
          name: row.name.trim(),
          email: row.email.trim() || null,
          title: row.title.trim() || null,
        }))
        .filter((row) => row.name.length > 0);
      for (const person of people) {
        await api.createContact({
          company_id: company.id,
          name: person.name,
          email: person.email,
          title: person.title,
        });
      }
      closeModal();
      await reload();
      navigate(recordPath("company", company.id));
    } finally {
      setBusy(false);
    }
  }

  async function createContact(e: FormEvent) {
    e.preventDefault();
    if (!contactName.trim() || !companyId || busy) return;
    setBusy(true);
    try {
      await api.createContact({
        company_id: companyId,
        name: contactName.trim(),
        email: contactEmail || null,
      });
      closeModal();
      await reload();
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1>CRM</h1>
          <p>Companies and contacts shared across support and engineering.</p>
        </div>
        <div style={{ display: "flex", gap: 8 }}>
          <button className="btn btn-primary" type="button" onClick={openCompanyModal}>
            New company
          </button>
          <button className="btn" type="button" onClick={() => setModal("contact")}>
            New contact
          </button>
        </div>
      </div>

      <div className="panel">
        <div className="docs-toolbar">
          <input
            className="docs-search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search companies and contacts…"
          />
        </div>
        <div className="list">
          {filteredCompanies.map((c) => (
            <Link key={c.id} className="list-item" to={recordPath("company", c.id)}>
              <div>
                <div className="title">{c.name}</div>
                <div className="meta">
                  Company · {c.domain || "No domain"} · {c.industry || "None"}
                </div>
              </div>
            </Link>
          ))}
          {filteredContacts.map((c) => (
            <div key={c.id} className="list-item">
              <div>
                <PersonChip name={c.name} department={c.title} />
                <div className="meta">Contact · {c.email || "No email"}</div>
              </div>
            </div>
          ))}
          {filteredCompanies.length === 0 && filteredContacts.length === 0 && (
            <div className="empty">{query ? "No matching records" : "No companies or contacts yet"}</div>
          )}
        </div>
      </div>

      {modal === "company" && (
        <div className="modal-backdrop" onClick={closeModal}>
          <div
            className="modal modal-form"
            role="dialog"
            aria-modal="true"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-head">
              <h2>New company</h2>
              <button className="btn btn-ghost" type="button" onClick={closeModal}>
                Close
              </button>
            </div>
            <form className="form-grid" onSubmit={createCompany}>
              <div className="form-row">
                <label>Name</label>
                <input
                  autoFocus
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="Acme Corp"
                  onKeyDown={(e) => e.key === "Escape" && closeModal()}
                />
              </div>
              <div className="form-row">
                <label>Domain</label>
                <input
                  value={domain}
                  onChange={(e) => setDomain(e.target.value)}
                  placeholder="acme.com"
                />
              </div>
              <div className="form-row">
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "space-between",
                    gap: 8,
                  }}
                >
                  <label style={{ margin: 0 }}>Contacts</label>
                  <button
                    className="btn btn-ghost"
                    type="button"
                    onClick={() => setDraftContacts((rows) => [...rows, emptyDraftContact()])}
                  >
                    Add contact
                  </button>
                </div>
                <div style={{ display: "grid", gap: 10 }}>
                  {draftContacts.map((row, index) => (
                    <div
                      key={row.key}
                      style={{
                        display: "grid",
                        gap: 8,
                        padding: 10,
                        borderRadius: 10,
                        border: "1px solid var(--border, #2a3140)",
                      }}
                    >
                      <div
                        style={{
                          display: "flex",
                          alignItems: "center",
                          justifyContent: "space-between",
                          gap: 8,
                        }}
                      >
                        <span className="meta">Contact {index + 1}</span>
                        <button
                          className="btn btn-ghost"
                          type="button"
                          onClick={() => removeDraftContact(row.key)}
                          disabled={draftContacts.length === 1 && !row.name && !row.email && !row.title}
                        >
                          Remove
                        </button>
                      </div>
                      <input
                        value={row.name}
                        onChange={(e) => updateDraftContact(row.key, { name: e.target.value })}
                        placeholder="Name"
                      />
                      <input
                        value={row.email}
                        onChange={(e) => updateDraftContact(row.key, { email: e.target.value })}
                        placeholder="Email"
                      />
                      <input
                        value={row.title}
                        onChange={(e) => updateDraftContact(row.key, { title: e.target.value })}
                        placeholder="Title"
                      />
                    </div>
                  ))}
                </div>
                <p className="meta" style={{ margin: 0 }}>
                  Leave a contact blank to skip it. Named contacts are saved with the company.
                </p>
              </div>
              <div className="modal-actions">
                <button className="btn" type="button" onClick={closeModal}>
                  Cancel
                </button>
                <button className="btn btn-primary" type="submit" disabled={busy || !name.trim()}>
                  Create company
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {modal === "contact" && (
        <div className="modal-backdrop" onClick={closeModal}>
          <div
            className="modal modal-form"
            role="dialog"
            aria-modal="true"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-head">
              <h2>New contact</h2>
              <button className="btn btn-ghost" type="button" onClick={closeModal}>
                Close
              </button>
            </div>
            <form className="form-grid" onSubmit={createContact}>
              <div className="form-row">
                <label>Company</label>
                <select value={companyId} onChange={(e) => setCompanyId(e.target.value)}>
                  {companies.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.name}
                    </option>
                  ))}
                </select>
              </div>
              <div className="form-row">
                <label>Name</label>
                <input
                  autoFocus
                  value={contactName}
                  onChange={(e) => setContactName(e.target.value)}
                  onKeyDown={(e) => e.key === "Escape" && closeModal()}
                />
              </div>
              <div className="form-row">
                <label>Email</label>
                <input value={contactEmail} onChange={(e) => setContactEmail(e.target.value)} />
              </div>
              <div className="modal-actions">
                <button className="btn" type="button" onClick={closeModal}>
                  Cancel
                </button>
                <button
                  className="btn btn-primary"
                  type="submit"
                  disabled={busy || !contactName.trim() || !companyId}
                >
                  Create contact
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
