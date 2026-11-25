/**
 * #2448 — deterministic static coverage for the newly gated admin
 * track-changes HTTP endpoint:
 *   (a) handle_accept_reject_all calls resolve_admin BEFORE
 *       load_document_from_drive, and consumes only the server-derived DocUser
 *       id (no request-supplied role, identity or header);
 *   (b) a non-admin or unresolved role is refused with the typed
 *       docs_acl_denied 403 and a stable reason token, never the document id,
 *       body or caller PII — and the catalog-level gate stays in lockstep;
 *   (c) three deterministic unit tests ship, including near-miss admin group
 *       names and an unresolved-role fail-closed case.
 *
 * All assertions are literal substrings of production source. Nothing is
 * executed: no server, no browser, no cargo, no network. NON-VACUITY: each case
 * first proves the real handler / denial builder / test module ships and that
 * the extracted span is non-empty, then asserts exact substrings and strict
 * ordering, so deleting the gate or moving it after the load FAILs.
 */
"use strict";

var h = require("./00_harness");

var TRACK = "botserver/crates/botdocs/src/handlers_api/track_changes.rs";
var AUTH = "botserver/src/core/bot/api_catalog/command_auth.rs";

var ACCEPT_ALL = "pub async fn handle_accept_reject_all(";
var LIST_ALL = "pub async fn handle_list_track_changes(";
var GATE_CALL = "match AdminAccess::from_resolved(resolve_admin(&state, user.id())) {";
var LOAD_CALL = "let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {";

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  h.assert(h.readRepo(relPath).indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath);
}

