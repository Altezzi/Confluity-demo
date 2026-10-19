import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import type { Activity, Company, Issue, Member, Ticket } from "@confluity/shared";
import { api, formatLabel, recordPath } from "../lib/api";
import { PersonChip, memberDepartment } from "../components/PersonChip";
import { useApp } from "../context/AppContext";

export function HomePage() {
  const { setFocus, settings } = useApp();
  const [companies, setCompanies] = useState<Company[]>([]);
  const [tickets, setTickets] = useState<Ticket[]>([]);
  const [issues, setIssues] = useState<Issue[]>([]);
  const [activities, setActivities] = useState<Activity[]>([]);
  const [members, setMembers] = useState<Member[]>([]);

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: null });
    void (async () => {
      const [c, t, i, a, m] = await Promise.all([
        api.listCompanies(),
        api.listTickets(),
        api.listIssues(),
        api.listActivities(12),
        api.listMembers(),
      ]);
      setCompanies(c);
      setTickets(t);
      setIssues(i);
      setActivities(a);
      setMembers(m);
    })();
  }, [setFocus]);

  const me = settings?.current_member_id ?? members[0]?.id ?? null;
  const openTickets = tickets.filter((t) => t.status !== "closed" && t.status !== "resolved");
  const blocked = issues.filter((i) => i.status === "blocked");
  const assignedTickets = openTickets.filter((t) => t.assignee_id && t.assignee_id === me);
  const assignedIssues = issues.filter(
    (i) => i.assignee_id === me && i.status !== "done",
  );

  const meMember = members.find((m) => m.id === me) ?? members[0] ?? null;

  return (
    <div className="page">
      <div className="home-hero">
        <div className="home-hero-copy">
          <h1>Dashboard</h1>
          <p>Customers, tickets, issues, and docs in one workspace.</p>
        </div>
        <div className="home-hero-actions">
          <Link className="btn btn-primary" to="/support">
            Open support
          </Link>
          <Link className="btn" to="/crm">
            View CRM
          </Link>
        </div>
      </div>

      {blocked.length > 0 && (
        <Link className="alert-banner" to="/projects?status=blocked">
          {blocked.length} blocked issue{blocked.length === 1 ? "" : "s"} need attention
        </Link>
      )}

      <div className="stat-row">
        <Link className="stat" to="/crm">
          <div className="value">{companies.length}</div>
          <div className="label">Companies</div>
        </Link>
        <Link className="stat" to="/support?status=open">
          <div className="value">{openTickets.length}</div>
          <div className="label">Open tickets</div>
        </Link>
        <Link className="stat" to="/projects?status=blocked">
          <div className="value">{blocked.length}</div>
          <div className="label">Blocked issues</div>
        </Link>
        <Link className="stat" to="/projects">
          <div className="value">{issues.length}</div>
          <div className="label">Issues</div>
        </Link>
      </div>

      <div className="grid-2">
        <div className="panel">
          <div className="panel-head">
            <h2>Assigned to me</h2>
            {meMember ? (
              <PersonChip
                size="sm"
                name={meMember.name}
                department={memberDepartment(meMember)}
              />
            ) : null}
          </div>
          <div className="list">
            {assignedTickets.map((t) => (
              <Link key={t.id} className="list-item" to={recordPath("ticket", t.id)}>
                <div>
                  <div className="title">{t.title}</div>
                  <div className="meta">
                    <span className={`badge ${t.status}`}>{formatLabel(t.status)}</span>
                    <span className={`badge ${t.priority}`}>{formatLabel(t.priority)}</span>
                  </div>
                </div>
              </Link>
            ))}
            {assignedIssues.map((i) => (
              <Link key={i.id} className="list-item" to={recordPath("issue", i.id)}>
                <div>
                  <div className="title">{i.title}</div>
                  <div className="meta">
                    <span className={`badge ${i.status}`}>{formatLabel(i.status)}</span>
                    <span className={`badge ${i.priority}`}>{formatLabel(i.priority)}</span>
                  </div>
                </div>
              </Link>
            ))}
            {assignedTickets.length === 0 && assignedIssues.length === 0 && (
              <div className="empty">Nothing assigned</div>
            )}
          </div>
        </div>
        <div className="panel">
          <div className="panel-head">
            <h2>Activity</h2>
            <span className="hint" style={{ color: "var(--text-tertiary)", fontWeight: 400 }}>
              Recent
            </span>
          </div>
          <div className="list">
            {activities.map((a) => (
              <Link key={a.id} className="list-item" to={recordPath(a.entity_type, a.entity_id)}>
                <div>
                  <div className="title">{a.summary}</div>
                  <div className="meta">
                    {a.entity_type} · {new Date(a.created_at).toLocaleString()}
                  </div>
                </div>
              </Link>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
