import { useEffect, useRef, useState, type FormEvent } from "react";
import { Link } from "react-router-dom";
import type { AiDraft, AiIntent, RecordType, SearchResult } from "@confluity/shared";
import { api, recordPath } from "../lib/api";
import { useApp } from "../context/AppContext";

interface Msg {
  role: "user" | "assistant";
  content: string;
  citations?: SearchResult[];
  draft?: AiDraft | null;
  applied?: string | null;
}

interface Prompt {
  label: string;
  ask: string;
  intent?: AiIntent;
}

function promptsFor(entityType: RecordType | null): Prompt[] {
  if (entityType === "ticket") {
    return [
      { label: "Summarize this ticket", ask: "Summarize this ticket" },
      { label: "What is blocking this?", ask: "What is blocking this ticket?" },
      { label: "Draft customer reply", ask: "Draft a customer reply for this ticket", intent: "draft_reply" },
      { label: "Draft internal note", ask: "Draft an internal note for this ticket", intent: "draft_note" },
      { label: "Draft knowledge article", ask: "Convert this ticket into a draft knowledge article", intent: "draft_article" },
      { label: "Prepare an RMA", ask: "Prepare an RMA for the linked asset", intent: "prepare_rma" },
      { label: "Create a follow-up ticket", ask: "Create a follow-up ticket for this work", intent: "create_followup" },
      { label: "Create an engineering issue", ask: "Create an engineering issue from this ticket", intent: "create_issue" },
      { label: "Link ticket to issue", ask: "Link this ticket to the engineering issue", intent: "link_issue" },
      { label: "Update this ticket", ask: "Update this ticket with the latest working notes", intent: "update_record" },
      { label: "Propose an action plan", ask: "Propose an action plan for this ticket", intent: "action_plan" },
    ];
  }
  if (entityType === "issue") {
    return [
      { label: "Summarize this issue", ask: "Summarize this issue" },
      { label: "What is blocking this?", ask: "What is blocking this issue?" },
      { label: "Draft customer update", ask: "Draft a customer update from this issue", intent: "draft_update" },
      { label: "Draft internal note", ask: "Draft an internal note for this issue", intent: "draft_note" },
      { label: "Draft knowledge article", ask: "Convert this issue into a draft knowledge article", intent: "draft_article" },
      { label: "Update this issue", ask: "Update this issue with the latest working notes", intent: "update_record" },
      { label: "Propose an action plan", ask: "Propose an action plan for this issue", intent: "action_plan" },
    ];
  }
  if (entityType === "company") {
    return [
      { label: "Summarize this customer", ask: "Summarize this customer" },
      { label: "Show related records", ask: "Show everything related to this customer" },
      { label: "Draft account note", ask: "Draft an internal account note", intent: "draft_note" },
      { label: "Draft customer update", ask: "Draft a customer update for this account", intent: "draft_update" },
    ];
  }
  if (entityType === "document") {
    return [
      { label: "Summarize this doc", ask: "Summarize this document" },
      { label: "Rewrite as article draft", ask: "Rewrite this as a cleaner knowledge article", intent: "draft_article" },
    ];
  }
  return [
    { label: "Summarize this customer", ask: "Summarize this customer" },
    { label: "What is blocking this ticket?", ask: "What is blocking this ticket?" },
    { label: "Show everything related to this issue", ask: "Show everything related to this issue" },
    { label: "Find the documentation about this problem", ask: "Find the documentation about this problem" },
    { label: "Draft a knowledge article", ask: "Draft a knowledge article from the current workspace context", intent: "draft_article" },
  ];
}

