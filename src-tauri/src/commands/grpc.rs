/// gRPC unary call command
/// Uses HTTP/2 with application/grpc+json content-type and standard 5-byte length-prefix framing.
/// Compatible with gRPC-JSON transcoding servers (grpc-gateway, Connect, etc.).
/// For plain protobuf gRPC servers, pass request_json as a JSON object — the server receives
/// it as raw bytes in the gRPC frame.
use std::collections::HashMap;
use std::time::Instant;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GrpcResponse {
    pub grpc_status: i32,
    pub grpc_message: String,
    pub http_status: u16,
    pub message_json: String,
    pub headers: HashMap<String, String>,
    pub trailers: HashMap<String, String>,
    pub duration_ms: u64,
}

#[tauri::command]
pub async fn grpc_unary(
    server_addr: String,
    method_path: String,
    request_json: String,
    metadata: HashMap<String, String>,
    tls_skip_verify: Option<bool>,
) -> Result<GrpcResponse, String> {
    let mut client_builder = reqwest::Client::builder()
        .http2_prior_knowledge();

    if tls_skip_verify.unwrap_or(false) {
        client_builder = client_builder.danger_accept_invalid_certs(true);
    }

    let client = client_builder.build().map_err(|e| e.to_string())?;

    // gRPC length-prefixed message framing: 1-byte flags + 4-byte big-endian length + body
    let body_bytes = request_json.as_bytes();
    let mut framed: Vec<u8> = Vec::with_capacity(5 + body_bytes.len());
    framed.push(0u8); // compressed flag = 0
    framed.extend_from_slice(&(body_bytes.len() as u32).to_be_bytes());
    framed.extend_from_slice(body_bytes);

    let base = server_addr.trim_end_matches('/');
    let path = method_path.trim_start_matches('/');
    let url = format!("{}/{}", base, path);

    let mut req_builder = client
        .post(&url)
        .header("content-type", "application/grpc+json")
        .header("te", "trailers")
        .header("user-agent", "Parallax-gRPC/0.1.0");

    for (k, v) in &metadata {
        req_builder = req_builder.header(k, v);
    }

    let start = Instant::now();
    let response = req_builder
        .body(framed)
        .send()
        .await
        .map_err(|e| format!("gRPC connect failed: {}", e))?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let http_status = response.status().as_u16();

    let headers: HashMap<String, String> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let grpc_status = headers
        .get("grpc-status")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);

    let grpc_message = headers
        .get("grpc-message")
        .cloned()
        .unwrap_or_default();

    let body_bytes = response.bytes().await.map_err(|e| e.to_string())?;

    // Decode gRPC frame: skip 5-byte header, rest is the message body
    let message_json = if body_bytes.len() >= 5 {
        let msg = &body_bytes[5..];
        // Try to pretty-print if valid JSON
        String::from_utf8_lossy(msg)
            .to_string()
            .trim()
            .to_string()
    } else {
        String::new()
    };

    // gRPC trailers may arrive as HTTP/2 trailing HEADERS frame
    // (reqwest exposes them as headers on the response object for HTTP/2)
    let trailers: HashMap<String, String> = headers
        .iter()
        .filter(|(k, _)| k.starts_with("grpc-"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    Ok(GrpcResponse {
        grpc_status,
        grpc_message,
        http_status,
        message_json,
        headers,
        trailers,
        duration_ms,
    })
}
