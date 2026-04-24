import type { CollectionRequest } from "../stores/app.svelte";

export type CodeLang =
  | "curl" | "python" | "javascript" | "typescript"
  | "rust" | "go" | "php" | "ruby" | "java" | "csharp"
  | "swift" | "kotlin" | "powershell";

export const CODE_LANGS: { id: CodeLang; label: string }[] = [
  { id: "curl",        label: "cURL" },
  { id: "python",      label: "Python" },
  { id: "javascript",  label: "JavaScript" },
  { id: "typescript",  label: "TypeScript" },
  { id: "rust",        label: "Rust" },
  { id: "go",          label: "Go" },
  { id: "php",         label: "PHP" },
  { id: "ruby",        label: "Ruby" },
  { id: "java",        label: "Java" },
  { id: "csharp",      label: "C#" },
  { id: "swift",       label: "Swift" },
  { id: "kotlin",      label: "Kotlin" },
  { id: "powershell",  label: "PowerShell" },
];

interface GenRequest {
  method: string;
  url: string;
  headers?: Record<string, string>;
  params?: Record<string, string>;
  body?: { type: string; content: unknown; raw?: string };
  auth?: { type: string; token?: string; username?: string; password?: string };
}

function buildUrl(req: GenRequest): string {
  const base = req.url;
  if (!req.params || Object.keys(req.params).length === 0) return base;
  const qs = Object.entries(req.params)
    .filter(([, v]) => v !== "")
    .map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`)
    .join("&");
  return qs ? `${base}?${qs}` : base;
}

function bodyString(req: GenRequest): string | null {
  if (!req.body || req.body.type === "none") return null;
  if (req.body.raw) return req.body.raw;
  return JSON.stringify(req.body.content, null, 2);
}

function headerEntries(req: GenRequest): [string, string][] {
  const h: [string, string][] = [];
  if (req.auth?.type === "bearer" && req.auth.token) {
    h.push(["Authorization", `Bearer ${req.auth.token}`]);
  } else if (req.auth?.type === "basic" && req.auth.username) {
    const b64 = btoa(`${req.auth.username}:${req.auth.password ?? ""}`);
    h.push(["Authorization", `Basic ${b64}`]);
  }
  for (const [k, v] of Object.entries(req.headers ?? {})) {
    h.push([k, v]);
  }
  if (req.body?.type === "json" && !h.find(([k]) => k.toLowerCase() === "content-type")) {
    h.push(["Content-Type", "application/json"]);
  } else if (req.body?.type === "urlencoded") {
    h.push(["Content-Type", "application/x-www-form-urlencoded"]);
  }
  return h;
}

// ── Generators ──────────────────────────────────────────────────────────────

function genCurl(req: GenRequest): string {
  const url = buildUrl(req);
  const method = req.method.toUpperCase();
  const lines: string[] = [`curl -X ${method} '${url}'`];
  for (const [k, v] of headerEntries(req)) {
    lines.push(`  -H '${k}: ${v}'`);
  }
  const body = bodyString(req);
  if (body) lines.push(`  -d '${body.replace(/'/g, "'\\''")}'`);
  return lines.join(" \\\n");
}

function genPython(req: GenRequest): string {
  const url = buildUrl(req);
  const headers = headerEntries(req);
  const body = bodyString(req);
  const lines: string[] = ["import requests", ""];
  if (headers.length) {
    lines.push("headers = {");
    for (const [k, v] of headers) lines.push(`    "${k}": "${v}",`);
    lines.push("}");
    lines.push("");
  }
  if (body) {
    if (req.body?.type === "json") {
      lines.push(`payload = ${body}`);
    } else {
      lines.push(`payload = """${body}"""`);
    }
    lines.push("");
  }
  const hArg = headers.length ? ", headers=headers" : "";
  const dArg = body ? (req.body?.type === "json" ? ", json=payload" : ", data=payload") : "";
  lines.push(`response = requests.${req.method.toLowerCase()}("${url}"${hArg}${dArg})`);
  lines.push("print(response.status_code, response.json())");
  return lines.join("\n");
}

