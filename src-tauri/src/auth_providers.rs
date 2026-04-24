/// Parallax Auth Providers — Pluggable ecosystem-aware authentication
/// Supports: Frappe, Django, Laravel, Rails, WordPress, OAuth2, Generic Bearer
use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use reqwest::Client;

/// All supported auth ecosystem providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EcosystemProvider {
    Frappe,
    Django,
    Laravel,
    Rails,
    WordPress,
    FastAPI,
    OAuth2,
    Generic,
}

impl EcosystemProvider {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "frappe" | "erpnext" => Self::Frappe,
            "django" => Self::Django,
            "laravel" => Self::Laravel,
            "rails" | "ruby_on_rails" => Self::Rails,
            "wordpress" | "wp" => Self::WordPress,
            "fastapi" => Self::FastAPI,
            "oauth2" | "oauth2_pkce" => Self::OAuth2,
            _ => Self::Generic,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            EcosystemProvider::Frappe => "Frappe / ERPNext",
            EcosystemProvider::Django => "Django",
            EcosystemProvider::Laravel => "Laravel",
            EcosystemProvider::Rails => "Ruby on Rails",
            EcosystemProvider::WordPress => "WordPress",
            EcosystemProvider::FastAPI => "FastAPI",
            EcosystemProvider::OAuth2 => "OAuth2 / PKCE",
            EcosystemProvider::Generic => "Generic Session",
        }
    }
}

/// Credentials the user provides for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCredentials {
    pub base_url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub extra: Option<HashMap<String, String>>,
}

/// The result of a successful auth session — injected into requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub provider: String,
    pub base_url: String,
    pub injected_headers: HashMap<String, String>,
    pub injected_cookies: HashMap<String, String>,
    pub session_data: serde_json::Value,
    pub expires_at: Option<i64>,
}

/// Top-level auth manager — dispatches to the correct provider
pub struct AuthProviderManager {
    client: Client,
}

impl AuthProviderManager {
    pub fn new() -> Self {
        let client = Client::builder()
            .cookie_store(true)
            .build()
            .expect("Failed to build auth client");
        Self { client }
    }

    /// Detect which framework is running at a given base URL
    pub async fn detect_framework(&self, base_url: &str) -> Result<EcosystemProvider> {
        // Try Frappe
        if let Ok(resp) = self.client
            .get(format!("{}/api/method/frappe.auth.get_logged_user", base_url))
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await
        {
            if resp.status().as_u16() == 403 || resp.status().as_u16() == 200 {
                return Ok(EcosystemProvider::Frappe);
            }
        }

        // Try Django
        if let Ok(resp) = self.client
            .get(format!("{}/admin/", base_url))
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await
        {
            let body = resp.text().await.unwrap_or_default();
            if body.contains("csrfmiddlewaretoken") || body.contains("Django") {
                return Ok(EcosystemProvider::Django);
            }
        }

        // Try Laravel
        if let Ok(resp) = self.client
            .get(format!("{}/api/user", base_url))
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await
        {
            let headers = resp.headers().clone();
            if headers.get("X-RateLimit-Limit").is_some()
                || headers.get("x-powered-by")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.contains("PHP"))
                    .unwrap_or(false)
            {
                return Ok(EcosystemProvider::Laravel);
            }
        }

        // Try WordPress
        if let Ok(resp) = self.client
            .get(format!("{}/wp-json/wp/v2", base_url))
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await
        {
            if resp.status().is_success() {
                return Ok(EcosystemProvider::WordPress);
            }
        }

