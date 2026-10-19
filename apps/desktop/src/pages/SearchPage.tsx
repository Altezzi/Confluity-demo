import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import type { SearchResult } from "@confluity/shared";
import { api, recordPath } from "../lib/api";
import { useApp } from "../context/AppContext";

export function SearchPage() {
  const { setFocus } = useApp();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "Search" });
  }, [setFocus]);

  useEffect(() => {
    const t = setTimeout(async () => {
      if (!query.trim()) {
        setResults([]);
        return;
      }
      setResults(await api.search(query));
    }, 150);
    return () => clearTimeout(t);
  }, [query]);

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1>Search</h1>
          <p>Find companies, contacts, tickets, issues, and docs.</p>
        </div>
      </div>

      <div className="panel" style={{ marginBottom: 16 }}>
        <div className="form-row">
          <label>Query</label>
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="find it here.."
          />
        </div>
      </div>

      <div className="panel list">
        {results.map((r) => (
          <Link
            key={`${r.entity_type}-${r.entity_id}`}
            className="list-item"
            to={recordPath(r.entity_type, r.entity_id)}
          >
            <div>
              <div className="title">{r.title}</div>
              <div className="meta">
                {r.entity_type} · {r.snippet}
              </div>
            </div>
          </Link>
        ))}
        {results.length === 0 && <div className="empty">No results</div>}
      </div>
    </div>
  );
}
