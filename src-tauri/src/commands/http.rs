use std::collections::HashMap;
use crate::http_engine::{HttpEngine, ParallaxRequest, ParallaxResponse};
use crate::AppState;

#[tauri::command]
pub async fn send_request(
    request: ParallaxRequest,
    environment: HashMap<String, String>,
    state: tauri::State<'_, AppState>,
) -> Result<ParallaxResponse, String> {
    let request_id = request.id.clone();

    let engine = HttpEngine::build_for_request(&request).map_err(|e| e.to_string())?;
    let req = request;
    let env = environment;

    let handle = tokio::spawn(async move {
        engine.execute(&req, &env).await
    });

    let abort_handle = handle.abort_handle();
    state.pending_requests.lock().unwrap().insert(request_id.clone(), abort_handle);

    let result = handle.await;
    state.pending_requests.lock().unwrap().remove(&request_id);

    match result {
        Ok(Ok(resp)) => Ok(resp),
        Ok(Err(e)) => Err(e.to_string()),
        Err(e) if e.is_cancelled() => Err("Request cancelled".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn cancel_request(
    request_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    if let Some(handle) = state.pending_requests.lock().unwrap().remove(&request_id) {
        handle.abort();
        println!("[Parallax] Cancelled request: {}", request_id);
    }
    Ok(())
}
