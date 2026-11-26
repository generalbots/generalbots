/**
 * #2967 — deterministic static guard for the VM tabs 401 flood fix.
 *
 * A signed-in admin sweep saw 203 GET /api/browser/vm/<vm>/tabs -> 401 events
 * across ~20 apps. The Browser app's tab-strip module armed a raw setInterval
 * that survived `gb-window-close`: closing the window detached the app root but
 * left the timer alive, so every subsequently opened app kept polling. The
 * poller also fell back to "default" when /api/agent/vm/status reported no VM,
 * and pre-attached a stale storage token that the central fetch interceptor
 * deliberately refuses to overwrite — a guaranteed 401.
 *
 * The fix has three halves, all pinned here:
 *   (1) owning-app gate — every poll re-checks the Browser root is connected;
 *   (2) Agent-VM gate — no VM / no session means no timer and no request;
 *   (3) canonical token — window.getGBAccessToken() is the only credential.
 * Both shipped VM tab modules (Browser app + desktop headless page) are covered.
 *
 * Non-vacuity: GB_VM_TABS_PROBE_ROOT may point at a /tmp tree holding a mutated
 * copy of vm-tabs.js (raw setInterval restored) — the owning-app case then FAILS.
 *
 * No server, browser, cargo or i18n change is involved; the driver owns live
 * UAT, issue closure and CI deployment.
 */
"use strict";

var h = require("./00_harness");
var fs = require("fs");
var path = require("path");

var APP_MODULE = "botui/ui/suite/browser/vm-tabs.js";
var DESKTOP_MODULE = "botui/ui/suite/desktop/vm-tabs.js";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var GUARD = "botui/ui/suite/tests/webtest-office-fidelity/93_vm_tabs_poller_gate.js";
var PROBE_ROOT = process.env.GB_VM_TABS_PROBE_ROOT || "";

function read(rel) {
  if (PROBE_ROOT && rel === APP_MODULE) {
    var candidate = path.join(PROBE_ROOT, "vm-tabs.js");
    if (fs.existsSync(candidate)) return fs.readFileSync(candidate, "utf8");
  }
  return h.readRepo(rel);
}
function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}
function absent(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1, (msg || "source must not contain") + " <" + needle + ">");
}
function countIn(src, needle) { return src.split(needle).length - 1; }
function span(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  h.assert(end > start, "empty span between <" + startMarker + "> and <" + endMarker + ">");
  return src.slice(start, end);
}

h.staticTest("#2967 the Browser app poller re-checks its owning app on every fetch", function () {
  var src = read(APP_MODULE);
  h.assert(src.length > 8000, "non-vacuity: vm-tabs.js is the shipped module, not a stub");
  contains(src, "function browserAppActive()", "the owning-app predicate must exist");
  var active = span(src, "function browserAppActive() {", "\n    }");
  contains(active, '.gb-app[data-gb-app="browser"]',
    "the predicate must resolve the Browser app root");
  contains(active, "root.isConnected",
    "a closed window's detached root must block the poll");
  contains(active, "!document.hidden",
    "a backgrounded tab must not poll either");
  var load = span(src, "function loadTabs() {", "\n    }");
  contains(load, "if (!vmEnabled || !browserAppActive()) return Promise.resolve(null);",
    "the tab fetch must short-circuit outside the owning app / without a VM");
  contains(load, 'api(tabPath())', "the VM tabs endpoint must stay behind the gate");
});

h.staticTest("#2967 no VM and no session mean no timer and no flood", function () {
  var src = read(APP_MODULE);
  var resolve = span(src, "function resolveVm() {", "\n    }");
  contains(resolve, "vmEnabled = true;",
    "the VM-on flag may only be set when a VM was actually resolved");
  contains(resolve, "data && data.has_vm && data.vm_name",
    "the Agent VM status must gate the tab surface");
  absent(resolve, "return DEFAULT_VM;",
    "the old unconditional 'default' fallback must be gone");
  contains(resolve, "if (!hasSession()) return Promise.resolve(null);",
    "an unauthenticated tab must not even call the VM status endpoint");
  var wire = span(src, "resolveVm().then(function (name) {", "\n        });");
  contains(wire, "if (!name) {", "a missing VM must abort arming the poller");
  contains(wire, 'strip.style.display = "none"', "no VM means no tab strip");
  h.assertEqual(countIn(src, "setInterval(loadTabs, POLL_MS)"), 0,
    "the leaked raw timer must be gone");
  contains(src, 'window.GBAppLifecycle.interval("browser", poll, POLL_MS)',
    "the poll must be a lifecycle-owned timer so window close clears it");
  contains(src, 'window.GBAppLifecycle.begin("browser")',
    "the app must announce its lifecycle bucket on init");
  var close = span(src, 'document.addEventListener("gb-window-close", function (event) {', "\n    });");
  contains(close, "stopPolling();", "window close must tear the timer down");
  contains(close, 'id.indexOf("browser-") === 0', "forced-new Browser instances must also stop");
});

