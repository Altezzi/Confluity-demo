use serde::Deserialize;
use std::sync::Arc;

use crate::db::{
    get_meta,
    models::get_ticket,
    obs::{
        emit_datadog_event_blocking, get_obs_settings, import_alert, list_alerts,
        list_alerts_for_ticket, update_obs_settings, ObsAlert, ObsConnectorSettings,
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
pub struct ImportObsAlertInput {
    pub source: String,
    pub payload_json: String,
    pub company_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateObsSettingsInput {
    pub datadog_api_key: Option<String>,
    pub datadog_site: Option<String>,
    pub grafana_url: Option<String>,
    pub cloudwatch_region: Option<String>,
    pub outbound_enabled: Option<bool>,
    pub default_company_id: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ImportObsAlertResult {
    pub alert: ObsAlert,
    pub ticket_id: String,
}

#[tauri::command]
pub fn get_obs_settings_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<ObsConnectorSettings, String> {
    crate::commands::roles::require_perm(&state, "settings.integrations")?;
    let conn = state.conn.lock();
    get_obs_settings(&conn).map_err(map_err)
}

#[tauri::command]
pub fn update_obs_settings_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: UpdateObsSettingsInput,
) -> Result<ObsConnectorSettings, String> {
    crate::commands::roles::require_perm(&state, "settings.integrations")?;
    let conn = state.conn.lock();
    let mut current = get_obs_settings(&conn).map_err(map_err)?;
    if let Some(v) = input.datadog_api_key {
        current.datadog_api_key = if v.is_empty() { None } else { Some(v) };
    }
    if let Some(v) = input.datadog_site {
        current.datadog_site = if v.is_empty() { None } else { Some(v) };
    }
    if let Some(v) = input.grafana_url {
        current.grafana_url = if v.is_empty() { None } else { Some(v) };
    }
    if let Some(v) = input.cloudwatch_region {
        current.cloudwatch_region = if v.is_empty() { None } else { Some(v) };
    }
    if let Some(v) = input.outbound_enabled {
        current.outbound_enabled = v;
    }
    if let Some(v) = input.default_company_id {
        current.default_company_id = if v.is_empty() { None } else { Some(v) };
    }
    update_obs_settings(&conn, &current).map_err(map_err)?;
    Ok(current)
}

#[tauri::command]
pub fn import_obs_alert_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    input: ImportObsAlertInput,
) -> Result<ImportObsAlertResult, String> {
    crate::commands::roles::require_perm(&state, "settings.integrations")?;
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    let (alert, ticket_id) = import_alert(
        &conn,
        &ws,
        &input.source,
        &input.payload_json,
        input.company_id.as_deref(),
    )
    .map_err(map_err)?;
    Ok(ImportObsAlertResult { alert, ticket_id })
}

#[tauri::command]
pub fn list_obs_alerts_cmd(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<ObsAlert>, String> {
    let ws = workspace_id(&state)?;
    let conn = state.conn.lock();
    list_alerts(&conn, &ws).map_err(map_err)
}

#[tauri::command]
pub fn list_obs_alerts_for_ticket_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    ticket_id: String,
) -> Result<Vec<ObsAlert>, String> {
    let conn = state.conn.lock();
    list_alerts_for_ticket(&conn, &ticket_id).map_err(map_err)
}

/// Emit outbound observability events for a resolved/closed ticket (Datadog Events).
pub fn maybe_emit_outbound_for_ticket(state: &AppState, ticket_id: &str, status: &str) {
    let status = status.to_lowercase();
    if status != "resolved" && status != "closed" {
        return;
    }
    let conn = state.conn.lock();
    let Ok(settings) = get_obs_settings(&conn) else {
        return;
    };
    if !settings.outbound_enabled {
        return;
    }
    let Some(api_key) = settings.datadog_api_key.filter(|k| !k.is_empty()) else {
        return;
    };
    let Ok(Some(ticket)) = get_ticket(&conn, ticket_id) else {
        return;
    };
    let site = settings
        .datadog_site
        .unwrap_or_else(|| "datadoghq.com".into());
    let title = format!("Confluity ticket {status}: {}", ticket.title);
    let text = format!(
        "Ticket `{}` marked **{}** in Confluity.\n\n{}",
        ticket.id,
        status,
        ticket.description.unwrap_or_default()
    );
    drop(conn);
    let _ = emit_datadog_event_blocking(&api_key, &site, &title, &text);
}