/** Count exact, literal occurrences of a needle in a file or span. */
function countIn(src, needle) {
  return src.split(needle).length - 1;
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

/** Every marker must be present, in strict order. */
function sequence(span, markers, msg) {
  var last = -1;
  markers.forEach(function (marker) {
    var at = span.indexOf(marker);
    h.assert(at !== -1, (msg || "span") + " must contain <" + marker + ">");
    h.assert(at > last, (msg || "span") + " order broken at <" + marker + ">");
    last = at;
  });
}

// (a) Authorize before the document lookup, using only the verified DocUser id.
h.staticTest("#2448 track-changes accept-all resolves admin before the drive load and uses only the server-derived DocUser id", function () {
  var src = h.readRepo(TRACK);

  // Non-vacuity: the handler, the gate call and the loader call really ship.
  contains(TRACK, ACCEPT_ALL, "accept-all handler is shipped");
  contains(TRACK, GATE_CALL, "admin gate call is shipped");
  contains(TRACK, LOAD_CALL, "document loader call is shipped");
  contains(TRACK, "principal id the auth middleware derived, not a client-supplied value",
    "the id source is documented as server-derived");

  // Scope tightly to accept-all so the ungated sibling handlers cannot satisfy
  // or invalidate any assertion here.
  var handler = between(src, ACCEPT_ALL, LIST_ALL);

  // The verified principal is the only identity input to the decision and the
  // subsequent load: DocUser in the signature, user.id() at both sites.
  spanHas(handler, "    user: DocUser,", "handler takes the authenticated DocUser");
  spanHas(handler, GATE_CALL, "admin is resolved from the server-side DocUser id");
  spanHas(handler, "let user_id = user.id().to_string();",
    "loader receives the same verified id the gate resolved");
  spanHas(handler, LOAD_CALL, "document is loaded with the verified id");
  h.assertEqual(countIn(handler, "resolve_admin(&state, user.id())"), 1,
    "exactly one admin resolution runs in the accept-all handler");

  // The only request-body fields consumed are the document id and the accept
  // flag — no client-supplied role, identity or header reaches the decision.
  spanHas(handler, "&req.doc_id", "request supplies only the target document id");
  spanHas(handler, "req.accept", "request supplies only the accept flag");
  h.assertEqual(countIn(handler, "req."), 2,
    "no other request-body field is consumed by the gated handler");
  ["req.role", "req.user_id", "req.admin", "req.is_admin", "HeaderMap", "headers",
    "role_claim", "claims", "e.to_string()"].forEach(function (token) {
    spanLacks(handler, token, "gated handler must not consume <" + token + ">");
  });

  // Both denial branches return before any document bytes are read. Removing
  // the gate, or moving it after the load, breaks these ordering assertions.
  sequence(handler, [
    GATE_CALL,
    "AdminAccess::Allowed => {}",
    "AdminAccess::NonAdmin => {",
    'return Err(admin_denial("insufficient-role"));',
    "AdminAccess::Unresolved => {",
    'return Err(admin_denial("no-identity"));',
    "let user_id = user.id().to_string();",
    LOAD_CALL
  ], "admin gate precedes both denials and the document load");
  ordered(handler, GATE_CALL, LOAD_CALL, "gate must run before any drive load");
  ordered(handler, 'return Err(admin_denial("no-identity"));', LOAD_CALL,
    "unresolved role must deny before any drive load");
});

// (b) Typed, stable, non-sensitive refusal — no document data or PII.
h.staticTest("#2448 track-changes admin refusal is a typed docs_acl_denied 403 with stable reasons and no document data", function () {
  var src = h.readRepo(TRACK);

  // Non-vacuity: the shared denial builder ships.
  contains(TRACK, "fn admin_denial(", "typed denial builder is shipped");
  var denial = between(src, "fn admin_denial(", "pub async fn handle_enable_track_changes(");

  spanHas(denial, "StatusCode::FORBIDDEN", "denial maps to 403");
  spanHas(denial, '"error": "Document not found or access denied"',
    "denial text is generic, never the document body or a DB error");
  spanHas(denial, '"code": "docs_acl_denied"', "denial carries the stable machine code");
  spanHas(denial, '"reason": reason,', "reason is a caller-supplied stable token, not interpolated data");
  h.assertEqual(countIn(denial, '"code":'), 1, "denial body has exactly one code field");
  h.assertEqual(countIn(denial, '"reason":'), 1, "denial body has exactly one reason field");

  // No document id/content/identity can appear in the refusal body.
  ["doc_id", "user_id", "content", "body", "owner", "email", "display_name",
    "format!", "log_and_sanitize", "e.to_string()"].forEach(function (token) {
    spanLacks(denial, token, "denial must not leak <" + token + ">");
  });

  // The two failure classes use fixed, stable reason tokens from the handler.
  var handler = between(src, ACCEPT_ALL, LIST_ALL);
  spanHas(handler, "AdminAccess::NonAdmin", "non-admin branch is present");
  spanHas(handler, 'admin_denial("insufficient-role")', "non-admin reason is stable");
  spanHas(handler, "AdminAccess::Unresolved", "unresolved branch is present");
  spanHas(handler, 'admin_denial("no-identity")', "unresolved reason is stable");
  h.assertEqual(countIn(handler, "admin_denial("), 2,
    "both failure classes use the shared typed denial builder");
  ordered(handler, "AdminAccess::NonAdmin", 'admin_denial("insufficient-role")',
    "non-admin must map to the insufficient-role denial");
  ordered(handler, "AdminAccess::Unresolved", 'admin_denial("no-identity")',
    "unresolved role must map to the no-identity denial");

  // The catalog-level gate on the chat path stays in lockstep with this HTTP
  // gate: the verb remains admin_only and uses the same fail-closed decision
  // table with fixed, non-sensitive messages (never a command or DB detail).
  contains(AUTH, "pub(super) fn authorize_admin_command(", "catalog admin gate is shipped");
  contains(AUTH, '"docs.track_changes.accept_all",', "track-changes verb stays admin_only in the catalog");
  var decision = between(h.readRepo(AUTH), "pub(super) fn authorize_admin_command(",
    "/// True when the catalog marks the named verb admin-only.");
  sequence(decision, [
    "if !admin_only {",
    "return Ok(());",
    "match role {",
    "Some(crate::security::user_role::ROLE_ADMIN) => Ok(()),",
    "Some(_) => Err(CommandAuthError::AdminRequired),",
    "None => Err(CommandAuthError::RoleUnresolved),"
  ], "catalog gate fails closed for non-admin and unresolved roles");
  contains(AUTH, '"command denied: admin role required"', "fixed admin-required message");
  contains(AUTH, '"command denied: caller role could not be resolved"',
    "fixed unresolved-role message");
});

// (c) Three deterministic unit tests: near misses, typed denial, fail-closed.
h.staticTest("#2448 three deterministic track-changes admin-gate tests ship, including near-miss names and unresolved-role fail-closed", function () {
  var src = h.readRepo(TRACK);

  // Non-vacuity: the compiled test module and its three tests really ship.
  contains(TRACK, "#[cfg(test)]", "test module is compiled in");
  contains(TRACK, "mod admin_gate_tests {", "admin gate test module is shipped");
  var tests = src.slice(src.indexOf("#[cfg(test)]"));
  h.assert(tests.length > 0, "test module span must be non-empty");
  h.assertEqual(countIn(tests, "#[test]"), 3, "exactly three deterministic tests ship");

  var near = between(tests, "fn admin_group_is_allowed_and_near_miss_is_not() {",
    "fn non_admin_gets_typed_denial() {");
  spanHas(near, "AdminAccess::from_resolved(Some(true))", "admin resolution is allowed");
  spanHas(near, "AdminAccess::Allowed", "admin keeps the success path (not a blanket deny)");
  spanHas(near, "for name in ADMIN_GROUP_NAMES {", "every mirrored admin name is checked");
  spanHas(near, 'assert!(is_admin_group_name(name), "admin name \'{name}\' must match");',
    "each canonical admin name matches");
  spanHas(near, 'for name in ["notadmin", "admintrainee", "admin-readonly", "user", ""] {',
    "near-miss admin names are enumerated");
  spanHas(near, "!is_admin_group_name(name)", "near misses must not grant admin");
  spanHas(near, '"near-miss name \'{name}\' must not grant admin"',
    "near-miss assertion is non-vacuous");

  // Full-string, case-insensitive matching is what keeps the near misses out.
  var names = between(src, "const ADMIN_GROUP_NAMES: &[&str] = &[",
    "#[derive(Debug, Clone, Copy, PartialEq, Eq)]");
  ["\"admin\"", "\"administrator\"", "\"superadmin\"", "\"super_admin\"",
    "\"super-admin\"", "\"super\""].forEach(function (name) {
    spanHas(names, name, "mirrored admin name <" + name + "> is present");
  });
  spanHas(names, "let candidate = name.trim();", "group names are trimmed before matching");
  spanHas(names, "candidate.eq_ignore_ascii_case(admin)", "matching is full-string and case-insensitive");

  var nonAdmin = between(tests, "fn non_admin_gets_typed_denial() {",
    "fn unresolved_role_fails_closed() {");
  sequence(nonAdmin, [
    "AdminAccess::from_resolved(Some(false))",
    "AdminAccess::NonAdmin",
    'admin_denial("insufficient-role")',
    "StatusCode::FORBIDDEN",
    'assert_eq!(body.0["code"], "docs_acl_denied");',
    'assert_eq!(body.0["reason"], "insufficient-role");',
    'assert!(body.0.get("doc_id").is_none());'
  ], "non-admin test pins the typed 403 and absence of a document id");

  var unresolved = tests.slice(tests.indexOf("fn unresolved_role_fails_closed() {"));
  sequence(unresolved, [
    "AdminAccess::from_resolved(None)",
    "AdminAccess::Unresolved",
    'admin_denial("no-identity")',
    "StatusCode::FORBIDDEN",
    'assert_eq!(body.0["code"], "docs_acl_denied");',
    'assert_eq!(body.0["reason"], "no-identity");'
  ], "unresolved-role test pins fail-closed denial");
});

h.note("FOLLOW-UP #2448: accept-all resolves admin from the DocUser id before any drive load and " +
  "refuses with a typed docs_acl_denied 403; these are source assertions only — no browser/server/cargo run.");
