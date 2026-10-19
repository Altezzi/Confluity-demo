export type StorageMode = "local" | "server" | "hybrid";

export type RecordType =
  | "company"
  | "contact"
  | "ticket"
  | "issue"
  | "document"
  | "project"
  | "product"
  | "asset"
  | "rma"
  | "order"
  | "invoice"
  | "vendor"
  | "purchase_order";

export type CommentVisibility = "internal" | "public";
export type ProjectStatus = "active" | "paused" | "done";
export type MemberRole = "admin" | "support" | "engineering" | "viewer";

export type TicketStatus = "open" | "in_progress" | "waiting" | "resolved" | "closed";
export type TicketPriority = "low" | "medium" | "high" | "urgent";
export type IssueStatus = "backlog" | "todo" | "in_progress" | "blocked" | "done";
export type IssuePriority = "low" | "medium" | "high" | "critical";

export interface Timestamps {
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
  version: number;
}

export interface Organization extends Timestamps {
  id: string;
  name: string;
}

export interface Workspace extends Timestamps {
  id: string;
  organization_id: string;
  name: string;
}

export interface Company extends Timestamps {
  id: string;
  workspace_id: string;
  name: string;
  domain: string | null;
  industry: string | null;
  notes: string | null;
}

export interface Contact extends Timestamps {
  id: string;
  workspace_id: string;
  company_id: string;
  name: string;
  email: string | null;
  title: string | null;
  phone: string | null;
  notes: string | null;
}

export interface Ticket extends Timestamps {
  id: string;
  workspace_id: string;
  company_id: string;
  contact_id: string | null;
  title: string;
  description: string | null;
  status: TicketStatus;
  priority: TicketPriority;
  tags: string | null;
  assignee_id: string | null;
  form_id: string | null;
  custom_fields: Record<string, string>;
}

export type TicketFieldKind = "system" | "text" | "textarea" | "dropdown" | "checkbox" | "number";

export interface TicketField {
  id: string;
  key: string;
  label: string;
  kind: TicketFieldKind | string;
  options: string[];
  hint: string | null;
  active: boolean;
  system: boolean;
}

export interface TicketFormField {
  field_id: string;
  position: number;
  required: boolean;
  field: TicketField;
}

export interface TicketForm {
  id: string;
  name: string;
  description: string | null;
  active: boolean;
  is_default: boolean;
  position: number;
  fields: TicketFormField[];
}

export interface Issue extends Timestamps {
  id: string;
  workspace_id: string;
  ticket_id: string | null;
  company_id: string | null;
  project_id: string | null;
  title: string;
  description: string | null;
  status: IssueStatus;
  priority: IssuePriority;
  blocked_reason: string | null;
  assignee_id: string | null;
}

export interface Member extends Timestamps {
  id: string;
  workspace_id: string;
  name: string;
  /** Denormalized highest-position role name for display. */
  role: MemberRole | string;
  title: string | null;
  /** Assigned org position (title in the company chart). */
  position_id: string | null;
  /** Assigned role ids (multi-role). */
  role_ids: string[];
  email: string | null;
  /** True until the member replaces their sign-in code with a password. */
  must_set_password: boolean;
  /** Present only right after an invite, or when an admin reveals it. */
  invite_code?: string | null;
}

export interface OrgCategory extends Timestamps {
  id: string;
  workspace_id: string;
  name: string;
  color: string;
  sort_order: number;
}

export interface Position extends Timestamps {
  id: string;
  workspace_id: string;
  category_id: string;
  name: string;
  parent_id: string | null;
  sort_order: number;
  /** Attached role ids for this position. */
  role_ids: string[];
  /** Permission keys granted to this position from each attached role. */
  role_permissions: Record<string, string[]>;
}

export interface Role extends Timestamps {
  id: string;
  workspace_id: string;
  name: string;
  color: string;
  position: number;
  is_system: boolean;
  /** Granted permission keys for this role. */
  permissions: string[];
}

export interface LiveQuery extends Timestamps {
  id: string;
  workspace_id: string;
  name: string;
  description: string | null;
  sql_text: string;
  created_by: string | null;
}

export interface LiveQueryResult {
  columns: string[];
  rows: (string | number | boolean | null)[][];
  truncated: boolean;
  ran_at: string;
}

export interface InsightTable {
  name: string;
  row_count: number;
}

export type ModuleMode = "native" | "connected";

export interface WorkspaceModule {
  workspace_id: string;
  module_id: string;
  enabled: boolean;
  mode: ModuleMode | string;
  connector_id: string | null;
  updated_at: string;
}

