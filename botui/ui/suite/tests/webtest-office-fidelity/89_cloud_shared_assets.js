/**
 * #2942 — deterministic static guard for the three shared assets that UIs
 * request from the cloud origin (port 4000): the palette CSS, the mandatory
 * DESIGN-SYSTEM identity tokens and the PWA manifest. Both ship in `botui/ui/` and the suite router (:3000) already
 * serves them, but the cloud router had no static route. Both requests fell
 * through to `serve_cloud_fallback`, which returned a 404 with an empty
 * Content-Type; with the global `nosniff` header Chrome refused the stylesheet
 * ("strict MIME checking is enabled") and could not parse the manifest.
 *
 * This guard pins both halves of the fix at the source level:
 *   (1) the cloud router registers three LITERAL routes, without adding broad
 *       static nests or weakening the login/signup redirects;
 *   (2) the MIME types are pinned in an allowlist and consumed by a handler
 *       that reads botui/ui (disk or embedded Assets), never the proxy.
 *
 * Non-vacuity: mutating either the MIME map or the manifest route in
 * `cloud.rs`/`mod.rs` makes a case below FAIL (see the /tmp mutation probe the
 * driver runs). No server, browser, cargo or i18n change is involved; the
 * driver owns live curl UAT, issue closure and CI deployment.
 */
"use strict";

var h = require("./00_harness");

var CLOUD = "botui/src/ui_server/cloud.rs";
var UI_ROUTER = "botui/src/ui_server/mod.rs";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var GUARD = "botui/ui/suite/tests/webtest-office-fidelity/89_cloud_shared_assets.js";

function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}

function absent(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1, (msg || "source must not contain") + " <" + needle + ">");
}

/** Ignore formatting so a rustfmt re-wrap cannot break a route assertion. */
function compact(src) {
  return src.replace(/\s+/g, "");
}

function countIn(src, needle) {
  return src.split(needle).length - 1;
}

