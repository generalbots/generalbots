/**
 * [KNOWN BUG] #2452 — source-backed guards for the four sharing/ACL criteria
 * that the live UAT (v7.1.2, 2026-11-25) still found failing:
 *   (a) the anonymous capability-link read returns the FULL document body with
 *       no caller identity or role gate — the handler takes only State + Path,
 *       the token lookup is keyed by token alone, content is loaded from the
 *       link owner's namespace and echoed as `content: document.content`;
 *   (b) export denial is namespace/owner-lookup based (404 "Document not
 *       found" from the loader / 401 from the DocUser extractor), never a role
 *       decision — all six handlers load from `user.id()` and there is no
 *       effective_role/acl call in export.rs;
 *   (c) the live audit rows (GET /api/activity DTO and the collab_activity
 *       INSERT used by both record_activity and access_audit) carry actor/time
 *       but NO ip / remote_addr field. The offline access_control::audit::
 *       AuditEvent does have `ip`, but it is not wired to the HTTP path;
 *   (d) the decision/denial primitive ships (decide(), acl_denied() with code
 *       "docs_acl_denied" in handlers_api/ast.rs, re-exported from access_control)
 *       yet BOTH read paths bypass it — the anonymous capability path never
 *       calls decide() and export.rs never calls it either, so no request can
 *       produce a 403/docs_acl_denied response. Expiry and malformed-token
 *       fail-closed DO work and are deliberately not claimed as gaps here.
 *
 * Each case asserts the CURRENT degraded shape and PASSES as a documented
 * KNOWN BUG. NON-VACUITY: every case first proves the shipped artifact exists
 * and carries its real handlers, SQL, DTO fields and primitive declarations;
 * the "gap" assertions are then scoped absence checks, so a missing, empty or
 * garbled file can never satisfy a case by accident. When a real fix lands an
 * asserted-in gap disappears, its knownBug fn throws, the case becomes FAIL and
 * this marker must be removed and replaced by a runtime test. No user-facing
 * string, no server, no browser, no cargo.
 */
"use strict";

var h = require("./00_harness");

var SHARED_LINK = "botserver/crates/botdocs/src/shared_link.rs";
var EXPORT_HANDLERS = "botserver/crates/botdocs/src/handlers_api/export.rs";
var AST_HANDLER = "botserver/crates/botdocs/src/handlers_api/ast.rs";
var ACCESS_CONTROL = "botserver/crates/botdocs/src/access_control/mod.rs";
var ACCESS_AUDIT = "botserver/crates/botdocs/src/access_audit.rs";
var POLICY_AUDIT = "botserver/crates/botdocs/src/access_control/audit.rs";
var COLLAB_DTO = "botserver/src/main_module/routes/collab_routes/dto.rs";
var COLLAB_HELPERS = "botserver/src/main_module/routes/collab_routes/helpers.rs";

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

// (a) The capability read has no caller to authorise: the handler signature is
//     State + Path only, the SQL key is the token alone, the body is loaded
//     from the link owner's namespace and the full text is returned verbatim.
//     The link's role is echoed as metadata, not computed by a decision.
h.knownBug("anonymous capability read echoes full document content with no caller identity/role gate", function () {
  contains(SHARED_LINK, "pub async fn handle_shared_document(", "shared document route is shipped");
  contains(SHARED_LINK, "pub struct SharedDocument {", "response DTO is shipped");
  contains(SHARED_LINK, "pub content: String,", "response DTO carries the document body");
  contains(SHARED_LINK, "pub role: String,", "response DTO carries a role field");
  contains(SHARED_LINK, "FROM resource_links WHERE token = $1", "token-only lookup is shipped");
  contains(SHARED_LINK, "resolve_owner(&state, &link)", "owner namespace comes from the link row");
  contains(SHARED_LINK, "load_document_from_drive(&state, &owner_id, &link.resource_id)",
    "content is loaded from the link owner's namespace");
  contains(SHARED_LINK, "content: document.content,", "full body is returned");
  contains(SHARED_LINK, "role: link.role,", "role is echoed from the link row");

  var sig = between(SHARED_LINK, "pub async fn handle_shared_document(", "let token = token.trim()");
  absent(sig, [
    "Extension<", "AuthenticatedUser", "DocUser", "user.id()", "my_role(",
    "effective_role(", "grantee_id", "require_role"
  ], "caller identity extractor on the shared route");

  var lookup = between(SHARED_LINK, "async fn load_link(", "/// Owner namespace of the linked document.");
  absent(lookup, [
    "AND owner_id", "AND org_id", "effective_role(", "my_role(", "grantee_id", "principal"
  ], "caller scope in the token lookup");

  var body = between(SHARED_LINK, "let token = token.trim()", "#[cfg(test)]");
  absent(body, [
    "Extension<", "AuthenticatedUser", "user.id()", "effective_role(", "my_role(",
    "grantee_id", "principal"
  ], "caller role check in the shared route body");
});

