use tauri::command;
use tonic::transport::Channel;
use crate::commands::worker::pb;
use pb::mock_service_client::MockServiceClient;
use pb::MockRule;

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
pub async fn list_mock_rules() -> Result<Vec<MockRule>, String> {
    let channel = Channel::from_static("http://127.0.0.1:50151")
        .connect()
        .await
        .map_err(|e| e.to_string())?;

    let mut client = MockServiceClient::new(channel);
    let request = tonic::Request::new(pb::GenericRequest {});

    let response = client.list_rules(request).await.map_err(|e| e.to_string())?;
    Ok(response.into_inner().rules)
}
