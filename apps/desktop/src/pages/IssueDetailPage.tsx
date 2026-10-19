import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import type { Issue, IssuePriority, IssueStatus, Member, Project } from "@confluity/shared";
import { api, formatLabel } from "../lib/api";
import { CommentsPanel } from "../components/CommentsPanel";
import { LinkedRecordsPanel } from "../components/LinkedRecords";
import { PersonChip, memberDepartment } from "../components/PersonChip";
import { useApp } from "../context/AppContext";

export function IssueDetailPage() {
  const { id = "" } = useParams();
  const { setFocus, can } = useApp();
  const [issue, setIssue] = useState<Issue | null>(null);
  const [members, setMembers] = useState<Member[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);

  async function reload() {
    const [i, people, projs] = await Promise.all([
      api.getIssue(id),
      api.listMembers(),
      api.listProjects(),
    ]);
    setIssue(i);
    setMembers(people);
    setProjects(projs);
    if (i) setFocus({ entityType: "issue", entityId: i.id, label: i.title });
  }

  useEffect(() => {
    void reload();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id, setFocus]);

  async function save(patch: Partial<Issue>) {
    if (!issue) return;
    const next = { ...issue, ...patch };
    const updated = await api.updateIssue({
      id: next.id,
      title: next.title,
      description: next.description,
      ticket_id: next.ticket_id,
      company_id: next.company_id,
      project_id: next.project_id,
      status: next.status,
      priority: next.priority,
      blocked_reason: next.blocked_reason,
      assignee_id: next.assignee_id,
    });
    setIssue(updated);
  }

  if (!issue) return <div className="empty">Issue not found</div>;

  const assignee = members.find((m) => m.id === issue.assignee_id);

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <div className="eyebrow">Issue</div>
          <h1>{issue.title}</h1>
          <p className="ticket-byline">
            <span className={`badge ${issue.status}`}>{formatLabel(issue.status)}</span>
            <span className={`badge ${issue.priority}`}>{formatLabel(issue.priority)}</span>
            <PersonChip
              name={assignee?.name ?? "Unassigned"}
              department={assignee ? memberDepartment(assignee) : null}
            />
          </p>
        </div>
        <Link className="btn" to="/projects">
          Back
        </Link>
      </div>

      <div className="grid-2">
        <div>
          <div className="panel detail-block">
            <h2>Description</h2>
            <p>{issue.description || "No description"}</p>
            {issue.blocked_reason && can("projects.field.blocked_reason") && (
              <p>
                <strong>Blocked:</strong> {issue.blocked_reason}
              </p>
            )}
            <div className="form-grid" style={{ marginTop: 16 }}>
              <div className="form-row">
                <label>Status</label>
                <select
                  value={issue.status}
                  onChange={(e) => void save({ status: e.target.value as IssueStatus })}
                >
                  <option value="backlog">backlog</option>
                  <option value="todo">todo</option>
                  <option value="in_progress">in progress</option>
                  <option value="blocked">blocked</option>
                  <option value="done">done</option>
                </select>
              </div>
              <div className="form-row">
                <label>Priority</label>
                <select
                  value={issue.priority}
                  onChange={(e) => void save({ priority: e.target.value as IssuePriority })}
                >
                  <option value="low">low</option>
                  <option value="medium">medium</option>
                  <option value="high">high</option>
                  <option value="critical">critical</option>
                </select>
              </div>
              <div className="form-row">
                <label>Project</label>
                <select
                  value={issue.project_id ?? ""}
                  onChange={(e) => void save({ project_id: e.target.value || null })}
                >
                  <option value="">None</option>
                  {projects.map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name}
                    </option>
                  ))}
                </select>
              </div>
              <div className="form-row">
                <label>Assignee</label>
                <select
                  value={issue.assignee_id ?? ""}
                  onChange={(e) => void save({ assignee_id: e.target.value || null })}
                >
                  <option value="">Unassigned</option>
                  {members.map((m) => (
                    <option key={m.id} value={m.id}>
                      {m.name}
                    </option>
                  ))}
                </select>
              </div>
            </div>
          </div>
          <CommentsPanel entityType="issue" entityId={issue.id} />
        </div>
        <LinkedRecordsPanel entityType="issue" entityId={issue.id} />
      </div>
    </div>
  );
}
