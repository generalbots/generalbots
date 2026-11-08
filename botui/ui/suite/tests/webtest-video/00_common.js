/**
 * GB7 + BBSIA Web Automation — Playwright Video Evidence Suite
 *
 * Each test shows visual overlays: title → steps → result, with pauses
 * so the recorded video has meaningful content.
 *
 * Usage:  node botui/ui/suite/tests/webtest_video.js
 *
 * Split from webtest_video.js for #1731: module 0/4 keeps the shared setup.
 */
"use strict";

const http = require("http");
const fs = require("fs");
const path = require("path");
const { chromium } = require("playwright");

const SUITE_DIR = path.resolve(__dirname, "..", "..");
const VIDEOS_DIR = path.join(__dirname, "..", "evidence");
const GALLERY_PATH = path.join(VIDEOS_DIR, "gallery.html");
const PORT = 9777;

const MIME = {
  ".html": "text/html", ".css": "text/css", ".js": "application/javascript",
  ".json": "application/json", ".svg": "image/svg+xml", ".png": "image/png",
  ".jpg": "image/jpeg", ".webp": "image/webp", ".webm": "video/webm",
  ".woff2": "font/woff2", ".woff": "font/woff", ".ttf": "font/ttf",
  ".ico": "image/x-icon", ".webmanifest": "application/manifest+json",
};

function startServer() {
  return new Promise((resolve) => {
    const srv = http.createServer((req, res) => {
      let u = decodeURIComponent(req.url.split("?")[0]);
      if (u === "/api/product") {
        res.writeHead(200, { "Content-Type": "application/json" });
        return res.end(JSON.stringify({
          name: "General Bots", version: "7.0.0-test",
          apps: ["chat", "drive"], preview_apps: ["designer", "fraud"],
          no_security: true, suite_default_app: "chat",
        }));
      }
      if (u === "/api/apps/catalog") { res.writeHead(200, { "Content-Type": "application/json" }); return res.end(JSON.stringify({ apps: [] })); }
      if (u.startsWith("/api/") || u === "/ws") { res.writeHead(200, { "Content-Type": "application/json" }); return res.end("{}"); }
      if (u.startsWith("/suite/")) u = u.slice(6);
      if (u === "/") u = "/tests/test-harness.html";
      const fp = path.join(SUITE_DIR, u);
      if (!fp.startsWith(SUITE_DIR)) { res.writeHead(403); return res.end(); }
      fs.readFile(fp, (err, data) => {
        if (err) { res.writeHead(404); return res.end("Not found: " + u); }
        res.writeHead(200, { "Content-Type": MIME[path.extname(fp).toLowerCase()] || "application/octet-stream" });
        res.end(data);
      });
    });
    srv.listen(PORT, () => { console.log("[server] http://localhost:" + PORT); resolve(srv); });
  });
}

const HARNESS = `http://localhost:${PORT}/suite/tests/test-harness.html`;

// ═══════════════════════════════════════════════════════════════════
// Visual overlay helpers (injected into each page)
// ═══════════════════════════════════════════════════════════════════

