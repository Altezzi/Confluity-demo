import { useEffect, useState, type FormEvent } from "react";
import { Link } from "react-router-dom";
import { BackLink } from "../../components/BackLink";
import type {
  CloudIdentitySettings,
  IdentitySession,
  StorageMode,
  SyncStatus,
  WorkspacePreview,
} from "@confluity/shared";
import { api } from "../../lib/api";
import { useApp } from "../../context/AppContext";

const MODES: {
  id: StorageMode;
  title: string;
  detail: string;
  to: string;
}[] = [
  {
    id: "local",
    title: "Local only",
    detail: "SQLite on this machine. No account required.",
    to: "/admin/data/local",
  },
  {
    id: "server",
    title: "Confluity Server",
    detail: "Organization endpoint. Local data stays cached.",
    to: "/admin/data/server",
  },
  {
    id: "hybrid",
    title: "Hybrid sync",
    detail: "Local outbox plus Sync now against a server URL.",
    to: "/admin/data/hybrid",
  },
];

function modeSummary(mode: StorageMode, active: StorageMode | undefined, sync: SyncStatus | null): string {
  if (active !== mode) return mode === "local" ? "Not selected" : "Not connected";
  if (mode === "local") return "Active";
  const status = sync?.server_status ?? "disconnected";
  if (status === "connected") return "Connected";
  return status.charAt(0).toUpperCase() + status.slice(1);
}

function cloudSummary(identity: CloudIdentitySettings | null, session: IdentitySession | null): string {
  if (!identity || identity.provider === "none") return "Not connected";
  if (identity.provider === "supabase") {
    return session?.signed_in && session.email ? `Supabase · ${session.email}` : "Supabase";
  }
  if (identity.provider === "custom_api") return "Custom API";
  return "Not connected";
}

export function DataServerSettingsPage() {
  const { settings, can } = useApp();
  const [sync, setSync] = useState<SyncStatus | null>(null);
  const [identity, setIdentity] = useState<CloudIdentitySettings | null>(null);
  const [session, setSession] = useState<IdentitySession | null>(null);

  useEffect(() => {
    if (!can("settings.data_server")) return;
    void Promise.all([
      api.getSyncStatus(),
      api.getCloudIdentitySettings().catch(() => null),
      api.getIdentitySession().catch(() => null),
    ]).then(([nextSync, nextIdentity, nextSession]) => {
      setSync(nextSync);
      setIdentity(nextIdentity);
      setSession(nextSession);
    });
  }, [can]);

  if (!can("settings.data_server")) {
    return <div className="empty">You do not have permission to manage Data &amp; Server.</div>;
  }

  return (
    <div className="panel">
      <h2 style={{ marginTop: 0 }}>Data &amp; Server</h2>
      <p className="meta" style={{ padding: "0 18px 12px", margin: 0 }}>
        Choose a storage or sign-in type to set it up.
      </p>
      <div className="positions-tile-grid insights-tile-grid" style={{ padding: "0 18px 24px" }}>
        {MODES.map((mode) => (
          <Link key={mode.id} to={mode.to} className="positions-tile" style={{ textDecoration: "none", color: "inherit" }}>
            <span className="title">{mode.title}</span>
            <span className="meta">{mode.detail}</span>
            <span className="meta">{modeSummary(mode.id, settings?.storage_mode, sync)}</span>
          </Link>
        ))}
        <Link to="/admin/data/identity" className="positions-tile" style={{ textDecoration: "none", color: "inherit" }}>
          <span className="title">Cloud login &amp; sync</span>
          <span className="meta">Supabase or a custom API for org sign-in and member sync</span>
          <span className="meta">{cloudSummary(identity, session)}</span>
        </Link>
      </div>
    </div>
  );
}

const MODE_COPY: Record<StorageMode, { title: string; body: string }> = {
  local: {
    title: "Local only",
    body: "SQLite on this machine. No account required.",
  },
  server: {
    title: "Confluity Server",
    body: "Point this workspace at an organization endpoint. Local data stays cached.",
  },
  hybrid: {
    title: "Hybrid sync",
    body: "Keep a local outbox and sync it against a connected server URL.",
  },
};

