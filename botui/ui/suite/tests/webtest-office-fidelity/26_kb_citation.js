/**
 * [STATIC] deterministic guards for the citation-only KB search payload
 * (issues #2457 / #2499). Every case reads real shipped Rust source as text
 * and FAILS if the contract regresses: no runtime claim, no server, no browser.
 *
 * Covered contracts:
 *   - the search response exposes citation metadata only (no content/text/body)
 *   - blank q and out-of-range limit return typed 400s; principal comes from
 *     the server-verified DocUser, never from the query string or a body
 *   - search filters tenant, then AclMetadata::allows(user), BEFORE scoring
 *   - AclMetadata::allows denies a foreign tenant before public/allow-list
 */
"use strict";

var h = require("./00_harness");

var KB_SEARCH = "botserver/crates/botdocs/src/handlers_api/kb_search.rs";
var VECTOR_INDEX = "botserver/crates/botcore/src/kb/structured/vector_index.rs";
var CHUNK = "botserver/crates/botcore/src/kb/structured/chunk.rs";

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  var src = h.readRepo(relPath);
  h.assert(src.indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath);
}

/** Return the body of `pub struct <name> { ... }` from a source file. */
function structBody(relPath, name) {
  var src = h.readRepo(relPath);
  var marker = "pub struct " + name + " {";
  var start = src.indexOf(marker);
  h.assert(start !== -1, "struct " + name + " missing in " + relPath);
  var open = src.indexOf("{", start);
  var close = src.indexOf("}", open);
  h.assert(close !== -1, "struct " + name + " has no closing brace in " + relPath);
  return src.slice(open + 1, close);
}

// 1. The wire DTO is citation metadata only. No field may carry document
//    bytes back to the caller — the index never stored them in the first place.
h.staticTest("KB search response carries citation metadata only, no content", function () {
  var body = structBody(KB_SEARCH, "KbSearchHitDto");
  ["pub id:", "pub source_ref:", "pub section:", "pub score:"].forEach(function (field) {
    h.assert(body.indexOf(field) !== -1,
      "KbSearchHitDto must expose " + field + " :: <" + body + ">");
  });
  ["content", "text", "body", "payload", "document_bytes"].forEach(function (forbidden) {
    h.assert(body.indexOf(forbidden) === -1,
      "KbSearchHitDto must NOT expose " + forbidden + " :: <" + body + ">");
  });
  // The response wrapper only collects that DTO — it cannot smuggle content.
  var wrapper = structBody(KB_SEARCH, "KbSearchResponse");
  h.assert(wrapper.indexOf("pub hits: Vec<KbSearchHitDto>") !== -1,
    "KbSearchResponse must wrap only citation DTOs :: <" + wrapper + ">");
});

// 2. Validation errors are typed and fixed-message (400), and the principal
//    is derived from the server-verified DocUser — never client-supplied.
h.staticTest("KB search returns typed 400s and derives principal from DocUser", function () {
  contains(KB_SEARCH,
    "Self::MissingQuery => (StatusCode::BAD_REQUEST, \"query is required\")",
    "blank query maps to typed 400");
  contains(KB_SEARCH,
    "Self::InvalidLimit => (StatusCode::BAD_REQUEST, \"limit must be between 1 and 50\")",
    "out-of-range limit maps to typed 400");
  contains(KB_SEARCH, "if text.is_empty() {", "blank q is rejected after trim");
  contains(KB_SEARCH, "if limit == 0 || limit > MAX_LIMIT {", "limit outside 1..50 rejected");

  // The query surface carries ONLY q + limit: no user/principal/tenant id.
  var query = structBody(KB_SEARCH, "KbSearchQuery");
  h.assert(query.indexOf("pub q:") !== -1 && query.indexOf("pub limit:") !== -1,
    "KbSearchQuery must carry q + limit :: <" + query + ">");
  h.assert(!/user|principal|owner|tenant|email|role/i.test(query),
    "query string must not carry a caller-supplied identity :: <" + query + ">");

  // The handler takes the verified DocUser and derives its own principal.
  var src = h.readRepo(KB_SEARCH);
  var fn = src.indexOf("pub async fn handle_kb_search(");
  h.assert(fn !== -1, "handle_kb_search present");
  var head = src.slice(fn, src.indexOf("{", fn));
  h.assert(head.indexOf("user: DocUser,") !== -1,
    "handler must take the verified DocUser :: <" + head + ">");
  h.assert(!/user_id|principal|owner/i.test(head),
    "handler must not accept an identity from query/body :: <" + head + ">");
  contains(KB_SEARCH, "principal_for(&state, &user)", "principal derived in handler");
  contains(KB_SEARCH, "parse_owner_id(user.id())", "owner id from DocUser, not params");
});

// 3. Filtering order is load-bearing: unauthorized points must be discarded
//    BEFORE cosine scoring so no existence/score hint can leak.
h.staticTest("KB search filters tenant then ACL before cosine scoring", function () {
  var src = h.readRepo(VECTOR_INDEX);
  var tenantFilter = src.indexOf(".filter(|(_, point)| organization.as_deref() == Some(point.tenant_id.as_str()))");
  var aclFilter = src.indexOf(".filter(|(_, point)| point.acl.allows(user))");
  var filterMap = src.indexOf(".filter_map(|(id, point)|");
  var scoring = src.indexOf("let score = cosine(query_embedding, &point.vector);");
  h.assert(tenantFilter !== -1, "tenant filter present in search()");
  h.assert(aclFilter !== -1, "ACL filter present in search()");
  h.assert(filterMap !== -1, "filter_map present in search()");
  h.assert(scoring !== -1, "cosine scoring present in search()");
  h.assert(tenantFilter < aclFilter, "tenant filter must precede ACL filter");
  h.assert(aclFilter < filterMap, "ACL filter must precede the scoring stage");
  h.assert(filterMap < scoring, "scoring must happen inside filter_map, after both filters");
  contains(VECTOR_INDEX, "apply `AclMetadata::allows`, then score",
    "filter-order contract stays documented as authoritative");
});

// 4. The ACL itself denies foreign tenants first — before the public shortcut
//    or any user/role/group allow-list can grant access.
h.staticTest("AclMetadata::allows denies foreign tenants before public/allow-list", function () {
  var src = h.readRepo(CHUNK);
  var allows = src.indexOf("pub fn allows(&self, user: &UserContext) -> bool {");
  h.assert(allows !== -1, "AclMetadata::allows present");
  var body = src.slice(allows);
  var tenantCheck = body.indexOf("if self.tenant_id != Self::tenant_of(user) {");
  var tenantDeny = body.indexOf("return false;", tenantCheck);
  var publicBranch = body.indexOf("if self.is_public {");
  // The roles/groups branches are formatted across lines (`self` + newline +
  // `.allowed_...`), so match the dotted member name rather than `self.<name>`.
  var users = body.indexOf(".allowed_users");
  var roles = body.indexOf(".allowed_roles");
  var groups = body.indexOf(".allowed_groups");
  var allowList = Math.min(users, roles, groups);
  h.assert(tenantCheck !== -1, "tenant check present in allows()");
  h.assert(tenantDeny !== -1, "tenant mismatch must return a denial");
  h.assert(publicBranch !== -1, "public branch present in allows()");
  h.assert(users !== -1 && roles !== -1 && groups !== -1, "allow-list branches present");
  h.assert(tenantCheck < tenantDeny, "tenant check must return false immediately");
  h.assert(tenantDeny < publicBranch, "tenant denial must precede the public shortcut");
  h.assert(publicBranch < allowList, "public branch must precede allow-list matching");
});
