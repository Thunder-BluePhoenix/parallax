use std::collections::HashMap;
use crate::http_engine::{HttpEngine, ParallaxRequest, ParallaxResponse};

#[tauri::command]
pub async fn send_request(
    request: ParallaxRequest,
    environment: HashMap<String, String>,
) -> Result<ParallaxResponse, String> {
    let engine = HttpEngine::default();
    engine
        .execute(&request, &environment)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cancel_request(request_id: String) -> Result<(), String> {
    // TODO: Implement cancellation via abort handles stored in AppState
    println!("[Parallax] Cancel request: {}", request_id);
    Ok(())
}
