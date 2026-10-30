/**
 * GB6 Web Automation Test Suite
 * Tests all GB6 features and BBSIA review fixes.
 * Run with: node botui/ui/suite/tests/webtest_gb6.js
 */
"use strict";

var fs = require("fs");
var path = require("path");

var SUITE_DIR = path.resolve(__dirname, "..");
var PASS = 0;
var FAIL = 0;
var RESULTS = [];

function test(name, fn) {
  try {
    fn();
    PASS++;
    RESULTS.push({ name: name, status: "PASS" });
  } catch (e) {
    FAIL++;
    RESULTS.push({ name: name, status: "FAIL", error: e.message });
  }
}

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}

function readFile(relPath) {
  return fs.readFileSync(path.join(SUITE_DIR, relPath), "utf8");
}

function fileExists(relPath) {
  return fs.existsSync(path.join(SUITE_DIR, relPath));
}

// ═══════════════════════════════════════════════════════════════════
// SECURITY FIXES
// ═══════════════════════════════════════════════════════════════════

test("SEC-01: XSS fix — sanitizeHtml function exists in 08_modals.js", function () {
  var src = readFile("drive/modules/08_modals.js");
  assert(src.indexOf("function sanitizeHtml(html)") !== -1, "sanitizeHtml not found");
  assert(src.indexOf("on\\w+") !== -1 || src.indexOf("on\\\\w") !== -1, "on* handler removal not found");
  assert(src.indexOf("javascript:") !== -1, "javascript: URL removal not found");
});

test("SEC-02: XSS fix — marked.parse output is sanitized", function () {
  var src = readFile("drive/modules/08_modals.js");
  assert(src.indexOf("sanitizeHtml(marked.parse(") !== -1, "marked.parse not wrapped in sanitizeHtml");
  assert(src.indexOf("previewEl.innerHTML = marked.parse(t.substring") === -1, "unsanitized innerHTML still present");
});

test("SEC-03: iframe sandbox — allow-same-origin removed", function () {
  var src = readFile("chat/modules/60_computer_use.js");
  assert(src.indexOf("allow-same-origin") === -1, "allow-same-origin still present");
  assert(src.indexOf('allow-scripts allow-forms') !== -1, "expected sandbox value not found");
});

test("SEC-04: Citation URL scheme validation", function () {
  var src = readFile("chat/modules/30_citations.js");
  assert(src.indexOf("https?:\\/\\/") !== -1 || src.indexOf("https?:") !== -1, "URL scheme check not found");
  assert(src.indexOf("noreferrer") !== -1, "noreferrer not found");
});

test("SEC-05: Backup import key sanitization", function () {
  var src = readFile("js/data-export.js");
  assert(src.indexOf('indexOf(ALLOWED_PREFIX)') !== -1 || src.indexOf('indexOf("gb_")') !== -1, "key prefix check not found");
});

test("SEC-06: gb-ws-frame single dispatch (double dispatch fixed)", function () {
  var src = readFile("chat/chat-websocket.js");
  assert(src.indexOf('gb-ws-frame') === -1, "gb-ws-frame dispatch still in chat-websocket.js");
  var tabs = readFile("chat/modules/20_tabs.js");
  assert(tabs.indexOf('gb-ws-frame') !== -1, "gb-ws-frame dispatch missing from 20_tabs.js");
});

test("SEC-07: Drive path traversal — normalize_path rejects '..'", function () {
  var src = fs.readFileSync(path.resolve(__dirname, "../../../../botserver/crates/botdrive/src/drive_handlers/part1.rs"), "utf8");
  assert(src.indexOf('".."') !== -1, ".. rejection not found in normalize_path");
  assert(src.indexOf('"."') !== -1, ". rejection not found in normalize_path");
});

// ═══════════════════════════════════════════════════════════════════
// FUNCTIONALITY FIXES
// ═══════════════════════════════════════════════════════════════════

test("FUNC-01: GBSandbox.addPending wired in computer_use", function () {
  var src = readFile("chat/modules/60_computer_use.js");
  assert(src.indexOf("GBSandbox.addPending") !== -1, "addPending not called in computer_use");
  assert(src.indexOf("GBSandbox.removePending") !== -1, "removePending not called in computer_use");
});

test("FUNC-02: GBSandbox.addPending wired in terminal_use", function () {
  var src = readFile("chat/modules/61_terminal_use.js");
  assert(src.indexOf("GBSandbox.addPending") !== -1, "addPending not called in terminal_use");
  assert(src.indexOf("GBSandbox.removePending") !== -1, "removePending not called in terminal_use");
});

test("FUNC-03: Privacy toggle controls drive_context and user_memory", function () {
  var src = readFile("chat/chat-init.js");
  assert(src.indexOf("GBPrivacy") !== -1, "GBPrivacy not referenced in chat-init");
  assert(src.indexOf("save_memory") !== -1, "save_memory check not found");
});

test("FUNC-04: Anonymous sessions skip memory and drive_context", function () {
  var src = readFile("chat/chat-init.js");
  assert(src.indexOf("isAnonymous") !== -1, "isAnonymous check not found");
  assert(src.indexOf("!isAnonymous") !== -1, "isAnonymous gating not applied");
});

