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
var PENDING = [];

function test(name, fn) {
  try {
    var output = fn();
    if (output && typeof output.then === "function") {
      PENDING.push(
        output.then(
          function () {
            PASS++;
            RESULTS.push({ name: name, status: "PASS" });
          },
          function (e) {
            FAIL++;
            RESULTS.push({ name: name, status: "FAIL", error: e.message });
          },
        ),
      );
      return;
    }
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
  var src = fs.readFileSync(path.resolve(__dirname, "../../../../botserver/crates/botdrive/src/drive_handlers/cached_quota.rs"), "utf8");
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
  "chat/chat-artifacts.js",
  "chat/chat-render.js",
  "chat/chat-messages.js",
  "chat/modules/30_citations.js",
  "chat/modules/50_memory.js",
  "chat/modules/60_computer_use.js",
  "chat/modules/61_terminal_use.js",
  "chat/modules/62_sandbox.js",
  "drive/modules/05_files.js",
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
  // #1644 — approval is recorded server-side and the page is read by the
  // server (the cross-origin iframe cannot); the fake screenshot is gone.
  assert(src.indexOf("/approve") !== -1, "server approval not requested");
  assert(src.indexOf("/execute") !== -1, "server execution not requested");
  assert(src.indexOf("case \"screenshot\"") === -1, "fake screenshot still present");
});

