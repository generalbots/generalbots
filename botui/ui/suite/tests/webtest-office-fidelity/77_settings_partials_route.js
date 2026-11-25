/**
 * #2935 criterion 3 — offline, deterministic coverage for the Settings window's
 * ten HTMX partial requests. Host evidence from pragmatismo.generalbots.org is
 * unobtainable on this box (DNS/connect failure), so the guard pins the source
 * chain that decides whether those ten URLs are 200 or 404 in a deployment:
 *
 *   (a) all ten partial files named in the issue ship as non-trivial HTML;
 *   (b) the Settings boot document (settings/index.html, loaded through the
 *       deep link /suite/settings/index.html) references EXACTLY those ten
 *       absolute paths — no stale/renamed URL and no `?v=` query on a partial;
 *   (c) /suite/:dir/partials/*path is proxied by botui to botserver, which
 *       serves /suite from BOTUI_PATH with ServeDir — a stale or unset
 *       BOTUI_PATH on the host is therefore the documented 404 root cause;
 *   (d) release packaging copies the WHOLE suite tree (Linux + Windows), so a
 *       partial cannot be dropped from the installer;
 *   (e) the guard is registered in the harness and stays under 450 lines.
 *
 * Each case fails on a targeted mutation: delete a partial, rename a boot URL,
 * remove the proxy route, or remove a packaging copy. No browser, server,
 * cargo, network or commit. The driver owns issue closure and host evidence.
 */
"use strict";

var h = require("./00_harness");

var INDEX = "botui/ui/suite/settings/index.html";
var REGISTRY = "botui/ui/suite/js/window-manager/01_registry.js";
var SETTINGS_DIR = "botui/ui/suite/settings";
var PARTIAL_DIR = "botui/ui/suite/settings/partials/";
var UI_ROUTER = "botui/src/ui_server/mod.rs";
var PROXY = "botui/src/ui_server/proxy.rs";
var AXUM = "botserver/src/main_module/server/run_axum_server.rs";
var LINUX_PKG = "desktop/linux/package.sh";
var RELEASE = ".github/workflows/release.yml";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var GUARD = "botui/ui/suite/tests/webtest-office-fidelity/77_settings_partials_route.js";

// The exact ten URLs from issue #2935 (browser console on the host).
var TEN = [
  "01_profile_security.html",
  "06_dialogs.html",
  "07_os.html",
  "08_shared_links.html",
  "10_system.html",
  "12_model.html",
  "13_instructions.html",
  "14_shortcuts.html",
  "15_os_appliance.html",
  "16_protection.html",
];

function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}

function countIn(src, needle) {
  return src.split(needle).length - 1;
}

function lineContaining(src, needle, msg) {
  var lines = src.split("\n");
  for (var i = 0; i < lines.length; i++) {
    if (lines[i].indexOf(needle) !== -1) return lines[i];
  }
  h.assert(false, (msg || "missing line with") + " <" + needle + ">");
}

/** Extract every hx-get partial URL without a regex (captures any query too). */
function partialRefs(src) {
  var marker = 'hx-get="/suite/settings/partials/';
  var refs = [];
  var at = 0;
  while (true) {
    var start = src.indexOf(marker, at);
    if (start === -1) break;
    var end = src.indexOf('"', start + marker.length);
    h.assert(end !== -1, "unterminated partial hx-get at offset " + start);
    refs.push("/suite/settings/partials/" + src.slice(start + marker.length, end));
    at = end + 1;
  }
  return refs;
}

/** Recursively collect .html files under the settings surface. */
function htmlFiles(dir, out) {
  h.fs.readdirSync(dir).forEach(function (name) {
    var full = h.path.join(dir, name);
    if (h.fs.statSync(full).isDirectory()) htmlFiles(full, out);
    else if (name.slice(-5) === ".html") out.push(full);
  });
  return out;
}

// (a) The ten files named in #2935 must exist as non-trivial HTML documents.
h.staticTest("#2935 all ten Settings partials from the host console ship as non-trivial HTML files", function () {
  h.assertEqual(TEN.length, 10, "the issue names exactly ten partials");
  TEN.forEach(function (name) {
    var rel = PARTIAL_DIR + name;
    h.assert(h.fileExistsRepo(rel), "missing shipped partial <" + rel + ">");
    var src = h.readRepo(rel);
    h.assert(src.length > 500, rel + " must be a non-trivial HTML fragment");
    h.assert(src.indexOf("<") !== -1 && src.indexOf(">") !== -1,
      rel + " must contain HTML markup");
  });
  var all = h.fs.readdirSync(h.path.join(h.REPO_DIR, PARTIAL_DIR))
    .filter(function (name) { return name.slice(-5) === ".html"; });
  h.assert(all.length >= TEN.length, "the partial directory must hold at least the ten issue files");
});

