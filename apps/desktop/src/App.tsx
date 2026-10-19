import { HashRouter, Navigate, Route, Routes, useLocation, useParams } from "react-router-dom";
import { AppProvider } from "./context/AppContext";
import { RequireAppAccess } from "./components/RequireSetupComplete";
import { Layout } from "./components/Layout";
import { LoginPage } from "./pages/LoginPage";
import { SetPasswordPage } from "./pages/SetPasswordPage";
import { SetupPage } from "./pages/SetupPage";
import { RequireModule } from "./components/RequireModule";
import { HomePage } from "./pages/HomePage";
import { CrmPage } from "./pages/CrmPage";
import { CompanyDetailPage } from "./pages/CompanyDetailPage";
import { SupportPage } from "./pages/SupportPage";
import { TicketDetailPage } from "./pages/TicketDetailPage";
import { ProjectsPage } from "./pages/ProjectsPage";
import { IssueDetailPage } from "./pages/IssueDetailPage";
import { DocsPage } from "./pages/DocsPage";
import { DocDetailPage } from "./pages/DocDetailPage";
import { SearchPage } from "./pages/SearchPage";
import { AdminLayout } from "./pages/admin/AdminLayout";
import { AdminDashboard } from "./pages/admin/AdminDashboard";
import { SettingsLayout } from "./pages/settings/SettingsLayout";
import { PreferencesSettingsPage } from "./pages/settings/PreferencesSettingsPage";
import { DataModeSettingsPage, DataServerSettingsPage } from "./pages/settings/DataServerSettingsPage";
import { CloudIdentitySettingsPage } from "./pages/settings/CloudIdentitySettingsPage";
import { AiSettingsPage } from "./pages/settings/AiSettingsPage";
import { ApiConnectionsSettingsPage } from "./pages/settings/ApiConnectionsSettingsPage";
import { McpConnectionsSettingsPage } from "./pages/settings/McpConnectionsSettingsPage";
import { RolesSettingsPage } from "./pages/settings/RolesSettingsPage";
import { PositionsSettingsPage } from "./pages/settings/PositionsSettingsPage";
import { MembersSettingsPage } from "./pages/settings/MembersSettingsPage";
import { TicketFormsSettingsPage } from "./pages/settings/TicketFormsSettingsPage";
import { IntegrationsSettingsPage } from "./pages/settings/IntegrationsSettingsPage";
import { ModulesSettingsPage } from "./pages/settings/ModulesSettingsPage";
import { ActivitySettingsPage } from "./pages/settings/ActivitySettingsPage";
import { OperationsPage } from "./pages/OperationsPage";
import { AssetsPage, ProductsPage, RmasPage } from "./pages/OpsListPages";
import { AssetDetailPage, ProductDetailPage, RmaDetailPage } from "./pages/OpsDetailPages";
import { FinancePage } from "./pages/FinancePage";
import {
  InvoicesPage,
  OrdersPage,
  PurchaseOrdersPage,
  VendorsPage,
} from "./pages/FinanceListPages";
import { InvoiceDetailPage, OrderDetailPage } from "./pages/FinanceDetailPages";
import { PortalPage } from "./pages/PortalPage";
import { ProfilePage } from "./pages/ProfilePage";
import { InsightsLayout } from "./pages/InsightsLayout";
import { InsightsDashboardPage } from "./pages/InsightsDashboardPage";
import { InsightsTablesPage } from "./pages/InsightsTablesPage";
import { InsightsCommandsPage } from "./pages/InsightsCommandsPage";
import { TeamsPage } from "./pages/TeamsPage";
import { InboxPage } from "./pages/InboxPage";

function LegacyLeadRedirect() {
  const { id } = useParams();
  return <Navigate to={id ? `/finance/leads/${id}` : "/finance/leads"} replace />;
}

function LegacyAdminRedirect() {
  const location = useLocation();
  const target = location.pathname.replace(/^\/settings(?=\/)/, "/admin");
  return <Navigate to={`${target}${location.search}${location.hash}`} replace />;
}

function Guarded({ children }: { children: React.ReactNode }) {
  return <RequireModule>{children}</RequireModule>;
}

