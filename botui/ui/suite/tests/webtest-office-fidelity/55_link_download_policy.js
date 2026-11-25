/**
 * #2452 — deterministic static coverage for the shipped share-link
 * `allow_download` policy (migration 6.9.6 + DTO + mint/list + read gate).
 *
 * All assertions are literal substrings of production source. Nothing is
 * executed: no server, no browser, no cargo, no network. NON-VACUITY: each case
 * first proves the real artifacts ship (functions/structs/migration markers)
 * and extracts a non-empty source span between literal markers; ordering and
 * absence checks are strict. The cases FAIL if the download gate is removed
 * from `body_for_link`, if the mint stops persisting the flag, if the list stops
 * exposing it, or if any of the five deterministic Rust tests is deleted.
 */
"use strict";

var h = require("./00_harness");

var MIGRATION = "botserver/migrations/6.9.6-resource-links-download-policy/up.sql";
var DTO = "botserver/src/main_module/routes/collab_routes/dto.rs";
var DTO_TESTS = "botserver/src/main_module/routes/collab_routes/dto/dto_tests.rs";
var PERMISSIONS = "botserver/src/main_module/routes/collab_routes/permissions.rs";
var SHARED = "botserver/crates/botdocs/src/shared_link.rs";

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

h.staticTest("#2452 migration adds nullable allow_download BOOLEAN DEFAULT TRUE; DTO accepts Option<bool> with serde default and absent -> allowed", function () {
  var sql = h.readRepo(MIGRATION);
  var dto = h.readRepo(DTO);
  var tests = h.readRepo(DTO_TESTS);

  // Non-vacuity: the migration, request DTO and its two unit tests all ship.
  contains(sql, "ALTER TABLE resource_links", "download-policy migration ships");
  contains(dto, "pub struct LinkBody {", "LinkBody request DTO ships");
  contains(dto, "impl LinkBody {", "LinkBody helper impl ships");
  contains(tests, "fn link_body_without_a_download_policy_keeps_legacy_behaviour()",
    "request-side legacy-behaviour test ships");

  // The column is nullable with a TRUE default and is never forced NOT NULL.
  var alter = span(sql, "ALTER TABLE resource_links", "COMMENT ON COLUMN resource_links.allow_download IS");
  contains(alter, "ADD COLUMN IF NOT EXISTS allow_download BOOLEAN DEFAULT TRUE;",
    "allow_download is a nullable boolean defaulting to TRUE");
  absent(sql, "NOT NULL", "migration must not add a NOT NULL constraint");
  contains(sql, "When FALSE, a shared-link read never emits editable source bytes.",
    "migration documents the deny-on-FALSE semantics");
  contains(sql, "NULL or TRUE keeps the legacy allow-download behaviour.",
    "migration records the backward-compatible NULL/TRUE rule");

  // The request field is optional, serde-defaulted, and absent means allowed.
  var body = span(dto, "pub struct LinkBody {", "impl LinkBody {");
  contains(body, "#[serde(default)]\n    pub allow_download: Option<bool>,",
    "LinkBody accepts an optional, serde-defaulted allow_download");
  var helper = span(dto, "impl LinkBody {", "pub struct PermissionItem {");
  contains(helper, "pub(crate) fn downloads_allowed(&self) -> bool {",
    "downloads_allowed helper ships on the request DTO");
  contains(helper, "self.allow_download.unwrap_or(true)",
    "an absent request policy resolves to allowed");
  absent(helper, "unwrap_or(false)", "an absent policy must never resolve to denied");

  // Both request-side tests are declared exactly once with their assertions.
  h.assertEqual(countIn(tests, "fn link_body_without_a_download_policy_keeps_legacy_behaviour()"), 1,
    "legacy-behaviour request test must be declared exactly once");
  h.assertEqual(countIn(tests, "fn link_body_with_downloads_disabled_is_recorded_explicitly()"), 1,
    "explicit-disable request test must be declared exactly once");
  contains(tests, "assert_eq!(body.allow_download, None);", "absent request policy stays None");
  contains(tests, "assert!(body.downloads_allowed());", "absent request policy is allowed");
  contains(tests, "assert_eq!(disabled.allow_download, Some(false));", "explicit FALSE is preserved");
  contains(tests, "assert!(!disabled.downloads_allowed());", "explicit FALSE denies downloads");
  contains(tests, "assert_eq!(enabled.allow_download, Some(true));", "explicit TRUE is preserved");
  contains(tests, "assert!(enabled.downloads_allowed());", "explicit TRUE allows downloads");
});

