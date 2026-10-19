import { useEffect } from "react";
import { Navigate, Outlet, useLocation } from "react-router-dom";
import { useApp } from "../../context/AppContext";

export function SettingsLayout() {
  const { can, setFocus } = useApp();
  const location = useLocation();
  const path = location.pathname.replace(/\/+$/, "");
  const canPreferences =
    can("settings.preferences") || can("module.settings.view") || can("module.settings.edit");

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "Settings" });
  }, [setFocus]);

  const onIndex = path.endsWith("/settings") || path.endsWith("/settings/");
  if (!onIndex && !path.endsWith("/settings/preferences")) {
    const target = path.replace(/\/settings(?=\/)/, "/admin");
    return <Navigate to={target} replace />;
  }

  if (!canPreferences) {
    return (
      <div className="page">
        <div className="empty">You do not have access to Settings.</div>
      </div>
    );
  }

  if (onIndex) {
    return <Navigate to="preferences" replace />;
  }

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1>Settings</h1>
          <p>Notifications, appearance, and how Confluity opens for you.</p>
        </div>
      </div>
      <div className="record-tab-panels settings-tab-panels">
        <Outlet />
      </div>
    </div>
  );
}
