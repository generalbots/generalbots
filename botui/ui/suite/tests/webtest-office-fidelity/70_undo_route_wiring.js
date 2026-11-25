/**
 * #2448 criterion 4 follow-up — deterministic static coverage for the LIVE
 * agent-batch undo endpoint that consumes the typed undo data model.
 *
 * Cases (literal source assertions, no execution):
 *   (a) POST /api/collab/undo is mounted and its typed body carries ONLY a
 *       batch id — no caller-supplied user, bot or session;
 *   (b) verified_scope derives user/bot/session exclusively from the verified
 *       AuthenticatedUser and validates before storage;
 *   (c) undo_plan_for consumes once (UndoStore::consume) and returns the LIFO
 *       before-state plan (UndoBatch::revert_plan); a named before version is
 *       restored only through the existing restore_version handler with the
 *       same server-side identity;
 *   (d) deterministic tests cover cross-session denial, double undo and the
 *       before-not-after rule, and the chat card carries the explicit batch id.
 *
 * NON-VACUITY: every case first proves the real files, functions, route and
 * test bodies ship with non-empty extracted spans; scoped absence checks are
 * restricted to those spans. Removing the route, widening the body, deriving
 * scope from a header, skipping consume/revert_plan, bypassing restore_version
 * or deleting any of the three tests makes the matching case FAIL. No server,
 * browser, cargo, network or user-facing string is involved.
 */
"use strict";

var h = require("./00_harness");

var ROUTE = "botserver/src/main_module/routes/collab_routes/undo.rs";
var MOD = "botserver/src/main_module/routes/collab_routes/mod.rs";
var MODEL = "botserver/src/core/bot/undo/mod.rs";
var STORE = "botserver/src/core/bot/undo/store.rs";
var CARD = "botui/ui/suite/chat/modules/40_consent_cards.js";

function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}
function absent(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1,
    (msg || "source must not contain") + " <" + needle + "> :: " + src.slice(0, 200));
}
function span(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  h.assert(end > start, "empty source span between <" + startMarker + "> and <" + endMarker + ">");
  return src.slice(start, end);
}
function ordered(src, before, after, msg) {
  var b = src.indexOf(before), a = src.indexOf(after);
  h.assert(b !== -1 && a !== -1 && b < a,
    (msg || "ordering regressed") + " :: <" + before + "> before <" + after + ">");
}
function countIn(src, needle) { return src.split(needle).length - 1; }

h.staticTest("#2448 undo route is mounted and its typed body carries only a batch id", function () {
  var route = h.readRepo(ROUTE), mod = h.readRepo(MOD);
  h.assert(route.length > 8000, "undo route source must be non-trivial");
  contains(mod, "mod undo;", "undo route module is declared");
  contains(mod, "pub use undo::*;", "undo route handlers are re-exported");
  contains(mod, '.route("/api/collab/undo", post(undo_batch))',
    "POST /api/collab/undo is mounted");

  contains(route, "pub struct UndoBody {", "typed undo body ships");
  var body = span(route, "pub struct UndoBody {", "/// Build the ownership scope");
  contains(body, "pub batch_id: String,", "body carries the explicit batch id");
  var fields = body.match(/pub\s+\w+\s*:/g) || [];
  h.assertEqual(fields.length, 1, "the undo body must have exactly one public field");
  ["user_id", "bot_id", "session_id", "owner", "actor", "scope"].forEach(function (forbidden) {
    absent(body, forbidden, "client must not supply " + forbidden);
  });

  contains(route, "pub async fn undo_batch(", "undo handler ships");
  var handler = span(route, "pub async fn undo_batch(", "#[cfg(test)]");
  contains(handler, "Extension(user): Extension<AuthenticatedUser>",
    "verified session principal is extracted by the middleware");
  contains(handler, "Json(body): Json<UndoBody>", "only the typed body is accepted");
  contains(handler, "undo_plan_for(&store, &scope, &body.batch_id)",
    "the handler delegates to the scoped single-use core");
  ordered(handler, "verified_scope(&user)", "undo_plan_for(&store, &scope, &body.batch_id)",
    "scope is resolved before any batch is consumed");
});

h.staticTest("#2448 verified_scope derives user/bot/session from the authenticated session, never from the request", function () {
  var route = h.readRepo(ROUTE);
  contains(route, "pub(crate) fn verified_scope(user: &AuthenticatedUser)", "scope resolver ships");
  var scopeFn = span(route, "pub(crate) fn verified_scope(user: &AuthenticatedUser)",
    "/// Single-use + LIFO core");
  h.assert(scopeFn.length > 600, "non-vacuity: scope resolver must be non-trivial");
  contains(scopeFn, "user.is_authenticated()", "anonymous callers fail closed");
  contains(scopeFn, "Err(UndoError::InvalidScope)", "invalid identities are typed rejections");
  contains(scopeFn, ".current_bot_id", "bot id comes from the verified session");
  contains(scopeFn, "user.accessible_bot_ids()", "the alternate bot source is verified grants");
  contains(scopeFn, "user.session_id.as_deref()", "session id comes from the verified session");
  contains(scopeFn, "UndoScope::new(user.user_id, bot_id, &session_id)",
    "the model scope is built from verified claims only");
  contains(scopeFn, "scope.validate()?;", "scope is validated before lookup");
  ordered(scopeFn, "user.is_authenticated()", "scope.validate()?;",
    "authentication precedes scope construction");
  ["body.", "req.", "Query(", "HeaderMap", "headers()"].forEach(function (forbidden) {
    absent(scopeFn, forbidden, "scope resolver must not read " + forbidden);
  });
});

