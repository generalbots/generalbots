/**
 * [STATIC] deterministic guards for the #2452 read-side access audit.
 * Every case reads real shipped Rust source as text and FAILS if a contract is
 * removed: no runtime claim, no server, no browser, no cargo.
 *
 * Covered contracts:
 *   (a) a successful shared-link read appends an `anonymous-guest` `view` event
 *       with resource_type = docs and a content-free payload (no token/body)
 *   (b) every mounted export format handler appends an `export` event carrying
 *       only the verified owner identity and the format — nothing content-bearing
 *   (c) both hooks are placed AFTER a successful load/render (fail-closed), the
 *       module is compiled into the crate, and both unit tests exist
 *
 * All assertions are literal source substrings. Removing a hook, its identity
 * fields, the fail-closed ordering or a regression test makes the matching case
 * FAIL. No user-facing string is introduced here.
 */
"use strict";

var h = require("./00_harness");

var ACCESS_AUDIT = "botserver/crates/botdocs/src/access_audit.rs";
var SHARED_LINK = "botserver/crates/botdocs/src/shared_link.rs";
var EXPORT_HANDLERS = "botserver/crates/botdocs/src/handlers_api/export.rs";
var LIB = "botserver/crates/botdocs/src/lib.rs";

var SHARED_HOOK = "crate::access_audit::record_shared_view(&state, &link.resource_id);";
var EXPORT_FN = "pub fn export(actor_id: &str, actor_name: &str, resource_id: &str, format: &str) -> Self {";
var VIEW_FN = "pub fn shared_view(resource_id: &str) -> Self {";

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  h.assert(
    h.readRepo(relPath).indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath
  );
}

/** Return the exact source span between two markers (exclusive of the end). */
function between(relPath, startMarker, endMarker) {
  var src = h.readRepo(relPath);
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + "> in " + relPath);
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + "> in " + relPath);
  var span = src.slice(start, end);
  h.assert(span.length > 0, "empty source span between <" + startMarker + "> and <" + endMarker + ">");
  return span;
}

/** The six mounted export format handlers, in source order. */
var FORMATS = ["pdf", "docx", "odt", "md", "html", "txt"];

/** Slice of export.rs owned by one handler (up to the next handler, or EOF). */
function handlerSpan(src, index) {
  var startMarkers = FORMATS.map(function (fmt) {
    return "pub async fn handle_export_" + fmt + "(";
  });
  var positions = startMarkers.map(function (marker) {
    var pos = src.indexOf(marker);
    h.assert(pos !== -1, "missing export handler <" + marker + ">");
    return pos;
  });
  var start = positions[index];
  var end = index + 1 < positions.length ? positions[index + 1] : src.length;
  h.assert(end > start, "export handler span must be non-empty");
  return src.slice(start, end);
}

// (a) The view constructor is content-free: a fixed docs resource type, the
//     explicit anonymous-guest actor, action "view", and a single-key payload
//     whose only entry is the "shared-link" provenance marker.
h.staticTest("Shared-link read emits an anonymous content-free view event", function () {
  var view = between(ACCESS_AUDIT, VIEW_FN, "/// An export performed by a verified Docs user");

  contains(ACCESS_AUDIT, 'const RESOURCE_DOCS: &str = "docs";',
    "Docs resource type is fixed to the share dialog's value");
  contains(ACCESS_AUDIT, 'const ANONYMOUS_GUEST: &str = "anonymous-guest";',
    "guest actor marker exists for a token redeemed without a session");

  [
    "resource_type: RESOURCE_DOCS,",
    "actor_id: ANONYMOUS_GUEST.to_string(),",
    "actor_name: ANONYMOUS_GUEST_NAME.to_string(),",
    'action: "view",',
    'payload: serde_json::json!({ "via": "shared-link" }),'
  ].forEach(function (field) {
    h.assert(view.indexOf(field) !== -1,
      "view event must carry <" + field + "> :: <" + view + ">");
  });

  // Non-vacuity: exactly one payload literal, and neither token nor body text
  // appears anywhere in the constructor.
  var payloads = view.split("serde_json::json!(").length - 1;
  h.assertEqual(payloads, 1, "view event must build exactly one payload literal, found " + payloads);
  h.assert(view.indexOf("token") === -1 && view.indexOf("content") === -1,
    "view payload must not mention token or content :: <" + view + ">");

  contains(SHARED_LINK, SHARED_HOOK, "shared-link read route records the view");
});

