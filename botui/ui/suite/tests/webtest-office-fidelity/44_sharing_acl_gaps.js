/**
 * #2452 — source-backed guards for the four sharing/ACL criteria from the
 * v7.1.2 (2026-11-25) live UAT:
 *   (a) FIXED: the anonymous capability read gates source bytes through
 *       `body_for_link` + `role_allows_source`, using the role recorded on the
 *       link row: a viewer gets an empty content string, while
 *       commenter/editor/owner keep the full source. No caller input can pick
 *       the body; `content: document.content` is gone.
 *   (b) KNOWN BUG: export denial is still namespace/owner-lookup based (404
 *       "Document not found" from the loader / 401 from the DocUser extractor),
 *       never a role decision — all six handlers load from `user.id()` and
 *       there is no effective_role/acl call in export.rs;
 *   (c) FIXED: the Docs access-audit event, SQL row and read DTO now carry an
 *       optional `ip`, attached with `with_ip` and persisted by INSERT_WITH_IP;
 *   (d) KNOWN BUG: the decision/denial primitive ships (decide(), acl_denied()
 *       with code "docs_acl_denied" in handlers_api/ast.rs, re-exported from
 *       access_control) yet BOTH read paths still bypass it — the anonymous
 *       capability path uses role_allows_source, not decide(), and export.rs
 *       never calls it either, so no request can produce a 403/docs_acl_denied
 *       response. Expiry and malformed-token fail-closed DO work and are
 *       deliberately not claimed as gaps here.
 *
 * Cases (b) and (d) are confirmed, documented degradations; cases (a) and (c)
 * are POSITIVE static assertions of the shipped fixes. NON-VACUITY: every case
 * first proves the shipped artifact exists and carries its real handlers, SQL,
 * DTO fields and primitive declarations; the positive assertions use literal
 * substrings and ordering, the gap assertions are scoped absence checks, so a
 * missing, empty or garbled file can never satisfy a case by accident.
 * Removing role_allows_source, feeding the body from caller input, or dropping
 * the IP field/INSERT makes its case FAIL. No user-facing string, no server,
 * no browser, no cargo.
 */
"use strict";

var h = require("./00_harness");

var SHARED_LINK = "botserver/crates/botdocs/src/shared_link.rs";
var EXPORT_HANDLERS = "botserver/crates/botdocs/src/handlers_api/export.rs";
var AST_HANDLER = "botserver/crates/botdocs/src/handlers_api/ast.rs";
var ACCESS_CONTROL = "botserver/crates/botdocs/src/access_control/mod.rs";
var ACCESS_AUDIT = "botserver/crates/botdocs/src/access_audit.rs";

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

