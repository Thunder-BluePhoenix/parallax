// Parallax Core Library
pub mod http_engine;
pub mod auth_providers;
pub mod schema_explorer;
pub mod commands;

use tauri_plugin_shell::ShellExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            // HTTP Engine
            commands::http::send_request,
            commands::http::cancel_request,
            // Persistence
            commands::collections::list_collections,
            commands::collections::load_collection,
            commands::collections::save_collection,
            commands::collections::delete_collection,
            commands::collections::save_history_entry,
            // Environments
            commands::environments::list_environments,
            commands::environments::load_environment,
            commands::environments::save_environment,
            // Auth Providers
            commands::auth::detect_framework,
            commands::auth::perform_auth,
            commands::auth::refresh_auth,
            // Schema / Ecosystem Explorer
            commands::schema::explore_schema,
            commands::schema::list_frameworks,
            // Workspace
            commands::workspace::open_workspace,
            commands::workspace::get_workspace_info,
        ])
        .setup(|app| {
            // Start Go sidecar worker (non-fatal in dev if binary not yet compiled)
            match app.shell().sidecar("parallax-worker") {
                Ok(sidecar_command) => {
                    let args: Vec<&str> = vec!["--grpc-port", "50151"];
                    match sidecar_command.args(args).spawn() {
                        Ok((_rx, _child)) => {
                            println!("[Parallax] Go sidecar started on grpc :50151");
                        }
                        Err(e) => {
                            eprintln!("[Parallax] Go sidecar failed to start: {e} (run: make build-worker)");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[Parallax] Go sidecar not found: {e} (Dashboard features unavailable)");
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Parallax");
}