export function DataModeSettingsPage({ mode }: { mode: StorageMode }) {
  const { settings, refreshSettings, can } = useApp();
  const copy = MODE_COPY[mode];
  const [dataDir, setDataDir] = useState("");
  const [modeError, setModeError] = useState<string | null>(null);
  const [serverUrl, setServerUrl] = useState(settings?.server_url ?? "");
  const [sync, setSync] = useState<SyncStatus | null>(null);
  const [preview, setPreview] = useState<WorkspacePreview | null>(null);
  const active = settings?.storage_mode === mode;

  useEffect(() => {
    if (mode !== "local") return;
    void api.getDataDir().then(setDataDir);
    void api.workspacePreview().then(setPreview).catch(() => setPreview(null));
  }, [mode]);

  useEffect(() => {
    if (mode === "local") return;
    void api.getSyncStatus().then(setSync);
  }, [mode]);

  useEffect(() => {
    if (settings) setServerUrl(settings.server_url ?? "");
  }, [settings]);

  if (!can("settings.data_server")) {
    return <div className="empty">You do not have permission to manage Data &amp; Server.</div>;
  }

  async function selectMode() {
    setModeError(null);
    try {
      await api.updateStorageMode(mode);
      await refreshSettings();
      if (mode !== "local") setSync(await api.getSyncStatus());
    } catch (e) {
      setModeError(e instanceof Error ? e.message : String(e));
    }
  }

  async function connect(e: FormEvent) {
    e.preventDefault();
    setModeError(null);
    try {
      const next = await api.connectServer(serverUrl, mode);
      setSync(next);
      await refreshSettings();
    } catch (err) {
      setModeError(err instanceof Error ? err.message : String(err));
    }
  }

  return (
    <div className="panel">
      <BackLink to="/admin/data">Data & Server</BackLink>
      <h2 style={{ marginTop: 0, padding: "0 18px" }}>{copy.title}</h2>
      <p className="meta" style={{ padding: "0 18px 12px", margin: 0 }}>
        {copy.body}
      </p>
      {modeError && <p style={{ color: "var(--danger)", padding: "0 18px" }}>{modeError}</p>}

      <div style={{ padding: "0 18px 18px", display: "flex", gap: 8, alignItems: "center" }}>
        <button className="btn btn-primary" type="button" disabled={active} onClick={() => void selectMode()}>
          {active ? "In use" : "Use this mode"}
        </button>
        {active && <span className="badge open">Active</span>}
      </div>

      {mode !== "local" && (
        <>
          <form className="form-grid" onSubmit={connect} style={{ padding: "0 18px 8px" }}>
            <div className="form-row">
              <label>Server URL</label>
              <input
                value={serverUrl}
                onChange={(e) => setServerUrl(e.target.value)}
                placeholder="https://confluity.example.com"
              />
            </div>
            <button className="btn btn-primary" type="submit">
              Connect
            </button>
            <button
              className="btn"
              type="button"
              onClick={() => void api.disconnectServer().then(setSync).then(() => refreshSettings())}
            >
              Disconnect
            </button>
          </form>
          <p className="meta" style={{ padding: "12px 18px" }}>
            Status: {sync?.server_status ?? "disconnected"}
            {sync?.last_sync ? ` · last sync ${new Date(sync.last_sync).toLocaleString()}` : ""}
            {` · queued ${sync?.queued_changes ?? 0}`}
          </p>
          <div style={{ padding: "0 18px 18px" }}>
            <button
              className="btn"
              type="button"
              onClick={() => void api.syncNow().then(setSync).catch((e) => setModeError(String(e)))}
            >
              Sync now
            </button>
          </div>
        </>
      )}

      {mode === "local" && (
        <>
          <div className="detail-block">
            <h2>Upload local workspace</h2>
            {preview && (
              <p>
                {preview.companies} companies, {preview.tickets} tickets, {preview.issues} issues,{" "}
                {preview.products} products, {preview.assets} assets, {preview.invoices} leads.
                Preview only. A hosted server is still later.
              </p>
            )}
          </div>
          <div className="detail-block">
            <h2>Local data directory</h2>
            <p style={{ fontFamily: "var(--mono)", fontSize: "0.85rem" }}>{dataDir}</p>
          </div>
        </>
      )}
    </div>
  );
}
