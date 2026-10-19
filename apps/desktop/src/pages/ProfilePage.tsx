import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import type { Issue, Member, OrgCategory, Position, Role, Ticket } from "@confluity/shared";
import { api, formatLabel, recordPath } from "../lib/api";
import { avatarTone, memberPlacement, personInitials } from "../components/PersonChip";
import { useApp } from "../context/AppContext";

function localEmail(name: string, org?: string | null): string {
  const slug = name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, ".")
    .replace(/^\.+|\.+$/g, "");
  const domain = (org || "confluity.local")
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "")
    .slice(0, 24);
  return `${slug || "user"}@${domain || "confluity"}.local`;
}

export function ProfilePage() {
  const { memberId } = useParams();
  const { settings, setFocus, can } = useApp();
  const [member, setMember] = useState<Member | null>(null);
  const [roles, setRoles] = useState<Role[]>([]);
  const [categories, setCategories] = useState<OrgCategory[]>([]);
  const [positions, setPositions] = useState<Position[]>([]);
  const [tickets, setTickets] = useState<Ticket[]>([]);
  const [issues, setIssues] = useState<Issue[]>([]);
  const [missing, setMissing] = useState(false);

  const currentId = settings?.current_member_id ?? null;
  const isSelf = !memberId || memberId === currentId;

  useEffect(() => {
    void (async () => {
      const [members, roleRows, cats, posRows, tix, iss] = await Promise.all([
        api.listMembers(),
        api.listRoles().catch(() => [] as Role[]),
        api.listOrgCategories().catch(() => [] as OrgCategory[]),
        api.listPositions().catch(() => [] as Position[]),
        api.listTickets(),
        api.listIssues(),
      ]);
      const target =
        (memberId ? members.find((m) => m.id === memberId) : null) ??
        members.find((m) => m.id === currentId) ??
        members[0] ??
        null;
      setMember(target);
      setMissing(Boolean(memberId) && !members.some((m) => m.id === memberId));
      setRoles(roleRows);
      setCategories(cats);
      setPositions(posRows);
      setTickets(tix);
      setIssues(iss);
      if (target) {
        setFocus({
          entityType: null,
          entityId: null,
          label: target.id === currentId ? "Profile" : target.name,
        });
      }
    })();
  }, [setFocus, currentId, memberId]);

  if (missing) {
    return (
      <div className="page">
        <div className="empty">Member not found.</div>
        <Link className="btn" to="/teams">
          Back to Teams
        </Link>
      </div>
    );
  }

  if (!member) {
    return (
      <div className="page">
        <div className="empty">No signed-in member found.</div>
      </div>
    );
  }

  const org = settings?.organization_name || "Your company";
  const email = member.email?.trim() || localEmail(member.name, org);
  const tone = avatarTone(member.name);
  const position = positions.find((p) => p.id === member.position_id) ?? null;
  const category = categories.find((c) => c.id === position?.category_id) ?? null;
  const placement = memberPlacement(member, category?.name, position?.name);
  const myRoles = roles.filter((r) => (member.role_ids ?? []).includes(r.id));
  const assignedTickets = tickets.filter(
    (t) => t.assignee_id === member.id && t.status !== "closed" && t.status !== "resolved",
  );
  const assignedIssues = issues.filter(
    (i) => i.assignee_id === member.id && i.status !== "done",
  );
  const about =
    member.title?.trim() && member.title.trim() !== position?.name
      ? member.title.trim()
      : position && category
        ? `${position.name} in ${category.name}.`
        : position
          ? position.name
          : "No position assigned yet.";

  return (
    <div className="page">
      <div className="page-header profile-header">
        <div className="profile-identity">
          <span className="profile-avatar" style={tone} aria-hidden>
            {personInitials(member.name)}
          </span>
          <div>
            <div className="eyebrow">{isSelf ? "Profile" : "Member"}</div>
            <h1>{member.name}</h1>
            <p>
              {placement || "No position assigned"} · {email}
            </p>
          </div>
        </div>
        <div className="form-actions">
          {!isSelf && can("module.teams.view") && (
            <Link className="btn" to={`/teams?dm=${member.id}`}>
              Message
            </Link>
          )}
          {isSelf && (
            <Link className="btn" to="/settings/preferences">
              Preferences
            </Link>
          )}
        </div>
      </div>

      <div className="grid-2">
        <div className="panel">
          <h2 style={{ marginTop: 0 }}>About</h2>
          <div className="detail-block">
            <p>{about}</p>
          </div>
          <div className="detail-block">
            <h2>Email</h2>
            <p className="meta" style={{ padding: 0 }}>
              {email}
            </p>
          </div>
        </div>

        <div className="panel">
          <h2 style={{ marginTop: 0 }}>Role &amp; position</h2>
          <div className="detail-block">
            <p>
              <strong>{position?.name || "Unassigned"}</strong>
              <br />
              <span className="meta">
                {category ? category.name : "No department yet"}
              </span>
            </p>
          </div>
          <div
            className="member-role-chips"
            style={{ maxWidth: "100%", justifyContent: "flex-start", padding: "0 18px 16px" }}
          >
            {myRoles.length > 0 ? (
              myRoles.map((role) => (
                <span
                  key={role.id}
                  className="role-chip active"
                  style={{ ["--role-color" as string]: role.color, cursor: "default" }}
                >
                  {role.name}
                </span>
              ))
            ) : (
              <div className="meta">No roles on this position yet</div>
            )}
          </div>
        </div>
      </div>

      <div className="grid-2" style={{ marginTop: 16 }}>
        <div className="panel">
          <div className="panel-head">
            <h2>Open tickets</h2>
            <span className="meta">{assignedTickets.length}</span>
          </div>
          <div className="list">
            {assignedTickets.map((t) => (
              <Link key={t.id} className="list-item" to={recordPath("ticket", t.id)}>
                <div>
                  <div className="title">{t.title}</div>
                  <div className="meta">
                    <span className={`badge ${t.status}`}>{formatLabel(t.status)}</span>
                  </div>
                </div>
              </Link>
            ))}
            {assignedTickets.length === 0 && <div className="empty">Nothing assigned</div>}
          </div>
        </div>
        <div className="panel">
          <div className="panel-head">
            <h2>Active issues</h2>
            <span className="meta">{assignedIssues.length}</span>
          </div>
          <div className="list">
            {assignedIssues.map((i) => (
              <Link key={i.id} className="list-item" to={recordPath("issue", i.id)}>
                <div>
                  <div className="title">{i.title}</div>
                  <div className="meta">
                    <span className={`badge ${i.status}`}>{formatLabel(i.status)}</span>
                  </div>
                </div>
              </Link>
            ))}
            {assignedIssues.length === 0 && <div className="empty">Nothing assigned</div>}
          </div>
        </div>
      </div>
    </div>
  );
}
