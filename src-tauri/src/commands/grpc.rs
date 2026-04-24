use tauri::Emitter;

/// gRPC unary call command
/// Uses HTTP/2 with application/grpc+json content-type and standard 5-byte length-prefix framing.
/// Compatible with gRPC-JSON transcoding servers (grpc-gateway, Connect, etc.).
/// For plain protobuf gRPC servers, pass request_json as a JSON object — the server receives
/// it as raw bytes in the gRPC frame.
use std::collections::HashMap;
use std::time::Instant;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GrpcResponse {
    pub grpc_status: i32,
    pub grpc_message: String,
    pub http_status: u16,
    pub message_json: String,
    pub headers: HashMap<String, String>,
    pub trailers: HashMap<String, String>,
    pub duration_ms: u64,
}

#[tauri::command]
pub async fn grpc_unary(
    server_addr: String,
    method_path: String,
    request_json: String,
    metadata: HashMap<String, String>,
    tls_skip_verify: Option<bool>,
) -> Result<GrpcResponse, String> {
    let mut client_builder = reqwest::Client::builder()
        .http2_prior_knowledge();

    if tls_skip_verify.unwrap_or(false) {
        client_builder = client_builder.danger_accept_invalid_certs(true);
    }

    let client = client_builder.build().map_err(|e| e.to_string())?;

    // gRPC length-prefixed message framing: 1-byte flags + 4-byte big-endian length + body
    let body_bytes = request_json.as_bytes();
    let mut framed: Vec<u8> = Vec::with_capacity(5 + body_bytes.len());
    framed.push(0u8); // compressed flag = 0
    framed.extend_from_slice(&(body_bytes.len() as u32).to_be_bytes());
    framed.extend_from_slice(body_bytes);

    let base = server_addr.trim_end_matches('/');
    let path = method_path.trim_start_matches('/');
    let url = format!("{}/{}", base, path);

    let mut req_builder = client
        .post(&url)
        .header("content-type", "application/grpc+json")
        .header("te", "trailers")
        .header("user-agent", "Parallax-gRPC/0.1.0");

    for (k, v) in &metadata {
        req_builder = req_builder.header(k, v);
    }

    let start = Instant::now();
    let response = req_builder
        .body(framed)
        .send()
        .await
        .map_err(|e| format!("gRPC connect failed: {}", e))?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let http_status = response.status().as_u16();

    let headers: HashMap<String, String> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let grpc_status = headers
        .get("grpc-status")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);

    let grpc_message = headers
        .get("grpc-message")
        .cloned()
        .unwrap_or_default();

    let body_bytes = response.bytes().await.map_err(|e| e.to_string())?;

    // Decode gRPC frame: skip 5-byte header, rest is the message body
    let message_json = if body_bytes.len() >= 5 {
        let msg = &body_bytes[5..];
        // Try to pretty-print if valid JSON
        String::from_utf8_lossy(msg)
            .to_string()
            .trim()
            .to_string()
    } else {
        String::new()
    };

    // gRPC trailers may arrive as HTTP/2 trailing HEADERS frame
    // (reqwest exposes them as headers on the response object for HTTP/2)
    let trailers: HashMap<String, String> = headers
        .iter()
        .filter(|(k, _)| k.starts_with("grpc-"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    Ok(GrpcResponse {
        grpc_status,
        grpc_message,
        http_status,
        message_json,
        headers,
        trailers,
        duration_ms,
    })
}

/// gRPC server-streaming call — emits `grpc_stream_message` Tauri events for each response frame,
/// then emits `grpc_stream_end` when the stream closes.
/// Uses HTTP/2 + application/grpc+json framing (same as grpc_unary).
#[tauri::command]
pub async fn grpc_server_stream(
    app: tauri::AppHandle,
    server_addr: String,
    method_path: String,
    request_json: String,
    metadata: HashMap<String, String>,
    stream_id: String,
    tls_skip_verify: Option<bool>,
) -> Result<(), String> {
    let mut client_builder = reqwest::Client::builder().http2_prior_knowledge();
    if tls_skip_verify.unwrap_or(false) {
        client_builder = client_builder.danger_accept_invalid_certs(true);
    }
    let client = client_builder.build().map_err(|e| e.to_string())?;

    let body_bytes = request_json.as_bytes();
    let mut framed: Vec<u8> = Vec::with_capacity(5 + body_bytes.len());
    framed.push(0u8);
    framed.extend_from_slice(&(body_bytes.len() as u32).to_be_bytes());
    framed.extend_from_slice(body_bytes);

    let base = server_addr.trim_end_matches('/');
    let path = method_path.trim_start_matches('/');
    let url = format!("{}/{}", base, path);

    let mut req_builder = client
        .post(&url)
        .header("content-type", "application/grpc+json")
        .header("te", "trailers")
        .header("user-agent", "Parallax-gRPC/0.1.0");

    for (k, v) in &metadata {
        req_builder = req_builder.header(k, v);
    }

    let response = req_builder
        .body(framed)
        .send()
        .await
        .map_err(|e| format!("gRPC connect failed: {}", e))?;

    let grpc_status_header = response
        .headers()
        .get("grpc-status")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("0")
        .to_string();

    use futures_util::StreamExt;
    let mut stream = response.bytes_stream();
    let mut buf: Vec<u8> = Vec::new();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(bytes) => {
                buf.extend_from_slice(&bytes);
                // Parse complete gRPC frames from buf
                loop {
                    if buf.len() < 5 { break; }
                    let msg_len = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;
                    if buf.len() < 5 + msg_len { break; }
                    let msg_bytes = &buf[5..5 + msg_len];
                    let msg_json = String::from_utf8_lossy(msg_bytes).trim().to_string();
                    let _ = app.emit(&format!("grpc_stream_message_{}", stream_id), msg_json);
                    buf.drain(..5 + msg_len);
                }
            }
            Err(e) => {
                let _ = app.emit(&format!("grpc_stream_end_{}", stream_id),
                    serde_json::json!({ "error": e.to_string(), "grpc_status": -1 }));
                return Ok(());
            }
        }
    }

    let _ = app.emit(&format!("grpc_stream_end_{}", stream_id),
        serde_json::json!({ "grpc_status": grpc_status_header }));
    Ok(())
}