        Ok(EcosystemProvider::Generic)
    }

    /// Authenticate using the detected/specified provider
    pub async fn perform_auth(
        &self,
        provider: EcosystemProvider,
        creds: ProviderCredentials,
    ) -> Result<AuthSession> {
        match provider {
            EcosystemProvider::Frappe => self.auth_frappe(creds).await,
            EcosystemProvider::Django => self.auth_django(creds).await,
            EcosystemProvider::Laravel => self.auth_laravel(creds).await,
            EcosystemProvider::Rails => self.auth_rails(creds).await,
            EcosystemProvider::WordPress => self.auth_wordpress(creds).await,
            EcosystemProvider::FastAPI => self.auth_fastapi(creds).await,
            EcosystemProvider::OAuth2 => self.auth_oauth2_pkce(creds).await,
            EcosystemProvider::Generic => self.auth_generic_bearer(creds).await,
        }
    }

    // =========================================================================
    // FRAPPE / ERPNext
    // Handles: sid cookie + X-Frappe-CSRF-Token header injection
    // =========================================================================
    async fn auth_frappe(&self, creds: ProviderCredentials) -> Result<AuthSession> {
        let login_url = format!("{}/api/method/login", creds.base_url);

        let mut form_data = HashMap::new();
        form_data.insert("usr", creds.username.as_deref().unwrap_or(""));
        form_data.insert("pwd", creds.password.as_deref().unwrap_or(""));

        let resp = self.client
            .post(&login_url)
            .form(&form_data)
            .send()
            .await?;

        let cookies: HashMap<String, String> = resp.cookies()
            .map(|c| (c.name().to_string(), c.value().to_string()))
            .collect();

        let sid = cookies.get("sid").cloned().unwrap_or_default();

        // Get CSRF token
        let csrf_resp = self.client
            .get(format!("{}/api/method/frappe.auth.get_logged_user", creds.base_url))
            .header("Cookie", format!("sid={}", sid))
            .send()
            .await?;

        let csrf_token = csrf_resp
            .headers()
            .get("X-Frappe-CSRF-Token")
            .or_else(|| csrf_resp.headers().get("x-frappe-csrf-token"))
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let mut headers = HashMap::new();
        headers.insert("X-Frappe-CSRF-Token".to_string(), csrf_token.clone());
        headers.insert("Cookie".to_string(), format!("sid={}", sid));

        let mut injected_cookies = HashMap::new();
        injected_cookies.insert("sid".to_string(), sid.clone());

        Ok(AuthSession {
            provider: "frappe".to_string(),
            base_url: creds.base_url,
            injected_headers: headers,
            injected_cookies,
            session_data: serde_json::json!({
                "sid": sid,
                "csrf_token": csrf_token,
            }),
            expires_at: None,
        })
    }

    // =========================================================================
    // DJANGO
    // Handles: csrftoken cookie + X-CSRFToken / csrfmiddlewaretoken injection
    // =========================================================================
    async fn auth_django(&self, creds: ProviderCredentials) -> Result<AuthSession> {
        // 1. GET login page to grab csrftoken cookie
        let login_page_url = format!("{}/accounts/login/", creds.base_url);
        let get_resp = self.client.get(&login_page_url).send().await?;

        let csrf_cookie = get_resp.cookies()
            .find(|c| c.name() == "csrftoken")
            .map(|c| c.value().to_string())
            .unwrap_or_default();

        // 2. POST with csrfmiddlewaretoken
        let mut form = HashMap::new();
        form.insert("username", creds.username.as_deref().unwrap_or(""));
        form.insert("password", creds.password.as_deref().unwrap_or(""));
        form.insert("csrfmiddlewaretoken", &csrf_cookie);

        let post_resp = self.client
            .post(&login_page_url)
            .header("Referer", &login_page_url)
            .header("X-CSRFToken", &csrf_cookie)
            .form(&form)
            .send()
            .await?;

        let session_id = post_resp.cookies()
            .find(|c| c.name() == "sessionid")
            .map(|c| c.value().to_string())
            .unwrap_or_default();

        let mut headers = HashMap::new();
        headers.insert("X-CSRFToken".to_string(), csrf_cookie.clone());
        headers.insert(
            "Cookie".to_string(),
            format!("csrftoken={}; sessionid={}", csrf_cookie, session_id),
        );

        let mut injected_cookies = HashMap::new();
        injected_cookies.insert("csrftoken".to_string(), csrf_cookie.clone());
        injected_cookies.insert("sessionid".to_string(), session_id.clone());

        Ok(AuthSession {
            provider: "django".to_string(),
            base_url: creds.base_url,
            injected_headers: headers,
            injected_cookies,
            session_data: serde_json::json!({
                "csrf_token": csrf_cookie,
                "session_id": session_id,
            }),
            expires_at: None,
        })
    }

    // =========================================================================
    // LARAVEL
    // Handles: XSRF-TOKEN cookie + X-XSRF-TOKEN header (Sanctum / default)
    // =========================================================================
    async fn auth_laravel(&self, creds: ProviderCredentials) -> Result<AuthSession> {
        // 1. GET CSRF cookie from Sanctum endpoint
        let _ = self.client
            .get(format!("{}/sanctum/csrf-cookie", creds.base_url))
            .send()
            .await?;

        let xsrf_token = self.client
            .get(format!("{}/", creds.base_url))
            .send()
            .await
            .ok()
            .and_then(|r| {
                r.cookies()
                    .find(|c| c.name() == "XSRF-TOKEN")
                    .map(|c| {
                        // XSRF-TOKEN is URL-encoded
                        urlencoding_decode(c.value())
                    })
            })
            .unwrap_or_default();

        // 2. POST to login
        let json_body = serde_json::json!({
            "email": creds.username.as_deref().unwrap_or(""),
            "password": creds.password.as_deref().unwrap_or(""),
        });

        let post_resp = self.client
            .post(format!("{}/login", creds.base_url))
            .header("X-XSRF-TOKEN", &xsrf_token)
            .header("Accept", "application/json")
            .json(&json_body)
            .send()
            .await?;

        // Extract bearer token from response if API mode
        let body: serde_json::Value = post_resp.json().await.unwrap_or_default();
        let bearer = body.get("token")
            .or_else(|| body.get("access_token"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

        let mut headers = HashMap::new();
        headers.insert("X-XSRF-TOKEN".to_string(), xsrf_token.clone());
        if !bearer.is_empty() {
            headers.insert("Authorization".to_string(), format!("Bearer {}", bearer));
        }

        Ok(AuthSession {
            provider: "laravel".to_string(),
            base_url: creds.base_url,
            injected_headers: headers,
            injected_cookies: HashMap::new(),
            session_data: serde_json::json!({
                "xsrf_token": xsrf_token,
                "bearer": bearer,
            }),
            expires_at: None,
        })
    }

    // =========================================================================
    // RUBY ON RAILS
    // Handles: authenticity_token from form + session cookie
    // =========================================================================
    async fn auth_rails(&self, creds: ProviderCredentials) -> Result<AuthSession> {
        let login_url = format!("{}/users/sign_in", creds.base_url);
        let get_resp = self.client.get(&login_url).send().await?;
        let body = get_resp.text().await.unwrap_or_default();

        // Extract authenticity_token from HTML
        let auth_token = extract_html_input_value(&body, "authenticity_token");

        let mut form = HashMap::new();
        form.insert("authenticity_token", auth_token.as_str());
        form.insert("user[email]", creds.username.as_deref().unwrap_or(""));
        form.insert("user[password]", creds.password.as_deref().unwrap_or(""));

        let post_resp = self.client
            .post(&login_url)
            .form(&form)
            .send()
            .await?;

        let session_cookie = post_resp.cookies()
            .find(|c| c.name().starts_with("_") && c.name().ends_with("_session"))
            .map(|c| format!("{}={}", c.name(), c.value()))
            .unwrap_or_default();

        let mut headers = HashMap::new();
        headers.insert("Cookie".to_string(), session_cookie.clone());
        headers.insert("X-CSRF-Token".to_string(), auth_token.clone());

        Ok(AuthSession {
            provider: "rails".to_string(),
            base_url: creds.base_url,
            injected_headers: headers,
            injected_cookies: HashMap::new(),
            session_data: serde_json::json!({
                "authenticity_token": auth_token,
                "session": session_cookie,
            }),
            expires_at: None,
        })
    }

    // =========================================================================
    // WORDPRESS
    // Handles: Application Password or JWT-based auth
    // =========================================================================
    async fn auth_wordpress(&self, creds: ProviderCredentials) -> Result<AuthSession> {
        // WordPress REST API uses Application Passwords (Basic Auth)
        let test_resp = self.client
            .get(format!("{}/wp-json/wp/v2/users/me", creds.base_url))
            .basic_auth(
                creds.username.as_deref().unwrap_or(""),
                Some(creds.password.as_deref().unwrap_or("")),
            )
            .send()
            .await?;

        let nonce = test_resp
            .headers()
            .get("X-WP-Nonce")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let user_data: serde_json::Value = test_resp.json().await.unwrap_or_default();

        let basic = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!(
                "{}:{}",
                creds.username.as_deref().unwrap_or(""),
                creds.password.as_deref().unwrap_or("")
            )
        );

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Basic {}", basic));
        if !nonce.is_empty() {
            headers.insert("X-WP-Nonce".to_string(), nonce.clone());
        }

        Ok(AuthSession {
            provider: "wordpress".to_string(),
            base_url: creds.base_url,
            injected_headers: headers,
            injected_cookies: HashMap::new(),
            session_data: serde_json::json!({
                "user": user_data,
                "nonce": nonce,
            }),
            expires_at: None,
        })
    }

    // =========================================================================
    // FASTAPI
    // Handles: OAuth2 password flow → Bearer token
    // =========================================================================
    async fn auth_fastapi(&self, creds: ProviderCredentials) -> Result<AuthSession> {
        let token_url = creds.extra
            .as_ref()
            .and_then(|e| e.get("token_url"))
            .cloned()
            .unwrap_or_else(|| format!("{}/token", creds.base_url));

        let mut form = HashMap::new();
        form.insert("username", creds.username.as_deref().unwrap_or("").to_string());
        form.insert("password", creds.password.as_deref().unwrap_or("").to_string());
        form.insert("grant_type", "password".to_string());

        let resp = self.client
            .post(&token_url)
            .form(&form)
            .send()
            .await?;

        let body: serde_json::Value = resp.json().await?;
        let access_token = body.get("access_token")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

        let expires_in = body.get("expires_in")
            .and_then(|e| e.as_i64())
            .unwrap_or(3600);

        let expires_at = chrono::Utc::now().timestamp() + expires_in;

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", access_token));

        Ok(AuthSession {
            provider: "fastapi".to_string(),
            base_url: creds.base_url,
            injected_headers: headers,
            injected_cookies: HashMap::new(),
            session_data: serde_json::json!({ "token_response": body }),
            expires_at: Some(expires_at),
        })
    }

    // =========================================================================
    // OAUTH2 — Authorization Code + PKCE (RFC 7636)
    // extra fields used:
    //   token_url      — required: token endpoint
    //   client_id      — required
    //   code           — required: authorization code from redirect
    //   redirect_uri   — required
    //   code_verifier  — required: plain code verifier (we hash it to code_challenge)
    // =========================================================================
    async fn auth_oauth2_pkce(&self, creds: ProviderCredentials) -> Result<AuthSession> {
        use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
        use sha2::{Sha256, Digest};

        let extra = creds.extra.as_ref().ok_or_else(|| anyhow::anyhow!("OAuth2: extra fields required"))?;
        let token_url   = extra.get("token_url").ok_or_else(|| anyhow::anyhow!("OAuth2: token_url required"))?;
        let client_id   = extra.get("client_id").ok_or_else(|| anyhow::anyhow!("OAuth2: client_id required"))?;
        let code        = extra.get("code").ok_or_else(|| anyhow::anyhow!("OAuth2: code required"))?;
        let redirect_uri = extra.get("redirect_uri").ok_or_else(|| anyhow::anyhow!("OAuth2: redirect_uri required"))?;
        let code_verifier = extra.get("code_verifier").ok_or_else(|| anyhow::anyhow!("OAuth2: code_verifier required"))?;

        // Derive code_challenge = BASE64URL(SHA256(code_verifier))
        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

        let mut form = HashMap::new();
        form.insert("grant_type", "authorization_code".to_string());
        form.insert("code", code.clone());
        form.insert("redirect_uri", redirect_uri.clone());
        form.insert("client_id", client_id.clone());
        form.insert("code_verifier", code_verifier.clone());
        form.insert("code_challenge", challenge);
        form.insert("code_challenge_method", "S256".to_string());

        let resp = self.client.post(token_url).form(&form).send().await?;
        let body: serde_json::Value = resp.json().await?;

        if let Some(err) = body.get("error").and_then(|e| e.as_str()) {
            return Err(anyhow::anyhow!("OAuth2 error: {} — {}", err,
                body.get("error_description").and_then(|d| d.as_str()).unwrap_or("")));
        }

        let access_token  = body["access_token"].as_str().unwrap_or("").to_string();
        let refresh_token = body["refresh_token"].as_str().unwrap_or("").to_string();
        let expires_in    = body["expires_in"].as_i64().unwrap_or(3600);
        let token_type    = body["token_type"].as_str().unwrap_or("Bearer").to_string();
        let expires_at    = chrono::Utc::now().timestamp() + expires_in;

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("{} {}", token_type, access_token));

        Ok(AuthSession {
            provider: "oauth2".to_string(),
            base_url: creds.base_url,
            injected_headers: headers,
            injected_cookies: HashMap::new(),
            session_data: serde_json::json!({
                "access_token": access_token,
                "refresh_token": refresh_token,
                "token_type": token_type,
                "expires_in": expires_in,
            }),
            expires_at: Some(expires_at),
        })
    }

    // =========================================================================
    // GENERIC BEARER
    // =========================================================================
    async fn auth_generic_bearer(&self, creds: ProviderCredentials) -> Result<AuthSession> {
        let token = creds.api_key.unwrap_or_default();
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", token));

        Ok(AuthSession {
            provider: "generic".to_string(),
            base_url: creds.base_url,
            injected_headers: headers,
            injected_cookies: HashMap::new(),
            session_data: serde_json::json!({}),
            expires_at: None,
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn urlencoding_decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s)
        .decode_utf8_lossy()
        .to_string()
}

fn extract_html_input_value(html: &str, field_name: &str) -> String {
    let pattern = format!(r#"name="{}"\s+value="([^"]+)""#, field_name);
    let re = regex::Regex::new(&pattern).unwrap();
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_default()
}

