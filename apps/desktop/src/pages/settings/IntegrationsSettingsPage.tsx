import { useEffect, useMemo, useState, type FormEvent } from "react";
import { Link } from "react-router-dom";
import type {
  Company,
  ConnectorDef,
  ConnectorInstance,
  ConnectorSyncSummary,
  ObsConnectorSettings,
  ObsSource,
} from "@confluity/shared";
import { api, formatLabel } from "../../lib/api";
import { useApp } from "../../context/AppContext";
import datadogSample from "../../fixtures/obs/datadog-sample.json";
import grafanaSample from "../../fixtures/obs/grafana-sample.json";
import cloudwatchSample from "../../fixtures/obs/cloudwatch-sample.json";

const OBS_SAMPLES: Record<ObsSource, string> = {
  datadog: JSON.stringify(datadogSample, null, 2),
  grafana: JSON.stringify(grafanaSample, null, 2),
  cloudwatch: JSON.stringify(cloudwatchSample, null, 2),
};

const ZENDESK_SAMPLE = JSON.stringify(
  [
    {
      id: "ZD-1001",
      subject: "Printer offline after firmware push",
      description: "Customer reports fleet devices dropping after OTA.",
      status: "open",
      priority: "high",
      url: "https://example.zendesk.com/agent/tickets/1001",
    },
    {
      id: "ZD-1002",
      subject: "Billing question on SO-8841",
      description: "Need invoice copy for AP.",
      status: "waiting",
      priority: "medium",
      url: "https://example.zendesk.com/agent/tickets/1002",
    },
  ],
  null,
  2,
);

const JIRA_SAMPLE = JSON.stringify(
  [
    {
      key: "ENG-441",
      summary: "Investigate OTA regression",
      description: "Mirrored from support handoff.",
      status: "todo",
      priority: "high",
      url: "https://example.atlassian.net/browse/ENG-441",
    },
    {
      key: "ENG-442",
      summary: "Add retry backoff on sync hub",
      status: "in_progress",
      priority: "medium",
      url: "https://example.atlassian.net/browse/ENG-442",
    },
  ],
  null,
  2,
);

function instanceFor(
  instances: ConnectorInstance[],
  connectorId: string,
): ConnectorInstance | undefined {
  return instances.find((c) => c.connector_id === connectorId);
}