export function AiPanel() {
  const { aiOpen, setAiOpen, focus, notifyRecordChange } = useApp();
  const [messages, setMessages] = useState<Msg[]>([
    {
      role: "assistant",
      content:
        "Ask about customers, blockers, or docs, or draft a reply, note, or article. Drafts need your approval before anything is saved.",
    },
  ]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [suggestionsOpen, setSuggestionsOpen] = useState(false);
  const suggestionsRef = useRef<HTMLDivElement>(null);
  const prompts = promptsFor(focus.entityType);

  useEffect(() => {
    if (!suggestionsOpen) return;
    function onPointerDown(e: MouseEvent) {
      if (!suggestionsRef.current?.contains(e.target as Node)) {
        setSuggestionsOpen(false);
      }
    }
    document.addEventListener("mousedown", onPointerDown);
    return () => document.removeEventListener("mousedown", onPointerDown);
  }, [suggestionsOpen]);

  async function ask(question: string, intent?: AiIntent) {
    setSuggestionsOpen(false);
    const next = [...messages, { role: "user" as const, content: question }];
    setMessages(next);
    setInput("");
    setLoading(true);
    try {
      const res = await api.aiChat({
        messages: next.map((m) => ({ role: m.role, content: m.content })),
        context_entity_type: focus.entityType,
        context_entity_id: focus.entityId,
        intent: intent ?? "ask",
      });
      setMessages([
        ...next,
        {
          role: "assistant",
          content: res.answer,
          citations: res.citations,
          draft: res.draft
            ? {
                ...res.draft,
                target_entity_type: focus.entityType ?? res.draft.target_entity_type,
                target_entity_id: focus.entityId ?? res.draft.target_entity_id,
              }
            : null,
        },
      ]);
    } catch (e) {
      setMessages([
        ...next,
        {
          role: "assistant",
          content: e instanceof Error ? e.message : String(e),
        },
      ]);
    } finally {
      setLoading(false);
    }
  }

  async function applyDraft(index: number) {
    const msg = messages[index];
    const draft = msg.draft;
    if (!draft || msg.applied) return;
    try {
      if (draft.kind === "rma") {
        const assets = await api.listAssets();
        const asset =
          (focus.entityType === "asset"
            ? assets.find((a) => a.id === focus.entityId)
            : assets.find((a) => a.ticket_id === focus.entityId)) ?? assets[0];
        const rma = await api.createRma({
          asset_id: asset?.id ?? null,
          company_id: asset?.company_id ?? null,
          ticket_id: asset?.ticket_id ?? (focus.entityType === "ticket" ? focus.entityId : null),
          reason: draft.body,
        });
        notifyRecordChange();
        setMessages((curr) =>
          curr.map((m, i) => (i === index ? { ...m, applied: `Opened RMA ${rma.id.slice(0, 8)}` } : m)),
        );
        return;
      }
      if (draft.kind === "ticket") {
        const companies = await api.listCompanies();
        const companyId =
          (focus.entityType === "company" ? focus.entityId : null) ??
          (await api.listTickets()).find((t) => t.id === focus.entityId)?.company_id ??
          companies[0]?.id ??
          "";
        const ticket = await api.createTicket({
          company_id: companyId,
          title: draft.title?.trim() || "Follow-up ticket",
          description: draft.body,
          status: "open",
          priority: "medium",
        });
        notifyRecordChange();
        setMessages((curr) =>
          curr.map((m, i) => (i === index ? { ...m, applied: `Created ticket: ${ticket.title}` } : m)),
        );
        return;
      }
      if (draft.kind === "issue") {
        const ticketId = focus.entityType === "ticket" ? focus.entityId : null;
        const ticket = ticketId ? await api.getTicket(ticketId) : null;
        const issue = await api.createIssue({
          title: draft.title?.trim() || "Engineering follow-up",
          description: draft.body,
          ticket_id: ticketId,
          company_id: ticket?.company_id ?? null,
          status: "todo",
          priority: "high",
        });
        notifyRecordChange();
        setMessages((curr) =>
          curr.map((m, i) => (i === index ? { ...m, applied: `Created issue: ${issue.title}` } : m)),
        );
        return;
      }
      if (draft.kind === "link") {
        const ticketId = focus.entityType === "ticket" ? focus.entityId : null;
        if (!ticketId) {
          setMessages((curr) =>
            curr.map((m, i) => (i === index ? { ...m, applied: "Open a ticket first, then link." } : m)),
          );
          return;
        }
        const ticket = await api.getTicket(ticketId);
        const issues = await api.listIssues();
        const existing = issues.find((i) => i.ticket_id === ticketId);
        if (existing) {
          await api.updateIssue({
            id: existing.id,
            title: existing.title,
            description: existing.description,
            ticket_id: ticketId,
            company_id: existing.company_id ?? ticket?.company_id ?? null,
            project_id: existing.project_id,
            status: existing.status,
            priority: existing.priority,
            blocked_reason: existing.blocked_reason,
            assignee_id: existing.assignee_id,
          });
          notifyRecordChange();
          setMessages((curr) =>
            curr.map((m, i) => (i === index ? { ...m, applied: `Linked to issue: ${existing.title}` } : m)),
          );
          return;
        }
        const issue = await api.createIssue({
          title: draft.title?.trim() || `Engineering: ${ticket?.title ?? "follow-up"}`,
          description: draft.body,
          ticket_id: ticketId,
          company_id: ticket?.company_id ?? null,
          status: "todo",
          priority: "high",
        });
        notifyRecordChange();
        setMessages((curr) =>
          curr.map((m, i) => (i === index ? { ...m, applied: `Linked new issue: ${issue.title}` } : m)),
        );
        return;
      }
      if (draft.kind === "record_update") {
        if (focus.entityType === "ticket" && focus.entityId) {
          const ticket = await api.getTicket(focus.entityId);
          if (!ticket) throw new Error("Ticket not found");
          await api.updateTicket({
            id: ticket.id,
            company_id: ticket.company_id,
            contact_id: ticket.contact_id,
            title: ticket.title,
            description: draft.body,
            status: ticket.status,
            priority: ticket.priority,
            tags: ticket.tags,
            assignee_id: ticket.assignee_id,
          });
          notifyRecordChange();
          setMessages((curr) =>
            curr.map((m, i) => (i === index ? { ...m, applied: "Updated this ticket" } : m)),
          );
          return;
        }
        if (focus.entityType === "issue" && focus.entityId) {
          const issue = await api.getIssue(focus.entityId);
          if (!issue) throw new Error("Issue not found");
          await api.updateIssue({
            id: issue.id,
            title: issue.title,
            description: draft.body,
            ticket_id: issue.ticket_id,
            company_id: issue.company_id,
            project_id: issue.project_id,
            status: issue.status,
            priority: issue.priority,
            blocked_reason: issue.blocked_reason,
            assignee_id: issue.assignee_id,
          });
          notifyRecordChange();
          setMessages((curr) =>
            curr.map((m, i) => (i === index ? { ...m, applied: "Updated this issue" } : m)),
          );
          return;
        }
        setMessages((curr) =>
          curr.map((m, i) => (i === index ? { ...m, applied: "Open a ticket or issue first." } : m)),
        );
        return;
      }
      if (draft.kind === "plan") {
        await api.createComment({
          entity_type: focus.entityType || "ticket",
          entity_id: focus.entityId || "",
          body: draft.body,
          visibility: "internal",
        });
        notifyRecordChange();
        setMessages((curr) =>
          curr.map((m, i) => (i === index ? { ...m, applied: "Saved action plan as an internal note" } : m)),
        );
        return;
      }
      if (draft.kind === "article") {
        const doc = await api.createDocument({
          title: draft.title?.trim() || "Draft knowledge article",
          body: draft.body,
          category: "Support",
          company_id: focus.entityType === "company" ? focus.entityId : null,
          ticket_id: focus.entityType === "ticket" ? focus.entityId : null,
          issue_id: focus.entityType === "issue" ? focus.entityId : null,
        });
        notifyRecordChange();
        setMessages((curr) =>
          curr.map((m, i) =>
            i === index ? { ...m, applied: `Saved as doc: ${doc.title}` } : m,
          ),
        );
        return;
      }
      const entityType = focus.entityType || draft.target_entity_type;
      const entityId = focus.entityId || draft.target_entity_id;
      if (!entityType || !entityId) {
        setMessages((curr) =>
          curr.map((m, i) =>
            i === index ? { ...m, applied: "Open a record first, then apply." } : m,
          ),
        );
        return;
      }
      await api.createComment({
        entity_type: entityType,
        entity_id: entityId,
        body: draft.body,
        visibility: draft.visibility === "public" ? "public" : "internal",
      });
      notifyRecordChange();
      requestAnimationFrame(() => {
        document.getElementById("comments")?.scrollIntoView({ behavior: "smooth", block: "start" });
      });
      const label =
        draft.kind === "note"
          ? "Added as internal note on this record"
          : "Added as comment on this record";
      setMessages((curr) => curr.map((m, i) => (i === index ? { ...m, applied: label } : m)));
    } catch (e) {
      setMessages((curr) =>
        curr.map((m, i) =>
          i === index
            ? { ...m, applied: e instanceof Error ? e.message : String(e) }
            : m,
        ),
      );
    }
  }

  function onSubmit(e: FormEvent) {
    e.preventDefault();
    if (!input.trim() || loading) return;
    void ask(input.trim());
  }

  return (
    <aside className="ai-panel" aria-hidden={!aiOpen} inert={!aiOpen}>
      <div className="ai-header">
        <h2>Assistant</h2>
        <button className="btn btn-ghost" onClick={() => setAiOpen(false)}>
          Close
        </button>
      </div>
      <div className="ai-context">
        {focus.label
          ? `${focus.entityType} · ${focus.label}`
          : "No record selected. Searching the whole workspace."}
      </div>
      <div className="ai-thread">
        <div className="ai-messages">
          {messages.map((m, idx) => (
            <div key={idx} className={`msg ${m.role}`}>
              {m.content}
              {m.citations && m.citations.length > 0 && (
                <div style={{ marginTop: 10 }}>
                  {m.citations.slice(0, 6).map((c) => (
                    <Link
                      key={`${c.entity_type}-${c.entity_id}`}
                      className="linked-link"
                      to={recordPath(c.entity_type, c.entity_id)}
                      style={{ marginBottom: 4 }}
                    >
                      [{c.entity_type}] {c.title}
                    </Link>
                  ))}
                </div>
              )}
              {m.draft && (
                <div className="ai-draft-actions">
                  {m.applied ? (
                    <span className="hint">{m.applied}</span>
                  ) : (
                    <>
                      <button
                        type="button"
                        className="btn btn-primary"
                        onClick={() => void applyDraft(idx)}
                      >
                        {m.draft.kind === "article"
                          ? "Save as doc"
                          : m.draft.kind === "note"
                            ? "Add as note"
                            : m.draft.kind === "rma"
                              ? "Open RMA"
                              : m.draft.kind === "ticket"
                                ? "Create ticket"
                                : m.draft.kind === "issue"
                                  ? "Create issue"
                                  : m.draft.kind === "link"
                                    ? "Link records"
                                    : m.draft.kind === "record_update"
                                      ? "Update record"
                                      : m.draft.kind === "plan"
                                        ? "Approve plan"
                                        : "Add as comment"}
                      </button>
                      <button
                        type="button"
                        className="btn"
                        onClick={() => void navigator.clipboard.writeText(m.draft?.body ?? "")}
                      >
                        Copy
                      </button>
                    </>
                  )}
                </div>
              )}
            </div>
          ))}
          {loading && <div className="msg assistant">Working…</div>}
        </div>
        <div className="ai-composer">
          <div className="ai-composer-tools" ref={suggestionsRef}>
            <button
              type="button"
              className="ai-suggestions-btn"
              disabled={loading}
              aria-label="Suggested questions"
              aria-expanded={suggestionsOpen}
              title="Suggested questions"
              onClick={() => setSuggestionsOpen((open) => !open)}
            >
              ?
            </button>
            {suggestionsOpen && (
              <div className="ai-suggestions-menu" role="menu">
                {prompts.map((p) => (
                  <button
                    key={p.label}
                    type="button"
                    className="ai-suggestions-item"
                    role="menuitem"
                    disabled={loading}
                    onClick={() => void ask(p.ask, p.intent)}
                  >
                    {p.label}
                  </button>
                ))}
              </div>
            )}
          </div>
          <form className="ai-input" onSubmit={onSubmit}>
            <input
              value={input}
              onChange={(e) => setInput(e.target.value)}
              placeholder="Ask or draft from this record…"
              disabled={loading}
            />
            <button className="btn btn-primary" disabled={loading || !input.trim()}>
              Ask
            </button>
          </form>
        </div>
      </div>
    </aside>
  );
}