export default function App() {
  return (
    <AppProvider>
      <HashRouter>
        <Routes>
          <Route path="/login" element={<LoginPage />} />
          <Route path="/set-password" element={<SetPasswordPage />} />
          <Route path="/setup" element={<SetupPage />} />
          <Route element={<RequireAppAccess />}>
          <Route element={<Layout />}>
            <Route
              index
              element={
                <Guarded>
                  <HomePage />
                </Guarded>
              }
            />
            <Route
              path="crm"
              element={
                <Guarded>
                  <CrmPage />
                </Guarded>
              }
            />
            <Route
              path="crm/companies/:id"
              element={
                <Guarded>
                  <CompanyDetailPage />
                </Guarded>
              }
            />
            <Route
              path="support"
              element={
                <Guarded>
                  <SupportPage />
                </Guarded>
              }
            />
            <Route
              path="support/tickets/:id"
              element={
                <Guarded>
                  <TicketDetailPage />
                </Guarded>
              }
            />
            <Route
              path="projects"
              element={
                <Guarded>
                  <ProjectsPage />
                </Guarded>
              }
            />
            <Route
              path="projects/issues/:id"
              element={
                <Guarded>
                  <IssueDetailPage />
                </Guarded>
              }
            />
            <Route
              path="docs"
              element={
                <Guarded>
                  <DocsPage />
                </Guarded>
              }
            />
            <Route
              path="docs/:id"
              element={
                <Guarded>
                  <DocDetailPage />
                </Guarded>
              }
            />
            <Route
              path="operations"
              element={
                <Guarded>
                  <OperationsPage />
                </Guarded>
              }
            />
            <Route
              path="operations/products"
              element={
                <Guarded>
                  <ProductsPage />
                </Guarded>
              }
            />
            <Route
              path="operations/products/:id"
              element={
                <Guarded>
                  <ProductDetailPage />
                </Guarded>
              }
            />
            <Route
              path="operations/assets"
              element={
                <Guarded>
                  <AssetsPage />
                </Guarded>
              }
            />
            <Route
              path="operations/assets/:id"
              element={
                <Guarded>
                  <AssetDetailPage />
                </Guarded>
              }
            />
            <Route
              path="operations/rmas"
              element={
                <Guarded>
                  <RmasPage />
                </Guarded>
              }
            />
            <Route
              path="operations/rmas/:id"
              element={
                <Guarded>
                  <RmaDetailPage />
                </Guarded>
              }
            />
            <Route
              path="finance"
              element={
                <Guarded>
                  <FinancePage />
                </Guarded>
              }
            />
            <Route
              path="finance/leads"
              element={
                <Guarded>
                  <InvoicesPage />
                </Guarded>
              }
            />
            <Route
              path="finance/leads/:id"
              element={
                <Guarded>
                  <InvoiceDetailPage />
                </Guarded>
              }
            />
            <Route path="finance/invoices" element={<Navigate to="/finance/leads" replace />} />
            <Route path="finance/invoices/:id" element={<LegacyLeadRedirect />} />
            <Route
              path="finance/orders"
              element={
                <Guarded>
                  <OrdersPage />
                </Guarded>
              }
            />
            <Route
              path="finance/orders/:id"
              element={
                <Guarded>
                  <OrderDetailPage />
                </Guarded>
              }
            />
            <Route
              path="finance/vendors"
              element={
                <Guarded>
                  <VendorsPage />
                </Guarded>
              }
            />
            <Route
              path="finance/purchase-orders"
              element={
                <Guarded>
                  <PurchaseOrdersPage />
                </Guarded>
              }
            />
            <Route
              path="portal"
              element={
                <Guarded>
                  <PortalPage />
                </Guarded>
              }
            />
            <Route
              path="insights"
              element={
                <Guarded>
                  <InsightsLayout />
                </Guarded>
              }
            >
              <Route index element={<InsightsDashboardPage />} />
              <Route path="tables/:name" element={<InsightsTablesPage />} />
              <Route path="commands" element={<InsightsCommandsPage />} />
            </Route>
            <Route
              path="teams"
              element={
                <Guarded>
                  <TeamsPage />
                </Guarded>
              }
            />
            <Route path="inbox" element={<InboxPage />} />
            <Route path="profile" element={<ProfilePage />} />
            <Route path="profile/:memberId" element={<ProfilePage />} />
            <Route
              path="search"
              element={
                <Guarded>
                  <SearchPage />
                </Guarded>
              }
            />
            <Route
              path="admin"
              element={
                <Guarded>
                  <AdminLayout />
                </Guarded>
              }
            >
              <Route index element={<Navigate to="overview" replace />} />
              <Route path="overview" element={<AdminDashboard />} />
              <Route path="members" element={<MembersSettingsPage />} />
              <Route path="roles" element={<RolesSettingsPage />} />
              <Route path="positions" element={<PositionsSettingsPage />} />
              <Route path="ticket-forms" element={<TicketFormsSettingsPage />} />
              <Route path="modules" element={<ModulesSettingsPage />} />
              <Route path="integrations" element={<IntegrationsSettingsPage />} />
              <Route path="data" element={<DataServerSettingsPage />} />
              <Route path="data/local" element={<DataModeSettingsPage mode="local" />} />
              <Route path="data/server" element={<DataModeSettingsPage mode="server" />} />
              <Route path="data/hybrid" element={<DataModeSettingsPage mode="hybrid" />} />
              <Route path="data/identity" element={<CloudIdentitySettingsPage />} />
              <Route path="ai" element={<AiSettingsPage />} />
              <Route path="ai/mcp" element={<McpConnectionsSettingsPage />} />
              <Route path="ai/api" element={<ApiConnectionsSettingsPage />} />
              <Route path="activity" element={<ActivitySettingsPage />} />
            </Route>
            <Route
              path="settings"
              element={
                <Guarded>
                  <SettingsLayout />
                </Guarded>
              }
            >
              <Route index element={<Navigate to="preferences" replace />} />
              <Route path="preferences" element={<PreferencesSettingsPage />} />
              <Route path="*" element={<LegacyAdminRedirect />} />
            </Route>
            <Route path="*" element={<Navigate to="/" replace />} />
          </Route>
          </Route>
        </Routes>
      </HashRouter>
    </AppProvider>
  );
}
