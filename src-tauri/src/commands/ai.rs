use tauri::command;
use tonic::transport::Channel;
use crate::commands::worker::pb;
use pb::ai_service_client::AiServiceClient;
use pb::AiTestRequest;
use std::collections::HashMap;

#[command]
pub async fn ai_generate_tests(
    config: serde_json::Value,
    request_id: String,
    method: String,
    url: String,
    response_body: String,
    response_status: i32,
    response_headers: HashMap<String, String>,
) -> Result<serde_json::Value, String> {
    let channel = Channel::from_static("http://127.0.0.1:50151")
        .connect()
        .await
        .map_err(|e| e.to_string())?;

    let mut client = AiServiceClient::new(channel);

    let request = tonic::Request::new(AiTestRequest {
        request_id,
        method,
        url,
        response_body,
        response_status,
        response_headers,
        model: config["model"].as_str().unwrap_or("").to_string(),
        provider: config["provider"].as_str().unwrap_or("").to_string(),
        api_key: config["apiKey"].as_str().unwrap_or("").to_string(),
        base_url: config["baseUrl"].as_str().unwrap_or("").to_string(),
    });

    let response = client.generate_tests(request).await.map_err(|e| e.to_string())?;
    let res = response.into_inner();

    Ok(serde_json::json!({
        "js": res.tests_js,
        "yaml": res.tests_yaml
    }))
}
