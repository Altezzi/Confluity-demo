import { useCallback, useEffect, useState, type FormEvent } from "react";
import type { McpConnection, McpTransport } from "@confluity/shared";
import { api } from "../../lib/api";
import { useApp } from "../../context/AppContext";
import { ConnectionRow, ListPageHeader, newConnectionId } from "./aiConnectionUi";

function emptyMcpDraft(): Omit<McpConnection, "auth_token_configured"> & { auth_token: string } {
  return {
    id: newConnectionId("mcp"),
    name: "",
    transport: "stdio",
    command: "npx",
    args: ["-y", "@modelcontextprotocol/server-filesystem", "."],
    url: "",
    auth_token: "",
    enabled: true,
  };
}

export function McpConnectionsSettingsPage() {
  const { can } = useApp();
  const [connections, setConnections] = useState<McpConnection[]>([]);
  const [formOpen, setFormOpen] = useState(false);
  const [draft, setDraft] = useState(emptyMcpDraft());
  const [editingId, setEditingId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [savedFlash, setSavedFlash] = useState<string | null>(null);

  const load = useCallback(async () => {
    setConnections(await api.listMcpConnections());
  }, []);

  useEffect(() => {
    if (!can("settings.ai")) return;
    void load().catch((e) => setError(e instanceof Error ? e.message : String(e)));
  }, [can, load]);

  if (!can("settings.ai")) {
    return <div className="empty">You do not have permission to manage MCP connections.</div>;
  }

  function flash(msg: string) {
    setSavedFlash(msg);
    setTimeout(() => setSavedFlash(null), 2000);
  }

  function openAdd() {
    setError(null);
    setEditingId(null);
    setDraft(emptyMcpDraft());
    setFormOpen(true);
  }

  function openEdit(conn: McpConnection) {
    setError(null);
    setEditingId(conn.id);
    setDraft({
      id: conn.id,
      name: conn.name,
      transport: conn.transport,
      command: conn.command ?? "",
      args: conn.args.length ? conn.args : [""],
      url: conn.url ?? "",
      auth_token: "",
      enabled: conn.enabled,
    });
    setFormOpen(true);
  }

  function closeForm() {
    setFormOpen(false);
    setEditingId(null);
    setDraft(emptyMcpDraft());
  }

  async function saveList(next: McpConnection[]) {
    const saved = await api.saveMcpConnections(
      next.map((c) => ({
        id: c.id,
        name: c.name,
        transport: c.transport,
        command: c.command,
        args: c.args,
        url: c.url,
        enabled: c.enabled,
      })),
    );
    setConnections(saved);
  }

  async function save(e: FormEvent) {
    e.preventDefault();
    setError(null);
    const name = draft.name.trim();
    if (!name) {
      setError("Connection name is required.");
      return;
    }
    if (draft.transport === "stdio" && !draft.command?.trim()) {
      setError("Command is required for stdio MCP servers.");
      return;
    }
    if (draft.transport === "sse" && !draft.url?.trim()) {
      setError("URL is required for remote MCP (SSE) servers.");
      return;
    }

    try {
      const args = draft.args.map((a) => a.trim()).filter(Boolean);
      const entry = {
        id: draft.id,
        name,
        transport: draft.transport,
        command: draft.transport === "stdio" ? draft.command?.trim() || null : null,
        args: draft.transport === "stdio" ? args : [],
        url: draft.transport === "sse" ? draft.url?.trim() || null : null,
        enabled: draft.enabled,
        auth_token: draft.auth_token.trim() || undefined,
      };

      const others = connections
        .filter((c) => c.id !== entry.id)
        .map((c) => ({
          id: c.id,
          name: c.name,
          transport: c.transport,
          command: c.command,
          args: c.args,
          url: c.url,
          enabled: c.enabled,
        }));

      await api.saveMcpConnections([...others, entry]);
      await load();
      closeForm();
      flash("MCP connection saved.");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function remove(id: string) {
    if (!confirm("Remove this MCP connection?")) return;
    setError(null);
    try {
      await saveList(connections.filter((c) => c.id !== id));
      if (editingId === id) closeForm();
      flash("MCP connection removed.");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function toggle(conn: McpConnection) {
    setError(null);
    try {
      await saveList(connections.map((c) => (c.id === conn.id ? { ...c, enabled: !c.enabled } : c)));
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  return (
    <div className="panel">
      <ListPageHeader
        title="MCP connections"
        onAdd={openAdd}
        addLabel="Add MCP connection"
        backTo="/admin/ai"
        backLabel="AI Connections"
      />
      <p className="meta" style={{ padding: "0 18px 12px", margin: 0 }}>
        Stdio commands or remote SSE endpoints for Model Context Protocol servers.
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
            No MCP connections yet. Use + to add one.
          </li>
        )}
        {connections.map((conn) => (
          <ConnectionRow
            key={conn.id}
            title={conn.name}
            subtitle={
              conn.transport === "stdio"
                ? `${conn.command ?? ""} ${conn.args.join(" ")}`.trim()
                : (conn.url ?? "")
            }
            meta={`${conn.enabled ? "Enabled" : "Disabled"}${conn.auth_token_configured ? " · auth configured" : ""}`}
            actions={
              <>
                <button className="btn" type="button" onClick={() => openEdit(conn)}>
                  Edit
                </button>
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
              placeholder="e.g. Filesystem tools"
            />
          </div>
          <div className="form-row">
            <label>Transport</label>
            <select
              value={draft.transport}
              onChange={(e) => setDraft({ ...draft, transport: e.target.value as McpTransport })}
            >
              <option value="stdio">Stdio (local command)</option>
              <option value="sse">Remote (SSE URL)</option>
            </select>
          </div>
          {draft.transport === "stdio" ? (
            <>
              <div className="form-row">
                <label>Command</label>
                <input
                  value={draft.command ?? ""}
                  onChange={(e) => setDraft({ ...draft, command: e.target.value })}
                />
              </div>
              <div className="form-row">
                <label>Arguments (one per line)</label>
                <textarea
                  value={draft.args.join("\n")}
                  onChange={(e) => setDraft({ ...draft, args: e.target.value.split("\n") })}
                  rows={4}
                  style={{ fontFamily: "var(--mono)", fontSize: "0.85rem" }}
                />
              </div>
            </>
          ) : (
            <>
              <div className="form-row">
                <label>MCP URL</label>
                <input
                  value={draft.url ?? ""}
                  onChange={(e) => setDraft({ ...draft, url: e.target.value })}
                  placeholder="https://…"
                />
              </div>
              <div className="form-row">
                <label>Bearer token (optional)</label>
                <input
                  type="password"
                  value={draft.auth_token}
                  onChange={(e) => setDraft({ ...draft, auth_token: e.target.value })}
                  placeholder={
                    editingId && connections.find((c) => c.id === editingId)?.auth_token_configured
                      ? "Leave blank to keep existing token"
                      : "Optional"
                  }
                  autoComplete="off"
                />
              </div>
            </>
          )}
          <label className="meta" style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <input
              type="checkbox"
              checked={draft.enabled}
              onChange={(e) => setDraft({ ...draft, enabled: e.target.checked })}
            />
            Enabled
          </label>
          <div className="form-actions">
            <button className="btn btn-primary" type="submit">
              {editingId ? "Save changes" : "Add connection"}
            </button>
            <button className="btn" type="button" onClick={closeForm}>
              Cancel
            </button>
          </div>
        </form>
      )}
    </div>
  );
}