h.staticTest("#2448 undo consumes once and reverts LIFO before-states through the existing version-restore handler", function () {
  var route = h.readRepo(ROUTE), model = h.readRepo(MODEL), store = h.readRepo(STORE);
  contains(route, "pub(crate) fn undo_plan_for<S: UndoStore>(", "single-use core ships");
  var core = span(route, "pub(crate) fn undo_plan_for<S: UndoStore>(",
    "/// A restore target is only read");
  contains(core, "let batch = store.consume(&key)?;", "consume is the single-use gate");
  contains(core, "Ok(batch.revert_plan())", "the plan is the LIFO revert plan");
  ordered(core, "store.consume(&key)?", "batch.revert_plan()",
    "consume must precede the revert plan");

  contains(model, "pub fn revert_plan(&self) -> Vec<RevertOperation> {", "model plan ships");
  var plan = span(model, "pub fn revert_plan(&self) -> Vec<RevertOperation> {",
    "pub struct RevertOperation {");
  contains(plan, ".rev()", "plan is reversed (LIFO)");
  contains(plan, "before: op.before.clone(),", "plan carries only before-state");
  absent(plan, "after:", "after-state is never a revert source");

  contains(store, "Conditional UPDATE is the single-use guard",
    "conditional single-use store ships");
  contains(store, "if changed == 0 {", "zero-row update is handled");

  contains(route, "fn before_version_id(before: &serde_json::Value)", "restore target reader ships");
  var reader = span(route, "fn before_version_id(before: &serde_json::Value)",
    "/// Typed failures map");
  contains(reader, '.get("version_id")', "only the before-state selects a version");
  absent(reader, "after", "the after-state can never select a restore target");

  var handler = span(route, "pub async fn undo_batch(", "#[cfg(test)]");
  contains(handler, "super::restore_version(", "restore is routed through the existing handler");
  contains(handler, "Extension(user.clone())", "the verified session is forwarded, not a body owner");
  contains(handler, "Path(version_id)", "only the recorded version id is passed");
  contains(handler, "let plan = peek(&store, &key)", "a read-only peek feeds the restore plan");
  ordered(handler, "let plan = peek(&store, &key)",
    "let restored = restore_plan(&state, &user, locale, &plan).await?;",
    "the batch is peeked before any restore is attempted");
  ordered(handler, "let restored = restore_plan(&state, &user, locale, &plan).await?;",
    "let reverted = undo_plan_for(&store, &scope, &body.batch_id)",
    "restore succeeds BEFORE the single-use consume (a failed restore leaves the batch active)");
  absent(handler, "store.consume", "the handler must not consume before restore");
  ["req.user_id", "body.user_id", "body.bot_id", "body.owner"].forEach(function (forbidden) {
    absent(handler, forbidden, "restore must not receive a caller-supplied " + forbidden);
  });
});

h.staticTest("#2448 deterministic tests pin cross-session denial, double undo and before-not-after, and the chat card carries the batch id", function () {
  var route = h.readRepo(ROUTE), ui = h.readRepo(CARD);
  var tests = route.slice(route.indexOf("#[cfg(test)]"));
  h.assert(tests.indexOf("#[cfg(test)]") === 0, "route test module must ship");
  h.assert(tests.length > 3000, "non-vacuity: route tests must be non-trivial");
  h.assert(countIn(tests, "#[test]") >= 4, "four deterministic route tests must ship");

  var names = [
    "fn cross_session_undo_is_denied_and_leaves_the_batch_active()",
    "fn double_undo_is_denied_by_the_single_use_core()",
    "fn revert_uses_before_state_only_never_the_after_state()",
    "fn verified_scope_never_accepts_anonymous_or_botless_identities()"
  ];
  names.forEach(function (name) { contains(tests, name, "route test must ship: " + name); });

  var cross = span(tests, names[0], "#[test]\n    " + names[1]);
  contains(cross, 'scope("session-2")', "a foreign session is exercised");
  contains(cross, "Err(UndoError::ScopeMismatch)", "the foreign session is denied");
  contains(cross, "BatchStatus::Active", "the denied batch is left active");

  var double = span(tests, names[1], "#[test]\n    " + names[2]);
  h.assertEqual(countIn(double, "undo_plan_for(&store"), 2,
    "double undo is attempted twice");
  contains(double, "Err(UndoError::AlreadyUndone)", "the second consume is denied");

  var before = span(tests, names[2], "#[test]\n    " + names[3]);
  contains(before, "before_version_id(&plan[0].before)", "restore target comes from before");
  contains(before, 'encoded[0].get("after").is_none()', "after-state never reaches the plan");
  contains(before, "assert_ne!(", "a different after version is not the restore target");

  contains(ui, "function batchIdFrom(req) {", "UI batch-id resolver ships");
  contains(ui, 'card.setAttribute("data-batch-id", b);', "card carries the explicit batch id");
  contains(ui, 'card.getAttribute("data-batch-id")', "undo affordance reads that batch id");
  contains(ui, 'body: JSON.stringify({ batch_id: id })', "UI sends only the batch id");
  contains(ui, 'fetch("/api/collab/undo"', "UI calls the typed undo endpoint");
  contains(ui, 't("action-undo", "Undo")', "undo affordance uses an existing i18n key");
  var undoUi = span(ui, "  function undoBatch(id, button, card) {", "  window.GBUndoBatch = undoBatch;");
  ["user_id", "bot_id", "session_id", "request_id"].forEach(function (forbidden) {
    absent(undoUi, forbidden, "the UI must not send " + forbidden);
  });
});

h.note("FOLLOW-UP #2448 criterion 4: source assertions only — the live scoped/single-use undo route, its LIFO before-state restore through version-restore, and its UI batch-id affordance. Browser UAT and a recording producer are still required; the driver owns issue closure.");
