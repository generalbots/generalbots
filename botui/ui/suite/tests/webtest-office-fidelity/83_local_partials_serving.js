/**
 * #2935 criterion 3 (fix half) — offline, deterministic guard that the ten
 * Settings partials are served from botui's own `ui/` tree instead of being
 * proxied to botserver's `BOTUI_PATH`. Guard 77 still pins the route
 * registration and the packaging chain; this guard pins the NEW behaviour at
 * the point where it was lost: `proxy_api` must consult the local static
 * helper *before* it builds a backend URL, while `/api/*` and `/ws` must stay
 * proxied exactly as before.
 *
 * Non-vacuity: every case is a targeted mutation of `local_static.rs` or
 * `proxy.rs` — drop `partials` from the allowlist, move the local call below
 * the backend URL, widen the allowlist to `/api` or `/ws`, drop a rejection
 * case, or remove the runner registration. No browser, server, cargo, network
 * or commit. The driver owns issue closure and host re-UAT after CI deploy.
 */
"use strict";

var h = require("./00_harness");

var UI_ROUTER = "botui/src/ui_server/mod.rs";
var PROXY = "botui/src/ui_server/proxy.rs";
var LOCAL = "botui/src/ui_server/local_static.rs";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var GUARD = "botui/ui/suite/tests/webtest-office-fidelity/83_local_partials_serving.js";

function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}

function notContains(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1, (msg || "source must not contain") + " <" + needle + ">");
}

/** Drop `//` and `//!` comment lines so prose cannot satisfy or fail a code check. */
function codeOnly(src) {
  return src
    .split("\n")
    .filter(function (line) { return line.trim().indexOf("//") !== 0; })
    .join("\n");
}

function countIn(src, needle) {
  return src.split(needle).length - 1;
}

/** Extract the quoted members of `const NAME: &[&str] = &[ ... ];`. */
function arrayMembers(src, name) {
  var marker = "const " + name + ": &[&str] = &[";
  var start = src.indexOf(marker);
  h.assert(start !== -1, "missing allowlist constant <" + name + ">");
  var open = src.indexOf("[", start + marker.length - 1);
  h.assert(open !== -1, "unterminated allowlist <" + name + ">");
  var close = src.indexOf("];", open);
  h.assert(close !== -1, "unterminated allowlist <" + name + ">");
  var body = src.slice(open + 1, close);
  var members = [];
  var at = 0;
  while (true) {
    var q1 = body.indexOf('"', at);
    if (q1 === -1) break;
    var q2 = body.indexOf('"', q1 + 1);
    h.assert(q2 !== -1, "unterminated string in allowlist <" + name + ">");
    members.push(body.slice(q1 + 1, q2));
    at = q2 + 1;
  }
  return members;
}

/** Verbatim body of a named Rust function (brace-balanced from its signature). */
function functionBody(src, signature) {
  var start = src.indexOf(signature);
  h.assert(start !== -1, "missing function <" + signature + ">");
  var open = src.indexOf("{", start);
  h.assert(open !== -1, "missing body for <" + signature + ">");
  var depth = 0;
  for (var i = open; i < src.length; i++) {
    if (src[i] === "{") depth++;
    else if (src[i] === "}") {
      depth--;
      if (depth === 0) return src.slice(open + 1, i);
    }
  }
  h.assert(false, "unbalanced body for <" + signature + ">");
}

// (1) The local helper is wired into the proxy and wins before any backend URL
// is built. This is the single point where the old BOTUI_PATH dependency lived.
h.staticTest("#2935 proxy_api serves local suite assets before building a backend URL", function () {
  var local = h.readRepo(LOCAL);
  var proxy = h.readRepo(PROXY);
  var ui = h.readRepo(UI_ROUTER);

  h.assert(h.fileExistsRepo(LOCAL), "local static helper must ship");
  contains(ui, "pub mod local_static;", "the helper must be a ui_server module");

  var call = "crate::ui_server::local_static::try_serve_local_suite_asset(&uri, &method).await";
  h.assertEqual(countIn(proxy, call), 1, "proxy_api must consult the local helper exactly once");
  var atCall = proxy.indexOf(call);
  var atBackend = proxy.indexOf("state.client.base_url()");
  h.assert(atCall !== -1 && atBackend !== -1 && atCall < atBackend,
    "local serving must precede the proxied backend target");
  h.assert(proxy.indexOf("return local;") > atCall && proxy.indexOf("return local;") < atBackend,
    "a local hit must return immediately, before any proxy request is made");

  // The proxy's own contract is unchanged for everything else.
  contains(proxy, "pub async fn proxy_api(", "proxy_api still exists");
  var apiRouter = functionBody(ui, "fn create_api_router() -> Router<AppState>");
  var wsRouter = functionBody(ui, "fn create_ws_router() -> Router<AppState>");
  contains(apiRouter, ".fallback(any(proxy_api))", "/api/* still falls back to the proxy");
  contains(wsRouter, ".fallback(any(ws_proxy))", "/ws still falls back to the WS proxy");
  contains(ui, '.route("/health", get(health))', "/health route is unchanged");
  contains(ui, '.route("/cloud/login", get(redirect_to_login))', "login redirect is unchanged");
  contains(ui, '.route("/cloud/signup", get(redirect_to_signup))', "signup redirect is unchanged");
  contains(ui, 'for dir in SUITE_DIRS', "suite proxy route registration is unchanged");
  h.assertEqual(countIn(ui, '.route("/suite/:dir/partials/*path", any(proxy_api))'), 2,
    "the historical partial route registrations still exist in both branches");
});

