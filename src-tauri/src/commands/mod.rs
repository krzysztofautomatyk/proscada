//! Tauri command surface — narrow, audited API.

use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use crate::audit::{AuditEntry, AuditStatus};
use crate::engine::{
    AlarmInstance, Engine, EngineSnapshot, TagValue, UserAccountInput, WriteReceipt,
};
use crate::modbus;
use crate::project::{water_tank_project, ScadaProject, UserSummary};

pub struct AppState {
    pub engine: Arc<Engine>,
}

#[derive(Serialize)]
pub struct StatusMsg {
    pub ok: bool,
    pub message: String,
}

#[tauri::command]
pub fn get_builtin_water_tank() -> ScadaProject {
    let mut project = water_tank_project();
    project.users.clear();
    project.recompute_hash();
    project
}

#[tauri::command]
pub fn load_project(state: State<'_, AppState>, project: ScadaProject) -> Result<(), String> {
    state.engine.authorize_project_load()?;
    state.engine.load_project(project)
}

#[tauri::command]
pub fn load_builtin_water_tank(state: State<'_, AppState>) -> Result<ScadaProject, String> {
    state.engine.authorize_project_load()?;
    let p = water_tank_project();
    state.engine.load_builtin_preserving_users(p)?;
    state
        .engine
        .get_project_redacted()
        .ok_or_else(|| "Built-in project missing after load".into())
}

#[tauri::command]
pub fn get_project(state: State<'_, AppState>) -> Option<ScadaProject> {
    state.engine.get_project_redacted()
}

#[tauri::command]
pub fn save_project_in_memory(
    state: State<'_, AppState>,
    project: ScadaProject,
) -> Result<ScadaProject, String> {
    state.engine.set_project_mut(project)?;
    state
        .engine
        .get_project_redacted()
        .ok_or_else(|| "Project missing after save".into())
}

#[tauri::command]
pub fn save_project_file(state: State<'_, AppState>, path: String) -> Result<(), String> {
    state.engine.save_project_file(&path)
}

#[tauri::command]
pub fn get_snapshot(state: State<'_, AppState>) -> EngineSnapshot {
    // Session expiry is evaluated explicitly here; reading the snapshot itself
    // must not be a state-changing operation.
    state.engine.expire_idle_session();
    state.engine.snapshot()
}

#[tauri::command]
pub fn start_polling(state: State<'_, AppState>, device_id: Option<String>) -> Result<(), String> {
    state.engine.start_polling(device_id)
}

#[tauri::command]
pub fn stop_polling(state: State<'_, AppState>) -> Result<(), String> {
    state.engine.stop_polling()
}

#[tauri::command]
pub async fn write_tag(
    state: State<'_, AppState>,
    tag_id: String,
    value: f64,
    pin: Option<String>,
) -> Result<WriteReceipt, String> {
    state
        .engine
        .write_tag_with_pin(&tag_id, value, pin.as_deref())
        .await
}

#[tauri::command]
pub fn ack_alarm(state: State<'_, AppState>, def_id: String) -> Result<(), String> {
    state.engine.ack_alarm(&def_id)
}

#[tauri::command]
pub fn set_mode(state: State<'_, AppState>, mode: String) -> Result<(), String> {
    state.engine.set_mode(mode)
}

#[tauri::command]
pub fn login(
    state: State<'_, AppState>,
    username: String,
    password: Option<String>,
) -> Result<UserSummary, String> {
    state.engine.login(&username, password.as_deref())
}

#[tauri::command]
pub fn logout(state: State<'_, AppState>) -> Result<(), String> {
    state.engine.logout()
}

#[tauri::command]
pub fn change_password(
    state: State<'_, AppState>,
    current_password: String,
    new_password: String,
) -> Result<UserSummary, String> {
    state
        .engine
        .change_password(&current_password, &new_password)
}

#[tauri::command]
pub fn bootstrap_admin(
    state: State<'_, AppState>,
    password: String,
) -> Result<UserSummary, String> {
    state.engine.bootstrap_admin(&password)
}

#[tauri::command]
pub fn list_users(state: State<'_, AppState>) -> Result<Vec<UserSummary>, String> {
    state.engine.list_users()
}

#[tauri::command]
pub fn save_user(
    state: State<'_, AppState>,
    user: UserAccountInput,
) -> Result<UserSummary, String> {
    state.engine.save_user(user)
}

#[tauri::command]
pub fn delete_user(state: State<'_, AppState>, user_id: String) -> Result<(), String> {
    state.engine.delete_user(&user_id)
}

#[tauri::command]
pub fn get_audit(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<AuditEntry>, String> {
    state.engine.authorize_audit_read()?;
    Ok(state.engine.audit().list(limit.unwrap_or(200)))
}

#[tauri::command]
pub fn verify_audit(state: State<'_, AppState>) -> Result<bool, String> {
    state.engine.authorize_audit_read()?;
    Ok(state.engine.audit().verify_chain())
}

#[tauri::command]
pub fn get_audit_status(state: State<'_, AppState>) -> Result<AuditStatus, String> {
    state.engine.authorize_audit_status()?;
    Ok(state.engine.audit().status_redacted())
}

#[tauri::command]
pub async fn test_device(
    state: State<'_, AppState>,
    device_id: String,
) -> Result<StatusMsg, String> {
    // Only a validated, canonical project device can be tested. The webview
    // cannot turn this command into an arbitrary host/port probe.
    let cfg = state.engine.device_connection_config_for_test(&device_id)?;
    let result = state
        .engine
        .runtime()
        .spawn(async move { modbus::test_connection(&cfg).await })
        .await
        .map_err(|e| format!("test task join: {e}"))?;
    match result {
        Ok(()) => Ok(StatusMsg {
            ok: true,
            message: "Transport reachable; no Modbus data was read".into(),
        }),
        Err(e) => Ok(StatusMsg {
            ok: false,
            message: format!("Transport unavailable: {e}"),
        }),
    }
}

#[tauri::command]
pub fn get_tag_values(state: State<'_, AppState>) -> Vec<TagValue> {
    state.engine.expire_idle_session();
    state.engine.snapshot().tags
}

#[tauri::command]
pub fn get_alarms(state: State<'_, AppState>) -> Vec<AlarmInstance> {
    state.engine.expire_idle_session();
    state.engine.snapshot().alarms
}