function genJavaScript(req: GenRequest, ts = false): string {
  const url = buildUrl(req);
  const headers = headerEntries(req);
  const body = bodyString(req);
  const lines: string[] = [];
  const hObj = headers.length
    ? `{\n${headers.map(([k, v]) => `    "${k}": "${v}"`).join(",\n")}\n  }`
    : "{}";
  const opts: string[] = [`method: "${req.method.toUpperCase()}"`, `headers: ${hObj}`];
  if (body) opts.push(`body: ${req.body?.type === "json" ? `JSON.stringify(${body})` : `\`${body}\``}`);
  const asyncFn = ts ? "async function request(): Promise<void>" : "async function request()";
  lines.push(`${asyncFn} {`);
  lines.push(`  const response = await fetch("${url}", {`);
  for (let i = 0; i < opts.length; i++) {
    lines.push(`    ${opts[i]}${i < opts.length - 1 ? "," : ""}`);
  }
  lines.push("  });");
  lines.push("  const data = await response.json();");
  lines.push("  console.log(data);");
  lines.push("}");
  lines.push("");
  lines.push("request();");
  return lines.join("\n");
}

function genRust(req: GenRequest): string {
  const url = buildUrl(req);
  const headers = headerEntries(req);
  const body = bodyString(req);
  const lines: string[] = [
    "use reqwest::header::{HeaderMap, HeaderName, HeaderValue};",
    "",
    "#[tokio::main]",
    "async fn main() -> Result<(), reqwest::Error> {",
    "    let client = reqwest::Client::new();",
  ];
  if (headers.length) {
    lines.push("    let mut headers = HeaderMap::new();");
    for (const [k, v] of headers) {
      lines.push(`    headers.insert(HeaderName::from_static("${k.toLowerCase()}"), HeaderValue::from_static("${v}"));`);
    }
  }
  const hArg = headers.length ? ".headers(headers)" : "";
  const method = req.method.toLowerCase();
  const validMethods = ["get", "post", "put", "patch", "delete", "head"];
  const m = validMethods.includes(method) ? method : "request(reqwest::Method::from_bytes(b\"" + method.toUpperCase() + "\").unwrap(), ";
  if (body && req.body?.type === "json") {
    lines.push(`    let body = serde_json::json!(${body});`);
    lines.push(`    let response = client.${m}("${url}")${hArg}.json(&body).send().await?;`);
  } else if (body) {
    lines.push(`    let response = client.${m}("${url}")${hArg}.body(r#"${body}"#).send().await?;`);
  } else {
    lines.push(`    let response = client.${m}("${url}")${hArg}.send().await?;`);
  }
  lines.push("    println!(\"{:?}\", response.status());");
  lines.push("    Ok(())");
  lines.push("}");
  return lines.join("\n");
}

function genGo(req: GenRequest): string {
  const url = buildUrl(req);
  const headers = headerEntries(req);
  const body = bodyString(req);
  const lines: string[] = [
    'package main',
    '',
    'import (',
    '    "fmt"',
    '    "net/http"',
  ];
  if (body) lines.push('    "strings"');
  lines.push(')');
  lines.push('');
  lines.push('func main() {');
  if (body) {
    lines.push(`    body := strings.NewReader(\`${body}\`)`);
    lines.push(`    req, _ := http.NewRequest("${req.method.toUpperCase()}", "${url}", body)`);
  } else {
    lines.push(`    req, _ := http.NewRequest("${req.method.toUpperCase()}", "${url}", nil)`);
  }
  for (const [k, v] of headers) {
    lines.push(`    req.Header.Set("${k}", "${v}")`);
  }
  lines.push('    client := &http.Client{}');
  lines.push('    resp, _ := client.Do(req)');
  lines.push('    defer resp.Body.Close()');
  lines.push('    fmt.Println(resp.Status)');
  lines.push('}');
  return lines.join("\n");
}

function genPHP(req: GenRequest): string {
  const url = buildUrl(req);
  const headers = headerEntries(req);
  const body = bodyString(req);
  const lines: string[] = [
    "<?php",
    "$client = new GuzzleHttp\\Client();",
    "$options = [",
  ];
  if (headers.length) {
    lines.push("    'headers' => [");
    for (const [k, v] of headers) lines.push(`        '${k}' => '${v}',`);
    lines.push("    ],");
  }
  if (body) {
    if (req.body?.type === "json") lines.push(`    'json' => json_decode('${body}', true),`);
    else lines.push(`    'body' => '${body}',`);
  }
  lines.push("];");
  lines.push(`$response = $client->request('${req.method.toUpperCase()}', '${url}', $options);`);
  lines.push("echo $response->getBody();");
  return lines.join("\n");
}

function genRuby(req: GenRequest): string {
  const url = buildUrl(req);
  const headers = headerEntries(req);
  const body = bodyString(req);
  const uri = `URI.parse('${url}')`;
  const lines: string[] = [
    "require 'net/http'",
    "require 'json'",
    "",
    `uri = ${uri}`,
    `http = Net::HTTP.new(uri.host, uri.port)`,
    "http.use_ssl = uri.scheme == 'https'",
    "",
    `request = Net::HTTP::${req.method.charAt(0).toUpperCase() + req.method.slice(1).toLowerCase()}.new(uri)`,
  ];
  for (const [k, v] of headers) lines.push(`request['${k}'] = '${v}'`);
  if (body) lines.push(`request.body = '${body.replace(/'/g, "\\'")}'`);
  lines.push("");
  lines.push("response = http.request(request)");
  lines.push("puts response.body");
  return lines.join("\n");
}

function genJava(req: GenRequest): string {
  const url = buildUrl(req);
  const headers = headerEntries(req);
  const body = bodyString(req);
  const lines: string[] = [
    "import java.net.http.*;",
    "import java.net.URI;",
    "import java.time.Duration;",
    "",
    "public class Request {",
    "    public static void main(String[] args) throws Exception {",
    "        var client = HttpClient.newBuilder()",
    "            .connectTimeout(Duration.ofSeconds(10)).build();",
  ];
  if (body) {
    lines.push(`        var body = HttpRequest.BodyPublishers.ofString("${body.replace(/"/g, '\\"').replace(/\n/g, "\\n")}");`);
  }
  lines.push(`        var request = HttpRequest.newBuilder()`);
  lines.push(`            .uri(URI.create("${url}"))`);
  for (const [k, v] of headers) lines.push(`            .header("${k}", "${v}")`);
  if (body) {
    lines.push(`            .method("${req.method.toUpperCase()}", body)`);
  } else {
    lines.push(`            .${req.method.toUpperCase() === "GET" ? "GET()" : `method("${req.method.toUpperCase()}", HttpRequest.BodyPublishers.noBody())`}`);
  }
  lines.push("            .build();");
  lines.push("        var response = client.send(request, HttpResponse.BodyHandlers.ofString());");
  lines.push("        System.out.println(response.body());");
  lines.push("    }");
  lines.push("}");
  return lines.join("\n");
}

function genCSharp(req: GenRequest): string {
  const url = buildUrl(req);
  const headers = headerEntries(req);
  const body = bodyString(req);
  const lines: string[] = [
    "using System.Net.Http;",
    "using System.Text;",
    "",
    "var client = new HttpClient();",
  ];
  for (const [k, v] of headers) {
    if (k.toLowerCase() === "content-type") continue;
    lines.push(`client.DefaultRequestHeaders.Add("${k}", "${v}");`);
  }
  if (body) {
    const ct = req.body?.type === "json" ? "application/json" : "text/plain";
    lines.push(`var content = new StringContent("""${body}""", Encoding.UTF8, "${ct}");`);
    lines.push(`var response = await client.${req.method.charAt(0).toUpperCase() + req.method.slice(1).toLowerCase()}Async("${url}", content);`);
  } else {
    lines.push(`var response = await client.${req.method.toUpperCase() === "GET" ? "GetAsync" : `SendAsync(new HttpRequestMessage(HttpMethod.${req.method.charAt(0).toUpperCase() + req.method.slice(1).toLowerCase()}, "${url}"))`}("${url}");`);
  }
  lines.push("var responseBody = await response.Content.ReadAsStringAsync();");
  lines.push("Console.WriteLine(responseBody);");
  return lines.join("\n");
}

function genSwift(req: GenRequest): string {
  const url = buildUrl(req);
  const headers = headerEntries(req);
  const body = bodyString(req);
  const lines: string[] = [
    "import Foundation",
    "",
    `let url = URL(string: "${url}")!`,
    "var request = URLRequest(url: url)",
    `request.httpMethod = "${req.method.toUpperCase()}"`,
  ];
  for (const [k, v] of headers) lines.push(`request.setValue("${v}", forHTTPHeaderField: "${k}")`);
  if (body) lines.push(`request.httpBody = Data("""${body}""".utf8)`);
  lines.push("");
  lines.push("let task = URLSession.shared.dataTask(with: request) { data, response, error in");
  lines.push("    if let data = data { print(String(data: data, encoding: .utf8) ?? \"\") }");
  lines.push("}");
  lines.push("task.resume()");
  return lines.join("\n");
}

function genKotlin(req: GenRequest): string {
  const url = buildUrl(req);
  const headers = headerEntries(req);
  const body = bodyString(req);
  const lines: string[] = [
    "import okhttp3.*",
    "import okhttp3.MediaType.Companion.toMediaType",
    "import okhttp3.RequestBody.Companion.toRequestBody",
    "",
    "val client = OkHttpClient()",
    "",
    "val request = Request.Builder()",
    `    .url("${url}")`,
  ];
  for (const [k, v] of headers.filter(([k]) => k.toLowerCase() !== "content-type")) {
    lines.push(`    .addHeader("${k}", "${v}")`);
  }
  if (body) {
    const ct = req.body?.type === "json" ? "application/json" : "text/plain";
    lines.push(`    .${req.method.toLowerCase()}("${body.replace(/"/g, '\\"')}".toRequestBody("${ct}".toMediaType()))`);
  } else {
    lines.push(`    .${req.method.toLowerCase() === "get" ? "get()" : `method("${req.method.toUpperCase()}", null)`}`);
  }
  lines.push("    .build()");
  lines.push("");
  lines.push("val response = client.newCall(request).execute()");
  lines.push("println(response.body?.string())");
  return lines.join("\n");
}

function genPowerShell(req: GenRequest): string {
  const url = buildUrl(req);
  const headers = headerEntries(req);
  const body = bodyString(req);
  const lines: string[] = [];
  if (headers.length) {
    lines.push("$headers = @{");
    for (const [k, v] of headers) lines.push(`    "${k}" = "${v}"`);
    lines.push("}");
  }
  const hArg = headers.length ? " -Headers $headers" : "";
  if (body) {
    lines.push(`$body = @'\n${body}\n'@`);
    lines.push(`$response = Invoke-RestMethod -Method ${req.method.toUpperCase()} -Uri "${url}"${hArg} -Body $body`);
  } else {
    lines.push(`$response = Invoke-RestMethod -Method ${req.method.toUpperCase()} -Uri "${url}"${hArg}`);
  }
  lines.push("$response | ConvertTo-Json");
  return lines.join("\n");
}

export function generateCode(lang: CodeLang, req: CollectionRequest | GenRequest): string {
  const r = req as GenRequest;
  switch (lang) {
    case "curl":        return genCurl(r);
    case "python":      return genPython(r);
    case "javascript":  return genJavaScript(r, false);
    case "typescript":  return genJavaScript(r, true);
    case "rust":        return genRust(r);
    case "go":          return genGo(r);
    case "php":         return genPHP(r);
    case "ruby":        return genRuby(r);
    case "java":        return genJava(r);
    case "csharp":      return genCSharp(r);
    case "swift":       return genSwift(r);
    case "kotlin":      return genKotlin(r);
    case "powershell":  return genPowerShell(r);
    default:            return "// Unsupported language";
  }
}
