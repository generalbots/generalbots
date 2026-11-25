/**
 * #2452 — deterministic static coverage for the just-shipped shared-link view
 * IP audit hook (`record_shared_view_with_ip` fed from `ConnectInfo`).
 *
 * All assertions are literal substrings of production source. Nothing is
 * executed: no server, no browser, no cargo, no network. NON-VACUITY: each case
 * first proves the real artifacts ship (function/constant markers) and extracts
 * a non-empty source span between literal markers; ordering and absence checks
 * are strict. The cases FAIL if the hook is reverted to the old
 * `record_shared_view` call or if the address is sourced from any request
 * header instead of the accepted socket.
 */
"use strict";

var h = require("./00_harness");

var SHARED = "botserver/crates/botdocs/src/shared_link.rs";
var AUDIT = "botserver/crates/botdocs/src/access_audit.rs";

var HOOK = "crate::access_audit::record_shared_view_with_ip(&state, &link.resource_id, ip.as_deref());";
var IP_ASSIGN = "let ip = client_ip(Some(&addr));";
var CONNECT = "ConnectInfo(addr): ConnectInfo<SocketAddr>,";
var TEST_IP_DECL = "const TEST_IP: &str = \"203.0.113.7\";";

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}

/** Absence assertion — a forbidden token must not appear in the given span. */
function absent(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1,
    (msg || "source must not contain") + " <" + needle + "> :: " + src.slice(0, 200));
}

/** Exact, non-empty source span between two literal markers (end exclusive). */
function span(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  h.assert(end > start, "empty source span between <" + startMarker + "> and <" + endMarker + ">");
  return src.slice(start, end);
}

/** Assert `before` occurs strictly earlier than `after` inside a source span. */
function ordered(src, before, after, msg) {
  var b = src.indexOf(before);
  var a = src.indexOf(after);
  h.assert(b !== -1 && a !== -1 && b < a,
    (msg || "ordering regressed") + " :: <" + before + "> before <" + after + ">");
}

/** Count exact, literal occurrences of a needle in a source string. */
function countIn(src, needle) {
  return src.split(needle).length - 1;
}

h.staticTest("#2452 shared-link client IP comes only from ConnectInfo, never a request header", function () {
  var src = h.readRepo(SHARED);

  // Non-vacuity: extractor, socket type, handler and IP helper all ship.
  contains(src, "use axum::extract::{ConnectInfo, Path, State};", "ConnectInfo extractor is imported");
  contains(src, "use std::net::SocketAddr;", "SocketAddr type is imported");
  contains(src, "pub async fn handle_shared_document(", "shared-link handler is shipped");
  contains(src, "fn client_ip(", "client_ip helper is shipped");

  // client_ip accepts only the peer socket and keeps the IP, never the port.
  var helper = span(src, "fn client_ip(", "/// `GET /api/docs/shared/:token`");
  contains(helper, "fn client_ip(addr: Option<&SocketAddr>) -> Option<String> {",
    "client_ip accepts only an optional socket address");
  contains(helper, "addr.map(|socket| socket.ip().to_string())",
    "client_ip derives the address from the socket and drops the port");
  ["HeaderMap", ".headers()", "headers", "x-forwarded-for", "X-Forwarded-For", "header("]
    .forEach(function (forbidden) {
      absent(helper, forbidden, "client_ip must not read request headers");
    });

  // The handler takes ConnectInfo and derives the IP from that socket alone.
  var handler = span(src, "pub async fn handle_shared_document(", "async fn load_link(");
  contains(handler, CONNECT, "handler receives the peer address from ConnectInfo");
  contains(handler, IP_ASSIGN, "handler derives the audit IP from the connected socket");
  ["HeaderMap", ".headers()", "x-forwarded-for", "X-Forwarded-For", "header("]
    .forEach(function (forbidden) {
      absent(handler, forbidden, "handler view IP must not come from a request header");
    });
});