/** Verbatim brace-balanced body of a named Rust fn/impl, from its signature. */
function functionBody(src, signature) {
  var start = src.indexOf(signature);
  h.assert(start !== -1, "missing signature <" + signature + ">");
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

h.staticTest("#2942 cloud router registers palette CSS, identity tokens and PWA manifest as literal routes", function () {
  var ui = h.readRepo(UI_ROUTER);
  h.assert(ui.length > 8000, "non-vacuity: mod.rs is the shipped router module, not a stub");
  var body = compact(functionBody(ui, "pub fn configure_cloud_router() -> Router"));

  // The three exact routes. These must survive rustfmt and are the manifest-route
  // half of the mutation probe.
  contains(body, '.route("/suite/css/palette-tokens.css",get(serve_cloud_shared_asset)',
    "the palette CSS must be a literal cloud route");
  contains(body, '.route("/suite/css/identity-tokens.css",get(serve_cloud_shared_asset)',
    "the DESIGN-SYSTEM identity tokens must be a literal cloud route");
  contains(body, '.route("/manifest.webmanifest",get(serve_cloud_shared_asset)',
    "the PWA manifest must be a literal cloud route");
  h.assertEqual(countIn(body, "get(serve_cloud_shared_asset)"), 3,
    "exactly three shared-asset routes may be registered on the cloud origin");

  // No route weakening: login/signup stay redirect-only and the cloud-page
  // fallback that serves static cloud HTML/JS/CSS is untouched.
  contains(body, '.route("/cloud/login",get(redirect_to_login))', "login redirect must be unchanged");
  contains(body, '.route("/cloud/signup",get(redirect_to_signup))', "signup redirect must be unchanged");
  contains(body, '.fallback(get(serve_cloud_fallback))', "cloud page fallback must be unchanged");
  contains(body, '.route("/cloud/*path",get(serve_cloud))', "legacy cloud asset route must be unchanged");
  absent(body, '.route("/login"', "login must never be served locally on port 4000");
  absent(body, '.route("/signup"', "signup must never be served locally on port 4000");

  // No broad static serving was smuggled in alongside the two literal routes.
  absent(body, "nest_service", "cloud router must not gain broad ServeDir nests");
  absent(body, "ServeDir", "cloud router must not gain a wildcard static service");
  h.assertEqual(countIn(body, '.route("/suite/css/'), 2,
    "only the two pinned token CSS paths may be /suite/ routes on port 4000");
});

h.staticTest("#2942 MIME types are pinned in an allowlist and consumed by the handler", function () {
  var src = h.readRepo(CLOUD);
  h.assert(src.length > 10000, "non-vacuity: cloud.rs is the shipped cloud handler module");

  // The MIME-map half of the mutation probe: both exact tuple lines.
  contains(src, 'const CLOUD_SHARED_ASSETS: &[(&str, &str)] = &[',
    "the shared-asset allowlist must exist");
  contains(src, '("suite/css/palette-tokens.css", "text/css"),',
    "palette CSS must be pinned to text/css");
  contains(src, '("suite/css/identity-tokens.css", "text/css"),',
    "identity tokens must be pinned to text/css");
  contains(src, '("manifest.webmanifest", "application/manifest+json"),',
    "the manifest must be pinned to application/manifest+json");

  var mimes = functionBody(src, "fn cloud_shared_asset_mime(path: &str) -> Option<&'static str>");
  contains(mimes, "CLOUD_SHARED_ASSETS", "the resolver must read the pinned allowlist");
  contains(mimes, ".find(|(asset, _)| *asset == path)", "lookup must be an exact allowlist match");
  contains(mimes, ".map(|(_, mime)| *mime)", "the pinned MIME must be returned");
  absent(mimes, "mime_guess", "MIME must be pinned, never inferred from the extension");

  var sigAt = src.indexOf("pub async fn serve_cloud_shared_asset(");
  h.assert(sigAt !== -1, "the shared-asset handler must exist");
  var decl = src.slice(sigAt, src.indexOf("{", sigAt));
  var handler = functionBody(src, "pub async fn serve_cloud_shared_asset(");
  contains(decl, "axum::extract::OriginalUri(uri): axum::extract::OriginalUri",
    "literal routes have no Path param; the filename comes from OriginalUri");
  contains(handler, "cloud_shared_asset_mime(filename)", "the handler must use the pinned mapping");
  contains(handler, "axum::http::header::CONTENT_TYPE, mime",
    "the pinned MIME must be sent as Content-Type");
  contains(handler, "StatusCode::NOT_FOUND", "an unknown path must 404, not be file-served");
  contains(handler, "get_ui_root()", "dev builds must read botui's own ui/ tree");
  contains(handler, "resolve_static_path(&root, &root.join(filename))",
    "the local read must be containment-checked");
  contains(handler, "Assets::get(filename)", "embedded builds must serve the embedded ui tree");

  var code = compact(handler);
  absent(code, "mime_guess", "the handler must not guess MIME types");
  absent(code, "base_url", "the handler must never build a backend URL");
  absent(code, "BOTUI_PATH", "the handler must not depend on botserver's static root");
  absent(code, "reqwest", "the handler must never make an HTTP request");
  absent(code, "cloud_root", "the handler must not resolve against the cloud/ directory");

  var unit = functionBody(src, "fn cloud_shared_assets_pin_their_content_types()");
  contains(unit, 'Some("text/css")', "unit test must pin the CSS MIME");
  contains(unit, 'Some("application/manifest+json")', "unit test must pin the manifest MIME");
});

h.staticTest("#2942 all pinned assets exist in the local ui tree", function () {
  var css = "botui/ui/suite/css/palette-tokens.css";
  var identity = "botui/ui/suite/css/identity-tokens.css";
  var manifest = "botui/ui/manifest.webmanifest";
  h.assert(h.fileExistsRepo(css), "palette-tokens.css must ship in botui/ui/suite/css");
  h.assert(h.fileExistsRepo(identity), "identity-tokens.css must ship in botui/ui/suite/css");
  h.assert(h.fileExistsRepo(manifest), "manifest.webmanifest must ship at the ui root");
  h.assert(h.readRepo(css).length > 200, "non-vacuity: the CSS is a real theme file");
  h.assert(h.readRepo(identity).indexOf("--id-paper:") !== -1,
    "non-vacuity: the identity CSS defines the design-system tokens");
  var mf = h.readRepo(manifest);
  h.assert(mf.indexOf('"name"') !== -1 && mf.indexOf('"icons"') !== -1,
    "non-vacuity: the manifest is a real PWA document");
});

h.staticTest("#2942 shared-assets guard is registered in the harness and stays focused", function () {
  var src = h.readRepo(GUARD);
  h.assert(src.length > 4500, "guard must be a non-trivial assertion set");
  h.assert(src.split("\n").length < 450, "guard must stay under 450 lines");
  contains(h.readRepo(RUNNER), 'require("./webtest-office-fidelity/89_cloud_shared_assets");',
    "the guard must be registered in webtest-office-fidelity.js to run in CI");
});

h.note("#2942: cloud origin (:4000) now serves the palette CSS and DESIGN-SYSTEM identity tokens as text/css and the PWA manifest as application/manifest+json from botui/ui, with login/signup still redirecting to :5000; live curl UAT belongs to the driver.");
