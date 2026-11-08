/**
 * webtest_gb7 — security and functionality fixes (SEC-01..07, FUNC-01..05)
 * Split from webtest_gb7.js for #1731; keeps the original test order.
 */
"use strict";

const {
  fs,
  path,
  TESTS_DIR,
  test,
  assert,
  readFile,
} = require("./00_harness");

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
  var src = fs.readFileSync(path.resolve(TESTS_DIR, "../../../../botserver/crates/botdrive/src/drive_handlers/cached_quota.rs"), "utf8");
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
  var src = readFile("chat/chat-send.js");
  assert(src.indexOf("GBPrivacy") !== -1, "GBPrivacy not referenced in chat-send");
  assert(src.indexOf("save_memory") !== -1, "save_memory check not found");
});

test("FUNC-04: Anonymous sessions skip memory and drive_context", function () {
  var src = readFile("chat/chat-send.js");
  assert(src.indexOf("isAnonymous") !== -1, "isAnonymous check not found");
  assert(src.indexOf("!isAnonymous") !== -1, "isAnonymous gating not applied");
});

test("FUNC-05: no_security wired to auth middleware require_auth", function () {
  var src = fs.readFileSync(path.resolve(TESTS_DIR, "../../../../botserver/src/main_module/routes/security_setup.rs"), "utf8");
  assert(src.indexOf("no_security") !== -1, "no_security not referenced in security_setup");
  assert(src.indexOf("require_auth = false") !== -1, "require_auth not set to false for no_security");
});
