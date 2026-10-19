import { invoke } from "@tauri-apps/api/core";
import type {
  Activity,
  AiChatRequest,
  AiChatResponse,
  AppSettings,
  ApiConnection,
  McpConnection,
  Asset,
  Attachment,
  AuditEntry,
  Comment,
  Company,
  ConnectorDef,
  ConnectorInstance,
  ConnectorSyncSummary,
  Contact,
  CreateCommentInput,
  CreateCompanyInput,
  CreateContactInput,
  CreateDocumentInput,
  CreateIssueInput,
  CreateProjectInput,
  CreateTicketInput,
  Document,
  EscalateTicketInput,
  ExternalRef,
  InsightTable,
  Invoice,
  Issue,
  LiveQuery,
  LiveQueryResult,
  LinkedRecords,
  Member,
  Notification,
  Order,
  OrgCategory,
  OutboxItem,
  Position,
  Product,
  Project,
  PurchaseOrder,
  RecordType,
  Rma,
  Role,
  SearchResult,
  StorageMode,
  CloudIdentitySettings,
  IdentitySession,
  InvitedMember,
  IdentitySyncResult,
  SetupStatus,
  SyncStatus,
  Team,
  TeamChannel,
  TeamMessage,
  DmThread,
  DmMessage,
  ImportObsAlertInput,
  ObsAlert,
  ObsConnectorSettings,
  Ticket,
  TicketField,
  TicketForm,
  TicketOwnership,
  UpdateCompanyInput,
  UpdateContactInput,
  UpdateDocumentInput,
  UpdateIssueInput,
  UpdateTicketInput,
  Vendor,
  Workspace,
  WorkspaceEvent,
  WorkspaceModule,
  WorkspacePreview,
} from "@confluity/shared";