h.staticTest("#2452 IP-aware view hook runs after load, before response; viewer gate and not_found fail closed", function () {
  var src = h.readRepo(SHARED);
  var audit = h.readRepo(AUDIT);

  // Non-vacuity: handler, role gate, body gate, not_found and audit hook ship.
  contains(src, "pub async fn handle_shared_document(", "shared-link handler is shipped");
  contains(src, "fn role_allows_source(", "role capability gate is shipped");
  contains(src, "fn body_for_link<'a>(", "body gate is shipped");
  contains(src, "fn not_found()", "generic not_found helper is shipped");
  contains(audit, "pub fn record_shared_view_with_ip(", "IP-aware view recorder ships");

  // Fail-closed gates remain, independent of the audit hook.
  var roles = span(src, "fn role_allows_source(", "/// The single decision for releasing source bytes");
  contains(roles, "\"commenter\" | \"editor\" | \"owner\"", "only source-capable roles are allowed");
  absent(roles, "\"viewer\"", "viewer must not be in the source-capable role set");

  var body = span(src, "fn body_for_link<'a>(", "/// Unknown, expired, revoked");
  contains(body, "role_allows_source(role)", "body gate applies the role capability");
  contains(body, "is_valid_token(token)", "body gate checks the token shape");
  contains(body, "if allowed { source } else { \"\" }", "disallowed roles get an empty body");
  var nf = span(src, "fn not_found()", "/// #2452 — the single redemption decision");
  contains(nf, "StatusCode::NOT_FOUND", "not_found returns 404");
  contains(nf, "\"Link not found\"", "not_found uses the generic message");

  // The IP-aware view hook is recorded only after a successful document load,
  // from the socket address, and strictly before the JSON response.
  var handler = span(src, "pub async fn handle_shared_document(", "async fn load_link(");
  contains(handler, IP_ASSIGN, "socket IP is captured in the handler");
  contains(handler, HOOK, "IP-aware view hook is present with ip.as_deref()");
  ordered(handler, "let document = load_document_from_drive(", IP_ASSIGN,
    "IP capture must follow a successful document load");
  ordered(handler, IP_ASSIGN, HOOK, "view hook must follow IP capture");
  ordered(handler, HOOK, "Ok(Json(SharedDocument {", "view hook must precede the response");
  contains(handler, "if !is_valid_token(&token)", "malformed token is rejected before any audit");
  contains(handler, "link_is_expired(link.expires_at, chrono::Utc::now())",
    "expired link is rejected before any audit");
  absent(handler, "record_shared_view(", "the old non-IP view hook must not be used");

  // The recorder builds an anonymous view event and binds the IP as nullable
  // text; the legacy wrapper only forwards None for older callers.
  var recorder = span(audit, "pub fn record_shared_view_with_ip(",
    "/// Call only after the export bytes have actually been produced.");
  contains(recorder, "AccessAuditEvent::shared_view(resource_id).with_ip(ip)",
    "recorder attaches the caller IP to a shared-view event");
  var append = span(audit, "fn append(conn: &mut PgConnection, event: &AccessAuditEvent) {",
    "fn record(state: &DocState, event: AccessAuditEvent) {");
  contains(append, ".bind::<Nullable<Text>, _>(event.ip.as_deref())", "IP is bound as Nullable<Text>");
  contains(audit, "record_shared_view_with_ip(state, resource_id, None);",
    "legacy wrapper stays additive and delegates to the IP-aware recorder");
});

h.staticTest("#2452 three deterministic shared-view IP tests ship: TEST-NET socket, None socket, anonymous content-free event", function () {
  var src = h.readRepo(SHARED);
  var start = src.indexOf("#[cfg(test)]");
  h.assert(start !== -1, "shared_link test module must ship");
  var tests = src.slice(start);
  h.assert(tests.length > 0, "test module span must be non-empty");

  contains(tests, TEST_IP_DECL, "RFC 5737 TEST-NET documentation address is pinned");
  contains(tests, "fn view_ip_comes_from_the_connected_socket()", "socket-IP test exists");
  contains(tests, "fn missing_socket_address_stays_none()", "None-socket test exists");
  contains(tests, "fn view_event_with_ip_stays_anonymous_and_content_free()",
    "anonymous content-free event test exists");
  h.assertEqual(countIn(tests, "fn view_ip_comes_from_the_connected_socket()"), 1,
    "socket-IP test must be declared exactly once");
  h.assertEqual(countIn(tests, "fn missing_socket_address_stays_none()"), 1,
    "None-socket test must be declared exactly once");
  h.assertEqual(countIn(tests, "fn view_event_with_ip_stays_anonymous_and_content_free()"), 1,
    "anonymous event test must be declared exactly once");

  contains(tests, "assert_eq!(ip.as_deref(), Some(TEST_IP));", "socket test asserts the exact peer address");
  contains(tests, "assert!(ip.is_some_and(|value| !value.contains(':')))", "port is excluded from the IP");
  contains(tests, "assert_eq!(client_ip(None), None);", "missing socket stays None, no fabricated IP");
  contains(tests, "AccessAuditEvent::shared_view(\"doc-1\").with_ip(ip.as_deref())",
    "event test attaches the socket IP");
  contains(tests, "assert_eq!(event.resource_type, \"docs\");", "event stays docs-scoped");
  contains(tests, "assert_eq!(event.actor_id, \"anonymous-guest\");", "event stays anonymous");
  contains(tests, "assert_eq!(event.action, \"view\");", "event action stays view");
  contains(tests, "assert!(event.payload.as_object().is_some_and(|map| map.len() == 1));",
    "payload keeps exactly one key");
  contains(tests, "assert!(event.payload.get(\"token\").is_none());", "no capability token in the payload");
  contains(tests, "assert!(event.payload.get(\"content\").is_none());", "no document body in the payload");
  contains(tests, "assert_eq!(event.ip.as_deref(), Some(TEST_IP));", "event carries the socket IP");
});

h.note("FOLLOW-UP #2452: the shared-link view audit hook takes its IP only from ConnectInfo; " +
  "these are literal source assertions only — no browser/server/cargo run.");