test("FUNC-05: no_security wired to auth middleware require_auth", function () {
  var src = fs.readFileSync(path.resolve(__dirname, "../../../../botserver/src/main_module/routes/security_setup.rs"), "utf8");
  assert(src.indexOf("no_security") !== -1, "no_security not referenced in security_setup");
  assert(src.indexOf("require_auth = false") !== -1, "require_auth not set to false for no_security");
});

// ═══════════════════════════════════════════════════════════════════
// GB6 FEATURE FILES EXIST
// ═══════════════════════════════════════════════════════════════════

var featureFiles = [
  "chat/modules/30_citations.js",
  "chat/modules/50_memory.js",
  "chat/modules/60_computer_use.js",
  "chat/modules/61_terminal_use.js",
  "chat/modules/62_sandbox.js",
  "drive/modules/08_modals.js",
  "drive/modules/11_external.js",
  "drive/modules/12_offline_sync.js",
  "drive/modules/98_actions.js",
  "js/auto-update.js",
  "js/data-export.js",
  "js/usage-stats.js",
  "js/privacy-controls.js",
  "js/accessibility.js",
  "desktop.html",
];

featureFiles.forEach(function (f) {
  test("FILE: " + f + " exists", function () {
    assert(fileExists(f), f + " not found");
  });
});

// ═══════════════════════════════════════════════════════════════════
// GB6 FEATURE CONTENT CHECKS
// ═══════════════════════════════════════════════════════════════════

test("GB6-01: No-security mode flag reference", function () {
  var src = readFile("chat/chat-init.js");
  assert(src.indexOf("__BOT_IS_PUBLIC__") !== -1, "__BOT_IS_PUBLIC__ not found");
});

test("GB6-08: Citations renderer", function () {
  var src = readFile("chat/modules/30_citations.js");
  assert(src.indexOf("GBRenderCitations") !== -1, "GBRenderCitations not found");
  assert(src.indexOf("source-chip") !== -1, "source-chip class not found");
});

test("GB6-14: Chat memory module", function () {
  var src = readFile("chat/modules/50_memory.js");
  assert(src.indexOf("ChatMemory") !== -1, "ChatMemory not found");
  assert(src.indexOf("getMemorySummary") !== -1, "getMemorySummary not found");
});

test("GB6-19: Computer use with consent", function () {
  var src = readFile("chat/modules/60_computer_use.js");
  assert(src.indexOf("GBBrowserUse") !== -1, "GBBrowserUse not found");
  assert(src.indexOf("needs_consent") !== -1, "needs_consent not found");
});

test("GB6-20: Terminal use with blocked patterns", function () {
  var src = readFile("chat/modules/61_terminal_use.js");
  assert(src.indexOf("BLOCKED_PATTERNS") !== -1, "BLOCKED_PATTERNS not found");
  assert(src.indexOf("/api/terminal/create") !== -1, "correct endpoint not found");
});

test("GB6-21: Sandbox approval queue", function () {
  var src = readFile("chat/modules/62_sandbox.js");
  assert(src.indexOf("GBSandbox") !== -1, "GBSandbox not found");
  assert(src.indexOf("approveAll") !== -1, "approveAll not found");
});

test("GB6-22: Offline sync with IndexedDB", function () {
  var src = readFile("drive/modules/12_offline_sync.js");
  assert(src.indexOf("DriveOfflineSync") !== -1, "DriveOfflineSync not found");
  assert(src.indexOf("gb_drive_offline") !== -1, "IndexedDB name not found");
});

test("GB6-23: Auto-update checks /api/product", function () {
  var src = readFile("js/auto-update.js");
  assert(src.indexOf("GBUpdate") !== -1, "GBUpdate not found");
  assert(src.indexOf("/api/product") !== -1, "correct endpoint not found");
  assert(src.indexOf("/api/product/version") === -1, "dead endpoint still referenced");
});

test("GB6-24: Data export backup/restore", function () {
  var src = readFile("js/data-export.js");
  assert(src.indexOf("GBDataExport") !== -1, "GBDataExport not found");
  assert(src.indexOf("export_version") !== -1, "export_version not found");
});

test("GB6-25: Usage stats tracking", function () {
  var src = readFile("js/usage-stats.js");
  assert(src.indexOf("GBUsageStats") !== -1, "GBUsageStats not found");
  assert(src.indexOf("TOKEN_COST_PER_1K") !== -1, "TOKEN_COST_PER_1K not found");
});

test("GB6-26: Privacy controls — all telemetry OFF by default", function () {
  var src = readFile("js/privacy-controls.js");
  assert(src.indexOf("GBPrivacy") !== -1, "GBPrivacy not found");
  assert(src.indexOf("telemetry_enabled: false") !== -1, "telemetry not OFF by default");
  assert(src.indexOf("/api/telemetry/event") === -1, "dead telemetry endpoint still referenced");
});

