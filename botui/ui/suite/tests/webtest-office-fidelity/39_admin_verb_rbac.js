/**
 * [STATIC] deterministic coverage for the #2448 admin-verb RBAC bypass fix.
 * Every case reads real shipped Rust source as text and FAILS if any part of
 * the guard disappears: no runtime claim, no server, no browser, no cargo.
 * Each case first proves the surrounding function exists (non-vacuity), then
 * asserts literal substrings and their strict ordering, so a truncated, empty
 * or reverted file can never satisfy a case by accident.
 *
 * Covered:
 *   (a) CommandAuthError is a typed error with fixed sanitized messages, the
 *       pure decision table denies non-admins and an unresolved role fails
 *       closed, and converting the error cannot leak a command or DB detail;
 *   (b) BOTH gates are present: execute_command's catalog-verb gate and the
 *       api.exec loopback gate call authorize_admin_command / the reverse
 *       admin_only_verb_binding lookup and return Err before any handler or
 *       HTTP call runs;
 *   (c) the seven targeted bin tests exist, including a non-vacuous loop that
 *       checks every admin-only binding on its own route, and the two Office
 *       verbs behind that loop are still declared admin_only.
 */
"use strict";

var h = require("./00_harness");

var AUTH = "botserver/src/core/bot/api_catalog/command_auth.rs";
var EXEC = "botserver/src/core/bot/api_catalog/execute_command.rs";
var VERB_ENDPOINTS = "botserver/src/apps/commands/verb_endpoints/mod.rs";
var OFFICE = "botserver/src/apps/commands/app_commands_extra/table/office_review_ops.rs";

var DERIVE = "#[derive(Debug, Clone, Copy, PartialEq, Eq)]";
var AUTH_FN = "pub(super) fn authorize_admin_command(";
var ROLE_FN = "pub(super) fn resolve_gated_role(";
var TESTS_MARK = "#[cfg(test)]";
var EXEC_FN = "pub async fn execute_command(";
var REVERSE_FN = "pub fn admin_only_verb_binding(method: &str, path: &str) -> bool {";

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  h.assert(h.readRepo(relPath).indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath);
}

