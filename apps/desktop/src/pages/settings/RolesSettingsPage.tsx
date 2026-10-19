import { useEffect, useMemo, useState, type ReactNode } from "react";
import { Link } from "react-router-dom";
import type { ModuleId, Role } from "@confluity/shared";
import { MODULE_PERMISSION_GROUPS, moduleEditKey, moduleViewKey } from "@confluity/shared";
import { api } from "../../lib/api";
import { useApp } from "../../context/AppContext";

function ModuleTabIcon({ id }: { id: ModuleId }) {
  const common = {
    className: "roles-perm-tab-icon",
    viewBox: "0 0 24 24",
    fill: "none",
    stroke: "currentColor",
    strokeWidth: 1.75,
  } as const;
  const icons: Record<ModuleId, ReactNode> = {
    dashboard: (
      <svg {...common}>
        <path d="M4 10.5 12 4l8 6.5V20a1 1 0 0 1-1 1h-5v-6H10v6H5a1 1 0 0 1-1-1v-9.5Z" strokeLinejoin="round" />
      </svg>
    ),
    crm: (
      <svg {...common}>
        <circle cx="9" cy="8" r="3" />
        <path d="M3.5 19c.8-3 2.9-4.5 5.5-4.5s4.7 1.5 5.5 4.5" strokeLinecap="round" />
        <circle cx="16.5" cy="9.5" r="2.2" />
        <path d="M14.2 19c.3-1.6 1.4-2.7 3.1-2.7 1.2 0 2.2.5 2.8 1.5" strokeLinecap="round" />
      </svg>
    ),
    support: (
      <svg {...common}>
        <path d="M7 8h10M7 12h6" strokeLinecap="round" />
        <path d="M6 4h12a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2h-5l-4 3v-3H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2Z" strokeLinejoin="round" />
      </svg>
    ),
    projects: (
      <svg {...common}>
        <path d="M4 7h16M4 12h16M4 17h10" strokeLinecap="round" />
      </svg>
    ),
    operations: (
      <svg {...common}>
        <rect x="4" y="4" width="7" height="7" rx="1.2" />
        <rect x="13" y="4" width="7" height="7" rx="1.2" />
        <rect x="4" y="13" width="7" height="7" rx="1.2" />
        <path d="M14.5 16.5h4M16.5 14.5v4" strokeLinecap="round" />
      </svg>
    ),
    finance: (
      <svg {...common}>
        <path d="M4 19V6.5A1.5 1.5 0 0 1 5.5 5H19" strokeLinecap="round" />
        <path d="M7 16h3M7 12h6M7 8h10" strokeLinecap="round" />
      </svg>
    ),
    portal: (
      <svg {...common}>
        <circle cx="12" cy="8" r="3" />
        <path d="M5 19c1.2-3.2 3.6-5 7-5s5.8 1.8 7 5" strokeLinecap="round" />
      </svg>
    ),
    docs: (
      <svg {...common}>
        <path d="M7 4h7l4 4v12a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1Z" strokeLinejoin="round" />
        <path d="M14 4v4h4" strokeLinejoin="round" />
      </svg>
    ),
    insights: (
      <svg {...common}>
        <path d="M4 19V6.5" strokeLinecap="round" />
        <rect x="7" y="12" width="3" height="7" rx="0.6" />
        <rect x="12" y="8" width="3" height="11" rx="0.6" />
        <rect x="17" y="4.5" width="3" height="14.5" rx="0.6" />
      </svg>
    ),
    teams: (
      <svg {...common}>
        <circle cx="9" cy="9" r="2.5" />
        <circle cx="16" cy="10" r="2.2" />
        <path d="M4.5 18c.7-2.4 2.4-3.6 4.5-3.6s3.8 1.2 4.5 3.6" strokeLinecap="round" />
        <path d="M13.2 18c.4-1.6 1.5-2.5 3-2.5 1.1 0 2 .4 2.6 1.2" strokeLinecap="round" />
      </svg>
    ),
    search: (
      <svg {...common}>
        <circle cx="11" cy="11" r="6.25" />
        <path d="m16 16 3.5 3.5" strokeLinecap="round" />
      </svg>
    ),
    settings: (
      <svg {...common}>
        <circle cx="12" cy="12" r="3" />
        <path d="M12 4v1.5M12 18.5V20M4 12h1.5M18.5 12H20M6.2 6.2l1.1 1.1M16.7 16.7l1.1 1.1M17.8 6.2l-1.1 1.1M7.3 16.7l-1.1 1.1" strokeLinecap="round" />
      </svg>
    ),
  };
  return icons[id];
}

