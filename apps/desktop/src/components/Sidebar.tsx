import { useEffect, useState } from "react";
import { NavLink } from "react-router-dom";
import type { Member } from "@confluity/shared";
import { moduleViewKey, type ModuleId } from "@confluity/shared";
import { api } from "../lib/api";
import { avatarTone, personInitials } from "./PersonChip";
import { canOpenAdmin } from "../pages/admin/AdminLayout";
import { useApp } from "../context/AppContext";

const items: {
  to: string;
  label: string;
  end?: boolean;
  module: ModuleId;
  icon: React.ReactNode;
}[] = [
  {
    to: "/",
    label: "Dashboard",
    end: true,
    module: "dashboard",
    icon: (
      <svg className="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
        <path d="M4 10.5 12 4l8 6.5V20a1 1 0 0 1-1 1h-5v-6H10v6H5a1 1 0 0 1-1-1v-9.5Z" strokeLinejoin="round" />
      </svg>
    ),
  },
  {
    to: "/crm",
    label: "CRM",
    module: "crm",
    icon: (
      <svg className="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
        <circle cx="9" cy="8" r="3" />
        <path d="M3.5 19c.8-3 2.9-4.5 5.5-4.5s4.7 1.5 5.5 4.5" strokeLinecap="round" />
        <circle cx="16.5" cy="9.5" r="2.2" />
        <path d="M14.2 19c.3-1.6 1.4-2.7 3.1-2.7 1.2 0 2.2.5 2.8 1.5" strokeLinecap="round" />
      </svg>
    ),
  },
  {
    to: "/support",
    label: "Support",
    module: "support",
    icon: (
      <svg className="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
        <path d="M7 8h10M7 12h6" strokeLinecap="round" />
        <path d="M6 4h12a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2h-5l-4 3v-3H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2Z" strokeLinejoin="round" />
      </svg>
    ),
  },
  {
    to: "/projects",
    label: "Projects",
    module: "projects",
    icon: (
      <svg className="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
        <path d="M4 7h16M4 12h16M4 17h10" strokeLinecap="round" />
      </svg>
    ),
  },
  {
    to: "/operations",
    label: "Operations",
    module: "operations",
    icon: (
      <svg className="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
        <rect x="4" y="4" width="7" height="7" rx="1.2" />
        <rect x="13" y="4" width="7" height="7" rx="1.2" />
        <rect x="4" y="13" width="7" height="7" rx="1.2" />
        <path d="M14.5 16.5h4M16.5 14.5v4" strokeLinecap="round" />
      </svg>
    ),
  },
  {
    to: "/finance",
    label: "Finance",
    module: "finance",
    icon: (
      <svg className="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
        <path d="M4 19V6.5A1.5 1.5 0 0 1 5.5 5H19" strokeLinecap="round" />
        <path d="M7 16h3M7 12h6M7 8h10" strokeLinecap="round" />
      </svg>
    ),
  },
  {
    to: "/portal",
    label: "Portal",
    module: "portal",
    icon: (
      <svg className="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
        <circle cx="12" cy="8" r="3" />
        <path d="M5 19c1.2-3.2 3.6-5 7-5s5.8 1.8 7 5" strokeLinecap="round" />
      </svg>
    ),
  },
  {
    to: "/docs",
    label: "Docs",
    module: "docs",
    icon: (
      <svg className="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
        <path d="M7 4h7l4 4v12a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1Z" strokeLinejoin="round" />
        <path d="M14 4v4h4" strokeLinejoin="round" />
      </svg>
    ),
  },
  {
    to: "/insights",
    label: "Insights",
    module: "insights",
    icon: (
      <svg className="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
        <path d="M4 19V6.5" strokeLinecap="round" />
        <rect x="7" y="12" width="3" height="7" rx="0.6" />
        <rect x="12" y="8" width="3" height="11" rx="0.6" />
        <rect x="17" y="4.5" width="3" height="14.5" rx="0.6" />
      </svg>
    ),
  },
  {
    to: "/teams",
    label: "Teams",
    module: "teams",
    icon: (
      <svg className="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
        <circle cx="9" cy="9" r="2.5" />
        <circle cx="16" cy="10" r="2.2" />
        <path d="M4.5 18c.7-2.4 2.4-3.6 4.5-3.6s3.8 1.2 4.5 3.6" strokeLinecap="round" />
        <path d="M13.2 18c.4-1.6 1.5-2.5 3-2.5 1.1 0 2 .4 2.6 1.2" strokeLinecap="round" />
      </svg>
    ),
  },
  {
    to: "/search",
    label: "Search",
    module: "search",
    icon: (
      <svg className="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
        <circle cx="11" cy="11" r="6.25" />
        <path d="m16 16 3.5 3.5" strokeLinecap="round" />
      </svg>
    ),
  },
  {
    to: "/settings",
    label: "Settings",
    module: "settings",
    icon: (
      <svg className="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
        <circle cx="12" cy="12" r="3" />
        <path d="M12 4v1.5M12 18.5V20M4 12h1.5M18.5 12H20M6.2 6.2l1.1 1.1M16.7 16.7l1.1 1.1M17.8 6.2l-1.1 1.1M7.3 16.7l-1.1 1.1" strokeLinecap="round" />
      </svg>
    ),
  },
];

