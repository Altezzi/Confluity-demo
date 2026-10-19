import { NavLink, Outlet, useLocation } from "react-router-dom";
import { useEffect } from "react";
import { useApp } from "../context/AppContext";

export function InsightsLayout() {
  const { setFocus } = useApp();
  const { pathname } = useLocation();
  const onTables = pathname === "/insights" || pathname.startsWith("/insights/tables");

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "Insights" });
  }, [setFocus]);

  return (
    <div className="page page-wide insights-page">
      <div className="page-header">
        <div>
          <h1>Insights</h1>
          <p>Live tables of the workspace. When a record changes, everyone sees it.</p>
        </div>
      </div>

      <div className="record-tabs" role="tablist" aria-label="Insights sections">
        <NavLink to="/insights" end role="tab" className={() => `record-tab${onTables ? " active" : ""}`}>
          Tables
        </NavLink>
        <NavLink
          to="/insights/commands"
          role="tab"
          className={({ isActive }) => `record-tab${isActive ? " active" : ""}`}
        >
          Commands
        </NavLink>
      </div>

      <Outlet />
    </div>
  );
}
