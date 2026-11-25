/**
 * #2452 — deterministic static coverage for the shipped org-wide link
 * inventory endpoint (`GET /api/collab/links/all`).
 *
 * All assertions are literal substrings of production source. Nothing is
 * executed: no server, no browser, no cargo, no network. NON-VACUITY: each case
 * first proves the real artifacts ship (the route table, the module's doc
 * header, the scope enum/function, the two SQL statements, the locale catalog),
 * then extracts non-empty spans between literal markers; ordering and absence
 * checks are strict. The cases FAIL if the static route is moved after
 * `/links/:token`, if an org id becomes a request parameter, if the
 * user_organizations join or the fail-closed 403 is removed, or if either of
 * the two deterministic Rust tests is deleted.
 */
"use strict";

var h = require("./00_harness");

var MOD = "botserver/src/main_module/routes/collab_routes/mod.rs";
var INVENTORY = "botserver/src/main_module/routes/collab_routes/link_inventory.rs";
var PERMISSIONS = "botserver/src/main_module/routes/collab_routes/permissions.rs";
var EN_ERRORS = "botlib/locales/en/errors.ftl";

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

h.staticTest("#2452 GET /api/collab/links/all is registered before /links/:token and takes no request-supplied org", function () {
  var mod = h.readRepo(MOD);
  var inv = h.readRepo(INVENTORY);

  // Non-vacuity: the real route table and the #2452 inventory module ship.
  contains(mod, "pub fn configure_collab_routes() -> Router<Arc<AppState>> {", "collab route table ships");
  contains(inv, "//! #2452 — org-wide share-link inventory.", "inventory module doc header ships");
  contains(inv, "pub async fn list_all_links(", "inventory handler ships");

  // The static inventory route is registered strictly before the token route;
  // axum matches in insertion order, so "all" must not be parsed as a token.
  var table = span(mod, ".route(\"/api/collab/links\", get(list_links).post(create_link))",
    "\"/api/collab/presence\",");
  contains(table, ".route(\"/api/collab/links/all\", get(list_all_links))",
    "static inventory route is registered");
  contains(table, ".route(\"/api/collab/links/:token\", delete(revoke_link))",
    "per-token revoke route is registered");
  ordered(table, ".route(\"/api/collab/links/all\", get(list_all_links))",
    ".route(\"/api/collab/links/:token\", delete(revoke_link))",
    "inventory route must precede the /:token route");
  contains(table, "axum matches paths in insertion", "registration-order rationale is documented");
  contains(table, "the static inventory path must precede `/:token`",
    "ordering constraint is documented next to the routes");

  // The handler signature has no Query/Json body or org parameter: only the
  // verified AuthenticatedUser extension and the locale are extracted.
  var sig = span(inv, "pub async fn list_all_links(", "    let locale = request_locale.locale();");
  contains(sig, "Extension(user): Extension<AuthenticatedUser>,", "handler takes the authenticated principal");
  contains(sig, "request_locale: botcore::i18n::RequestLocale,", "handler takes only the locale extractor");
  absent(sig, "Query<", "handler must not accept a query-string org parameter");
  absent(sig, "org_id", "handler signature must not accept a client-supplied org");
  absent(sig, "LinkBody", "handler must not accept a request body");

  // Scope is projected exclusively from verified claims, not request input.
  var handler = span(inv, "    let locale = request_locale.locale();", "#[cfg(test)]");
  contains(handler, "let created_by = collab_user_id(&user);", "caller identity comes from the principal");
  contains(handler, "let is_admin = user.is_admin() || user.is_super_admin();",
    "admin flags come from the principal");
  contains(handler, "link_inventory_scope(is_admin, user.organization_id, &created_by)",
    "scope uses the authenticated org id");
  absent(handler, "Query<", "handler body must not read a query-string org");
});