// (b) Every export handler authenticates only through the DocUser extractor and
//     the workclass admission gate; the common loader reads from that bearer's
//     own namespace and turns a miss into 404 "Document not found". No role
//     lookup, ACL decision or 403 exists in the file, so denial stays a
//     namespace/auth artifact rather than a role decision.
h.knownBug("export denial is namespace/auth based with no role gate in any format handler", function () {
  var FORMATS = ["pdf", "docx", "odt", "md", "html", "txt"];
  var src = h.readRepo(EXPORT_HANDLERS);
  FORMATS.forEach(function (fmt) {
    contains(EXPORT_HANDLERS, "pub async fn handle_export_" + fmt + "(", "export handler " + fmt + " is shipped");
  });
  h.assert((src.match(/user: DocUser,/g) || []).length >= FORMATS.length,
    "non-vacuity: every export handler must take the DocUser extractor");
  contains(EXPORT_HANDLERS, "fn load_export_document(", "export loader is shipped");
  contains(EXPORT_HANDLERS, "let user_id = user.id();", "loader derives the namespace from the bearer");
  contains(EXPORT_HANDLERS, "load_document_from_drive(state, user_id, &query.id)",
    "loader reads only the bearer's namespace");
  contains(EXPORT_HANDLERS, '"error": "Document not found"', "namespace miss is a 404");
  contains(EXPORT_HANDLERS, "admit(", "workclass admission gate is shipped");

  var loader = between(EXPORT_HANDLERS, "fn load_export_document(", "enum HtmlBlock {");
  h.assert(loader.indexOf("StatusCode::NOT_FOUND") !== -1,
    "the loader must still deny via 404 :: <" + loader.slice(0, 400) + ">");

  var noRole = [
    "role", "effective_role(", "my_role(", "grantee_id", "acl", "FORBIDDEN",
    "Principal", "principal", "Action::"
  ];
  absent(src, noRole, "role-based export gate in export.rs");
  absent(loader, noRole, "role-based decision in the export loader");
});

// (c) The rows a reader actually sees through GET /api/activity are ActivityItem
//     from collab_routes/dto.rs, written by helpers::record_activity and by
//     access_audit::append into the same collab_activity table. All three carry
//     actor + created_at but no IP/remote_addr column. The offline
//     access_control::audit::AuditEvent has `ip`, proving the field is known to
//     the codebase yet absent from the live audit path.
h.knownBug("live audit rows carry actor/time but no IP or remote_addr field", function () {
  contains(COLLAB_DTO, "pub struct ActivityItem {", "activity response DTO is shipped");
  contains(COLLAB_DTO, "pub actor_id: String,", "actor id is recorded");
  contains(COLLAB_DTO, "pub actor_name: String,", "actor name is recorded");
  contains(COLLAB_DTO, "pub created_at: String,", "timestamp is recorded");
  contains(COLLAB_DTO, "pub(crate) struct ActivityRow {", "activity SQL row is shipped");
  contains(COLLAB_HELPERS, "pub(crate) fn record_activity(", "generic activity writer is shipped");
  contains(ACCESS_AUDIT, "pub struct AccessAuditEvent {", "Docs access audit event is shipped");

  var columns = "(resource_type, resource_id, actor_id, actor_name, action, payload)";
  contains(COLLAB_HELPERS, columns, "generic activity INSERT column list is shipped");
  contains(ACCESS_AUDIT, columns, "Docs access audit INSERT column list is shipped");
  contains(POLICY_AUDIT, "pub struct AuditEvent {", "offline audit event is shipped");
  contains(POLICY_AUDIT, "pub ip: Option<String>,", "the offline policy event knows the IP field");

  var ipFields = [
    "remote_addr", "ip_address", "client_ip", "peer_addr", "remote_ip",
    "pub ip:", "    ip:", '"ip"'
  ];
  absent(h.readRepo(COLLAB_DTO), ipFields, "IP field on the activity DTO/row");
  absent(h.readRepo(COLLAB_HELPERS), ipFields, "IP field in the activity INSERT");
  absent(h.readRepo(ACCESS_AUDIT), ipFields, "IP field in the Docs access audit event/INSERT");
});

// (d) A typed refusal primitive is fully implemented for the authenticated AST
//     route (decide() + acl_denied() with code "docs_acl_denied", 403), but the
//     anonymous capability path and every export handler bypass it: neither
//     file references decide, acl_denied, the denial code or FORBIDDEN. A real
//     fix that routes either read path through the decision flips this case.
h.knownBug("docs_acl_denied/decide primitive ships but is bypassed by the anonymous and export paths", function () {
  contains(AST_HANDLER, "fn acl_denied(", "typed ACL refusal is shipped");
  contains(AST_HANDLER, '"code": "docs_acl_denied"', "machine-readable denial code is shipped");
  contains(AST_HANDLER, "StatusCode::FORBIDDEN", "ACL denial maps to 403 in ast.rs");
  contains(AST_HANDLER, "decide(", "policy decision is invoked by ast.rs");
  contains(ACCESS_CONTROL, "pub use decision::decide;", "decision primitive is re-exported");

  var decisionTokens = [
    "decide(", "acl_denied", "docs_acl_denied", "FORBIDDEN", "Action::",
    "Principal", "principal", "AuditLog::record"
  ];
  absent(h.readRepo(SHARED_LINK), decisionTokens, "ACL decision on the anonymous capability path");
  absent(h.readRepo(EXPORT_HANDLERS), decisionTokens, "ACL decision in the export handlers");
});

h.note(
  "FOLLOW-UP #2452 criteria 2/3/4: route the capability and export reads through " +
  "decide() with a real principal, return docs_acl_denied/403 on insufficient role, " +
  "and persist an IP/remote_addr on the live collab_activity audit rows."
);
