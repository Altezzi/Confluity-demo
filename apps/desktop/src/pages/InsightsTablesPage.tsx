import { useEffect, useMemo, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import type { LiveQueryResult } from "@confluity/shared";
import { api, recordPath } from "../lib/api";
import { TABLE_ENTITY, tableLabel } from "./insightsMeta";

function cellText(v: string | number | boolean | null): string {
  if (v == null) return "";
  if (typeof v === "boolean") return v ? "1" : "0";
  return String(v);
}

function cellKind(v: string | number | boolean | null): "num" | "null" | "text" {
  if (v == null) return "null";
  if (typeof v === "number") return "num";
  return "text";
}

export function InsightsTablesPage() {
  const { name } = useParams();
  const navigate = useNavigate();
  const [result, setResult] = useState<LiveQueryResult | null>(null);
  const [rowCount, setRowCount] = useState<number | null>(null);
  const [filter, setFilter] = useState("");
  const [live, setLive] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const table = name?.trim() || "";

  async function reload() {
    if (!table) return;
    const [out, catalog] = await Promise.all([api.loadInsightTable(table), api.listInsightTables()]);
    setResult(out);
    setRowCount(catalog.find((t) => t.name === table)?.row_count ?? out.rows.length);
    setError(null);
  }

  useEffect(() => {
    setResult(null);
    setFilter("");
    void reload().catch((e) => setError(e instanceof Error ? e.message : String(e)));
  }, [table]);

  useEffect(() => {
    if (!live || !table) return;
    const id = window.setInterval(() => {
      void reload().catch((e) => setError(e instanceof Error ? e.message : String(e)));
    }, 4000);
    return () => window.clearInterval(id);
  }, [live, table]);

  const q = filter.trim().toLowerCase();
  const visibleRows = useMemo(() => {
    if (!result) return [];
    if (!q) return result.rows.map((row, i) => ({ row, i }));
    return result.rows
      .map((row, i) => ({ row, i }))
      .filter(({ row }) => row.some((cell) => cellText(cell).toLowerCase().includes(q)));
  }, [result, q]);

  const entity = TABLE_ENTITY[table];
  const idCol = result?.columns.findIndex((c) => c === "id") ?? -1;

  function openRow(row: (string | number | boolean | null)[]) {
    if (!entity || idCol < 0) return;
    const id = cellText(row[idCol]);
    if (!id) return;
    navigate(recordPath(entity, id));
  }

  return (
    <>
      <div className="insights-toolbar">
        <Link className="btn" to="/insights">
          ← Tables
        </Link>
        <span className="meta" style={{ padding: 0 }}>
          {tableLabel(table)}
          {rowCount != null ? ` · ${rowCount} row${rowCount === 1 ? "" : "s"}` : ""}
          {result?.truncated ? " · showing first 2000" : ""}
          {result ? ` · ${new Date(result.ran_at).toLocaleTimeString()}` : ""}
        </span>
        <label className="insights-live-toggle">
          <span className={`insights-live-dot${live ? " on" : ""}`} aria-hidden />
          {live ? "Live" : "Paused"}
          <input type="checkbox" checked={live} onChange={(e) => setLive(e.target.checked)} />
        </label>
        <input
          className="insights-filter"
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          placeholder="Filter rows"
          disabled={!result}
        />
      </div>

      {error && <p style={{ color: "var(--danger)", margin: "0 0 8px" }}>{error}</p>}

      <div className="insights-sheet insights-sheet-solo">
        {result && result.columns.length > 0 ? (
          <table className="sheet-table">
            <thead>
              <tr>
                <th className="sheet-row-n" />
                {result.columns.map((col) => (
                  <th key={col}>{col}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {visibleRows.map(({ row, i }) => (
                <tr
                  key={i}
                  className={entity ? "insights-row" : undefined}
                  onClick={entity ? () => openRow(row) : undefined}
                >
                  <th className="sheet-row-n">{i + 1}</th>
                  {row.map((cell, ci) => {
                    const kind = cellKind(cell);
                    return (
                      <td key={ci} className={kind === "text" ? undefined : `sheet-${kind}`}>
                        {kind === "null" ? "" : cellText(cell)}
                      </td>
                    );
                  })}
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          !error && <div className="empty">Loading table…</div>
        )}
        {result && visibleRows.length === 0 && (
          <div className="empty">{q ? "No matching rows" : "Table is empty"}</div>
        )}
      </div>
    </>
  );
}