// (b) Every mounted export handler appends the same audit hook with the
//     verified Docs user (not a link guest) and only the format token. The
//     constructor keeps exactly one payload key and never carries body bytes.
h.staticTest("Every export handler appends an owner-and-format-only event", function () {
  var exportFn = between(ACCESS_AUDIT, EXPORT_FN,
    "/// Mirror of `collab_routes::helpers::record_activity`'s INSERT");

  [
    "resource_type: RESOURCE_DOCS,",
    "actor_id: actor_id.to_string(),",
    "actor_name: actor_name.to_string(),",
    'action: "export",',
    'payload: serde_json::json!({ "format": format }),'
  ].forEach(function (field) {
    h.assert(exportFn.indexOf(field) !== -1,
      "export event must carry <" + field + "> :: <" + exportFn + ">");
  });

  var payloads = exportFn.split("serde_json::json!(").length - 1;
  h.assertEqual(payloads, 1, "export event must build exactly one payload literal, found " + payloads);
  h.assert(exportFn.indexOf("content") === -1 && exportFn.indexOf("bytes") === -1,
    "export payload must not carry document content :: <" + exportFn + ">");

  // The verified owner, not a guest marker, is forwarded into the event.
  contains(ACCESS_AUDIT,
    "AccessAuditEvent::export(user.id(), user.display_name(), resource_id, format),",
    "export hook forwards the verified Docs user identity and format only");

  // One hook per mounted format; its format token must match the route.
  var src = h.readRepo(EXPORT_HANDLERS);
  FORMATS.forEach(function (fmt, index) {
    var span = handlerSpan(src, index);
    var hook = 'crate::access_audit::record_export_access(&state, &user, &doc.id, "' + fmt + '");';
    h.assert(span.indexOf(hook) !== -1,
      "handle_export_" + fmt + " must append the export audit hook :: <" + span.slice(0, 200) + ">");
  });
});

// (c) Fail-closed placement: a view is recorded only after the document is
//     actually loaded, and an export only after the bytes/render were produced.
//     The module is compiled into the crate and both regression tests exist.
h.staticTest("Audit hooks are fail-closed and both unit tests exist", function () {
  contains(LIB, "pub mod access_audit;", "access_audit is compiled into botdocs");

  // Shared link: load -> audit -> response, so a rejected request cannot emit
  // a success event.
  var shared = between(SHARED_LINK, "pub async fn handle_shared_document(", "async fn load_link(");
  var load = shared.indexOf("let document = load_document_from_drive(");
  var viewHook = shared.indexOf(SHARED_HOOK);
  var response = shared.indexOf("Ok(Json(SharedDocument {");
  h.assert(load !== -1 && viewHook !== -1 && response !== -1 && load < viewHook && viewHook < response,
    "view hook must run only after a successful load :: <" + shared.slice(0, 360) + ">");

  // Exports: the analytics call follows the renderer for each format, and the
  // audit hook follows that call — never a pre-render or failure path.
  var src = h.readRepo(EXPORT_HANDLERS);
  FORMATS.forEach(function (fmt, index) {
    var span = handlerSpan(src, index);
    var analytics = 'record_export(&state, "' + fmt + '", started).await;';
    var hook = 'crate::access_audit::record_export_access(&state, &user, &doc.id, "' + fmt + '");';
    var a = span.indexOf(analytics);
    var b = span.indexOf(hook);
    h.assert(a !== -1 && b !== -1 && a < b,
      "export hook must follow successful render for " + fmt + " :: <" + span.slice(-260) + ">");
  });

  // The two regression tests pin down the contracts above.
  var auditSrc = h.readRepo(ACCESS_AUDIT);
  var tests = auditSrc.slice(auditSrc.indexOf("#[cfg(test)]"));
  h.assert(tests.indexOf("fn shared_view_is_anonymous_and_content_free()") !== -1,
    "shared-view unit test must exist");
  h.assert(tests.indexOf("fn export_keeps_only_owner_and_format()") !== -1,
    "export unit test must exist");
  h.assert(tests.indexOf('assert!(event.payload.get("token").is_none());') !== -1 &&
    tests.indexOf('assert!(event.payload.get("content").is_none());') !== -1,
    "shared-view test must assert no token/content in payload");
  h.assert(tests.indexOf('assert!(event.payload.as_object().is_some_and(|map| map.len() == 1));') !== -1,
    "export test must assert exactly one payload key");
});