h.staticTest("#2967 the canonical token is the only credential for VM tabs", function () {
  var src = read(APP_MODULE);
  contains(src, "function hasSession()", "one session predicate must exist");
  contains(src, 'typeof window.getGBAccessToken === "function" && !!window.getGBAccessToken()',
    "getGBAccessToken() is the single session authority");
  var headers = span(src, "function authHeaders() {", "\n    }");
  contains(headers, 'window.getGBAccessToken()',
    "the Bearer token must come from the canonical accessor");
  absent(headers, "localStorage", "no stale localStorage token may be pre-attached");
  absent(headers, "sessionStorage", "no stale sessionStorage token may be pre-attached");
  var api = span(src, "function api(path, method, body) {", "\n    }");
  contains(api, "if (!hasSession()) return Promise.resolve(null);",
    "the request must be dropped before fetch when the canonical token is absent");
  absent(src, "originalFetch", "the central interceptor must not be bypassed");
});

h.staticTest("#2967 the desktop headless VM tabs module shares the same gates", function () {
  var src = read(DESKTOP_MODULE);
  h.assert(src.length > 3000, "non-vacuity: desktop/vm-tabs.js is a shipped module");
  contains(src, "function vmAppActive()", "the owning-surface predicate must exist");
  contains(src, "strip.isConnected", "a detached strip must block the poll");
  contains(src, "!document.hidden", "a backgrounded page must not poll");
  var refresh = span(src, "function refresh() {", "\n    }");
  contains(refresh, "if (!vmEnabled || !vmAppActive() || !hasSession()) return Promise.resolve(null);",
    "every refresh must be gated on VM-on, owning surface and session");
  contains(refresh, "fetch(path(), { headers: authHeaders() })",
    "the VM tabs endpoint must stay behind the gate");
  var headers = span(src, "function authHeaders() {", "\n    }");
  contains(headers, "window.getGBAccessToken()", "canonical token only");
  absent(headers, "localStorage", "no stale storage token");
  absent(headers, "sessionStorage", "no stale session token");
  h.assertEqual(countIn(src, "setInterval(refresh, POLL_MS)"), 0,
    "the leaked raw timer must be gone");
  contains(src, "window.GBAppLifecycle.interval('browser', poll, POLL_MS)",
    "the poll must be lifecycle-owned");
});

h.staticTest("#2967 guard 93 is registered and mutation-probeable", function () {
  [APP_MODULE, DESKTOP_MODULE].forEach(function (rel) {
    var src = h.readRepo(rel);
    h.assert(src.split("\n").length < 450, rel + " must stay under 450 lines");
    h.assert(src.length > 500, rel + " must remain a shipped module");
  });
  var guardSrc = h.readRepo(GUARD);
  h.assert(guardSrc.split("\n").length < 450, "guard must stay under 450 lines");
  contains(h.readRepo(RUNNER), 'require("./webtest-office-fidelity/93_vm_tabs_poller_gate");',
    "guard 93 must be registered in webtest-office-fidelity.js to run in CI");
  var hooks = guardSrc.match(/=\s*process\.env\.GB_VM_TABS_PROBE_ROOT\b/g) || [];
  h.assertEqual(hooks.length, 1,
    "the guard exposes exactly one GB_VM_TABS_PROBE_ROOT hook for mutation testing");
});

h.note("#2967: offline static proof only — no browser/server/cargo, no locale edit. " +
  "The driver owns live UAT: open Browser with the agent VM off (no /api/browser/vm/*/tabs " +
  "requests), then with the VM on and close/reopen the window (no 401 flood from other apps). " +
  "GB_VM_TABS_PROBE_ROOT pointing at a /tmp copy with the raw setInterval restored fails the " +
  "no-flood case.");