h.staticTest("#2452 mint persists the flag and list exposes it; DTO serializes it as nullable", function () {
  var permissions = h.readRepo(PERMISSIONS);
  var dto = h.readRepo(DTO);

  // Non-vacuity: mint, list and the response/row DTO types all ship.
  contains(permissions, "pub async fn create_link(", "link mint handler ships");
  contains(permissions, "pub async fn list_links(", "link list handler ships");
  contains(dto, "pub struct LinkItem {", "link response DTO ships");
  contains(dto, "pub(crate) struct LinkRow {", "link query row ships");

  // The mint resolves the policy once, writes it, then echoes it back.
  var create = span(permissions, "pub async fn create_link(",
    "/// `GET /api/collab/links?resource_type=&resource_id=` — list active links.");
  contains(create, "let allow_download = req.downloads_allowed();",
    "mint resolves the request policy with the legacy-aware helper");
  contains(create, "created_by, owner_id, ", "mint INSERT names the owner_id column before the policy");
  contains(create, "allow_download)", "mint INSERT includes the allow_download column");
  contains(create, "VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    "mint INSERT binds eight values including the policy");
  contains(create, ".bind::<Bool, _>(allow_download)", "mint binds the resolved policy as a boolean");
  contains(create, "allow_download: Some(allow_download),", "mint response exposes the stored policy");
  ordered(create, "let allow_download = req.downloads_allowed();",
    ".bind::<Bool, _>(allow_download)", "policy must be resolved before it is persisted");
  ordered(create, ".bind::<Bool, _>(allow_download)", "Ok(Json(LinkItem {",
    "policy must be persisted before the response is built");
  absent(create, ".bind::<Bool, _>(!allow_download)", "mint must not invert the requested policy");
  absent(create, ".bind::<Bool, _>(true)", "mint must not hard-code downloads as allowed");

  // The list selects and forwards the raw nullable flag for legacy rows.
  var list = span(permissions, "pub async fn list_links(",
    "/// `DELETE /api/collab/links/:token` — revoke a link (owner of its resource");
  contains(list, "SELECT token, role, expires_at, created_at, allow_download FROM resource_links",
    "list SELECT reads the persisted policy");
  contains(list, "allow_download: r.allow_download,", "list forwards the policy without rewriting it");
  ordered(list, "SELECT token, role, expires_at, created_at, allow_download FROM resource_links",
    "allow_download: r.allow_download,", "list must read the column before mapping it");

  // Response and query rows model the column as nullable (legacy NULL allowed).
  var item = span(dto, "pub struct LinkItem {", "pub(crate) struct PermissionRow {");
  contains(item, "pub allow_download: Option<bool>,", "LinkItem serializes a nullable policy");
  var row = span(dto, "pub(crate) struct LinkRow {", "pub(crate) fn valid_editable_role(");
  contains(row, "#[diesel(sql_type = Nullable<Bool>)]", "LinkRow treats the column as nullable");
  contains(row, "pub(crate) allow_download: Option<bool>,", "LinkRow reads the nullable policy");
});

h.staticTest("#2452 body_for_link gates source on allow_download.unwrap_or(true) alongside token/expiry/role; five deterministic tests ship", function () {
  var src = h.readRepo(SHARED);
  var dtoTests = h.readRepo(DTO_TESTS);

  // Non-vacuity: the gate, the role set, the handler and the test module ship.
  contains(src, "fn body_for_link<'a>(", "source-body gate ships");
  contains(src, "fn role_allows_source(", "role capability gate ships");
  contains(src, "pub async fn handle_shared_document(", "shared-link handler ships");
  contains(src, "#[cfg(test)]", "shared_link test module ships");
  var tests = src.slice(src.indexOf("#[cfg(test)]"));
  h.assert(tests.length > 0, "shared_link test module span must be non-empty");

  // Every gate is part of ONE conjunction that feeds the body field only.
  var body = span(src, "fn body_for_link<'a>(",
    "/// Unknown, expired, revoked and wrong-type links are indistinguishable:");
  contains(body, "allow_download: Option<bool>,", "body gate takes the nullable link policy");
  contains(body, "is_valid_token(token)", "body gate checks the token shape");
  contains(body, "!link_is_expired(expires_at, now)", "body gate checks expiry");
  contains(body, "allow_download.unwrap_or(true)", "body gate denies only explicit FALSE");
  contains(body, "role_allows_source(role)", "body gate checks source-capable roles");
  contains(body, "if allowed { source } else { \"\" }", "a disallowed link yields an empty body");
  ordered(body, "is_valid_token(token)", "!link_is_expired(expires_at, now)",
    "token check must precede expiry check");
  ordered(body, "!link_is_expired(expires_at, now)", "allow_download.unwrap_or(true)",
    "expiry check must precede the download policy");
  ordered(body, "allow_download.unwrap_or(true)", "role_allows_source(role)",
    "download policy must gate before the role capability");
  absent(body, "unwrap_or(false)", "legacy NULL must never deny the body");
  absent(body, ".unwrap()", "nullable policy must not be force-unwrapped");

  // The read path must actually load and pass the stored flag to the gate.
  contains(src, "expires_at, allow_download", "load_link SELECT reads the policy column");
  contains(src, "link.allow_download,", "handler passes the stored policy into body_for_link");

  // Five deterministic tests: two request-side, three read-side.
  h.assertEqual(countIn(dtoTests, "fn link_body_without_a_download_policy_keeps_legacy_behaviour()"), 1,
    "request legacy-behaviour test must be declared exactly once");
  h.assertEqual(countIn(dtoTests, "fn link_body_with_downloads_disabled_is_recorded_explicitly()"), 1,
    "request explicit-disable test must be declared exactly once");
  var readTests = [
    "fn download_disabled_commenter_link_releases_no_source()",
    "fn download_disabled_viewer_link_releases_no_source()",
    "fn absent_or_enabled_download_policy_keeps_role_behaviour()",
  ];
  readTests.forEach(function (name) {
    h.assertEqual(countIn(tests, name), 1, "read-side test <" + name + "> must be declared exactly once");
    contains(tests, name, "read-side policy test <" + name + "> ships");
  });
  contains(tests, "body_for_link(&token, role, Some(false), None, now, SOURCE)",
    "download-disabled commenter/editor get an empty body");
  contains(tests, "body_for_link(&token, \"viewer\", Some(false), None, now, SOURCE)",
    "download-disabled viewer stays empty");
  contains(tests, "for policy in [None, Some(true)]",
    "legacy NULL and explicit TRUE keep the prior role behaviour");
});

h.note("FOLLOW-UP #2452: share links persist and honor `allow_download` — an explicit FALSE " +
  "suppresses source bytes while legacy NULL/TRUE keep prior behaviour; literal source assertions " +
  "only, no browser/server/cargo run.");
