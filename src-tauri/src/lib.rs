// Parallax Core Library
pub mod http_engine;
pub mod auth_providers;
pub mod schema_explorer;
pub mod commands;

use std::collections::HashMap;
use std::sync::Mutex;
use tauri_plugin_shell::ShellExt;
use commands::websocket::new_ws_state;
use commands::sse::new_sse_state;

/// Shared Tauri application state
pub struct AppState {
    /// In-flight request abort handles keyed by request ID
    pub pending_requests: Mutex<HashMap<String, tokio::task::AbortHandle>>,
}

impl Default for AppState {
    fn default() -> Self { Self::new() }
}

impl AppState {
    pub fn new() -> Self {
        Self { pending_requests: Mutex::new(HashMap::new()) }
    }
}

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
            // gRPC
            commands::grpc::grpc_unary,
            commands::grpc::grpc_server_stream,
            commands::grpc::grpc_reflect,
            // File utilities
            commands::workspace::read_file_for_template,
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
            commands::environments::load_globals,
            commands::environments::save_globals,
            // Auth Providers
            commands::auth::detect_framework,
            commands::auth::perform_auth,
            commands::auth::refresh_auth,
            // Ecosystem Schema Explorer (Phase 4.1 & 4.2)
            commands::schema::list_frameworks,
            commands::schema::explore_schema,
            commands::schema::save_design_spec,
            // Workspace
            commands::workspace::open_workspace,
            commands::workspace::create_workspace,
            commands::workspace::get_workspace_info,
            commands::workspace::get_current_branch,
            // Worker
            commands::worker::ping_worker,
            // WebSocket
            commands::websocket::ws_connect,
            commands::websocket::ws_send,
            commands::websocket::ws_disconnect,
            // SSE
            commands::sse::sse_connect,
            commands::sse::sse_disconnect,
            // Load Test
            commands::loadtest::run_load_test,
            commands::mock::add_mock_rule,
            commands::mock::remove_mock_rule,
            commands::mock::list_mock_rules,
            commands::mock::mock_import_from_traffic,
            // Dashboard (Proxy & Health)
            commands::proxy::start_proxy_stream,
            commands::proxy::get_proxy_traffic,
            commands::proxy::clear_proxy_traffic,
            commands::proxy::set_proxy_filter,
            commands::health::start_health_stream,
            commands::health::add_health_target,
            commands::health::remove_health_target,
            commands::health::get_health_statuses,
            commands::watcher::watch_workspace,
            commands::watcher::unwatch_workspace,
            commands::ai::ai_generate_tests,
            commands::ai::ai_repair_request,
            commands::ai::ai_generate_script,
            commands::ai::ai_create_collection,
            // Git (Phase 2.5)
            commands::git::git_init,
            commands::git::git_status,
            commands::git::git_commit,
            commands::git::git_push,
            commands::git::git_pull,
            commands::git::git_stash,
            commands::git::git_stash_pop,
            commands::git::git_branches,
            commands::git::git_create_branch,
            commands::git::git_checkout_branch,
            commands::git::git_log,
            commands::git::git_diff,
            commands::git::git_set_remote,
            // GitHub OAuth + API (Phase 2.5)
            commands::github::github_device_auth_start,
            commands::github::github_device_auth_poll,
            commands::github::github_get_identity,
            commands::github::github_sign_out,
            commands::github::github_list_collaborators,
            commands::github::github_invite_collaborator,
            commands::github::github_remove_collaborator,
            commands::github::github_publish_docs,
            // Chat bridge (Phase 2.5)
            commands::chat::chat_send_message,
            commands::chat::chat_get_history,
            commands::chat::chat_set_presence,
            commands::chat::chat_get_presence,
            commands::chat::chat_start_stream,
        ])
        .manage(AppState::new())
        .manage(new_ws_state())
        .manage(new_sse_state())
        .setup(|app| {
            // Start Go sidecar worker (non-fatal in dev if binary not yet compiled)
            match app.shell().sidecar("parallax-worker") {
                Ok(sidecar_command) => {
                    let args: Vec<&str> = vec!["--grpc-port", "50151"];
                    match sidecar_command.args(args).spawn() {
                        Ok((mut rx, _child)) => {
                            println!("[Parallax] Go sidecar started on grpc :50151");
                            
                            // Pipe sidecar output to terminal
                            tauri::async_runtime::spawn(async move {
                                while let Some(event) = rx.recv().await {
                                    if let tauri_plugin_shell::process::CommandEvent::Stdout(line) = event {
                                        println!("[parallax-worker] {}", String::from_utf8_lossy(&line).trim());
                                    } else if let tauri_plugin_shell::process::CommandEvent::Stderr(line) = event {
                                        eprintln!("[parallax-worker-err] {}", String::from_utf8_lossy(&line).trim());
                                    }
                                }
                            });
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
