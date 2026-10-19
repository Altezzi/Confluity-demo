export const TABLE_LABELS: Record<string, string> = {
  tickets: "Tickets",
  issues: "Issues",
  companies: "Companies",
  contacts: "Contacts",
  members: "Members",
  rmas: "RMAs",
  orders: "Orders",
  invoices: "Invoices",
  payments: "Payments",
  assets: "Assets",
  products: "Products",
  documents: "Documents",
  projects: "Projects",
  vendors: "Vendors",
  purchase_orders: "Purchase orders",
};

export const TABLE_ENTITY: Record<string, string> = {
  tickets: "ticket",
  issues: "issue",
  companies: "company",
  contacts: "contact",
  rmas: "rma",
  orders: "order",
  invoices: "invoice",
  assets: "asset",
  products: "product",
  documents: "document",
  projects: "project",
  vendors: "vendor",
  purchase_orders: "purchase_order",
};

export function tableLabel(name: string) {
  return TABLE_LABELS[name] ?? name;
}