h.staticTest("#2452 admin+org joins user_organizations; ordinary callers see only own links; admin without org fails closed 403", function () {
  var inv = h.readRepo(INVENTORY);
  var permissions = h.readRepo(PERMISSIONS);
  var errors = h.readRepo(EN_ERRORS);

  // Non-vacuity: scope enum/function and both SQL statements ship.
  contains(inv, "pub(crate) enum LinkInventoryScope<'a> {", "scope enum ships");
  contains(inv, "pub(crate) fn link_inventory_scope(", "pure scope projection ships");
  contains(inv, "const ORGANIZATION_LINKS_SQL: &str =", "organization SQL ships");
  contains(inv, "const OWN_LINKS_SQL: &str =", "own-links SQL ships");
  contains(inv, "#[derive(Debug, Clone, PartialEq, Eq)]", "scope enum is deterministically comparable");

  // Two explicit variants: org-wide for admins, own-links for everyone else.
  var variants = span(inv, "pub(crate) enum LinkInventoryScope<'a> {", "/// Pure scope projection.");
  contains(variants, "Organization { org_id: uuid::Uuid },", "organization variant ships");
  contains(variants, "Own { created_by: &'a str },", "own-links variant ships");
  contains(inv, "/// The scope an inventory request is allowed to use. `None` means the caller",
    "fail-closed semantics are documented");
  contains(inv, "the handler must fail closed with 403.", "403 fail-closed behaviour is documented");

  var scope = span(inv, "pub(crate) fn link_inventory_scope(", "/// `GET /api/collab/links/all`");
  contains(scope, "if is_admin {", "admin branch ships");
  contains(scope, "Some(org_id) if org_id != uuid::Uuid::nil() => {", "admin needs a real, non-nil org");
  contains(scope, "Some(LinkInventoryScope::Organization { org_id })", "admin+org gets organization scope");
  contains(scope, "_ => None,", "admin without a usable org is denied");
  contains(scope, "Some(LinkInventoryScope::Own { created_by })", "non-admin is restricted to own links");
  ordered(scope, "if is_admin {", "Some(LinkInventoryScope::Organization { org_id })",
    "admin branch must grant organization scope");
  ordered(scope, "Some(LinkInventoryScope::Organization { org_id })", "_ => None,",
    "missing org must fall through to denial");

  // The org view is membership-verified in SQL and bound from the claims only.
  var orgSql = span(inv, "const ORGANIZATION_LINKS_SQL: &str =", "/// Ordinary scope:");
  contains(orgSql, "EXISTS (", "org view uses an existence check over membership");
  contains(orgSql, "JOIN user_organizations uo ON uo.user_id = u.id", "org view joins user_organizations");
  contains(orgSql, "uo.org_id = $1", "org id is bound from the verified principal");
  contains(orgSql, "l.created_by = $2", "caller identity is bound from the principal");
  ordered(orgSql, "JOIN user_organizations uo ON uo.user_id = u.id", "uo.org_id = $1",
    "membership join must precede the org predicate");
  absent(orgSql, "$3", "no third (client-supplied org) bind is allowed");

  // The ordinary view has no org predicate at all.
  var ownSql = span(inv, "const OWN_LINKS_SQL: &str =", "/// The scope an inventory request");
  contains(ownSql, "WHERE created_by = $1 ORDER BY created_at DESC", "own view filters on the caller only");
  absent(ownSql, "user_organizations", "non-admin view must not join organizations");
  absent(ownSql, "org_id", "non-admin view must not use an org predicate");

  // A scope-less caller gets the already-catalogued 403 error key, not a new one.
  var handler = span(inv, "pub async fn list_all_links(", "#[cfg(test)]");
  contains(handler, "let Some(scope) = link_inventory_scope(is_admin, user.organization_id, &created_by) else {",
    "handler fails closed when the scope is absent");
  contains(handler, "StatusCode::FORBIDDEN,", "scope-less caller receives 403");
  contains(handler, "\"error-collab-owner-only\"", "403 reuses the existing collab error key");
  ordered(handler, "link_inventory_scope(is_admin, user.organization_id, &created_by)",
    "StatusCode::FORBIDDEN,", "scope projection must precede the 403");
  contains(permissions, "StatusCode::FORBIDDEN, \"error-collab-owner-only\"",
    "the same 403 key is used by the existing collab handlers");
  contains(errors, "error-collab-owner-only = Only the owner can do this.",
    "the 403 key is present in the English catalog");
});

