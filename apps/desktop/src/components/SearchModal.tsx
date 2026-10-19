import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import type { SearchResult } from "@confluity/shared";
import { api, recordPath } from "../lib/api";
import { useApp } from "../context/AppContext";

export function SearchModal() {
  const { searchOpen, setSearchOpen } = useApp();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);

  useEffect(() => {
    if (!searchOpen) return;
    const t = setTimeout(async () => {
      if (!query.trim()) {
        setResults([]);
        return;
      }
      setResults(await api.search(query));
    }, 150);
    return () => clearTimeout(t);
  }, [query, searchOpen]);

  if (!searchOpen) return null;

  return (
    <div className="modal-backdrop" onClick={() => setSearchOpen(false)}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <input
          autoFocus
          placeholder="Search workspace…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => e.key === "Escape" && setSearchOpen(false)}
        />
        <div className="modal-results">
          {results.length === 0 ? (
            <div className="empty">{query ? "No matches" : "Type to search across CRM, Support, Projects, and Docs"}</div>
          ) : (
            results.map((r) => (
              <Link
                key={`${r.entity_type}-${r.entity_id}`}
                className="list-item"
                to={recordPath(r.entity_type, r.entity_id)}
                onClick={() => setSearchOpen(false)}
              >
                <div>
                  <div className="title">{r.title}</div>
                  <div className="meta">
                    {r.entity_type} · {r.snippet}
                  </div>
                </div>
              </Link>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
