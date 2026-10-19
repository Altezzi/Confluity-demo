import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import type { OrgCategory, Position, Role } from "@confluity/shared";
import { MODULE_PERMISSION_GROUPS, moduleEditKey, moduleViewKey } from "@confluity/shared";
import { api } from "../../lib/api";
import { useApp } from "../../context/AppContext";

function positionDepth(
  positions: Position[],
  id: string,
  seen = new Set<string>(),
): number {
  if (seen.has(id)) return 0;
  seen.add(id);
  const p = positions.find((x) => x.id === id);
  if (!p?.parent_id) return 0;
  return 1 + positionDepth(positions, p.parent_id, seen);
}

function sortTree(positions: Position[], categoryId: string): Position[] {
  const inCat = positions.filter((p) => p.category_id === categoryId);
  const byParent = new Map<string | null, Position[]>();
  for (const p of inCat) {
    const key = p.parent_id;
    const list = byParent.get(key) ?? [];
    list.push(p);
    byParent.set(key, list);
  }
  for (const list of byParent.values()) {
    list.sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name));
  }
  const out: Position[] = [];
  function walk(parentId: string | null) {
    for (const p of byParent.get(parentId) ?? []) {
      out.push(p);
      walk(p.id);
    }
  }
  walk(null);
  return out;
}

type Step = "categories" | "positions" | "detail";

