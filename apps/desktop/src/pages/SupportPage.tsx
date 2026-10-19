import { useEffect, useMemo, useState, type FormEvent } from "react";
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import type { Company, Contact, Member, Ticket, TicketForm } from "@confluity/shared";
import { api, formatLabel, recordPath } from "../lib/api";
import { TicketFormFields } from "../components/TicketFormFields";
import { emptyTicketDraft, fallbackTicketForm, missingRequired, type TicketDraft } from "../lib/ticketDraft";
import { useApp } from "../context/AppContext";

export function SupportPage() {
  const navigate = useNavigate();
  const { setFocus, moduleMode } = useApp();
  const connected = moduleMode("support") === "connected";
  const [searchParams] = useSearchParams();
  const statusFilter = searchParams.get("status");
  const [tickets, setTickets] = useState<Ticket[]>([]);
  const [companies, setCompanies] = useState<Company[]>([]);
  const [contacts, setContacts] = useState<Contact[]>([]);
  const [members, setMembers] = useState<Member[]>([]);
  const [forms, setForms] = useState<TicketForm[]>([]);
  const [formId, setFormId] = useState("");
  const [draft, setDraft] = useState<TicketDraft>(() => emptyTicketDraft());
  const [formError, setFormError] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);

  async function reload() {
    const [t, c, people, team, ticketForms] = await Promise.all([
      api.listTickets(),
      api.listCompanies(),
      api.listContacts(),
      api.listMembers().catch(() => [] as Member[]),
      api.listTicketForms().catch(() => [] as TicketForm[]),
    ]);
    setTickets(t);
    setCompanies(c);
    setContacts(people);
    setMembers(team);
    setForms(ticketForms);
    const active = ticketForms.filter((form) => form.active);
    const preferred = active.find((form) => form.is_default) ?? active[0] ?? null;
    setFormId((curr) => curr || preferred?.id || "");
    setDraft((curr) => (curr.companyId ? curr : emptyTicketDraft(c[0]?.id ?? "")));
  }

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "Support" });
    void reload();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [setFocus]);

  const activeForms = forms.filter((form) => form.active);
  const selectedForm =
    activeForms.find((form) => form.id === formId) ??
    activeForms.find((form) => form.is_default) ??
    activeForms[0] ??
    (forms.length === 0 ? fallbackTicketForm : null);

  const filtered = useMemo(() => {
    const base =
      statusFilter === "open"
        ? tickets.filter((t) => t.status !== "closed" && t.status !== "resolved")
        : statusFilter
          ? tickets.filter((t) => t.status === statusFilter)
          : tickets;
    const q = query.trim().toLowerCase();
    if (!q) return base;
    return base.filter((t) =>
      `${t.title} ${t.description ?? ""} ${t.status} ${t.priority} ${t.tags ?? ""}`
        .toLowerCase()
        .includes(q),
    );
  }, [tickets, statusFilter, query]);

  function closeModal() {
    setOpen(false);
    setFormError(null);
    setDraft(emptyTicketDraft(companies[0]?.id ?? ""));
  }

  async function createTicket(e: FormEvent) {
    e.preventDefault();
    const problem = missingRequired(selectedForm, draft);
    if (problem || busy) {
      setFormError(problem);
      return;
    }
    setBusy(true);
    setFormError(null);
    try {
      const ticket = await api.createTicket({
        company_id: draft.companyId,
        contact_id: draft.contactId || null,
        title: draft.title.trim(),
        description: draft.description || null,
        priority: draft.priority,
        status: draft.status,
        tags: draft.tags || null,
        assignee_id: draft.assigneeId || null,
        form_id: selectedForm?.id || null,
        custom_fields: draft.custom,
      });
      closeModal();
      await reload();
      navigate(recordPath("ticket", ticket.id));
    } catch (err) {
      setFormError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1>Support</h1>
          <p>Tickets linked to customers, contacts, and engineering issues.</p>
        </div>
        <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
          {statusFilter && (
            <Link className="hint" to="/support">
              View all
            </Link>
          )}
          <button className="btn btn-primary" type="button" onClick={() => setOpen(true)}>
            New ticket
          </button>
        </div>
      </div>

      {connected && (
        <Link className="alert-banner" to="/admin/integrations">
          Support is Connected — data can sync from an external tool. Adopt in Integrations when you
          are ready to make Confluity the source of truth.
        </Link>
      )}

      <div className="panel">
        <div className="docs-toolbar">
          <input
            className="docs-search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={
              statusFilter === "open" ? "Search open tickets…" : "Search tickets…"
            }
          />
        </div>
        <div className="list">
          {filtered.map((t) => (
            <Link key={t.id} className="list-item" to={recordPath("ticket", t.id)}>
              <div>
                <div className="title">{t.title}</div>
                <div className="meta">
                  <span className={`badge ${t.status}`}>{formatLabel(t.status)}</span>{" "}
                  <span className={`badge ${t.priority}`}>{formatLabel(t.priority)}</span>
                  {t.tags ? ` · ${t.tags}` : ""}
                </div>
              </div>
            </Link>
          ))}
          {filtered.length === 0 && (
            <div className="empty">{query ? "No matching tickets" : "No tickets"}</div>
          )}
        </div>
      </div>

      {open && (
        <div className="modal-backdrop" onClick={closeModal}>
          <div
            className="modal modal-form"
            role="dialog"
            aria-modal="true"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-head">
              <h2>New ticket</h2>
              <button className="btn btn-ghost" type="button" onClick={closeModal}>
                Close
              </button>
            </div>
            <form className="form-grid" onSubmit={createTicket}>
              {activeForms.length > 1 && (
                <div className="form-row">
                  <label>Form</label>
                  <select value={selectedForm?.id ?? ""} onChange={(e) => setFormId(e.target.value)}>
                    {activeForms.map((form) => (
                      <option key={form.id} value={form.id}>
                        {form.name}
                      </option>
                    ))}
                  </select>
                </div>
              )}
              {selectedForm ? (
                <TicketFormFields
                  form={selectedForm}
                  draft={draft}
                  onChange={setDraft}
                  companies={companies}
                  contacts={contacts}
                  members={members}
                />
              ) : (
                <p className="meta">No active ticket form yet. An admin can add one in Admin.</p>
              )}
              {formError && <p className="setup-error">{formError}</p>}
              <div className="modal-actions">
                <button className="btn" type="button" onClick={closeModal}>
                  Cancel
                </button>
                <button className="btn btn-primary" type="submit" disabled={busy || !selectedForm}>
                  Create ticket
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
