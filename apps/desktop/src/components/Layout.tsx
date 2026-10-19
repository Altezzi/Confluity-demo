import { Outlet, useLocation } from "react-router-dom";
import { useEffect } from "react";
import { useApp } from "../context/AppContext";
import { Sidebar } from "./Sidebar";
import { TopBar } from "./TopBar";
import { AiPanel } from "./AiPanel";
import { SearchModal } from "./SearchModal";
import { TitleBar } from "./TitleBar";
import { rememberPath } from "../lib/preferences";
import mark from "../assets/confluity-mark.png";

export function Layout() {
  const { ready, error, aiOpen } = useApp();
  const location = useLocation();

  useEffect(() => {
    if (ready) rememberPath(location.pathname);
  }, [ready, location.pathname]);

  if (error) {
    return (
      <div className="app-frame">
        <TitleBar />
        <div className="boot-screen">
          <div className="boot-card">
            <img src={mark} alt="" style={{ width: 36, height: 36, objectFit: "contain" }} />
            <h1>Couldn’t open workspace</h1>
            <p>{error}</p>
          </div>
        </div>
      </div>
    );
  }

  if (!ready) {
    return (
      <div className="app-frame">
        <TitleBar />
        <div className="boot-screen">
          <div className="boot-card">
            <img src={mark} alt="" style={{ width: 36, height: 36, objectFit: "contain" }} />
            <h1>Opening Confluity</h1>
            <p>Loading local workspace…</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="app-frame">
      <TitleBar />
      <div className={`app-shell${aiOpen ? " ai-open" : ""}`}>
        <Sidebar />
        <div className="main-column">
          <TopBar />
          <main className="content">
            <Outlet />
          </main>
        </div>
        <div className="ai-panel-slot">
          <AiPanel />
        </div>
        <SearchModal />
      </div>
    </div>
  );
}
