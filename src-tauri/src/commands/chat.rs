use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

const CHAT_PORT: u16 = 50152;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub workspace_id: String,
    pub sender: String,
    pub sender_name: String,
    pub body: String,
    pub ts: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatPresence {
    pub login: String,
    pub name: String,
    pub ts: i64,
}

#[tauri::command]
pub async fn chat_send_message(
    workspace: String,
    sender: String,
    sender_name: String,
    body: String,
) -> Result<(), String> {
    Client::new()
        .post(format!("http://127.0.0.1:{}/chat/message", CHAT_PORT))
        .json(&serde_json::json!({
            "workspace_id": workspace,
            "sender": sender,
            "sender_name": sender_name,
            "body": body,
        }))
        .send()
        .await
        .map_err(|e| format!("Chat service unavailable: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn chat_get_history(workspace: String) -> Result<Vec<ChatMessage>, String> {
    let resp = Client::new()
        .get(format!("http://127.0.0.1:{}/chat/history", CHAT_PORT))
        .query(&[("workspace", &workspace)])
        .send()
        .await
        .map_err(|e| format!("Chat service unavailable: {}", e))?;

    let messages: Vec<ChatMessage> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(messages)
}

#[tauri::command]
pub async fn chat_set_presence(
    workspace: String,
    login: String,
    name: String,
) -> Result<(), String> {
    Client::new()
        .post(format!("http://127.0.0.1:{}/chat/presence", CHAT_PORT))
        .json(&serde_json::json!({
            "workspace_id": workspace,
            "login": login,
            "name": name,
        }))
        .send()
        .await
        .map_err(|e| format!("Chat service unavailable: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn chat_get_presence(workspace: String) -> Result<Vec<ChatPresence>, String> {
    let resp = Client::new()
        .get(format!("http://127.0.0.1:{}/chat/presence", CHAT_PORT))
        .query(&[("workspace", &workspace)])
        .send()
        .await
        .map_err(|e| format!("Chat service unavailable: {}", e))?;

    let presence: Vec<ChatPresence> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(presence)
}

/// Subscribe to the chat SSE stream from the Go sidecar and forward events
/// as Tauri `chat_message` events to the frontend.
#[tauri::command]
pub async fn chat_start_stream(app: AppHandle, workspace: String) -> Result<(), String> {
    let url = format!(
        "http://127.0.0.1:{}/chat/stream?workspace={}",
        CHAT_PORT,
        percent_encoding::utf8_percent_encode(
            &workspace,
            percent_encoding::NON_ALPHANUMERIC
        )
    );

    tokio::spawn(async move {
        let client = Client::new();
        let Ok(resp) = client.get(&url).send().await else {
            return;
        };
        let mut stream = resp.bytes_stream();
        let mut buf = String::new();

        while let Some(Ok(chunk)) = stream.next().await {
            buf.push_str(&String::from_utf8_lossy(&chunk));
            // SSE events are delimited by double newlines
            while let Some(pos) = buf.find("\n\n") {
                let block = buf[..pos].to_string();
                buf = buf[pos + 2..].to_string();
                for line in block.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(msg) = serde_json::from_str::<ChatMessage>(data) {
                            let _ = app.emit("chat_message", msg);
                        } else if let Ok(presence) =
                            serde_json::from_str::<ChatPresence>(data)
                        {
                            let _ = app.emit("chat_presence", presence);
                        }
                    }
                }
            }
        }
    });

    Ok(())
}
