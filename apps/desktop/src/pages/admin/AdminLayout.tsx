import { useEffect, useLayoutEffect, useRef, useState, type ReactNode } from "react";
import { Navigate, NavLink, Outlet, useLocation } from "react-router-dom";
import { useApp } from "../../context/AppContext";

type AdminTab = {
  to: string;
  label: string;
  anyOf: string[];
};

export const ADMIN_TABS: AdminTab[] = [
  {
    to: "overview",
    label: "Overview",
    anyOf: [
      "settings.manage_members",
      "settings.manage_roles",
      "settings.workspace_admin",
      "settings.integrations",
      "settings.data_server",
      "settings.ai",
    ],
  },
  { to: "ticket-forms", label: "Ticket forms", anyOf: ["settings.workspace_admin"] },
  { to: "modules", label: "Modules", anyOf: ["settings.workspace_admin"] },
  { to: "ai", label: "AI Connections", anyOf: ["settings.ai"] },
  { to: "activity", label: "Activity", anyOf: ["settings.data_server"] },
];

export const SETUP_TABS: AdminTab[] = [
  { to: "positions", label: "Org chart", anyOf: ["settings.manage_roles"] },
  { to: "roles", label: "Roles & permissions", anyOf: ["settings.manage_roles"] },
  { to: "members", label: "Invite the company", anyOf: ["settings.manage_members"] },
  { to: "integrations", label: "Company integrations", anyOf: ["settings.integrations"] },
  { to: "data", label: "Data & sync", anyOf: ["settings.data_server"] },
];

function TabIcon({ children }: { children: ReactNode }) {
  return (
    <svg className="settings-tab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" aria-hidden>
      {children}
    </svg>
  );
}

const TAB_ICONS: Record<string, ReactNode> = {
  overview: (
    <TabIcon>
      <rect x="3.5" y="3.5" width="7" height="7" rx="1.4" />
      <rect x="13.5" y="3.5" width="7" height="4" rx="1.2" />
      <rect x="13.5" y="10.5" width="7" height="10" rx="1.4" />
      <rect x="3.5" y="13.5" width="7" height="7" rx="1.4" />
    </TabIcon>
  ),
  members: (
    <TabIcon>
      <circle cx="9" cy="8" r="2.6" />
      <path d="M4.2 18c.6-2.5 2.4-3.8 4.8-3.8s4.2 1.3 4.8 3.8" strokeLinecap="round" />
      <circle cx="16.2" cy="9" r="2.1" />
      <path d="M14.2 18c.3-1.5 1.3-2.4 2.8-2.4 1.1 0 2 .5 2.5 1.4" strokeLinecap="round" />
    </TabIcon>
  ),
  roles: (
    <TabIcon>
      <circle cx="8" cy="14" r="3" />
      <path d="M11 14h8.2M16.6 14v2.3M19.2 14v1.6" strokeLinecap="round" />
    </TabIcon>
  ),
  positions: (
    <TabIcon>
      <rect x="9" y="3.5" width="6" height="4.5" rx="1" />
      <rect x="3.5" y="16" width="6" height="4.5" rx="1" />
      <rect x="14.5" y="16" width="6" height="4.5" rx="1" />
      <path d="M12 8v4M6.5 16v-2.2A1.8 1.8 0 0 1 8.3 12h7.4a1.8 1.8 0 0 1 1.8 1.8V16" />
    </TabIcon>
  ),
  "ticket-forms": (
    <TabIcon>
      <path d="M7 3.5h7l4 4V20a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1V4.5a1 1 0 0 1 1-1Z" strokeLinejoin="round" />
      <path d="M14 3.5V7.5h4M9 12.5h6M9 16.5h4" strokeLinecap="round" />
    </TabIcon>
  ),
  modules: (
    <TabIcon>
      <rect x="4" y="4" width="6.5" height="6.5" rx="1.2" />
      <rect x="13.5" y="4" width="6.5" height="6.5" rx="1.2" />
      <rect x="4" y="13.5" width="6.5" height="6.5" rx="1.2" />
      <rect x="13.5" y="13.5" width="6.5" height="6.5" rx="1.2" />
    </TabIcon>
  ),
  integrations: (
    <TabIcon>
      <path d="M9 6.5V11M15 6.5V11" strokeLinecap="round" />
      <path d="M7.5 11h9v2.4a4.5 4.5 0 0 1-9 0V11Z" strokeLinejoin="round" />
      <path d="M12 17.9V20.5" strokeLinecap="round" />
    </TabIcon>
  ),
  data: (
    <TabIcon>
      <rect x="4" y="4" width="16" height="6" rx="1.4" />
      <rect x="4" y="14" width="16" height="6" rx="1.4" />
      <path d="M8 7h.01M8 17h.01" strokeLinecap="round" strokeWidth="2.4" />
    </TabIcon>
  ),
  ai: (
    <TabIcon>
      <path d="M12 3.2 13.7 8.8 19.2 10.5 13.7 12.2 12 17.8 10.3 12.2 4.8 10.5 10.3 8.8 12 3.2Z" strokeLinejoin="round" />
    </TabIcon>
  ),
  activity: (
    <TabIcon>
      <path d="M3 12h4l2.2-5.2L13 17.2 15.2 12H21" strokeLinecap="round" strokeLinejoin="round" />
    </TabIcon>
  ),
};

