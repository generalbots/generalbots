/**
 * [KNOWN BUG] #2452 — source-backed guards for the two gaps that BLOCKED the
 * browser UAT of ownership transfer on v7.1.2 (2026-11-24):
 *
 *   (a) POST /api/collab/permissions/transfer deletes the old owner's row,
 *       inserts the new owner's row and appends a `transfer` collab_activity,
 *       but it never moves the document bytes. Docs storage is keyed by the
 *       verified user namespace (`users/{owner}/docs/...`), and the comments /
 *       versions live in the sidecar under that same namespace. The ACL and the
 *       audit row survive, but the new owner's namespace has no copy of the
 *       document, so the transfer cannot be verified end-to-end in a browser.
 *   (b) There is no browser-reachable second real user in local dev:
 *       `/api/directory` and `/api/users` map to the admin-only settings app
 *       and `admin.users.list` is catalogued as admin_only; the cloud dev login
 *       accepts only the ONE identity in `admin-credentials.json` (the Zitadel
 *       v2 sessions API is absent here), and the login page hardcodes
 *       `admin@localhost`/`dev`. Imported Zitadel users therefore cannot
 *       authenticate through the dev login path, so there is nobody to become
 *       the new owner and log in as during the UAT.
 *
 * NON-VACUITY: each case first proves the shipped source exists and carries its
 * real function names, SQL statements, route prefixes, namespace helpers and
 * catalog flags, and asserts the scans covered many non-trivial files/bytes.
 * The gap assertions are then SCOPED absence checks over those exact sources,
 * so a missing, empty, truncated or garbled file cannot satisfy a case. Every
 * case flips to FAIL the moment a real fix lands: a document-copy path inside
 * the transfer handler / collab routes, or a browser-reachable second-user dev
 * login or non-admin user-list route. At that point this knownBug must be
 * removed and replaced by a runtime browser test.
 * No user-facing string, no server, no browser, no cargo, no network.
 */
"use strict";

var h = require("./00_harness");
var fs = h.fs;
var path = h.path;

var PERMISSIONS = "botserver/src/main_module/routes/collab_routes/permissions.rs";
var TRANSFER = "botserver/src/main_module/routes/collab_routes/transfer.rs";
var COLLAB_DIR = "botserver/src/main_module/routes/collab_routes";
var STORAGE_CORE = "botserver/crates/botdocs/src/storage_core.rs";
var STORAGE_DRIVE = "botserver/crates/botdocs/src/storage_drive.rs";
var BOOK_SECURITY = "botbook/src/09-security/README.md";

var LOGIN = "botserver/crates/botcloud/src/api/login.rs";
var LOGIN_HELPERS = "botserver/crates/botcloud/src/api/login_helpers.rs";
var LOGIN_UI = "botui/ui/login/index.html";
var COMMAND_CATALOG = "botserver/src/apps/commands/deep_link_params_for_app.rs";
var PREFIX_MAP = "botserver/src/core/bot/endpoint_prefix_map.rs";
var RBAC_PUBLIC = "botserver/crates/botsecurity-auth/src/rbac_middleware/routes_public.rs";

var DEV_LOGIN_DIRS = ["botserver/crates/botcloud/src", "botui/ui/login"];
var SKIP_DIRS = ["vendor", "node_modules", "target", "tests", "webtest-office-fidelity"];

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  h.assert(h.readRepo(relPath).indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath);
}

/** Exact, non-empty source span between two literal markers (end exclusive). */
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
    h.assert(span.indexOf(token) === -1,
      (what || "gap") + " must still be absent: found <" + token + "> :: " + span.slice(0, 200));
  });
}

/**
 * Case-insensitive literal scan for fix tokens over repo-relative directories.
 * Test dirs are skipped so this module's own literals cannot make the scan hit.
 */
