/**
 * Parallax CORS Proxy — Cloudflare Worker
 *
 * Routes:
 *   POST /exchange   — GitHub OAuth code → access_token exchange
 *                      Body: { code: string }
 *                      Env vars: GITHUB_CLIENT_ID, GITHUB_CLIENT_SECRET
 *
 *   ANY  /proxy/*    — Transparent CORS proxy for arbitrary HTTP targets.
 *                      The real URL is passed in the X-Proxy-URL header.
 *                      Parallax uses this to reach services that don't set
 *                      Access-Control-Allow-Origin from the browser.
 *
 * Deployment:
 *   1. wrangler secret put GITHUB_CLIENT_ID
 *   2. wrangler secret put GITHUB_CLIENT_SECRET
 *   3. wrangler deploy
 */

const ALLOWED_ORIGINS = [
  "https://parallax.pages.dev",
  "http://localhost:1421",   // local dev:web
  "http://localhost:5173",   // fallback Vite default
];

function corsHeaders(origin) {
  const allowed = ALLOWED_ORIGINS.includes(origin) ? origin : ALLOWED_ORIGINS[0];
  return {
    "Access-Control-Allow-Origin":  allowed,
    "Access-Control-Allow-Methods": "GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD",
    "Access-Control-Allow-Headers": "Content-Type, Authorization, X-Proxy-URL, X-Requested-With",
    "Access-Control-Max-Age":       "86400",
    "Vary":                          "Origin",
  };
}

function preflight(origin) {
  return new Response(null, { status: 204, headers: corsHeaders(origin) });
}

// ── /exchange — GitHub OAuth code-for-token ───────────────────────────────────

async function handleExchange(request, env, origin) {
  if (request.method !== "POST") {
    return new Response("Method Not Allowed", { status: 405, headers: corsHeaders(origin) });
  }

  let body;
  try {
    body = await request.json();
  } catch {
    return new Response("Bad JSON body", { status: 400, headers: corsHeaders(origin) });
  }

  const { code } = body;
  if (!code) {
    return new Response("Missing 'code' field", { status: 400, headers: corsHeaders(origin) });
  }

  const res = await fetch("https://github.com/login/oauth/access_token", {
    method: "POST",
    headers: {
      Accept: "application/json",
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      client_id:     env.GITHUB_CLIENT_ID,
      client_secret: env.GITHUB_CLIENT_SECRET,
      code,
    }),
  });

  const data = await res.json();

  if (!res.ok || data.error) {
    return new Response(
      JSON.stringify({ error: data.error ?? "exchange_failed", description: data.error_description }),
      { status: 400, headers: { "Content-Type": "application/json", ...corsHeaders(origin) } },
    );
  }

  return new Response(
    JSON.stringify({ access_token: data.access_token }),
    { status: 200, headers: { "Content-Type": "application/json", ...corsHeaders(origin) } },
  );
}

// ── /proxy/* — transparent CORS proxy ────────────────────────────────────────

async function handleProxy(request, origin) {
  const targetUrl = request.headers.get("X-Proxy-URL");
  if (!targetUrl) {
    return new Response("Missing X-Proxy-URL header", { status: 400, headers: corsHeaders(origin) });
  }

  // Strip the proxy-specific header before forwarding
  const forwardHeaders = new Headers(request.headers);
  forwardHeaders.delete("X-Proxy-URL");
  forwardHeaders.delete("Origin");   // prevent the target from seeing our worker origin

  let bodyInit;
  if (!["GET", "HEAD"].includes(request.method)) {
    bodyInit = await request.arrayBuffer();
  }

  let upstreamRes;
  try {
    upstreamRes = await fetch(targetUrl, {
      method:  request.method,
      headers: forwardHeaders,
      body:    bodyInit,
      redirect: "follow",
    });
  } catch (e) {
    return new Response(`Upstream fetch failed: ${e.message}`, {
      status: 502,
      headers: corsHeaders(origin),
    });
  }

  // Rebuild response with CORS headers added
  const responseHeaders = new Headers(upstreamRes.headers);
  for (const [k, v] of Object.entries(corsHeaders(origin))) {
    responseHeaders.set(k, v);
  }
  // Remove upstream CORS headers that would conflict
  responseHeaders.delete("Access-Control-Allow-Origin");
  for (const [k, v] of Object.entries(corsHeaders(origin))) {
    responseHeaders.set(k, v);
  }

  return new Response(upstreamRes.body, {
    status:  upstreamRes.status,
    headers: responseHeaders,
  });
}

// ── Main handler ──────────────────────────────────────────────────────────────

export default {
  async fetch(request, env) {
    const origin = request.headers.get("Origin") ?? "";
    const url    = new URL(request.url);

    if (request.method === "OPTIONS") return preflight(origin);

    if (url.pathname === "/exchange") return handleExchange(request, env, origin);
    if (url.pathname.startsWith("/proxy/")) return handleProxy(request, origin);

    return new Response("Not Found", { status: 404, headers: corsHeaders(origin) });
  },
};
