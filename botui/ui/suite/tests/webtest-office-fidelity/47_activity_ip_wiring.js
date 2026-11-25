/**
 * #2452 — deterministic static coverage for the IP field newly wired onto the
 * LIVE `/api/activity` path (botserver collab_routes + botdocs export handlers):
 *   (a) the migration adds a nullable `collab_activity.ip VARCHAR(64)` and the
 *       DTO (`ActivityItem`) and Diesel row (`ActivityRow`) expose `ip` as
 *       Option<String>;
 *   (b) `GET /api/activity` SELECTs `ip` and maps it into each `ActivityItem`,
 *       and the shared INSERT is IP-aware with a legacy 6-column fallback so a
 *       row is never dropped on a deployment that predates the migration;
 *   (c) all six botdocs export handlers thread a server-resolved ConnectInfo IP
 *       into `record_export_access_with_ip`, and the two DTO serialization tests
 *       for a present / absent IP ship.
 *
 * All assertions are literal substrings of production source. Nothing is
 * executed: no server, no browser, no cargo, no network. NON-VACUITY: each case
 * first proves the real artifacts ship and that the extracted span is non-empty;
 * INSERT fallback checks use strict ordering, so deleting the legacy shape or
 * moving it ahead of the primary INSERT makes the case FAIL.
 */
"use strict";

var h = require("./00_harness");

var MIGRATION = "botserver/migrations/6.9.5-collab-activity-ip/up.sql";
var DTO = "botserver/src/main_module/routes/collab_routes/dto.rs";
var ACTIVITY = "botserver/src/main_module/routes/collab_routes/engagement/activity.rs";
var HELPERS = "botserver/src/main_module/routes/collab_routes/helpers.rs";
var EXPORT = "botserver/crates/botdocs/src/handlers_api/export.rs";

var FORMATS = ["pdf", "docx", "odt", "md", "html", "txt"];

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  h.assert(h.readRepo(relPath).indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath);
}

