/**
 * Anonymous Drive deep-link regression guard (offline, no browser/server).
 *
 * A deep link resolves apps from `window.APPS_REGISTRY`, which 01_registry.js
 * filters for anonymous visitors. Before the fix, `drive` was absent so the
 * fallback hxGet became `/suite/partials/drive.html` — a file that does not
 * exist — leaving a blank window stuck on "Application loading...".
 *
 * The fix asks the registry whether a *known* app was withheld by the
 * anonymous gate and, when it was, injects a localized login prompt using two
 * catalogue keys that already ship in all eight locales. Unknown ids keep the
 * legacy partial fallback; preview gating and authenticated deep links are
 * untouched.
 *
 * Probes:
 *   (a) openDeepLink checks known + auth-withheld BEFORE any fetch;
 *   (b) the registry exports the same predicate it uses for its own filter;
 *   (c) runtime: withheld drive → zero fetches, login prompt injected;
 *   (d) runtime: authenticated drive → real partial still fetched;
 *   (e) runtime: unknown id → legacy fallback still fetched;
 *   (f) no new i18n key, no locale edit, pinned literals and APIs survive.
 *
 * GB_DEEPLINK_PROBE_ROOT can point at a /tmp tree for mutation probes without
 * touching the repository. The driver owns issue closure and live browser UAT.
 */
"use strict";

var h = require("./00_harness");
var fs = require("fs");
var path = require("path");

var DEEPLINK = "botui/ui/suite/js/window-manager/06_deeplinks.js";
var REGISTRY = "botui/ui/suite/js/window-manager/01_registry.js";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];
var KEYS = ["cloud-appstores-login-first", "auth-go-to-login"];