export const api = {
  getSettings: () => invoke<AppSettings>("get_settings_cmd"),
  updateStorageMode: (mode: StorageMode) =>
    invoke<AppSettings>("update_storage_mode_cmd", { mode }),
  updateAiSettings: (input: {
    ai_api_base_url?: string | null;
    ai_model?: string | null;
    ai_api_key?: string | null;
    clear_ai_api_key?: boolean;
  }) => invoke<AppSettings>("update_ai_settings_cmd", { input }),
  listApiConnections: () => invoke<ApiConnection[]>("list_api_connections_cmd"),
  saveApiConnections: (
    connections: Array<
      Omit<ApiConnection, "api_key_configured"> & {
        api_key?: string | null;
        clear_api_key?: boolean;
      }
    >,
  ) => invoke<ApiConnection[]>("save_api_connections_cmd", { connections }),
  listMcpConnections: () => invoke<McpConnection[]>("list_mcp_connections_cmd"),
  saveMcpConnections: (
    connections: Array<
      Omit<McpConnection, "auth_token_configured"> & { auth_token?: string | null }
    >,
  ) => invoke<McpConnection[]>("save_mcp_connections_cmd", { connections }),
  ensureSeeded: () => invoke<Workspace>("ensure_seeded_cmd"),
  getSetupStatus: () => invoke<SetupStatus>("get_setup_status_cmd"),
  clearDemoForSetup: () => invoke<SetupStatus>("clear_demo_for_setup_cmd"),
  saveSetupWorkspace: (input: { organization_name: string; workspace_name: string }) =>
    invoke<SetupStatus>("save_setup_workspace_cmd", { input }),
  saveSetupServer: (input: {
    provider: string;
    supabase_url?: string | null;
    supabase_anon_key?: string | null;
    supabase_db_password?: string | null;
    custom_api_url?: string | null;
  }) => invoke<SetupStatus>("save_setup_server_cmd", { input }),
  saveSetupAdmin: (input: { name: string; email: string; password?: string | null }) =>
    invoke<SetupStatus>("save_setup_admin_cmd", { input }),
  completeSetupRoles: (useDefaults: boolean) =>
    invoke<Role[]>("complete_setup_roles_cmd", { input: { use_defaults: useDefaults } }),
  completeSetup: () => invoke<SetupStatus>("complete_setup_cmd"),
  getDataDir: () => invoke<string>("get_data_dir_cmd"),

  listCompanies: () => invoke<Company[]>("list_companies_cmd"),
  getCompany: (id: string) => invoke<Company | null>("get_company_cmd", { id }),
  createCompany: (input: CreateCompanyInput) =>
    invoke<Company>("create_company_cmd", { input }),
  updateCompany: (input: UpdateCompanyInput) =>
    invoke<Company>("update_company_cmd", { input }),

  listContacts: (companyId?: string | null) =>
    invoke<Contact[]>("list_contacts_cmd", { company_id: companyId ?? null }),
  getContact: (id: string) => invoke<Contact | null>("get_contact_cmd", { id }),
  createContact: (input: CreateContactInput) =>
    invoke<Contact>("create_contact_cmd", { input }),
  updateContact: (input: UpdateContactInput) =>
    invoke<Contact>("update_contact_cmd", { input }),

  listTickets: (companyId?: string | null) =>
    invoke<Ticket[]>("list_tickets_cmd", { company_id: companyId ?? null }),
  getTicket: (id: string) => invoke<Ticket | null>("get_ticket_cmd", { id }),
  createTicket: (input: CreateTicketInput) =>
    invoke<Ticket>("create_ticket_cmd", { input }),
  updateTicket: (input: UpdateTicketInput) =>
    invoke<Ticket>("update_ticket_cmd", { input }),
  listTicketForms: () => invoke<TicketForm[]>("list_ticket_forms_cmd"),
  listTicketFields: () => invoke<TicketField[]>("list_ticket_fields_cmd"),
  saveTicketForm: (input: {
    id?: string | null;
    name: string;
    description?: string | null;
    active: boolean;
    is_default: boolean;
    fields: {
      field_id: string;
      required: boolean;
      label: string;
      hint?: string | null;
      options: string[];
    }[];
  }) => invoke<TicketForm>("save_ticket_form_cmd", { input }),
  createTicketField: (input: {
    label: string;
    kind: string;
    options: string[];
    hint?: string | null;
    form_id?: string | null;
  }) => invoke<TicketField>("create_ticket_field_cmd", { input }),
  deleteTicketField: (id: string) => invoke<void>("delete_ticket_field_cmd", { id }),
  deleteTicketForm: (id: string) => invoke<TicketForm[]>("delete_ticket_form_cmd", { id }),
  escalateTicket: (input: EscalateTicketInput) =>
    invoke<Issue>("escalate_ticket_cmd", { input }),
  listTicketOwnership: (ticketId: string) =>
    invoke<TicketOwnership[]>("list_ticket_ownership_cmd", {
      ticketId,
      ticket_id: ticketId,
    }),

  listIssues: (projectId?: string | null) =>
    invoke<Issue[]>("list_issues_cmd", { project_id: projectId ?? null }),
  getIssue: (id: string) => invoke<Issue | null>("get_issue_cmd", { id }),
  createIssue: (input: CreateIssueInput) =>
    invoke<Issue>("create_issue_cmd", { input }),
  updateIssue: (input: UpdateIssueInput) =>
    invoke<Issue>("update_issue_cmd", { input }),

  listMembers: () => invoke<Member[]>("list_members_cmd"),
  listProjects: () => invoke<Project[]>("list_projects_cmd"),
  createProject: (input: CreateProjectInput) =>
    invoke<Project>("create_project_cmd", { input }),
  listComments: (entityType: RecordType | string, entityId: string) =>
    invoke<Comment[]>("list_comments_cmd", {
      input: { entity_type: entityType, entity_id: entityId },
      entityType,
      entityId,
      entity_type: entityType,
      entity_id: entityId,
    }),
  createComment: (input: CreateCommentInput) =>
    invoke<Comment>("create_comment_cmd", { input }),

  listDocuments: () => invoke<Document[]>("list_documents_cmd"),
  getDocument: (id: string) => invoke<Document | null>("get_document_cmd", { id }),
  createDocument: (input: CreateDocumentInput) =>
    invoke<Document>("create_document_cmd", { input }),
  updateDocument: (input: UpdateDocumentInput) =>
    invoke<Document>("update_document_cmd", { input }),

  search: (query: string) => invoke<SearchResult[]>("search_cmd", { query }),
  getRelated: (entityType: RecordType, entityId: string) =>
    invoke<LinkedRecords>("get_related_cmd", {
      entity_type: entityType,
      entity_id: entityId,
    }),
  listActivities: (limit?: number) =>
    invoke<Activity[]>("list_activities_cmd", { limit: limit ?? 30 }),

  aiChat: (request: AiChatRequest) =>
    invoke<AiChatResponse>("ai_chat_cmd", { request }),

  listProducts: () => invoke<Product[]>("list_products_cmd"),
  getProduct: (id: string) => invoke<Product | null>("get_product_cmd", { id }),
  createProduct: (input: {
    name: string;
    sku?: string | null;
    description?: string | null;
    inventory_qty?: number;
  }) => invoke<Product>("create_product_cmd", { input }),
  updateProduct: (input: {
    id: string;
    name: string;
    sku?: string | null;
    description?: string | null;
    inventory_qty: number;
  }) => invoke<Product>("update_product_cmd", { input }),
  listAssets: () => invoke<Asset[]>("list_assets_cmd"),
  getAsset: (id: string) => invoke<Asset | null>("get_asset_cmd", { id }),
  createAsset: (input: {
    name: string;
    product_id?: string | null;
    company_id?: string | null;
    ticket_id?: string | null;
    serial?: string | null;
    warranty_expires_at?: string | null;
    notes?: string | null;
  }) => invoke<Asset>("create_asset_cmd", { input }),
  listRmas: () => invoke<Rma[]>("list_rmas_cmd"),
  getRma: (id: string) => invoke<Rma | null>("get_rma_cmd", { id }),
  createRma: (input: {
    company_id?: string | null;
    asset_id?: string | null;
    ticket_id?: string | null;
    reason?: string | null;
    status?: string | null;
  }) => invoke<Rma>("create_rma_cmd", { input }),
  updateRma: (input: {
    id: string;
    status: string;
    reason?: string | null;
    resolution?: string | null;
  }) => invoke<Rma>("update_rma_cmd", { input }),
  listOrders: () => invoke<Order[]>("list_orders_cmd"),
  getOrder: (id: string) => invoke<Order | null>("get_order_cmd", { id }),
  createOrder: (input: {
    company_id?: string | null;
    status?: string | null;
    reference?: string | null;
    notes?: string | null;
    total_cents?: number;
  }) => invoke<Order>("create_order_cmd", { input }),
  listInvoices: () => invoke<Invoice[]>("list_invoices_cmd"),
  getInvoice: (id: string) => invoke<Invoice | null>("get_invoice_cmd", { id }),
  createInvoice: (input: {
    company_id?: string | null;
    order_id?: string | null;
    status?: string | null;
    reference?: string | null;
    amount_cents?: number;
    due_at?: string | null;
    notes?: string | null;
    lines?: { description: string; quantity?: number; unit_cents?: number }[];
  }) => invoke<Invoice>("create_invoice_cmd", { input }),
  createPayment: (input: {
    invoice_id: string;
    amount_cents: number;
    method?: string | null;
    notes?: string | null;
  }) => invoke<unknown>("create_payment_cmd", { input }),
  listVendors: () => invoke<Vendor[]>("list_vendors_cmd"),
  createVendor: (input: { name: string; email?: string | null; notes?: string | null }) =>
    invoke<Vendor>("create_vendor_cmd", { input }),
  listPurchaseOrders: () => invoke<PurchaseOrder[]>("list_purchase_orders_cmd"),
  createPurchaseOrder: (input: {
    vendor_id?: string | null;
    product_id?: string | null;
    status?: string | null;
    quantity?: number;
    unit_cents?: number;
    reference?: string | null;
    notes?: string | null;
  }) => invoke<PurchaseOrder>("create_purchase_order_cmd", { input }),
  listAttachments: (entityType: string, entityId: string) =>
    invoke<Attachment[]>("list_attachments_cmd", {
      entityType,
      entity_type: entityType,
      entityId,
      entity_id: entityId,
    }),
  addAttachment: (input: {
    entity_type: string;
    entity_id: string;
    filename: string;
    mime?: string | null;
    bytes: number[];
  }) => invoke<Attachment>("add_attachment_cmd", { input }),
  listNotifications: () => invoke<Notification[]>("list_notifications_cmd"),
  markNotificationRead: (id: string) => invoke<void>("mark_notification_read_cmd", { id }),
  markAllNotificationsRead: () => invoke<void>("mark_all_notifications_read_cmd"),
  listAudit: (limit?: number) => invoke<AuditEntry[]>("list_audit_cmd", { limit: limit ?? 40 }),
  listOutbox: () => invoke<OutboxItem[]>("list_outbox_cmd"),
  listEvents: () => invoke<WorkspaceEvent[]>("list_events_cmd"),
  getSyncStatus: () => invoke<SyncStatus>("get_sync_status_cmd"),
  getCloudIdentitySettings: () =>
    invoke<CloudIdentitySettings>("get_cloud_identity_settings_cmd"),
  updateCloudIdentitySettings: (input: {
    provider: string;
    supabase_url?: string | null;
    supabase_anon_key?: string | null;
    clear_supabase_anon_key?: boolean;
    supabase_service_role?: string | null;
    clear_supabase_service_role?: boolean;
    custom_api_url?: string | null;
    sync_members_enabled?: boolean;
    members_table?: string | null;
    sync_data_enabled?: boolean;
  }) => invoke<CloudIdentitySettings>("update_cloud_identity_settings_cmd", { input }),
  getIdentitySession: () => invoke<IdentitySession>("get_identity_session_cmd"),
  testCloudIdentity: () => invoke<string>("test_cloud_identity_cmd"),
  identitySignIn: (email: string, password: string) =>
    invoke<IdentitySession>("identity_sign_in_cmd", { input: { email, password } }),
  workspaceSignIn: (email: string, password: string) =>
    invoke<IdentitySession>("workspace_sign_in_cmd", { input: { email, password } }),
  inviteMember: (input: { name: string; email: string; position_id?: string | null }) =>
    invoke<InvitedMember>("invite_member_cmd", { input }),
  revealMemberInvite: (memberId: string) =>
    invoke<string>("reveal_member_invite_cmd", { input: { member_id: memberId } }),
  completeMemberPassword: (password: string, confirm: string) =>
    invoke<IdentitySession>("complete_member_password_cmd", {
      input: { password, confirm },
    }),
  identitySignOut: () => invoke<IdentitySession>("identity_sign_out_cmd"),
  syncCloudIdentity: () => invoke<IdentitySyncResult>("sync_cloud_identity_cmd"),
  connectServer: (serverUrl: string, mode?: string) =>
    invoke<SyncStatus>("connect_server_cmd", {
      input: { server_url: serverUrl, mode: mode ?? "hybrid" },
    }),
  disconnectServer: () => invoke<SyncStatus>("disconnect_server_cmd"),
  syncNow: () => invoke<SyncStatus>("sync_now_cmd"),
  workspacePreview: () => invoke<WorkspacePreview>("workspace_preview_cmd"),
  createMember: (input: { name: string; role?: string | null; title?: string | null }) =>
    invoke<Member>("create_member_cmd", { input }),
  updateMember: (input: { id: string; name: string; role: string; title?: string | null }) =>
    invoke<Member>("update_member_cmd", { input }),
  listRoles: () => invoke<Role[]>("list_roles_cmd"),
  createRole: (input: { name: string; color?: string | null; permissions?: string[] | null }) =>
    invoke<Role>("create_role_cmd", { input }),
  updateRole: (input: { id: string; name: string; color: string; position: number }) =>
    invoke<Role>("update_role_cmd", { input }),
  deleteRole: (id: string) => invoke<void>("delete_role_cmd", { id }),
  resetDefaultRoles: () => invoke<Role[]>("reset_default_roles_cmd"),
  setRolePermissions: (roleId: string, permissions: string[]) =>
    invoke<Role>("set_role_permissions_cmd", {
      input: { role_id: roleId, permissions },
    }),
  setMemberRoles: (memberId: string, roleIds: string[]) =>
    invoke<Member>("set_member_roles_cmd", {
      input: { member_id: memberId, role_ids: roleIds },
    }),
  listOrgCategories: () => invoke<OrgCategory[]>("list_org_categories_cmd"),
  createOrgCategory: (name: string, color?: string | null) =>
    invoke<OrgCategory>("create_org_category_cmd", {
      input: { name, color: color ?? null },
    }),
  updateOrgCategory: (input: { id: string; name: string; color: string }) =>
    invoke<OrgCategory>("update_org_category_cmd", { input }),
  deleteOrgCategory: (id: string) => invoke<void>("delete_org_category_cmd", { id }),
  listPositions: () => invoke<Position[]>("list_positions_cmd"),
  createPosition: (input: {
    category_id: string;
    name: string;
    parent_id?: string | null;
  }) => invoke<Position>("create_position_cmd", { input }),
  updatePosition: (input: { id: string; name: string; parent_id?: string | null }) =>
    invoke<Position>("update_position_cmd", { input }),
  deletePosition: (id: string) => invoke<void>("delete_position_cmd", { id }),
  setPositionRoles: (positionId: string, roleIds: string[]) =>
    invoke<Position>("set_position_roles_cmd", {
      input: { position_id: positionId, role_ids: roleIds },
    }),
  setPositionRolePermissions: (positionId: string, roleId: string, permissions: string[]) =>
    invoke<Position>("set_position_role_permissions_cmd", {
      input: { position_id: positionId, role_id: roleId, permissions },
    }),
  setMemberPosition: (memberId: string, positionId: string | null) =>
    invoke<Member>("set_member_position_cmd", {
      input: { member_id: memberId, position_id: positionId },
    }),
  reorderRoles: (roleId: string, swapWithId: string) =>
    invoke<Role[]>("reorder_roles_cmd", {
      input: { role_id: roleId, swap_with_id: swapWithId },
    }),
  getMyPermissions: () => invoke<string[]>("get_my_permissions_cmd"),
  listTeams: () => invoke<Team[]>("list_teams_cmd"),
  getTeam: (id: string) => invoke<Team | null>("get_team_cmd", { id }),
  createTeam: (input: {
    name: string;
    description?: string | null;
    department?: string | null;
    color?: string | null;
    member_ids?: string[] | null;
  }) => invoke<Team>("create_team_cmd", { input }),
  setTeamMembers: (teamId: string, memberIds: string[]) =>
    invoke<Team>("set_team_members_cmd", {
      input: { team_id: teamId, member_ids: memberIds },
    }),
  listTeamChannels: (teamId: string) =>
    invoke<TeamChannel[]>("list_team_channels_cmd", { teamId, team_id: teamId }),
  createTeamChannel: (input: { team_id: string; name: string; description?: string | null }) =>
    invoke<TeamChannel>("create_team_channel_cmd", { input }),
  createGroupChat: (input: {
    name: string;
    description?: string | null;
    member_ids: string[];
    color?: string | null;
  }) => invoke<Team>("create_group_chat_cmd", { input }),
  canCreateTeamChats: () => invoke<boolean>("can_create_team_chats_cmd"),
  listTeamMessages: (teamId: string, channelId?: string | null) =>
    invoke<TeamMessage[]>("list_team_messages_cmd", {
      teamId,
      team_id: teamId,
      channelId: channelId ?? null,
      channel_id: channelId ?? null,
    }),
  sendTeamMessage: (teamId: string, body: string, channelId?: string | null) =>
    invoke<TeamMessage>("send_team_message_cmd", {
      input: { team_id: teamId, body, channel_id: channelId ?? null },
    }),
  listDmThreads: () => invoke<DmThread[]>("list_dm_threads_cmd"),
  openDm: (memberId: string) =>
    invoke<DmThread>("open_dm_cmd", { input: { member_id: memberId } }),
  listDmMessages: (threadId: string) =>
    invoke<DmMessage[]>("list_dm_messages_cmd", { threadId, thread_id: threadId }),
  sendDmMessage: (threadId: string, body: string) =>
    invoke<DmMessage>("send_dm_message_cmd", {
      input: { thread_id: threadId, body },
    }),
  emailToTicket: (input: {
    company_id: string;
    contact_id?: string | null;
    from_email?: string | null;
    subject: string;
    body: string;
  }) => invoke<Ticket>("email_to_ticket_cmd", { input }),
  getObsSettings: () => invoke<ObsConnectorSettings>("get_obs_settings_cmd"),
  updateObsSettings: (input: Partial<ObsConnectorSettings>) =>
    invoke<ObsConnectorSettings>("update_obs_settings_cmd", { input }),
  importObsAlert: (input: ImportObsAlertInput) =>
    invoke<{ alert: ObsAlert; ticket_id: string }>("import_obs_alert_cmd", { input }),
  listObsAlerts: () => invoke<ObsAlert[]>("list_obs_alerts_cmd"),
  listObsAlertsForTicket: (ticketId: string) =>
    invoke<ObsAlert[]>("list_obs_alerts_for_ticket_cmd", {
      ticketId,
      ticket_id: ticketId,
    }),
  importCompaniesCsv: (csv: string) =>
    invoke<number>("import_companies_csv_cmd", { input: { csv } }),
  importContactsCsv: (companyId: string, csv: string) =>
    invoke<number>("import_contacts_csv_cmd", { input: { company_id: companyId, csv } }),
  notifyWorkspace: (title: string, body?: string, entityType?: string, entityId?: string) =>
    invoke<void>("notify_workspace_cmd", {
      title,
      body: body ?? null,
      entity_type: entityType ?? null,
      entity_id: entityId ?? null,
    }),
  listInsightTables: () => invoke<InsightTable[]>("list_insight_tables_cmd"),
  loadInsightTable: (name: string) => invoke<LiveQueryResult>("load_insight_table_cmd", { name }),
  listLiveQueries: () => invoke<LiveQuery[]>("list_live_queries_cmd"),
  runLiveQuery: (sqlText: string) =>
    invoke<LiveQueryResult>("run_live_query_cmd", { sqlText, sql_text: sqlText }),
  upsertLiveQuery: (input: {
    id?: string | null;
    name: string;
    description?: string | null;
    sql_text: string;
  }) => invoke<LiveQuery>("upsert_live_query_cmd", { input }),
  deleteLiveQuery: (id: string) => invoke<void>("delete_live_query_cmd", { id }),

  listWorkspaceModules: () => invoke<WorkspaceModule[]>("list_workspace_modules_cmd"),
  setWorkspaceModule: (input: {
    module_id: string;
    enabled: boolean;
    mode: string;
    connector_id?: string | null;
  }) => invoke<WorkspaceModule>("set_workspace_module_cmd", { input }),
  adoptModule: (moduleId: string) =>
    invoke<WorkspaceModule>("adopt_module_cmd", { moduleId, module_id: moduleId }),
  listConnectorCatalog: () => invoke<ConnectorDef[]>("list_connector_catalog_cmd"),
  listConnectors: () => invoke<ConnectorInstance[]>("list_connectors_cmd"),
  upsertConnector: (input: {
    connector_id: string;
    status?: string | null;
    config_json?: string | null;
  }) => invoke<ConnectorInstance>("upsert_connector_cmd", { input }),
  disconnectConnector: (id: string) => invoke<ConnectorInstance>("disconnect_connector_cmd", { id }),
  importConnectorBatch: (input: {
    connector_id: string;
    payload_json: string;
    company_id?: string | null;
  }) => invoke<ConnectorSyncSummary>("import_connector_batch_cmd", { input }),
  listExternalRefs: (entityType: string, entityId: string) =>
    invoke<ExternalRef[]>("list_external_refs_cmd", {
      entityType,
      entity_type: entityType,
      entityId,
      entity_id: entityId,
    }),
};

/** Display snake_case values like in_progress as "in progress". */
export function formatLabel(value: string): string {
  return value.replace(/_/g, " ");
}

export function recordPath(type: string, id: string): string {
  switch (type) {
    case "company":
      return `/crm/companies/${id}`;
    case "contact":
      return `/crm/contacts`;
    case "ticket":
      return `/support/tickets/${id}`;
    case "issue":
      return `/projects/issues/${id}`;
    case "document":
      return `/docs/${id}`;
    case "project":
      return `/projects`;
    case "product":
      return `/operations/products/${id}`;
    case "asset":
      return `/operations/assets/${id}`;
    case "rma":
      return `/operations/rmas/${id}`;
    case "order":
      return `/finance/orders/${id}`;
    case "invoice":
      return `/finance/leads/${id}`;
    case "vendor":
      return `/finance/vendors`;
    case "purchase_order":
      return `/finance/purchase-orders`;
    default:
      return "/";
  }
}
