mod audit;
mod commands;
mod engine;
mod modbus;
mod project;

use std::sync::Arc;

use audit::AuditLog;
use commands::AppState;
use engine::Engine;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter("info,proscada=debug")
        .init();

    let audit = Arc::new(AuditLog::new());
    audit.append("system", "system", "app.start", "ProScada core online");
    let engine = Arc::new(Engine::new(audit.clone()));

    // Auto-load Water Tank for first-run experience. The engine still holds no
    // privileges until a user signs in.
    if let Err(error) = engine.load_builtin(project::water_tank_project()) {
        tracing::error!("failed to load the built-in Water Tank project: {error}");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
            // The audit trail must survive a restart, so attach the durable
            // sink as soon as the app data directory is resolvable.
            match app.path().app_data_dir() {
                Ok(dir) => {
                    let path = dir.join("audit.jsonl");
                    match audit.attach_sink(&path) {
                        Ok(recovered) => {
                            tracing::info!(
                                "audit trail persisted to {} ({recovered} entries recovered)",
                                path.display()
                            );
                        }
                        Err(error) => {
                            tracing::error!("audit trail is memory-only: {error}");
                        }
                    }
                }
                Err(error) => tracing::error!("audit trail is memory-only: {error}"),
            }
            Ok(())
        })
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
            commands::set_mode,
            commands::login,
            commands::logout,
            commands::change_password,
            commands::verify_pin,
            commands::list_users,
            commands::save_user,
            commands::delete_user,
            commands::get_audit,
            commands::verify_audit,
            commands::get_audit_status,
            commands::test_device,
            commands::get_tag_values,
            commands::get_alarms,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ProScada");
}
