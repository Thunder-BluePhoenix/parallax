use tauri::command;
use tonic::transport::Channel;
use std::time::Duration;

// Include the generated gRPC client
pub mod pb {
    tonic::include_proto!("parallax");
}

use pb::worker_service_client::WorkerServiceClient;
use pb::GenericRequest;

#[command]
pub async fn ping_worker() -> Result<bool, String> {
    println!("[Rust] ping_worker received");
    // Try to connect to the Go sidecar on 50151
    let channel = Channel::from_static("http://127.0.0.1:50151")
        .timeout(Duration::from_secs(2))
        .connect()
        .await
        .map_err(|e| {
            let err = format!("Failed to connect to worker: {}", e);
            println!("[Rust] {}", err);
            err
        })?;

    let mut client = WorkerServiceClient::new(channel);

    let request = tonic::Request::new(GenericRequest {});

    let response = client
        .ping(request)
        .await
        .map_err(|e| {
            let err = format!("Ping failed: {}", e);
            println!("[Rust] {}", err);
            err
        })?;
    
    println!("[Rust] ping_worker successful");

    Ok(response.into_inner().success)
}