function scanSources(relDirs, extensions, tokens) {
  var lowerTokens = tokens.map(function (token) { return token.toLowerCase(); });
  var stats = { files: 0, bytes: 0 };
  var hits = [];
  relDirs.forEach(function (relDir) {
    var root = path.join(h.REPO_DIR, relDir);
    h.assert(fs.existsSync(root), "scan root must ship: " + relDir);
    (function walk(dir) {
      fs.readdirSync(dir, { withFileTypes: true })
        .sort(function (a, b) { return a.name < b.name ? -1 : a.name > b.name ? 1 : 0; })
        .forEach(function (entry) {
          var full = path.join(dir, entry.name);
          if (entry.isDirectory()) {
            if (SKIP_DIRS.indexOf(entry.name) === -1) walk(full);
            return;
          }
          if (extensions.indexOf(path.extname(entry.name)) === -1) return;
          var src = fs.readFileSync(full, "utf8").toLowerCase();
          stats.files++;
          stats.bytes += src.length;
          lowerTokens.forEach(function (token) {
            if (src.indexOf(token) !== -1) {
              hits.push(path.relative(h.REPO_DIR, full).split(path.sep).join("/") + " :: " + token);
            }
          });
        });
    })(root);
  });
  return { stats: stats, hits: hits };
}

/** Non-vacuity first, then the documented absence of a fix. */
function assertAbsent(scan, what, minFiles, minBytes) {
  h.assert(scan.stats.files > minFiles,
    "non-vacuity: " + what + " scan covered only " + scan.stats.files + " files");
  h.assert(scan.stats.bytes > minBytes,
    "non-vacuity: " + what + " scan read only " + scan.stats.bytes + " bytes");
  h.assertEqual(scan.hits.length, 0,
    what + " must still be absent; found: " + scan.hits.slice(0, 6).join(" | "));
}

// (a) The transfer route changes the ACL and the audit row only; no code path
//     copies the namespace-keyed document (and its sidecar) to the new owner.
h.knownBug("#2452 BLOCKED: ownership transfer is metadata-only — no document bytes move between user namespaces", function () {
  // The handler moved from permissions.rs to transfer.rs (re-exported by mod.rs).
  contains(TRANSFER, "pub async fn transfer_ownership(", "transfer handler ships");
  contains(TRANSFER, "DELETE FROM resource_permissions", "old owner ACL row is removed");
  contains(TRANSFER, "INSERT INTO resource_permissions", "new owner ACL row is inserted");
  contains(TRANSFER, "record_activity(", "activity audit call ships");
  contains(TRANSFER, '"transfer"', "transfer activity verb ships");
  contains(BOOK_SECURITY, "revoke and ownership transfer are owner-only",
    "book discloses the owner-only transfer");
  contains(BOOK_SECURITY, "Share, revoke, transfer, comment and version events are appended",
    "book discloses that transfer is an audit event");
  contains(STORAGE_CORE, "pub fn get_user_docs_path(user_identifier: &str) -> String {",
    "docs namespace helper ships");
  contains(STORAGE_CORE, 'format!("users/{}/docs", sanitize_owner(user_identifier))',
    "document keys are the owner namespace");
  contains(STORAGE_DRIVE, "let base_path = get_user_docs_path(user_identifier);",
    "document writes resolve through that namespace");

  var transferSrc = h.readRepo(TRANSFER);
  var handler = transferSrc.slice(transferSrc.indexOf("pub async fn transfer_ownership("));
  h.assert(handler.indexOf("DELETE FROM resource_permissions") !== -1 &&
    handler.indexOf("INSERT INTO resource_permissions") !== -1 &&
    handler.indexOf("record_activity(") !== -1 && handler.indexOf('"transfer"') !== -1,
    "non-vacuity: transfer span must still update the ACL and audit row");

  // A real fix would copy/copy-on-write the .html + .meta.json/.docx sidecar
  // into the new owner's namespace (get_user_docs_path / put_object / copy).
  absent(handler, [
    "get_user_docs_path", "save_document_to_drive", "load_document_from_drive",
    "put_object", "copy_object", "copyObjects", "copyObject", "namespace",
    "migrate", "meta.json", ".html", "docx"
  ], "document-byte migration in the transfer handler");

  var scan = scanSources([COLLAB_DIR], [".rs"], [
    "get_user_docs_path", "save_document_to_drive", "load_document_from_drive",
    "put_object", "copy_object", "migrate_document", "transfer_document",
    "document_namespace", "copy_documents"
  ]);
  assertAbsent(scan, "a collab-route document migration path", 5, 20000);
});

