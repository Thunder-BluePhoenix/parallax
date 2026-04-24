/// Parallax HTTP Engine — Rust/reqwest powered HTTP/2+HTTP/3 execution core
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use reqwest::{Client, Method, Response};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest as Sha2Digest};
use md5::Md5;
use md4::Md4;
type HmacSha256 = Hmac<Sha256>;
type HmacMd5 = Hmac<Md5>;

fn rand_u32() -> u32 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().subsec_nanos() ^ 0xdeadbeef
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
    // NTLM fields
    pub ntlm_domain: Option<String>,
    pub ntlm_workstation: Option<String>,
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
    /// NTLM / NTLMv2 (Windows domains, IIS, SSPI-compatible servers)
    Ntlm,
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

        // NTLM auth — 3-way handshake: negotiate → challenge → authenticate
        if let Some(auth) = &req.auth {
            if auth.auth_type == AuthType::Ntlm {
                let username = Self::resolve_env(auth.username.as_deref().unwrap_or(""), env);
                let password = Self::resolve_env(auth.password.as_deref().unwrap_or(""), env);
                let domain = Self::resolve_env(auth.ntlm_domain.as_deref().unwrap_or(""), env);
                let workstation = Self::resolve_env(auth.ntlm_workstation.as_deref().unwrap_or(""), env);
                let method_str = req.method.to_uppercase();

                // Step 1 — send NEGOTIATE
                let negotiate_b64 = Self::ntlm_negotiate_msg(&domain, &workstation);
                let method1 = Method::from_bytes(method_str.as_bytes()).unwrap_or(Method::GET);
                let mut b1 = self.client.request(method1, reqwest::Url::parse(&resolved_url)?);
                if let Some(ms) = req.timeout_ms { b1 = b1.timeout(Duration::from_millis(ms)); }
                for (k, v) in &req.headers {
                    b1 = b1.header(Self::resolve_env(k, env), Self::resolve_env(v, env));
                }
                b1 = b1.header("Authorization", format!("NTLM {}", negotiate_b64));
                let r1 = b1.send().await?;

                // Step 2 — parse CHALLENGE from server
                if r1.status() == 401 {
                    if let Some(challenge_hdr) = r1.headers().get("www-authenticate").cloned() {
                        let challenge_str = challenge_hdr.to_str().unwrap_or("").to_string();
                        if let Some(b64) = challenge_str.strip_prefix("NTLM ").map(|s| s.trim()) {
                            if let Ok(challenge_bytes) = base64::Engine::decode(
                                &base64::engine::general_purpose::STANDARD, b64) {
                                // Step 3 — build AUTHENTICATE
                                let auth_b64 = Self::ntlm_authenticate_msg(
                                    &username, &password, &domain, &workstation, &challenge_bytes,
                                );
                                let method3 = Method::from_bytes(method_str.as_bytes()).unwrap_or(Method::GET);
                                let mut b3 = self.client.request(method3, reqwest::Url::parse(&resolved_url)?);
                                if let Some(ms) = req.timeout_ms { b3 = b3.timeout(Duration::from_millis(ms)); }
                                for (k, v) in &req.headers {
                                    b3 = b3.header(Self::resolve_env(k, env), Self::resolve_env(v, env));
                                }
                                b3 = b3.header("Authorization", format!("NTLM {}", auth_b64));
                                if let Some(body) = &req.body {
                                    b3 = self.apply_body(b3, body, env)?;
                                }
                                response = b3.send().await?;
                            }
                        }
                    }
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
            AuthType::ApiKey
                if auth.api_key_location.as_deref() != Some("query") => {
                if let (Some(header), Some(value)) = (&auth.api_key_header, &auth.api_key_value) {
                    let header = Self::resolve_env(header, env);
                    let value = Self::resolve_env(value, env);
                    return builder.header(header, value);
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

    // ── NTLM helpers ─────────────────────────────────────────────────────────

    fn utf16le(s: &str) -> Vec<u8> {
        s.encode_utf16().flat_map(|c| c.to_le_bytes()).collect()
    }

    fn nt_hash(password: &str) -> Vec<u8> {
        let pw_utf16 = Self::utf16le(password);
        let hash = Md4::digest(&pw_utf16);
        hash.to_vec()
    }

    fn ntlmv2_hash(nt_hash: &[u8], username: &str, domain: &str) -> Vec<u8> {
        let identity = Self::utf16le(&format!("{}{}", username.to_uppercase(), domain));
        let mut mac = HmacMd5::new_from_slice(nt_hash).expect("HMAC key ok");
        mac.update(&identity);
        mac.finalize().into_bytes().to_vec()
    }

    fn write_u16le(buf: &mut Vec<u8>, v: u16) { buf.extend_from_slice(&v.to_le_bytes()); }
    fn write_u32le(buf: &mut Vec<u8>, v: u32) { buf.extend_from_slice(&v.to_le_bytes()); }

    fn ntlm_security_buffer(buf: &mut Vec<u8>, offset: u32, data: &[u8]) {
        let len = data.len() as u16;
        Self::write_u16le(buf, len);   // length
        Self::write_u16le(buf, len);   // max length
        Self::write_u32le(buf, offset); // offset
    }

    /// Build NTLM NEGOTIATE_MESSAGE (Type 1)
    fn ntlm_negotiate_msg(domain: &str, workstation: &str) -> String {
        let mut msg: Vec<u8> = Vec::new();
        msg.extend_from_slice(b"NTLMSSP\0");  // signature
        Self::write_u32le(&mut msg, 1);        // MessageType = 1
        // Negotiate flags: NTLM + Unicode + OEM + RequestTarget + NTLM + AlwaysSign + ExtendedSecurity + Version
        let flags: u32 = 0x60088215;
        Self::write_u32le(&mut msg, flags);
        // Domain and workstation security buffers (empty, at offset 40)
        let base_offset = 40u32;
        let dom_bytes = domain.as_bytes();
        let ws_bytes  = workstation.as_bytes();
        Self::ntlm_security_buffer(&mut msg, base_offset, dom_bytes);
        Self::ntlm_security_buffer(&mut msg, base_offset + dom_bytes.len() as u32, ws_bytes);
        // Version (8 bytes) — Windows 10 / NTLM revision 15
        msg.extend_from_slice(&[0x0a, 0x00, 0x63, 0x45, 0x00, 0x00, 0x00, 0x0f]);
        msg.extend_from_slice(dom_bytes);
        msg.extend_from_slice(ws_bytes);
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &msg)
    }

    /// Parse server challenge (8 bytes) from CHALLENGE_MESSAGE (Type 2)
    fn parse_ntlm_challenge(challenge_msg: &[u8]) -> Option<([u8; 8], Vec<u8>)> {
        // Challenge is at offset 24, TargetInfo at offset specified in message
        if challenge_msg.len() < 56 { return None; }
        let mut sc = [0u8; 8];
        sc.copy_from_slice(&challenge_msg[24..32]);

        // TargetInfo security buffer at offset 40: len(2) + maxlen(2) + offset(4)
        let ti_len  = u16::from_le_bytes([challenge_msg[40], challenge_msg[41]]) as usize;
        let ti_off  = u32::from_le_bytes([challenge_msg[44], challenge_msg[45], challenge_msg[46], challenge_msg[47]]) as usize;
        let target_info = if ti_off + ti_len <= challenge_msg.len() {
            challenge_msg[ti_off..ti_off + ti_len].to_vec()
        } else {
            vec![]
        };
        Some((sc, target_info))
    }

    /// Build NTLM AUTHENTICATE_MESSAGE (Type 3) with NTLMv2 response
    fn ntlm_authenticate_msg(
        username: &str, password: &str, domain: &str, workstation: &str,
        challenge_msg: &[u8],
    ) -> String {
        let (server_challenge, target_info) = Self::parse_ntlm_challenge(challenge_msg).unwrap_or_default();

        let nt_h = Self::nt_hash(password);
        let ntv2_h = Self::ntlmv2_hash(&nt_h, username, domain);

        // NTLMv2 blob — 8-byte client challenge from low/high 32 bits of now
        let client_challenge: [u8; 8] = {
            let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
            let lo = dur.subsec_nanos().to_le_bytes();
            let hi = (dur.as_secs() as u32).to_le_bytes();
            [lo[0], lo[1], lo[2], lo[3], hi[0], hi[1], hi[2], hi[3]]
        };

        // Windows FILETIME (100ns intervals since 1601-01-01)
        let filetime: u64 = {
            let unix_secs = SystemTime::now()
                .duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
            (unix_secs + 11644473600) * 10_000_000
        };

        let mut blob: Vec<u8> = Vec::new();
        blob.extend_from_slice(&[0x01, 0x01, 0x00, 0x00]); // blob signature
        blob.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // reserved
        blob.extend_from_slice(&filetime.to_le_bytes());    // timestamp
        blob.extend_from_slice(&client_challenge);
        blob.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // reserved
        blob.extend_from_slice(&target_info);
        blob.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // trailing

        // NTProofStr = HMAC-MD5(ntv2_h, server_challenge || blob)
        let mut mac = HmacMd5::new_from_slice(&ntv2_h).expect("HMAC key ok");
        mac.update(&server_challenge);
        mac.update(&blob);
        let nt_proof: Vec<u8> = mac.finalize().into_bytes().to_vec();

        // NTChallengeResponse = NTProofStr || blob
        let mut nt_response = nt_proof.clone();
        nt_response.extend_from_slice(&blob);

        // LMv2 response (simplified — use NTProofStr + zeros)
        let mut lm_response = nt_proof.clone();
        lm_response.extend_from_slice(&client_challenge);

        // Session base key
        let mut mac2 = HmacMd5::new_from_slice(&ntv2_h).expect("HMAC key ok");
        mac2.update(&nt_proof);
        let _session_base_key = mac2.finalize().into_bytes();

        // Encode fields
        let user_utf16  = Self::utf16le(username);
        let dom_utf16   = Self::utf16le(domain);
        let ws_utf16    = Self::utf16le(workstation);

        // Build message with security buffers
        // Fixed header = 12 (signature + type) + 4 (flags) = layout below
        // Layout: sig(8) + type(4) + LmChallengeResponseFields(8) + NtChallengeResponseFields(8)
        //         + DomainNameFields(8) + UserNameFields(8) + WorkstationFields(8)
        //         + EncryptedRandomSessionKeyFields(8) + NegotiateFlags(4) = 72 bytes header
        let header_len: u32 = 72;
        let lm_off  = header_len;
        let nt_off  = lm_off + lm_response.len() as u32;
        let dom_off = nt_off + nt_response.len() as u32;
        let usr_off = dom_off + dom_utf16.len() as u32;
        let ws_off  = usr_off + user_utf16.len() as u32;
        let key_off = ws_off  + ws_utf16.len() as u32;

        let mut msg: Vec<u8> = Vec::new();
        msg.extend_from_slice(b"NTLMSSP\0");
        Self::write_u32le(&mut msg, 3); // MessageType = 3
        Self::ntlm_security_buffer(&mut msg, lm_off,  &lm_response);
        Self::ntlm_security_buffer(&mut msg, nt_off,  &nt_response);
        Self::ntlm_security_buffer(&mut msg, dom_off, &dom_utf16);
        Self::ntlm_security_buffer(&mut msg, usr_off, &user_utf16);
        Self::ntlm_security_buffer(&mut msg, ws_off,  &ws_utf16);
        Self::ntlm_security_buffer(&mut msg, key_off, &[]); // encrypted session key (empty)
        Self::write_u32le(&mut msg, 0x60088215); // NegotiateFlags
        msg.extend_from_slice(&lm_response);
        msg.extend_from_slice(&nt_response);
        msg.extend_from_slice(&dom_utf16);
        msg.extend_from_slice(&user_utf16);
        msg.extend_from_slice(&ws_utf16);

        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &msg)
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
