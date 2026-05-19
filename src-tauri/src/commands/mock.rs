use tauri::command;
use tonic::transport::Channel;
use crate::commands::worker::pb;
use pb::mock_service_client::MockServiceClient;
use pb::proxy_service_client::ProxyServiceClient;
use pb::{MockRule, TrafficRequest};
use serde::Serialize;

#[derive(Serialize)]
pub struct LocalMockRule {
    pub id: String,
    pub path: String,
    pub method: String,
    pub status_code: i32,
    pub body: String,
    pub headers: std::collections::HashMap<String, String>,
    pub content_type: String,
}

#[command]
pub async fn add_mock_rule(
    id: String,
    path: String,
    method: String,
    status_code: i32,
    body: String,
    headers: std::collections::HashMap<String, String>,
    content_type: String,
) -> Result<(), String> {
    let channel = Channel::from_static("http://127.0.0.1:50151")
        .connect()
        .await
        .map_err(|e| e.to_string())?;

    let mut client = MockServiceClient::new(channel);
    let request = tonic::Request::new(MockRule {
        id,
        path,
        method,
        status_code,
        body,
        headers,
        content_type,
    });

    client.add_rule(request).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub async fn remove_mock_rule(id: String) -> Result<(), String> {
    let channel = Channel::from_static("http://127.0.0.1:50151")
        .connect()
        .await
        .map_err(|e| e.to_string())?;

    let mut client = MockServiceClient::new(channel);
    let request = tonic::Request::new(pb::TargetIdRequest { id });

    client.remove_rule(request).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub async fn list_mock_rules() -> Result<Vec<LocalMockRule>, String> {
    let channel = Channel::from_static("http://127.0.0.1:50151")
        .connect()
        .await
        .map_err(|e| e.to_string())?;

    let mut client = MockServiceClient::new(channel);
    let request = tonic::Request::new(pb::GenericRequest {});

    let response = client.list_rules(request).await.map_err(|e| e.to_string())?;
    let rules = response.into_inner().rules.into_iter().map(|r| LocalMockRule {
        id: r.id,
        path: r.path,
        method: r.method,
        status_code: r.status_code,
        body: r.body,
        headers: r.headers,
        content_type: r.content_type,
    }).collect();
    
    Ok(rules)
}

#[command]
pub async fn mock_import_from_traffic(limit: i32) -> Result<Vec<LocalMockRule>, String> {
    let channel = Channel::from_static("http://127.0.0.1:50151")
        .connect()
        .await
        .map_err(|e| e.to_string())?;

    let mut proxy_client = ProxyServiceClient::new(channel.clone());
    let traffic = proxy_client
        .get_traffic(tonic::Request::new(TrafficRequest { limit }))
        .await
        .map_err(|e| e.to_string())?;

    let mut mock_client = MockServiceClient::new(channel);
    let mut created = vec![];

    for entry in traffic.into_inner().entries {
        let path = extract_url_path(&entry.url);
        let id = format!("rec-{}", entry.id);
        let rule = MockRule {
            id: id.clone(),
            path: path.clone(),
            method: entry.method.clone(),
            status_code: entry.status_code,
            body: entry.preview.clone(),
            headers: entry.response_headers.clone(),
            content_type: entry.content_type.clone(),
        };
        mock_client
            .add_rule(tonic::Request::new(rule))
            .await
            .map_err(|e| e.to_string())?;
        created.push(LocalMockRule {
            id,
            path,
            method: entry.method,
            status_code: entry.status_code,
            body: entry.preview,
            headers: entry.response_headers,
            content_type: entry.content_type,
        });
    }

    Ok(created)
}

fn extract_url_path(url: &str) -> String {
    // Parse out just the path (and query) from a full URL like https://host/path?query
    if let Some(after_scheme) = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://")) {
        if let Some(slash) = after_scheme.find('/') {
            return after_scheme[slash..].to_string();
        }
        return "/".to_string();
    }
    if url.starts_with('/') {
        return url.to_string();
    }
    "/".to_string()
}
