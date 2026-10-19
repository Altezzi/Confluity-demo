import { useCallback, useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { api } from "../../lib/api";
import { useApp } from "../../context/AppContext";

export function AiSettingsPage() {
  const { can } = useApp();
  const [mcpCount, setMcpCount] = useState(0);
  const [apiCount, setApiCount] = useState(0);
  const [mcpActive, setMcpActive] = useState(0);
  const [apiDefault, setApiDefault] = useState<string | null>(null);

  const load = useCallback(async () => {
    const [mcp, apiList] = await Promise.all([api.listMcpConnections(), api.listApiConnections()]);
    setMcpCount(mcp.length);
    setMcpActive(mcp.filter((c) => c.enabled).length);
    setApiCount(apiList.length);
    setApiDefault(apiList.find((c) => c.is_default)?.name ?? null);
  }, []);

  useEffect(() => {
    if (!can("settings.ai")) return;
    void load().catch(() => {});
  }, [can, load]);

  const mcpSummary = useMemo(() => {
    if (mcpCount === 0) return "No servers configured";
    return `${mcpActive} active · ${mcpCount} total`;
  }, [mcpActive, mcpCount]);

  const apiSummary = useMemo(() => {
    if (apiCount === 0) return "No providers configured";
    return apiDefault ? `${apiCount} total · default: ${apiDefault}` : `${apiCount} configured`;
  }, [apiCount, apiDefault]);

  if (!can("settings.ai")) {
    return <div className="empty">You do not have permission to manage AI Connections.</div>;
  }

  return (
    <div className="panel">
      <h2 style={{ marginTop: 0 }}>AI Connections</h2>
      <p className="meta" style={{ padding: "0 18px 12px", margin: 0 }}>
        Choose a connection type to view and manage your workspace catalog.
      </p>

      <div className="positions-tile-grid insights-tile-grid" style={{ padding: "0 18px 24px" }}>
        <Link to="/admin/ai/mcp" className="positions-tile" style={{ textDecoration: "none", color: "inherit" }}>
          <span className="title">MCP connections</span>
          <span className="meta">Model Context Protocol servers (tools &amp; data)</span>
          <span className="meta">{mcpSummary}</span>
        </Link>
        <Link to="/admin/ai/api" className="positions-tile" style={{ textDecoration: "none", color: "inherit" }}>
          <span className="title">API connections</span>
          <span className="meta">OpenAI-compatible chat completion providers</span>
          <span className="meta">{apiSummary}</span>
        </Link>
      </div>
    </div>
  );
}
