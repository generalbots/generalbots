/**
 * #2961 — offline guard for the Monitoring app's HTMX partial chain.
 *
 * The live sweep reported `GET /suite/monitoring/partials/01_live_view.html`
 * -> 404 on an authenticated admin open. The partial itself ships in this tree
 * and is served locally by the #2935 `local_static` short-circuit; the 404 is
 * therefore a deployment/static-root failure, not a missing source file.
 * This guard pins the three things that decide whether those URLs are 200:
 *
 *   (a) every partial URL referenced by the Monitoring HTML surface resolves
 *       to a shipped, non-trivial file (including the 01 -> 02 chain);
 *   (b) no stale/renamed URL or `?v=` cache-buster is introduced on a partial;
 *   (c) the local-first serving chain (botui `local_static` -> proxy route) is
 *       present, so a current build answers the partial from `ui/` instead of
 *       depending on the backend's BOTUI_PATH;
 *
 * Each case fails on a targeted mutation: delete/rename a partial, add a query
 * string, break the 01->02 chain, or remove the local-serving short-circuit.
 * No browser, server, cargo, network or commit. The driver owns host evidence.
 */
"use strict";

var h = require("./00_harness");

var BOOT = "botui/ui/suite/monitoring/monitoring.html";
var MON_DIR = "botui/ui/suite/monitoring";
var PARTIAL_DIR = "botui/ui/suite/monitoring/partials/";
var PARTIAL_PREFIX = "/suite/monitoring/partials/";
var BOOT_REF = PARTIAL_PREFIX + "01_live_view.html";
var CHAIN_REF = PARTIAL_PREFIX + "02_grid_view.html";
var LOCAL = "botui/src/ui_server/local_static.rs";
var PROXY = "botui/src/ui_server/proxy.rs";
var ROUTER = "botui/src/ui_server/mod.rs";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var GUARD = "botui/ui/suite/tests/webtest-office-fidelity/99_monitoring_partials_route.js";

function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + ">");
}
function countIn(src, needle) { return src.split(needle).length - 1; }

/** Extract every absolute Monitoring partial hx-get, keeping any query string. */
function partialRefs(src) {
  var marker = 'hx-get="' + PARTIAL_PREFIX;
  var refs = [];
  var at = 0;
  while (true) {
    var start = src.indexOf(marker, at);
    if (start === -1) break;
    var end = src.indexOf('"', start + marker.length);
    h.assert(end !== -1, "unterminated partial hx-get at offset " + start);
    refs.push(PARTIAL_PREFIX + src.slice(start + marker.length, end));
    at = end + 1;
  }
  return refs;
}

/** Recursively collect .html files under the Monitoring surface. */
function htmlFiles(dir, out) {
  h.fs.readdirSync(dir).forEach(function (name) {
    var full = h.path.join(dir, name);
    if (h.fs.statSync(full).isDirectory()) htmlFiles(full, out);
    else if (name.slice(-5) === ".html") out.push(full);
  });
  return out;
}

// (a) The 404'd partial and its chained successor ship as non-trivial HTML,
// and the boot document issues exactly the expected absolute URL.
h.staticTest("#2961 Monitoring partials 01_live_view and 02_grid_view ship as non-trivial HTML", function () {
  [BOOT_REF, CHAIN_REF].forEach(function (url) {
    var rel = PARTIAL_DIR + url.slice(PARTIAL_PREFIX.length);
    h.assert(h.fileExistsRepo(rel), "missing shipped partial <" + rel + ">");
    var src = h.readRepo(rel);
    h.assert(src.length > 500, rel + " must be a non-trivial HTML fragment");
    h.assert(src.indexOf("<") !== -1 && src.indexOf(">") !== -1,
      rel + " must contain HTML markup");
  });

  var bootRefs = partialRefs(h.readRepo(BOOT));
  h.assertEqual(bootRefs.length, 1,
    "the Monitoring boot document must issue exactly one partial request");
  h.assertEqual(bootRefs[0], BOOT_REF,
    "the boot partial URL must not be renamed or stale");

  // The chain is what renders the grid view after the live view swaps in.
  var live = h.readRepo(PARTIAL_DIR + "01_live_view.html");
  h.assert(partialRefs(live).indexOf(CHAIN_REF) !== -1,
    "01_live_view.html must chain to 02_grid_view.html");
});

