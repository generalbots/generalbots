/**
 * sample-node-app — a minimal, dependency-free Node.js HTTP service.
 *
 * Demonstrates the General Bots incus dev → prod hosting flow:
 *   - `GET /`          → small landing page
 *   - `GET /health`    → JSON liveness/readiness probe
 *   - `GET /api/info`  → JSON payload (app name, version, uptime)
 *   - `GET /api/time`  → JSON payload (server timestamp)
 *
 * Port is configurable via PORT (default 8080). No npm install required.
 */
"use strict";

const http = require("http");

const APP_NAME = process.env.APP_NAME || "sample-node-app";
const APP_VERSION = process.env.APP_VERSION || "1.0.0";
const PORT = Number(process.env.PORT || 8080);
const startedAt = Date.now();

function uptimeSeconds() {
  return Math.floor((Date.now() - startedAt) / 1000);
}

function sendJson(res, status, payload) {
  const body = JSON.stringify(payload, null, 2);
  res.writeHead(status, {
    "Content-Type": "application/json; charset=utf-8",
    "Cache-Control": "no-store",
  });
  res.end(body + "\n");
}

function sendHtml(res, status, html) {
  res.writeHead(status, {
    "Content-Type": "text/html; charset=utf-8",
    "Cache-Control": "no-store",
  });
  res.end(html);
}

const landing = `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>${APP_NAME}</title>
  <style>
    body { font-family: system-ui, sans-serif; margin: 0; background: #0f172a; color: #e2e8f0;
           display: flex; align-items: center; justify-content: center; min-height: 100vh; }
    main { text-align: center; padding: 2rem; }
    h1 { font-size: 2.5rem; margin: 0 0 0.5rem; color: #84d669; }
    p { opacity: 0.8; }
    code { background: #1e293b; padding: 0.2rem 0.5rem; border-radius: 0.3rem; }
  </style>
</head>
<body>
  <main>
    <h1>${APP_NAME}</h1>
    <p>v${APP_VERSION} — running for <span id="up">${uptimeSeconds()}</span>s</p>
    <p>Endpoints: <code>/health</code> · <code>/api/info</code> · <code>/api/time</code></p>
  </main>
  <script>
    setInterval(function () {
      var el = document.getElementById("up");
      if (el) el.textContent = String(Number(el.textContent) + 1);
    }, 1000);
  </script>
</body>
</html>`;

const server = http.createServer(function (req, res) {
  const url = new URL(req.url, "http://localhost");
  const path = url.pathname;

  if (path === "/health") {
    sendJson(res, 200, {
      status: "ok",
      app: APP_NAME,
      version: APP_VERSION,
      uptime_seconds: uptimeSeconds(),
    });
    return;
  }

  if (path === "/api/info") {
    sendJson(res, 200, {
      app: APP_NAME,
      version: APP_VERSION,
      uptime_seconds: uptimeSeconds(),
      node: process.version,
      pid: process.pid,
    });
    return;
  }

  if (path === "/api/time") {
    sendJson(res, 200, {
      app: APP_NAME,
      server_time: new Date().toISOString(),
      epoch_ms: Date.now(),
    });
    return;
  }

  if (path === "/") {
    sendHtml(res, 200, landing);
    return;
  }

  sendJson(res, 404, { status: "error", message: "not found", path });
});

server.listen(PORT, function () {
  console.log(`${APP_NAME} v${APP_VERSION} listening on :${PORT}`);
});

function shutdown(signal) {
  console.log(`${signal} received — shutting down`);
  server.close(function () {
    process.exit(0);
  });
}

process.on("SIGTERM", function () { shutdown("SIGTERM"); });
process.on("SIGINT", function () { shutdown("SIGINT"); });
