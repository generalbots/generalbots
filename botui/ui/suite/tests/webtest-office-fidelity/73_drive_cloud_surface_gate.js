/**
 * #2925 — deterministic static coverage for desktop-only Drive surfaces.
 *
 * On a cloud/multi-tenant host (`no_security:false`, `shell_mode:false`) Drive
 * must not advertise Home / Connected / Removable, and must not issue the OS /
 * connector probes those surfaces own. The authoritative capability is the
 * /api/product config already consumed by isDesktopLocal(); no host, port or
 * URL heuristics are allowed. Desktop-local must keep every surface intact.
 *
 * Probes (source assertions only — no server, browser, cargo or i18n change):
 *   (a) Home and Connected ship hidden in drive.html markup (no first-paint flash);
 *   (b) isDesktopLocal() reads product flags, never host/port/URL;
 *   (c) 99_init.js reveals both tabs only under the gate and re-applies on the
 *       async `gb-product-config` event, while keeping the shell bindings;
 *   (d) 12_removable.js guards panel creation, poll and eject, with a late-config
 *       retry and the exported desktop API preserved;
 *   (e) 11_external.js guards `/connections` and keeps the consent/scope literals;
 *   (f) no Drive module contains the LAN-exposure probe or an unguarded OS/connector
 *       URL, and the runner requires this guard.
 *
 * NON-VACUITY: every span is proved non-empty before its absence checks run.
 * Reverting any one of the shipped changes makes the matching case FAIL.
 * GB_DRIVE_GATE_PROBE_ROOT can point at a /tmp copy for the mutation probe
 * without touching the repository.
 */
"use strict";

var h = require("./00_harness");
var fs = require("fs");
var path = require("path");

var HTML = "botui/ui/suite/drive/drive.html";
var INIT = "botui/ui/suite/drive/modules/99_init.js";
var REMOVABLE = "botui/ui/suite/drive/modules/12_removable.js";
var EXTERNAL = "botui/ui/suite/drive/modules/11_external.js";
var UTILS = "botui/ui/suite/drive/modules/03_utils.js";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";

function readSrc(rel) {
  var probe = process.env.GB_DRIVE_GATE_PROBE_ROOT;
  if (probe) {
    var candidate = path.join(probe, rel);
    if (fs.existsSync(candidate)) return fs.readFileSync(candidate, "utf8");
  }
  return h.readRepo(rel);
}

function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + ">");
}

function absent(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1,
    (msg || "source must not contain") + " <" + needle + "> :: " + src.slice(0, 160));
}

function span(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  h.assert(end > start, "empty source span between <" + startMarker + "> and <" + endMarker + ">");
  return src.slice(start, end);
}

function guardedBefore(body, call, msg) {
  var pos = body.indexOf(call);
  h.assert(pos !== -1, "missing call <" + call + ">");
  var before = body.slice(0, pos);
  h.assert(before.indexOf("if (!isDesktopLocal()) return;") !== -1,
    (msg || "call") + " <" + call + "> is not capability-gated");
}

function tabTag(html, tabId) {
  var match = new RegExp('<button[^>]*id="tab-' + tabId + '"[^>]*>').exec(html);
  h.assert(!!match, "tab button <" + tabId + "> missing from drive.html");
  h.assert(match[0].length > 40, "non-vacuity: tab <" + tabId + "> markup is trivial");
  return match[0];
}

h.staticTest("#2925 Home and Connected ship hidden before first paint", function () {
  var html = readSrc(HTML);
  h.assert(html.length > 5000, "non-vacuity: drive.html must be substantial");
  [tabTag(html, "home"), tabTag(html, "external")].forEach(function (tag) {
    h.assert(tag.indexOf("display:none") !== -1 || /\shidden(\s|>)/.test(tag),
      "desktop-only tab not hidden in markup :: " + tag);
  });
  // Cloud has no OS/network shell tabs in Drive's own markup either.
  absent(html, "/api/os/", "drive.html must not probe OS endpoints");
});

h.staticTest("#2925 capability gate is /api/product based, not a host heuristic", function () {
  var src = readSrc(UTILS);
  var fn = span(src, "function isDesktopLocal() {", "\n}");
  h.assert(fn.length > 100, "non-vacuity: isDesktopLocal() body must be substantial");
  contains(fn, "window.productConfig", "capability must come from product config");
  contains(fn, "shell_mode", "shell_mode capability missing");
  contains(fn, "no_security", "no_security capability missing");
  ["location.host", "location.hostname", "location.port", "location.origin", "location.href",
    "__BOT_IS_PUBLIC__"]
    .forEach(function (heuristic) { absent(fn, heuristic, "non-capability flag in gate"); });
});

