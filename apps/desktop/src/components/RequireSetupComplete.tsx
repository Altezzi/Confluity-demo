import { Navigate, Outlet } from "react-router-dom";
import { useApp } from "../context/AppContext";

export function RequireAppAccess() {
  const { settings, ready, signedIn, mustSetPassword } = useApp();

  if (!ready) {
    return (
      <div className="page">
        <div className="empty">Loading workspace…</div>
      </div>
    );
  }

  if (!signedIn) {
    return <Navigate to="/login" replace />;
  }

  if (mustSetPassword) {
    return <Navigate to="/set-password" replace />;
  }

  if (settings && !settings.setup_complete) {
    return <Navigate to="/setup" replace />;
  }

  return <Outlet />;
}