export type ConnectorCapability = "sync" | "import" | "webhook" | "embed";

export interface ConnectorDef {
  id: string;
  label: string;
  description: string;
  modules: string[];
  capabilities: ConnectorCapability[];
}

export type ConnectorStatus =
  | "disconnected"
  | "connected"
  | "syncing"
  | "error"
  | "archived";

export interface ConnectorInstance {
  id: string;
  workspace_id: string;
  connector_id: string;
  status: ConnectorStatus | string;
  config_json: string;
  last_sync_at: string | null;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
  version: number;
}

export interface ExternalRef {
  id: string;
  workspace_id: string;
  provider: string;
  external_id: string;
  entity_type: string;
  entity_id: string;
  source_url: string | null;
  metadata_json: string | null;
  last_synced_at: string;
  sync_conflict: boolean;
  created_at: string;
  updated_at: string;
}

export interface ConnectorSyncSummary {
  provider: string;
  created: number;
  updated: number;
  conflicts: number;
  skipped: number;
  message: string | null;
}

export interface AdoptModuleResult {
  module_id: string;
  mode: ModuleMode | string;
  summary: ConnectorSyncSummary | null;
}

export interface Team extends Timestamps {
  id: string;
  workspace_id: string;
  name: string;
  description: string | null;
  department: string | null;
  color: string;
  /** `department` for org-category teams, `chat` for invite-only chats. */
  kind: "department" | "chat" | string;
  created_by: string | null;
  /** Org category this department team is bound to. */
  category_id: string | null;
  member_ids: string[];
  channels: TeamChannel[];
}

export interface TeamChannel extends Timestamps {
  id: string;
  workspace_id: string;
  team_id: string;
  name: string;
  description: string | null;
  created_by: string | null;
}

export interface TeamMessage extends Timestamps {
  id: string;
  workspace_id: string;
  team_id: string;
  channel_id: string | null;
  author_id: string;
  author_name: string;
  body: string;
}

export interface DmThread {
  id: string;
  workspace_id: string;
  member_a_id: string;
  member_b_id: string;
  other_member_id: string;
  other_member_name: string;
  last_body: string | null;
  created_at: string;
  updated_at: string;
}

export interface DmMessage extends Timestamps {
  id: string;
  workspace_id: string;
  thread_id: string;
  author_id: string;
  author_name: string;
  body: string;
}

export type ObsSource = "datadog" | "grafana" | "cloudwatch";

export interface ObsAlert extends Timestamps {
  id: string;
  workspace_id: string;
  source: ObsSource | string;
  fingerprint: string;
  title: string;
  severity: string | null;
  external_url: string | null;
  raw_json: string;
  ticket_id: string | null;
  asset_id: string | null;
}

export interface ObsConnectorSettings {
  datadog_api_key: string | null;
  datadog_site: string | null;
  grafana_url: string | null;
  cloudwatch_region: string | null;
  outbound_enabled: boolean;
  default_company_id: string | null;
}

export interface ImportObsAlertInput {
  source: ObsSource | string;
  payload_json: string;
  company_id?: string | null;
}

export interface Project extends Timestamps {
  id: string;
  workspace_id: string;
  name: string;
  description: string | null;
  status: ProjectStatus | string;
}

export interface TicketOwnership {
  id: string;
  workspace_id: string;
  ticket_id: string;
  assignee_id: string | null;
  assignee_name: string;
  previous_assignee_id: string | null;
  previous_assignee_name: string | null;
  created_at: string;
}

export interface Comment extends Timestamps {
  id: string;
  workspace_id: string;
  entity_type: RecordType | string;
  entity_id: string;
  author: string;
  body: string;
  visibility: CommentVisibility | string;
}

export interface Document extends Timestamps {
  id: string;
  workspace_id: string;
  title: string;
  body: string;
  category: string | null;
  company_id: string | null;
  ticket_id: string | null;
  issue_id: string | null;
  visibility?: string | null;
}

export interface Activity extends Timestamps {
  id: string;
  workspace_id: string;
  actor: string;
  action: string;
  entity_type: RecordType;
  entity_id: string;
  summary: string;
  metadata: string | null;
}

export interface SearchResult {
  entity_type: RecordType;
  entity_id: string;
  title: string;
  snippet: string;
  rank: number;
}

export interface LinkedRecords {
  company: Company | null;
  contacts: Contact[];
  tickets: Ticket[];
  issues: Issue[];
  documents: Document[];
  products?: Product[];
  assets?: Asset[];
  rmas?: Rma[];
  orders?: Order[];
  invoices?: Invoice[];
}