const OVERLAY_CSS = `
  #_ov{position:fixed;inset:0;z-index:999999;display:flex;flex-direction:column;align-items:center;justify-content:center;
    background:rgba(15,23,42,0.92);font-family:'Segoe UI',system-ui;color:#e2e8f0;transition:opacity .3s}
  #_ov h1{font-size:42px;font-weight:800;margin-bottom:8px;letter-spacing:-1px}
  #_ov .desc{font-size:18px;color:#94a3b8;margin-bottom:32px;max-width:700px;text-align:center;line-height:1.5}
  #_ov .steps{display:flex;flex-direction:column;gap:12px;width:640px}
  #_ov .step{display:flex;align-items:center;gap:12px;padding:12px 18px;background:rgba(30,41,59,0.8);border-radius:10px;
    border-left:4px solid #334155;font-size:15px;transition:all .4s}
  #_ov .step.active{border-left-color:#3b82f6;background:rgba(59,130,246,0.12)}
  #_ov .step.done{border-left-color:#22c55e;background:rgba(34,197,94,0.08)}
  #_ov .step.fail{border-left-color:#ef4444;background:rgba(239,68,68,0.08)}
  #_ov .step .icon{width:24px;height:24px;border-radius:50%;display:flex;align-items:center;justify-content:center;font-size:13px;
    background:#334155;color:#94a3b8;font-weight:700;flex-shrink:0;transition:all .3s}
  #_ov .step.done .icon{background:#22c55e;color:#fff}
  #_ov .step.fail .icon{background:#ef4444;color:#fff}
  #_ov .step.active .icon{background:#3b82f6;color:#fff}
  #_ov .result{margin-top:40px;font-size:64px;font-weight:900;letter-spacing:2px;opacity:0;transform:scale(0.5);transition:all .5s}
  #_ov .result.show{opacity:1;transform:scale(1)}
  #_ov .result.pass{color:#22c55e}
  #_ov .result.fail{color:#ef4444}
  #_ov .detail{margin-top:12px;font-size:13px;color:#64748b;font-family:monospace;max-width:600px;text-align:center;word-break:break-all}
  #_ov .tag{position:absolute;top:20px;left:24px;font-size:12px;color:#475569;font-weight:600;letter-spacing:1px}
  #_ov .tcid{position:absolute;top:20px;right:24px;font-size:14px;color:#3b82f6;font-weight:700}
`;

function overlayScript(tcId, title, desc, stepLabels) {
  return `
    (function(){
      if(document.getElementById('_ovStyle')) return;
      var s=document.createElement('style');s.id='_ovStyle';s.textContent=${JSON.stringify(OVERLAY_CSS)};document.head.appendChild(s);
      var o=document.createElement('div');o.id='_ov';
      o.innerHTML='<div class="tag">GB7+BBSIA EVIDENCE</div><div class="tcid">${tcId}</div>'
        +'<h1>${title.replace(/'/g,"\\'")}</h1><div class="desc">${desc.replace(/'/g,"\\'")}</div>'
        +'<div class="steps">${stepLabels.map((l,i) => `<div class="step" id="_s${i}"><div class="icon">${i+1}</div><span>${l.replace(/'/g,"\\'")}</span></div>`).join("")}</div>'
        +'<div class="result" id="_res"></div><div class="detail" id="_det"></div>';
      document.body.appendChild(o);
    })();
  `;
}

function activateStep(i) {
  return `document.getElementById('_s${i}').classList.add('active');document.getElementById('_s${i}').querySelector('.icon').textContent='...';`;
}
function completeStep(i, ok) {
  const cls = ok ? "done" : "fail";
  const icon = ok ? "✓" : "✗";
  return `var e=document.getElementById('_s${i}');e.classList.remove('active');e.classList.add('${cls}');e.querySelector('.icon').textContent='${icon}';`;
}
function showResult(pass, detail) {
  const cls = pass ? "pass" : "fail";
  const txt = pass ? "PASS" : "FAIL";
  const safeDetail = (detail || "").replace(/'/g, "\\'").replace(/\n/g, " ");
  return `var r=document.getElementById('_res');r.className='result show ${cls}';r.textContent='${txt}';document.getElementById('_det').textContent='${safeDetail}';`;
}

const W = 400; // ms pause per visual step

async function loadHarness(page) {
  await page.goto(HARNESS, { waitUntil: "load", timeout: 15000 });
  await page.waitForSelector("#harness-ready", { state: "visible", timeout: 10000 });
}

module.exports = {
  VIDEOS_DIR,
  GALLERY_PATH,
  startServer,
  overlayScript,
  activateStep,
  completeStep,
  showResult,
  W,
  loadHarness,
};
