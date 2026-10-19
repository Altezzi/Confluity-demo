import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import type { InsightTable } from "@confluity/shared";
import { api } from "../lib/api";
import { tableLabel } from "./insightsMeta";

export function InsightsDashboardPage() {
  const navigate = useNavigate();
  const [tables, setTables] = useState<InsightTable[]>([]);
  const [error, setError] = useState<string | null>(null);

  async function reload() {
    try {
      setTables(await api.listInsightTables());
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  useEffect(() => {
    void reload();
    const id = window.setInterval(() => void reload(), 4000);
    return () => window.clearInterval(id);
  }, []);

  return (
    <>
      {error && <p style={{ color: "var(--danger)" }}>{error}</p>}
      <div className="positions-tile-grid insights-tile-grid">
        {tables.map((t) => (
          <button
            key={t.name}
            type="button"
            className="positions-tile"
            onClick={() => navigate(`/insights/tables/${t.name}`)}
          >
            <span className="title">{tableLabel(t.name)}</span>
            <span className="meta">
              {t.row_count} row{t.row_count === 1 ? "" : "s"}
            </span>
          </button>
        ))}
        {tables.length === 0 && !error && (
          <div className="empty" style={{ gridColumn: "1 / -1" }}>
            No tables yet
          </div>
        )}
      </div>
    </>
  );
}