h.staticTest("#2925 init reveals desktop tabs only under the gate, keeps bindings", function () {
  var src = readSrc(INIT);
  h.assert(src.length > 4000, "non-vacuity: 99_init.js must be substantial");
  var gate = span(src, "function applyDesktopOnlyTabs() {", "\n}");
  contains(gate, "isDesktopLocal()", "tab gate does not consult capability");
  contains(gate, '"tab-home"', "Home tab not covered by gate");
  contains(gate, '"tab-external"', "Connected tab not covered by gate");
  contains(gate, "style.display", "tab gate does not change visibility");
  contains(src, 'document.addEventListener("gb-product-config", applyDesktopOnlyTabs);',
    "late product config does not re-apply the tab gate");
  var initStart = span(src, "async function init() {", "bindTopTabs();");
  contains(initStart, "applyDesktopOnlyTabs();", "init() does not apply the gate");
  contains(src, "if (window.DriveExternal && DriveExternal.bind) DriveExternal.bind();",
    "desktop shell binding was removed");
  contains(src, '"branchdrive", "bots", "root", "desktop", "external"',
    "saved-tab reset for the legacy external tab was removed");
});

h.staticTest("#2925 removable panel is never created or polled on cloud", function () {
  var src = readSrc(REMOVABLE);
  h.assert(src.length > 2000, "non-vacuity: 12_removable.js must be substantial");
  var loadFn = span(src, "function load() {", "function init() {");
  guardedBefore(loadFn, 'fetch("/api/os/removable"', "removable poll");
  var ejectFn = span(src, "function eject(device) {", "function load() {");
  guardedBefore(ejectFn, 'fetch("/api/os/removable/eject"', "removable eject");
  var initFn = span(src, "function init() {", "\n  // #2925");
  var pos = initFn.indexOf("panel();");
  h.assert(pos !== -1, "panel() creation missing from removable init");
  h.assert(initFn.slice(0, pos).indexOf("if (!isDesktopLocal()) return;") !== -1,
    "removable panel is created without the capability gate");
  contains(src, 'document.addEventListener("gb-product-config", init);',
    "late product config does not retry the removable panel on desktop");
  contains(src, "window.DriveRemovable = { load: load, eject: eject };",
    "desktop removable API export was removed");
});

h.staticTest("#2925 external loader never requests /connections on cloud", function () {
  var src = readSrc(EXTERNAL);
  h.assert(src.length > 5000, "non-vacuity: 11_external.js must be substantial");
  var loadFn = span(src, "async function loadExternalTab() {", "async function browseExternalProvider(");
  guardedBefore(loadFn, 'extApiRequest("/connections")', "connector listing");
  // Desktop consent/scope behaviour pinned by the GB7 guard stays intact.
  ["showConnectorConsent", "requested_scopes", "scope_descriptions", "Granted scopes",
    "Visibility:", "consented_at"].forEach(function (literal) {
    contains(src, literal, "desktop connector literal removed: " + literal);
  });
  contains(src, "window.DriveExternal = {", "desktop external API export was removed");
  contains(src, "load: loadExternalTab", "desktop external loader export was removed");
  contains(src, "bind: bindExternalEvents", "desktop external binding export was removed");
});

h.staticTest("#2925 no Drive module issues LAN-exposure or unguarded OS probes", function () {
  var probe = process.env.GB_DRIVE_GATE_PROBE_ROOT;
  var root = probe && fs.existsSync(path.join(probe, "botui/ui/suite/drive/modules"))
    ? probe
    : path.resolve(__dirname, "..", "..", "..", "..", "..");
  var modulesDir = path.join(root, "botui/ui/suite/drive/modules");
  h.assert(fs.existsSync(modulesDir), "drive modules directory missing: " + modulesDir);
  var files = fs.readdirSync(modulesDir).filter(function (name) { return /\.js$/.test(name); });
  h.assert(files.length >= 25, "non-vacuity: drive module set must be complete");
  files.forEach(function (name) {
    var src = fs.readFileSync(path.join(modulesDir, name), "utf8");
    absent(src, "lan-exposure", name + " must not probe LAN exposure");
    absent(src, "/api/external-drives/connections", name + " must not probe connectors directly");
    var at = src.indexOf('fetch("/api/os/removable"');
    if (at !== -1) {
      h.assert(name === "12_removable.js", "unexpected OS removable probe in " + name);
      h.assert(src.slice(0, at).indexOf("if (!isDesktopLocal()) return;") !== -1,
        name + " has an unguarded OS removable probe");
    }
  });
  contains(readSrc(RUNNER), 'require("./webtest-office-fidelity/73_drive_cloud_surface_gate")',
    "this guard is no longer required by the runner");
});

h.staticTest("LAN-exposure requests are admin-gated on every page (#2441)", function () {
  var src = h.readRepo("botui/ui/suite/js/network-security.js");
  contains(src, "function adminSurface()", "the shared admin predicate must ship");
  var gates = src.split("if (!adminSurface()) return;").length - 1;
  h.assert(gates >= 2, "both LAN-exposure fetches must be admin-gated (got " + gates + ")");
  contains(src, "gb:auth:login", "an admin signing in late must re-read the flag");
});
