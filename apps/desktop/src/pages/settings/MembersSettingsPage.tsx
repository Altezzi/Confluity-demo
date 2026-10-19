import { useEffect, useMemo, useState, type FormEvent } from "react";
import { Link } from "react-router-dom";
import type { Member, OrgCategory, Position, Role } from "@confluity/shared";
import { api } from "../../lib/api";
import { useApp } from "../../context/AppContext";

export function MembersSettingsPage() {
  const { can, refreshPermissions } = useApp();
  const [members, setMembers] = useState<Member[]>([]);
  const [roles, setRoles] = useState<Role[]>([]);
  const [categories, setCategories] = useState<OrgCategory[]>([]);
  const [positions, setPositions] = useState<Position[]>([]);
  const [query, setQuery] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [assignPositionId, setAssignPositionId] = useState("");
  const [memberName, setMemberName] = useState("");
  const [memberEmail, setMemberEmail] = useState("");
  const [newMemberPositionId, setNewMemberPositionId] = useState("");
  const [issued, setIssued] = useState<{ name: string; email: string; code: string } | null>(null);
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [savedFlash, setSavedFlash] = useState(false);

  const positionsByCategory = useMemo(() => {
    return categories.map((c) => ({
      category: c,
      positions: positions
        .filter((p) => p.category_id === c.id)
        .sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name)),
    }));
  }, [categories, positions]);

  const selected = useMemo(
    () => members.find((m) => m.id === selectedId) ?? null,
    [members, selectedId],
  );

  const matches = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return [];
    return members
      .filter((m) => {
        const pos = positions.find((p) => p.id === m.position_id);
        const cat = pos ? categories.find((c) => c.id === pos.category_id) : null;
        const hay = [m.name, m.email, m.title, pos?.name, cat?.name, m.role]
          .filter(Boolean)
          .join(" ")
          .toLowerCase();
        return hay.includes(q);
      })
      .slice(0, 20);
  }, [members, positions, categories, query]);

  async function reload(preferId?: string | null) {
    const [m, r, cats, pos] = await Promise.all([
      api.listMembers(),
      api.listRoles(),
      api.listOrgCategories().catch(() => [] as OrgCategory[]),
      api.listPositions().catch(() => [] as Position[]),
    ]);
    setMembers(m);
    setRoles(r);
    setCategories(cats);
    setPositions(pos);
    if (!newMemberPositionId && pos[0]) setNewMemberPositionId(pos[0].id);
    const nextId = preferId ?? selectedId;
    if (nextId && m.some((row) => row.id === nextId)) {
      setSelectedId(nextId);
      const member = m.find((row) => row.id === nextId);
      setAssignPositionId(member?.position_id ?? "");
    }
  }

  useEffect(() => {
    void reload().catch((e) => setError(e instanceof Error ? e.message : String(e)));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (selected) setAssignPositionId(selected.position_id ?? "");
  }, [selected?.id, selected?.position_id]);

  if (!can("settings.manage_members")) {
    return <div className="empty">You do not have permission to manage Members.</div>;
  }

  function positionLabel(positionId: string | null | undefined): string {
    if (!positionId) return "No position";
    const p = positions.find((x) => x.id === positionId);
    if (!p) return "Unknown position";
    const cat = categories.find((c) => c.id === p.category_id);
    return cat ? `${cat.name} · ${p.name}` : p.name;
  }

  function pickMember(member: Member) {
    setSelectedId(member.id);
    setAssignPositionId(member.position_id ?? "");
    setSavedFlash(false);
    setQuery(member.name);
  }

  async function savePosition() {
    if (!selected || busy) return;
    setBusy(true);
    setError(null);
    setSavedFlash(false);
    try {
      await api.setMemberPosition(selected.id, assignPositionId || null);
      await reload(selected.id);
      await refreshPermissions();
      setSavedFlash(true);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function copyCode(code: string) {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
    } catch {
      setCopied(false);
      setError("Could not copy the code. Select it and copy it manually.");
    }
  }

  async function showCode(memberId: string) {
    setBusy(true);
    setError(null);
    setCopied(false);
    try {
      const code = await api.revealMemberInvite(memberId);
      const member = members.find((m) => m.id === memberId);
      setIssued({
        name: member?.name ?? "Member",
        email: member?.email ?? "",
        code,
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function addMember(e: FormEvent) {
    e.preventDefault();
    if (!memberName.trim() || !memberEmail.trim() || busy) return;
    setBusy(true);
    setError(null);
    setCopied(false);
    try {
      const created = await api.inviteMember({
        name: memberName.trim(),
        email: memberEmail.trim(),
        position_id: newMemberPositionId || null,
      });
      setIssued({
        name: created.member.name,
        email: created.member.email ?? memberEmail.trim(),
        code: created.invite_code,
      });
      setMemberName("");
      setMemberEmail("");
      setQuery(created.member.name);
      await reload(created.member.id);
      await refreshPermissions();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  const effectiveRoles = selected
    ? roles.filter((r) => (selected.role_ids ?? []).includes(r.id))
    : [];

  return (
    <div className="panel">
      <h2 style={{ marginTop: 0 }}>Members</h2>
      <p className="meta" style={{ padding: "0 18px 8px", margin: 0 }}>
        Search for someone in the company, then assign their <strong>position</strong>. Access comes
        from roles on that position — manage packs under{" "}
        <Link to="/admin/roles">Roles</Link> and attach them under{" "}
        <Link to="/admin/positions">Positions</Link>.
      </p>
      {error && <p style={{ color: "var(--danger)", padding: "0 18px" }}>{error}</p>}

      <div className="form-grid" style={{ padding: "8px 18px 0", gap: 10 }}>
        <div className="form-row">
          <label htmlFor="member-search">Find member</label>
          <input
            id="member-search"
            type="search"
            value={query}
            placeholder="Search by name, title, or position…"
            autoComplete="off"
            onChange={(e) => {
              setQuery(e.target.value);
              setSavedFlash(false);
              if (selected && e.target.value.trim().toLowerCase() !== selected.name.toLowerCase()) {
                setSelectedId(null);
              }
            }}
          />
        </div>
      </div>

      {query.trim() ? (
        <div className="list" style={{ marginTop: 8 }}>
          {matches.length === 0 ? (
            <div className="empty" style={{ padding: 18 }}>
              No members match “{query.trim()}”.
            </div>
          ) : (
            matches.map((m) => (
              <button
                key={m.id}
                type="button"
                className={`list-item${selected?.id === m.id ? " active" : ""}`}
                style={{ width: "100%", textAlign: "left" }}
                onClick={() => pickMember(m)}
              >
                <div>
                  <div className="title">{m.name}</div>
                  <div className="meta">
                    {[m.email, positionLabel(m.position_id), m.must_set_password ? "Waiting for password" : null]
                      .filter(Boolean)
                      .join(" · ")}
                  </div>
                </div>
              </button>
            ))
          )}
        </div>
      ) : (
        <p className="meta" style={{ padding: "12px 18px" }}>
          Type a name to find someone already in the workspace ({members.length} members).
        </p>
      )}

      {selected && (
        <div className="settings-subblock" style={{ margin: "12px 18px 0" }}>
          <div className="settings-subblock-head">
            <h4 style={{ margin: 0 }}>{selected.name}</h4>
            {savedFlash && <span className="badge open">Position saved</span>}
          </div>
          <div className="form-grid" style={{ gap: 10 }}>
            {selected.email && (
              <div className="form-row">
                <label>Email</label>
                <div className="meta" style={{ padding: 0 }}>
                  {selected.email}
                </div>
              </div>
            )}
            {selected.must_set_password && (
              <div className="form-actions" style={{ marginTop: 0 }}>
                <button className="btn" type="button" disabled={busy} onClick={() => void showCode(selected.id)}>
                  Show sign-in code
                </button>
              </div>
            )}
            <div className="form-row">
              <label>Current</label>
              <div className="meta" style={{ padding: 0 }}>
                {positionLabel(selected.position_id)}
              </div>
            </div>
            <div className="form-row">
              <label>Assign position</label>
              <select
                value={assignPositionId}
                disabled={busy}
                onChange={(e) => {
                  setAssignPositionId(e.target.value);
                  setSavedFlash(false);
                }}
              >
                <option value="">Unassigned</option>
                {positionsByCategory.map(({ category, positions: plist }) => (
                  <optgroup key={category.id} label={category.name}>
                    {plist.map((p) => (
                      <option key={p.id} value={p.id}>
                        {p.name}
                      </option>
                    ))}
                  </optgroup>
                ))}
              </select>
            </div>
            <div className="form-row">
              <label>Roles from position</label>
              <div
                className="member-role-chips"
                style={{ maxWidth: "100%", justifyContent: "flex-start" }}
              >
                {effectiveRoles.map((role) => (
                  <span
                    key={role.id}
                    className="role-chip active"
                    style={{ ["--role-color" as string]: role.color, cursor: "default" }}
                  >
                    {role.name}
                  </span>
                ))}
                {effectiveRoles.length === 0 && (
                  <span className="meta">
                    {assignPositionId || selected.position_id
                      ? "No roles on this position yet"
                      : "Pick a position to grant access"}
                  </span>
                )}
              </div>
            </div>
            <div className="form-actions" style={{ marginTop: 4 }}>
              <button
                className="btn btn-primary"
                type="button"
                disabled={busy || (assignPositionId || "") === (selected.position_id || "")}
                onClick={() => void savePosition()}
              >
                Save position
              </button>
              <button
                className="btn"
                type="button"
                disabled={busy}
                onClick={() => {
                  setSelectedId(null);
                  setQuery("");
                  setSavedFlash(false);
                }}
              >
                Clear
              </button>
            </div>
          </div>
        </div>
      )}

      {issued && (
        <div className="invite-issued">
          <div className="title">Sign-in code for {issued.name}</div>
          <p className="meta" style={{ margin: 0, padding: 0 }}>
            {issued.email} signs in with this code, then chooses a password. The code stops working
            after that.
          </p>
          <p className="invite-code">{issued.code}</p>
          <div className="form-actions" style={{ marginTop: 0 }}>
            <button className="btn" type="button" onClick={() => void copyCode(issued.code)}>
              {copied ? "Copied" : "Copy code"}
            </button>
          </div>
        </div>
      )}

      <form className="form-grid" style={{ marginTop: 20, borderTop: "1px solid var(--border)", paddingTop: 16 }} onSubmit={addMember}>
        <div className="nav-label" style={{ padding: "0 0 4px" }}>
          Add member
        </div>
        <p className="meta" style={{ margin: 0, padding: 0 }}>
          Enter their name and email. Confluity creates a one-time sign-in code and a Supabase
          account. They set their own password on first sign-in.
        </p>
        <div className="form-row">
          <label>Name</label>
          <input
            value={memberName}
            onChange={(e) => setMemberName(e.target.value)}
            placeholder="New person"
          />
        </div>
        <div className="form-row">
          <label>Email</label>
          <input
            type="email"
            value={memberEmail}
            onChange={(e) => setMemberEmail(e.target.value)}
            placeholder="name@company.com"
            autoComplete="off"
          />
        </div>
        <div className="form-row">
          <label>Starting position</label>
          <select
            value={newMemberPositionId}
            onChange={(e) => setNewMemberPositionId(e.target.value)}
          >
            <option value="">Unassigned</option>
            {positionsByCategory.map(({ category, positions: plist }) => (
              <optgroup key={category.id} label={category.name}>
                {plist.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name}
                  </option>
                ))}
              </optgroup>
            ))}
          </select>
        </div>
        <button
          className="btn btn-primary"
          type="submit"
          disabled={busy || !memberName.trim() || !memberEmail.trim()}
        >
          Add member
        </button>
      </form>
    </div>
  );
}
