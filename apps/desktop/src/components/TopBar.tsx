import { useEffect, useRef, useState } from "react";
import { Link } from "react-router-dom";
import type { Notification } from "@confluity/shared";
import { api, recordPath } from "../lib/api";
import { alertForNotifications } from "../lib/preferences";
import { useApp } from "../context/AppContext";

export function TopBar() {
  const { setSearchOpen, setAiOpen, aiOpen, recordPulse, preferences } = useApp();
  const [notes, setNotes] = useState<Notification[]>([]);
  const [open, setOpen] = useState(false);
  const knownIds = useRef<Set<string> | null>(null);
  const prefsRef = useRef(preferences);
  prefsRef.current = preferences;

  async function reload() {
    try {
      const next = await api.listNotifications();
      setNotes(next);
      const newcomers = knownIds.current
        ? next.filter((note) => !note.read_at && !knownIds.current?.has(note.id))
        : [];
      knownIds.current = new Set(next.map((note) => note.id));
      if (newcomers.length > 0) alertForNotifications(prefsRef.current, newcomers);
    } catch {
      setNotes([]);
    }
  }

  useEffect(() => {
    void reload();
  }, [recordPulse]);

  const unread = notes.filter((n) => !n.read_at).length;

  return (
    <header className="topbar">
      <button className="search-trigger" onClick={() => setSearchOpen(true)}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
          <circle cx="11" cy="11" r="6.25" />
          <path d="m16 16 3.5 3.5" strokeLinecap="round" />
        </svg>
        <span>Search workspace…</span>
        <span className="kbd" style={{ marginLeft: "auto" }}>
          Ctrl K
        </span>
      </button>
      <div className="topbar-actions">
        <div className="notify-wrap">
          <button className="btn" onClick={() => setOpen((v) => !v)}>
            Notifications{unread > 0 ? ` (${unread})` : ""}
          </button>
          {open && (
            <div className="notify-menu">
              <div className="panel-head">
                <h2>Notifications</h2>
                {unread > 0 && (
                  <button
                    className="hint"
                    type="button"
                    onClick={() => void api.markAllNotificationsRead().then(() => reload())}
                  >
                    Mark all read
                  </button>
                )}
              </div>
              <div className="list">
                {notes.slice(0, 8).map((n) => (
                  <Link
                    key={n.id}
                    className={`list-item${n.read_at ? "" : " unread"}`}
                    to={n.entity_type && n.entity_id ? recordPath(n.entity_type, n.entity_id) : "/inbox"}
                    onClick={() => {
                      void api.markNotificationRead(n.id).then(() => reload());
                      setOpen(false);
                    }}
                  >
                    <div>
                      <div className="title">{n.title}</div>
                      <div className="meta">{n.body}</div>
                    </div>
                  </Link>
                ))}
                {notes.length === 0 && <div className="empty">No notifications</div>}
              </div>
              <div className="notify-menu-foot">
                <Link className="btn" to="/inbox" onClick={() => setOpen(false)}>
                  View all
                </Link>
              </div>
            </div>
          )}
        </div>
        <button className="btn" onClick={() => setAiOpen(!aiOpen)}>
          {aiOpen ? "Hide assistant" : "Assistant"}
        </button>
      </div>
    </header>
  );
}
