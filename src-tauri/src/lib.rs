mod audit;
mod commands;
mod engine;
mod modbus;
mod project;

use std::sync::Arc;

use audit::AuditLog;
use commands::AppState;
use engine::Engine;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter("info,proscada=debug")
        .init();

    let audit = Arc::new(AuditLog::new());
    audit.append("system", "system", "app.start", "ProScada core online");
    let engine = Arc::new(Engine::new(audit));

    // Auto-load Water Tank for first-run experience
    let _ = engine.load_project(project::water_tank_project());

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState { engine })
        .invoke_handler(tauri::generate_handler![
            commands::get_builtin_water_tank,
            commands::load_project,
            commands::load_builtin_water_tank,
            commands::get_project,
            commands::save_project_in_memory,
            commands::get_snapshot,
            commands::start_polling,
            commands::stop_polling,
            commands::write_tag,
            commands::ack_alarm,
            commands::set_role,
            commands::set_mode,
            commands::get_audit,
            commands::verify_audit,
            commands::test_device,
            commands::get_tag_values,
            commands::get_alarms,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ProScada");
}
