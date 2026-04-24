/// Parallax HTTP Engine — Rust/reqwest powered HTTP/2+HTTP/3 execution core
use std::collections::HashMap;
use std::time::{Duration, Instant};

use reqwest::{Client, Method, Response};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest as Sha2Digest};
use md5::Md5;
type HmacSha256 = Hmac<Sha256>;

fn rand_u32() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    (SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().subsec_nanos()) ^ 0xdeadbeef
}

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
    pub scripts: Option<RequestScripts>,
    /// Skip TLS certificate verification (insecure — dev only)
    pub tls_skip_verify: Option<bool>,
    /// HTTP/HTTPS proxy URL e.g. "http://127.0.0.1:8888"
    pub proxy_url: Option<String>,
    /// PEM-encoded client certificate for mTLS
    pub client_cert_pem: Option<String>,
    /// PEM-encoded client private key for mTLS
    pub client_key_pem: Option<String>,
    /// Disable the cookie jar for this request
    pub disable_cookies: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestScripts {
    #[serde(rename = "preRequest")]
    pub pre_request: Option<String>,
    pub tests: Option<String>,
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
    // AWS SigV4 fields
    pub aws_access_key: Option<String>,
    pub aws_secret_key: Option<String>,
    pub aws_region: Option<String>,
    pub aws_service: Option<String>,
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
    /// AWS Signature Version 4
    AwsSigV4,
    /// HTTP Digest authentication (RFC 7617)
    Digest,
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

    /// Build an engine customised for a specific request's TLS/proxy/cert/cookie config.
    /// Falls back to the default engine when no special config is present.
    pub fn build_for_request(req: &ParallaxRequest) -> Result<Self> {
        let needs_custom = req.tls_skip_verify.unwrap_or(false)
            || req.proxy_url.is_some()
            || req.client_cert_pem.is_some()
            || req.disable_cookies.unwrap_or(false);

        if !needs_custom {
            return Self::new();
        }

        let mut builder = Client::builder()
            .user_agent("Parallax/0.1.0 (API Super-Client)")
            .timeout(Duration::from_secs(30))
            .gzip(true)
            .brotli(true);

        if !req.disable_cookies.unwrap_or(false) {
            builder = builder.cookie_store(true);
        }

        if req.tls_skip_verify.unwrap_or(false) {
            builder = builder.danger_accept_invalid_certs(true);
        }

        if let Some(proxy_url) = &req.proxy_url {
            let proxy = reqwest::Proxy::all(proxy_url).map_err(|e| anyhow::anyhow!(e))?;
            builder = builder.proxy(proxy);
        }

        if let (Some(cert_pem), Some(key_pem)) = (&req.client_cert_pem, &req.client_key_pem) {
            let combined = format!("{}\n{}", cert_pem, key_pem);
            let identity = reqwest::Identity::from_pem(combined.as_bytes())
                .map_err(|e| anyhow::anyhow!(e))?;
            builder = builder.identity(identity);
        }

        Ok(Self { client: builder.build()? })
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

        // AWS SigV4 — sign after body is set
        if let Some(auth) = &req.auth {
            if auth.auth_type == AuthType::AwsSigV4 {
                builder = self.apply_sigv4(builder, auth, &req.method, &resolved_url, req, env)?;
            }
        }

        let mut response = builder.send().await?;

        // Digest auth — 401 challenge-response
        if let Some(auth) = &req.auth {
            if auth.auth_type == AuthType::Digest && response.status() == 401 {
                if let Some(www_auth) = response.headers().get("www-authenticate").cloned() {
                    let www_auth_str = www_auth.to_str().unwrap_or("").to_string();
                    // Rebuild request with Digest header
                    let method_str = req.method.to_uppercase();
                    let uri_path = reqwest::Url::parse(&resolved_url)
                        .map(|u| u.path().to_string())
                        .unwrap_or_else(|_| "/".to_string());
                    let digest_header = Self::build_digest_header(
                        auth, env, &method_str, &uri_path, &www_auth_str,
                    );
                    let method2 = Method::from_bytes(method_str.as_bytes()).unwrap_or(Method::GET);
                    let mut builder2 = self.client.request(method2, reqwest::Url::parse(&resolved_url)?);
                    if let Some(ms) = req.timeout_ms {
                        builder2 = builder2.timeout(Duration::from_millis(ms));
                    }
                    for (k, v) in &req.headers {
                        builder2 = builder2.header(Self::resolve_env(k, env), Self::resolve_env(v, env));
                    }
                    builder2 = builder2.header("Authorization", digest_header);
                    if let Some(body) = &req.body {
                        builder2 = self.apply_body(builder2, body, env)?;
                    }
                    response = builder2.send().await?;
                }
            }
        }

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

    fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key len ok");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    fn sha256_hex(data: &[u8]) -> String {
        let hash = Sha256::digest(data);
        hex::encode(hash)
    }

    fn apply_sigv4(
        &self,
        builder: reqwest::RequestBuilder,
        auth: &AuthConfig,
        method: &str,
        url_str: &str,
        req: &ParallaxRequest,
        env: &HashMap<String, String>,
    ) -> Result<reqwest::RequestBuilder> {
        let access_key = auth.aws_access_key.as_deref().unwrap_or("");
        let secret_key = auth.aws_secret_key.as_deref().unwrap_or("");
        let region = Self::resolve_env(auth.aws_region.as_deref().unwrap_or("us-east-1"), env);
        let service = Self::resolve_env(auth.aws_service.as_deref().unwrap_or("execute-api"), env);

        let access_key = Self::resolve_env(access_key, env);
        let secret_key = Self::resolve_env(secret_key, env);

        let parsed = reqwest::Url::parse(url_str)?;
        let canonical_uri = parsed.path().to_string();
        let canonical_query = parsed.query().unwrap_or("").to_string();

        // Body hash
        let body_bytes = if let Some(body) = &req.body {
            serde_json::to_vec(&body.content).unwrap_or_default()
        } else {
            vec![]
        };
        let payload_hash = Self::sha256_hex(&body_bytes);

        let now = chrono::Utc::now();
        let datestamp = now.format("%Y%m%d").to_string();
        let amzdate = now.format("%Y%m%dT%H%M%SZ").to_string();

        // Canonical headers
        let host = parsed.host_str().unwrap_or("");
        let canonical_headers = format!(
            "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
            host, payload_hash, amzdate
        );
        let signed_headers = "host;x-amz-content-sha256;x-amz-date";

        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            method.to_uppercase(),
            canonical_uri,
            canonical_query,
            canonical_headers,
            signed_headers,
            payload_hash,
        );

        let credential_scope = format!("{}/{}/{}/aws4_request", datestamp, region, service);
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{}\n{}\n{}",
            amzdate,
            credential_scope,
            Self::sha256_hex(canonical_request.as_bytes()),
        );

        // Signing key derivation
        let signing_key = {
            let k_date = Self::hmac_sha256(
                format!("AWS4{}", secret_key).as_bytes(),
                datestamp.as_bytes(),
            );
            let k_region = Self::hmac_sha256(&k_date, region.as_bytes());
            let k_service = Self::hmac_sha256(&k_region, service.as_bytes());
            Self::hmac_sha256(&k_service, b"aws4_request")
        };

        let signature = hex::encode(Self::hmac_sha256(&signing_key, string_to_sign.as_bytes()));

        let auth_header = format!(
            "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
            access_key, credential_scope, signed_headers, signature,
        );

        Ok(builder
            .header("x-amz-date", &amzdate)
            .header("x-amz-content-sha256", &payload_hash)
            .header("Authorization", auth_header))
    }

    fn build_digest_header(
        auth: &AuthConfig,
        env: &HashMap<String, String>,
        method: &str,
        uri: &str,
        www_auth: &str,
    ) -> String {
        let username = Self::resolve_env(auth.username.as_deref().unwrap_or(""), env);
        let password = Self::resolve_env(auth.password.as_deref().unwrap_or(""), env);

        // Parse WWW-Authenticate: Digest realm="...", nonce="...", qop="...", opaque="..."
        fn extract<'a>(src: &'a str, key: &str) -> &'a str {
            let search = format!("{}=\"", key);
            if let Some(start) = src.find(&search) {
                let rest = &src[start + search.len()..];
                if let Some(end) = rest.find('"') {
                    return &rest[..end];
                }
            }
            ""
        }

        let realm = extract(www_auth, "realm");
        let nonce = extract(www_auth, "nonce");
        let opaque = extract(www_auth, "opaque");
        let qop_str = extract(www_auth, "qop");
        let use_auth_qop = qop_str.contains("auth");

        let ha1 = {
            let mut h = Md5::new();
            h.update(format!("{}:{}:{}", username, realm, password).as_bytes());
            hex::encode(h.finalize())
        };
        let ha2 = {
            let mut h = Md5::new();
            h.update(format!("{}:{}", method, uri).as_bytes());
            hex::encode(h.finalize())
        };

        let nc = "00000001";
        let cnonce = format!("{:x}", rand_u32());

        let response_hash = if use_auth_qop {
            let mut h = Md5::new();
            h.update(format!("{}:{}:{}:{}:auth:{}", ha1, nonce, nc, cnonce, ha2).as_bytes());
            hex::encode(h.finalize())
        } else {
            let mut h = Md5::new();
            h.update(format!("{}:{}:{}", ha1, nonce, ha2).as_bytes());
            hex::encode(h.finalize())
        };

        let mut header = format!(
            r#"Digest username="{}", realm="{}", nonce="{}", uri="{}", response="{}""#,
            username, realm, nonce, uri, response_hash,
        );
        if use_auth_qop {
            header.push_str(&format!(r#", qop=auth, nc={}, cnonce="{}""#, nc, cnonce));
        }
        if !opaque.is_empty() {
            header.push_str(&format!(r#", opaque="{}""#, opaque));
        }
        header
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