/** Count exact, literal occurrences of a needle in a file. */
function countIn(relPath, needle) {
  return h.readRepo(relPath).split(needle).length - 1;
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

/** Assert `before` occurs strictly earlier than `after` inside a source span. */
function ordered(span, before, after, msg) {
  var b = span.indexOf(before);
  var a = span.indexOf(after);
  h.assert(b !== -1 && a !== -1 && b < a,
    (msg || "ordering regressed") + " :: <" + before + "> before <" + after +
    "> in <" + span.slice(0, 160) + ">");
}

// (a) Migration column + DTO/row Option<String> exposure.
h.staticTest("#2452 nullable collab_activity.ip VARCHAR(64) migration and ActivityItem/ActivityRow Option<String> fields ship", function () {
  var mig = h.readRepo(MIGRATION);
  var dto = h.readRepo(DTO);

  // The migration really adds the nullable column (no NOT NULL clause).
  contains(MIGRATION, "ALTER TABLE collab_activity", "activity table migration is shipped");
  spanHas(mig, "ADD COLUMN IF NOT EXISTS ip VARCHAR(64);", "nullable ip column is added");
  h.assert(mig.indexOf("NOT NULL") === -1,
    "collab_activity.ip must be nullable (no NOT NULL in the migration)");
  spanHas(mig, "deliberately nullable", "nullable intent is documented");

  // The serializable item exposes `ip: Option<String>`; older rows serialize
  // as JSON null so the live timeline stays readable.
  var item = between(dto, "pub struct ActivityItem {", "pub(crate) struct ActivityRow {");
  spanHas(item, "pub ip: Option<String>,", "ActivityItem exposes ip as Option<String>");
  spanHas(item, "serializes as JSON `null`", "older rows without ip remain readable");

  // The Diesel row binds the same column as Nullable<Text>.
  var row = between(dto, "pub(crate) struct ActivityRow {", "Version history types");
  spanHas(row, "pub(crate) ip: Option<String>,", "ActivityRow exposes ip as Option<String>");
  spanHas(row, "#[diesel(sql_type = Nullable<Text>)]", "ip column is bound Nullable<Text>");
  ordered(row, "#[diesel(sql_type = Nullable<Text>)]", "pub(crate) ip: Option<String>,",
    "Nullable<Text> must annotate the ip field");
});

// (b) The live GET /api/activity path SELECTs and maps ip; the shared INSERT
//     is IP-aware with a legacy 6-column fallback (no row loss).
h.staticTest("#2452 live /api/activity SELECT+map ip and IP-aware INSERT keeps a legacy 6-column fallback", function () {
  var act = h.readRepo(ACTIVITY);
  var helpers = h.readRepo(HELPERS);

  // Non-vacuity: the live list handler, its row mapping and the shared writer.
  contains(ACTIVITY, "pub async fn list_activity(", "live activity list handler is shipped");
  contains(HELPERS, "pub(crate) fn record_activity_with_ip(", "IP-aware shared writer is shipped");
  contains(HELPERS, "pub(crate) fn record_activity(", "legacy writer wrapper is shipped");

  // The SELECT reads ip and each ActivityItem maps it through unchanged.
  spanHas(act, "SELECT id::text, actor_id, actor_name, action, payload, ip, created_at",
    "/api/activity SELECT includes ip");
  spanHas(act, "ip: r.ip,", "row ip is mapped into the ActivityItem");
  ordered(act, "SELECT id::text, actor_id, actor_name, action, payload, ip, created_at",
    "ip: r.ip,", "SELECT must precede the item mapping");

  // The POST side resolves the peer address from ConnectInfo only — the body
  // never carries one — and forwards it to the shared IP-aware writer.
  contains(ACTIVITY, "use axum::extract::ConnectInfo;", "ConnectInfo extractor is imported");
  spanHas(act, "ConnectInfo(addr): ConnectInfo<SocketAddr>,", "activity write receives ConnectInfo");
  spanHas(act, "let client_ip = addr.ip().to_string();", "server resolves the peer IP");
  spanHas(act, "record_activity_with_ip(", "write uses the IP-aware shared writer");
  spanHas(act, "Some(client_ip.as_str()),", "resolved IP is forwarded, not a body value");

  // Two INSERT shapes: the 7-column IP-aware primary and a 6-column legacy
  // fallback. They are scoped so each literal must match its own declaration.
  var withIp = between(helpers, "const ACTIVITY_INSERT_WITH_IP:", "const ACTIVITY_INSERT_WITHOUT_IP:");
  var withoutIp = between(helpers, "const ACTIVITY_INSERT_WITHOUT_IP:", "/// Append an audit-trail row");
  spanHas(withIp, "(resource_type, resource_id, actor_id, actor_name, action, payload, ip)",
    "primary INSERT includes the ip column");
  spanHas(withIp, "VALUES ($1, $2, $3, $4, $5, $6, $7)", "primary INSERT binds seven values");
  spanHas(withoutIp, "(resource_type, resource_id, actor_id, actor_name, action, payload)",
    "legacy INSERT keeps the original six columns");
  spanHas(withoutIp, 'VALUES ($1, $2, $3, $4, $5, $6)";', "legacy INSERT binds six values");

  // No row loss: the IP-aware insert is attempted first, and a failure retries
  // through the legacy shape rather than dropping the audit row.
  var writer = between(helpers, "pub(crate) fn record_activity_with_ip(", "/// Resolve the (resource_type");
  spanHas(writer, "ip: Option<&str>,", "writer accepts an optional IP");
  spanHas(writer, ".bind::<Nullable<Text>, _>(ip)", "writer binds ip as Nullable<Text>");
  spanHas(writer, "diesel::sql_query(ACTIVITY_INSERT_WITH_IP)", "writer attempts the IP-aware INSERT");
  spanHas(writer, "if let Err(e) = result {", "writer detects an IP-aware INSERT failure");
  spanHas(writer, "let fallback = diesel::sql_query(ACTIVITY_INSERT_WITHOUT_IP)",
    "writer retries the legacy INSERT on failure");
  ordered(writer, "diesel::sql_query(ACTIVITY_INSERT_WITH_IP)",
    "diesel::sql_query(ACTIVITY_INSERT_WITHOUT_IP)",
    "legacy fallback must run only after the IP-aware INSERT");
});

// (c) All six export handlers thread ConnectInfo IP into the audit, and the
//     two DTO serialization tests for present/absent IP exist.
h.staticTest("#2452 all six export handlers thread ConnectInfo IP into record_export_access_with_ip and both DTO IP serialization tests ship", function () {
  var src = h.readRepo(EXPORT);
  var dto = h.readRepo(DTO);

  // Non-vacuity: the extractor, address type and audit primitive are present.
  contains(EXPORT, "extract::{ConnectInfo, Query, State}", "ConnectInfo extractor is imported");
  contains(EXPORT, "use std::net::SocketAddr;", "SocketAddr type is imported");
  contains(EXPORT, "record_export_access_with_ip(", "IP-aware export audit is used");

  // Every one of the six handlers receives ConnectInfo, resolves the IP, and
  // forwards Some(client_ip) into the audit — a single reverted handler fails.
  h.assertEqual(countIn(EXPORT, "ConnectInfo(addr): ConnectInfo<SocketAddr>"), 6,
    "all six export handlers must receive ConnectInfo");
  h.assertEqual(countIn(EXPORT, "let client_ip = addr.ip().to_string();"), 6,
    "all six export handlers must resolve the client IP");
  h.assertEqual(countIn(EXPORT, "crate::access_audit::record_export_access_with_ip("), 6,
    "all six export handlers must record IP-aware access");
  h.assertEqual(countIn(EXPORT, "Some(client_ip.as_str())"), 6,
    "all six audit calls must forward Some(client_ip)");

  FORMATS.forEach(function (fmt) {
    var start = src.indexOf("pub async fn handle_export_" + fmt + "(");
    h.assert(start !== -1, "missing export handler <" + fmt + ">");
    var handler = src.slice(start, src.indexOf("pub async fn handle_export_", start + 1) === -1
      ? src.length
      : src.indexOf("pub async fn handle_export_", start + 1));
    h.assert(handler.length > 0, "empty handler span for <" + fmt + ">");
    spanHas(handler, "ConnectInfo(addr): ConnectInfo<SocketAddr>", fmt + " receives ConnectInfo");
    spanHas(handler, "let client_ip = addr.ip().to_string();", fmt + " resolves the IP");
    spanHas(handler, "crate::access_audit::record_export_access_with_ip(", fmt + " audits access");
    spanHas(handler, "Some(client_ip.as_str())", fmt + " forwards the IP");
    ordered(handler, "let client_ip = addr.ip().to_string();",
      "record_export_access_with_ip(", fmt + " must resolve IP before auditing");
  });

  // The two DTO serialization tests pin present -> address and absent -> null.
  var tests = dto.slice(dto.indexOf("#[cfg(test)]"));
  h.assert(tests.length > 0, "dto test module must be present");
  spanHas(tests, 'const TEST_IP: &str = "203.0.113.7";', "documentation-only test IP is defined");
  spanHas(tests, "fn activity_item_serializes_a_present_ip()", "present-IP test exists");
  spanHas(tests, "fn activity_item_serializes_an_absent_ip_as_null()", "absent-IP test exists");
  spanHas(tests, 'assert_eq!(json.get("ip").and_then(|v| v.as_str()), Some(TEST_IP));',
    "present-IP test asserts the exact serialized address");
  spanHas(tests, "assert!(json.get(\"ip\").is_some_and(serde_json::Value::is_null));",
    "absent-IP test asserts JSON null, not an omitted key");
});

h.note("FOLLOW-UP #2452: the live /api/activity timeline now carries a nullable " +
  "collab_activity.ip end-to-end; these are source assertions only — no browser/server/cargo run.");
