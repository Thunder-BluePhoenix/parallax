use tauri::{command, AppHandle, Emitter};
use tonic::transport::Channel;
use crate::commands::worker::pb;
use pb::watcher_service_client::WatcherServiceClient;
use pb::{GenericRequest, WatchRequest};
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct FileChangeEvent {
    pub path: String,
    pub operation: String,
    pub is_collection: bool,
    pub is_environment: bool,
}

#[command]
pub async fn watch_workspace(app: AppHandle, workspace_path: String) -> Result<(), String> {
    tauri::async_runtime::spawn(async move {
        let channel = match Channel::from_static("http://127.0.0.1:50151")
            .connect()
            .await
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to connect to watcher gRPC: {}", e);
                return;
            }
        };

        let mut client = WatcherServiceClient::new(channel);
        let mut stream = match client
            .watch_workspace(tonic::Request::new(WatchRequest {
                workspace_path: workspace_path.clone(),
            }))
            .await
        {
            Ok(s) => s.into_inner(),
            Err(e) => {
                eprintln!("Failed to start watch_workspace stream: {}", e);
                return;
            }
        };

        while let Ok(Some(event)) = stream.message().await {
            let _ = app.emit(
                "workspace_file_changed",
                FileChangeEvent {
                    path: event.path,
                    operation: event.operation,
                    is_collection: event.is_collection,
                    is_environment: event.is_environment,
                },
            );
        }
    });

    Ok(())
}

#[command]
pub async fn unwatch_workspace(workspace_path: String) -> Result<(), String> {
    let channel = Channel::from_static("http://127.0.0.1:50151")
        .connect()
        .await
        .map_err(|e| e.to_string())?;

    let mut client = WatcherServiceClient::new(channel);
    client
        .unwatch_workspace(tonic::Request::new(WatchRequest { workspace_path }))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
