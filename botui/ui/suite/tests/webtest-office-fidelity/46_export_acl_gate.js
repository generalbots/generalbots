/**
 * #2452 — deterministic static coverage for the SHIPPED role-based export ACL
 * gate (botserver/crates/botdocs/src/handlers_api/export_acl.rs + export.rs):
 *   (a) the shared loader authorizes BEFORE any document bytes are read,
 *       rendered or audited, and all six export formats go through it;
 *   (b) a denial is the typed `docs_acl_denied` 403 with a stable, non-sensitive
 *       reason and never the document body/id/identity, and every unresolved
 *       store/role path fails closed;
 *   (c) three deterministic unit tests ship (editor success, viewer denial,
 *       unresolved role) and the IP-aware access audit stays on all success
 *       paths.
 *
 * All assertions are literal substrings of production source. Nothing is
 * executed, no server, no browser, no cargo, no network. NON-VACUITY: each case
 * first proves the real gate/handler/test artifacts ship and that the extracted
 * source span is non-empty; formating checks use strict ordering, so removing
 * `export_decision` or moving it after `load_document_from_drive` makes the
 * case FAIL.
 */
"use strict";

var h = require("./00_harness");

var EXPORT = "botserver/crates/botdocs/src/handlers_api/export.rs";
var ACL = "botserver/crates/botdocs/src/handlers_api/export_acl.rs";

var FORMATS = ["pdf", "docx", "odt", "md", "html", "txt"];
var RENDER = {
  pdf: "html_to_pdf(&doc.content)",
  docx: "docx_bytes_with_fidelity(",
  odt: "html_to_odt_zip(",
  md: "html_to_markdown(&doc.content)",
  html: "render_html_export(",
  txt: "strip_html(&doc.content)"
};
var LOAD_CALL = "let doc = load_export_document(&state, &user, &query).await?;";
var GATE_CALL = "let decision = export_decision(state, user, &query.id);";

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  h.assert(h.readRepo(relPath).indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath);
}

/** Exact, non-empty source span between two literal markers (end exclusive). */
function between(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  var span = src.slice(start, end);
  h.assert(span.length > 0, "empty source span between <" + startMarker + "> and <" + endMarker + ">");
  return span;
}

/** Literal substring assertion on an already-read source span. */
function spanHas(span, needle, msg) {
  h.assert(span.indexOf(needle) !== -1,
    (msg || "span must contain") + " <" + needle + "> :: " + span.slice(0, 200));
}

/** Assert a token is absent from a span (the non-disclosure assertion). */
function spanLacks(span, needle, msg) {
  h.assert(span.indexOf(needle) === -1,
    (msg || "span must lack") + " found <" + needle + "> :: " + span.slice(0, 200));
}

/** Assert `before` occurs strictly earlier than `after` inside a source span. */
function ordered(span, before, after, msg) {
  var b = span.indexOf(before);
  var a = span.indexOf(after);
  h.assert(b !== -1 && a !== -1 && b < a,
    (msg || "ordering regressed") + " :: <" + before + "> before <" + after +
    "> in <" + span.slice(0, 160) + ">");
}

/** Start offsets of the seven top-level handlers/loader, in shipped order. */
function handlerBoundaries(src) {
  var names = [
    "pub async fn handle_export_pdf(",
    "pub async fn handle_export_docx(",
    "pub async fn handle_export_odt(",
    "async fn load_export_document(",
    "pub async fn handle_export_md(",
    "pub async fn handle_export_html(",
    "pub async fn handle_export_txt("
  ];
  var positions = names.map(function (name) {
    var p = src.indexOf(name);
    h.assert(p !== -1, "missing top-level marker <" + name + ">");
    return p;
  });
  for (var i = 1; i < positions.length; i++) {
    h.assert(positions[i] > positions[i - 1],
      "top-level marker ordering regressed at <" + names[i] + ">");
  }
  return positions.concat([src.length]);
}

