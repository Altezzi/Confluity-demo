import { useEffect, useMemo, useState, type FormEvent } from "react";
import { Link, useNavigate } from "react-router-dom";
import type { Company, Document, Issue, Ticket } from "@confluity/shared";
import { api, recordPath } from "../lib/api";
import { useApp } from "../context/AppContext";

export function DocsPage() {
  const navigate = useNavigate();
  const { setFocus } = useApp();
  const [docs, setDocs] = useState<Document[]>([]);
  const [companies, setCompanies] = useState<Company[]>([]);
  const [tickets, setTickets] = useState<Ticket[]>([]);
  const [issues, setIssues] = useState<Issue[]>([]);
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [category, setCategory] = useState("Support");
  const [companyId, setCompanyId] = useState("");
  const [ticketId, setTicketId] = useState("");
  const [issueId, setIssueId] = useState("");

  async function reload() {
    const [d, c, t, i] = await Promise.all([
      api.listDocuments(),
      api.listCompanies(),
      api.listTickets(),
      api.listIssues(),
    ]);
    setDocs(d);
    setCompanies(c);
    setTickets(t);
    setIssues(i);
  }

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "Docs" });
    void reload();
  }, [setFocus]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return docs;
    return docs.filter((d) => {
      const hay = `${d.title} ${d.category ?? ""} ${d.body}`.toLowerCase();
      return hay.includes(q);
    });
  }, [docs, query]);

  function closeModal() {
    setOpen(false);
    setTitle("");
    setBody("");
    setCategory("Support");
    setCompanyId("");
    setTicketId("");
    setIssueId("");
  }

  async function createDoc(e: FormEvent) {
    e.preventDefault();
    if (!title.trim() || !body.trim() || busy) return;
    setBusy(true);
    try {
      const doc = await api.createDocument({
        title: title.trim(),
        body,
        category: category || null,
        company_id: companyId || null,
        ticket_id: ticketId || null,
        issue_id: issueId || null,
      });
      closeModal();
      await reload();
      navigate(recordPath("document", doc.id));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1>Docs</h1>
          <p>Internal knowledge linked to tickets, issues, and companies.</p>
        </div>
        <button className="btn btn-primary" type="button" onClick={() => setOpen(true)}>
          New document
        </button>
      </div>

      <div className="panel">
        <div className="docs-toolbar">
          <input
            className="docs-search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search documents…"
          />
        </div>
        <div className="list">
          {filtered.map((d) => (
            <Link key={d.id} className="list-item" to={recordPath("document", d.id)}>
              <div>
                <div className="title">{d.title}</div>
                <div className="meta">{d.category || "Uncategorized"}</div>
              </div>
            </Link>
          ))}
          {filtered.length === 0 && (
            <div className="empty">{query ? "No matching documents" : "No documents yet"}</div>
          )}
        </div>
      </div>

      {open && (
        <div className="modal-backdrop" onClick={closeModal}>
          <div
            className="modal modal-form"
            role="dialog"
            aria-modal="true"
            aria-labelledby="new-doc-title"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-head">
              <h2 id="new-doc-title">New document</h2>
              <button className="btn btn-ghost" type="button" onClick={closeModal}>
                Close
              </button>
            </div>
            <form className="form-grid" onSubmit={createDoc}>
              <div className="form-row">
                <label>Title</label>
                <input
                  autoFocus
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  onKeyDown={(e) => e.key === "Escape" && closeModal()}
                />
              </div>
              <div className="form-row">
                <label>Category</label>
                <input value={category} onChange={(e) => setCategory(e.target.value)} />
              </div>
              <div className="form-row">
                <label>Link company</label>
                <select value={companyId} onChange={(e) => setCompanyId(e.target.value)}>
                  <option value="">None</option>
                  {companies.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.name}
                    </option>
                  ))}
                </select>
              </div>
              <div className="form-row">
                <label>Link ticket</label>
                <select value={ticketId} onChange={(e) => setTicketId(e.target.value)}>
                  <option value="">None</option>
                  {tickets.map((t) => (
                    <option key={t.id} value={t.id}>
                      {t.title}
                    </option>
                  ))}
                </select>
              </div>
              <div className="form-row">
                <label>Link issue</label>
                <select value={issueId} onChange={(e) => setIssueId(e.target.value)}>
                  <option value="">None</option>
                  {issues.map((i) => (
                    <option key={i.id} value={i.id}>
                      {i.title}
                    </option>
                  ))}
                </select>
              </div>
              <div className="form-row">
                <label>Body</label>
                <textarea value={body} onChange={(e) => setBody(e.target.value)} rows={8} />
              </div>
              <div className="modal-actions">
                <button className="btn" type="button" onClick={closeModal}>
                  Cancel
                </button>
                <button
                  className="btn btn-primary"
                  type="submit"
                  disabled={busy || !title.trim() || !body.trim()}
                >
                  Create document
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
