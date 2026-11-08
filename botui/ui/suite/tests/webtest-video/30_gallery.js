/**
 * webtest_video evidence gallery generator
 * Split from webtest_video.js for #1731.
 */
"use strict";

const fs = require("fs");
const path = require("path");
const { VIDEOS_DIR } = require("./00_common");

function gallery(results) {
  const passN = results.filter(r => r.pass).length;
  const failN = results.length - passN;
  const cards = results.map(r => {
    const col = r.pass ? "#22c55e" : "#ef4444";
    const lbl = r.pass ? "PASS" : "FAIL";
    const vid = r.id + ".webm";
    const vidOk = fs.existsSync(path.join(VIDEOS_DIR, vid));
    return `<div class="c" style="border-color:${col}40">
      <div class="ch"><span class="b" style="background:${col}">${lbl}</span><h3>${r.id} — ${r.title}</h3></div>
      <p class="d">${r.desc}</p>
      <div class="m">${vidOk
        ? `<video controls preload="metadata" style="width:100%;border-radius:8px"><source src="${vid}" type="video/webm"></video>`
        : `<div style="padding:20px;color:#64748b;text-align:center">No video</div>`}</div>
      <div class="dt">${(r.detail||"").replace(/</g,"&lt;")}</div>
      <div class="tm">${(r.dur/1000).toFixed(1)}s</div></div>`;
  }).join("");
  return `<!DOCTYPE html><html><head><meta charset="UTF-8">
<title>GB7+BBSIA Video Evidence</title><style>
*{margin:0;padding:0;box-sizing:border-box}body{background:#0f172a;color:#e2e8f0;font-family:'Segoe UI',system-ui;padding:24px}
h1{text-align:center;font-size:28px;margin-bottom:8px;color:#f8fafc}.s{text-align:center;font-size:16px;margin-bottom:32px;color:#94a3b8}
.s .p{color:#22c55e;font-weight:700}.s .f{color:#ef4444;font-weight:700}
.g{display:grid;grid-template-columns:repeat(auto-fill,minmax(520px,1fr));gap:20px;max-width:1600px;margin:0 auto}
.c{background:#1e293b;border:1px solid #334155;border-radius:12px;padding:18px;border-left:4px solid}
.ch{display:flex;align-items:center;gap:10px;margin-bottom:6px}.b{padding:2px 10px;border-radius:6px;font-size:11px;font-weight:700;color:#fff}
.c h3{font-size:15px;color:#f8fafc}.d{font-size:12px;color:#94a3b8;margin-bottom:10px}
.m{margin-bottom:10px;background:#0f172a;border-radius:8px;overflow:hidden;min-height:40px}
.dt{font-size:11px;color:#64748b;font-family:monospace;word-break:break-all}.tm{font-size:10px;color:#475569;margin-top:4px}
video{display:block}.ts{text-align:center;color:#475569;font-size:11px;margin-top:20px}
</style></head><body>
<h1>GB7 + BBSIA — Video Evidence Gallery</h1>
<div class="s"><span class="p">${passN} passed</span> &middot; <span class="f">${failN} failed</span> &middot; ${results.length} total &middot; ${new Date().toLocaleString()}</div>
<div class="g">${cards}</div>
<div class="ts">Generated ${new Date().toISOString()}</div></body></html>`;
}

module.exports = gallery;
