use serde::Deserialize;
use std::sync::Arc;

use crate::db::{
    connectors::{
        connector_catalog, disconnect_connector, import_support_batch, list_connectors,
        list_external_refs_for_entity, upsert_connector, ConnectorDef, ConnectorInstance,
        ConnectorSyncSummary, ExternalRef,
    },
    get_meta,
    modules::{
        adopt_module, list_workspace_modules, set_workspace_module, WorkspaceModule,
    },
    DbError,
};
use crate::state::AppState;

fn map_err(e: DbError) -> String {
    e.to_string()
}

fn workspace_id(state: &AppState) -> Result<String, String> {
    let conn = state.conn.lock();
    get_meta(&conn, "workspace_id")
        .map_err(map_err)?
        .ok_or_else(|| "workspace not initialized".into())
}

#[derive(Debug, Deserialize)]
pub struct SetWorkspaceModuleInput {
    pub module_id: String,
    pub enabled: bool,
    pub mode: String,
    pub connector_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertConnectorInput {
    pub connector_id: String,
    pub status: Option<String>,
    pub config_json: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImportConnectorBatchInput {
    pub connector_id: String,
    pub payload_json: String,
    pub company_id: Option<String>,
}

#[tauri::command]
pub fn list_workspace_modules_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<WorkspaceModule>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_workspace_modules(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn is_module_enabled_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    module_id: String,
) -> Result<bool, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    crate::db::modules::is_module_enabled(&conn, &ws, module_id.trim()).map_err(map_err)
}

#[tauri::command]
pub fn set_workspace_module_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: SetWorkspaceModuleInput,
) -> Result<WorkspaceModule, String> {
    crate::commands::roles::require_perm(&state, "settings.workspace_admin")?;
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    set_workspace_module(
        &conn,
        &ws,
        input.module_id.trim(),
        input.enabled,
        input.mode.trim(),
        input.connector_id.as_deref(),
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn adopt_module_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    module_id: String,
) -> Result<WorkspaceModule, String> {
    crate::commands::roles::require_perm(&state, "settings.workspace_admin")?;
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    adopt_module(&conn, &ws, module_id.trim()).map_err(map_err)
}

#[tauri::command]
pub fn list_connector_catalog_cmd() -> Result<Vec<ConnectorDef>, String> {
    Ok(connector_catalog())
}

#[tauri::command]
pub fn list_connectors_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<ConnectorInstance>, String> {
    crate::commands::roles::require_perm(&state, "settings.integrations")?;
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_connectors(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn upsert_connector_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: UpsertConnectorInput,
) -> Result<ConnectorInstance, String> {
    crate::commands::roles::require_perm(&state, "settings.integrations")?;
    let ws = workspace_id(&state)?;
    let status = input.status.unwrap_or_else(|| "connected".into());
    let config = input.config_json.unwrap_or_else(|| "{}".into());
    let conn = state.conn.lock();
    upsert_connector(&conn, &ws, input.connector_id.trim(), status.trim(), &config).map_err(map_err)
}

#[tauri::command]
pub fn disconnect_connector_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<ConnectorInstance, String> {
    crate::commands::roles::require_perm(&state, "settings.integrations")?;
    let conn = state.conn.lock();
    disconnect_connector(&conn, &id).map_err(map_err)
}

#[tauri::command]
pub fn import_connector_batch_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: ImportConnectorBatchInput,
) -> Result<ConnectorSyncSummary, String> {
    crate::commands::roles::require_perm(&state, "settings.integrations")?;
    let provider = input.connector_id.trim().to_string();
    if provider != "zendesk" && provider != "jira" {
        return Err("batch import is available for zendesk and jira stubs".into());
    }
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    let _ = upsert_connector(&conn, &ws, &provider, "connected", "{}").map_err(map_err)?;
    // Wire module to connected mode when importing.
    let module_id = if provider == "jira" { "projects" } else { "support" };
    let _ = set_workspace_module(&conn, &ws, module_id, true, "connected", Some(&provider));
    import_support_batch(
        &conn,
        &ws,
        &provider,
        &input.payload_json,
        input.company_id.as_deref(),
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn list_external_refs_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    entity_type: String,
    entity_id: String,
) -> Result<Vec<ExternalRef>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_external_refs_for_entity(&conn, &ws, &entity_type, &entity_id).map_err(map_err)
}
