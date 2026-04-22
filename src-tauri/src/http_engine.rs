/// Parallax HTTP Engine — Rust/reqwest powered HTTP/2+HTTP/3 execution core
use std::collections::HashMap;
use std::time::{Duration, Instant};

use reqwest::{Client, Method, Response};
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// A single HTTP request definition (maps to .parallax YAML)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallaxRequest {
    pub id: String,
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub params: Option<HashMap<String, String>>,
    pub body: Option<RequestBody>,
    pub auth: Option<AuthConfig>,
    pub timeout_ms: Option<u64>,
    pub follow_redirects: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    #[serde(rename = "type")]
    pub body_type: BodyType,
    pub content: serde_json::Value,
    pub raw: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BodyType {
    Json,
    FormData,
    UrlEncoded,
    Raw,
    Binary,
    GraphQL,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(rename = "type")]
    pub auth_type: AuthType,
    pub token: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub api_key_header: Option<String>,
    pub api_key_value: Option<String>,
    pub api_key_location: Option<String>, // "header" (default) or "query"
    /// Provider-specific context (e.g., "frappe", "django", "laravel")
    pub provider: Option<String>,
    pub provider_session: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    None,
    Bearer,
    Basic,
    ApiKey,
    OAuth2,
    /// Ecosystem-specific (Frappe, Django, Laravel, Rails, WordPress...)
    EcosystemProvider,
}

