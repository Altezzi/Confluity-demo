import { useCallback, useEffect, useState, type FormEvent } from "react";
import { BackLink } from "../../components/BackLink";
import type { CloudIdentityProvider, CloudIdentitySettings, IdentitySession } from "@confluity/shared";
import { api } from "../../lib/api";
import { useApp } from "../../context/AppContext";

export function CloudIdentitySettingsPage() {
  const { refreshSettings, can } = useApp();
  const [settings, setSettings] = useState<CloudIdentitySettings | null>(null);
  const [session, setSession] = useState<IdentitySession | null>(null);
  const [provider, setProvider] = useState<CloudIdentityProvider>("none");
  const [supabaseUrl, setSupabaseUrl] = useState("");
  const [anonKey, setAnonKey] = useState("");
  const [serviceRole, setServiceRole] = useState("");
  const [customUrl, setCustomUrl] = useState("");
  const [membersTable, setMembersTable] = useState("workspace_members");
  const [syncMembers, setSyncMembers] = useState(true);
  const [syncData, setSyncData] = useState(false);
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [info, setInfo] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const load = useCallback(async () => {
    const [cfg, sess] = await Promise.all([
      api.getCloudIdentitySettings(),
      api.getIdentitySession(),
    ]);
    setSettings(cfg);
    setSession(sess);
    setProvider((cfg.provider as CloudIdentityProvider) || "none");
    setSupabaseUrl(cfg.supabase_url ?? "");
    setCustomUrl(cfg.custom_api_url ?? "");
    setMembersTable(cfg.members_table || "workspace_members");
    setSyncMembers(cfg.sync_members_enabled);
    setSyncData(cfg.sync_data_enabled);
  }, []);

  useEffect(() => {
    if (!can("settings.data_server")) return;
    void load().catch((e) => setError(e instanceof Error ? e.message : String(e)));
  }, [can, load]);

  if (!can("settings.data_server")) {
    return <div className="empty">You do not have permission to manage cloud login.</div>;
  }

  async function saveConfig(e: FormEvent) {
    e.preventDefault();
    setError(null);
    setInfo(null);
    setBusy(true);
    try {
      const next = await api.updateCloudIdentitySettings({
        provider,
        supabase_url: supabaseUrl || null,
        supabase_anon_key: anonKey.trim() || undefined,
        supabase_service_role: serviceRole.trim() || undefined,
        custom_api_url: customUrl || null,
        sync_members_enabled: syncMembers,
        members_table: membersTable,
        sync_data_enabled: syncData,
      });
      setSettings(next);
      setAnonKey("");
      setServiceRole("");
      await refreshSettings();
      setInfo("Cloud login settings saved. Hybrid sync uses this server when configured.");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function testConnection() {
    setError(null);
    setInfo(null);
    setBusy(true);
    try {
      const msg = await api.testCloudIdentity();
      setInfo(msg);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function signIn(e: FormEvent) {
    e.preventDefault();
    setError(null);
    setInfo(null);
    setBusy(true);
    try {
      const sess = await api.identitySignIn(email, password);
      setSession(sess);
      setPassword("");
      setInfo(`Signed in as ${sess.email ?? sess.user_id ?? "user"}.`);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function signOut() {
    setBusy(true);
    try {
      setSession(await api.identitySignOut());
      setInfo("Signed out.");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function syncNow() {
    setError(null);
    setInfo(null);
    setBusy(true);
    try {
      const result = await api.syncCloudIdentity();
      setInfo(result.message);
      await refreshSettings();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="panel">
      <BackLink to="/admin/data">Data & Server</BackLink>
      <h2 style={{ marginTop: 0, padding: "0 18px" }}>Cloud login &amp; sync</h2>
      <p className="meta" style={{ padding: "0 18px 12px", margin: 0 }}>
        Connect Supabase (or another HTTP API) for organization sign-in and optional member sync into
        this workspace. Keys and sessions stay on this machine.
      </p>
      {error && <p style={{ color: "var(--danger)", padding: "0 18px" }}>{error}</p>}
      {info && (
        <p className="meta" style={{ padding: "0 18px", color: "var(--success)" }}>
          {info}
        </p>
      )}

      <form className="form-grid" style={{ padding: "0 18px 18px" }} onSubmit={saveConfig}>
        <div className="form-row">
          <label>Provider</label>
          <select
            value={provider}
            onChange={(e) => setProvider(e.target.value as CloudIdentityProvider)}
          >
            <option value="none">None (local members only)</option>
            <option value="supabase">Supabase</option>
            <option value="custom_api">Custom API server</option>
          </select>
        </div>

        {provider === "supabase" && (
          <>
            <div className="form-row">
              <label>Supabase project URL</label>
              <input
                value={supabaseUrl}
                onChange={(e) => setSupabaseUrl(e.target.value)}
                placeholder="https://your-project.supabase.co"
              />
            </div>
            <div className="form-row">
              <label>Anon (public) key</label>
              <input
                type="password"
                value={anonKey}
                onChange={(e) => setAnonKey(e.target.value)}
                placeholder={
                  settings?.supabase_anon_key_configured
                    ? "Leave blank to keep existing key"
                    : "eyJ…"
                }
                autoComplete="off"
              />
            </div>
            <div className="form-row">
              <label>Service role key (optional, for member sync)</label>
              <input
                type="password"
                value={serviceRole}
                onChange={(e) => setServiceRole(e.target.value)}
                placeholder={
                  settings?.supabase_service_role_configured
                    ? "Leave blank to keep existing key"
                    : "Only needed if RLS blocks member reads"
                }
                autoComplete="off"
              />
            </div>
          </>
        )}

        {provider === "custom_api" && (
          <div className="form-row">
            <label>API base URL</label>
            <input
              value={customUrl}
              onChange={(e) => setCustomUrl(e.target.value)}
              placeholder="https://api.your-company.com"
            />
          </div>
        )}

        {provider !== "none" && (
          <>
            <label className="meta" style={{ display: "flex", gap: 8, alignItems: "center" }}>
              <input
                type="checkbox"
                checked={syncMembers}
                onChange={(e) => setSyncMembers(e.target.checked)}
              />
              Sync members from cloud table
            </label>
            {provider === "supabase" && syncMembers && (
              <div className="form-row">
                <label>Supabase table name</label>
                <input
                  value={membersTable}
                  onChange={(e) => setMembersTable(e.target.value)}
                  placeholder="workspace_members"
                />
                <span className="meta" style={{ marginTop: 6 }}>
                  Expected columns: <code>id</code>, <code>name</code> or <code>full_name</code>, optional{" "}
                  <code>email</code>, <code>role</code>, <code>title</code>.
                </span>
              </div>
            )}
            <label className="meta" style={{ display: "flex", gap: 8, alignItems: "center" }}>
              <input
                type="checkbox"
                checked={syncData}
                onChange={(e) => setSyncData(e.target.checked)}
              />
              Enable data sync (records) — reserved for a future Confluity Server / Supabase sync pass
            </label>
          </>
        )}

        <div className="form-actions">
          <button className="btn btn-primary" type="submit" disabled={busy}>
            Save cloud settings
          </button>
          {provider !== "none" && (
            <button className="btn" type="button" disabled={busy} onClick={() => void testConnection()}>
              Test connection
            </button>
          )}
        </div>
      </form>

      {provider === "supabase" && (
        <div className="settings-block" style={{ padding: "0 18px 24px" }}>
          <h3 style={{ marginTop: 0 }}>Sign in</h3>
          {session?.signed_in ? (
            <div>
              <p>
                Signed in as <strong>{session.email ?? session.user_id}</strong>
                {session.provider ? ` via ${session.provider}` : ""}.
              </p>
              <div className="form-actions">
                <button className="btn" type="button" disabled={busy} onClick={() => void signOut()}>
                  Sign out
                </button>
                <button className="btn btn-primary" type="button" disabled={busy} onClick={() => void syncNow()}>
                  Sync members now
                </button>
              </div>
            </div>
          ) : (
            <form className="form-grid" onSubmit={signIn}>
              <div className="form-row">
                <label>Email</label>
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  autoComplete="username"
                />
              </div>
              <div className="form-row">
                <label>Password</label>
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  autoComplete="current-password"
                />
              </div>
              <button className="btn btn-primary" type="submit" disabled={busy}>
                Sign in with Supabase
              </button>
            </form>
          )}
        </div>
      )}
    </div>
  );
}