h.staticTest("#2452 two deterministic Rust tests pin claim-only org scope, the own-links branch and no extra SQL bind", function () {
  var inv = h.readRepo(INVENTORY);
  contains(inv, "#[cfg(test)]", "inventory test module ships");
  var tests = inv.slice(inv.indexOf("#[cfg(test)]"));
  h.assert(tests.length > 0, "inventory test module span must be non-empty");

  h.assertEqual(countIn(tests, "#[test]"), 2, "exactly two deterministic tests ship");
  contains(tests, "fn admin_scope_uses_the_authenticated_org_and_fails_closed_without_it()",
    "admin-scope fail-closed test ships");
  contains(tests, "fn ordinary_callers_only_see_links_they_minted()",
    "ordinary-caller scoping test ships");

  // No client-supplied org: the org id is a claim, bound once as $1; no $3.
  contains(tests, "// The org id is bound from the principal, never from a request field.",
    "test documents claim-only org provenance");
  contains(tests, "assert!(ORGANIZATION_LINKS_SQL.contains(\"uo.org_id = $1\"));",
    "test pins the org bind to the principal");
  contains(tests, "assert!(ORGANIZATION_LINKS_SQL.contains(\"l.created_by = $2\"));",
    "test pins the caller bind to the principal");
  contains(tests, "assert!(ORGANIZATION_LINKS_SQL.contains(\"JOIN user_organizations\"));",
    "test pins the membership join");
  contains(tests, "assert!(!ORGANIZATION_LINKS_SQL.contains(\"$3\"));",
    "test forbids an extra client-supplied org bind");
  h.assertEqual(countIn(tests, "assert!(!ORGANIZATION_LINKS_SQL.contains(\"$3\"));"), 1,
    "no-extra-bind assertion must be declared exactly once");

  // Admin with nil/missing org fails closed; non-admin always gets own scope.
  contains(tests, "link_inventory_scope(true, Some(org), \"alice@example.test\"),",
    "test grants admin+org organization scope");
  contains(tests, "assert_eq!(link_inventory_scope(true, None, \"alice@example.test\"), None);",
    "test denies admin without an org");
  contains(tests, "link_inventory_scope(true, Some(uuid::Uuid::nil()), \"alice@example.test\"),",
    "test denies admin with a nil org");
  contains(tests, "assert!(OWN_LINKS_SQL.contains(\"WHERE created_by = $1\"));",
    "test pins the own-links predicate");
  contains(tests, "assert!(!OWN_LINKS_SQL.contains(\"user_organizations\"));",
    "test forbids an org join for ordinary callers");
  contains(tests, "assert!(!OWN_LINKS_SQL.contains(\"org_id\"));",
    "test forbids an org predicate for ordinary callers");

  // Both branches project the same existing LinkRow/LinkItem shape.
  contains(tests, "for sql in [ORGANIZATION_LINKS_SQL, OWN_LINKS_SQL] {",
    "test checks both SQL branches share the DTO shape");
  contains(tests, "\"SELECT token, role, expires_at, created_at, allow_download FROM resource_links\"",
    "both branches select the existing link columns");
});

h.note("FOLLOW-UP #2452: static coverage for the org-wide link inventory — static route registered " +
  "before /links/:token, scope derived only from AuthenticatedUser claims, user_organizations join " +
  "for admins with an org, fail-closed 403 for admins without one, and two deterministic Rust tests; " +
  "literal source assertions only, no browser/server/cargo run.");
