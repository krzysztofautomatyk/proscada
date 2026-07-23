//! Tauri command surface — narrow, audited API.

use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use crate::engine::{AlarmInstance, Engine, EngineSnapshot, TagValue};
use crate::project::{Role, ScadaProject, water_tank_project};
use crate::audit::AuditEntry;
use crate::modbus::{self, ConnectionConfig};

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
    water_tank_project()
}

#[tauri::command]
pub fn load_project(state: State<'_, AppState>, project: ScadaProject) -> Result<(), String> {
    state.engine.load_project(project)
}

#[tauri::command]
pub fn load_builtin_water_tank(state: State<'_, AppState>) -> Result<ScadaProject, String> {
    let p = water_tank_project();
    state.engine.load_project(p.clone())?;
    Ok(p)
}

#[tauri::command]
pub fn get_project(state: State<'_, AppState>) -> Option<ScadaProject> {
    state.engine.get_project()
}

#[tauri::command]
pub fn save_project_in_memory(
    state: State<'_, AppState>,
    project: ScadaProject,
) -> Result<ScadaProject, String> {
    state.engine.set_project_mut(project)?;
    state
        .engine
        .get_project()
        .ok_or_else(|| "Project missing after save".into())
}

#[tauri::command]
pub fn get_snapshot(state: State<'_, AppState>) -> EngineSnapshot {
    state.engine.snapshot()
}

#[tauri::command]
pub fn start_polling(
    state: State<'_, AppState>,
    device_id: Option<String>,
) -> Result<(), String> {
    state.engine.start_polling(device_id)
}

#[tauri::command]
pub fn stop_polling(state: State<'_, AppState>) {
    state.engine.stop_polling();
}

#[tauri::command]
pub async fn write_tag(
    state: State<'_, AppState>,
    tag_id: String,
    value: f64,
) -> Result<(), String> {
    state.engine.write_tag(&tag_id, value).await
}

#[tauri::command]
pub fn ack_alarm(state: State<'_, AppState>, def_id: String) -> Result<(), String> {
    state.engine.ack_alarm(&def_id)
}

#[tauri::command]
pub fn set_role(state: State<'_, AppState>, role: Role, actor: String) {
    state.engine.set_role(role, actor);
}

#[tauri::command]
pub fn set_mode(state: State<'_, AppState>, mode: String) {
    state.engine.set_mode(mode);
}

#[tauri::command]
pub fn get_audit(state: State<'_, AppState>, limit: Option<usize>) -> Vec<AuditEntry> {
    state.engine.audit().list(limit.unwrap_or(200))
}

#[tauri::command]
pub fn verify_audit(state: State<'_, AppState>) -> bool {
    state.engine.audit().verify_chain()
}

#[tauri::command]
pub async fn test_device(
    state: State<'_, AppState>,
    host: String,
    port: u16,
    unit_id: u8,
    timeout_ms: u64,
) -> Result<StatusMsg, String> {
    // Reuse engine runtime so test works even when invoked from mixed contexts.
    let cfg = ConnectionConfig {
        host,
        port,
        unit_id,
        timeout_ms,
    };
    let result = state
        .engine
        .runtime()
        .spawn(async move { modbus::test_connection(&cfg).await })
        .await
        .map_err(|e| format!("test task join: {e}"))?;
    match result {
        Ok(()) => Ok(StatusMsg {
            ok: true,
            message: "Connection OK".into(),
        }),
        Err(e) => Ok(StatusMsg {
            ok: false,
            message: e.to_string(),
        }),
    }
}

#[tauri::command]
pub fn get_tag_values(state: State<'_, AppState>) -> Vec<TagValue> {
    state.engine.snapshot().tags
}

#[tauri::command]
pub fn get_alarms(state: State<'_, AppState>) -> Vec<AlarmInstance> {
    state.engine.snapshot().alarms
}