export function PositionsSettingsPage() {
  const { can, refreshPermissions } = useApp();
  const [categories, setCategories] = useState<OrgCategory[]>([]);
  const [positions, setPositions] = useState<Position[]>([]);
  const [roles, setRoles] = useState<Role[]>([]);
  const [step, setStep] = useState<Step>("categories");
  const [categoryId, setCategoryId] = useState<string | null>(null);
  const [positionId, setPositionId] = useState<string | null>(null);
  const [newCatName, setNewCatName] = useState("");
  const [newPosName, setNewPosName] = useState("");
  const [newPosParent, setNewPosParent] = useState("");
  const [roleTabId, setRoleTabId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const selectedCategory = useMemo(
    () => categories.find((c) => c.id === categoryId) ?? null,
    [categories, categoryId],
  );

  const tree = useMemo(
    () => (selectedCategory ? sortTree(positions, selectedCategory.id) : []),
    [positions, selectedCategory],
  );

  const selectedPosition = useMemo(
    () => (positionId ? positions.find((p) => p.id === positionId) ?? null : null),
    [positions, positionId],
  );

  const attachedRoles = useMemo(
    () =>
      (selectedPosition?.role_ids ?? [])
        .map((id) => roles.find((r) => r.id === id))
        .filter((r): r is Role => Boolean(r)),
    [selectedPosition, roles],
  );

  const activeRole = useMemo(
    () => attachedRoles.find((r) => r.id === roleTabId) ?? attachedRoles[0] ?? null,
    [attachedRoles, roleTabId],
  );

  async function reload() {
    const [cats, pos, roleRows] = await Promise.all([
      api.listOrgCategories(),
      api.listPositions(),
      api.listRoles(),
    ]);
    setCategories(cats);
    setPositions(pos);
    setRoles(roleRows);
    setCategoryId((curr) => (curr && cats.some((c) => c.id === curr) ? curr : null));
    setPositionId((curr) => (curr && pos.some((p) => p.id === curr) ? curr : null));
  }

  useEffect(() => {
    void reload().catch((e) => setError(e instanceof Error ? e.message : String(e)));
  }, []);

  useEffect(() => {
    if (!selectedPosition) {
      setRoleTabId(null);
      return;
    }
    setRoleTabId((curr) => {
      if (curr && selectedPosition.role_ids.includes(curr)) return curr;
      return selectedPosition.role_ids[0] ?? null;
    });
  }, [selectedPosition?.id, selectedPosition?.role_ids]);

  if (!can("settings.manage_roles")) {
    return <div className="empty">You do not have permission to manage Positions.</div>;
  }

  function openCategory(id: string) {
    setCategoryId(id);
    setPositionId(null);
    setNewPosName("");
    setNewPosParent("");
    setStep("positions");
  }

  function openPosition(id: string) {
    setPositionId(id);
    setStep("detail");
  }

  function backToCategories() {
    setStep("categories");
    setCategoryId(null);
    setPositionId(null);
  }

  function backToPositions() {
    setStep("positions");
    setPositionId(null);
  }

  async function addCategory() {
    if (!newCatName.trim() || busy) return;
    setBusy(true);
    setError(null);
    try {
      const cat = await api.createOrgCategory(newCatName.trim());
      setNewCatName("");
      await reload();
      openCategory(cat.id);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function addPosition() {
    if (!selectedCategory || !newPosName.trim() || busy) return;
    setBusy(true);
    setError(null);
    try {
      const pos = await api.createPosition({
        category_id: selectedCategory.id,
        name: newPosName.trim(),
        parent_id: newPosParent || null,
      });
      setNewPosName("");
      setNewPosParent("");
      await reload();
      setCategoryId(selectedCategory.id);
      openPosition(pos.id);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function toggleRole(roleId: string) {
    if (!selectedPosition || busy) return;
    const next = new Set(selectedPosition.role_ids);
    const attaching = !next.has(roleId);
    if (attaching) next.add(roleId);
    else next.delete(roleId);
    setBusy(true);
    setError(null);
    try {
      await api.setPositionRoles(selectedPosition.id, [...next]);
      await reload();
      await refreshPermissions();
      if (attaching) setRoleTabId(roleId);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function togglePositionPerm(role: Role, key: string) {
    if (!selectedPosition || busy || !role.permissions.includes(key)) return;
    const current = new Set(selectedPosition.role_permissions?.[role.id] ?? []);
    if (current.has(key)) current.delete(key);
    else current.add(key);
    if (key.endsWith(".edit") && current.has(key)) {
      current.add(key.replace(/\.edit$/, ".view"));
    }
    const allowed = new Set(role.permissions);
    const next = [...current].filter((k) => allowed.has(k));
    setBusy(true);
    setError(null);
    try {
      await api.setPositionRolePermissions(selectedPosition.id, role.id, next);
      await reload();
      await refreshPermissions();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function renamePosition(name: string) {
    if (!selectedPosition || !name.trim() || busy) return;
    setBusy(true);
    setError(null);
    try {
      await api.updatePosition({
        id: selectedPosition.id,
        name: name.trim(),
        parent_id: selectedPosition.parent_id,
      });
      await reload();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function removePosition() {
    if (!selectedPosition || busy) return;
    setBusy(true);
    setError(null);
    try {
      await api.deletePosition(selectedPosition.id);
      setPositionId(null);
      await reload();
      setStep("positions");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function removeCategory() {
    if (!selectedCategory || busy) return;
    setBusy(true);
    setError(null);
    try {
      await api.deleteOrgCategory(selectedCategory.id);
      setCategoryId(null);
      setPositionId(null);
      await reload();
      setStep("categories");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="panel">
      <h2 style={{ marginTop: 0 }}>Positions</h2>
      <p className="meta" style={{ padding: "0 18px 12px", margin: 0 }}>
        Drill into a category, then a position, to edit its title and permission roles. Positions
        are job titles — attach <Link to="/admin/roles">roles</Link> for access.
      </p>
      {error && <p style={{ color: "var(--danger)", padding: "0 18px" }}>{error}</p>}

      <div className="positions-drill" key={step}>
        {step === "categories" && (
          <>
            <div className="nav-label">Categories</div>
            <div className="positions-tile-grid">
              {categories.map((c) => (
                <button
                  key={c.id}
                  type="button"
                  className="positions-tile"
                  onClick={() => openCategory(c.id)}
                >
                  <span className="teams-dm-dot" style={{ background: c.color }} aria-hidden />
                  <span className="title">{c.name}</span>
                  <span className="meta">
                    {positions.filter((p) => p.category_id === c.id).length} positions
                  </span>
                </button>
              ))}
              {categories.length === 0 && (
                <div className="empty" style={{ gridColumn: "1 / -1" }}>
                  Add a category to begin the org chart
                </div>
              )}
            </div>
            <div className="positions-drill-add">
              <input
                value={newCatName}
                onChange={(e) => setNewCatName(e.target.value)}
                placeholder="New category"
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    void addCategory();
                  }
                }}
              />
              <button
                className="btn btn-primary"
                type="button"
                disabled={busy || !newCatName.trim()}
                onClick={() => void addCategory()}
              >
                Add category
              </button>
            </div>
          </>
        )}

        {step === "positions" && selectedCategory && (
          <>
            <div className="positions-drill-head">
              <button className="btn" type="button" onClick={backToCategories}>
                ← Categories
              </button>
              <div>
                <div className="nav-label" style={{ padding: 0 }}>
                  {selectedCategory.name}
                </div>
                <div className="meta" style={{ padding: 0 }}>
                  Positions in this category
                </div>
              </div>
            </div>
            <div className="positions-tile-grid">
              {tree.map((p) => {
                const depth = positionDepth(positions, p.id);
                return (
                  <button
                    key={p.id}
                    type="button"
                    className="positions-tile"
                    style={{ ["--pos-depth" as string]: depth }}
                    onClick={() => openPosition(p.id)}
                  >
                    <span className="title">
                      {depth > 0 ? `${"· ".repeat(depth)}${p.name}` : p.name}
                    </span>
                    <span className="meta">{p.role_ids.length} roles attached</span>
                  </button>
                );
              })}
              {tree.length === 0 && (
                <div className="empty" style={{ gridColumn: "1 / -1" }}>
                  No positions yet — add one below
                </div>
              )}
            </div>
            <div className="positions-drill-add">
              <input
                value={newPosName}
                onChange={(e) => setNewPosName(e.target.value)}
                placeholder="New position title"
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    void addPosition();
                  }
                }}
              />
              <select value={newPosParent} onChange={(e) => setNewPosParent(e.target.value)}>
                <option value="">Top level (no parent)</option>
                {tree.map((p) => (
                  <option key={p.id} value={p.id}>
                    {"—".repeat(positionDepth(positions, p.id))} {p.name}
                  </option>
                ))}
              </select>
              <button
                className="btn btn-primary"
                type="button"
                disabled={busy || !newPosName.trim()}
                onClick={() => void addPosition()}
              >
                Add position
              </button>
              <button className="btn" type="button" disabled={busy} onClick={() => void removeCategory()}>
                Delete category
              </button>
            </div>
          </>
        )}

        {step === "detail" && selectedPosition && selectedCategory && (
          <>
            <div className="positions-drill-head">
              <button className="btn" type="button" onClick={backToPositions}>
                ← {selectedCategory.name}
              </button>
              <div>
                <div className="nav-label" style={{ padding: 0 }}>
                  Position details
                </div>
                <div className="meta" style={{ padding: 0 }}>
                  {selectedCategory.name} · {selectedPosition.name}
                </div>
              </div>
            </div>
            <div className="form-grid" style={{ gap: 14 }}>
              <div className="form-row">
                <label>Title</label>
                <input
                  key={selectedPosition.id}
                  defaultValue={selectedPosition.name}
                  onBlur={(e) => {
                    if (e.target.value.trim() !== selectedPosition.name) {
                      void renamePosition(e.target.value);
                    }
                  }}
                />
              </div>
              <div className="form-row">
                <label>Attach roles</label>
                <p className="meta" style={{ margin: "0 0 8px", padding: 0 }}>
                  Each attached role becomes a tab. Inside the tab, pick which of that role’s
                  permissions this position actually gets.{" "}
                  <Link to="/admin/roles">Edit role packs</Link>
                </p>
                <div
                  className="member-role-chips"
                  style={{ maxWidth: "100%", justifyContent: "flex-start" }}
                >
                  {roles.map((role) => {
                    const on = selectedPosition.role_ids.includes(role.id);
                    return (
                      <button
                        key={role.id}
                        type="button"
                        className={`role-chip${on ? " active" : ""}`}
                        style={{ ["--role-color" as string]: role.color }}
                        disabled={busy}
                        onClick={() => void toggleRole(role.id)}
                      >
                        {role.name}
                      </button>
                    );
                  })}
                </div>
              </div>
            </div>

            {attachedRoles.length > 0 && activeRole ? (
              <>
                <div className="record-tabs" role="tablist" aria-label="Position role access">
                  {attachedRoles.map((role) => {
                    const granted = selectedPosition.role_permissions?.[role.id] ?? [];
                    return (
                      <button
                        key={role.id}
                        type="button"
                        role="tab"
                        aria-selected={activeRole.id === role.id}
                        className={`record-tab${activeRole.id === role.id ? " active" : ""}`}
                        style={
                          activeRole.id === role.id
                            ? { ["--accent" as string]: role.color, color: role.color }
                            : undefined
                        }
                        onClick={() => setRoleTabId(role.id)}
                      >
                        {role.name}
                        <span className="meta" style={{ marginLeft: 8, padding: 0 }}>
                          {granted.length}/{role.permissions.length}
                        </span>
                      </button>
                    );
                  })}
                </div>
                <p className="meta" style={{ padding: "0 0 12px", margin: 0 }}>
                  Access this position gets from <strong>{activeRole.name}</strong>. Only keys on
                  that role can be selected.
                </p>
                <div className="roles-perm-groups">
                  {MODULE_PERMISSION_GROUPS.map((group) => {
                    const viewKey = moduleViewKey(group.id);
                    const editKey = moduleEditKey(group.id);
                    const granted = selectedPosition.role_permissions?.[activeRole.id] ?? [];
                    const roleHasView = activeRole.permissions.includes(viewKey);
                    const roleHasEdit = activeRole.permissions.includes(editKey);
                    const hasView = granted.includes(viewKey) || granted.includes(editKey);
                    const hasEdit = granted.includes(editKey);
                    const fields = group.fields.filter((field) =>
                      activeRole.permissions.includes(field.key),
                    );
                    if (!roleHasView && !roleHasEdit && fields.length === 0) return null;
                    return (
                      <div key={group.id} className="roles-perm-group">
                        <h3>{group.label}</h3>
                        {roleHasView && (
                          <label className="roles-check">
                            <input
                              type="checkbox"
                              checked={hasView}
                              disabled={busy}
                              onChange={() => void togglePositionPerm(activeRole, viewKey)}
                            />
                            View
                          </label>
                        )}
                        {roleHasEdit && (
                          <label className="roles-check">
                            <input
                              type="checkbox"
                              checked={hasEdit}
                              disabled={busy}
                              onChange={() => void togglePositionPerm(activeRole, editKey)}
                            />
                            Edit
                          </label>
                        )}
                        {fields.map((field) => (
                          <label key={field.key} className="roles-check">
                            <input
                              type="checkbox"
                              checked={granted.includes(field.key)}
                              disabled={busy}
                              onChange={() => void togglePositionPerm(activeRole, field.key)}
                            />
                            {field.label}
                          </label>
                        ))}
                      </div>
                    );
                  })}
                </div>
              </>
            ) : (
              <div className="empty">Attach a role above to choose this position’s access.</div>
            )}

            <div className="form-actions" style={{ marginTop: 16 }}>
              <button className="btn" type="button" disabled={busy} onClick={() => void removePosition()}>
                Delete position
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