test("GB6-20: Terminal use with blocked patterns", function () {
  var src = readFile("chat/modules/61_terminal_use.js");
  assert(src.indexOf("BLOCKED_PATTERNS") !== -1, "BLOCKED_PATTERNS not found");
  assert(src.indexOf("/api/terminal/exec") !== -1, "exec endpoint not found");
  // #1645 — execution is bound to a server-side approval, and the durable
  // audit trail is reachable from the tray.
  assert(src.indexOf("/approve") !== -1, "approval record not requested");
  assert(src.indexOf("action_id") !== -1, "action id not sent");
  assert(src.indexOf("/api/terminal/audit") !== -1, "audit endpoint not used");
  assert(src.indexOf("gbTerminalAuditBtn") !== -1, "audit entry point missing");
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

test("GB6-25: Usage panel is reachable and reads the server", function () {
  var usage = readFile("js/usage-stats.js");
  assert(usage.indexOf("/api/user/usage/summary") !== -1, "server endpoint not used");
  assert(usage.indexOf("pricing_available") !== -1, "pricing availability not read");
  assert(usage.indexOf("data-usage-days") !== -1, "period selector missing");
  var desktop = readFile("desktop.html");
  assert(desktop.indexOf("GBUsageStats.showPanel") !== -1, "no UI entry point");
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
  // #1643 — the modal now renders the server-declared scopes, not a static list.
  assert(src.indexOf("requested_scopes") !== -1, "server scopes not used");
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
  var settingsJs = fs.readFileSync(path.join(SUITE_DIR, "settings/settings-main.js"), "utf8");
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
// ARTIFACTS (GB6-09 / #1636)
// ═══════════════════════════════════════════════════════════════════

test("GB6-09: artifact detector covers prose and markdown, not only fenced code", function () {
  var src = readFile("chat/chat-artifacts.js");
  assert(src.indexOf("window.GBArtifacts") !== -1, "GBArtifacts helper not found");
  assert(src.indexOf("words >= 250") !== -1, "prose length threshold not found");
  assert(src.indexOf("headings > 0") !== -1, "markdown structure detection not found");
  assert(src.indexOf("words >= 120") !== -1, "structured short-document path not found");
});

test("GB6-09: markdown artifacts render as documents and save under the title", function () {
  var src = readFile("chat/chat-artifacts.js");
  assert(src.indexOf("gb-artifact-doc") !== -1, "document container not found");
  assert(src.indexOf("renderMarkdown") !== -1, "markdown renderer not wired");
  assert(src.indexOf("gb-artifact-folder") !== -1, "folder selection not found");
  assert(src.indexOf("window.GBArtifacts.slug(title)") !== -1, "title-derived filename not found");
});

// ═══════════════════════════════════════════════════════════════════
// SHARE AND EXPORT (GB6-13 / #1639)
// ═══════════════════════════════════════════════════════════════════

test("GB6-13: share modal sends a visible expiry", function () {
  var src = readFile("drive/modules/08_modals.js");
  assert(src.indexOf("share-expiry") !== -1, "expiry selector not found");
  assert(src.indexOf("expires_at") !== -1, "expires_at not sent to the API");
  assert(src.indexOf("isExpired") !== -1, "expired links not detected in the modal");
});

test("GB6-13: recipient sharing and archive export are wired", function () {
  var modals = readFile("drive/modules/08_modals.js");
  assert(modals.indexOf("/share/recipients") !== -1, "recipient search not wired");
  assert(modals.indexOf('"/share"') !== -1, "share call not wired");
  var actions = readFile("drive/modules/98_actions.js");
  assert(actions.indexOf("/export") !== -1, "archive export endpoint not used");
  assert(actions.indexOf("exportPaths") !== -1, "export helper not found");
});

// ═══════════════════════════════════════════════════════════════════
// TRASH AND VERSIONS (GB6-14 / #1640)
// ═══════════════════════════════════════════════════════════════════

test("GB6-14: user deletes move to Trash, not hard delete", function () {
  var src = readFile("drive/modules/98_actions.js");
  assert(src.indexOf("/trash/move") !== -1, "trash/move not called");
  assert(src.indexOf('"/delete"') === -1, "hard delete still used by the UI");
  assert(src.indexOf("30 days") !== -1, "retention copy not found");
});

test("GB6-14: Trash shows how long is left before purge", function () {
  var src = readFile("drive/modules/05_files.js");
  assert(src.indexOf("daysLeft") !== -1, "retention computation not found");
  assert(src.indexOf("expires_at") !== -1, "expires_at not read");
  assert(src.indexOf("purge due") !== -1, "expired state not shown");
});

// ═══════════════════════════════════════════════════════════════════
// MEMORY (GB6-16 / #1642)
// ═══════════════════════════════════════════════════════════════════

test("GB6-16: Settings memory reads and writes the server API", function () {
  var settings = readFile("settings/index.html");
  assert(settings.indexOf("/api/user/memory") !== -1, "settings does not call the memory API");
  assert(settings.indexOf("serverPut") !== -1, "settings does not persist edits");
  assert(settings.indexOf("serverDelete") !== -1, "settings does not delete server-side");
  var chat = readFile("chat/chat-init.js");
  assert(chat.indexOf("user_memory:") === -1, "dead user_memory payload still sent");
});

// ═══════════════════════════════════════════════════════════════════
// CONNECTOR CONSENT (GB6-18 / #1643)
// ═══════════════════════════════════════════════════════════════════

test("GB6-18: consent renders the server's requested scopes", function () {
  var src = readFile("drive/modules/11_external.js");
  assert(src.indexOf("requested_scopes") !== -1, "requested_scopes not read");
  assert(src.indexOf("scope_descriptions") !== -1, "scope descriptions not read");
  assert(src.indexOf("CONNECTOR_PERMISSIONS") === -1, "static permission copy still present");
});

test("GB6-18: connected card shows granted scopes and visibility", function () {
  var src = readFile("drive/modules/11_external.js");
  assert(src.indexOf("Granted scopes") !== -1, "granted scopes not rendered");
  assert(src.indexOf("Visibility:") !== -1, "visibility not rendered");
  assert(src.indexOf("consented_at") !== -1, "consent timestamp not rendered");
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
// BEHAVIOURAL TESTS (execute the shipped functions, not just source text)
// ═══════════════════════════════════════════════════════════════════

var vm = require("vm");

function loadScript(relPath, sandbox) {
  var context = vm.createContext(sandbox);
  vm.runInContext(readFile(relPath), context, { filename: relPath });
  return context;
}

function memoryStorage() {
  var store = {};
  return {
    getItem: function (key) { return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null; },
    setItem: function (key, value) { store[key] = String(value); },
    removeItem: function (key) { delete store[key]; },
  };
}

test("BEHAVE GB6-09: artifact detector classifies prose, markdown, HTML and code", function () {
  var window = {};
  var context = loadScript("chat/chat-artifacts.js", {
    window: window,
    document: {},
    console: console,
  });
  var artifacts = context.window.GBArtifacts;
  assert(artifacts, "GBArtifacts was not exported");
  assert(artifacts.detect("too short") === null, "short text must not become an artifact");
  var prose = "This paragraph explains the quarterly result in detail. ".repeat(60);
  assert(artifacts.detect(prose) && artifacts.detect(prose).kind === "markdown",
    "a long prose answer must be offered as a document");
  var code = "```js\n" + "var x = 1;\n".repeat(60) + "```";
  assert(artifacts.detect(code).kind === "code", "fenced code must stay a code artifact");
  assert(artifacts.detect("<html><body>hi</body></html>").kind === "html", "HTML page not detected");
  assert(artifacts.slug("Meu Relatório de Vendas") === "meu-relatorio-de-vendas",
    "slug must transliterate the title");
});

test("BEHAVE GB6-09: artifact HTML sanitizer strips scripts and handlers", function () {
  function node() {
    return {
      attrs: [],
      removed: false,
      remove: function () { this.removed = true; },
      get attributes() { return this.attrs; },
      removeAttribute: function (name) { this.attrs = this.attrs.filter(function (a) { return a.name !== name; }); },
    };
  }
  var nodes = [];
  var document = {
    createElement: function () {
      return {
        innerHTML: "",
        content: {
          querySelectorAll: function (selector) {
            if (selector === "*") return nodes;
            return nodes;
          },
        },
      };
    },
  };
  var context = loadScript("chat/chat-artifacts.js", { window: {}, document: document, console: console });
  var evil = node();
  evil.attrs = [{ name: "onclick", value: "alert(1)" }, { name: "href", value: "javascript:alert(2)" }];
  nodes.push(evil);
  var cleaned = context.window.GBArtifacts.safeHtml("<a onclick='alert(1)'>x</a>");
  assert(evil.attrs.length === 0, "event handler and javascript: URL must be removed");
  assert(typeof cleaned === "string", "sanitizer must return markup");
});

test("BEHAVE GB6-25: usage tracking counts a message once and totals it", function () {
  var storage = memoryStorage();
  var window = {};
  window.addEventListener = function () {};
  var context = loadScript("js/usage-stats.js", {
    window: window,
    document: { addEventListener: function () {}, getElementById: function () { return null; } },
    localStorage: storage,
    sessionStorage: memoryStorage(),
    console: console,
  });
  var usage = context.window.GBUsageStats;
  assert(usage, "GBUsageStats was not exported");
  usage.track("user", 120, "auto", "m-1");
  usage.track("user", 120, "auto", "m-1");
  usage.track("bot", 240, "auto", "m-2");
  var summary = usage.getSummary(1);
  assert(summary.messages_sent === 1, "duplicate frame must be counted once, got " + summary.messages_sent);
  assert(summary.messages_received === 1, "bot reply not counted");
  assert(summary.tokens_estimated === 90, "4 chars/token estimate: " + summary.tokens_estimated);
});

test("BEHAVE GB6-13: share expiry offers 7 days and refuses a past date", function () {
  var selectValue = "7";
  var context = loadScript("drive/modules/08_modals.js", {
    window: { location: { origin: "http://localhost:8080" } },
    document: { getElementById: function (id) { return id === "share-expiry" ? { value: selectValue } : null; } },
    localStorage: memoryStorage(),
    sessionStorage: memoryStorage(),
    console: console,
  });
  var iso = context.shareExpiryIso();
  var when = Date.parse(iso);
  var days = (when - Date.now()) / 86400000;
  assert(days > 6.9 && days < 7.1, "expiry must be ~7 days, got " + days);
  selectValue = "never";
  assert(context.shareExpiryIso() === null, "never must send no expiry");
  assert(context.isExpired("2000-01-01T00:00:00Z") === true, "past link must be expired");
  assert(context.isExpired(null) === false, "no expiry must not be expired");
});

test("BEHAVE #1638: Drive search asks for every bucket and file content", async function () {
  var requested = null;
  var context = loadScript("drive/modules/05_files.js", {
    URLSearchParams: URLSearchParams,
    document: { getElementById: function () { return { innerHTML: "" }; } },
    apiRequest: function (endpoint) {
      requested = endpoint;
      return Promise.resolve([
        { name: "report.pdf", path: "docs/report.pdf", is_dir: false, size: 10, bucket: "bot.gbai" },
      ]);
    },
    renderFiles: function () {},
    showNotification: function () {},
    escapeHtml: function (value) { return String(value); },
    humanizeError: function (value) { return String(value); },
    currentPath: "",
    currentBucket: "bot.gbai",
    currentScope: "user",
    getEffectiveBucket: function () { return "bot.gbai"; },
    console: console,
  });
  return context.searchFiles("report", true).then(function () {
    assert(requested, "search did not call the API");
    assert(requested.indexOf("all_buckets=true") !== -1, "cross-bucket flag missing: " + requested);
    assert(requested.indexOf("content=true") !== -1, "content flag missing: " + requested);
    assert(requested.indexOf("query=report") !== -1, "query missing: " + requested);
  });
});

// ═══════════════════════════════════════════════════════════════════
// ISSUE COVERAGE (one test per open issue)
// ═══════════════════════════════════════════════════════════════════

test("ISSUE #1632: stored Vibe pins are dropped by the launcher", function () {
  var storage = memoryStorage();
  storage.setItem("gb-pinned-launcher", JSON.stringify([
    { kind: "app", appId: "vibe" },
    { kind: "app", appId: "vibe-code" },
    { kind: "app", appId: "chat" },
  ]));
  var window = {};
  var context = loadScript("js/pinned-launcher.js", {
    window: window,
    localStorage: storage,
    document: { addEventListener: function () {}, getElementById: function () { return null; } },
    console: console,
  });
  var list = context.window.GBPinnedLauncher.list();
  assert(list.length === 1 && list[0].appId === "chat",
    "Vibe entries must be purged, got " + JSON.stringify(list));
});

test("ISSUE #1633: payload trailer reader and sidecar stack fallback exist", function () {
  var payload = readFile("../../../botapp/src/desktop/payload.rs");
  assert(payload.indexOf("GB6PAYLOAD!!") !== -1, "trailer magic missing");
  assert(payload.indexOf("payload_digest") !== -1, "checksum verification missing");
  assert(payload.indexOf("--extract-only") !== -1, "offline extraction flag missing");
  var sidecar = readFile("../../../botapp/src/desktop/sidecar.rs");
  assert(sidecar.indexOf("Self::stack_path().join(name)") !== -1,
    "sidecar does not fall back to the extracted stack");
});

test("ISSUE #1634: release workflow verifies the stable aliases", function () {
  var workflow = readFile("../../../.github/workflows/desktop-binaries.yml");
  assert(workflow.indexOf("Verify the stable download aliases") !== -1, "alias check missing");
  assert(workflow.indexOf("sha256sum -c SHA256SUMS-linux --ignore-missing") !== -1,
    "checksum verification missing");
  assert(workflow.indexOf("make_latest: true") !== -1, "latest release not pinned");
});

test("ISSUE #1635: web sources become structured citations", function () {
  var web = readFile("../../../botserver/src/core/bot/pipeline/web_context.rs");
  assert(web.indexOf("pub async fn search_and_inject") !== -1 &&
    web.indexOf("-> Vec<serde_json::Value>") !== -1, "search does not return citations");
  var stream = readFile("../../../botserver/src/core/bot/ws/stream.rs");
  assert(stream.indexOf("pending_citations") !== -1, "final frame does not drain citations");
  var exec = readFile("../../../botserver/src/core/bot/pipeline/exec.rs");
  assert(exec.indexOf("pending_citations") !== -1, "turn does not store citations");
});

test("ISSUE #1637: model tiers select real models and the picker is hydrated", function () {
  var llm = readFile("../../../botserver/src/core/bot/pipeline/enriched_llm.rs");
  assert(llm.indexOf("llm-model-{tier}") !== -1, "per-tier model key missing");
  var handlers = readFile("../../../botserver/src/main_module/routes/chat_handlers.rs");
  assert(handlers.indexOf("/api/chat/models") !== -1, "tier endpoint missing");
  var init = readFile("chat/chat-init.js");
  assert(init.indexOf("/api/chat/models") !== -1, "picker does not hydrate from the endpoint");
  assert(init.indexOf("option.disabled") !== -1, "unconfigured tiers are not disabled");
});

test("ISSUE #1641: attachment content is extracted and injected", function () {
  var text = readFile("../../../botserver/src/core/bot/pipeline/attachment_text.rs");
  assert(text.indexOf("pub fn extract_excerpt") !== -1, "excerpt extractor missing");
  assert(text.indexOf("fn docx_text") !== -1, "DOCX reader missing");
  assert(text.indexOf("is_media") !== -1, "media honesty check missing");
  var exec = readFile("../../../botserver/src/core/bot/pipeline/exec.rs");
  assert(exec.indexOf("turn_hooks::ingest_attachment") !== -1, "attachment not ingested");
  assert(exec.indexOf("attachment_note") !== -1, "attachment content not injected");
});

test("ISSUE #1644: browser actions have a producer, consent and feedback", function () {
  var core = readFile("../../../botserver/src/core/computer_use.rs");
  assert(core.indexOf("propose_from_text") !== -1, "URL producer missing");
  assert(core.indexOf("take_completed_for_prompt") !== -1, "results not fed back");
  var routes = readFile("../../../botserver/src/main_module/routes/computer_use_routes.rs");
  assert(routes.indexOf("/approve") !== -1 && routes.indexOf("/execute") !== -1,
    "consent/execute endpoints missing");
  var client = readFile("chat/modules/60_computer_use.js");
  assert(client.indexOf("/execute") !== -1 && client.indexOf("case \"screenshot\"") === -1,
    "client still fakes screenshots or lacks execute");
});

test("ISSUE #1645: terminal consent is bound to an audit record", function () {
  var routes = readFile("../../../botserver/src/main_module/routes/computer_use_routes.rs");
  assert(routes.indexOf("body.action_id") !== -1, "execution not bound to an action id");
  assert(routes.indexOf("/api/terminal/audit") !== -1, "audit endpoint missing");
  var migration = readFile("../../../botserver/migrations/6.8.3-terminal-audit/up.sql");
  assert(migration.indexOf("CREATE TABLE IF NOT EXISTS terminal_audit") !== -1,
    "audit table missing");
  var client = readFile("chat/modules/61_terminal_use.js");
  assert(client.indexOf("/api/terminal/audit") !== -1, "audit panel not reachable");
});

test("ISSUE #1649: the published page is generated from the record", function () {
  var record = JSON.parse(readFile("../../../desktop/gb6/features.json"));
  var ids = record.features.map(function (f) { return f.id; });
  assert(ids.length === 14, "expected 14 features, got " + ids.length);
  assert(ids.indexOf("GB6-22") === -1 && ids.indexOf("GB6-24") === -1,
    "removed features still listed");
  record.features.forEach(function (feature) {
    assert(feature.evidence && feature.evidence.issue, feature.id + " has no issue link");
    if (feature.status === "shipped") {
      assert(feature.evidence.e2e, feature.id + " is shipped without E2E evidence");
      assert(feature.last_verified && feature.last_verified.date,
        feature.id + " has no verification date");
    }
  });
});

test("ISSUE #1650: the epic definition of done is recorded", function () {
  var record = JSON.parse(readFile("../../../desktop/gb6/features.json"));
  assert(record.epic && record.epic.issue === "#1650", "epic metadata missing");
  assert(Array.isArray(record.epic.definition_of_done) &&
    record.epic.definition_of_done.length >= 4, "definition of done missing");
  assert(record.epic.removed_from_scope.indexOf("GB6-22") !== -1,
    "removed scope not recorded");
});

// ═══════════════════════════════════════════════════════════════════
// RESULTS
// ═══════════════════════════════════════════════════════════════════

function report() {
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
}

if (PENDING.length) {
  Promise.all(PENDING).then(report);
} else {
  report();
}
