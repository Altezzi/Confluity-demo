import { Navigate, useLocation } from "react-router-dom";
import { moduleForPath, moduleViewKey } from "@confluity/shared";
import { useApp } from "../context/AppContext";

/** Redirects away from modules the current member cannot view. */
export function RequireModule({ children }: { children: React.ReactNode }) {
  const { can, ready, moduleEnabled } = useApp();
  const location = useLocation();

  if (!ready) return null;

  const moduleId = moduleForPath(location.pathname);
  if (!moduleId) return <>{children}</>;

  if (moduleId === "settings") {
    const settingsAccess =
      can("module.settings.view") ||
      can("module.settings.edit") ||
      can("settings.manage_roles") ||
      can("settings.manage_members") ||
      can("settings.integrations") ||
      can("settings.ai") ||
      can("settings.data_server") ||
      can("settings.workspace_admin");
    if (!settingsAccess) return <Navigate to="/" replace />;
    return <>{children}</>;
  }

  if (!moduleEnabled(moduleId)) {
    return <Navigate to="/" replace />;
  }

  if (!can(moduleViewKey(moduleId))) {
    return <Navigate to="/" replace />;
  }

  return <>{children}</>;
}