export type McpTransport = "stdio" | "sse";

export interface ApiConnection {
  id: string;
  name: string;
  base_url: string;
  model: string;
  api_key_configured: boolean;
  enabled: boolean;
  is_default: boolean;
}

export interface McpConnection {
  id: string;
  name: string;
  transport: McpTransport;
  /** Stdio: executable (e.g. `npx`) */
  command: string | null;
  /** Stdio: one argument per line when editing */
  args: string[];
  /** Remote MCP (SSE) endpoint URL */
  url: string | null;
  auth_token_configured: boolean;
  enabled: boolean;
}

export interface AppSettings {
  storage_mode: StorageMode;
  server_url: string | null;
  workspace_id: string;
  workspace_name: string;
  organization_id: string;
  organization_name: string;
  ai_api_base_url: string | null;
  ai_model: string | null;
  ai_api_key_configured: boolean;
  seeded: boolean;
  current_member_id: string | null;
  server_status?: string;
  last_sync?: string | null;
  queued_changes?: number;
  setup_complete: boolean;
}

export interface SetupStatus {
  setup_complete: boolean;
  demo_data_present: boolean;
  organization_name: string;
  workspace_name: string;
  server_provider: string;
  admin_configured: boolean;
}

export interface Product extends Timestamps {
  id: string;
  workspace_id: string;
  name: string;
  sku: string | null;
  description: string | null;
  inventory_qty: number;
}

export interface Asset extends Timestamps {
  id: string;
  workspace_id: string;
  product_id: string | null;
  company_id: string | null;
  ticket_id: string | null;
  name: string;
  serial: string | null;
  warranty_expires_at: string | null;
  notes: string | null;
}

export interface Rma extends Timestamps {
  id: string;
  workspace_id: string;
  company_id: string | null;
  asset_id: string | null;
  ticket_id: string | null;
  status: string;
  reason: string | null;
  resolution: string | null;
}

export interface Order extends Timestamps {
  id: string;
  workspace_id: string;
  company_id: string | null;
  status: string;
  reference: string | null;
  notes: string | null;
  total_cents: number;
}

export interface InvoiceLine {
  id: string;
  invoice_id: string;
  description: string;
  quantity: number;
  unit_cents: number;
}

export interface Invoice extends Timestamps {
  id: string;
  workspace_id: string;
  company_id: string | null;
  order_id: string | null;
  status: string;
  reference: string | null;
  amount_cents: number;
  due_at: string | null;
  notes: string | null;
  lines: InvoiceLine[];
}

export interface Payment extends Timestamps {
  id: string;
  workspace_id: string;
  invoice_id: string;
  amount_cents: number;
  method: string | null;
  received_at: string | null;
  notes: string | null;
}

export interface Vendor extends Timestamps {
  id: string;
  workspace_id: string;
  name: string;
  email: string | null;
  notes: string | null;
}

export interface PurchaseOrder extends Timestamps {
  id: string;
  workspace_id: string;
  vendor_id: string | null;
  product_id: string | null;
  status: string;
  quantity: number;
  unit_cents: number;
  reference: string | null;
  notes: string | null;
}

export interface Attachment extends Timestamps {
  id: string;
  workspace_id: string;
  entity_type: string;
  entity_id: string;
  filename: string;
  stored_name: string;
  mime: string | null;
  size_bytes: number;
}

export interface AuditEntry {
  id: string;
  workspace_id: string;
  actor: string;
  action: string;
  entity_type: string;
  entity_id: string;
  summary: string;
  metadata: string | null;
  created_at: string;
}

export interface OutboxItem {
  id: string;
  workspace_id: string;
  entity_type: string;
  entity_id: string;
  action: string;
  payload: string | null;
  status: string;
  created_at: string;
  sent_at: string | null;
}

export interface Notification {
  id: string;
  workspace_id: string;
  member_id: string | null;
  title: string;
  body: string | null;
  entity_type: string | null;
  entity_id: string | null;
  read_at: string | null;
  created_at: string;
}

export interface WorkspaceEvent {
  id: string;
  workspace_id: string;
  event_type: string;
  entity_type: string;
  entity_id: string;
  payload: string | null;
  created_at: string;
}

export interface SyncStatus {
  server_url: string | null;
  server_status: string;
  last_sync: string | null;
  queued_changes: number;
}

export type CloudIdentityProvider = "none" | "supabase" | "custom_api";

