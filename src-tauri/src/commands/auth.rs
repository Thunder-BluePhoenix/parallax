use serde::{Deserialize, Serialize};
use crate::auth_providers::{AuthProviderManager, EcosystemProvider, ProviderCredentials, AuthSession};

#[derive(Debug, Serialize, Deserialize)]
pub struct DetectFrameworkResult {
    pub provider: String,
    pub display_name: String,
}

#[tauri::command]
pub async fn detect_framework(base_url: String) -> Result<DetectFrameworkResult, String> {
    let manager = AuthProviderManager::new();
    let provider = manager.detect_framework(&base_url).await.map_err(|e| e.to_string())?;
    Ok(DetectFrameworkResult {
        display_name: provider.display_name().to_string(),
        provider: format!("{:?}", provider).to_lowercase(),
    })
}

#[derive(Debug, Deserialize)]
pub struct PerformAuthInput {
    pub provider: String,
    pub credentials: ProviderCredentials,
}

#[tauri::command]
pub async fn perform_auth(input: PerformAuthInput) -> Result<AuthSession, String> {
    let manager = AuthProviderManager::new();
    let provider = EcosystemProvider::from_str(&input.provider);
    manager
        .perform_auth(provider, input.credentials)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn refresh_auth(
    _provider: String,
    _session: serde_json::Value,
    _base_url: String,
) -> Result<AuthSession, String> {
    // Re-authenticate using stored session metadata
    // For now, delegate back to perform_auth with stored creds
    Err("refresh_auth: not yet implemented — re-login to refresh".to_string())
}
