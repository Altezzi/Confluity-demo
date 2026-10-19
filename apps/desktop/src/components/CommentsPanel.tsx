import { useEffect, useState, type FormEvent } from "react";
import type { Comment, CommentVisibility, Contact, Member, RecordType } from "@confluity/shared";
import { api } from "../lib/api";
import { formatTimestamp } from "../lib/preferences";
import { useApp } from "../context/AppContext";
import { PersonChip, resolvePerson } from "./PersonChip";

export function CommentsPanel({
  entityType,
  entityId,
}: {
  entityType: RecordType | string;
  entityId: string;
}) {
  const { recordPulse, notifyRecordChange, can, preferences } = useApp();
  const [comments, setComments] = useState<Comment[]>([]);
  const [members, setMembers] = useState<Member[]>([]);
  const [contacts, setContacts] = useState<Contact[]>([]);
  const [body, setBody] = useState("");
  const [visibility, setVisibility] = useState<CommentVisibility>("internal");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const canSee = can("panel.comments");
  const canSeeInternal = can("support.field.internal_comments") || can("panel.comments");

  async function reload() {
    try {
      const [rows, people, companyPeople] = await Promise.all([
        api.listComments(entityType, entityId),
        api.listMembers(),
        api.listContacts(),
      ]);
      const filtered = canSeeInternal
        ? rows
        : rows.filter((c) => c.visibility !== "internal");
      setComments(filtered);
      setMembers(people);
      setContacts(companyPeople);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  useEffect(() => {
    void reload();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [entityType, entityId, recordPulse]);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    if (!body.trim() || busy) return;
    setBusy(true);
    setError(null);
    try {
      const created = await api.createComment({
        entity_type: entityType,
        entity_id: entityId,
        body: body.trim(),
        visibility,
      });
      setBody("");
      setComments((curr) => (curr.some((c) => c.id === created.id) ? curr : [...curr, created]));
      notifyRecordChange();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  if (!canSee) return null;

  return (
    <div className="panel" id="comments">
      <div className="panel-head">
        <h2>Comments</h2>
        <span className="hint" style={{ color: "var(--text-tertiary)", fontWeight: 400 }}>
          {comments.length}
        </span>
      </div>
      {error && <p style={{ color: "var(--danger)", padding: "0 18px" }}>{error}</p>}
      <div className="comment-list">
        {comments.map((c) => {
          const person = resolvePerson(c.author, members, contacts);
          return (
            <div key={c.id} className="comment-item">
              <div className="comment-meta">
                <PersonChip name={person.name} department={person.department} />
                <span className={`badge ${c.visibility === "internal" ? "waiting" : "open"}`}>
                  {c.visibility}
                </span>
                <span>{formatTimestamp(c.created_at, preferences.timeFormat)}</span>
              </div>
              <p>{c.body}</p>
            </div>
          );
        })}
        {comments.length === 0 && !error && <div className="empty">No comments yet</div>}
      </div>
      <form className="form-grid comment-form" onSubmit={onSubmit}>
        <div className="form-row">
          <label>Add comment</label>
          <textarea
            value={body}
            onChange={(e) => setBody(e.target.value)}
            placeholder="Write a note. Use @Jordan Hale to mention someone."
          />
        </div>
        <div className="form-row">
          <label>Visibility</label>
          <select
            value={visibility}
            onChange={(e) => setVisibility(e.target.value as CommentVisibility)}
          >
            <option value="internal">internal</option>
            <option value="public">public</option>
          </select>
        </div>
        <button className="btn btn-primary" type="submit" disabled={busy || !body.trim()}>
          Add comment
        </button>
      </form>
    </div>
  );
}