test("GB6-27: Accessibility — skip-nav, ARIA, keyboard shortcuts", function () {
  var src = readFile("js/accessibility.js");
  assert(src.indexOf("GBAccessibility") !== -1, "GBAccessibility not found");
  assert(src.indexOf("gb-skip-nav") !== -1, "skip-nav not found");
  assert(src.indexOf("aria-label") !== -1, "ARIA landmarks not found");
  assert(src.indexOf("prefers-reduced-motion") !== -1, "reduced-motion not found");
});

test("GB6-18: Connector consent modal", function () {
  var src = readFile("drive/modules/11_external.js");
  assert(src.indexOf("showConnectorConsent") !== -1, "showConnectorConsent not found");
  assert(src.indexOf("CONNECTOR_PERMISSIONS") !== -1, "CONNECTOR_PERMISSIONS not found");
});

test("GB6-11: Drive preview (markdown/CSV/JSON)", function () {
  var src = readFile("drive/modules/08_modals.js");
  assert(src.indexOf("csv") !== -1, "CSV preview not found");
  assert(src.indexOf("json") !== -1, "JSON preview not found");
  assert(src.indexOf("marked") !== -1, "markdown preview not found");
});

// ═══════════════════════════════════════════════════════════════════
// BBSIA META 10 REVIEW FINDINGS
// ═══════════════════════════════════════════════════════════════════

test("BBSIA-01: drive/02_api.js — no admin self-assignment for public bot", function () {
  var src = readFile("drive/modules/02_api.js");
  assert(src.indexOf('roles: ["admin"]') === -1, "admin self-assignment still present in 02_api.js");
  assert(src.indexOf('roles: ["user"]') !== -1, "user role not set for public bot");
});

test("BBSIA-02: gb-preview-changed event dispatched by GBPreviewMode.set()", function () {
  var src = readFile("js/window-manager.js");
  assert(src.indexOf("gb-preview-changed") !== -1, "gb-preview-changed event not found");
  assert(src.indexOf('new CustomEvent("gb-preview-changed"') !== -1, "CustomEvent dispatch not found");
});

test("BBSIA-03: deep-link blocking for preview apps when preview OFF", function () {
  var src = readFile("js/window-manager.js");
  assert(src.indexOf("isPreviewId(appId)") !== -1, "preview check in openDeepLink not found");
});

test("BBSIA-04: settings preview mode toggle exists", function () {
  var settingsHtml = fs.readFileSync(path.join(SUITE_DIR, "settings/index.html"), "utf8");
  assert(settingsHtml.indexOf("settings-preview-toggle") !== -1, "preview toggle not in settings");
  assert(settingsHtml.indexOf("togglePreviewMode") !== -1, "togglePreviewMode handler not in settings HTML");
  var settingsJs = fs.readFileSync(path.join(SUITE_DIR, "settings/settings.p1.js"), "utf8");
  assert(settingsJs.indexOf("togglePreviewMode") !== -1, "togglePreviewMode not in settings JS");
  assert(settingsJs.indexOf("gb-preview-changed") !== -1, "gb-preview-changed listener not in settings JS");
});

test("BBSIA-05: drive_context framed as untrusted data", function () {
  var src = readFile("chat/chat-init.js");
  assert(src.indexOf("untrusted: true") !== -1, "untrusted flag not found in drive_context");
  assert(src.indexOf("source: \"user_drive\"") !== -1, "source label not found in drive_context");
});

test("BBSIA-06: 99_init.js under 350 lines (file split)", function () {
  var src = readFile("drive/modules/99_init.js");
  var lineCount = src.split("\n").length;
  assert(lineCount <= 350, "99_init.js has " + lineCount + " lines (max 350)");
  assert(fileExists("drive/modules/98_actions.js"), "98_actions.js not found (extracted file actions)");
});

test("BBSIA-07: 98_actions.js loaded in drive.html", function () {
  var driveHtml = fs.readFileSync(path.join(SUITE_DIR, "drive/drive.html"), "utf8");
  assert(driveHtml.indexOf("98_actions.js") !== -1, "98_actions.js not referenced in drive.html");
});

// ═══════════════════════════════════════════════════════════════════
// JS SYNTAX VALIDATION
// ═══════════════════════════════════════════════════════════════════

var allJsFiles = featureFiles.filter(function (f) { return f.endsWith(".js"); });
allJsFiles.forEach(function (f) {
  test("SYNTAX: " + f, function () {
    var src = readFile(f);
    try { new Function(src); } catch (e) {
      throw new Error("Syntax error: " + e.message);
    }
  });
});

// ═══════════════════════════════════════════════════════════════════
// RESULTS
// ═══════════════════════════════════════════════════════════════════

console.log("\n" + "=".repeat(60));
console.log("GB6 WEB AUTOMATION TEST RESULTS");
console.log("=".repeat(60));
RESULTS.forEach(function (r) {
  var icon = r.status === "PASS" ? "PASS" : "FAIL";
  console.log("[" + icon + "] " + r.name + (r.error ? " — " + r.error : ""));
});
console.log("=".repeat(60));
console.log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
console.log("=".repeat(60));

process.exit(FAIL > 0 ? 1 : 0);
