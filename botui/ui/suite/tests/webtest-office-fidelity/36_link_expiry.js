/**
 * [STATIC] deterministic coverage for the #2452 share-link expiry fix.
 *
 * Every case reads real shipped source as text and FAILS if any part of the
 * guard disappears: no runtime claim, no server, no browser, no cargo. Each
 * case first proves the surrounding function/route/DTO exists (non-vacuity),
 * then asserts literal substrings and their ordering, so a truncated, empty or
 * garbled file can never satisfy a case by accident.
 *
 * Covered:
 *   (a) LinkBody.expires_at — RFC3339 parse, future validation, preference over
 *       expires_in_hours, and storage/return of the computed deadline.
 *   (b) Redemption — link_is_expired() checked before owner resolution and
 *       document load, returning the same generic 404 as an unknown token.
 *   (c) Docs adapter — datetime-local input, invalid/past validation with
 *       existing i18n keys, and expiry sent only when the user set it.
 *   (d) The two pure unit tests for the expired / still-redeemable boundary.
 */
"use strict";

var h = require("./00_harness");

var LINK_DTO = "botserver/src/main_module/routes/collab_routes/dto.rs";
var LINK_PERMISSIONS = "botserver/src/main_module/routes/collab_routes/permissions.rs";
var SHARED_LINK = "botserver/crates/botdocs/src/shared_link.rs";
var DOCS_READONLY = "botui/ui/suite/docs/modules/15_readonly.js";

var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  var src = h.readRepo(relPath);
  h.assert(src.indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath);
}