/** Literal substring assertion on an already-read source span. */
function spanHas(span, needle, msg) {
  h.assert(span.indexOf(needle) !== -1,
    (msg || "span must contain") + " <" + needle + "> :: " + span.slice(0, 220));
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

/** Every marker must be present, in this strict order. */
function ordered(span, markers, what) {
  var last = -1;
  markers.forEach(function (marker) {
    var at = span.indexOf(marker);
    h.assert(at !== -1, (what || "span") + " must contain <" + marker + "> :: " + span.slice(0, 220));
    h.assert(at > last,
      (what || "span") + " order broken at <" + marker + "> :: " + at + " <= " + last);
    last = at;
  });
}

/** Count literal (non-overlapping) occurrences in a span. */
function count(span, needle) {
  return span.split(needle).length - 1;
}

// (a) Typed error, fixed messages, fail-closed decision table and resolver.
//     A denial must never interpolate a command name, REST path or DB error.
h.staticTest("Typed CommandAuthError uses fixed messages and fails closed on role resolution", function () {
  var typeSpan = between(AUTH, DERIVE, "impl From<CommandAuthError> for String {");
  ordered(typeSpan, [
    "pub(super) enum CommandAuthError {",
    "    AdminRequired,",
    "    RoleUnresolved,",
    "impl CommandAuthError {",
    "    pub(super) fn sanitized_message(self) -> &'static str {",
    "            Self::AdminRequired => \"command denied: admin role required\",",
    "            Self::RoleUnresolved => \"command denied: caller role could not be resolved\","
  ], "typed error variants and their fixed sanitized messages");

  var conversion = between(AUTH, "impl From<CommandAuthError> for String {",
    "/// Pure decision table");
  ordered(conversion, [
    "    fn from(error: CommandAuthError) -> Self {",
    "        error.sanitized_message().to_string()"
  ], "error converts only through the fixed message");

  var decision = between(AUTH, AUTH_FN,
    "/// True when the catalog marks the named verb admin-only.");
  spanHas(decision, ") -> Result<(), CommandAuthError> {",
    "the gate returns the typed error, not a raw String");
  ordered(decision, [
    "    if !admin_only {",
    "        return Ok(());",
    "    match role {",
    "        Some(crate::security::user_role::ROLE_ADMIN) => Ok(()),",
    "        Some(_) => Err(CommandAuthError::AdminRequired),",
    "        None => Err(CommandAuthError::RoleUnresolved),"
  ], "fail-closed admin decision table");

  var resolve = between(AUTH, ROLE_FN, TESTS_MARK);
  ordered(resolve, [
    "    state.conn.get().ok()?;",
    "    let role = crate::security::user_role::resolve_user_role(&state.conn, user_id);",
    "    (!role.is_empty()).then_some(role)"
  ], "a pool failure leaves the gated role unresolved");
  h.assertEqual(count(resolve, "resolve_user_role("), 1,
    "role resolution must call the shared resolver exactly once");
});

// (b) Both enforcement points must survive. Removing either the verb gate at
//     the top of execute_command or the api.exec route gate breaks a case.
h.staticTest("The catalog verb gate and the api.exec route gate both deny before execution", function () {
  var src = h.readRepo(EXEC);
  contains(EXEC, "use super::command_auth::{authorize_admin_command, requires_admin, resolve_gated_role};",
    "executor imports the typed gate helpers");
  var execStart = src.indexOf(EXEC_FN);
  h.assert(execStart !== -1, "missing trusted executor <" + EXEC_FN + "> in " + EXEC);
  var dispatch = src.indexOf("    match name {", execStart);
  h.assert(dispatch !== -1, "missing handler dispatch <    match name {> in " + EXEC);
  var beforeDispatch = src.slice(execStart, dispatch);

  // Gate #1 — every catalog verb is authorised before the dispatch match can
  //           run a handler.
  ordered(beforeDispatch, [
    "    let gated = requires_admin(name);",
    "    let role = gated.then(|| resolve_gated_role(state, user_id)).flatten();",
    "    if let Err(error) = authorize_admin_command(role.as_deref(), gated) {",
    "        botlog::safe_info!(\"command '{name}' denied: {}\", error.sanitized_message());",
    "        return Err(error.into());",
    "    let obj = match params {"
  ], "catalog verb gate runs before handler dispatch");
  h.assertEqual(count(beforeDispatch, "authorize_admin_command("), 1,
    "the catalog gate must call authorize_admin_command exactly once before dispatch");

  // Gate #2 — a prompt injection that names a bound admin REST route instead
  //           of the verb is denied before the trusted HTTP loopback.
  var apiExec = between(EXEC, "        \"api.exec\" => {",
    "        \"crm.people.list\" | \"people.list\" =>");
  ordered(apiExec, [
    "            if crate::apps::commands::admin_only_verb_binding(&method, &path) {",
    "                let role = resolve_gated_role(state, user_id);",
    "                if let Err(error) = authorize_admin_command(role.as_deref(), true) {",
    "                    return Err(error.into());",
    "            crate::core::bot::api_exec::exec_endpoint_trusted("
  ], "api.exec route gate denies before the HTTP loopback");
  h.assertEqual(count(apiExec, "authorize_admin_command("), 1,
    "the api.exec gate must make its own authorize_admin_command call");

  // The reverse lookup keys on the catalog admin_only flag of the verb bound
  // to the concrete (method, path), never on the supplied path text alone.
  var reverse = between(VERB_ENDPOINTS, REVERSE_FN,
    "/// Compares a filled path with a bind template.");
  ordered(reverse, [
    "    table::all().iter().any(|binding| {",
    "        binding.method.eq_ignore_ascii_case(method)",
    "            && path_matches_template(path, binding.path)",
    "            && crate::apps::commands::command_by_name(binding.verb)",
    "                .is_some_and(|command| command.admin_only)"
  ], "reverse lookup keys on the catalog admin_only flag");
});

// (c) Seven targeted tests pin both gates and the reverse lookup. The
//     every-binding loop asserts a non-zero gated count so it cannot pass
//     vacuously, and the two Office verbs it covers stay admin_only.
h.staticTest("Seven targeted RBAC tests exist and cover every admin-only binding", function () {
  contains(AUTH, TESTS_MARK, "command_auth test module is compiled in");
  contains(VERB_ENDPOINTS, TESTS_MARK, "reverse-lookup test module is compiled in");
  contains(VERB_ENDPOINTS, "mod admin_binding_tests {", "admin binding test module exists");

  var sources = {};
  sources[AUTH] = h.readRepo(AUTH);
  sources[VERB_ENDPOINTS] = h.readRepo(VERB_ENDPOINTS);
  var cases = [
    {
      file: AUTH,
      entry: "    fn admin_allowed_non_admin_denied() {",
      marks: [
        "authorize_admin_command(Some(ROLE_ADMIN), true), Ok(())",
        "Err(CommandAuthError::AdminRequired)"
      ]
    },
    {
      file: AUTH,
      entry: "    fn unresolved_role_denied_and_plain_verbs_open() {",
      marks: [
        "authorize_admin_command(None, true),",
        "Err(CommandAuthError::RoleUnresolved)",
        "CommandAuthError::AdminRequired.sanitized_message(),"
      ]
    },
    {
      file: AUTH,
      entry: "    fn state_changing_commands_require_admin() {",
      marks: [
        "assert!(requires_admin(name), \"{name} is not gated for admin\");",
        "assert!(!requires_admin(\"database.tables.list\"));"
      ]
    },
    {
      file: AUTH,
      entry: "    fn every_database_verb_is_admin_only() {",
      marks: [
        "database_verbs::declared_verbs()",
        "\"database verb {name} needs admin_only\""
      ]
    },
    {
      file: VERB_ENDPOINTS,
      entry: "    fn admin_only_verbs_match_their_bound_routes() {",
      marks: [
        "\"/api/docs/track-changes/accept-reject-all\"",
        "\"/api/sheet/worksheets/delete\""
      ]
    },
    {
      file: VERB_ENDPOINTS,
      entry: "    fn other_routes_and_methods_are_not_admin_only() {",
      marks: [
        "\"/api/docs/comment/reply\"",
        "assert!(!admin_only_verb_binding("
      ]
    },
    {
      file: VERB_ENDPOINTS,
      entry: "    fn every_admin_only_binding_is_denied_on_its_own_route() {",
      marks: [
        "let mut gated = 0usize;",
        "for binding in table::all() {",
        "admin_only_verb_binding(binding.method, binding.path),",
        "gated += 1;",
        "assert!(gated > 0, \"no admin-only verb bindings found\");"
      ]
    }
  ];
  h.assertEqual(cases.length, 7, "the RBAC fix must keep all seven targeted tests");

  var last = -1;
  cases.forEach(function (testCase) {
    var src = sources[testCase.file];
    var at = src.indexOf(testCase.entry);
    h.assert(at !== -1, "missing targeted test <" + testCase.entry + "> in " + testCase.file);
    if (testCase.file === AUTH) {
      h.assert(at > last, "targeted test order broken at <" + testCase.entry + ">");
      last = at;
    }
    var body = src.slice(at, at + 1400);
    testCase.marks.forEach(function (marker) {
      h.assert(body.indexOf(marker) !== -1,
        "<" + testCase.entry + "> must assert <" + marker + "> :: " + body.slice(0, 240));
    });
  });

  var loop = between(VERB_ENDPOINTS,
    "    fn every_admin_only_binding_is_denied_on_its_own_route() {",
    "/// Docs exposes one route per export format;");
  ordered(loop, [
    "        let mut gated = 0usize;",
    "        for binding in table::all() {",
    "            if crate::apps::commands::command_by_name(binding.verb)",
    "                .is_some_and(|command| command.admin_only)",
    "                assert!(",
    "                    admin_only_verb_binding(binding.method, binding.path),",
    "                gated += 1;",
    "        assert!(gated > 0, \"no admin-only verb bindings found\");"
  ], "every admin-only binding is checked and the loop cannot be empty");

  // The two Office verbs whose routes the reverse-lookup test pins are the
  // admin_only:true declarations in office_review_ops.rs.
  ["docs.track_changes.accept_all", "sheet.worksheets.delete"].forEach(function (verb) {
    var block = between(OFFICE, "\"" + verb + "\",", "    cmd(");
    ordered(block, ["        None,", "        true,"],
      verb + " must still be declared admin_only");
  });
});
