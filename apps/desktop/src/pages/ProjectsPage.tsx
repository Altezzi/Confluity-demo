import { useEffect, useState, type FormEvent } from "react";
import { Link, useSearchParams } from "react-router-dom";
import type { Issue, IssuePriority, IssueStatus, Member, Project } from "@confluity/shared";
import { api, formatLabel, recordPath } from "../lib/api";
import { PersonChip, memberDepartment } from "../components/PersonChip";
import { useApp } from "../context/AppContext";

const columns: { id: IssueStatus; label: string }[] = [
  { id: "backlog", label: "Backlog" },
  { id: "todo", label: "Todo" },
  { id: "in_progress", label: "In progress" },
  { id: "blocked", label: "Blocked" },
  { id: "done", label: "Done" },
];

export function ProjectsPage() {
  const { setFocus } = useApp();
  const [searchParams, setSearchParams] = useSearchParams();
  const statusFilter = searchParams.get("status");
  const projectFilter = searchParams.get("project");
  const [issues, setIssues] = useState<Issue[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [members, setMembers] = useState<Member[]>([]);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [priority, setPriority] = useState<IssuePriority>("medium");
  const [status, setStatus] = useState<IssueStatus>("backlog");
  const [blockedReason, setBlockedReason] = useState("");
  const [projectId, setProjectId] = useState("");
  const [assigneeId, setAssigneeId] = useState("");

  async function reload() {
    const [i, p, m] = await Promise.all([
      api.listIssues(projectFilter),
      api.listProjects(),
      api.listMembers(),
    ]);
    setIssues(i);
    setProjects(p);
    setMembers(m);
    if (!projectId && p[0]) setProjectId(p[0].id);
  }

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "Projects" });
    void reload();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [setFocus, projectFilter]);

  const boardIssues = statusFilter ? issues.filter((i) => i.status === statusFilter) : issues;

  async function createIssue(e: FormEvent) {
    e.preventDefault();
    if (!title.trim()) return;
    await api.createIssue({
      title: title.trim(),
      description: description || null,
      priority,
      status,
      blocked_reason: blockedReason || null,
      project_id: projectId || null,
      assignee_id: assigneeId || null,
    });
    setTitle("");
    setDescription("");
    setBlockedReason("");
    await reload();
  }

  async function moveIssue(issue: Issue, nextStatus: IssueStatus) {
    if (issue.status === nextStatus) return;
    await api.updateIssue({
      id: issue.id,
      title: issue.title,
      description: issue.description,
      ticket_id: issue.ticket_id,
      company_id: issue.company_id,
      project_id: issue.project_id,
      status: nextStatus,
      priority: issue.priority,
      blocked_reason: issue.blocked_reason,
      assignee_id: issue.assignee_id,
    });
    await reload();
  }

  function memberById(id: string | null) {
    return members.find((m) => m.id === id) ?? null;
  }

  return (
    <div className="page page-wide">
      <div className="page-header">
        <div>
          <h1>Projects</h1>
          <p>Kanban board for engineering issues, linked back to support and customers.</p>
        </div>
        <div className="home-hero-actions">
          <select
            value={projectFilter ?? ""}
            onChange={(e) => {
              const next = new URLSearchParams(searchParams);
              if (e.target.value) next.set("project", e.target.value);
              else next.delete("project");
              setSearchParams(next);
            }}
          >
            <option value="">All projects</option>
            {projects.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name}
              </option>
            ))}
          </select>
          {statusFilter && (
            <Link className="btn" to="/projects">
              Clear filter
            </Link>
          )}
        </div>
      </div>

      <div className="kanban">
        {columns.map((col) => (
          <div
            key={col.id}
            className="kanban-col"
            onDragOver={(e) => e.preventDefault()}
            onDrop={(e) => {
              const id = e.dataTransfer.getData("issue-id");
              const issue = issues.find((i) => i.id === id);
              if (issue) void moveIssue(issue, col.id);
            }}
          >
            <div className="kanban-col-head">
              <h2>{col.label}</h2>
              <span>{boardIssues.filter((i) => i.status === col.id).length}</span>
            </div>
            {boardIssues
              .filter((i) => i.status === col.id)
              .map((issue) => (
                <article
                  key={issue.id}
                  className="kanban-card"
                  draggable
                  onDragStart={(e) => e.dataTransfer.setData("issue-id", issue.id)}
                >
                  <Link to={recordPath("issue", issue.id)} className="kanban-card-title">
                    {issue.title}
                  </Link>
                  <div className="meta">
                    <span className={`badge ${issue.priority}`}>{formatLabel(issue.priority)}</span>
                    <PersonChip
                      size="sm"
                      name={memberById(issue.assignee_id)?.name ?? "Unassigned"}
                      department={memberDepartment(memberById(issue.assignee_id))}
                    />
                  </div>
                </article>
              ))}
          </div>
        ))}
      </div>

      <div className="panel" style={{ marginTop: 20 }}>
        <h2>New issue</h2>
        <form className="form-grid" onSubmit={createIssue}>
          <div className="form-row">
            <label>Title</label>
            <input value={title} onChange={(e) => setTitle(e.target.value)} />
          </div>
          <div className="form-row">
            <label>Project</label>
            <select value={projectId} onChange={(e) => setProjectId(e.target.value)}>
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
            <select value={assigneeId} onChange={(e) => setAssigneeId(e.target.value)}>
              <option value="">Unassigned</option>
              {members.map((m) => (
                <option key={m.id} value={m.id}>
                  {m.name}
                </option>
              ))}
            </select>
          </div>
          <div className="form-row">
            <label>Status</label>
            <select value={status} onChange={(e) => setStatus(e.target.value as IssueStatus)}>
              {columns.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.id}
                </option>
              ))}
            </select>
          </div>
          <div className="form-row">
            <label>Priority</label>
            <select value={priority} onChange={(e) => setPriority(e.target.value as IssuePriority)}>
              <option value="low">low</option>
              <option value="medium">medium</option>
              <option value="high">high</option>
              <option value="critical">critical</option>
            </select>
          </div>
          <div className="form-row">
            <label>Blocked reason</label>
            <input value={blockedReason} onChange={(e) => setBlockedReason(e.target.value)} />
          </div>
          <div className="form-row">
            <label>Description</label>
            <textarea value={description} onChange={(e) => setDescription(e.target.value)} />
          </div>
          <button className="btn btn-primary" type="submit">
            Create issue
          </button>
        </form>
      </div>
    </div>
  );
}
