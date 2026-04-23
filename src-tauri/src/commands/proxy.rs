use tauri::{command, AppHandle, Emitter};
use tonic::transport::Channel;
use std::time::Duration;
use crate::commands::worker::pb;
use pb::proxy_service_client::ProxyServiceClient;
use pb::{GenericRequest, TrafficRequest, TrafficFilter};
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct TrafficEvent {
    pub id: String,
    pub timestamp_ms: i64,
    pub method: String,
    pub url: String,
    pub status_code: i32,
    pub latency_ms: f64,
    pub content_type: String,
    pub preview: String,
}

#[command]
pub async fn start_proxy_stream(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn(async move {
        let channel = match Channel::from_static("http://127.0.0.1:50151")
            .connect()
            .await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to connect to proxy gRPC: {}", e);
                    return;
                }
            };

        let mut client = ProxyServiceClient::new(channel);
        
        let mut stream = match client.watch_traffic(tonic::Request::new(GenericRequest {})).await {
            Ok(s) => s.into_inner(),
            Err(e) => {
                eprintln!("Failed to start watch_traffic stream: {}", e);
                return;
            }
        };

        while let Ok(Some(entry)) = stream.message().await {
            let event = TrafficEvent {
                id: entry.id,
                timestamp_ms: entry.timestamp_ms,
                method: entry.method,
                url: entry.url,
                status_code: entry.status_code,
                latency_ms: entry.latency_ms,
                content_type: entry.content_type,
                preview: entry.preview,
            };
            let _ = app.emit("proxy_traffic_event", event);
        }
    });

    Ok(())
}

#[command]
pub async fn get_proxy_traffic(limit: i32) -> Result<Vec<TrafficEvent>, String> {
    let channel = Channel::from_static("http://127.0.0.1:50151")
        .timeout(Duration::from_secs(2))
        .connect()
        .await
        .map_err(|e| e.to_string())?;

    let mut client = ProxyServiceClient::new(channel);
    let res = client.get_traffic(tonic::Request::new(TrafficRequest { limit })).await.map_err(|e| e.to_string())?;
    
    let entries = res.into_inner().entries.into_iter().map(|entry| TrafficEvent {
        id: entry.id,
        timestamp_ms: entry.timestamp_ms,
        method: entry.method,
        url: entry.url,
        status_code: entry.status_code,
        latency_ms: entry.latency_ms,
        content_type: entry.content_type,
        preview: entry.preview,
    }).collect();

    Ok(entries)
}

#[command]
pub async fn clear_proxy_traffic() -> Result<(), String> {
    let channel = Channel::from_static("http://127.0.0.1:50151")
        .timeout(Duration::from_secs(2))
        .connect()
        .await
        .map_err(|e| e.to_string())?;

    let mut client = ProxyServiceClient::new(channel);
    client.clear_traffic(tonic::Request::new(GenericRequest {})).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub async fn set_proxy_filter(
    include_domains: Vec<String>,
    exclude_domains: Vec<String>,
    only_methods: Vec<String>,
    min_status: i32,
) -> Result<(), String> {
    let channel = Channel::from_static("http://127.0.0.1:50151")
        .timeout(Duration::from_secs(2))
        .connect()
        .await
        .map_err(|e| e.to_string())?;

    let mut client = ProxyServiceClient::new(channel);
    client.set_filter(tonic::Request::new(TrafficFilter {
        include_domains,
        exclude_domains,
        only_methods,
        min_status,
    })).await.map_err(|e| e.to_string())?;
    Ok(())
}