// (2) The allowlist is narrow: only the four static-suffix suite segments,
// never /api, /ws or /cloud. Mutations that widen it are caught here.
h.staticTest("#2935 local serving is restricted to static suite suffixes, never /api or /ws", function () {
  var local = h.readRepo(LOCAL);
  var segments = arrayMembers(local, "LOCAL_SEGMENTS");
  h.assertEqual(segments.length, 4, "exactly the four proxied suite suffixes are eligible");
  h.assert(segments.indexOf("partials") !== -1, "partials must be served locally");
  h.assert(segments.indexOf("fragments") !== -1, "fragments share the proxied route family");
  h.assert(segments.indexOf("modals") !== -1, "modals share the proxied route family");
  h.assert(segments.indexOf("forms") !== -1, "forms share the proxied route family");
  h.assert(segments.indexOf("api") === -1 && segments.indexOf("ws") === -1,
    "API and WS segments must never be served locally");

  var suffixes = arrayMembers(local, "STATIC_SUFFIXES");
  h.assert(suffixes.indexOf(".html") !== -1, "HTML partials must be eligible");
  h.assert(suffixes.indexOf(".js") !== -1 && suffixes.indexOf(".css") !== -1,
    "other shipped suite static assets must be eligible too");

  var gate = functionBody(local, "pub fn is_local_suite_asset(method: &Method, path: &str) -> bool");
  contains(gate, "Method::GET", "only GET requests are eligible");
  contains(gate, "Method::HEAD", "HEAD requests are eligible");
  contains(gate, 'strip_prefix("/suite/")', "the path must be under /suite/");
  // #2471 — the top-level /suite/partials/<file> nest is also served locally;
  // the predicate resolves `kind` from either the second or the first segment,
  // then enforces the same allowlist. See 99_monitoring_partials_route.js.
  contains(gate, "(_, Some(segment)) if LOCAL_SEGMENTS.contains(&segment)",
    "the segment allowlist must be enforced on the nested shape");
  contains(gate, "is_safe_relative_path(rest)", "traversal must be rejected before any file lookup");
  contains(gate, "STATIC_SUFFIXES.iter().any", "a static extension must be required");
});

// (3) Local serving really reads botui's ui/ tree (ServeDir / embedded Assets)
// and never reaches for botserver's BOTUI_PATH or its HTTP client.
h.staticTest("#2935 local helper reads botui/ui directly via ServeDir or embedded Assets", function () {
  var local = h.readRepo(LOCAL);
  contains(local, "ServeDir::new(root)", "dev builds serve through ServeDir");
  contains(local, "service.call(request).await", "ServeDir must actually answer the request");
  contains(local, "serve_from_disk(&get_ui_root(), uri, method)",
    "the local root is botui's UI directory, not a deploy-time env var");
  contains(local, "Assets::get(asset_path)", "release builds serve from the embedded ui/ tree");
  contains(local, "no-cache", "local assets revalidate like the other static routes");

  var code = codeOnly(local);
  notContains(code, "BOTUI_PATH", "local serving must not depend on botserver's static root");
  notContains(code, "base_url", "the local helper must never build a backend URL");
  notContains(code, "reqwest", "the local helper must never make an HTTP request");
  notContains(code, "proxy_api", "the local helper must not re-enter the proxy");
});

// (4) Non-vacuity of the classifier: its own unit tests must explicitly pin
// the proxied surfaces so a future widening of the allowlist is visible.
h.staticTest("#2935 local classifier unit tests pin /api, /ws and /cloud as proxied", function () {
  var local = h.readRepo(LOCAL);
  var body = functionBody(local, "fn api_ws_cloud_and_writes_are_never_locally_served()");
  contains(body, '"/api/chat"', "chat API must stay proxied");
  contains(body, '"/api/files/write"', "write API must stay proxied");
  contains(body, '"/ws/task-progress"', "WS must stay proxied");
  contains(body, '"/cloud/partials/sidebar.html"', "cloud partials must stay proxied on port 4000");
  contains(body, "&Method::POST", "POST must stay proxied");

  var disk = functionBody(local, "async fn disk_serving_reads_the_local_tree_and_misses_fall_through()");
  contains(disk, "std::fs::write", "the disk test must plant a real local partial");
  contains(disk, "is_some()", "the planted partial must be served locally");
  contains(disk, "is_none()", "a local miss must fall through to the backend");
});

// (5) Guard hygiene: registered in the harness and under the file-size cap.
h.staticTest("#2935 local-serving guard is registered in the harness and stays focused", function () {
  var src = h.readRepo(GUARD);
  h.assert(src.length > 4000, "guard must be a non-trivial assertion set");
  h.assert(src.split("\n").length < 450, "guard must stay under 450 lines");
  var runner = h.readRepo(RUNNER);
  contains(runner, 'require("./webtest-office-fidelity/83_local_partials_serving");',
    "the guard must be registered in webtest-office-fidelity.js to run in CI");
});

h.note("#2935: local-serving guard only — partials/fragments/modals/forms static GETs are answered from botui/ui before the backend proxy; /api, /ws, /cloud/partials, POSTs and dynamic fragments stay proxied. Host re-UAT requires CI deployment of the new botui binary first.");
