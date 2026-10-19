import type { ConnectorCapability, ConnectorDef } from "./types";
import { MODULES, type ModuleId } from "./permissions";

/** Code-defined connector catalog — instances live in the DB. */
export const CONNECTOR_CATALOG: ConnectorDef[] = [
  {
    id: "zendesk",
    label: "Zendesk",
    description: "Sync support tickets. Stay on Zendesk, or adopt into Confluity Support.",
    modules: ["support"],
    capabilities: ["sync", "import"],
  },
  {
    id: "jira",
    label: "Jira",
    description: "Import engineering issues. Mirror boards, then adopt into Projects.",
    modules: ["projects"],
    capabilities: ["sync", "import"],
  },
  {
    id: "datadog",
    label: "Datadog",
    description: "Observability alerts → Support tickets with deep links.",
    modules: ["support", "operations"],
    capabilities: ["import", "webhook"],
  },
  {
    id: "grafana",
    label: "Grafana",
    description: "Import Grafana alert payloads into Support.",
    modules: ["support", "operations"],
    capabilities: ["import", "webhook"],
  },
  {
    id: "cloudwatch",
    label: "CloudWatch",
    description: "Import AWS CloudWatch alarms into Support.",
    modules: ["support", "operations"],
    capabilities: ["import", "webhook"],
  },
  {
    id: "csv",
    label: "CSV Import",
    description: "Bulk import companies and contacts.",
    modules: ["crm"],
    capabilities: ["import"],
  },
  {
    id: "email",
    label: "Email to ticket",
    description: "Turn inbound email into Support tickets.",
    modules: ["support"],
    capabilities: ["import", "webhook"],
  },
  {
    id: "slack",
    label: "Slack",
    description: "Coming soon — notify channels from workspace events.",
    modules: ["teams", "support"],
    capabilities: ["embed", "webhook"],
  },
  {
    id: "quickbooks",
    label: "QuickBooks",
    description: "Coming soon — sync invoices and payments into Finance.",
    modules: ["finance"],
    capabilities: ["sync", "import"],
  },
];

export function connectorDef(id: string): ConnectorDef | undefined {
  return CONNECTOR_CATALOG.find((c) => c.id === id);
}

export function connectorsForModules(enabledModuleIds: Iterable<string>): ConnectorDef[] {
  const set = new Set(enabledModuleIds);
  return CONNECTOR_CATALOG.filter((c) => c.modules.some((m) => set.has(m)));
}

export function isKnownModule(id: string): id is ModuleId {
  return (MODULES as readonly string[]).includes(id);
}

export type { ConnectorCapability };