// (b) The boot document references exactly those ten clean absolute paths.
h.staticTest("#2935 Settings boot document references exactly the ten partial URLs, no stale path and no ?v= query", function () {
  var index = h.readRepo(INDEX);
  var refs = partialRefs(index);
  h.assertEqual(refs.length, 10, "the boot document must issue exactly ten partial requests");
  var unique = refs.filter(function (url, i) { return refs.indexOf(url) === i; });
  h.assertEqual(unique.length, 10, "each partial must be requested once");
  var expected = TEN.map(function (name) { return "/suite/settings/partials/" + name; });
  expected.forEach(function (url) {
    h.assert(refs.indexOf(url) !== -1, "boot document lost partial <" + url + ">");
  });
  refs.forEach(function (url) {
    h.assert(expected.indexOf(url) !== -1, "stale or renamed boot partial <" + url + ">");
    h.assert(url.indexOf("?") === -1, "no ?v= cache-buster on a partial URL: <" + url + ">");
  });

  // The deep link that opens this boot document is the Settings app entry.
  var registry = h.readRepo(REGISTRY);
  var entry = lineContaining(registry, 'id: "settings"', "settings app registry entry");
  h.assert(entry.indexOf('hxGet: "/suite/settings/index.html"') !== -1,
    "settings deep link must fetch the boot document, not a renamed page");

  // Nested partials are not boot requests, but every reference in the whole
  // Settings surface must still resolve (catches a stale nested chain).
  var files = htmlFiles(h.path.join(h.REPO_DIR, SETTINGS_DIR), []);
  h.assert(files.length > TEN.length, "nested Settings partials must be present");
  files.forEach(function (file) {
    var src = h.fs.readFileSync(file, "utf8");
    partialRefs(src).forEach(function (url) {
      h.assert(url.indexOf("?") === -1, "no query string on <" + url + "> in " + file);
      var name = url.slice("/suite/settings/partials/".length);
      h.assert(h.fs.existsSync(h.path.join(h.REPO_DIR, PARTIAL_DIR, name)),
        "unresolved partial <" + url + "> referenced by " + h.path.relative(h.REPO_DIR, file));
    });
  });
});

// (c) The static-serving chain: botui proxy -> botserver ServeDir(BOTUI_PATH).
h.staticTest("#2935 partials proxy from botui to botserver, which serves /suite from BOTUI_PATH", function () {
  var ui = h.readRepo(UI_ROUTER);
  var genericPartialRoute = '.route("/suite/:dir/partials/*path", any(proxy_api))';
  h.assertEqual(countIn(ui, genericPartialRoute), 2,
    "the generic partials proxy route must exist in both embed and non-embed branches");
  contains(ui, 'for dir in SUITE_DIRS', "static suite dirs are enumerated for explicit proxy routes");
  contains(ui, '.route(&format!("/suite/{dir}/partials/*path"), any(proxy_api))',
    "the SUITE_DIRS-specific partials proxy route is registered");
  contains(ui, "matchit ranks static segments", "the proxy-vs-ServeDir precedence is documented in source");

  var proxy = h.readRepo(PROXY);
  contains(proxy, "pub async fn proxy_api(", "botui forwards requests to the backend");
  contains(proxy, "state.client.base_url()", "the proxy target is the botserver base URL");

  var axum = h.readRepo(AXUM);
  contains(axum, "pub(super) fn build_base_router(", "the botserver static router builder ships");
  contains(axum, 'std::env::var("BOTUI_PATH")', "effective static root comes from BOTUI_PATH");
  contains(axum, "let ui_path_exists", "static serving is conditional on that path existing");
  contains(axum, '.nest_service("/suite", ServeDir::new(&ui_path))',
    "the proxied /suite/* request is served by ServeDir from BOTUI_PATH");
  contains(axum, "Serving UI from external folder", "the effective root is logged for diagnosis");
  h.assert(axum.indexOf("let ui_path_exists") < axum.indexOf('.nest_service("/suite", ServeDir::new(&ui_path))'),
    "the existence check must gate the ServeDir mount");
});

// (d) Packaging copies the whole suite tree on both release platforms.
h.staticTest("#2935 Linux and Windows release packaging copy the whole suite tree; CI sets BOTUI_PATH", function () {
  var pkg = h.readRepo(LINUX_PKG);
  var linuxLine = lineContaining(pkg, '"$ROOT/botui/ui/suite"', "Linux suite copy line");
  h.assert(linuxLine.indexOf("cp -a") !== -1 && linuxLine.indexOf('"$DEST/ui/suite"') !== -1,
    "Linux packaging must recursively copy the entire suite tree: <" + linuxLine.trim() + ">");

  var release = h.readRepo(RELEASE);
  var linux = lineContaining(release, "cp -a botui/ui/suite  dist/payload/ui/suite", "Linux release suite copy");
  h.assert(linux.indexOf("dist/payload/ui/suite") !== -1,
    "the Linux payload must contain the full suite tree");
  var windows = lineContaining(release, "Copy-Item -Recurse botui/ui/suite  dist/payload/ui/suite",
    "Windows release suite copy");
  h.assert(windows.indexOf("-Recurse") !== -1,
    "the Windows payload must recursively copy the full suite tree");
  contains(release, 'BOTUI_PATH="$PWD/ui/suite"',
    "the CI smoke test starts botserver against the packaged suite root");
});

// (e) Non-vacuity: registered, substantial, scoped to #2935.
h.staticTest("#2935 guard is wired into the harness and stays a small focused static check", function () {
  var src = h.readRepo(GUARD);
  h.assert(src.length > 4000, "guard must be a non-trivial assertion set");
  h.assert(src.split("\n").length < 450, "guard must stay under 450 lines");
  var runner = h.readRepo(RUNNER);
  contains(runner, 'require("./webtest-office-fidelity/77_settings_partials_route");',
    "the guard must be registered in webtest-office-fidelity.js to run in CI");
});

h.note("#2935: static guard only — ten partial files, exact boot URLs, botui proxy -> botserver ServeDir(BOTUI_PATH), whole-tree packaging. Host pragmatismo.generalbots.org unreachable here; no host evidence asserted. A host-reachable runner must run the live browser smoke test (zero 404s for /suite/settings/partials/*) and check the effective BOTUI_PATH.");
