use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{AppHandle, Emitter};
use serde::{Deserialize, Serialize};
use reqwest::Method;
use futures_util::StreamExt;
use crate::http_engine::{HttpEngine, ParallaxRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseEvent {
    pub connection_id: String,
    pub event_type: String, // "message", "connected", "error", "closed"
    pub data: Option<String>,
    pub timestamp_ms: u64,
}

pub type SseMap = Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>;

pub fn new_sse_state() -> SseMap {
    Arc::new(Mutex::new(HashMap::new()))
}

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

fn emit_sse_event(app: &AppHandle, event: SseEvent) {
    let _ = app.emit("sse_event", event);
}

#[tauri::command]
pub async fn sse_connect(
    app: AppHandle,
    sse_map: tauri::State<'_, SseMap>,
    req: ParallaxRequest,
    env: HashMap<String, String>,
) -> Result<String, String> {
    let connection_id = uuid::Uuid::new_v4().to_string();
    let cid = connection_id.clone();

    // Notify connected
    emit_sse_event(&app, SseEvent {
        connection_id: cid.clone(),
        event_type: "connected".into(),
        data: None,
        timestamp_ms: now_ms(),
    });

    let _engine = HttpEngine::default();
    
    // Create the reqwest request builder just like execute()
    let resolved_url = HttpEngine::resolve_env(&req.url, &env);
    let mut url = reqwest::Url::parse(&resolved_url).map_err(|e| e.to_string())?;

    if let Some(params) = &req.params {
        let mut pairs = url.query_pairs_mut();
        for (k, v) in params {
            let k = HttpEngine::resolve_env(k, &env);
            let v = HttpEngine::resolve_env(v, &env);
            pairs.append_pair(&k, &v);
        }
    }

    let method = Method::from_bytes(req.method.to_uppercase().as_bytes())
        .unwrap_or(Method::GET);

    let client = reqwest::Client::new();
    let mut builder = client.request(method, url)
        .header("Accept", "text/event-stream");

    for (k, v) in &req.headers {
        let k = HttpEngine::resolve_env(k, &env);
        let v = HttpEngine::resolve_env(v, &env);
        builder = builder.header(k, v);
    }

    // Spawn task to read the stream
    let app2 = app.clone();
    let handle = tokio::spawn(async move {
        match builder.send().await {
            Ok(response) => {
                let mut stream = response.bytes_stream();
                let mut buffer = String::new();

                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(bytes) => {
                            if let Ok(text) = std::str::from_utf8(&bytes) {
                                buffer.push_str(text);
                                
                                // Split by double newline which separates SSE messages
                                while let Some(idx) = buffer.find("\n\n") {
                                    let message = buffer[..idx].to_string();
                                    buffer = buffer[idx + 2..].to_string();

                                    // Parse SSE lines (data: ...)
                                    let mut data_lines = Vec::new();
                                    for line in message.lines() {
                                        if let Some(d) = line.strip_prefix("data: ") {
                                            data_lines.push(d);
                                        } else if let Some(d) = line.strip_prefix("data:") {
                                            data_lines.push(d);
                                        }
                                    }

                                    if !data_lines.is_empty() {
                                        emit_sse_event(&app2, SseEvent {
                                            connection_id: cid.clone(),
                                            event_type: "message".into(),
                                            data: Some(data_lines.join("\n")),
                                            timestamp_ms: now_ms(),
                                        });
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            emit_sse_event(&app2, SseEvent {
                                connection_id: cid.clone(),
                                event_type: "error".into(),
                                data: Some(e.to_string()),
                                timestamp_ms: now_ms(),
                            });
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                emit_sse_event(&app2, SseEvent {
                    connection_id: cid.clone(),
                    event_type: "error".into(),
                    data: Some(e.to_string()),
                    timestamp_ms: now_ms(),
                });
            }
        }

        emit_sse_event(&app2, SseEvent {
            connection_id: cid.clone(),
            event_type: "closed".into(),
            data: None,
            timestamp_ms: now_ms(),
        });
    });

    sse_map.lock().await.insert(connection_id.clone(), handle);

    Ok(connection_id)
}

#[tauri::command]
pub async fn sse_disconnect(
    sse_map: tauri::State<'_, SseMap>,
    connection_id: String,
) -> Result<(), String> {
    if let Some(handle) = sse_map.lock().await.remove(&connection_id) {
        handle.abort();
    }
    Ok(())
}
