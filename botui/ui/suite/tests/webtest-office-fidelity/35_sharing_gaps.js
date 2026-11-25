/**
 * [KNOWN BUG] #2452 — source-backed guards for the three still-failing
 * sharing criteria (browser UAT on v7.1.2, 2026-11-24):
 *   1. viewer link + expiry via UI: dialog has no expiry/password/domain/
 *      watermark/download controls and the minted link has expires_at=null;
 *   2. foreign principal cannot read shared content: the capability endpoint
 *      looks the link up by token only, resolves the document through the link
 *      owner's namespace, and never calls decide() with a caller principal;
 *   3. exports gated by role: export handlers load from the bearer's own
 *      namespace and never call decide(), so denial is namespace-based, not
 *      role-based.
 *
 * Each case asserts the CURRENT degraded shape and PASSES as a documented
 * KNOWN BUG. NON-VACUITY: every case first proves the shipped source exists and
 * contains the real function/route plus its structural markers (the mint POST,
 * the SQL lookup, the loader, the six handlers). The "gap" assertions are then
 * scoped absence checks, so they can never pass on a missing, empty or garbled
 * file. When a real fix lands one of those asserted-in gaps disappears, its
 * knownBug fn throws, the case becomes FAIL and this marker must be removed and
 * replaced by a runtime test. No user-facing string, no server, no browser,
 * no cargo.
 */
"use strict";

var h = require("./00_harness");

var SHARE_UI = "botui/ui/suite/js/collab-share.js";
var LINK_DTO = "botserver/src/main_module/routes/collab_routes/dto.rs";
var LINK_PERMISSIONS = "botserver/src/main_module/routes/collab_routes/permissions.rs";
var SHARED_LINK = "botserver/crates/botdocs/src/shared_link.rs";
var EXPORT_HANDLERS = "botserver/crates/botdocs/src/handlers_api/export.rs";

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  h.assert(
    h.readRepo(relPath).indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath
  );
}

/** Exact source span between two markers (exclusive of the end marker). */
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

/** Assert every token is absent from a span (the documented defect). */
function absent(span, tokens, what) {
  tokens.forEach(function (token) {
    h.assert(
      span.indexOf(token) === -1,
      (what || "gap") + " must still be absent: found <" + token + "> :: <" + span.slice(0, 200) + ">"
    );
  });
}

// 1. The GENERIC share dialog exposes only a role selector and a Create-link
//    button; the backend accepts expires_in_hours and an explicit expires_at,
//    and the Docs adapter (docs/modules/15_readonly.js) now injects an optional
//    expiry. The generic dialog itself still lacks expiry/password/domain/
//    watermark/download controls — this guard flips when those are added.
h.knownBug("generic share dialog has no built-in expiry/password/domain/watermark/download controls; Docs adds expiry via an adapter", function () {
  contains("botui/ui/suite/docs/modules/15_readonly.js", "expires_in_hours", "Docs adapter sends the supported expiry field");
  contains(SHARE_UI, "function createLink() {", "createLink is shipped");
  contains(SHARE_UI, 'id="gbs-create-link"', "the Create-link button is shipped");
  contains(SHARE_UI, 'id="gbs-link-role"', "the link-role selector is shipped");

  var mint = between(SHARE_UI, "function createLink() {", "\n  function revokeLink(");
  [
    'api("/api/collab/links", {',
    'method: "POST",',
    "resource_type: state.resourceType,",
    "resource_id: state.resourceId,",
    "role: role"
  ].forEach(function (marker) {
    h.assert(mint.indexOf(marker) !== -1,
      "mint POST must still carry <" + marker + "> :: <" + mint + ">");
  });

  // Non-vacuity: the capability exists server-side but the UI never uses it.
  contains(LINK_DTO, "pub expires_in_hours: Option<i64>,", "backend LinkBody accepts an expiry");
  contains(LINK_PERMISSIONS, ".expires_in_hours", "backend reads the requested expiry");

  absent(mint, [
    "expires_in_hours", "expires_at", "expiry", "expires", "password",
    "watermark", "allow_download", "allowlist", "domain_allow", "domain-allow"
  ], "link-policy fields in the mint POST");

  var ui = h.readRepo(SHARE_UI);
  absent(ui, [
    "gbs-expiry", "gbs-password", "gbs-domain", "gbs-watermark", "gbs-download",
    "expires_in_hours", "expires_at", "password", "watermark", "allow_download",
    "domain_allowlist", "domain-allowlist"
  ], "link-policy controls in the share dialog");
});

// 2. GET /api/docs/shared/:token looks the row up by token alone, resolves the
//    namespace from the link's owner_id, and loads that owner's document. No
//    caller principal, org scope or decide() call exists on either path.
h.knownBug("shared-link capability endpoint reads by link owner only with no identity/org scope and no decide()", function () {
  contains(SHARED_LINK, "pub async fn handle_shared_document(", "shared-document route is shipped");
  contains(SHARED_LINK, "async fn load_link(", "token lookup is shipped");
  contains(SHARED_LINK, "FROM resource_links WHERE token = $1", "lookup is keyed by token alone");
  contains(SHARED_LINK, "resolve_owner(&state, &link)", "namespace comes from the link owner");
  contains(SHARED_LINK, "load_document_from_drive(&state, &owner_id, &link.resource_id)",
    "content is loaded from the link owner's namespace");

  var handler = between(SHARED_LINK, "pub async fn handle_shared_document(", "async fn load_link(");
  var lookup = between(SHARED_LINK, "async fn load_link(", "/// Owner namespace of the linked document.");
  var noScope = [
    "decide(", "principal", "org_id", "organization_id", "AuthenticatedUser",
    "DocUser", "grantee_id", "grantee_type"
  ];
  absent(handler, noScope, "principal org scope / decide() in the shared route");
  absent(lookup, noScope.concat(["AND owner_id", "AND org_id"]), "principal org scope in the token lookup");
  absent(h.readRepo(SHARED_LINK), ["decide(", "principal", "org_id"], "any ACL decision in shared_link.rs");
});

// 3. FIXED (#2452): all six export handlers share the loader, which now runs
//    export_decision (role ACL) BEFORE any document load or audit. This case
//    flips to FAIL if the gate is removed or moved after the load.
h.staticTest("export handlers gate by role before loading any document", function () {
  var FORMATS = ["pdf", "docx", "odt", "md", "html", "txt"];
  h.assert(h.readRepo(EXPORT_HANDLERS).indexOf("export_decision(") !== -1,
    "export gate is shipped");
  FORMATS.forEach(function (fmt) {
    contains(EXPORT_HANDLERS, "pub async fn handle_export_" + fmt + "(", "export handler " + fmt + " is shipped");
  });
  var loader = between(EXPORT_HANDLERS, "fn load_export_document(", "pub async fn handle_export_md(");
  h.assert(loader.indexOf("export_decision(") !== -1, "loader invokes the role gate");
  h.assert(loader.indexOf("load_document_from_drive(") !== -1, "loader still loads the document");
  h.assert(loader.indexOf("export_decision(") < loader.indexOf("load_document_from_drive("),
    "role gate must precede document load :: <" + loader.slice(0, 500) + ">");
});