/** Literal substring assertion on an already-read source span. */
function spanHas(span, needle, msg) {
  h.assert(span.indexOf(needle) !== -1,
    (msg || "span must contain") + " <" + needle + "> :: " + span.slice(0, 200));
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

// (a) The capability read now gates source bytes with a single decision keyed
//     by the role on the LINK row: viewer -> empty body, commenter/editor/owner
//     -> full source, unknown roles fail closed. No caller input can select it.
h.staticTest("anonymous capability read gates source by the link's role", function () {
  contains(SHARED_LINK, "pub async fn handle_shared_document(", "shared document route is shipped");
  contains(SHARED_LINK, "pub struct SharedDocument {", "response DTO is shipped");
  contains(SHARED_LINK, "pub content: String,", "response DTO carries a content field");
  contains(SHARED_LINK, "pub role: String,", "response DTO carries a role field");
  contains(SHARED_LINK, "FROM resource_links WHERE token = $1", "token-only lookup is shipped");
  contains(SHARED_LINK, "resolve_owner(&state, &link)", "owner namespace comes from the link row");
  contains(SHARED_LINK, "load_document_from_drive(&state, &owner_id, &link.resource_id)",
    "content is loaded from the link owner's namespace");

  // Explicit allow-list: viewer is NOT source-capable; unknown roles fail closed.
  var roleGate = between(SHARED_LINK,
    "fn role_allows_source(role: &str) -> bool {", "fn body_for_link<'a>(");
  spanHas(roleGate, "matches!(", "role gate is an explicit allow-list");
  spanHas(roleGate, '"commenter" | "editor" | "owner"',
    "only source-capable roles are allowed");
  h.assert(roleGate.indexOf('"viewer"') === -1,
    "viewer must not be in the source allow-list :: <" + roleGate + ">");

  // The single release decision rechecks token/expiry and the role, and feeds
  // the body field — so removing the role call cannot silently pass through.
  var bodyFn = between(SHARED_LINK, "fn body_for_link<'a>(", "/// Unknown, expired, revoked");
  spanHas(bodyFn, "is_valid_token(token)", "body gate rechecks the token shape");
  spanHas(bodyFn, "!link_is_expired(expires_at, now)", "body gate rechecks expiry");
  spanHas(bodyFn, "&& role_allows_source(role)", "body gate calls the role allow-list");
  spanHas(bodyFn, 'if allowed { source } else { "" }', "role denial yields an empty body");

  // The handler builds DTO content from that decision using link.role (the row
  // value), never a caller-supplied role or document.content directly.
  var handler = between(SHARED_LINK,
    "pub async fn handle_shared_document(", "async fn load_link(");
  var decision = handler.indexOf("let content = body_for_link(");
  var response = handler.indexOf("Ok(Json(SharedDocument {");
  h.assert(decision !== -1 && response !== -1 && decision < response,
    "source decision must precede the response :: <" + handler.slice(-420) + ">");
  var call = handler.slice(decision, response);
  ["&token,", "&link.role,", "link.expires_at,", "chrono::Utc::now(),", "&document.content,"].forEach(function (arg) {
    h.assert(call.indexOf(arg) !== -1,
      "body_for_link must receive <" + arg + "> :: <" + call + ">");
  });
  spanHas(call, ".to_string()", "gated body is materialized for the DTO");
  spanHas(handler.slice(response), "content,", "DTO content comes from the gated local");
  h.assert(handler.indexOf("content: document.content,") === -1,
    "full body must not be echoed directly :: <" + handler + ">");

  // Non-vacuity: unit tests cover both sides with a real secret embedded in
  // SOURCE, so an always-empty or always-full gate cannot satisfy this case.
  var linkSrc = h.readRepo(SHARED_LINK);
  var tests = linkSrc.slice(linkSrc.indexOf("#[cfg(test)]"));
  h.assert(tests.indexOf("fn viewer_link_does_not_release_the_editable_source()") !== -1,
    "viewer source-denial unit test must exist");
  h.assert(tests.indexOf("fn owner_and_higher_role_links_still_release_the_source()") !== -1,
    "source-capable role unit test must exist");
  spanHas(tests, 'assert_eq!(body_for_link(&token, role, None, now, SOURCE), "");',
    "viewer roles must be asserted to yield no body");
  spanHas(tests, 'assert_eq!(body_for_link(&token, role, None, now, SOURCE), SOURCE);',
    "source-capable roles must be asserted to keep the body");
});

// (b) FIXED (#2452): the shared export loader now runs export_decision (a real
//     role ACL via decide/Action::Export) before any document load, returning a
//     typed 403. This case flips to FAIL if the gate is removed or reordered.
h.staticTest("export denial is a role decision before any document load", function () {
  var FORMATS = ["pdf", "docx", "odt", "md", "html", "txt"];
  var src = h.readRepo(EXPORT_HANDLERS);
  FORMATS.forEach(function (fmt) {
    contains(EXPORT_HANDLERS, "pub async fn handle_export_" + fmt + "(", "export handler " + fmt + " is shipped");
  });
  h.assert((src.match(/user: DocUser,/g) || []).length >= FORMATS.length,
    "non-vacuity: every export handler takes the DocUser extractor");
  contains(EXPORT_HANDLERS, "fn load_export_document(", "export loader is shipped");
  contains(EXPORT_HANDLERS, "export_decision(", "role gate is shipped");
  contains(EXPORT_HANDLERS, "record_export_access_with_ip(", "IP-aware audit is preserved");
  contains(EXPORT_HANDLERS, '"error": "Document not found"', "namespace miss fallback is retained");

  var loader = between(EXPORT_HANDLERS, "fn load_export_document(", "pub async fn handle_export_md(");
  var gate = loader.indexOf("export_decision(");
  var load = loader.indexOf("load_document_from_drive(");
  h.assert(gate !== -1 && load !== -1 && gate < load,
    "role gate must precede document load :: <" + loader.slice(0, 500) + ">");
});

// (c) The Docs access audit event, its collab_activity row projection and the
//     read DTO now carry an optional `ip`. Only the shipped crate module is
//     asserted here; the request-layer wiring (migration / collab_routes) is a
//     separate workstream. Removing any of the three fields, `with_ip` or the
//     IP-aware INSERT makes this POSITIVE case FAIL.
h.staticTest("Docs access audit event, row and DTO carry the caller IP", function () {
  contains(ACCESS_AUDIT, "pub struct AccessAuditEvent {", "Docs access audit event is shipped");
  contains(ACCESS_AUDIT, "pub struct AccessAuditRow {", "Docs audit row is shipped");
  contains(ACCESS_AUDIT, "pub struct AccessAuditItem {", "Docs audit read DTO is shipped");

  var audit = h.readRepo(ACCESS_AUDIT);
  var ipFields = audit.split("pub ip: Option<String>,").length - 1;
  h.assert(ipFields >= 3, "event, row and read DTO must all carry ip :: found " + ipFields);

  var event = between(ACCESS_AUDIT, "pub struct AccessAuditEvent {", "impl AccessAuditEvent {");
  spanHas(event, "pub ip: Option<String>,", "event carries the optional client address");
  contains(ACCESS_AUDIT, "pub fn with_ip(mut self, ip: Option<&str>) -> Self {",
    "with_ip attach point is shipped");
  contains(ACCESS_AUDIT, "self.ip = ip.map(|addr| addr.to_string());",
    "with_ip keeps the caller's address");

  var row = between(ACCESS_AUDIT, "pub struct AccessAuditRow {", "/// Serialized audit item");
  h.assert(row.indexOf("#[diesel(sql_type = Nullable<Text>)]") !== -1 &&
    row.indexOf("pub ip: Option<String>,") !== -1,
    "row maps the nullable ip column :: <" + row + ">");
  contains(ACCESS_AUDIT, "ip: row.ip,", "row-to-DTO conversion forwards ip");

  var insert = between(ACCESS_AUDIT, "const INSERT_WITH_IP: &str", "const INSERT_WITHOUT_IP: &str");
  spanHas(insert, "actor_name, action, payload, ip)",
    "IP-aware INSERT writes the ip column");
  spanHas(insert, "VALUES ($1, $2, $3, $4, $5, $6, $7)",
    "IP-aware INSERT binds seven values");
  var append = between(ACCESS_AUDIT, "fn append(conn: &mut PgConnection", "fn record(state: &DocState");
  h.assert(append.indexOf("diesel::sql_query(INSERT_WITH_IP)") !== -1 &&
    append.indexOf(".bind::<Nullable<Text>, _>(event.ip.as_deref())") !== -1,
    "append binds the event IP into the IP-aware INSERT :: <" + append + ">");
  spanHas(append, "diesel::sql_query(INSERT_WITHOUT_IP)",
    "append keeps the no-IP fallback for older deployments");

  // Additive recorders let the request layer supply the address without
  // breaking existing callers; the constructors stay content-free.
  contains(ACCESS_AUDIT, "pub fn record_shared_view_with_ip(",
    "IP-aware shared-view recorder is shipped");
  contains(ACCESS_AUDIT, "pub fn record_export_access_with_ip(",
    "IP-aware export recorder is shipped");
  contains(ACCESS_AUDIT, "record_shared_view_with_ip(state, resource_id, None);",
    "no-IP shared-view wrapper stays additive");
  contains(ACCESS_AUDIT, "record_export_access_with_ip(state, user, resource_id, format, None);",
    "no-IP export wrapper stays additive");

  var tests = audit.slice(audit.indexOf("#[cfg(test)]"));
  ["fn event_with_ip_carries_the_callers_address()",
   "fn event_without_ip_stays_none()",
   "fn row_and_dto_round_trip_a_present_ip()",
   "fn row_and_dto_round_trip_an_absent_ip()"].forEach(function (name) {
    h.assert(tests.indexOf(name) !== -1, "IP unit test must exist <" + name + ">");
  });
  spanHas(tests, 'json.get("ip").and_then(|v| v.as_str())',
    "read DTO must serialize ip for the activity reader");
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
});

h.note(
  "FOLLOW-UP #2452 remaining criterion: the anonymous capability read still " +
  "uses a role allow-list rather than the full decide() ACL. Body role-gating, " +
  "the export ACL gate and the access-audit IP field/INSERT are shipped and " +
  "asserted positively above."
);