/// gRPC Server Reflection — lists services and their methods using the
/// grpc.reflection.v1alpha.ServerReflection protocol.
/// Uses hand-encoded protobuf to avoid code generation dependencies.
#[tauri::command]
pub async fn grpc_reflect(
    server_addr: String,
    tls_skip_verify: Option<bool>,
) -> Result<Vec<String>, String> {
    let mut client_builder = reqwest::Client::builder().http2_prior_knowledge();
    if tls_skip_verify.unwrap_or(false) {
        client_builder = client_builder.danger_accept_invalid_certs(true);
    }
    let client = client_builder.build().map_err(|e| e.to_string())?;

    let base = server_addr.trim_end_matches('/');

    // ── Step 1: ListServices ─────────────────────────────────────────────────
    // ServerReflectionRequest { list_services: "" } — field 7, wire type 2, length 0
    // protobuf: tag=(7<<3)|2=0x3A, len=0x00
    let list_req_proto: Vec<u8> = vec![0x3A, 0x00];
    let services = reflect_call(&client, base, &list_req_proto).await
        .map_err(|e| e.to_string())?;

    // Parse service names from ListServicesResponse
    // ServerReflectionResponse.list_services_response (field 4):
    //   ListServiceResponse.service (field 1, repeated):
    //     ServiceResponse.name (field 1, string)
    let service_names = parse_list_services(&services);
    if service_names.is_empty() {
        return Ok(vec![]);
    }

    // ── Step 2: FileContainingSymbol for each service ─────────────────────────
    let mut methods: Vec<String> = Vec::new();
    for svc in &service_names {
        // ServerReflectionRequest { file_containing_symbol: svc } — field 4, wire type 2
        let sym_proto = encode_string_field(4, svc);
        if let Ok(resp_bytes) = reflect_call(&client, base, &sym_proto).await {
            let svc_methods = parse_file_descriptor_methods(&resp_bytes, svc);
            methods.extend(svc_methods);
        }
        if methods.is_empty() {
            // Fallback: just expose service name so user can type method manually
            methods.push(format!("{}/", svc));
        }
    }

    Ok(methods)
}

/// Send one ServerReflectionRequest body (already proto-encoded, without framing)
/// over a gRPC server-streaming call and return the first response body bytes.
async fn reflect_call(client: &reqwest::Client, base: &str, req_proto: &[u8]) -> anyhow::Result<Vec<u8>> {
    use futures_util::StreamExt;

    let mut framed: Vec<u8> = Vec::with_capacity(5 + req_proto.len());
    framed.push(0u8);
    framed.extend_from_slice(&(req_proto.len() as u32).to_be_bytes());
    framed.extend_from_slice(req_proto);

    let url = format!("{}/grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo", base);

    let response = client
        .post(&url)
        .header("content-type", "application/grpc+proto")
        .header("te", "trailers")
        .header("user-agent", "Parallax-gRPC/0.1.0")
        .body(framed)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("gRPC reflection connect: {}", e))?;

    let mut stream = response.bytes_stream();
    let mut buf: Vec<u8> = Vec::new();

    while let Some(chunk) = stream.next().await {
        if let Ok(bytes) = chunk {
            buf.extend_from_slice(&bytes);
        }
    }

    // Return the first gRPC frame body
    if buf.len() >= 5 {
        let msg_len = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;
        if buf.len() >= 5 + msg_len {
            return Ok(buf[5..5 + msg_len].to_vec());
        }
    }
    Ok(buf)
}

/// Encode a single string protobuf field: field_num (wire type 2) + varint length + bytes
fn encode_string_field(field_num: u32, value: &str) -> Vec<u8> {
    let tag = (field_num << 3) | 2;
    let bytes = value.as_bytes();
    let mut out = Vec::new();
    encode_varint(&mut out, tag as u64);
    encode_varint(&mut out, bytes.len() as u64);
    out.extend_from_slice(bytes);
    out
}