// (b) Every partial reference in the whole Monitoring surface resolves to a
// shipped file, with no query string and no path escaping the partial dir.
h.staticTest("#2961 every Monitoring partial reference resolves; no stale URL or ?v= query", function () {
  var files = htmlFiles(h.path.join(h.REPO_DIR, MON_DIR), []);
  h.assertGreater(files.length, 2, "the Monitoring surface must hold its HTML files");
  var refs = [];
  files.forEach(function (file) {
    var src = h.fs.readFileSync(file, "utf8");
    partialRefs(src).forEach(function (url) {
      refs.push(url);
      h.assert(url.indexOf("?") === -1,
        "no query string on <" + url + "> in " + h.path.relative(h.REPO_DIR, file));
      var name = url.slice(PARTIAL_PREFIX.length);
      h.assert(name.indexOf("/") === -1 && name.length > 0,
        "partial URL must name a file directly under the partial dir: <" + url + ">");
      h.assert(h.fileExistsRepo(PARTIAL_DIR + name),
        "unresolved partial <" + url + "> referenced by " + h.path.relative(h.REPO_DIR, file));
    });
  });
  h.assert(refs.indexOf(BOOT_REF) !== -1 && refs.indexOf(CHAIN_REF) !== -1,
    "the boot and chained partials must both be referenced somewhere in the surface");
});

// (c) A current build serves those partials from botui's own ui/ tree, so a
// stale backend BOTUI_PATH cannot turn them into 404s (#2935 mechanism).
h.staticTest("#2961 Monitoring partials are covered by the local-first serving chain", function () {
  var local = h.readRepo(LOCAL);
  contains(local, '"partials"', 'LOCAL_SEGMENTS must include the partials suffix');
  contains(local, '".html"', 'STATIC_SUFFIXES must include .html');
  contains(local, "(_, Some(segment)) if LOCAL_SEGMENTS.contains(&segment)",
    "the /suite/<dir>/partials/<file> shape must be accepted locally");
  contains(local, "pub fn is_local_suite_asset(", "the local-asset predicate must ship");
  contains(local, "pub async fn try_serve_local_suite_asset(", "the local-serving entry point must ship");

  var proxy = h.readRepo(PROXY);
  var call = "crate::ui_server::local_static::try_serve_local_suite_asset(&uri, &method).await";
  h.assertEqual(countIn(proxy, call), 1, "proxy_api must consult the local helper once");
  h.assert(proxy.indexOf("return local;") > proxy.indexOf(call),
    "a local hit must be returned before the backend proxy");

  var router = h.readRepo(ROUTER);
  h.assertEqual(countIn(router, '.route("/suite/:dir/partials/*path", any(proxy_api))'), 2,
    "the generic partials proxy route must exist in both embed and non-embed branches");
});

// (d) Non-vacuity: registered, substantial, scoped to #2961, under 450 lines.
h.staticTest("#2961 guard is wired into the harness and stays a small focused static check", function () {
  var src = h.readRepo(GUARD);
  h.assert(src.length > 2500, "guard must be a non-trivial assertion set");
  h.assert(src.split("\n").length < 450, "guard must stay under 450 lines");
  var runner = h.readRepo(RUNNER);
  contains(runner, 'require("./webtest-office-fidelity/99_monitoring_partials_route");',
    "the guard must be registered in webtest-office-fidelity.js to run in CI");
});

h.note("#2961: static guard only — the 404'd partial ships and is locally served by current code; a stale deployed botui/static root is the host-side cause. No browser, server, cargo or host evidence asserted here. A host-reachable runner must confirm GET /suite/monitoring/partials/01_live_view.html is 200 after the current botui binary and suite tree are deployed.");
