import { useEffect, useRef, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import type { ExternalRef, Member, ObsAlert, Ticket, TicketForm, TicketOwnership, TicketStatus } from "@confluity/shared";
import { api, formatLabel, recordPath } from "../lib/api";
import { formatTimestamp } from "../lib/preferences";
import { AttachmentsPanel } from "../components/AttachmentsPanel";
import { CommentsPanel } from "../components/CommentsPanel";
import { LinkedRecordsPanel } from "../components/LinkedRecords";
import { PersonChip, memberDepartment } from "../components/PersonChip";
import { RecordActions } from "../components/RecordActions";
import { useApp } from "../context/AppContext";

type TicketTab = "overview" | "comments" | "linked";

function ownershipLabel(event: TicketOwnership): string {
  if (event.previous_assignee_name) {
    if (event.assignee_name === "Unassigned") {
      return `Unassigned, was ${event.previous_assignee_name}`;
    }
    return `Took over from ${event.previous_assignee_name}`;
  }
  if (event.assignee_name === "Unassigned") return "Opened unassigned";
  return "Assigned";
}

function sourceLabel(source: string): string {
  if (source === "datadog") return "Datadog";
  if (source === "grafana") return "Grafana";
  if (source === "cloudwatch") return "CloudWatch";
  return source;
}

function openInLabel(source: string): string {
  return `Open in ${sourceLabel(source)}`;
}

export function TicketDetailPage() {
  const { id = "" } = useParams();
  const navigate = useNavigate();
  const { setFocus, recordPulse, preferences } = useApp();
  const [ticket, setTicket] = useState<Ticket | null>(null);
  const [members, setMembers] = useState<Member[]>([]);
  const [ownership, setOwnership] = useState<TicketOwnership[]>([]);
  const [obsAlerts, setObsAlerts] = useState<ObsAlert[]>([]);
  const [externalRefs, setExternalRefs] = useState<ExternalRef[]>([]);
  const [forms, setForms] = useState<TicketForm[]>([]);
  const [custom, setCustom] = useState<Record<string, string>>({});
  const [status, setStatus] = useState<TicketStatus>("open");
  const [tab, setTab] = useState<TicketTab>("overview");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const seenPulse = useRef(recordPulse);

  async function reload() {
    const [t, people, history, alerts, refs, ticketForms] = await Promise.all([
      api.getTicket(id),
      api.listMembers(),
      api.listTicketOwnership(id),
      api.listObsAlertsForTicket(id).catch(() => [] as ObsAlert[]),
      api.listExternalRefs("ticket", id).catch(() => [] as ExternalRef[]),
      api.listTicketForms().catch(() => [] as TicketForm[]),
    ]);
    setTicket(t);
    setMembers(people);
    setOwnership(history);
    setObsAlerts(alerts);
    setExternalRefs(refs);
    setForms(ticketForms);
    setCustom({ ...(t?.custom_fields ?? {}) });
    if (t) {
      setStatus(t.status);
      setFocus({ entityType: "ticket", entityId: t.id, label: t.title });
    }
  }

  useEffect(() => {
    setTab("overview");
    seenPulse.current = recordPulse;
    void reload();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id, setFocus]);

  useEffect(() => {
    if (recordPulse === seenPulse.current) return;
    seenPulse.current = recordPulse;
    setTab("comments");
  }, [recordPulse]);

  async function saveStatus() {
    if (!ticket || status === ticket.status) return;
    setBusy(true);
    setError(null);
    try {
      const updated = await api.updateTicket({
        id: ticket.id,
        company_id: ticket.company_id,
        contact_id: ticket.contact_id,
        title: ticket.title,
        description: ticket.description,
        status,
        priority: ticket.priority,
        tags: ticket.tags,
        assignee_id: ticket.assignee_id,
      });
      setTicket(updated);
      setStatus(updated.status);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function escalate() {
    if (!ticket) return;
    setBusy(true);
    setError(null);
    try {
      const issue = await api.escalateTicket({
        ticket_id: ticket.id,
        blocked_reason: "Escalated from support. Needs engineering investigation.",
      });
      navigate(recordPath("issue", issue.id));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function saveCustom() {
    if (!ticket) return;
    setBusy(true);
    setError(null);
    try {
      const updated = await api.updateTicket({
        id: ticket.id,
        company_id: ticket.company_id,
        contact_id: ticket.contact_id,
        title: ticket.title,
        description: ticket.description,
        status: ticket.status,
        priority: ticket.priority,
        tags: ticket.tags,
        assignee_id: ticket.assignee_id,
        form_id: ticket.form_id,
        custom_fields: custom,
      });
      setTicket(updated);
      setCustom({ ...(updated.custom_fields ?? {}) });
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  if (!ticket) return <div className="empty">Ticket not found</div>;

  const assignee = members.find((m) => m.id === ticket.assignee_id);
  const ticketForm =
    forms.find((form) => form.id === ticket.form_id) ??
    forms.find((form) => form.is_default) ??
    null;
  const customPlacements = (ticketForm?.fields ?? []).filter((field) => !field.field.system);

  return (
    <div className="page">
      <div className="page-header ticket-page-header">
        <div className="ticket-header-main">
          <div className="eyebrow">Ticket</div>
          <h1>{ticket.title}</h1>
          <p className="ticket-byline">
            <span className={`badge ${ticket.priority}`}>{formatLabel(ticket.priority)}</span>
            {ticket.tags ? <span>· {ticket.tags}</span> : null}
            {ticketForm ? <span>· {ticketForm.name}</span> : null}
            <PersonChip
              name={assignee?.name ?? "Unassigned"}
              department={assignee ? memberDepartment(assignee) : null}
            />
          </p>
        </div>
      </div>
      <div className="ticket-action-bar">
        <div className="ticket-nav-actions">
          <Link className="btn" to="/support">
            Back
          </Link>
          <button
            className="btn btn-primary"
            disabled={busy || status === ticket.status}
            onClick={() => void saveStatus()}
          >
            Save
          </button>
        </div>
        <div className="ticket-status-save">
          <label>
            Status
            <select
              value={status}
              onChange={(e) => setStatus(e.target.value as TicketStatus)}
            >
              <option value="open">open</option>
              <option value="in_progress">in progress</option>
              <option value="waiting">waiting</option>
              <option value="resolved">resolved</option>
              <option value="closed">closed</option>
            </select>
          </label>
          <button className="btn" disabled={busy} onClick={() => void escalate()}>
            Escalate to engineering
          </button>
          <RecordActions entityType="ticket" entityId={ticket.id} label={ticket.title} />
        </div>
      </div>
      {error && <p style={{ color: "var(--danger)" }}>{error}</p>}

      <div className="ticket-layout">
        <div className="ticket-main">
          <div className="record-tabs" role="tablist">
            <button
              type="button"
              role="tab"
              aria-selected={tab === "overview"}
              className={tab === "overview" ? "record-tab active" : "record-tab"}
              onClick={() => setTab("overview")}
            >
              Overview
            </button>
            <button
              type="button"
              role="tab"
              aria-selected={tab === "comments"}
              className={tab === "comments" ? "record-tab active" : "record-tab"}
              onClick={() => setTab("comments")}
            >
              Comments
            </button>
            <button
              type="button"
              role="tab"
              aria-selected={tab === "linked"}
              className={tab === "linked" ? "record-tab active" : "record-tab"}
              onClick={() => setTab("linked")}
            >
              Linked
            </button>
          </div>

          <div className="record-tab-panels">
            <div role="tabpanel" hidden={tab !== "overview"}>
              <div className="panel detail-block">
                <h2>Description</h2>
                <p>{ticket.description || "No description"}</p>
              </div>
              {customPlacements.length > 0 && (
                <div className="panel detail-block">
                  <h2>{ticketForm?.name ?? "Fields"}</h2>
                  <div className="form-grid">
                    {customPlacements.map((placement) => {
                      const field = placement.field;
                      const value = custom[field.id] ?? "";
                      return (
                        <div className="form-row" key={field.id}>
                          <label>
                            {field.label}
                            {placement.required ? " *" : ""}
                          </label>
                          {field.kind === "textarea" ? (
                            <textarea
                              value={value}
                              onChange={(e) => setCustom({ ...custom, [field.id]: e.target.value })}
                            />
                          ) : field.kind === "dropdown" ? (
                            <select
                              value={value}
                              onChange={(e) => setCustom({ ...custom, [field.id]: e.target.value })}
                            >
                              <option value="">Select</option>
                              {field.options.map((option) => (
                                <option key={option} value={option}>
                                  {option}
                                </option>
                              ))}
                            </select>
                          ) : field.kind === "checkbox" ? (
                            <label className="login-remember">
                              <input
                                type="checkbox"
                                checked={value === "true"}
                                onChange={(e) =>
                                  setCustom({ ...custom, [field.id]: e.target.checked ? "true" : "false" })
                                }
                              />
                              {field.label}
                            </label>
                          ) : (
                            <input
                              type={field.kind === "number" ? "number" : "text"}
                              value={value}
                              onChange={(e) => setCustom({ ...custom, [field.id]: e.target.value })}
                            />
                          )}
                          {field.hint && <p className="meta ticket-field-hint">{field.hint}</p>}
                        </div>
                      );
                    })}
                    <div className="form-actions">
                      <button className="btn btn-primary" type="button" disabled={busy} onClick={() => void saveCustom()}>
                        Save fields
                      </button>
                    </div>
                  </div>
                </div>
              )}
              {obsAlerts.length > 0 && (
                <div className="panel detail-block">
                  <h2>Observability</h2>
                  <div className="obs-alert-list">
                    {obsAlerts.map((alert) => {
                      let rawSummary = "";
                      try {
                        const parsed = JSON.parse(alert.raw_json) as Record<string, unknown>;
                        rawSummary =
                          (typeof parsed.body === "string" && parsed.body) ||
                          (typeof parsed.message === "string" && parsed.message) ||
                          (typeof parsed.NewStateReason === "string" && parsed.NewStateReason) ||
                          alert.title;
                      } catch {
                        rawSummary = alert.title;
                      }
                      return (
                        <div key={alert.id} className="obs-alert-item" style={{ marginBottom: "1rem" }}>
                          <div className="meta" style={{ display: "flex", gap: "0.5rem", flexWrap: "wrap" }}>
                            <span className="badge">{sourceLabel(alert.source)}</span>
                            {alert.severity && (
                              <span className="badge">{formatLabel(alert.severity)}</span>
                            )}
                          </div>
                          <div style={{ marginTop: "0.35rem" }}>{alert.title}</div>
                          <div className="meta" style={{ marginTop: "0.25rem" }}>
                            Fingerprint: {alert.fingerprint}
                          </div>
                          <p className="meta" style={{ marginTop: "0.35rem" }}>
                            {rawSummary.slice(0, 280)}
                            {rawSummary.length > 280 ? "…" : ""}
                          </p>
                          {alert.external_url && (
                            <a href={alert.external_url} target="_blank" rel="noreferrer">
                              {openInLabel(alert.source)}
                            </a>
                          )}
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}
              {externalRefs.length > 0 && (
                <div className="panel detail-block">
                  <h2>External links</h2>
                  <div className="list">
                    {externalRefs.map((ref) => (
                      <div key={ref.id} className="list-item">
                        <div>
                          <div className="title">
                            {formatLabel(ref.provider)} · {ref.external_id}
                          </div>
                          <div className="meta">
                            Synced {formatTimestamp(ref.last_synced_at, preferences.timeFormat)}
                            {ref.sync_conflict ? " · conflict" : ""}
                          </div>
                        </div>
                        {ref.source_url && (
                          <a href={ref.source_url} target="_blank" rel="noreferrer">
                            Open in {formatLabel(ref.provider)}
                          </a>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              )}
              <AttachmentsPanel entityType="ticket" entityId={ticket.id} />
            </div>
            <div role="tabpanel" hidden={tab !== "comments"}>
              <CommentsPanel entityType="ticket" entityId={ticket.id} />
            </div>
            <div role="tabpanel" hidden={tab !== "linked"}>
              <LinkedRecordsPanel entityType="ticket" entityId={ticket.id} />
            </div>
          </div>
        </div>

        <aside className="ticket-aside">
          <div className="panel">
            <div className="panel-head">
              <h2>Ownership</h2>
            </div>
            <div className="ownership-list">
              {ownership.map((event) => {
                const owner = members.find((m) => m.id === event.assignee_id);
                return (
                  <div key={event.id} className="ownership-item">
                    <PersonChip
                      name={event.assignee_name}
                      department={owner ? memberDepartment(owner) : null}
                    />
                    <div className="ownership-copy">
                      <span>{ownershipLabel(event)}</span>
                      <span>{formatTimestamp(event.created_at, preferences.timeFormat)}</span>
                    </div>
                  </div>
                );
              })}
              {ownership.length === 0 && <div className="empty">No ownership history yet</div>}
            </div>
          </div>
        </aside>
      </div>
    </div>
  );
}
