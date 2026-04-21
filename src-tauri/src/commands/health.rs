use tauri::{command, AppHandle, Emitter};
use tonic::transport::Channel;
use std::time::Duration;
use crate::commands::worker::pb;
use pb::health_service_client::HealthServiceClient;
use pb::{GenericRequest, TargetIdRequest, ServiceTarget};
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct HealthEvent {
    pub id: String,
    pub name: String,
    pub url: String,
    pub status: String,
    pub latency_ms: f64,
    pub status_code: i32,
    pub last_checked: i64,
    pub error_msg: String,
}

#[command]
pub async fn start_health_stream(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn(async move {
        let channel = match Channel::from_static("http://127.0.0.1:50151")
            .timeout(Duration::from_secs(5))
            .connect()
            .await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to connect to health gRPC: {}", e);
                    return;
                }
            };

        let mut client = HealthServiceClient::new(channel);
        
        let mut stream = match client.watch_statuses(tonic::Request::new(GenericRequest {})).await {
            Ok(s) => s.into_inner(),
            Err(e) => {
                eprintln!("Failed to start watch_statuses stream: {}", e);
                return;
            }
        };

        while let Ok(Some(st)) = stream.message().await {
            let event = HealthEvent {
                id: st.id,
                name: st.name,
                url: st.url,
                status: st.status,
                latency_ms: st.latency_ms,
                status_code: st.status_code,
                last_checked: st.last_checked,
                error_msg: st.error_msg,
            };
            let _ = app.emit("health_status_event", event);
        }
    });

    Ok(())
}

#[command]
pub async fn add_health_target(id: String, name: String, url: String, interval_sec: i32, timeout_ms: i32) -> Result<(), String> {
    let channel = Channel::from_static("http://127.0.0.1:50151")
        .timeout(Duration::from_secs(2))
        .connect()
        .await
        .map_err(|e| e.to_string())?;

    let mut client = HealthServiceClient::new(channel);
    client.add_target(tonic::Request::new(ServiceTarget {
        id, name, url, interval_sec, timeout_ms
    })).await.map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub async fn remove_health_target(id: String) -> Result<(), String> {
    let channel = Channel::from_static("http://127.0.0.1:50151")
        .timeout(Duration::from_secs(2))
        .connect()
        .await
        .map_err(|e| e.to_string())?;

    let mut client = HealthServiceClient::new(channel);
    client.remove_target(tonic::Request::new(TargetIdRequest { id })).await.map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub async fn get_health_statuses() -> Result<Vec<HealthEvent>, String> {
    let channel = Channel::from_static("http://127.0.0.1:50151")
        .timeout(Duration::from_secs(2))
        .connect()
        .await
        .map_err(|e| e.to_string())?;

    let mut client = HealthServiceClient::new(channel);
    let res = client.get_statuses(tonic::Request::new(GenericRequest {})).await.map_err(|e| e.to_string())?;
    
    let entries = res.into_inner().statuses.into_iter().map(|st| HealthEvent {
        id: st.id,
        name: st.name,
        url: st.url,
        status: st.status,
        latency_ms: st.latency_ms,
        status_code: st.status_code,
        last_checked: st.last_checked,
        error_msg: st.error_msg,
    }).collect();

    Ok(entries)
}