/** Literal substring assertion on an already-read source span. */
function spanHas(span, needle, msg) {
  h.assert(span.indexOf(needle) !== -1,
    (msg || "span must contain") + " <" + needle + "> :: " + span.slice(0, 200));
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

/** Every marker must be present, in this strict order (guards control flow). */
function ordered(span, markers, what) {
  var last = -1;
  markers.forEach(function (marker) {
    var at = span.indexOf(marker);
    h.assert(at !== -1, (what || "span") + " must contain <" + marker + ">");
    h.assert(at > last, (what || "span") + " order broken at <" + marker + "> :: " + at + " <= " + last);
    last = at;
  });
}

/** Count literal occurrences (used to require both reject sites). */
function occurrences(src, needle) {
  var count = 0;
  var at = src.indexOf(needle);
  while (at !== -1) {
    count++;
    at = src.indexOf(needle, at + needle.length);
  }
  return count;
}

/** Assert every token is absent from a span. */
function absent(span, tokens, what) {
  tokens.forEach(function (token) {
    h.assert(span.indexOf(token) === -1,
      (what || "span") + " must not contain <" + token + "> :: " + span.slice(0, 200));
  });
}

// (a) LinkBody accepts an explicit RFC3339 deadline. create_link parses it,
//     rejects malformed/past values with the shared validation error, and uses
//     the legacy hours field only when expires_at was not supplied.
h.staticTest("#2452 explicit expires_at is parsed, validated future and preferred over expires_in_hours", function () {
  contains(LINK_DTO, "pub struct LinkBody {", "LinkBody is shipped");
  contains(LINK_DTO, "pub expires_at: Option<String>,", "LinkBody accepts an explicit deadline");
  contains(LINK_PERMISSIONS, "pub async fn create_link(", "mint route is shipped");

  var span = between(LINK_PERMISSIONS,
    "let expires_at = match req.expires_at.as_deref() {",
    "let created_at = chrono::Utc::now();");

  ordered(span, [
    "let expires_at = match req.expires_at.as_deref() {",
    "Some(raw) => {",
    "chrono::DateTime::parse_from_rfc3339(raw)",
    "let deadline = parsed.with_timezone(&chrono::Utc);",
    "if deadline <= chrono::Utc::now() {",
    "None => req",
    ".expires_in_hours"
  ], "create_link expiry decision");

  h.assert(occurrences(span, '"error-validation-date-future"') >= 2,
    "both an unparseable and a past deadline must be rejected :: " + span);
  contains(LINK_PERMISSIONS, ".filter(|h| *h > 0)", "non-positive hours are ignored");
  contains(LINK_PERMISSIONS, ".map(|h| chrono::Utc::now() + chrono::Duration::hours(h)),",
    "legacy hours still produce a deadline");
  contains(LINK_PERMISSIONS, ".bind::<Nullable<Timestamptz>, _>(expires_at)",
    "deadline is persisted into resource_links");
  contains(LINK_PERMISSIONS, "expires_at: expires_at.map(|d| d.to_rfc3339()),",
    "deadline is returned to the caller");
});

// (b) Redemption refuses an expired link through the SAME generic 404 used for
//     an unknown/wrong-type token, before any owner lookup or document load,
//     so the caller gets no existence hint and no document body.
h.staticTest("#2452 expired links return the generic 404 before owner/document load with no existence hint", function () {
  contains(SHARED_LINK, "fn link_is_expired(", "pure expiry helper is shipped");
  contains(SHARED_LINK, "expires_at.is_some_and(|deadline| now >= deadline)",
    "deadline equality is already expired");
  contains(SHARED_LINK, "pub async fn handle_shared_document(", "redemption route is shipped");
  contains(SHARED_LINK, "fn not_found() -> (StatusCode, Json<serde_json::Value>) {",
    "generic not_found response is shipped");

  var handler = between(SHARED_LINK, "pub async fn handle_shared_document(", "async fn load_link(");
  ordered(handler, [
    "if link_is_expired(link.expires_at, chrono::Utc::now()) {",
    "resolve_owner(&state, &link)",
    "load_document_from_drive(&state, &owner_id, &link.resource_id)",
    "record_shared_view(&state, &link.resource_id)"
  ], "redemption order");
  var expiryBranch = handler.slice(
    handler.indexOf("if link_is_expired(link.expires_at, chrono::Utc::now()) {"),
    handler.indexOf("resolve_owner(&state, &link)")
  );
  contains(SHARED_LINK, "return Err(not_found());", "expired link returns generic 404");
  h.assert(expiryBranch.indexOf("return Err(not_found());") !== -1,
    "expired branch must return not_found() :: " + expiryBranch);
  absent(expiryBranch, ["internal(", "log::", "warn!"], "leaky expiry error handling");

  var lookup = between(SHARED_LINK, "async fn load_link(", "/// Owner namespace of the linked document.");
  contains(SHARED_LINK, "SELECT resource_type, resource_id, role, owner_id, expires_at",
    "expiry is selected with the link row");
  spanHas(lookup, "FROM resource_links WHERE token = $1", "token-only lookup is shipped");

  var generic = between(SHARED_LINK,
    "fn not_found() -> (StatusCode, Json<serde_json::Value>) {",
    "/// #2452 — the single redemption decision for link expiry.");
  spanHas(generic, "StatusCode::NOT_FOUND", "generic response is a 404");
  spanHas(generic, 'Json(serde_json::json!({ "error": "Link not found" }))',
    "generic response has one fixed message");
  absent(generic, ["token", "resource_id", "expires_at", "deadline", "document", "expired"],
    "existence hint in the generic 404 body");
});

// (c) The Docs adapter injects an optional datetime-local expiry into the
//     shared dialog: empty means permanent, invalid/past values are stopped
//     before the POST, and the two fields are added only for a chosen instant.
h.staticTest("#2452 Docs UI sends expiry only when set and validates invalid/past dates with existing i18n keys", function () {
  contains(DOCS_READONLY, 'input.type = "datetime-local";', "datetime-local input is shipped");
  contains(DOCS_READONLY, 'input.id = "gbs-docs-expiry";', "expiry input is installed in the dialog");
  contains(DOCS_READONLY, "function docsShareExpiryIso()", "input-to-RFC3339 conversion is shipped");
  contains(DOCS_READONLY, "return isFinite(when.getTime()) ? when.toISOString() : \"\";",
    "unparseable input yields no deadline");
  contains(DOCS_READONLY, 'el.setAttribute("role", "status");', "error region role=status");
  contains(DOCS_READONLY, 'el.setAttribute("aria-live", "polite");', "error region aria-live=polite");

  var click = between(DOCS_READONLY, "function onShareClick(event) {", "function linkFetch(input, init) {");
  ordered(click, [
    "if (!raw) return;",
    "var iso = docsShareExpiryIso();",
    "if (!iso) {",
    "Date.parse(iso) <= Date.now()",
    "docsShareExpiryPending = iso;"
  ], "onShareClick expiry validation");
  spanHas(click, "event.preventDefault();", "invalid date stops the POST");
  spanHas(click, "event.stopImmediatePropagation();", "dialog listener cannot mint the link");
  spanHas(click, 'docsT("error-invalid-value", "The supplied value could not be parsed.")',
    "invalid date uses the existing parse-error key");
  spanHas(click, 'docsT("error-validation-date-future", "{ $field } must be in the future.", {',
    "past date uses the existing future-validation key");

  var fetch = between(DOCS_READONLY, "function linkFetch(input, init) {", "shared.open = function (opts) {");
  ordered(fetch, [
    "if (!isLinkPost) return originalFetch(input, init);",
    "var iso = docsShareExpiryPending;",
    'if (iso && typeof init.body === "string")',
    "if (!body.expires_at) body.expires_at = iso;",
    "if (!body.expires_in_hours) {",
    "var hours = Math.ceil((Date.parse(iso) - Date.now()) / 3600000);"
  ], "linkFetch expiry injection");
  spanHas(fetch, "if (body && body.resource_type === \"docs\")", "only Docs links get the expiry");

  // Non-vacuity: all four keys are pre-existing catalog entries in all 8 locales.
  [["errors.ftl", "error-validation-date-future"],
   ["errors.ftl", "error-invalid-value"],
   ["ui-admin.ftl", "admin-roles-expiration-optional"],
   ["ui-admin.ftl", "admin-roles-expiration-help"]].forEach(function (entry) {
    LOCALES.forEach(function (loc) {
      contains("botlib/locales/" + loc + "/" + entry[0], entry[1] + " =",
        loc + " catalog defines " + entry[1]);
    });
  });
});

// (d) The pure expiry decision is unit-tested at both boundaries: past and
//     exactly-now are expired; a future deadline and an open-ended link are not.
h.staticTest("#2452 both expiry unit tests cover the past/boundary and future/open-ended cases", function () {
  contains(SHARED_LINK, "mod tests {", "unit test module is shipped");
  contains(SHARED_LINK, "use super::{is_valid_token, link_is_expired};", "helper imported by tests");

  var expired = between(SHARED_LINK,
    "fn expired_deadlines_are_refused_including_the_boundary() {",
    "fn future_and_open_ended_links_stay_redeemable() {");
  spanHas(expired, "assert!(link_is_expired(", "past deadline expires");
  spanHas(expired, "now - chrono::Duration::seconds(1)", "past case is actually past");
  spanHas(expired, "assert!(link_is_expired(Some(now), now));", "equality-at-deadline case");

  var src = h.readRepo(SHARED_LINK);
  var futureStart = src.indexOf("fn future_and_open_ended_links_stay_redeemable() {");
  h.assert(futureStart !== -1, "future/open-ended unit test is missing");
  var future = src.slice(futureStart);
  contains(SHARED_LINK, "assert!(!link_is_expired(", "future/open-ended link stays redeemable");
  h.assert(future.indexOf("assert!(!link_is_expired(") !== -1,
    "future case must live in the future test :: " + future.slice(0, 200));
  h.assert(future.indexOf("Some(now + chrono::Duration::hours(1))") !== -1,
    "future case uses an actual future deadline");
  h.assert(future.indexOf("assert!(!link_is_expired(None, now));") !== -1,
    "open-ended (None) link stays redeemable");

  h.assert(occurrences(src, "#[test]") >= 4, "both expiry tests must be annotated #[test]");
});