export interface CloudIdentitySettings {
  provider: CloudIdentityProvider | string;
  supabase_url: string | null;
  supabase_anon_key_configured: boolean;
  supabase_service_role_configured: boolean;
  custom_api_url: string | null;
  sync_members_enabled: boolean;
  members_table: string;
  sync_data_enabled: boolean;
}

export interface IdentitySession {
  signed_in: boolean;
  provider: string | null;
  user_id: string | null;
  email: string | null;
  must_set_password: boolean;
}

export interface InvitedMember {
  member: Member;
  invite_code: string;
}

export interface IdentitySyncResult {
  ok: boolean;
  message: string;
  members_upserted: number;
}

export interface WorkspacePreview {
  companies: number;
  contacts: number;
  tickets: number;
  issues: number;
  documents: number;
  products: number;
  assets: number;
  rmas: number;
  orders: number;
  invoices: number;
}

export interface CreateCompanyInput {
  name: string;
  domain?: string | null;
  industry?: string | null;
  notes?: string | null;
}

export interface UpdateCompanyInput {
  id: string;
  name: string;
  domain?: string | null;
  industry?: string | null;
  notes?: string | null;
}

export interface CreateContactInput {
  company_id: string;
  name: string;
  email?: string | null;
  title?: string | null;
  phone?: string | null;
  notes?: string | null;
}

export interface UpdateContactInput {
  id: string;
  company_id: string;
  name: string;
  email?: string | null;
  title?: string | null;
  phone?: string | null;
  notes?: string | null;
}

export interface CreateTicketInput {
  company_id: string;
  contact_id?: string | null;
  title: string;
  description?: string | null;
  status?: TicketStatus;
  priority?: TicketPriority;
  tags?: string | null;
  assignee_id?: string | null;
  form_id?: string | null;
  custom_fields?: Record<string, string>;
}

export interface UpdateTicketInput {
  id: string;
  company_id: string;
  contact_id?: string | null;
  title: string;
  description?: string | null;
  status: TicketStatus;
  priority: TicketPriority;
  tags?: string | null;
  assignee_id?: string | null;
  form_id?: string | null;
  custom_fields?: Record<string, string>;
}

export interface CreateIssueInput {
  title: string;
  description?: string | null;
  ticket_id?: string | null;
  company_id?: string | null;
  project_id?: string | null;
  status?: IssueStatus;
  priority?: IssuePriority;
  blocked_reason?: string | null;
  assignee_id?: string | null;
}

export interface UpdateIssueInput {
  id: string;
  title: string;
  description?: string | null;
  ticket_id?: string | null;
  company_id?: string | null;
  project_id?: string | null;
  status: IssueStatus;
  priority: IssuePriority;
  blocked_reason?: string | null;
  assignee_id?: string | null;
}

export interface CreateProjectInput {
  name: string;
  description?: string | null;
  status?: ProjectStatus | string;
}

export interface CreateCommentInput {
  entity_type: RecordType | string;
  entity_id: string;
  body: string;
  visibility?: CommentVisibility | string;
  author?: string | null;
}

export interface CreateDocumentInput {
  title: string;
  body: string;
  category?: string | null;
  company_id?: string | null;
  ticket_id?: string | null;
  issue_id?: string | null;
}

export interface UpdateDocumentInput {
  id: string;
  title: string;
  body: string;
  category?: string | null;
  company_id?: string | null;
  ticket_id?: string | null;
  issue_id?: string | null;
}

export interface EscalateTicketInput {
  ticket_id: string;
  title?: string | null;
  description?: string | null;
  priority?: IssuePriority | null;
  blocked_reason?: string | null;
}

export interface AiChatMessage {
  role: "user" | "assistant" | "system";
  content: string;
}

export type AiIntent =
  | "ask"
  | "draft_reply"
  | "draft_note"
  | "draft_article"
  | "draft_update"
  | "prepare_rma"
  | "create_followup"
  | "create_issue"
  | "link_issue"
  | "update_record"
  | "action_plan";

export type AiDraftKind =
  | "reply"
  | "note"
  | "article"
  | "update"
  | "rma"
  | "ticket"
  | "issue"
  | "link"
  | "plan"
  | "record_update";

export interface AiDraft {
  kind: AiDraftKind;
  title: string | null;
  body: string;
  visibility: CommentVisibility | null;
  target_entity_type: RecordType | null;
  target_entity_id: string | null;
}

export interface AiChatRequest {
  messages: AiChatMessage[];
  context_entity_type?: RecordType | null;
  context_entity_id?: string | null;
  intent?: AiIntent | null;
}

export interface AiChatResponse {
  answer: string;
  citations: SearchResult[];
  draft: AiDraft | null;
}