// (a) The shared loader authorizes before any load/render/audit, and every one
//     of the six export handlers funnels through that single gated loader.
h.staticTest("#2452 export ACL gate precedes load/render/audit in the shared loader and covers all six formats", function () {
  var src = h.readRepo(EXPORT);

  // Non-vacuity: the shared loader and its gate call really ship.
  contains(EXPORT, "async fn load_export_document(", "shared export loader is shipped");
  contains(EXPORT, GATE_CALL, "export ACL gate call is shipped");
  contains(ACL, "pub(super) fn export_decision(", "export decision primitive is shipped");

  // All six handlers call the one gated loader (no format bypasses the gate).
  var loadCalls = src.split(LOAD_CALL).length - 1;
  h.assertEqual(loadCalls, 6, "all six export handlers must call the shared gated loader");

  var boundaries = handlerBoundaries(src);
  FORMATS.forEach(function (fmt) {
    var start = src.indexOf("pub async fn handle_export_" + fmt + "(");
    h.assert(start !== -1, "missing handler <" + fmt + ">");
    var next = boundaries.filter(function (p) { return p > start; })[0];
    var handler = src.slice(start, next);
    h.assert(handler.length > 0, "empty handler span for <" + fmt + ">");
    spanHas(handler, LOAD_CALL, fmt + " handler goes through the shared gate");
    ordered(handler, LOAD_CALL, RENDER[fmt], fmt + " gate must precede rendering");
    ordered(handler, LOAD_CALL, "record_export_access_with_ip(",
      fmt + " gate must precede the access audit");
  });

  // Strict ordering inside the shared loader: the decision happens before any
  // document bytes (or the DLP redaction of those bytes) are read.
  var loader = between(src, "async fn load_export_document(", "pub async fn handle_export_md(");
  spanHas(loader, GATE_CALL, "loader calls the export decision");
  spanHas(loader, "if !decision.is_allowed() {", "loader short-circuits on denial");
  ordered(loader, GATE_CALL, "load_document_from_drive(state, user_id, &query.id).await",
    "gate must run before any document bytes are read");
  ordered(loader, GATE_CALL, "return Err(acl_denied(decision.reason()));",
    "denial must be returned from the gate, not after the load");
});

// (b) A refusal is typed 403 / docs_acl_denied with a stable reason token and
//     no object payload or identity, and unresolved store/role paths deny.
h.staticTest("#2452 export denial is a typed docs_acl_denied 403 with a stable reason, no document data, and fails closed", function () {
  var aclSrc = h.readRepo(ACL);

  // Non-vacuity: the decision and denial primitives really ship.
  contains(ACL, "pub(super) fn export_decision(", "export decision is shipped");
  contains(ACL, "fn decide_export(", "pure export decision is shipped");
  contains(ACL, "decide(&principal, Action::Export, acl, now)",
    "Export action runs through the shared ACL policy");
  contains(ACL, "pub(super) fn acl_denied(", "typed denial builder is shipped");
  contains(ACL, "fn reason_code(", "stable reason mapper is shipped");

  var denial = between(aclSrc, "pub(super) fn acl_denied(", "#[cfg(test)]");
  spanHas(denial, "StatusCode::FORBIDDEN", "denial maps to 403");
  spanHas(denial, '"code": "docs_acl_denied"', "denial carries the stable machine code");
  spanHas(denial, '"reason": reason_code(reason)', "denial carries a stable reason token");
  spanHas(denial, '"error": "Document not found or access denied"',
    "denial error text is generic, never the object body");

  // No document id, body, owner or caller identity can appear in the refusal.
  ["doc.id", ".id", "content", "body", "grantee", "display_name", "owner", "email"].forEach(function (token) {
    spanLacks(denial, token, "denial must not leak <" + token + ">");
  });

  // The reason vocabulary is a closed, stable set in reason_code.
  var reasons = between(aclSrc, "fn reason_code(", "/// Non-sensitive reason token for logging");
  ["insufficient-role", "no-grant", "no-identity", "cross-tenant"].forEach(function (token) {
    spanHas(reasons, '"' + token + '"', "stable reason <" + token + "> is defined");
  });

  // Fail closed: unknown persisted roles, store errors and a missing grant all
  // deny without any document access.
  contains(ACL, "fn policy_role(role: &str) -> Option<Role> {", "role parser is shipped");
  contains(ACL, "let role = policy_role(&row.role)?;", "unknown role aborts projection");
  contains(ACL, "fn load_acl(", "ACL loader is shipped");
  contains(ACL, "docs export: permission store unavailable; denying", "pool failure denies");
  contains(ACL, "docs export: permission lookup failed; denying", "lookup failure denies");
  var decisionFn = between(aclSrc, "pub(super) fn export_decision(", "/// Stable, non-sensitive code");
  spanHas(decisionFn, "None => return Decision::deny(Reason::NoGrant)",
    "unresolved role fails closed with NoGrant");
});

