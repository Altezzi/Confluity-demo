import type {
  Activity,
  AiChatRequest,
  AiChatResponse,
  AppSettings,
  ApiConnection,
  McpConnection,
  Comment,
  Company,
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
  Issue,
  Invoice,
  LinkedRecords,
  Member,
  Notification,
  Order,
  Product,
  Project,
  RecordType,
  Rma,
  Role,
  SearchResult,
  StorageMode,
  SyncStatus,
  Ticket,
  TicketOwnership,
  UpdateCompanyInput,
  UpdateContactInput,
  UpdateDocumentInput,
  UpdateIssueInput,
  UpdateTicketInput,
  Workspace,
} from "./types";

/** Repository contract — LocalProvider today, Remote/Hybrid later. */
export interface ConfluityRepository {
  getSettings(): Promise<AppSettings>;
  updateStorageMode(mode: StorageMode): Promise<AppSettings>;
  updateAiSettings(input: {
    ai_api_base_url?: string | null;
    ai_model?: string | null;
    ai_api_key?: string | null;
    clear_ai_api_key?: boolean | null;
  }): Promise<AppSettings>;
  listApiConnections(): Promise<ApiConnection[]>;
  saveApiConnections(
    connections: Array<Omit<ApiConnection, "api_key_configured"> & { api_key?: string | null; clear_api_key?: boolean }>,
  ): Promise<ApiConnection[]>;
  listMcpConnections(): Promise<McpConnection[]>;
  saveMcpConnections(connections: McpConnection[]): Promise<McpConnection[]>;
  ensureSeeded(): Promise<Workspace>;

  listCompanies(): Promise<Company[]>;
  getCompany(id: string): Promise<Company | null>;
  createCompany(input: CreateCompanyInput): Promise<Company>;
  updateCompany(input: UpdateCompanyInput): Promise<Company>;

  listContacts(companyId?: string | null): Promise<Contact[]>;
  getContact(id: string): Promise<Contact | null>;
  createContact(input: CreateContactInput): Promise<Contact>;
  updateContact(input: UpdateContactInput): Promise<Contact>;

  listTickets(companyId?: string | null): Promise<Ticket[]>;
  getTicket(id: string): Promise<Ticket | null>;
  createTicket(input: CreateTicketInput): Promise<Ticket>;
  updateTicket(input: UpdateTicketInput): Promise<Ticket>;
  escalateTicket(input: EscalateTicketInput): Promise<Issue>;
  listTicketOwnership(ticketId: string): Promise<TicketOwnership[]>;

  listIssues(projectId?: string | null): Promise<Issue[]>;
  getIssue(id: string): Promise<Issue | null>;
  createIssue(input: CreateIssueInput): Promise<Issue>;
  updateIssue(input: UpdateIssueInput): Promise<Issue>;

  listMembers(): Promise<Member[]>;
  listRoles(): Promise<Role[]>;
  getMyPermissions(): Promise<string[]>;
  listProjects(): Promise<Project[]>;
  createProject(input: CreateProjectInput): Promise<Project>;
  listComments(entityType: RecordType | string, entityId: string): Promise<Comment[]>;
  createComment(input: CreateCommentInput): Promise<Comment>;

  listDocuments(): Promise<Document[]>;
  getDocument(id: string): Promise<Document | null>;
  createDocument(input: CreateDocumentInput): Promise<Document>;
  updateDocument(input: UpdateDocumentInput): Promise<Document>;

  search(query: string): Promise<SearchResult[]>;
  getRelated(entityType: RecordType, entityId: string): Promise<LinkedRecords>;
  listActivities(limit?: number): Promise<Activity[]>;

  aiChat(request: AiChatRequest): Promise<AiChatResponse>;

  listProducts(): Promise<Product[]>;
  listAssets(): Promise<import("./types").Asset[]>;
  listRmas(): Promise<Rma[]>;
  listOrders(): Promise<Order[]>;
  listInvoices(): Promise<Invoice[]>;
  listNotifications(): Promise<Notification[]>;
  getSyncStatus(): Promise<SyncStatus>;
}
