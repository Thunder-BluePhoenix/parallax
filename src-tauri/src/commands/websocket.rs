// Parallax WebSocket Command — bidirectional frame streaming via Tauri events
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures::{SinkExt, StreamExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsFrame {
    pub connection_id: String,
    pub frame_type:    String, // "text" | "binary" | "ping" | "pong" | "close" | "error" | "connected"
    pub data:          Option<String>,
    pub timestamp_ms:  u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsConnectRequest {
    pub url:     String,
    pub headers: HashMap<String, String>,
}

// Shared map from connection_id → sender channel (for sending messages into the WS)
type WsMap = Arc<Mutex<HashMap<String, mpsc::Sender<String>>>>;

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

fn emit_frame(app: &AppHandle, frame: WsFrame) {
    let _ = app.emit("ws_frame", frame);
}

#[tauri::command]
pub async fn ws_connect(
    app:    AppHandle,
    ws_map: tauri::State<'_, WsMap>,
    req:    WsConnectRequest,
) -> Result<String, String> {
    let connection_id = uuid::Uuid::new_v4().to_string();
    let cid = connection_id.clone();

    let mut request = tokio_tungstenite::tungstenite::handshake::client::Request::builder()
        .uri(&req.url)
        .header("User-Agent", "Parallax/0.1.0");
    for (k, v) in &req.headers {
        request = request.header(k.as_str(), v.as_str());
    }
    let request = request.body(()).map_err(|e| e.to_string())?;

    let (ws_stream, _) = connect_async(request).await.map_err(|e| e.to_string())?;
    let (mut write, mut read) = ws_stream.split();

    emit_frame(&app, WsFrame {
        connection_id: cid.clone(),
        frame_type:    "connected".into(),
        data:          None,
        timestamp_ms:  now_ms(),
    });

    // Channel for sending frames into the WS
    let (tx, mut rx) = mpsc::channel::<String>(64);
    ws_map.lock().await.insert(cid.clone(), tx);

    // Spawn writer task
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if write.send(Message::Text(msg)).await.is_err() { break; }
        }
    });

    // Spawn reader task — forwards incoming frames to Svelte via events
    let app2 = app.clone();
    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(t)) => emit_frame(&app2, WsFrame {
                    connection_id: cid.clone(), frame_type: "text".into(),
                    data: Some(t.to_string()), timestamp_ms: now_ms(),
                }),
                Ok(Message::Binary(b)) => emit_frame(&app2, WsFrame {
                    connection_id: cid.clone(), frame_type: "binary".into(),
                    data: Some(format!("<{} bytes>", b.len())), timestamp_ms: now_ms(),
                }),
                Ok(Message::Close(_)) => {
                    emit_frame(&app2, WsFrame {
                        connection_id: cid.clone(), frame_type: "close".into(),
                        data: None, timestamp_ms: now_ms(),
                    });
                    break;
                }
                Err(e) => {
                    emit_frame(&app2, WsFrame {
                        connection_id: cid.clone(), frame_type: "error".into(),
                        data: Some(e.to_string()), timestamp_ms: now_ms(),
                    });
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(connection_id)
}

#[tauri::command]
pub async fn ws_send(
    ws_map:        tauri::State<'_, WsMap>,
    connection_id: String,
    message:       String,
) -> Result<(), String> {
    let map = ws_map.lock().await;
    if let Some(tx) = map.get(&connection_id) {
        tx.send(message).await.map_err(|e| e.to_string())
    } else {
        Err(format!("No WebSocket connection: {}", connection_id))
    }
}

#[tauri::command]
pub async fn ws_disconnect(
    ws_map:        tauri::State<'_, WsMap>,
    connection_id: String,
) -> Result<(), String> {
    ws_map.lock().await.remove(&connection_id);
    Ok(())
}

pub type WsStateMap = WsMap;
pub fn new_ws_state() -> WsStateMap {
    Arc::new(Mutex::new(HashMap::new()))
}
