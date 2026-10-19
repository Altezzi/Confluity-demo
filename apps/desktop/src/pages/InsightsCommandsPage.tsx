import { useEffect, useMemo, useState } from "react";
import type { LiveQuery, LiveQueryResult } from "@confluity/shared";
import { api } from "../lib/api";
import { useApp } from "../context/AppContext";

function cellText(v: string | number | boolean | null): string {
  if (v == null) return "";
  return String(v);
}

function numericMax(rows: LiveQueryResult["rows"], col: number): number {
  let max = 0;
  for (const row of rows) {
    const n = Number(row[col]);
    if (Number.isFinite(n) && n > max) max = n;
  }
  return max || 1;
}

export function InsightsCommandsPage() {
  const { can } = useApp();
  const canEdit = can("module.insights.edit");
  const [queries, setQueries] = useState<LiveQuery[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [name, setName] = useState("");
  const [sql, setSql] = useState("");
  const [result, setResult] = useState<LiveQueryResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [live, setLive] = useState(true);

  const selected = useMemo(
    () => queries.find((q) => q.id === selectedId) ?? null,
    [queries, selectedId],
  );

  async function reload(preferId?: string | null) {
    const rows = await api.listLiveQueries();
    setQueries(rows);
    setSelectedId((curr) => {
      const want = preferId ?? curr;
      if (want && rows.some((q) => q.id === want)) return want;
      return rows[0]?.id ?? null;
    });
  }

  useEffect(() => {
    void reload().catch((e) => setError(e instanceof Error ? e.message : String(e)));
  }, []);

  useEffect(() => {
    if (!selected) return;
    setName(selected.name);
    setSql(selected.sql_text);
  }, [selected?.id, selected?.sql_text, selected?.name]);

  async function run(nextSql = sql) {
    if (!nextSql.trim()) return;
    setBusy(true);
    setError(null);
    try {
      const out = await api.runLiveQuery(nextSql);
      setResult(out);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    if (!sql.trim()) return;
    void run(sql);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedId]);

  useEffect(() => {
    if (!live || !sql.trim()) return;
    const id = window.setInterval(() => {
      void api
        .runLiveQuery(sql)
        .then(setResult)
        .catch((e) => setError(e instanceof Error ? e.message : String(e)));
    }, 4000);
    return () => window.clearInterval(id);
  }, [live, sql]);

  async function save() {
    if (!canEdit || !name.trim() || busy) return;
    setBusy(true);
    setError(null);
    try {
      const saved = await api.upsertLiveQuery({
        id: selected?.id ?? null,
        name: name.trim(),
        sql_text: sql,
      });
      await reload(saved.id);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function remove() {
    if (!canEdit || !selected || busy) return;
    if (!confirm(`Delete “${selected.name}”?`)) return;
    setBusy(true);
    try {
      await api.deleteLiveQuery(selected.id);
      setSelectedId(null);
      await reload(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  const chartable =
    result &&
    result.columns.length === 2 &&
    result.rows.length > 0 &&
    result.rows.every((row) => row[1] == null || Number.isFinite(Number(row[1])));
  const chartMax = chartable && result ? numericMax(result.rows, 1) : 1;

  return (
    <>
      <div className="insights-toolbar">
        <label className="insights-live-toggle">
          <span className={`insights-live-dot${live ? " on" : ""}`} aria-hidden />
          {live ? "Live" : "Paused"}
          <input type="checkbox" checked={live} onChange={(e) => setLive(e.target.checked)} />
        </label>
      </div>

      <div className="insights-shell">
        <aside className="panel insights-rail">
          <div className="panel-head">
            <h2 style={{ margin: 0 }}>Saved commands</h2>
            {canEdit && (
              <button
                className="btn"
                type="button"
                onClick={() => {
                  setSelectedId(null);
                  setName("New command");
                  setSql("");
                  setResult(null);
                }}
              >
                New
              </button>
            )}
          </div>
          <div className="list">
            {queries.map((q) => (
              <button
                key={q.id}
                type="button"
                className={`list-item${selectedId === q.id ? " active" : ""}`}
                style={{ width: "100%", textAlign: "left" }}
                onClick={() => setSelectedId(q.id)}
              >
                <div>
                  <div className="title">{q.name}</div>
                  <div className="meta">{q.description || "Live SQL"}</div>
                </div>
              </button>
            ))}
            {queries.length === 0 && <div className="empty">No saved commands yet</div>}
          </div>
        </aside>

        <section className="panel insights-main">
          <div className="form-grid" style={{ gap: 10 }}>
            <div className="form-row">
              <label>Name</label>
              <input
                value={name}
                disabled={!canEdit || busy}
                onChange={(e) => setName(e.target.value)}
                placeholder="Queue by status"
              />
            </div>
            <div className="form-row">
              <label>SQL</label>
              <textarea
                className="insights-sql"
                value={sql}
                disabled={busy}
                spellCheck={false}
                onChange={(e) => setSql(e.target.value)}
                rows={8}
              />
              <p className="meta" style={{ margin: "6px 0 0", padding: 0 }}>
                SELECT only. Tables include tickets, issues, companies, contacts, members, assets,
                rmas, orders.
              </p>
            </div>
            <div className="insights-actions">
              <button className="btn btn-primary" type="button" disabled={busy || !sql.trim()} onClick={() => void run()}>
                Run
              </button>
              {canEdit && (
                <button className="btn" type="button" disabled={busy || !name.trim()} onClick={() => void save()}>
                  Save for everyone
                </button>
              )}
              {canEdit && selected && (
                <button className="btn" type="button" disabled={busy} onClick={() => void remove()}>
                  Delete
                </button>
              )}
              {result && (
                <span className="meta" style={{ padding: 0 }}>
                  {result.rows.length} row{result.rows.length === 1 ? "" : "s"}
                  {result.truncated ? " (capped)" : ""} · refreshed {new Date(result.ran_at).toLocaleTimeString()}
                </span>
              )}
            </div>
          </div>

          {error && <p style={{ color: "var(--danger)", padding: "0 18px" }}>{error}</p>}

          {chartable && result && (
            <div className="insights-bars">
              {result.rows.map((row, i) => {
                const label = cellText(row[0]);
                const n = Number(row[1]) || 0;
                return (
                  <div key={`${label}-${i}`} className="insights-bar-row">
                    <span className="insights-bar-label">{label || "—"}</span>
                    <span className="insights-bar-track">
                      <span style={{ width: `${Math.max(4, (n / chartMax) * 100)}%` }} />
                    </span>
                    <span className="insights-bar-n">{n}</span>
                  </div>
                );
              })}
            </div>
          )}

          {result && result.columns.length > 0 && (
            <div className="insights-table-wrap">
              <table className="insights-table">
                <thead>
                  <tr>
                    {result.columns.map((col) => (
                      <th key={col}>{col}</th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {result.rows.map((row, ri) => (
                    <tr key={ri}>
                      {row.map((cell, ci) => (
                        <td key={ci}>{cellText(cell)}</td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
              {result.rows.length === 0 && <div className="empty">No rows right now</div>}
            </div>
          )}
        </section>
      </div>
    </>
  );
}
