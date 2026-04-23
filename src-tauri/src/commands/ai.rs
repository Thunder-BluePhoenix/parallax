use tauri::command;
use tonic::transport::Channel;
use crate::commands::worker::pb;
use pb::ai_service_client::AiServiceClient;
use pb::{AiTestRequest, AiRepairRequest, AiScriptRequest, AiCollectionRequest, AiConfig};
use std::collections::HashMap;

fn channel_url() -> &'static str {
    "http://127.0.0.1:50151"
}

async fn ai_client() -> Result<AiServiceClient<Channel>, String> {
    let channel = Channel::from_static(channel_url())
        .connect()
        .await
        .map_err(|e| e.to_string())?;
    Ok(AiServiceClient::new(channel))
}

fn extract_config(config: &serde_json::Value) -> AiConfig {
    AiConfig {
        provider: config["provider"].as_str().unwrap_or("ollama").to_string(),
        model:    config["model"].as_str().unwrap_or("").to_string(),
        api_key:  config["apiKey"].as_str().unwrap_or("").to_string(),
        base_url: config["baseUrl"].as_str().unwrap_or("").to_string(),
        air_gap_mode: config["airGapMode"].as_bool().unwrap_or(false),
    }
}

/// Generate pm.test() + YAML assertions from a response.
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
    let mut client = ai_client().await?;

    let req = tonic::Request::new(AiTestRequest {
        request_id,
        method,
        url,
        response_body,
        response_status,
        response_headers,
        model:    config["model"].as_str().unwrap_or("").to_string(),
        provider: config["provider"].as_str().unwrap_or("").to_string(),
        api_key:  config["apiKey"].as_str().unwrap_or("").to_string(),
        base_url: config["baseUrl"].as_str().unwrap_or("").to_string(),
    });

    let res = client.generate_tests(req).await.map_err(|e| e.to_string())?.into_inner();
    Ok(serde_json::json!({ "js": res.tests_js, "yaml": res.tests_yaml }))
}

/// Analyze a 4xx/5xx response and suggest request fixes.
#[command]
pub async fn ai_repair_request(
    config: serde_json::Value,
    method: String,
    url: String,
    request_headers: HashMap<String, String>,
    request_body: String,
    response_status: i32,
    response_body: String,
    env_keys: Vec<String>,
) -> Result<serde_json::Value, String> {
    let mut client = ai_client().await?;

    let req = tonic::Request::new(AiRepairRequest {
        config: Some(extract_config(&config)),
        method,
        url,
        request_headers,
        request_body,
        response_status,
        response_body,
        env_keys,
    });

    let res = client.repair_request(req).await.map_err(|e| e.to_string())?.into_inner();
    Ok(serde_json::json!({
        "diagnosis": res.diagnosis,
        "fixes":     serde_json::from_str::<serde_json::Value>(&res.fixes_json).unwrap_or(serde_json::json!([])),
        "priority":  res.priority
    }))
}

/// Generate a pre-request or test script from natural language.
#[command]
pub async fn ai_generate_script(
    config: serde_json::Value,
    script_type: String,
    user_prompt: String,
    method: String,
    url: String,
) -> Result<String, String> {
    let mut client = ai_client().await?;

    let req = tonic::Request::new(AiScriptRequest {
        config: Some(extract_config(&config)),
        script_type,
        user_prompt,
        method,
        url,
    });

    let res = client.generate_script(req).await.map_err(|e| e.to_string())?.into_inner();
    Ok(res.script)
}

/// Create a full collection YAML from a natural language description.
#[command]
pub async fn ai_create_collection(
    config: serde_json::Value,
    description: String,
) -> Result<String, String> {
    let mut client = ai_client().await?;

    let req = tonic::Request::new(AiCollectionRequest {
        config: Some(extract_config(&config)),
        description,
    });

    let res = client.create_collection(req).await.map_err(|e| e.to_string())?.into_inner();
    Ok(res.collection_yaml)
}
