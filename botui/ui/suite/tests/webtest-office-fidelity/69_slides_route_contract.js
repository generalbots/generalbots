/**
 * #2922 — deterministic static guard for the four Slides paths that 404'd in
 * production: `GET /api/slides/media/list`, `POST /api/slides/slide/add`,
 * `POST /api/slides/presenter/start` and `/ws/slides/:presentation_id`.
 *
 * Two halves, both literal source assertions:
 *   (1) the assembled router in botslides declares each path with the right
 *       HTTP method and handler, is installed behind the `slides` feature and
 *       is covered by an offline route-contract test that proves a real 101;
 *   (2) every shipped client caller uses that exact path, so a client/server
 *       path drift cannot ship unnoticed.
 *
 * This guard cannot prove what binary a remote host serves — that is browser
 * UAT and the driver's job. It proves the source contract the built binary is
 * produced from. No server, browser, cargo or network call is involved.
 */
"use strict";

var h = require("./00_harness");

var ROUTES = "botserver/crates/botslides/src/routes.rs";
var SERVER_LAYERS = "botserver/src/main_module/server/server_layers.rs";
var RUN_AXUM = "botserver/src/main_module/server/run_axum_server.rs";
var CONTRACT_TEST = "botserver/crates/botslides/src/route_contract_tests.rs";
var LIB = "botserver/crates/botslides/src/lib.rs";
var COLLAB = "botui/ui/suite/js/collab.js";

function countIn(src, needle) {
  return src.split(needle).length - 1;
}

function compact(src) {
  return src.replace(/\s+/g, "");
}

function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + ">");
}

function absent(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1,
    (msg || "source must not contain") + " <" + needle + ">");
}

/** A path must be declared once and immediately wired to method(handler). */
function route(flat, path, methodAndHandler) {
  var needle = '"' + path + '"';
  var at = flat.indexOf(needle);
  h.assert(at !== -1, "router must declare <" + path + ">");
  h.assertEqual(countIn(flat, needle), 1, "<" + path + "> must be declared exactly once");
  h.assert(flat.slice(at, at + 160).indexOf(methodAndHandler) !== -1,
    "<" + path + "> must be wired to <" + methodAndHandler + "> :: " + flat.slice(at, at + 160));
}

h.staticTest("#2922 the four required Slides paths are registered with the documented methods", function () {
  var src = h.readRepo(ROUTES);
  h.assert(src.length > 4000, "routes.rs must be the non-trivial shipped router");
  var flat = compact(src);

  // Non-vacuity: this is the router builder, with many other sibling routes.
  contains(flat, "pubfnconfigure_slides_routes", "the router builder must exist");
  h.assertGreater(countIn(flat, ".route("), 30,
    "the builder must contain many sibling routes, not only the four asserted ones");

  route(flat, "/api/slides/slide/add", "post(handle_add_slide::<D>)");
  route(flat, "/api/slides/media/list", "get(handle_list_media::<D>)");
  route(flat, "/api/slides/presenter/start", "post(handle_start_presenter::<D>)");
  route(flat, "/ws/slides/:presentation_id", "get(handle_slides_websocket::<D>)");

  // The router runs on axum 0.7: `{name}` would be a literal segment (404).
  absent(flat, '"/ws/slides/{presentation_id}"', "axum 0.8 spelling must not be used");
  absent(flat, '"/api/slides/{id}"', "axum 0.8 spelling must not be used");

  // Source declarations are worthless if the served router never installs them.
  var layers = compact(h.readRepo(SERVER_LAYERS));
  contains(layers, '#[cfg(feature="slides")]', " Slides routes are feature gated");
  contains(layers, "pub(super)fnadd_slides_routes", "add_slides_routes must exist");
  contains(layers, "crate::slides::routes::configure_slides_routes()",
    "the host router must merge the botslides router");
  contains(layers, "slides_user_middleware", "sibling authorization middleware is preserved");
  contains(layers, ".with_state(state)", "the merged router must carry its state");
  contains(compact(h.readRepo(RUN_AXUM)), "add_slides_routes(base_router,&app_state)",
    "run_axum_server must install the Slides router");
});

h.staticTest("#2922 every shipped caller uses the exact registered path", function () {
  var callers = [
    ["botui/ui/suite/slides/modules/13_delta.js", 'fetch("/api/slides/slide/add"'],
    ["botui/ui/suite/slides/modules/13_insert_place.js", 'this.post("/api/slides/slide/add"'],
    ["botui/ui/suite/slides/modules/44_stm_media_controls.js", '"/api/slides/media/list?presentation_id="'],
    ["botserver/crates/botslides/src/ui_fragments/sidebars.rs", 'hx-post="/api/slides/presenter/start"'],
    ["botserver/crates/botslides/src/ui_fragments/panels.rs", 'hx-post="/api/slides/presenter/start"']
  ];

  callers.forEach(function (entry) {
    var src = h.readRepo(entry[0]);
    h.assert(src.length > 200, "non-vacuity: non-trivial caller " + entry[0]);
    contains(src, entry[1], entry[0] + " must call the registered path");
    // No trailing-slash or singular variant may creep back in.
    absent(src, "/api/slides/slide/add/", entry[0] + " stale slide/add path");
    absent(src, "/api/slides/media/list/", entry[0] + " stale media/list path");
    absent(src, "/api/slides/presenter/start/", entry[0] + " stale presenter/start path");
  });

  // GBCollab builds the WS URL from the app name; `slides` must opt into it,
  // otherwise the client falls back to REST presence and never upgrades.
  var collab = h.readRepo(COLLAB);
  h.assert(collab.length > 12000, "non-vacuity: collab.js is the shipped client");
  contains(collab, 'return proto + "//" + loc.host + "/ws/" + app + "/" + encodeURIComponent(docId);',
    "the default WS URL must be /ws/<app>/<docId>");
  contains(collab, "slides: true", "slides must use the realtime WS route");
  absent(collab, '"/ws/slide/"', "no singular /ws/slide/ path");
});

h.staticTest("#2922 an offline contract test proves 101 and fails on a missing route", function () {
  h.assert(h.fileExistsRepo(CONTRACT_TEST), "the offline route-contract test must ship");
  var src = h.readRepo(CONTRACT_TEST);
  h.assert(src.length > 4000, "non-vacuity: the contract test is non-trivial");

  ["/api/slides/media/list", "/api/slides/slide/add", "/api/slides/presenter/start",
    "/ws/slides/", "handshake(", "HTTP/1.1 101", "NOT_FOUND",
    "configure_slides_routes", "verified_ws_token"
  ].forEach(function (needle) {
    contains(src, needle, "route contract test must pin <" + needle + ">");
  });

  var lib = compact(h.readRepo(LIB));
  contains(lib, '#[path="route_contract_tests.rs"]', "the contract test must be registered");
  contains(lib, "modroute_contract_tests;", "the contract test module must build");
});

h.staticTest("#2922 affected Slides handlers do not leak raw Drive errors", function () {
  var handlers = [
    "botserver/crates/botslides/src/handlers/media.rs",
    "botserver/crates/botslides/src/handlers/presenter.rs",
    "botserver/crates/botslides/src/handlers/slides/mod.rs"
  ];
  handlers.forEach(function (rel) {
    var src = h.readRepo(rel);
    h.assert(src.length > 500, "non-vacuity: non-trivial handler " + rel);
    absent(src, '"error": e', rel + " must not return a raw error string");
    contains(src, "log_and_sanitize_json", rel + " must use the shared sanitizer");
  });
});
