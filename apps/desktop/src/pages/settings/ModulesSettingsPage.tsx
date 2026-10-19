import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import type { WorkspaceModule } from "@confluity/shared";
import {
  CONNECTOR_CATALOG,
  isAlwaysOnModule,
  MODULE_PERMISSION_GROUPS,
  type ModuleId,
} from "@confluity/shared";
import { api } from "../../lib/api";
import { useApp } from "../../context/AppContext";

const MODULE_BLURB: Partial<Record<ModuleId, string>> = {
  dashboard: "Home overview",
  crm: "Companies and contacts",
  support: "Tickets and queue",
  projects: "Issues and boards",
  operations: "Products, assets, RMAs",
  finance: "Orders, invoices, vendors",
  portal: "Customer portal preview",
  docs: "Knowledge base",
  insights: "Live tables and SQL",
  teams: "Department chats",
  search: "Workspace search",
  settings: "Workspace configuration",
};

export function ModulesSettingsPage() {
  const { can, refreshModules, workspaceModules } = useApp();
  const [rows, setRows] = useState<WorkspaceModule[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [adoptMsg, setAdoptMsg] = useState<string | null>(null);

  async function reload() {
    const list = await api.listWorkspaceModules();
    setRows(list);
    await refreshModules();
  }

  useEffect(() => {
    void reload().catch((e) => setError(e instanceof Error ? e.message : String(e)));
  }, []);

  useEffect(() => {
    setRows(workspaceModules);
  }, [workspaceModules]);

  if (!can("settings.workspace_admin")) {
    return <div className="empty">Only workspace admins can change the module pack.</div>;
  }

  async function save(mod: WorkspaceModule, patch: Partial<WorkspaceModule>) {
    setBusy(true);
    setError(null);
    setAdoptMsg(null);
    try {
      await api.setWorkspaceModule({
        module_id: mod.module_id,
        enabled: patch.enabled ?? mod.enabled,
        mode: (patch.mode as string) ?? mod.mode,
        connector_id: patch.connector_id === undefined ? mod.connector_id : patch.connector_id,
      });
      await reload();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function adopt(moduleId: string) {
    if (!confirm(`Adopt ${moduleId} into Confluity? External tools stay as deep links; Confluity becomes source of truth.`)) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await api.adoptModule(moduleId);
      setAdoptMsg(`${moduleId} is now native Confluity.`);
      await reload();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  const connectorsFor = (moduleId: string) =>
    CONNECTOR_CATALOG.filter((c) => c.modules.includes(moduleId));

  return (
    <div className="panel">
      <h2 style={{ marginTop: 0 }}>Modules</h2>
      <p className="meta" style={{ padding: "0 18px 12px", margin: 0 }}>
        Turn product areas on or off for this workspace. Connected modules keep an external tool in
        the loop; Adopt merges synced data and makes Confluity the source of truth.{" "}
        <Link to="/admin/integrations">Manage connectors</Link>.
      </p>
      {error && <p style={{ color: "var(--danger)", padding: "0 18px" }}>{error}</p>}
      {adoptMsg && <p style={{ color: "var(--success)", padding: "0 18px" }}>{adoptMsg}</p>}

      <div className="positions-tile-grid insights-tile-grid" style={{ padding: "0 18px 18px" }}>
        {(rows.length ? rows : []).map((mod) => {
          const id = mod.module_id as ModuleId;
          const label = MODULE_PERMISSION_GROUPS.find((g) => g.id === id)?.label ?? mod.module_id;
          const always = isAlwaysOnModule(mod.module_id);
          const connectors = connectorsFor(mod.module_id);
          return (
            <div key={mod.module_id} className={`positions-tile modules-tile${mod.enabled ? "" : " off"}`}>
              <span className="title">{label}</span>
              <span className="meta">{MODULE_BLURB[id] ?? mod.module_id}</span>
              <label className="modules-toggle">
                <input
                  type="checkbox"
                  checked={mod.enabled}
                  disabled={busy || always}
                  onChange={(e) => void save(mod, { enabled: e.target.checked })}
                />
                {always ? "Always on" : mod.enabled ? "Enabled" : "Off"}
              </label>
              {mod.enabled && (
                <div className="modules-mode">
                  <select
                    value={mod.mode === "connected" ? "connected" : "native"}
                    disabled={busy || connectors.length === 0}
                    onChange={(e) => {
                      const mode = e.target.value;
                      const connector_id =
                        mode === "connected"
                          ? mod.connector_id || connectors[0]?.id || null
                          : null;
                      void save(mod, { mode, connector_id });
                    }}
                  >
                    <option value="native">Native Confluity</option>
                    <option value="connected" disabled={connectors.length === 0}>
                      Connected (external)
                    </option>
                  </select>
                  {mod.mode === "connected" && connectors.length > 0 && (
                    <select
                      value={mod.connector_id ?? connectors[0].id}
                      disabled={busy}
                      onChange={(e) =>
                        void save(mod, { mode: "connected", connector_id: e.target.value })
                      }
                    >
                      {connectors.map((c) => (
                        <option key={c.id} value={c.id}>
                          {c.label}
                        </option>
                      ))}
                    </select>
                  )}
                  {mod.mode === "connected" && (
                    <button
                      className="btn btn-primary"
                      type="button"
                      disabled={busy}
                      onClick={() => void adopt(mod.module_id)}
                    >
                      Adopt into Confluity
                    </button>
                  )}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
