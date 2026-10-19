import { useCallback, useEffect, useState, type FormEvent } from "react";
import type { ApiConnection } from "@confluity/shared";
import { api } from "../../lib/api";
import { useApp } from "../../context/AppContext";
import { ConnectionRow, ListPageHeader, newConnectionId } from "./aiConnectionUi";

function emptyApiDraft(): Omit<ApiConnection, "api_key_configured"> & { api_key: string } {
  return {
    id: newConnectionId("api"),
    name: "",
    base_url: "https://api.openai.com/v1",
    model: "gpt-4o-mini",
    api_key: "",
    enabled: true,
    is_default: false,
  };
}

export function ApiConnectionsSettingsPage() {
  const { refreshSettings, can } = useApp();
  const [connections, setConnections] = useState<ApiConnection[]>([]);
  const [formOpen, setFormOpen] = useState(false);
  const [draft, setDraft] = useState(emptyApiDraft());
  const [editingId, setEditingId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [savedFlash, setSavedFlash] = useState<string | null>(null);

  const load = useCallback(async () => {
    setConnections(await api.listApiConnections());
  }, []);

  useEffect(() => {
    if (!can("settings.ai")) return;
    void load().catch((e) => setError(e instanceof Error ? e.message : String(e)));
  }, [can, load]);

  if (!can("settings.ai")) {
    return <div className="empty">You do not have permission to manage API connections.</div>;
  }

  function flash(msg: string) {
    setSavedFlash(msg);
    setTimeout(() => setSavedFlash(null), 2000);
  }

  function openAdd() {
    setError(null);
    setEditingId(null);
    setDraft({ ...emptyApiDraft(), is_default: connections.length === 0 });
    setFormOpen(true);
  }

  function openEdit(conn: ApiConnection) {
    setError(null);
    setEditingId(conn.id);
    setDraft({
      id: conn.id,
      name: conn.name,
      base_url: conn.base_url,
      model: conn.model,
      api_key: "",
      enabled: conn.enabled,
      is_default: conn.is_default,
    });
    setFormOpen(true);
  }

  function closeForm() {
    setFormOpen(false);
    setEditingId(null);
    setDraft(emptyApiDraft());
  }

  async function persist(
    next: Array<
      Omit<ApiConnection, "api_key_configured"> & { api_key?: string | null; clear_api_key?: boolean }
    >,
  ) {
    const saved = await api.saveApiConnections(next);
    setConnections(saved);
    await refreshSettings();
  }

  async function save(e: FormEvent, opts?: { clearApiKey?: boolean }) {
    e.preventDefault();
    setError(null);
    const name = draft.name.trim();
    if (!name) {
      setError("Connection name is required.");
      return;
    }
    if (!draft.base_url.trim() || !draft.model.trim()) {
      setError("Base URL and model are required.");
      return;
    }

    try {
      const entry = {
        id: draft.id,
        name,
        base_url: draft.base_url.trim(),
        model: draft.model.trim(),
        enabled: draft.enabled,
        is_default: draft.is_default || connections.length === 0,
        api_key: draft.api_key.trim() || undefined,
        clear_api_key: opts?.clearApiKey,
      };

      const others = connections
        .filter((c) => c.id !== entry.id)
        .map((c) => ({
          id: c.id,
          name: c.name,
          base_url: c.base_url,
          model: c.model,
          enabled: c.enabled,
          is_default: entry.is_default ? false : c.is_default,
        }));

      await persist([...others, entry]);
      await load();
      closeForm();
      flash("API connection saved.");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function remove(id: string) {
    if (!confirm("Remove this API connection?")) return;
    setError(null);
    try {
      await persist(
        connections
          .filter((c) => c.id !== id)
          .map((c) => ({
            id: c.id,
            name: c.name,
            base_url: c.base_url,
            model: c.model,
            enabled: c.enabled,
            is_default: c.is_default,
          })),
      );
      if (editingId === id) closeForm();
      flash("API connection removed.");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function toggle(conn: ApiConnection) {
    setError(null);
    try {
      await persist(
        connections.map((c) =>
          c.id === conn.id
            ? {
                id: c.id,
                name: c.name,
                base_url: c.base_url,
                model: c.model,
                enabled: !c.enabled,
                is_default: c.is_default,
              }
            : {
                id: c.id,
                name: c.name,
                base_url: c.base_url,
                model: c.model,
                enabled: c.enabled,
                is_default: c.is_default,
              },
        ),
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function setDefault(conn: ApiConnection) {
    setError(null);
    try {
      await persist(
        connections.map((c) => ({
          id: c.id,
          name: c.name,
          base_url: c.base_url,
          model: c.model,
          enabled: c.enabled,
          is_default: c.id === conn.id,
        })),
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  return (
    <div className="panel">
      <ListPageHeader
        title="API connections"
        onAdd={openAdd}
        addLabel="Add API connection"
        backTo="/admin/ai"
        backLabel="AI Connections"
      />
      <p className="meta" style={{ padding: "0 18px 12px", margin: 0 }}>
        OpenAI-compatible providers exposing <code>/chat/completions</code>.
      </p>
      {error && <p style={{ color: "var(--danger)", padding: "0 18px" }}>{error}</p>}
      {savedFlash && (
        <p className="meta" style={{ padding: "0 18px", color: "var(--success)" }}>
          {savedFlash}
        </p>
      )}

      <ul style={{ margin: 0, padding: "0 18px 12px" }}>
        {connections.length === 0 && (
          <li className="empty" style={{ listStyle: "none", padding: "8px 0" }}>
            No API connections yet. Use + to add one.
          </li>
        )}
        {connections.map((conn) => (
          <ConnectionRow
            key={conn.id}
            title={conn.name}
            subtitle={`${conn.model} · ${conn.base_url}`}
            meta={`${conn.enabled ? "Enabled" : "Disabled"}${conn.api_key_configured ? " · key on file" : ""}${conn.is_default ? " · default for companion" : ""}`}
            actions={
              <>
                <button className="btn" type="button" onClick={() => openEdit(conn)}>
                  Edit
                </button>
                {!conn.is_default && (
                  <button className="btn" type="button" onClick={() => void setDefault(conn)}>
                    Set default
                  </button>
                )}
                <button className="btn" type="button" onClick={() => void toggle(conn)}>
                  {conn.enabled ? "Disable" : "Enable"}
                </button>
                <button className="btn" type="button" onClick={() => void remove(conn.id)}>
                  Remove
                </button>
              </>
            }
          />
        ))}
      </ul>

      {formOpen && (
        <form className="form-grid settings-block" style={{ padding: "0 18px 24px" }} onSubmit={save}>
          <div className="form-row">
            <label>{editingId ? "Edit connection" : "New connection"}</label>
            <input
              value={draft.name}
              onChange={(e) => setDraft({ ...draft, name: e.target.value })}
              placeholder="e.g. OpenAI production"
            />
          </div>
          <div className="form-row">
            <label>API base URL</label>
            <input
              value={draft.base_url}
              onChange={(e) => setDraft({ ...draft, base_url: e.target.value })}
            />
          </div>
          <div className="form-row">
            <label>Model</label>
            <input value={draft.model} onChange={(e) => setDraft({ ...draft, model: e.target.value })} />
          </div>
          <div className="form-row">
            <label>API key</label>
            <input
              type="password"
              value={draft.api_key}
              onChange={(e) => setDraft({ ...draft, api_key: e.target.value })}
              placeholder={
                editingId && connections.find((c) => c.id === editingId)?.api_key_configured
                  ? "Leave blank to keep existing key"
                  : "sk-… or provider key"
              }
              autoComplete="off"
            />
          </div>
          <label className="meta" style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <input
              type="checkbox"
              checked={draft.enabled}
              onChange={(e) => setDraft({ ...draft, enabled: e.target.checked })}
            />
            Enabled
          </label>
          <label className="meta" style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <input
              type="checkbox"
              checked={draft.is_default}
              onChange={(e) => setDraft({ ...draft, is_default: e.target.checked })}
            />
            Default for AI companion
          </label>
          <div className="form-actions">
            <button className="btn btn-primary" type="submit">
              {editingId ? "Save changes" : "Add connection"}
            </button>
            <button className="btn" type="button" onClick={closeForm}>
              Cancel
            </button>
            {editingId && connections.find((c) => c.id === editingId)?.api_key_configured && (
              <button className="btn" type="button" onClick={(ev) => void save(ev, { clearApiKey: true })}>
                Remove stored key
              </button>
            )}
          </div>
        </form>
      )}
    </div>
  );
}
