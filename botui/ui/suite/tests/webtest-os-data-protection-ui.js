/**
 * Offline UI guard for #2470 — loads the real settings-os-appliance.js in a
 * `vm` with stubbed window/document/fetch and asserts that the data-protection
 * actions fail closed. No server, no browser, no cargo, no real credentials.
 *
 * Run with: node botui/ui/suite/tests/webtest-os-data-protection-ui.js
 * Optional: GB_OS_APPLIANCE_JS=/path/to/module.js to point at a modified copy
 * (used by the non-vacuity probe). The fake token below is a test fixture,
 * never a credential.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var FILE = process.env.GB_OS_APPLIANCE_JS || path.resolve(
    __dirname, "..", "settings", "settings-os-appliance.js"
);
var FAKE_TOKEN = "fixture-token-2470-abcdef";
var FAKE_SCOPE = "full-archive";
var PASS = 0;
var FAIL = 0;

function check(name, condition, detail) {
  if (condition) {
    PASS++;
    console.log("[PASS] " + name);
  } else {
    FAIL++;
    console.log("[FAIL] " + name + (detail ? " — " + detail : ""));
  }
}

function finish() {
  console.log("=".repeat(64));
  console.log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  console.log("=".repeat(64));
  process.exitCode = FAIL === 0 ? 0 : 1;
}

var src;
try {
  src = fs.readFileSync(FILE, "utf8");
} catch (e) {
  check("module readable: " + FILE, false, String(e));
  finish();
  process.exit(1);
}

var calls = [];
var failures = false;
var echoToken = false;

function jsonResponse(ok, payload) {
  return Promise.resolve({
    ok: ok,
    status: ok ? 200 : 503,
    json: function () { return Promise.resolve(payload); }
  });
}

function fetchStub(url, opts) {
  calls.push({ url: url, method: (opts && opts.method) || "GET", body: opts && opts.body });
  if ((opts && opts.method) === "POST") {
    if (failures) {
      return jsonResponse(false, { error: "raw failure " + FAKE_TOKEN });
    }
    return jsonResponse(true, {
      message: echoToken ? ("done with " + FAKE_TOKEN) : "done"
    });
  }
  if (url === "/api/os/data-protection") {
    return jsonResponse(true, {
      reset: { confirmation_token: FAKE_TOKEN },
      export: { contract: { request: { scope: FAKE_SCOPE } } }
    });
  }
  if (url === "/api/os/storage") {
    return jsonResponse(true, { message: "storage ok" });
  }
  return jsonResponse(false, {});
}

function element(id) {
  return {
    id: id,
    value: "",
    disabled: false,
    textContent: "",
    attrs: {},
    listeners: {},
    getAttribute: function (key) {
      return Object.prototype.hasOwnProperty.call(this.attrs, key) ? this.attrs[key] : null;
    },
    setAttribute: function (key, value) { this.attrs[key] = String(value); },
    addEventListener: function (key, fn) {
      if (!this.listeners[key]) this.listeners[key] = [];
      this.listeners[key].push(fn);
    }
  };
}

var ids = [
  "osDataReset", "osDataResetToken", "osDataReason",
  "osDataExportStatus", "osDataSnapshotStatus", "osDataResetStatus",
  "os-data-status-body", "os-storage-reset-status-body", "osCriticalAction"
];
var nodes = {};
ids.forEach(function (id) { nodes[id] = element(id); });

function fireTokenInput() {
  (nodes.osDataResetToken.listeners.input || []).forEach(function (fn) { fn(); });
}

function postsTo(suffix) {
  return calls.filter(function (c) {
    return c.method === "POST" && c.url.indexOf(suffix) !== -1;
  });
}

function postedBody(suffix) {
  var found = postsTo(suffix);
  if (!found.length) return null;
  try { return JSON.parse(found[found.length - 1].body); } catch (e) { return null; }
}

function flush() {
  return new Promise(function (resolve) { setImmediate(resolve); });
}

var win = {
  location: { hash: "" },
  _t: function (key, fallback) {
    if (key === "os-backend-unavailable") return "LOCALIZED: " + fallback;
    return fallback;
  },
  addEventListener: function () {}
};
var doc = {
  readyState: "complete",
  getElementById: function (id) { return nodes[id] || null; },
  addEventListener: function () {},
  dispatchEvent: function () {}
};
var sandbox = {
  window: win,
  document: doc,
  fetch: fetchStub,
  console: console,
  Promise: Promise,
  JSON: JSON,
  localStorage: { getItem: function () { return null; }, setItem: function () {} },
  CustomEvent: function () {}
};

async function main() {
  vm.createContext(sandbox);
  vm.runInContext(src, sandbox, { filename: FILE });
  var gb = win.GBDataProtection;

  check("module publishes GBDataProtection", !!gb);
  check("module publishes loadOsAppliancePage", typeof win.loadOsAppliancePage === "function");

  // Precondition guards: no status payload yet, so no action may POST.
  await gb.manifest();
  check("export does not POST before a status payload", postsTo("/export").length === 0);
  nodes.osDataReason.value = "";
  await gb.snapshot();
  check("snapshot does not POST without a reason", postsTo("/snapshots").length === 0);

  check("reset button starts disabled", nodes.osDataReset.disabled === true);

  win.loadOsAppliancePage("os-storage-reset");
  await flush();
  await flush();
  check("reset stays disabled after status until exact token typed", nodes.osDataReset.disabled === true);

  // Typed-token gate, exercised with a real server-issued token in scope.
  nodes.osDataResetToken.value = "wrong-fixture-token";
  fireTokenInput();
  await gb.reset();
  check("no reset POST with a wrong token", postsTo("/reset").length === 0);
  check("reset button stays disabled with a wrong token", nodes.osDataReset.disabled === true);

  nodes.osDataResetToken.value = FAKE_TOKEN;
  fireTokenInput();
  check("reset button enables only on the exact server token", nodes.osDataReset.disabled === false);

  await gb.reset();
  var resetBody = postedBody("/reset");
  check("reset POST carries the exact confirmation token",
    !!resetBody && resetBody.confirmation === FAKE_TOKEN, JSON.stringify(resetBody));

  await gb.manifest();
  var exportBody = postedBody("/export");
  check("export POST uses the manifest scope from the status payload",
    !!exportBody && exportBody.scope === FAKE_SCOPE, JSON.stringify(exportBody));

  nodes.osDataReason.value = "nightly-fixture-reason";
  await gb.snapshot();
  var snapshotBody = postedBody("/snapshots");
  check("snapshot POST carries the reason",
    !!snapshotBody && snapshotBody.reason === "nightly-fixture-reason", JSON.stringify(snapshotBody));

  // Failed requests: localized message, never a raw token/error echo.
  failures = true;
  await gb.reset();
  var resetStatus = nodes.osDataResetStatus.textContent;
  check("failed reset shows a localized status message", resetStatus.indexOf("LOCALIZED:") === 0, resetStatus);
  check("failed reset never echoes the token", resetStatus.indexOf(FAKE_TOKEN) === -1, resetStatus);
  await gb.manifest();
  var exportStatus = nodes.osDataExportStatus.textContent;
  check("failed export shows a localized status message", exportStatus.indexOf("LOCALIZED:") === 0, exportStatus);
  check("failed export never echoes the token", exportStatus.indexOf(FAKE_TOKEN) === -1, exportStatus);

  failures = false;
  echoToken = true;
  await gb.manifest();
  var echoStatus = nodes.osDataExportStatus.textContent;
  check("a server message echoing the token is redacted", echoStatus.indexOf(FAKE_TOKEN) === -1, echoStatus);

  // Every status region written by the module must be an ARIA live region.
  ["os-storage-reset-status-body", "os-data-status-body", "osDataExportStatus",
    "osDataSnapshotStatus", "osDataResetStatus"].forEach(function (id) {
    check(id + " has role=status", nodes[id].getAttribute("role") === "status",
      String(nodes[id].getAttribute("role")));
    check(id + " has aria-live=polite", nodes[id].getAttribute("aria-live") === "polite",
      String(nodes[id].getAttribute("aria-live")));
  });

  // Static wiring: the safety gates must exist in source, not just in a stub.
  var resetStart = src.indexOf("function requestReset");
  var resetEnd = src.indexOf("\n  }", resetStart);
  var resetFn = src.slice(resetStart, resetEnd >= 0 ? resetEnd : src.length);
  var guardAt = resetFn.indexOf("input.value !== resetToken");
  var postAt = resetFn.indexOf('postJson("/api/os/data-protection/reset"');
  check("reset compares the typed value to the server token",
    guardAt >= 0 && /!resetToken/.test(resetFn));
  check("reset POST is placed after the typed-token gate", guardAt >= 0 && postAt > guardAt);
  check("status role/aria-live are set from JS",
    /setAttribute\("role", "status"\)/.test(src) &&
    /setAttribute\("aria-live", "polite"\)/.test(src));

  var actionSrc = src.slice(src.indexOf("function unavailableMessage"));
  var usedKeys = [];
  var re = /\bt\(\s*"([^"]+)"\s*,\s*"([^"]*)"/g;
  var match;
  while ((match = re.exec(actionSrc)) !== null) usedKeys.push([match[1], match[2]]);
  check("every data-protection i18n key has an English fallback",
    usedKeys.length > 0 && usedKeys.every(function (pair) { return pair[1].length > 0; }),
    JSON.stringify(usedKeys));
  check("os-backend-unavailable carries an English fallback",
    usedKeys.some(function (pair) { return pair[0] === "os-backend-unavailable" && pair[1].length > 0; }));
  check("os-data-not-implemented carries an English fallback",
    usedKeys.some(function (pair) { return pair[0] === "os-data-not-implemented" && pair[1].length > 0; }));

  finish();
}

main().catch(function (e) {
  check("harness ran without an exception", false, String(e));
  finish();
});