export function IntegrationsSettingsPage() {
  const { can, moduleEnabled, refreshModules } = useApp();
  const [catalog, setCatalog] = useState<ConnectorDef[]>([]);
  const [instances, setInstances] = useState<ConnectorInstance[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [companies, setCompanies] = useState<Company[]>([]);
  const [companyId, setCompanyId] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [summary, setSummary] = useState<ConnectorSyncSummary | null>(null);
  const [busy, setBusy] = useState(false);

  const [payload, setPayload] = useState(ZENDESK_SAMPLE);
  const [obsSettings, setObsSettings] = useState<ObsConnectorSettings | null>(null);
  const [obsSource, setObsSource] = useState<ObsSource>("datadog");
  const [importedTicketId, setImportedTicketId] = useState<string | null>(null);
  const [emailSubject, setEmailSubject] = useState("");
  const [emailBody, setEmailBody] = useState("");
  const [emailFrom, setEmailFrom] = useState("maya.chen@northwindrobotics.com");
  const [csv, setCsv] = useState("name,domain,industry\n");
  const [csvContacts, setCsvContacts] = useState("name,email,title\n");
  const [importCount, setImportCount] = useState<number | null>(null);
  const [importContacts, setImportContacts] = useState<number | null>(null);

  async function reload() {
    const [cat, inst, comps] = await Promise.all([
      api.listConnectorCatalog(),
      api.listConnectors(),
      api.listCompanies(),
    ]);
    setCatalog(cat);
    setInstances(inst);
    setCompanies(comps);
    if (comps[0] && !companyId) setCompanyId(comps[0].id);
    await refreshModules();
  }

  useEffect(() => {
    void reload().catch((e) => setError(e instanceof Error ? e.message : String(e)));
    void api
      .getObsSettings()
      .then((s) => {
        setObsSettings(s);
        if (s.default_company_id) setCompanyId(s.default_company_id);
      })
      .catch(() => undefined);
  }, []);

  const visible = useMemo(
    () => catalog.filter((c) => c.modules.some((m) => moduleEnabled(m))),
    [catalog, moduleEnabled],
  );

  const selected = catalog.find((c) => c.id === selectedId) ?? null;
  const selectedInst = selected ? instanceFor(instances, selected.id) : undefined;

  useEffect(() => {
    if (!selected) return;
    if (selected.id === "zendesk") setPayload(ZENDESK_SAMPLE);
    else if (selected.id === "jira") setPayload(JIRA_SAMPLE);
    else if (selected.id === "datadog" || selected.id === "grafana" || selected.id === "cloudwatch") {
      setObsSource(selected.id);
      setPayload(OBS_SAMPLES[selected.id]);
    }
  }, [selected?.id]);

  if (!can("settings.integrations")) {
    return <div className="empty">You do not have permission to manage Integrations.</div>;
  }

  async function connect(def: ConnectorDef) {
    setBusy(true);
    setError(null);
    try {
      await api.upsertConnector({ connector_id: def.id, status: "connected", config_json: "{}" });
      const moduleId = def.modules[0];
      if (moduleId) {
        await api.setWorkspaceModule({
          module_id: moduleId,
          enabled: true,
          mode: "connected",
          connector_id: def.id,
        });
      }
      await reload();
      setSelectedId(def.id);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function disconnect(inst: ConnectorInstance) {
    setBusy(true);
    setError(null);
    try {
      await api.disconnectConnector(inst.id);
      await reload();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function runBatchImport() {
    if (!selected || (selected.id !== "zendesk" && selected.id !== "jira")) return;
    setBusy(true);
    setError(null);
    setSummary(null);
    try {
      const out = await api.importConnectorBatch({
        connector_id: selected.id,
        payload_json: payload,
        company_id: companyId || null,
      });
      setSummary(out);
      await reload();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function adoptSelected() {
    if (!selected) return;
    const moduleId = selected.modules[0];
    if (!moduleId) return;
    if (!confirm(`Adopt ${moduleId}? Confluity becomes source of truth; deep links stay.`)) return;
    setBusy(true);
    setError(null);
    try {
      await api.adoptModule(moduleId);
      await reload();
      setSummary({
        provider: selected.id,
        created: 0,
        updated: 0,
        conflicts: 0,
        skipped: 0,
        message: `${moduleId} adopted — now native Confluity`,
      });
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="panel">
      <h2 style={{ marginTop: 0 }}>Integrations</h2>
      <p className="meta" style={{ padding: "0 18px 12px", margin: 0 }}>
        Connect external SaaS tools without forcing a full switch. Sync or import into Confluity,
        then <strong>Adopt</strong> when you are ready. Module pack:{" "}
        <Link to="/admin/modules">Modules</Link>.
      </p>
      {error && <p style={{ color: "var(--danger)", padding: "0 18px" }}>{error}</p>}
      {summary && (
        <p className="meta" style={{ padding: "0 18px", color: "var(--success)" }}>
          {summary.message ||
            `${summary.provider}: +${summary.created} / ~${summary.updated} / skip ${summary.skipped}`}
        </p>
      )}

      <div className="positions-tile-grid insights-tile-grid" style={{ padding: "0 18px 18px" }}>
        {visible.map((def) => {
          const inst = instanceFor(instances, def.id);
          const status = inst?.status ?? "disconnected";
          return (
            <button
              key={def.id}
              type="button"
              className={`positions-tile${selectedId === def.id ? " active-tile" : ""}`}
              onClick={() => setSelectedId(def.id)}
            >
              <span className="title">{def.label}</span>
              <span className="meta">{def.description}</span>
              <span className="meta">
                {formatLabel(status)}
                {inst?.last_sync_at
                  ? ` · synced ${new Date(inst.last_sync_at).toLocaleString()}`
                  : ""}
              </span>
            </button>
          );
        })}
        {visible.length === 0 && (
          <div className="empty" style={{ gridColumn: "1 / -1" }}>
            Enable modules that use connectors first.
          </div>
        )}
      </div>

      {selected && (
        <div className="settings-block" style={{ padding: "0 18px 24px" }}>
          <div className="settings-subblock-head" style={{ marginBottom: 12 }}>
            <h3 style={{ margin: 0 }}>{selected.label}</h3>
            <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
              {(!selectedInst || selectedInst.status === "disconnected") && (
                <button className="btn btn-primary" type="button" disabled={busy} onClick={() => void connect(selected)}>
                  Connect
                </button>
              )}
              {selectedInst && selectedInst.status !== "disconnected" && selectedInst.status !== "archived" && (
                <button className="btn" type="button" disabled={busy} onClick={() => void disconnect(selectedInst)}>
                  Disconnect
                </button>
              )}
              {(selected.id === "zendesk" || selected.id === "jira") && (
                <button className="btn btn-primary" type="button" disabled={busy} onClick={() => void adoptSelected()}>
                  Adopt into Confluity
                </button>
              )}
            </div>
          </div>
          <p className="settings-block-lead">{selected.description}</p>
          <p className="meta" style={{ padding: 0 }}>
            Modules: {selected.modules.join(", ")} · Capabilities: {selected.capabilities.join(", ")}
          </p>

          {(selected.id === "zendesk" || selected.id === "jira") && (
            <form
              className="form-grid"
              style={{ marginTop: 12 }}
              onSubmit={(e) => {
                e.preventDefault();
                void runBatchImport();
              }}
            >
              <div className="form-row">
                <label>Import JSON (idempotent via external refs)</label>
                <textarea
                  value={payload}
                  onChange={(e) => setPayload(e.target.value)}
                  rows={12}
                  style={{ fontFamily: "var(--mono)", fontSize: "0.85rem" }}
                />
              </div>
              {selected.id === "zendesk" && (
                <div className="form-row">
                  <label>Company for tickets</label>
                  <select value={companyId} onChange={(e) => setCompanyId(e.target.value)}>
                    {companies.map((c) => (
                      <option key={c.id} value={c.id}>
                        {c.name}
                      </option>
                    ))}
                  </select>
                </div>
              )}
              <div className="form-actions">
                <button className="btn btn-primary" type="submit" disabled={busy}>
                  Sync / import
                </button>
                <button
                  className="btn"
                  type="button"
                  onClick={() => setPayload(selected.id === "jira" ? JIRA_SAMPLE : ZENDESK_SAMPLE)}
                >
                  Load sample
                </button>
              </div>
            </form>
          )}

          {(selected.id === "datadog" || selected.id === "grafana" || selected.id === "cloudwatch") &&
            obsSettings && (
              <>
                <form
                  className="form-grid"
                  style={{ marginTop: 12 }}
                  onSubmit={async (e) => {
                    e.preventDefault();
                    setBusy(true);
                    try {
                      setObsSettings(await api.updateObsSettings(obsSettings));
                      await api.upsertConnector({
                        connector_id: selected.id,
                        status: "connected",
                        config_json: JSON.stringify({ source: selected.id }),
                      });
                      await reload();
                    } catch (err) {
                      setError(err instanceof Error ? err.message : String(err));
                    } finally {
                      setBusy(false);
                    }
                  }}
                >
                  {selected.id === "datadog" && (
                    <>
                      <div className="form-row">
                        <label>Datadog API key</label>
                        <input
                          type="password"
                          value={obsSettings.datadog_api_key ?? ""}
                          onChange={(e) =>
                            setObsSettings({ ...obsSettings, datadog_api_key: e.target.value || null })
                          }
                          autoComplete="off"
                        />
                      </div>
                      <div className="form-row">
                        <label>Datadog site</label>
                        <input
                          value={obsSettings.datadog_site ?? "datadoghq.com"}
                          onChange={(e) =>
                            setObsSettings({ ...obsSettings, datadog_site: e.target.value || null })
                          }
                        />
                      </div>
                    </>
                  )}
                  {selected.id === "grafana" && (
                    <div className="form-row">
                      <label>Grafana URL</label>
                      <input
                        value={obsSettings.grafana_url ?? ""}
                        onChange={(e) =>
                          setObsSettings({ ...obsSettings, grafana_url: e.target.value || null })
                        }
                      />
                    </div>
                  )}
                  {selected.id === "cloudwatch" && (
                    <div className="form-row">
                      <label>Region</label>
                      <input
                        value={obsSettings.cloudwatch_region ?? "us-east-1"}
                        onChange={(e) =>
                          setObsSettings({
                            ...obsSettings,
                            cloudwatch_region: e.target.value || null,
                          })
                        }
                      />
                    </div>
                  )}
                  <div className="form-actions">
                    <button className="btn" type="submit" disabled={busy}>
                      Save connector
                    </button>
                  </div>
                </form>
                <form
                  className="form-grid"
                  style={{ marginTop: 12 }}
                  onSubmit={async (e: FormEvent) => {
                    e.preventDefault();
                    setImportedTicketId(null);
                    try {
                      const result = await api.importObsAlert({
                        source: obsSource,
                        payload_json: payload,
                        company_id: companyId || obsSettings.default_company_id || null,
                      });
                      setImportedTicketId(result.ticket_id);
                      await api.upsertConnector({
                        connector_id: selected.id,
                        status: "connected",
                        config_json: "{}",
                      });
                      await reload();
                    } catch (err) {
                      setError(err instanceof Error ? err.message : String(err));
                    }
                  }}
                >
                  <div className="form-row">
                    <label>Alert JSON</label>
                    <textarea
                      value={payload}
                      onChange={(e) => setPayload(e.target.value)}
                      rows={10}
                      style={{ fontFamily: "var(--mono)", fontSize: "0.85rem" }}
                    />
                  </div>
                  <div className="form-actions">
                    <button className="btn btn-primary" type="submit">
                      Import alert → ticket
                    </button>
                    {importedTicketId && (
                      <Link to={`/support/tickets/${importedTicketId}`}>Open ticket</Link>
                    )}
                  </div>
                </form>
              </>
            )}

          {selected.id === "email" && (
            <form
              className="form-grid"
              style={{ marginTop: 12 }}
              onSubmit={async (e) => {
                e.preventDefault();
                if (!companyId || !emailSubject.trim()) return;
                try {
                  await api.emailToTicket({
                    company_id: companyId,
                    from_email: emailFrom,
                    subject: emailSubject,
                    body: emailBody,
                  });
                  await api.upsertConnector({ connector_id: "email", status: "connected" });
                  setEmailSubject("");
                  setEmailBody("");
                  await reload();
                } catch (err) {
                  setError(err instanceof Error ? err.message : String(err));
                }
              }}
            >
              <div className="form-row">
                <label>Company</label>
                <select value={companyId} onChange={(e) => setCompanyId(e.target.value)}>
                  {companies.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.name}
                    </option>
                  ))}
                </select>
              </div>
              <div className="form-row">
                <label>From</label>
                <input value={emailFrom} onChange={(e) => setEmailFrom(e.target.value)} />
              </div>
              <div className="form-row">
                <label>Subject</label>
                <input value={emailSubject} onChange={(e) => setEmailSubject(e.target.value)} />
              </div>
              <div className="form-row">
                <label>Body</label>
                <textarea value={emailBody} onChange={(e) => setEmailBody(e.target.value)} />
              </div>
              <div className="form-actions">
                <button className="btn btn-primary" type="submit">
                  Create ticket from email
                </button>
              </div>
            </form>
          )}

          {selected.id === "csv" && (
            <div style={{ marginTop: 12 }}>
              <form
                className="form-grid"
                onSubmit={async (e) => {
                  e.preventDefault();
                  try {
                    const n = await api.importCompaniesCsv(csv);
                    setImportCount(n);
                    await api.upsertConnector({ connector_id: "csv", status: "connected" });
                    await reload();
                  } catch (err) {
                    setError(err instanceof Error ? err.message : String(err));
                  }
                }}
              >
                <div className="form-row">
                  <label>Companies CSV</label>
                  <textarea value={csv} onChange={(e) => setCsv(e.target.value)} />
                </div>
                <div className="form-actions">
                  <button className="btn" type="submit">
                    Import companies
                  </button>
                  {importCount !== null && <span className="meta">Imported {importCount}</span>}
                </div>
              </form>
              <form
                className="form-grid"
                style={{ marginTop: 12 }}
                onSubmit={async (e) => {
                  e.preventDefault();
                  if (!companyId) return;
                  try {
                    const n = await api.importContactsCsv(companyId, csvContacts);
                    setImportContacts(n);
                  } catch (err) {
                    setError(err instanceof Error ? err.message : String(err));
                  }
                }}
              >
                <div className="form-row">
                  <label>Contacts CSV</label>
                  <textarea value={csvContacts} onChange={(e) => setCsvContacts(e.target.value)} />
                </div>
                <div className="form-row">
                  <label>Company</label>
                  <select value={companyId} onChange={(e) => setCompanyId(e.target.value)}>
                    {companies.map((c) => (
                      <option key={c.id} value={c.id}>
                        {c.name}
                      </option>
                    ))}
                  </select>
                </div>
                <div className="form-actions">
                  <button className="btn" type="submit">
                    Import contacts
                  </button>
                  {importContacts !== null && <span className="meta">Imported {importContacts}</span>}
                </div>
              </form>
            </div>
          )}

          {(selected.id === "slack" || selected.id === "quickbooks") && (
            <p className="meta" style={{ padding: "12px 0 0" }}>
              Stub catalog entry — OAuth and live sync arrive with Confluity Server.
            </p>
          )}
        </div>
      )}
    </div>
  );
}
