import { useEffect, useState } from "react";
import type { AuditEntry, OutboxItem, WorkspaceEvent } from "@confluity/shared";
import { api } from "../../lib/api";
import { useApp } from "../../context/AppContext";

export function ActivitySettingsPage() {
  const { can } = useApp();
  const [outbox, setOutbox] = useState<OutboxItem[]>([]);
  const [events, setEvents] = useState<WorkspaceEvent[]>([]);
  const [audit, setAudit] = useState<AuditEntry[]>([]);

  useEffect(() => {
    void Promise.all([api.listOutbox(), api.listEvents(), api.listAudit(20)]).then(
      ([o, ev, au]) => {
        setOutbox(o);
        setEvents(ev);
        setAudit(au);
      },
    );
  }, []);

  if (!can("settings.data_server") && !can("settings.workspace_admin")) {
    return <div className="empty">You do not have permission to view Activity.</div>;
  }

  return (
    <div className="grid-2">
      <div className="panel">
        <h2 style={{ marginTop: 0 }}>Outbox</h2>
        <div className="list">
          {outbox.slice(0, 12).map((item) => (
            <div key={item.id} className="list-item">
              <div>
                <div className="title">
                  {item.action} · {item.entity_type}
                </div>
                <div className="meta">{item.status}</div>
              </div>
            </div>
          ))}
          {outbox.length === 0 && <div className="empty">Queue is empty</div>}
        </div>
      </div>
      <div className="panel">
        <h2 style={{ marginTop: 0 }}>Event log</h2>
        <div className="list">
          {events.slice(0, 12).map((ev) => (
            <div key={ev.id} className="list-item">
              <div>
                <div className="title">{ev.event_type}</div>
                <div className="meta">{ev.payload}</div>
              </div>
            </div>
          ))}
        </div>
      </div>
      <div className="panel" style={{ gridColumn: "1 / -1" }}>
        <h2 style={{ marginTop: 0 }}>Audit</h2>
        <div className="list">
          {audit.map((a) => (
            <div key={a.id} className="list-item">
              <div>
                <div className="title">{a.summary}</div>
                <div className="meta">
                  {a.action} · {a.entity_type} · {new Date(a.created_at).toLocaleString()}
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