function initials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return "C";
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  return `${parts[0][0]}${parts[1][0]}`.toUpperCase();
}

function localEmail(name: string, org?: string | null): string {
  const slug = name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, ".")
    .replace(/^\.+|\.+$/g, "");
  const domain = (org || "confluity.local")
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "")
    .slice(0, 24);
  return `${slug || "user"}@${domain || "confluity"}.local`;
}

function canSeeModule(can: (key: string) => boolean, module: ModuleId): boolean {
  if (module === "settings") {
    return (
      can(moduleViewKey("settings")) ||
      can("module.settings.edit") ||
      can("settings.preferences")
    );
  }
  return can(moduleViewKey(module));
}

const adminIcon = (
  <svg className="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
    <path d="M12 3.5 19 6.2v5.3c0 4.2-2.7 7.2-7 8.9-4.3-1.7-7-4.7-7-8.9V6.2L12 3.5Z" strokeLinejoin="round" />
    <path d="m9 12 2 2 4-4" strokeLinecap="round" strokeLinejoin="round" />
  </svg>
);

export function Sidebar() {
  const { settings, can, moduleEnabled } = useApp();
  const [member, setMember] = useState<Member | null>(null);
  const companyName = settings?.organization_name || "Your company";
  const workspaceName = settings?.workspace_name || "Workspace";

  useEffect(() => {
    void (async () => {
      try {
        const members = await api.listMembers();
        const me =
          members.find((m) => m.id === settings?.current_member_id) ?? members[0] ?? null;
        setMember(me);
      } catch {
        setMember(null);
      }
    })();
  }, [settings?.current_member_id]);

  const profileName = member?.name || "Local user";
  const profileMeta = member?.title?.trim() || localEmail(profileName, companyName);
  const tone = avatarTone(profileName);
  const visibleItems = items.filter(
    (item) => moduleEnabled(item.module) && canSeeModule(can, item.module),
  );
  const navItems = visibleItems.map((item) => ({
    to: item.to,
    label: item.label,
    end: item.end,
    icon: item.icon,
  }));
  if (canOpenAdmin(can)) {
    const settingsIndex = navItems.findIndex((item) => item.to === "/settings");
    const adminItem = { to: "/admin", label: "Admin", end: false, icon: adminIcon };
    if (settingsIndex >= 0) navItems.splice(settingsIndex, 0, adminItem);
    else navItems.push(adminItem);
  }

  return (
    <aside className="sidebar">
      <div className="company-brand">
        <div className="company-avatar" aria-hidden>
          {initials(companyName)}
        </div>
        <div className="company-copy">
          <div className="company-name">{companyName}</div>
          <div className="company-meta">{workspaceName}</div>
        </div>
      </div>
      <nav className="nav-section">
        <div className="nav-label">Workspace</div>
        {navItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            end={item.end}
            className={({ isActive }) => `nav-item${isActive ? " active" : ""}`}
          >
            {item.icon}
            {item.label}
          </NavLink>
        ))}
      </nav>
      <NavLink
        className={({ isActive }) => `sidebar-profile${isActive ? " active" : ""}`}
        to="/profile"
        title="Open profile"
      >
        <span className="sidebar-profile-avatar" style={tone} aria-hidden>
          {personInitials(profileName)}
        </span>
        <span className="sidebar-profile-copy">
          <span className="sidebar-profile-name">{profileName}</span>
          <span className="sidebar-profile-meta">{profileMeta}</span>
        </span>
      </NavLink>
    </aside>
  );
}