function readSrc(rel) {
  var probe = process.env.GB_DEEPLINK_PROBE_ROOT;
  if (probe) {
    var candidate = path.join(probe, rel);
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
function span(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  h.assert(end > start, "empty span between <" + startMarker + "> and <" + endMarker + ">");
  return src.slice(start, end);
}
function countIn(src, needle) { return src.split(needle).length - 1; }

h.staticTest("#2944 openDeepLink gates a known-but-withheld app before any fetch", function () {
  var src = readSrc(DEEPLINK);
  var fn = span(src, "openDeepLink(appId, params, opts) {", "_openLoginRequired(appId, title, opts) {");
  h.assert(fn.length > 800, "non-vacuity: openDeepLink body must be substantial, got " + fn.length);
  contains(fn, "const app = (window.APPS_REGISTRY || APPS_REGISTRY).find((a) => a.id === appId);",
    "filtered registry still resolves the app first");
  contains(fn, "const known = APPS_REGISTRY.find((a) => a.id === appId);",
    "full embedded registry must be consulted for known ids");
  contains(fn, "WM.isAuthWithheldApp(appId)", "auth-withheld predicate must gate the branch");
  contains(fn, "this._openLoginRequired(appId, known.title || appId, opts);", "auth prompt is opened");
  var knownAt = fn.indexOf("const known = ");
  var branchAt = fn.indexOf("WM.isAuthWithheldApp(appId)");
  var fetchAt = fn.indexOf("fetch(hxGet");
  h.assert(knownAt !== -1 && branchAt > knownAt && fetchAt > branchAt,
    "auth branch must run before the partial fetch (known=" + knownAt + " branch=" + branchAt + " fetch=" + fetchAt + ")");
  contains(fn, "`/suite/partials/${appId}.html`", "legacy partial fallback literal must survive for unknown ids");
  contains(src, "if (!window.GBPreviewMode.isOn() && isPreviewId(appId)) {",
    "preview gate still runs before the auth branch");
});

h.staticTest("#2944 registry exports the same predicate it uses to filter anonymous apps", function () {
  var src = readSrc(REGISTRY);
  contains(src, 'var AUTH_GATED_IDS = ["drive", "terminal", "editor", "browser", "integrations", "admin"];',
    "one authoritative privileged-id list");
  contains(src, "anonymousGate = !authenticated && !noSecurityBuild;",
    "the gate state is refreshed on every catalog rebuild");
  contains(src, "if (anonymousGate) merged = merged.filter(function (app) { return AUTH_GATED_IDS.indexOf(app.id) === -1; });",
    "the shipped anonymous filter uses the shared list");
  contains(src, "WM.isAuthWithheldApp = function (id) { return anonymousGate && AUTH_GATED_IDS.indexOf(id) !== -1; };",
    "the predicate is exported to deep links");
  ["drive", "terminal", "editor", "browser", "integrations", "admin"].forEach(function (id) {
    contains(src, '"' + id + '"', "privileged id missing from the shared list: " + id);
  });
  contains(src, "window.APPS_REGISTRY = merged;", "filtered registry assignment preserved");
  contains(src, "gb-preview-changed", "preview event preserved");
});

h.staticTest("#2944 localized prompt reuses only existing catalogue keys", function () {
  var src = readSrc(DEEPLINK);
  var fn = span(src, "_openLoginRequired(appId, title, opts) {", "// #2226 — a deep link");
  h.assert(fn.length > 300, "non-vacuity: login-prompt body must be substantial, got " + fn.length);
  contains(fn, 't("cloud-appstores-login-first", "Please login first")', "existing login-first key");
  contains(fn, 't("auth-go-to-login", "Go to Login")', "existing go-to-login key");
  contains(fn, "<p>", "prompt is readable text, not a blank shell");
  contains(fn, '<a href="', "prompt offers a login link");
  absent(fn, 'role="status"', "no new status-region literal (aria baseline is pinned)");
  absent(fn, "aria-live=", "no new aria-live literal (aria baseline is pinned)");
  contains(fn, 'window.GB_LOGIN_URL || "/login"', "login URL comes from the injected login origin");
  contains(fn, "encodeURIComponent(window.location.href)", "return_to keeps the current suite URL");
  h.assertEqual(countIn(src, "gbT("), 1, "one t() helper calls gbT; no extra direct calls were added");
  KEYS.forEach(function (key) {
    h.assertEqual(countIn(src, '"' + key + '"'), 1, "exactly one reuse of key " + key);
  });
  LOCALES.forEach(function (locale) {
    var dir = path.join(h.REPO_DIR, "botlib", "locales", locale);
    var all = fs.readdirSync(dir).filter(function (name) { return /\.ftl$/.test(name); })
      .map(function (name) { return fs.readFileSync(path.join(dir, name), "utf8"); }).join("\n");
    KEYS.forEach(function (key) {
      h.assert(new RegExp("^" + key + "\\s*=", "m").test(all), locale + " is missing key " + key);
    });
  });
});

h.staticTest("#2944 non-vacuity: small modules, pinned APIs and required guard", function () {
  [DEEPLINK, REGISTRY].forEach(function (rel) {
    var src = readSrc(rel);
    var lines = src.split("\n").length;
    h.assert(lines < 450, rel + " must stay under 450 lines, got " + lines);
    h.assert(src.length > 5000, rel + " must remain a shipped module");
  });
  var deep = readSrc(DEEPLINK);
  contains(deep, "openDeepLink(appId, params, opts)", "deep-link entry point preserved");
  contains(deep, "isPreviewId(appId)", "preview deep-link gate preserved");
  contains(deep, 'new CustomEvent("gb:deep-link"', "delta re-target event preserved");
  contains(deep, "offerPreviewApp(appId, params, opts)", "preview app offer preserved");
  contains(readSrc(RUNNER), 'require("./webtest-office-fidelity/80_deeplink_auth_gate")',
    "this guard must be required by the fidelity runner");
});

/** Deterministic WindowManager sandbox: no DOM, no network, no server. */
function makeWindow(withheld, filteredIds) {
  var state = { fetchCalls: [], opened: [], injected: [] };
  var full = [
    { id: "drive", title: "Explorer", hxGet: "/suite/drive/drive.html" },
    { id: "ghost", title: "Ghost", hxGet: "/suite/ghost/ghost.html" },
  ];
  var filtered = full.filter(function (a) { return filteredIds.indexOf(a.id) !== -1; });
  var win = {
    window: null,
    __GBWindowManager: {
      WindowManager: function WindowManager() {},
      APPS_REGISTRY: full,
      isPreviewId: function () { return false; },
      defineMethods: function (proto, methods) { Object.assign(proto, methods); },
      isAuthWithheldApp: function (id) { return withheld === id; },
    },
    APPS_REGISTRY: filtered,
    GBPreviewMode: { isOn: function () { return false; } },
    gbT: function (key, fallback) { return "LOC:" + key + ":" + fallback; },
    GB_LOGIN_URL: "http://login.test",
    location: { href: "http://suite.test/default" },
  };
  win.document = {
    getElementById: function (id) { return { id: id }; },
    addEventListener: function () {},
    dispatchEvent: function () {},
  };
  win.CustomEvent = function (type, opts) { this.type = type; this.detail = opts.detail; };
  win.fetch = function (url) {
    state.fetchCalls.push(url);
    return { then: function () { return this; }, catch: function () { return this; } };
  };
  var proto = win.__GBWindowManager.WindowManager.prototype;
  proto.openWindows = [];
  proto.closeStartMenu = function () {};
  proto.getWindow = function () { return null; };
  proto.open = function (id, title, url, opts) { state.opened.push({ id: id, title: title, opts: opts }); };
  proto._injectBodyContent = function (id, html) { state.injected.push({ id: id, html: html }); };
  win.window = win;
  win.__state = state;
  return win;
}
function loadManager(win) {
  var ctx = h.vm.createContext(win);
  h.vm.runInContext(readSrc(DEEPLINK), ctx, { filename: DEEPLINK });
  return new win.__GBWindowManager.WindowManager();
}

h.test("#2944 runtime: withheld drive opens a login prompt and fetches nothing", function () {
  var win = makeWindow("drive", []);
  loadManager(win).openDeepLink("drive", {}, {});
  var s = win.__state;
  h.assertEqual(s.fetchCalls.length, 0, "no /suite/partials/<app>.html request may be issued");
  h.assertEqual(s.opened.length, 1, "a window is opened instead of a blank loading shell");
  h.assertEqual(s.opened[0].id, "drive", "the requested app window is shown");
  h.assertEqual(s.opened[0].title, "Explorer", "prompt uses the known registry title");
  h.assertEqual(s.injected.length, 1, "exactly one body injection");
  var html = s.injected[0].html;
  contains(html, "LOC:cloud-appstores-login-first:", "localized login message was rendered");
  contains(html, "LOC:auth-go-to-login:", "localized login action was rendered");
  contains(html, "http://login.test?return_to=", "login link uses the injected login origin");
  contains(html, encodeURIComponent("http://suite.test/default"), "return_to is encoded");
  absent(html, "Application loading...", "no stuck loading placeholder");
});

h.test("#2944 runtime: authenticated deep link still fetches the real drive partial", function () {
  var win = makeWindow(null, ["drive"]);
  loadManager(win).openDeepLink("drive", {}, {});
  var s = win.__state;
  h.assertEqual(s.injected.length, 0, "no auth prompt for an available app");
  h.assertEqual(s.fetchCalls.length, 1, "authenticated deep link still fetches");
  h.assert(s.fetchCalls[0].indexOf("/suite/drive/drive.html?") === 0,
    "real hxGet wins over the missing-partial fallback: " + s.fetchCalls[0]);
  h.assert(s.fetchCalls[0].indexOf("_=") !== -1, "cache buster preserved");
});

h.test("#2944 runtime: an id unknown to the full registry keeps the legacy fallback", function () {
  var win = makeWindow("mystery", []);
  loadManager(win).openDeepLink("mystery", {}, {});
  var s = win.__state;
  h.assertEqual(s.opened.length, 1, "unknown id still opens a window");
  h.assertEqual(s.fetchCalls.length, 1, "unknown id still fetches");
  h.assert(s.fetchCalls[0].indexOf("/suite/partials/mystery.html?") === 0,
    "unknown-id fallback preserved: " + s.fetchCalls[0]);
});

h.note("#2944: offline static + stub-vm proof only — no browser/server/cargo, no locale edit. Driver owns the live Chrome UAT and issue closure.");
