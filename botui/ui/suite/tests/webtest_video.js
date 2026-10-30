/**
 * GB6 + BBSIA Web Automation — Playwright Video Evidence Suite
 *
 * Each test shows visual overlays: title → steps → result, with pauses
 * so the recorded video has meaningful content.
 *
 * Usage:  node botui/ui/suite/tests/webtest_video.js
 */
"use strict";

const http = require("http");
const fs = require("fs");
const path = require("path");
const { chromium } = require("playwright");

const SUITE_DIR = path.resolve(__dirname, "..");
const VIDEOS_DIR = path.join(__dirname, "evidence");
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
          name: "General Bots", version: "6.7.0-test",
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
      o.innerHTML='<div class="tag">GB6+BBSIA EVIDENCE</div><div class="tcid">${tcId}</div>'
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

// ═══════════════════════════════════════════════════════════════════
// 15 Test Cases — each with visual steps
// ═══════════════════════════════════════════════════════════════════

const TESTS = [
  {
    id: "TC01", title: "Desktop Globals Load",
    desc: "Verify WindowManager, GBPreviewMode, GBUpdate, GBPrivacy, GBAccessibility are available after bootstrap",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Check WindowManager", "Check GBPreviewMode", "Check GBUpdate", "Check GBPrivacy & GBAccessibility", "Verify APPS_REGISTRY"];
      await page.evaluate(overlayScript("TC01", "Desktop Globals Load", "All core modules must be present after harness loads", steps));
      await page.waitForTimeout(W);

      const checks = [
        { expr: "typeof window.WindowManager !== 'undefined'", label: "WindowManager" },
        { expr: "typeof window.GBPreviewMode !== 'undefined'", label: "GBPreviewMode" },
        { expr: "typeof window.GBUpdate !== 'undefined'", label: "GBUpdate" },
        { expr: "typeof window.GBPrivacy !== 'undefined' && typeof window.GBAccessibility !== 'undefined'", label: "GBPrivacy+A11y" },
        { expr: "Array.isArray(window.APPS_REGISTRY) && window.APPS_REGISTRY.length > 0", label: "APPS_REGISTRY" },
      ];
      let allOk = true;
      for (let i = 0; i < checks.length; i++) {
        await page.evaluate(activateStep(i));
        await page.waitForTimeout(W);
        const ok = await page.evaluate(checks[i].expr);
        allOk = allOk && ok;
        await page.evaluate(completeStep(i, ok));
        await page.waitForTimeout(300);
      }
      const detail = await page.evaluate(() => {
        return "WM:" + (typeof WindowManager !== "undefined") + " PM:" + (typeof GBPreviewMode !== "undefined")
          + " Up:" + (typeof GBUpdate !== "undefined") + " Pr:" + (typeof GBPrivacy !== "undefined")
          + " A11y:" + (typeof GBAccessibility !== "undefined") + " Reg:" + (window.APPS_REGISTRY||[]).length;
      });
      await page.evaluate(showResult(allOk, detail));
      await page.waitForTimeout(1200);
      return { pass: allOk, detail };
    },
  },
  {
    id: "TC02", title: "Window Manager Open & Close",
    desc: "Open a window programmatically, verify it appears, then close it",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Open test window", "Verify window exists", "Close window", "Verify window removed"];
      await page.evaluate(overlayScript("TC02", "Window Manager", "Open/Close lifecycle test", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      await page.waitForTimeout(300);
      await page.evaluate(() => {
        WindowManager.open("tc02", "Evidence Window",
          "<div style='padding:40px;color:#f8fafc;text-align:center'><h2 style='font-size:28px'>Window Opened!</h2><p style='color:#94a3b8;margin-top:12px'>Timestamp: " + new Date().toLocaleTimeString() + "</p></div>");
      });
      await page.evaluate(completeStep(0, true));
      await page.waitForTimeout(600);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const existed = await page.evaluate(() => WindowManager.openWindows.some(w => w.id === "tc02"));
      await page.evaluate(completeStep(1, existed));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      await page.evaluate(() => WindowManager.close("tc02"));
      await page.evaluate(completeStep(2, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(3));
      await page.waitForTimeout(300);
      const gone = await page.evaluate(() => !WindowManager.openWindows.some(w => w.id === "tc02"));
      await page.evaluate(completeStep(3, gone));
      await page.waitForTimeout(300);

      const pass = existed && gone;
      await page.evaluate(showResult(pass, `Opened:${existed} Closed:${gone}`));
      await page.waitForTimeout(1200);
      return { pass, detail: `Opened:${existed} Closed:${gone}` };
    },
  },
  {
    id: "TC03", title: "Preview Mode Event",
    desc: "GBPreviewMode.set() fires gb-preview-changed CustomEvent",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Register event listener", "Set preview ON", "Set preview OFF", "Verify 2 events fired"];
      await page.evaluate(overlayScript("TC03", "Preview Mode Event", "CustomEvent gb-preview-changed must fire on toggle", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      await page.evaluate(() => { window._prevEvts = []; window.addEventListener("gb-preview-changed", e => window._prevEvts.push(e.detail)); });
      await page.evaluate(completeStep(0, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      await page.evaluate(() => GBPreviewMode.set(true));
      const isOn = await page.evaluate(() => GBPreviewMode.isOn());
      await page.evaluate(completeStep(1, isOn));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      await page.evaluate(() => GBPreviewMode.set(false));
      const isOff = await page.evaluate(() => !GBPreviewMode.isOn());
      await page.evaluate(completeStep(2, isOff));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(3));
      await page.waitForTimeout(300);
      const evtCount = await page.evaluate(() => window._prevEvts.length);
      const pass = evtCount === 2 && isOn && isOff;
      await page.evaluate(completeStep(3, pass));
      const detail = `Events:${evtCount} on:${isOn} off:${isOff}`;
      await page.evaluate(showResult(pass, detail));
      await page.waitForTimeout(1200);
      return { pass, detail };
    },
  },
  {
    id: "TC04", title: "Deep-Link Blocking",
    desc: "Preview apps blocked from opening when GBPreviewMode is OFF",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Turn preview OFF", "Try opening 'designer' deep-link", "Verify window did NOT open"];
      await page.evaluate(overlayScript("TC04", "Deep-Link Block", "Preview app deep-links must be blocked when preview is OFF", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      await page.evaluate(() => GBPreviewMode.set(false));
      await page.evaluate(completeStep(0, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const before = await page.evaluate(() => WindowManager.openWindows.length);
      await page.evaluate(() => { try { WindowManager.openDeepLink("designer", {}); } catch(e){} });
      await page.evaluate(completeStep(1, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      const after = await page.evaluate(() => WindowManager.openWindows.length);
      const pass = before === after;
      await page.evaluate(completeStep(2, pass));
      const detail = `Windows ${before} -> ${after} (blocked=${pass})`;
      await page.evaluate(showResult(pass, detail));
      await page.waitForTimeout(1200);
      return { pass, detail };
    },
  },
  {
    id: "TC05", title: "XSS Sanitization",
    desc: "sanitizeHtml strips script tags, onerror, javascript: URLs, onmouseover",
    run: async (page) => {
      await loadHarness(page);
      const vectors = [
        { input: '<script>alert(1)</scr' + 'ipt><b>ok</b>', bad: "script", name: "Script tag" },
        { input: '<img onerror="x" src=x>', bad: "onerror", name: "onerror handler" },
        { input: '<a href="javascript:void">x</a>', bad: "javascript:", name: "javascript: URL" },
        { input: '<div onmouseover="x">y</div>', bad: "onmouseover", name: "onmouseover" },
      ];
      const steps = vectors.map(v => `Block: ${v.name}`);
      await page.evaluate(overlayScript("TC05", "XSS Sanitization", "sanitizeHtml must neutralize all attack vectors", steps));
      await page.waitForTimeout(W);

      const sanitizeFn = await page.evaluate(async () => {
        var src = await (await fetch("/suite/drive/modules/08_modals.js?_=" + Date.now())).text();
        var fn = new Function(src + "; return typeof sanitizeHtml === 'function' ? sanitizeHtml : null;");
        window._testSanitize = fn();
        return !!window._testSanitize;
      });

      let allOk = true;
      for (let i = 0; i < vectors.length; i++) {
        await page.evaluate(activateStep(i));
        await page.waitForTimeout(300);
        const ok = await page.evaluate((v) => {
          var out = window._testSanitize(v.input);
          return out.indexOf(v.bad) === -1;
        }, vectors[i]);
        allOk = allOk && ok;
        await page.evaluate(completeStep(i, ok));
        await page.waitForTimeout(W);
      }
      const detail = allOk ? "All 4 XSS vectors neutralized" : "Some vectors survived";
      await page.evaluate(showResult(allOk, detail));
      await page.waitForTimeout(1200);
      return { pass: allOk, detail };
    },
  },
  {
    id: "TC06", title: "iframe Sandbox",
    desc: "Computer use iframes must NOT have allow-same-origin",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Fetch 60_computer_use.js source", "Check allow-same-origin absent", "Check allow-scripts allow-forms present"];
      await page.evaluate(overlayScript("TC06", "iframe Sandbox", "allow-same-origin must be removed from sandbox attribute", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      const src = await page.evaluate(async () => await (await fetch("/suite/chat/modules/60_computer_use.js?_=" + Date.now())).text());
      await page.evaluate(completeStep(0, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const noSameOrigin = !src.includes("allow-same-origin");
      await page.evaluate(completeStep(1, noSameOrigin));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      const hasScriptsForms = src.includes("allow-scripts allow-forms");
      await page.evaluate(completeStep(2, hasScriptsForms));
      await page.waitForTimeout(300);

      const pass = noSameOrigin && hasScriptsForms;
      await page.evaluate(showResult(pass, `same-origin absent:${noSameOrigin} scripts+forms:${hasScriptsForms}`));
      await page.waitForTimeout(1200);
      return { pass, detail: `same-origin absent:${noSameOrigin} scripts+forms:${hasScriptsForms}` };
    },
  },
  {
    id: "TC07", title: "Citation URL Validation",
    desc: "Citations enforce https?:// scheme, noopener, noreferrer",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Fetch 30_citations.js", "Check https?:// scheme filter", "Check noreferrer + noopener"];
      await page.evaluate(overlayScript("TC07", "Citation URLs", "Only https?:// URLs allowed, with noreferrer noopener", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      const src = await page.evaluate(async () => await (await fetch("/suite/chat/modules/30_citations.js?_=" + Date.now())).text());
      await page.evaluate(completeStep(0, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const scheme = /https\?:/.test(src);
      await page.evaluate(completeStep(1, scheme));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      const refs = src.includes("noreferrer") && src.includes("noopener");
      await page.evaluate(completeStep(2, refs));

      const pass = scheme && refs;
      await page.evaluate(showResult(pass, `scheme:${scheme} noreferrer+noopener:${refs}`));
      await page.waitForTimeout(1200);
      return { pass, detail: `scheme:${scheme} noreferrer+noopener:${refs}` };
    },
  },
  {
    id: "TC08", title: "Backup Key Prefix Guard",
    desc: "data-export.js only restores localStorage keys prefixed with gb_",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Fetch data-export.js", "Check ALLOWED_PREFIX / gb_ guard"];
      await page.evaluate(overlayScript("TC08", "Backup Import Guard", "Only gb_ prefixed keys restored from backup", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      const src = await page.evaluate(async () => await (await fetch("/suite/js/data-export.js?_=" + Date.now())).text());
      await page.evaluate(completeStep(0, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const pass = src.includes("ALLOWED_PREFIX") || src.includes('"gb_"');
      await page.evaluate(completeStep(1, pass));
      await page.evaluate(showResult(pass, "prefix guard found: " + pass));
      await page.waitForTimeout(1200);
      return { pass, detail: "prefix guard: " + pass };
    },
  },
  {
    id: "TC09", title: "Drive Context Framing",
    desc: "drive_context includes untrusted:true and source:user_drive",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Fetch chat-init.js", "Check untrusted: true", "Check source: user_drive"];
      await page.evaluate(overlayScript("TC09", "Drive Context", "Untrusted data must be flagged before sending to LLM", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      const src = await page.evaluate(async () => await (await fetch("/suite/chat/chat-init.js?_=" + Date.now())).text());
      await page.evaluate(completeStep(0, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const untrusted = src.includes("untrusted: true");
      await page.evaluate(completeStep(1, untrusted));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      const source = src.includes('source: "user_drive"');
      await page.evaluate(completeStep(2, source));

      const pass = untrusted && source;
      await page.evaluate(showResult(pass, `untrusted:${untrusted} source:${source}`));
      await page.waitForTimeout(1200);
      return { pass, detail: `untrusted:${untrusted} source:${source}` };
    },
  },
  {
    id: "TC10", title: "No Admin Self-Assignment",
    desc: "__BOT_IS_PUBLIC__ must assign roles:[\"user\"], never admin",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Fetch drive/02_api.js", "Verify roles:[\"admin\"] absent", "Verify roles:[\"user\"] present"];
      await page.evaluate(overlayScript("TC10", "No Admin Self-Assign", "Public bot users get 'user' role, never 'admin'", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      const src = await page.evaluate(async () => await (await fetch("/suite/drive/modules/02_api.js?_=" + Date.now())).text());
      await page.evaluate(completeStep(0, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const noAdmin = !src.includes('roles: ["admin"]');
      await page.evaluate(completeStep(1, noAdmin));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      const hasUser = src.includes('roles: ["user"]');
      await page.evaluate(completeStep(2, hasUser));

      const pass = noAdmin && hasUser;
      await page.evaluate(showResult(pass, `admin removed:${noAdmin} user set:${hasUser}`));
      await page.waitForTimeout(1200);
      return { pass, detail: `admin removed:${noAdmin} user set:${hasUser}` };
    },
  },
  {
    id: "TC11", title: "Privacy Defaults",
    desc: "GBPrivacy starts with all telemetry/tracking disabled",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Load GBPrivacy settings", "Check telemetry OFF", "Check usage stats OFF", "Check crash reports OFF"];
      await page.evaluate(overlayScript("TC11", "Privacy Defaults", "All telemetry must be OFF by default — user opts in", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      await page.waitForTimeout(300);
      const s = await page.evaluate(() => window.GBPrivacy ? GBPrivacy.getSettings() : null);
      await page.evaluate(completeStep(0, !!s));
      await page.waitForTimeout(W);

      const checks = [
        { ok: s && s.telemetry_enabled === false, lbl: "telemetry" },
        { ok: s && s.share_usage_stats === false, lbl: "usage_stats" },
        { ok: s && s.crash_reports === false, lbl: "crash_reports" },
      ];
      let allOk = !!s;
      for (let i = 0; i < checks.length; i++) {
        await page.evaluate(activateStep(i + 1));
        await page.waitForTimeout(300);
        allOk = allOk && checks[i].ok;
        await page.evaluate(completeStep(i + 1, checks[i].ok));
        await page.waitForTimeout(W);
      }
      const detail = s ? `telemetry:${s.telemetry_enabled} stats:${s.share_usage_stats} crash:${s.crash_reports}` : "GBPrivacy missing";
      await page.evaluate(showResult(allOk, detail));
      await page.waitForTimeout(1200);
      return { pass: allOk, detail };
    },
  },
  {
    id: "TC12", title: "Accessibility",
    desc: "GBAccessibility injects skip-nav link and ARIA landmarks",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Check GBAccessibility loaded", "Find #gb-skip-nav element", "Count aria-label attributes"];
      await page.evaluate(overlayScript("TC12", "Accessibility", "Skip navigation and ARIA landmarks must be injected", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      await page.waitForTimeout(300);
      const loaded = await page.evaluate(() => typeof window.GBAccessibility !== "undefined");
      await page.evaluate(completeStep(0, loaded));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const skip = await page.evaluate(() => !!document.getElementById("gb-skip-nav"));
      await page.evaluate(completeStep(1, skip));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      const ariaCount = await page.evaluate(() => document.querySelectorAll("[aria-label]").length);
      const ariaOk = ariaCount > 0;
      await page.evaluate(completeStep(2, ariaOk));

      const pass = loaded && skip && ariaOk;
      await page.evaluate(showResult(pass, `skip-nav:${skip} aria-labels:${ariaCount}`));
      await page.waitForTimeout(1200);
      return { pass, detail: `skip-nav:${skip} aria-labels:${ariaCount}` };
    },
  },
  {
    id: "TC13", title: "Settings Preview Toggle",
    desc: "Settings HTML has preview toggle wired to GBPreviewMode",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Fetch settings/index.html", "Find #settings-preview-toggle", "Check JS wiring in settings.p1.js"];
      await page.evaluate(overlayScript("TC13", "Settings Toggle", "Preview mode toggle must exist and be wired in settings", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      const html = await page.evaluate(async () => await (await fetch("/suite/settings/index.html?_=" + Date.now())).text());
      await page.evaluate(completeStep(0, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const toggle = html.includes("settings-preview-toggle") && html.includes("togglePreviewMode");
      await page.evaluate(completeStep(1, toggle));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      const js = await page.evaluate(async () => await (await fetch("/suite/settings/settings.p1.js?_=" + Date.now())).text());
      const wired = js.includes("togglePreviewMode") && js.includes("gb-preview-changed");
      await page.evaluate(completeStep(2, wired));

      const pass = toggle && wired;
      await page.evaluate(showResult(pass, `toggle:${toggle} wired:${wired}`));
      await page.waitForTimeout(1200);
      return { pass, detail: `toggle:${toggle} wired:${wired}` };
    },
  },
  {
    id: "TC14", title: "Auto-Update Module",
    desc: "GBUpdate has check(), version, dismiss(), showReleaseNotes()",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Check GBUpdate exists", "Verify check() function", "Verify version string", "Verify dismiss + releaseNotes"];
      await page.evaluate(overlayScript("TC14", "Auto-Update", "GBUpdate module must expose update lifecycle methods", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      await page.waitForTimeout(300);
      const exists = await page.evaluate(() => typeof window.GBUpdate !== "undefined");
      await page.evaluate(completeStep(0, exists));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const hasCheck = await page.evaluate(() => typeof GBUpdate.check === "function");
      await page.evaluate(completeStep(1, hasCheck));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      const hasVersion = await page.evaluate(() => typeof GBUpdate.version === "string");
      await page.evaluate(completeStep(2, hasVersion));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(3));
      await page.waitForTimeout(300);
      const hasOthers = await page.evaluate(() => typeof GBUpdate.dismiss === "function" && typeof GBUpdate.showReleaseNotes === "function");
      await page.evaluate(completeStep(3, hasOthers));

      const pass = exists && hasCheck && hasVersion && hasOthers;
      const ver = exists ? await page.evaluate(() => GBUpdate.version) : "n/a";
      await page.evaluate(showResult(pass, `version:${ver} check:${hasCheck} dismiss:${hasOthers}`));
      await page.waitForTimeout(1200);
      return { pass, detail: `version:${ver} check:${hasCheck} dismiss:${hasOthers}` };
    },
  },
  {
    id: "TC15", title: "Apps Registry",
    desc: "APPS_REGISTRY includes chat and drive apps",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Load APPS_REGISTRY", "Find 'chat' app", "Find 'drive' app"];
      await page.evaluate(overlayScript("TC15", "Apps Registry", "Registry must include core apps: chat and drive", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      await page.waitForTimeout(300);
      const count = await page.evaluate(() => (window.APPS_REGISTRY || []).length);
      await page.evaluate(completeStep(0, count > 0));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const hasChat = await page.evaluate(() => (window.APPS_REGISTRY || []).some(a => a.id === "chat"));
      await page.evaluate(completeStep(1, hasChat));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      const hasDrive = await page.evaluate(() => (window.APPS_REGISTRY || []).some(a => a.id === "drive"));
      await page.evaluate(completeStep(2, hasDrive));

      const pass = count > 0 && hasChat && hasDrive;
      await page.evaluate(showResult(pass, `total:${count} chat:${hasChat} drive:${hasDrive}`));
      await page.waitForTimeout(1200);
      return { pass, detail: `total:${count} chat:${hasChat} drive:${hasDrive}` };
    },
  },
];

// ═══════════════════════════════════════════════════════════════════
// Gallery Generator
// ═══════════════════════════════════════════════════════════════════

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
<title>GB6+BBSIA Video Evidence</title><style>
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
<h1>GB6 + BBSIA — Video Evidence Gallery</h1>
<div class="s"><span class="p">${passN} passed</span> &middot; <span class="f">${failN} failed</span> &middot; ${results.length} total &middot; ${new Date().toLocaleString()}</div>
<div class="g">${cards}</div>
<div class="ts">Generated ${new Date().toISOString()}</div></body></html>`;
}

// ═══════════════════════════════════════════════════════════════════
// Runner
// ═══════════════════════════════════════════════════════════════════

async function main() {
  fs.mkdirSync(VIDEOS_DIR, { recursive: true });
  try { for (const f of fs.readdirSync(VIDEOS_DIR)) { try { fs.unlinkSync(path.join(VIDEOS_DIR, f)); } catch(e){} } } catch(e) {}

  const server = await startServer();
  const browser = await chromium.launch({ headless: true });
  const results = [];

  console.log("\n" + "=".repeat(60));
  console.log("GB6+BBSIA VIDEO EVIDENCE — " + TESTS.length + " tests");
  console.log("=".repeat(60) + "\n");

  for (const tc of TESTS) {
    const ctx = await browser.newContext({
      viewport: { width: 1280, height: 720 },
      recordVideo: { dir: VIDEOS_DIR, size: { width: 1280, height: 720 } },
    });
    const page = await ctx.newPage();
    page.on("pageerror", () => {});
    const t0 = Date.now();
    let res;
    try { res = await tc.run(page); } catch (e) { res = { pass: false, detail: "ERROR: " + e.message }; }
    const dur = Date.now() - t0;

    await page.close();
    await ctx.close();

    await new Promise(r => setTimeout(r, 500));
    const unnamed = fs.readdirSync(VIDEOS_DIR).filter(f => f.endsWith(".webm") && !results.some(r => f === r.id + ".webm") && f !== tc.id + ".webm");
    if (unnamed.length) {
      unnamed.sort((a,b) => fs.statSync(path.join(VIDEOS_DIR,b)).mtimeMs - fs.statSync(path.join(VIDEOS_DIR,a)).mtimeMs);
      try { fs.renameSync(path.join(VIDEOS_DIR, unnamed[0]), path.join(VIDEOS_DIR, tc.id + ".webm")); } catch(e){}
    }

    console.log(`[${res.pass?"PASS":"FAIL"}] ${tc.id}: ${tc.title} (${(dur/1000).toFixed(1)}s)`);
    if (!res.pass) console.log(`       ${res.detail}`);
    results.push({ id: tc.id, title: tc.title, desc: tc.desc, pass: res.pass, detail: res.detail, dur });
  }

  await browser.close();
  server.close();

  fs.writeFileSync(GALLERY_PATH, gallery(results));
  const p = results.filter(r=>r.pass).length, f = results.length - p;
  console.log("\n" + "=".repeat(60));
  console.log(`TOTAL: ${results.length} | PASS: ${p} | FAIL: ${f}`);
  console.log("=".repeat(60));
  console.log("\nGallery: " + GALLERY_PATH);

  require("child_process").exec(`start "" "${GALLERY_PATH}"`);
  process.exit(f > 0 ? 1 : 0);
}

main().catch(e => { console.error("Fatal:", e.message); process.exit(1); });