function groupHasGrant(role: Role, groupId: ModuleId): boolean {
  const viewKey = moduleViewKey(groupId);
  const editKey = moduleEditKey(groupId);
  const group = MODULE_PERMISSION_GROUPS.find((g) => g.id === groupId);
  if (role.permissions.includes(viewKey) || role.permissions.includes(editKey)) return true;
  return Boolean(group?.fields.some((f) => role.permissions.includes(f.key)));
}

export function RolesSettingsPage({ embedded = false }: { embedded?: boolean }) {
  const { can, refreshPermissions, moduleEnabled } = useApp();
  const [roles, setRoles] = useState<Role[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [draftName, setDraftName] = useState("");
  const [newName, setNewName] = useState("");
  const [newColor, setNewColor] = useState("#6B7280");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [permTab, setPermTab] = useState<ModuleId>(MODULE_PERMISSION_GROUPS[0].id);

  const selected = useMemo(
    () => roles.find((r) => r.id === selectedId) ?? null,
    [roles, selectedId],
  );
  const activePermGroup =
    MODULE_PERMISSION_GROUPS.find((g) => g.id === permTab) ?? MODULE_PERMISSION_GROUPS[0];

  async function reload(preferId?: string | null) {
    const rows = await api.listRoles();
    setRoles(rows);
    setSelectedId((curr) => {
      const want = preferId ?? curr;
      if (want && rows.some((r) => r.id === want)) return want;
      const firstEditable = rows.find((r) => !r.is_system);
      return firstEditable?.id ?? rows[0]?.id ?? null;
    });
  }

  useEffect(() => {
    void reload().catch((e) => setError(e instanceof Error ? e.message : String(e)));
  }, []);

  useEffect(() => {
    if (selected) setDraftName(selected.name);
    else setDraftName("");
  }, [selected?.id, selected?.name]);

  if (!embedded && !can("settings.manage_roles")) {
    return <div className="empty">You do not have permission to manage Roles.</div>;
  }

  async function createRole() {
    if (!newName.trim() || busy) return;
    setBusy(true);
    setError(null);
    try {
      const created = await api.createRole({
        name: newName.trim(),
        color: newColor,
        permissions: [],
      });
      setNewName("");
      await reload(created.id);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function saveMeta(role: Role, patch: Partial<Pick<Role, "name" | "color" | "position">>) {
    setBusy(true);
    setError(null);
    try {
      await api.updateRole({
        id: role.id,
        name: patch.name ?? role.name,
        color: patch.color ?? role.color,
        position: patch.position ?? role.position,
      });
      await reload(role.id);
      await refreshPermissions();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function saveName() {
    if (!selected || selected.is_system || busy) return;
    const name = draftName.trim();
    if (!name || name === selected.name) return;
    await saveMeta(selected, { name });
  }

  async function togglePerm(role: Role, key: string) {
    if (role.is_system) return;
    const granted = new Set(role.permissions);
    if (granted.has(key)) granted.delete(key);
    else granted.add(key);
    if (key.endsWith(".edit") && granted.has(key)) {
      granted.add(key.replace(/\.edit$/, ".view"));
    }
    setBusy(true);
    setError(null);
    try {
      await api.setRolePermissions(role.id, [...granted]);
      await reload(role.id);
      await refreshPermissions();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function move(role: Role, direction: -1 | 1) {
    const idx = roles.findIndex((r) => r.id === role.id);
    const swap = roles[idx + direction];
    if (!swap) return;
    setBusy(true);
    try {
      await api.reorderRoles(role.id, swap.id);
      await reload(role.id);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function resetDefaults() {
    if (busy) return;
    if (!confirm("Are you sure? This restores the default roles and permissions.")) return;
    setBusy(true);
    setError(null);
    try {
      await api.resetDefaultRoles();
      await reload(null);
      await refreshPermissions();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function remove(role: Role) {
    if (role.is_system || busy) return;
    if (!confirm(`Delete role “${role.name}”? Members keep their positions; remove this role from positions first if needed.`))
      return;
    setBusy(true);
    try {
      await api.deleteRole(role.id);
      await reload(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className={`grid-2 roles-layout${embedded ? " setup-roles-editor" : ""}`}>
      <div className="panel">
        <div className="panel-head">
          <h2 style={{ margin: 0 }}>Roles</h2>
          <button
            type="button"
            className="btn btn-icon"
            title="Reset to default"
            aria-label="Reset to default"
            disabled={busy}
            onClick={() => void resetDefaults()}
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden>
              <path
                d="M21 12a9 9 0 1 1-2.64-6.36"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
              />
              <path
                d="M21 3v6h-6"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          </button>
        </div>
        <p className="meta" style={{ padding: "0 18px 8px", margin: 0 }}>
          {embedded
            ? "Add your own roles, rename them, and choose what each one can do."
            : (
              <>
                Permission packs — what people can see and do. Attach roles to{" "}
                <Link to="/admin/positions">positions</Link> so each job title gets the right access.
              </>
            )}
        </p>

        <div className="form-grid" style={{ padding: "0 18px 12px", gap: 8 }}>
          <div className="nav-label">Add role</div>
          <div className="form-row">
            <label>Name</label>
            <input
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="e.g. Finance, Contractor, Lead"
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  void createRole();
                }
              }}
            />
          </div>
          <div className="form-row">
            <label>Color</label>
            <input type="color" value={newColor} onChange={(e) => setNewColor(e.target.value)} />
          </div>
          <button
            className="btn btn-primary"
            type="button"
            disabled={busy || !newName.trim()}
            onClick={() => void createRole()}
          >
            Create role
          </button>
        </div>

        <div className="list">
          {roles.map((role, index) => (
            <button
              key={role.id}
              type="button"
              className={`list-item${selected?.id === role.id ? " active" : ""}`}
              style={{ width: "100%", textAlign: "left" }}
              onClick={() => setSelectedId(role.id)}
            >
              <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
                <span
                  aria-hidden
                  style={{
                    width: 12,
                    height: 12,
                    borderRadius: 3,
                    background: role.color,
                    flexShrink: 0,
                  }}
                />
                <div>
                  <div className="title">
                    {role.name}
                    {role.is_system ? " · system" : ""}
                  </div>
                  <div className="meta">{role.permissions.length} permissions</div>
                </div>
              </div>
              <div style={{ display: "flex", gap: 4 }} onClick={(e) => e.stopPropagation()}>
                <button
                  className="btn"
                  type="button"
                  disabled={busy || index === 0}
                  onClick={() => void move(role, -1)}
                  title="Move up"
                >
                  ↑
                </button>
                <button
                  className="btn"
                  type="button"
                  disabled={busy || index === roles.length - 1}
                  onClick={() => void move(role, 1)}
                  title="Move down"
                >
                  ↓
                </button>
              </div>
            </button>
          ))}
        </div>
        {error && <p style={{ color: "var(--danger)", padding: 12 }}>{error}</p>}
      </div>

      <div className="panel">
        {selected ? (
          <>
            <div className="panel-head" style={{ alignItems: "center" }}>
              <h2 style={{ margin: 0 }}>Edit {selected.name}</h2>
              {!selected.is_system && (
                <button className="btn" type="button" disabled={busy} onClick={() => void remove(selected)}>
                  Delete
                </button>
              )}
            </div>
            <div className="form-grid">
              <div className="form-row">
                <label>Name</label>
                <div style={{ display: "flex", gap: 8 }}>
                  <input
                    value={draftName}
                    disabled={selected.is_system || busy}
                    onChange={(e) => setDraftName(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        e.preventDefault();
                        void saveName();
                      }
                    }}
                  />
                  {!selected.is_system && (
                    <button
                      className="btn"
                      type="button"
                      disabled={busy || !draftName.trim() || draftName.trim() === selected.name}
                      onClick={() => void saveName()}
                    >
                      Save name
                    </button>
                  )}
                </div>
              </div>
              <div className="form-row">
                <label>Color</label>
                <input
                  type="color"
                  value={selected.color}
                  disabled={busy}
                  onChange={(e) => void saveMeta(selected, { color: e.target.value })}
                />
              </div>
            </div>

            <div className="nav-label" style={{ padding: "8px 18px 0" }}>
              Permissions
            </div>
            {selected.is_system ? (
              <p className="meta" style={{ padding: 12 }}>
                System Admin always has every permission and cannot be edited. Create a custom role
                or select Support / Management / Engineering / Viewer to change access.
              </p>
            ) : (
              <>
                <div className="roles-perm-tabs" role="tablist" aria-label="Permission sections">
                  {MODULE_PERMISSION_GROUPS.map((group) => {
                    const active = permTab === group.id;
                    const granted = groupHasGrant(selected, group.id);
                    const enabled = moduleEnabled(group.id);
                    return (
                      <button
                        key={group.id}
                        type="button"
                        role="tab"
                        aria-selected={active}
                        title={enabled ? group.label : `${group.label} (module off)`}
                        className={`roles-perm-tab${active ? " active" : ""}${granted ? " granted" : ""}${enabled ? "" : " muted"}`}
                        onClick={() => setPermTab(group.id)}
                      >
                        <ModuleTabIcon id={group.id} />
                        <span className="roles-perm-tab-label">{group.label}</span>
                      </button>
                    );
                  })}
                </div>
                <div className="roles-perm-groups">
                  <div className="roles-perm-group solo">
                    <h3>
                      {activePermGroup.label}
                      {!moduleEnabled(activePermGroup.id) && (
                        <span className="meta" style={{ marginLeft: 8, fontWeight: 400 }}>
                          Module off in workspace pack —{" "}
                          <Link to="/admin/modules">enable in Modules</Link>
                        </span>
                      )}
                    </h3>
                    <label className="roles-check">
                      <input
                        type="checkbox"
                        checked={
                          selected.permissions.includes(moduleViewKey(activePermGroup.id)) ||
                          selected.permissions.includes(moduleEditKey(activePermGroup.id))
                        }
                        disabled={busy}
                        onChange={() => void togglePerm(selected, moduleViewKey(activePermGroup.id))}
                      />
                      View
                    </label>
                    <label className="roles-check">
                      <input
                        type="checkbox"
                        checked={selected.permissions.includes(moduleEditKey(activePermGroup.id))}
                        disabled={busy}
                        onChange={() => void togglePerm(selected, moduleEditKey(activePermGroup.id))}
                      />
                      Edit
                    </label>
                    {activePermGroup.fields.map((field) => (
                      <label key={field.key} className="roles-check">
                        <input
                          type="checkbox"
                          checked={selected.permissions.includes(field.key)}
                          disabled={busy}
                          onChange={() => void togglePerm(selected, field.key)}
                        />
                        {field.label}
                      </label>
                    ))}
                  </div>
                </div>
              </>
            )}
          </>
        ) : (
          <div className="empty">Select a role to edit, or create one on the left.</div>
        )}
      </div>
    </div>
  );
}