// (c) Three deterministic tests cover editor success, viewer denial and an
//     unresolved role; the IP-aware success-path audit is preserved.
h.staticTest("#2452 three deterministic export-ACL tests ship (viewer denial + unresolved role) and IP threading stays on all success paths", function () {
  var aclSrc = h.readRepo(ACL);
  var tests = aclSrc.slice(aclSrc.indexOf("#[cfg(test)]"));
  h.assert(tests.length > 0, "export_acl test module must be present");

  ["fn editor_role_exports_and_no_grants_keeps_owner_export()",
   "fn viewer_role_gets_typed_denial_not_404()",
   "fn unresolved_role_fails_closed()"].forEach(function (name) {
    h.assert(tests.indexOf(name) !== -1, "deterministic test must exist <" + name + ">");
  });

  var viewer = between(tests, "fn viewer_role_gets_typed_denial_not_404()", "fn unresolved_role_fails_closed()");
  spanHas(viewer, "Reason::InsufficientRole {", "viewer denial asserts the exact reason");
  spanHas(viewer, "required: Role::Editor,", "viewer denial requires Editor");
  spanHas(viewer, "actual: Role::Viewer,", "viewer denial records Viewer as actual");
  spanHas(viewer, 'assert_eq!(body.0["code"], "docs_acl_denied");', "viewer denial asserts the typed code");
  spanHas(viewer, 'assert_eq!(body.0["reason"], "insufficient-role");', "viewer denial asserts a stable reason");
  spanHas(viewer, 'assert!(body.0.get("content").is_none());', "viewer denial asserts no document body");
  spanHas(viewer, "StatusCode::FORBIDDEN", "viewer denial asserts 403, not 404");

  var unresolvedIdx = tests.indexOf("fn unresolved_role_fails_closed()");
  h.assert(unresolvedIdx !== -1, "unresolved role test body must be present");
  var unresolved = tests.slice(unresolvedIdx);
  spanHas(unresolved, 'assert!(project_acl(&rows, "bob", "bob", ORG).is_none());',
    "unknown role is unresolvable");
  spanHas(unresolved, "Decision::deny(Reason::NoGrant)", "unresolved role fails closed");
  spanHas(unresolved, 'denial_code(decision.reason()), "no-grant"',
    "a foreign caller with no grant is denied");

  // The #2448 IP-aware access audit still runs on every success path; a denial
  // returns from the gate before any of this code is reached.
  var src = h.readRepo(EXPORT);
  var ips = src.split("let client_ip = addr.ip().to_string();").length - 1;
  h.assertEqual(ips, 6, "all six success paths must thread the client IP");
  var audits = src.split("record_export_access_with_ip(").length - 1;
  h.assertEqual(audits, 6, "all six success paths must record IP-aware export access");
  var conn = src.split("ConnectInfo(addr): ConnectInfo<SocketAddr>").length - 1;
  h.assertEqual(conn, 6, "all six handlers must receive ConnectInfo");
  var threaded = src.split("Some(client_ip.as_str())").length - 1;
  h.assertEqual(threaded, 6, "all six audit calls must forward Some(client_ip)");
});

h.note("FOLLOW-UP #2452: the six export formats are gated by the shared export_decision load " +
  "with a typed docs_acl_denied 403; these are source assertions only — no browser or server run.");