export function canOpenAdmin(can: (key: string) => boolean): boolean {
  return ADMIN_TABS.some((tab) => tab.anyOf.some((key) => can(key)));
}

function textRowWidth(bar: HTMLElement): number {
  const style = getComputedStyle(bar);
  const pad = parseFloat(style.paddingLeft) + parseFloat(style.paddingRight);
  const gap = parseFloat(style.columnGap || style.gap || "0") || 0;
  const labels = [...bar.querySelectorAll<HTMLElement>(".settings-tab-label")];
  if (labels.length === 0) return 0;
  const widths = labels.map((label) => label.getBoundingClientRect().width);
  if (widths.some((width) => width <= 0)) return 0;
  const buttons = widths.reduce((sum, width) => sum + width + 26, 0);
  return buttons + Math.max(0, labels.length - 1) * gap + pad;
}

export function AdminLayout() {
  const { can, setFocus } = useApp();
  const location = useLocation();
  const tabsRef = useRef<HTMLDivElement>(null);
  const [compact, setCompact] = useState(false);

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "Admin" });
  }, [setFocus]);

  const visibleTabs = ADMIN_TABS.filter((tab) => tab.anyOf.some((key) => can(key)));

  useLayoutEffect(() => {
    const bar = tabsRef.current;
    if (!bar) return;

    const fit = () => {
      const available = bar.clientWidth;
      const needed = textRowWidth(bar);
      if (needed <= 0 || available <= 0) return;
      setCompact((prev) => {
        if (!prev && needed > available + 1) return true;
        if (prev && needed < available - 12) return false;
        return prev;
      });
    };

    fit();
    const observer = new ResizeObserver(fit);
    observer.observe(bar);
    return () => observer.disconnect();
  }, [location.pathname, visibleTabs.length]);

  if (visibleTabs.length === 0) {
    return (
      <div className="page">
        <div className="empty">You do not have access to Admin.</div>
      </div>
    );
  }

  const path = location.pathname.replace(/\/+$/, "");
  const onIndex = path.endsWith("/admin") || path.endsWith("/admin/");
  if (onIndex) {
    return <Navigate to={visibleTabs[0].to} replace />;
  }

  const tabMatchesPath = (tabTo: string) =>
    path.endsWith(`/admin/${tabTo}`) || path.includes(`/admin/${tabTo}/`);

  const knownTabs = [...ADMIN_TABS, ...SETUP_TABS];
  const onKnownTab = knownTabs.some((tab) => tabMatchesPath(tab.to));
  const allowedHere = knownTabs.some(
    (tab) => tabMatchesPath(tab.to) && tab.anyOf.some((key) => can(key)),
  );
  if (!onKnownTab && path.includes("/admin/")) {
    return <Navigate to={visibleTabs[0].to} replace />;
  }
  if (onKnownTab && !allowedHere) {
    return <Navigate to={visibleTabs[0].to} replace />;
  }

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1>Admin</h1>
          <p>People, access, ticket forms, modules, and workspace connections.</p>
        </div>
      </div>

      <div
        ref={tabsRef}
        className={`record-tabs settings-tabs${compact ? " compact" : ""}`}
        role="tablist"
        aria-label="Admin sections"
      >
        {visibleTabs.map((tab) => (
          <NavLink
            key={tab.to}
            to={tab.to}
            role="tab"
            aria-label={tab.label}
            title={compact ? tab.label : undefined}
            className={({ isActive }) => {
              const nestedSection =
                tab.to === "ai" && (path.endsWith("/admin/ai") || path.includes("/admin/ai/"));
              return `record-tab${isActive || nestedSection ? " active" : ""}`;
            }}
          >
            {TAB_ICONS[tab.to]}
            <span className="settings-tab-label">{tab.label}</span>
          </NavLink>
        ))}
      </div>

      <div className="record-tab-panels settings-tab-panels">
        <Outlet />
      </div>
    </div>
  );
}
