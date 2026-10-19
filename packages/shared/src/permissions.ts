/** Permission catalog — code source of truth for Roles UI and enforcers. */

export const MODULES = [
  "dashboard",
  "crm",
  "support",
  "projects",
  "operations",
  "finance",
  "portal",
  "docs",
  "insights",
  "teams",
  "search",
  "settings",
] as const;

export type ModuleId = (typeof MODULES)[number];

export type PermissionKey = string;

export interface PermissionFieldDef {
  key: PermissionKey;
  label: string;
}

export interface ModulePermissionGroup {
  id: ModuleId;
  label: string;
  fields: PermissionFieldDef[];
}

export const PANEL_FIELDS: PermissionFieldDef[] = [
  { key: "panel.comments", label: "Comments" },
  { key: "panel.attachments", label: "Attachments" },
  { key: "panel.linked_records", label: "Linked records" },
];

export const MODULE_PERMISSION_GROUPS: ModulePermissionGroup[] = [
  { id: "dashboard", label: "Dashboard", fields: [] },
  {
    id: "crm",
    label: "CRM",
    fields: [
      { key: "crm.field.notes", label: "Company notes" },
      { key: "crm.field.contact_pii", label: "Contact email & phone" },
      ...PANEL_FIELDS,
    ],
  },
  {
    id: "support",
    label: "Support",
    fields: [
      { key: "support.field.internal_comments", label: "Internal comments" },
      { key: "support.field.attachments", label: "Ticket attachments" },
      ...PANEL_FIELDS,
    ],
  },
  {
    id: "projects",
    label: "Projects",
    fields: [
      { key: "projects.field.blocked_reason", label: "Blocked reason" },
      ...PANEL_FIELDS,
    ],
  },
  {
    id: "operations",
    label: "Operations",
    fields: [
      { key: "operations.field.serial_numbers", label: "Serial numbers" },
      ...PANEL_FIELDS,
    ],
  },
  {
    id: "finance",
    label: "Finance",
    fields: [
      { key: "finance.field.amounts", label: "Amounts" },
      { key: "finance.field.line_items", label: "Line items" },
      ...PANEL_FIELDS,
    ],
  },
  { id: "portal", label: "Portal", fields: [...PANEL_FIELDS] },
  {
    id: "docs",
    label: "Docs",
    fields: [
      { key: "docs.field.internal", label: "Internal documents" },
      ...PANEL_FIELDS,
    ],
  },
  { id: "insights", label: "Insights", fields: [] },
  {
    id: "teams",
    label: "Teams",
    fields: [
      { key: "teams.manage", label: "Manage teams & membership" },
      { key: "teams.message", label: "Send messages & DMs" },
      { key: "teams.see_all_chats", label: "See all department chats" },
    ],
  },
  { id: "search", label: "Search", fields: [] },
  {
    id: "settings",
    label: "Settings",
    fields: [
      { key: "settings.preferences", label: "Personal preferences" },
      { key: "settings.manage_roles", label: "Manage roles" },
      { key: "settings.manage_members", label: "Manage members" },
      { key: "settings.integrations", label: "Integrations" },
      { key: "settings.ai", label: "AI Connections" },
      { key: "settings.data_server", label: "Data & Server" },
      { key: "settings.workspace_admin", label: "Workspace admin" },
    ],
  },
];

/** Every grantable key in the catalog (module view/edit + fields). */
export function allPermissionKeys(): PermissionKey[] {
  const keys: PermissionKey[] = [];
  for (const mod of MODULES) {
    keys.push(`module.${mod}.view`, `module.${mod}.edit`);
  }
  const seen = new Set<string>();
  for (const group of MODULE_PERMISSION_GROUPS) {
    for (const field of group.fields) {
      if (!seen.has(field.key)) {
        seen.add(field.key);
        keys.push(field.key);
      }
    }
  }
  return keys;
}

export function moduleViewKey(moduleId: ModuleId | string): PermissionKey {
  return `module.${moduleId}.view`;
}

export function moduleEditKey(moduleId: ModuleId | string): PermissionKey {
  return `module.${moduleId}.edit`;
}

/** Expand grants: module.*.edit implies module.*.view. */
export function expandEffectivePermissions(granted: Iterable<string>): Set<string> {
  const out = new Set<string>();
  for (const key of granted) {
    out.add(key);
    const m = /^module\.([a-z_]+)\.edit$/.exec(key);
    if (m) out.add(`module.${m[1]}.view`);
  }
  return out;
}

export function hasPermission(granted: Iterable<string>, key: string): boolean {
  const set = expandEffectivePermissions(granted);
  if (set.has(key)) return true;
  const m = /^module\.([a-z_]+)\.view$/.exec(key);
  if (m && set.has(`module.${m[1]}.edit`)) return true;
  return false;
}

/** Route prefix → module id for sidebar / route guards. */
export const ROUTE_MODULE: Record<string, ModuleId> = {
  "/": "dashboard",
  "/crm": "crm",
  "/support": "support",
  "/projects": "projects",
  "/operations": "operations",
  "/finance": "finance",
  "/portal": "portal",
  "/docs": "docs",
  "/insights": "insights",
  "/teams": "teams",
  "/search": "search",
  "/settings": "settings",
  "/admin": "settings",
};

/** Modules that stay available even when the workspace pack is minimal. */
export const ALWAYS_ON_MODULES: readonly ModuleId[] = [
  "dashboard",
  "search",
  "settings",
] as const;

/** Default enabled modules for a new workspace skeleton. */
export const DEFAULT_ENABLED_MODULES: readonly ModuleId[] = [
  "dashboard",
  "crm",
  "support",
  "projects",
  "docs",
  "insights",
  "teams",
  "search",
  "settings",
] as const;

export function isAlwaysOnModule(moduleId: ModuleId | string): boolean {
  return (ALWAYS_ON_MODULES as readonly string[]).includes(moduleId);
}

export function moduleForPath(pathname: string): ModuleId | null {
  const path = pathname.replace(/\/+$/, "") || "/";
  if (path === "/" || path === "") return "dashboard";
  const hit = Object.entries(ROUTE_MODULE).find(
    ([prefix, _]) => prefix !== "/" && (path === prefix || path.startsWith(`${prefix}/`)),
  );
  return hit ? hit[1] : null;
}