// (b) The user-list surfaces are admin-gated and the dev login has exactly one
//     credential, so no second real identity can be reached from a browser.
h.knownBug("#2452 BLOCKED: no browser-reachable second real user in local dev (admin-only user list, single admin-credential dev login)", function () {
  contains(LOGIN, "pub(crate) async fn handle_login(", "cloud login handler ships");
  contains(LOGIN, "/v2/sessions", "Zitadel v2 password verification ships");
  contains(LOGIN,
    "dev-only bootstrap admin (admin-credentials.json, present only in dev)",
    "single dev-admin fallback ships");
  contains(LOGIN, "lookup_admin_credentials_user_id(&body.email, &body.password)",
    "dev login consults the one bootstrap credential");
  contains(LOGIN_HELPERS, "pub(crate) fn credentials_authorize(",
    "single-credential authorization predicate ships");
  contains(LOGIN_HELPERS, "cred_email.eq_ignore_ascii_case(email)",
    "predicate matches the one email in that file");
  contains(LOGIN_HELPERS, "password_hash", "predicate verifies that file's hash");
  contains(LOGIN_UI, "admin@localhost", "dev login page hardcodes the admin identity");
  contains(LOGIN_UI, "password:'dev'", "dev login page hardcodes the dev password");
  contains(COMMAND_CATALOG,
    'cmd("settings", "admin.users.list", "List users", "List organization users, roles and groups.", &[], None, true),',
    "user-list verb is catalogued admin_only=true");
  contains(PREFIX_MAP, '("/api/directory", "settings")',
    "directory routes map to the settings app");
  contains(PREFIX_MAP, '("/api/users", "settings")',
    "user routes map to the settings app");
  contains(RBAC_PUBLIC, 'matching route permission found" (403) even for a valid credential',
    "RBAC default-deny 403 is shipped");

  var predicate = between(LOGIN_HELPERS, "pub(crate) fn credentials_authorize(",
    "pub(crate) fn lookup_admin_credentials_user_id(");
  h.assert(predicate.indexOf("cred_email.eq_ignore_ascii_case(email)") !== -1,
    "non-vacuity: predicate span must still pin one credential");
  absent(predicate, [
    "list_users", "directory/users", "users/list", "dev_login", "dev-login",
    "login_as", "login-as", "impersonat", "switch_user", "user_picker", "selectUser"
  ], "second-user discovery/auth in the dev credential predicate");

  var ui = h.readRepo(LOGIN_UI);
  absent(ui, [
    "user-picker", "userPicker", "login-as", "loginAs", "dev-login",
    "switch-user", "selectUser", "list_users", "/api/directory/users"
  ], "second-user controls in the dev login page");

  var scan = scanSources(DEV_LOGIN_DIRS, [".rs", ".js", ".html"], [
    "dev_login", "dev-login", "login_as", "login-as", "impersonat",
    "switch_user", "switch-user", "user_picker", "user-picker", "selectUser",
    "list_users", "directory/users"
  ]);
  assertAbsent(scan, "a browser-reachable second-user dev login path", 10, 20000);
});

h.note("FOLLOW-UP #2452: transfer needs a namespace-aware document copy before browser UAT, and local dev needs a real second identity (a non-admin user list plus a dev login that can authenticate an imported user). Until then both verifications remain BLOCKED; no server, browser or cargo was run.");