fn encode_varint(buf: &mut Vec<u8>, mut v: u64) {
    loop {
        if v < 0x80 { buf.push(v as u8); break; }
        buf.push((v as u8 & 0x7f) | 0x80);
        v >>= 7;
    }
}

/// Decode a protobuf varint from a byte slice, returning (value, bytes_consumed)
fn decode_varint(buf: &[u8]) -> Option<(u64, usize)> {
    let mut value: u64 = 0;
    let mut shift = 0u32;
    for (i, &b) in buf.iter().enumerate() {
        value |= ((b & 0x7f) as u64) << shift;
        if b & 0x80 == 0 { return Some((value, i + 1)); }
        shift += 7;
        if shift >= 64 { return None; }
    }
    None
}

/// Walk a protobuf-encoded byte slice and call `cb` for each field
fn walk_proto(buf: &[u8], mut cb: impl FnMut(u32, u8, &[u8])) {
    let mut pos = 0;
    while pos < buf.len() {
        let Some((tag, tn)) = decode_varint(&buf[pos..]) else { break };
        pos += tn;
        let field_num = (tag >> 3) as u32;
        let wire_type = (tag & 0x7) as u8;
        match wire_type {
            0 => { // varint
                let Some((_, vn)) = decode_varint(&buf[pos..]) else { break };
                pos += vn;
            }
            1 => { pos += 8; } // 64-bit
            2 => { // length-delimited
                let Some((len, ln)) = decode_varint(&buf[pos..]) else { break };
                pos += ln;
                let end = pos + len as usize;
                if end > buf.len() { break; }
                cb(field_num, wire_type, &buf[pos..end]);
                pos = end;
            }
            5 => { pos += 4; } // 32-bit
            _ => break,
        }
    }
}

/// Parse ListServicesResponse from ServerReflectionResponse bytes
/// ServerReflectionResponse.list_services_response = field 4
/// ListServiceResponse.service (repeated) = field 1
/// ServiceResponse.name = field 1 (string)
fn parse_list_services(buf: &[u8]) -> Vec<String> {
    let mut services = Vec::new();
    walk_proto(buf, |field, _, data| {
        if field == 4 {
            // ListServiceResponse
            walk_proto(data, |f2, _, svc_data| {
                if f2 == 1 {
                    // ServiceResponse
                    walk_proto(svc_data, |f3, _, name_bytes| {
                        if f3 == 1 {
                            if let Ok(s) = std::str::from_utf8(name_bytes) {
                                services.push(s.to_string());
                            }
                        }
                    });
                }
            });
        }
    });
    services
}

/// Parse a FileDescriptorProto from a FileDescriptorResponse to extract method paths
/// ServerReflectionResponse.file_descriptor_response = field 5
/// FileDescriptorResponse.file_descriptor_proto (repeated bytes) = field 1
/// FileDescriptorProto.service (repeated ServiceDescriptorProto) = field 6
/// ServiceDescriptorProto.name = field 1, .method (repeated MethodDescriptorProto) = field 2
/// MethodDescriptorProto.name = field 1
fn parse_file_descriptor_methods(buf: &[u8], service_hint: &str) -> Vec<String> {
    let mut methods = Vec::new();

    walk_proto(buf, |field, _, data| {
        if field == 5 {
            // FileDescriptorResponse
            walk_proto(data, |f2, _, fd_bytes| {
                if f2 == 1 {
                    // FileDescriptorProto — parse package + services
                    let mut package = String::new();
                    walk_proto(fd_bytes, |f, _, v| {
                        if f == 2 { // package
                            if let Ok(s) = std::str::from_utf8(v) {
                                package = s.to_string();
                            }
                        }
                    });

                    walk_proto(fd_bytes, |f, _, svc_bytes| {
                        if f == 6 {
                            // ServiceDescriptorProto
                            let mut svc_name = String::new();
                            let mut method_names: Vec<String> = Vec::new();

                            walk_proto(svc_bytes, |sf, _, sv| {
                                if sf == 1 {
                                    if let Ok(s) = std::str::from_utf8(sv) {
                                        svc_name = s.to_string();
                                    }
                                } else if sf == 2 {
                                    // MethodDescriptorProto
                                    walk_proto(sv, |mf, _, mv| {
                                        if mf == 1 {
                                            if let Ok(s) = std::str::from_utf8(mv) {
                                                method_names.push(s.to_string());
                                            }
                                        }
                                    });
                                }
                            });

                            let full_svc = if package.is_empty() {
                                svc_name.clone()
                            } else {
                                format!("{}.{}", package, svc_name)
                            };

                            for m in &method_names {
                                methods.push(format!("{}/{}", full_svc, m));
                            }
                        }
                    });
                }
            });
        }
    });

    // If we got nothing but we know the service name, fallback to just the service path
    if methods.is_empty() && !service_hint.is_empty() {
        methods.push(format!("{}/", service_hint));
    }

    methods
}
