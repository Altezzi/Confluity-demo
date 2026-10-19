import { useEffect, useMemo, useState } from "react";
import { Link, useSearchParams } from "react-router-dom";
import type { Member, Notification } from "@confluity/shared";
import { api, formatLabel, recordPath } from "../lib/api";
import { useApp } from "../context/AppContext";

function relativeTime(iso: string): string {
  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) return "";
  const delta = Date.now() - then;
  const mins = Math.round(delta / 60_000);
  if (mins < 1) return "Just now";
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.round(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.round(hours / 24);
  if (days < 7) return `${days}d ago`;
  return new Date(iso).toLocaleDateString();
}

export function InboxPage() {
  const { setFocus, recordPulse } = useApp();
  const [searchParams, setSearchParams] = useSearchParams();
  const [notes, setNotes] = useState<Notification[]>([]);
  const [members, setMembers] = useState<Member[]>([]);
  const [filter, setFilter] = useState<"all" | "unread">("all");
  const [selectedId, setSelectedId] = useState<string | null>(searchParams.get("id"));

  async function reload() {
    const [n, m] = await Promise.all([api.listNotifications(), api.listMembers().catch(() => [] as Member[])]);
    setNotes(n);
    setMembers(m);
  }

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "Inbox" });
    void reload().catch(() => {
      setNotes([]);
      setMembers([]);
    });
  }, [setFocus, recordPulse]);

  const unread = notes.filter((n) => !n.read_at).length;
  const visible = useMemo(() => {
    if (filter !== "unread") return notes;
    return notes.filter((n) => !n.read_at || n.id === selectedId);
  }, [notes, filter, selectedId]);

  const selected =
    visible.find((n) => n.id === selectedId) ?? visible[0] ?? notes.find((n) => n.id === selectedId) ?? null;

  useEffect(() => {
    if (!selected) return;
    if (selectedId !== selected.id) setSelectedId(selected.id);
  }, [selected, selectedId]);

  async function openNote(note: Notification) {
    setSelectedId(note.id);
    setSearchParams(note.id ? { id: note.id } : {}, { replace: true });
    if (!note.read_at) {
      await api.markNotificationRead(note.id);
      await reload();
    }
  }

  function sender(note: Notification): string {
    if (note.member_id) {
      return members.find((m) => m.id === note.member_id)?.name ?? "Teammate";
    }
    return "Confluity";
  }

  return (
    <div className="page page-wide inbox-page">
      <div className="page-header">
        <div>
          <h1>Inbox</h1>
          <p>
            {unread > 0
              ? `${unread} unread notification${unread === 1 ? "" : "s"}`
              : "You’re caught up"}
          </p>
        </div>
        <div className="form-actions">
          <div className="source-tabs">
            <button
              type="button"
              className={filter === "all" ? "btn btn-primary" : "btn"}
              onClick={() => setFilter("all")}
            >
              All
            </button>
            <button
              type="button"
              className={filter === "unread" ? "btn btn-primary" : "btn"}
              onClick={() => setFilter("unread")}
            >
              Unread{unread > 0 ? ` (${unread})` : ""}
            </button>
          </div>
          {unread > 0 && (
            <button
              className="btn"
              type="button"
              onClick={() => void api.markAllNotificationsRead().then(() => reload())}
            >
              Mark all read
            </button>
          )}
        </div>
      </div>

      <div className="inbox-shell">
        <aside className="inbox-list" aria-label="Notifications">
          {visible.map((n) => {
            const active = selected?.id === n.id;
            return (
              <button
                key={n.id}
                type="button"
                className={`inbox-row${active ? " active" : ""}${n.read_at ? "" : " unread"}`}
                onClick={() => void openNote(n)}
              >
                <span className="inbox-unread-dot" aria-hidden />
                <span className="inbox-row-copy">
                  <span className="inbox-row-top">
                    <span className="title">{sender(n)}</span>
                    <span className="meta">{relativeTime(n.created_at)}</span>
                  </span>
                  <span className="inbox-subject">{n.title}</span>
                  <span className="meta inbox-snippet">{n.body || "No preview"}</span>
                </span>
              </button>
            );
          })}
          {visible.length === 0 && (
            <div className="empty">
              {filter === "unread" ? "No unread notifications" : "Inbox is empty"}
            </div>
          )}
        </aside>

        <section className="inbox-read" aria-label="Message">
          {selected ? (
            <>
              <div className="inbox-read-head">
                <div>
                  <div className="eyebrow">
                    {selected.entity_type ? formatLabel(selected.entity_type) : "Notification"}
                  </div>
                  <h2>{selected.title}</h2>
                  <p className="meta" style={{ padding: 0, margin: "6px 0 0" }}>
                    From {sender(selected)} · {new Date(selected.created_at).toLocaleString()}
                  </p>
                </div>
                {selected.entity_type && selected.entity_id && (
                  <Link className="btn btn-primary" to={recordPath(selected.entity_type, selected.entity_id)}>
                    Open record
                  </Link>
                )}
              </div>
              <div className="inbox-read-body">
                <p>{selected.body || "No additional details."}</p>
              </div>
            </>
          ) : (
            <div className="empty">Select a notification to read it</div>
          )}
        </section>
      </div>
    </div>
  );
}
