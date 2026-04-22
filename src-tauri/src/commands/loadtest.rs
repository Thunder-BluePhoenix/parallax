use tauri::{command, AppHandle, Emitter};
use tonic::transport::Channel;
use crate::commands::worker::pb;
use pb::load_test_service_client::LoadTestServiceClient;
use pb::LoadTestConfig;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct LoadTestProgress {
    pub completed: i64,
    pub total: i64,
    pub current_rps: f64,
    pub done: bool,
    pub result: Option<LoadTestResult>,
}

#[derive(Serialize, Clone)]
pub struct LoadTestResult {
    pub total_requests: i64,
    pub successful: i64,
    pub failed: i64,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub reqs_per_sec: f64,
    pub errors: Vec<String>,
    pub histogram: Vec<i64>,
}

#[command]
pub async fn run_load_test(
    _app: AppHandle,
    url: String,
    method: String,
    concurrent: i32,
    total_requests: i32,
) -> Result<(), String> {
    tauri::async_runtime::spawn(async move {
        let channel = match Channel::from_static("http://127.0.0.1:50151")
            .connect()
            .await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to connect to loadtest gRPC: {}", e);
                    return;
                }
            };

        let mut client = LoadTestServiceClient::new(channel);
        let request = tonic::Request::new(LoadTestConfig {
            url,
            method,
            headers: std::collections::HashMap::new(),
            body: "".to_string(),
            concurrent,
            total_requests,
            duration_sec: 0,
        });

        match client.run_load_test(request).await {
            Ok(response) => {
                let mut stream = response.into_inner();
                while let Ok(Some(progress)) = stream.message().await {
                    let event = LoadTestProgress {
                        completed: progress.completed,
                        total: progress.total,
                        current_rps: progress.current_rps,
                        done: progress.done,
                        result: progress.result.map(|r| LoadTestResult {
                            total_requests: r.total_requests,
                            successful: r.successful,
                            failed: r.failed,
                            avg_latency_ms: r.avg_latency_ms,
                            p50_latency_ms: r.p50_latency_ms,
                            p95_latency_ms: r.p95_latency_ms,
                            p99_latency_ms: r.p99_latency_ms,
                            min_latency_ms: r.min_latency_ms,
                            max_latency_ms: r.max_latency_ms,
                            reqs_per_sec: r.reqs_per_sec,
                            errors: r.errors,
                            histogram: r.histogram,
                        }),
                    };
                    let _ = _app.emit("loadtest_progress_event", event);
                    if progress.done {
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to start load test stream: {}", e);
            }
        }
    });

    Ok(())
}