/// The result returned to the Svelte frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct ParallaxResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: ResponseBody,
    pub timing: ResponseTiming,
    pub cookies: Vec<CookieInfo>,
    pub size_bytes: usize,
    pub redirects: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseBody {
    pub raw: String,
    pub json: Option<serde_json::Value>,
    pub content_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseTiming {
    pub total_ms: u128,
    pub dns_ms: Option<u128>,
    pub connect_ms: Option<u128>,
    pub ttfb_ms: Option<u128>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CookieInfo {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub secure: bool,
    pub http_only: bool,
}

/// The main HTTP engine executor
pub struct HttpEngine {
    client: Client,
}

impl HttpEngine {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Parallax/0.1.0 (API Super-Client)")
            .timeout(Duration::from_secs(30))
            .gzip(true)
            .brotli(true)
            .cookie_store(true)
            .build()?;
        Ok(Self { client })
    }

    /// Resolve environment variables in a string ({{var}} syntax)
    pub fn resolve_env(template: &str, env: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        for (key, value) in env {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }
        result
    }

    /// Execute a request with full environment variable resolution
    pub async fn execute(
        &self,
        req: &ParallaxRequest,
        env: &HashMap<String, String>,
    ) -> Result<ParallaxResponse> {
        let start = Instant::now();

        // Resolve URL and all template vars
        let resolved_url = Self::resolve_env(&req.url, env);
        let mut url = reqwest::Url::parse(&resolved_url)?;

        // Query params
        if let Some(params) = &req.params {
            let mut pairs = url.query_pairs_mut();
            for (k, v) in params {
                let k = Self::resolve_env(k, env);
                let v = Self::resolve_env(v, env);
                pairs.append_pair(&k, &v);
            }
        }

        // API key injected as query param
        if let Some(auth) = &req.auth {
            if auth.auth_type == AuthType::ApiKey
                && auth.api_key_location.as_deref() == Some("query")
            {
                if let (Some(key), Some(val)) = (&auth.api_key_header, &auth.api_key_value) {
                    let key = Self::resolve_env(key, env);
                    let val = Self::resolve_env(val, env);
                    url.query_pairs_mut().append_pair(&key, &val);
                }
            }
        }

        let method = Method::from_bytes(req.method.to_uppercase().as_bytes())
            .unwrap_or(Method::GET);

        let mut builder = self.client.request(method, url);

        // Apply timeout
        if let Some(ms) = req.timeout_ms {
            builder = builder.timeout(Duration::from_millis(ms));
        }

        // Headers
        for (k, v) in &req.headers {
            let k = Self::resolve_env(k, env);
            let v = Self::resolve_env(v, env);
            builder = builder.header(k, v);
        }

        // Auth
        if let Some(auth) = &req.auth {
            builder = self.apply_auth(builder, auth, env);
        }

        // Body
        if let Some(body) = &req.body {
            builder = self.apply_body(builder, body, env)?;
        }

        let response = builder.send().await?;
        let elapsed = start.elapsed();

        self.parse_response(response, elapsed).await
    }

    fn apply_auth(
        &self,
        builder: reqwest::RequestBuilder,
        auth: &AuthConfig,
        env: &HashMap<String, String>,
    ) -> reqwest::RequestBuilder {
        match auth.auth_type {
            AuthType::Bearer => {
                if let Some(token) = &auth.token {
                    let token = Self::resolve_env(token, env);
                    return builder.bearer_auth(token);
                }
            }
            AuthType::Basic => {
                let user = auth.username.as_deref().unwrap_or("");
                let pass = auth.password.as_deref().unwrap_or("");
                let user = Self::resolve_env(user, env);
                let pass = Self::resolve_env(pass, env);
                return builder.basic_auth(user, Some(pass));
            }
            AuthType::ApiKey => {
                if auth.api_key_location.as_deref() != Some("query") {
                    if let (Some(header), Some(value)) = (&auth.api_key_header, &auth.api_key_value) {
                        let header = Self::resolve_env(header, env);
                        let value = Self::resolve_env(value, env);
                        return builder.header(header, value);
                    }
                }
            }
            AuthType::EcosystemProvider => {
                // Ecosystem providers inject headers via their session context
                if let Some(session) = &auth.provider_session {
                    if let Some(headers) = session.get("injected_headers").and_then(|h| h.as_object()) {
                        let mut b = builder;
                        for (k, v) in headers {
                            if let Some(v) = v.as_str() {
                                b = b.header(k.clone(), v.to_string());
                            }
                        }
                        return b;
                    }
                }
            }
            _ => {}
        }
        builder
    }

    fn apply_body(
        &self,
        builder: reqwest::RequestBuilder,
        body: &RequestBody,
        env: &HashMap<String, String>,
    ) -> Result<reqwest::RequestBuilder> {
        match body.body_type {
            BodyType::Json => {
                // Resolve env vars in JSON string
                let json_str = serde_json::to_string(&body.content)?;
                let resolved = Self::resolve_env(&json_str, env);
                let json_val: serde_json::Value = serde_json::from_str(&resolved)?;
                Ok(builder.json(&json_val))
            }
            BodyType::Raw => {
                let raw = body.raw.as_deref().unwrap_or("");
                let raw = Self::resolve_env(raw, env);
                Ok(builder.body(raw))
            }
            BodyType::UrlEncoded => {
                if let Some(obj) = body.content.as_object() {
                    let pairs: Vec<(String, String)> = obj
                        .iter()
                        .map(|(k, v)| {
                            let v = v.as_str().unwrap_or("").to_string();
                            (Self::resolve_env(k, env), Self::resolve_env(&v, env))
                        })
                        .collect();
                    Ok(builder.form(&pairs))
                } else {
                    Ok(builder)
                }
            }
            BodyType::GraphQL => {
                // GraphQL: { query, variables }
                Ok(builder.json(&body.content))
            }
            _ => Ok(builder),
        }
    }

    async fn parse_response(
        &self,
        response: Response,
        elapsed: Duration,
    ) -> Result<ParallaxResponse> {
        let status = response.status();
        let status_text = status.canonical_reason().unwrap_or("Unknown").to_string();

        let headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().to_string(),
                    v.to_str().unwrap_or("").to_string(),
                )
            })
            .collect();

        let content_type = headers
            .get("content-type")
            .cloned()
            .unwrap_or_default();

        // Parse Set-Cookie headers before consuming response
        let cookies: Vec<CookieInfo> = headers
            .iter()
            .filter(|(k, _)| k.to_lowercase() == "set-cookie")
            .map(|(_, v)| parse_set_cookie(v))
            .collect();

        let body_bytes = response.bytes().await?;
        let size_bytes = body_bytes.len();
        let raw = String::from_utf8_lossy(&body_bytes).to_string();

        let json = if content_type.contains("application/json") {
            serde_json::from_str::<serde_json::Value>(&raw).ok()
        } else {
            None
        };

        Ok(ParallaxResponse {
            status: status.as_u16(),
            status_text,
            headers,
            body: ResponseBody { raw, json, content_type },
            timing: ResponseTiming {
                total_ms: elapsed.as_millis(),
                dns_ms: None, connect_ms: None, ttfb_ms: None,
            },
            cookies,
            size_bytes,
            redirects: vec![],
        })
    }
}

fn parse_set_cookie(header: &str) -> CookieInfo {
    let parts: Vec<&str> = header.split(';').collect();
    let (name, value) = parts[0].split_once('=').unwrap_or(("", parts[0]));
    let mut cookie = CookieInfo {
        name: name.trim().to_string(),
        value: value.trim().to_string(),
        domain: None, path: None, secure: false, http_only: false,
    };
    for attr in parts.iter().skip(1) {
        let a = attr.trim().to_lowercase();
        if a == "secure"   { cookie.secure    = true; }
        if a == "httponly" { cookie.http_only  = true; }
        if let Some(d) = attr.trim().strip_prefix("domain=").or_else(|| attr.trim().strip_prefix("Domain=")) {
            cookie.domain = Some(d.to_string());
        }
        if let Some(p) = attr.trim().strip_prefix("path=").or_else(|| attr.trim().strip_prefix("Path=")) {
            cookie.path = Some(p.to_string());
        }
    }
    cookie
}


impl Default for HttpEngine {
    fn default() -> Self {
        Self::new().expect("Failed to build HTTP engine")
    }
}
