/**
 * #2448 + #2452 — deterministic static coverage for the shipped typed
 * agent-batch UNDO model and the deterministic ownership-transfer path.
 *
 * Cases (all literal source assertions, no execution):
 *   (a) UndoScope binds user/bot/session, nil and empty ids are rejected,
 *       validate_operations is all-or-nothing and revert_plan is LIFO, with
 *       unit tests for foreign-scope and double-undo rejection;
 *   (b) PgUndoStore persists typed envelopes into consent_audit with distinct
 *       outcome markers and a conditional (single-use) UPDATE, while the UI
 *       attaches data-batch-id from an explicit batch id, never a request id;
 *   (c) transfer.rs ships transfer_ownership with validated targets, unchanged
 *       owner-or-admin authorization and sanitized rejection, and the shared
 *       link remains read-only with role/download gated source bytes;
 *   (d) dto_tests.rs ships the allow_download / role / domain DTO tests.
 *
 * NON-VACUITY: every case first proves the real production files ship with
 * their real function names, struct fields, SQL and test bodies and that the
 * extracted spans are non-empty; absence and ordering checks are then SCOPED
 * to those exact spans/files. Reverting any of these files flips the matching
 * case to FAIL. Source assertions only: no runtime verification is implied;
 * the driver owns issue closure and deployment.
 */
"use strict";

var h = require("./00_harness");

var UNDO_MOD = "botserver/src/core/bot/undo/mod.rs";
var UNDO_STORE = "botserver/src/core/bot/undo/store.rs";
var UNDO_TESTS = "botserver/src/core/bot/undo/tests.rs";
var CONSENT_GATE = "botserver/src/core/bot/consent_gate.rs";
var CONSENT_UI = "botui/ui/suite/chat/modules/40_consent_cards.js";
var TRANSFER = "botserver/src/main_module/routes/collab_routes/transfer.rs";
var SHARED_LINK = "botserver/crates/botdocs/src/shared_link.rs";
var DTO_TESTS = "botserver/src/main_module/routes/collab_routes/dto/dto_tests.rs";

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

// (a) The typed undo model binds the authenticated actor, bot and session,
// refuses nil/empty identities and empty/non-reversible batches, and produces
// a reversed restore plan. The tests pin foreign-scope and double-undo denial.
h.staticTest("#2448 UndoScope binds user/bot/session, nil/empty ids and non-reversible batches are rejected, revert_plan is LIFO, and tests cover foreign scope plus double undo", function () {
  var src = h.readRepo(UNDO_MOD);
  var tests = h.readRepo(UNDO_TESTS);

  h.assert(src.length > 6000, "undo model source must be non-trivial");
  h.assert(tests.length > 6000, "undo unit-test source must be non-trivial");
  contains(src, "//! Durable, session- and bot-scoped agent-batch undo data model (#2448 #4).",
    "undo module names criterion 4");
  contains(src, "pub struct UndoScope {", "UndoScope ships");
  contains(src, "pub struct BatchKey {", "explicit batch identity ships");
  contains(src, "pub struct UndoBatch {", "persisted batch ships");
  contains(src, "pub struct RevertOperation {", "typed revert plan ships");

  var scope = span(src, "pub struct UndoScope {", "impl BatchKey {");
  contains(scope, "pub user_id: Uuid,", "scope binds user id");
  contains(scope, "pub bot_id: Uuid,", "scope binds bot id");
  contains(scope, "pub session_id: String,", "scope binds session id");
  contains(scope, "pub thread_id: Option<String>,", "thread id is informational");
  ordered(scope, "pub user_id: Uuid,", "pub bot_id: Uuid,", "scope field order");
  ordered(scope, "pub bot_id: Uuid,", "pub session_id: String,", "scope field order");

  var validate = span(src, "pub fn validate(&self) -> Result<(), UndoError> {",
    "#[must_use]\n    pub fn matches(");
  contains(validate, "self.user_id.is_nil()", "nil user is refused");
  contains(validate, "self.bot_id.is_nil()", "nil bot is refused");
  contains(validate, "self.session_id.trim().is_empty()", "empty session is refused");
  contains(validate, "self.session_id.len() > MAX_SESSION_CHARS", "oversized session is refused");
  contains(validate, "return Err(UndoError::InvalidScope);", "typed scope failure");
  ordered(validate, "self.user_id.is_nil()", "self.session_id.trim().is_empty()",
    "identity checks precede the session check");

  var batchNew = span(src, "pub fn new(scope: UndoScope, batch_id: &str) -> Result<Self, UndoError> {",
    "/// Unambiguous in-memory key.");
  contains(batchNew, "scope.validate()?;", "a batch can never bypass scope validation");
  contains(batchNew, "batch_id.trim().is_empty()", "empty batch id is refused");
  contains(batchNew, "batch_id.len() > MAX_ID_CHARS", "oversized batch id is refused");
  contains(batchNew, "Err(UndoError::InvalidBatch)", "typed batch failure");
  ordered(batchNew, "scope.validate()?;", "batch_id.trim().is_empty()",
    "scope is validated before the batch id");

  var validateOps = span(src,
    "pub fn validate_operations(operations: &[UndoOperation]) -> Result<(), UndoError> {",
    "#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]");
  contains(validateOps, "if operations.is_empty() {", "empty batches are invalid");
  contains(validateOps, "Err(UndoError::InvalidBatch)", "empty batch fails typed");
  contains(validateOps, "!operation.is_reversible()", "a missing before state invalidates the batch");
  contains(validateOps, "!seen.insert(operation.op_id.as_str())", "duplicate op ids are rejected");
  contains(validateOps, "Err(UndoError::NotReversible)", "irreversible batch fails typed");
  ordered(validateOps, "if operations.is_empty() {", "!operation.is_reversible()",
    "emptiness is checked before per-operation validation");

  var plan = span(src, "pub fn revert_plan(&self) -> Vec<RevertOperation> {",
    "pub struct RevertOperation {");
  contains(plan, "self.operations", "plan is derived from recorded operations");
  contains(plan, ".iter()", "plan walks the operations");
  contains(plan, ".rev()", "plan is reversed (LIFO)");
  contains(plan, "before: op.before.clone(),", "plan carries the recorded before state");
  absent(plan, "after: op.after.clone(),", "after state is diagnostic, never a revert source");
  ordered(plan, ".iter()", ".rev()", "iter must precede rev");

  h.assert(countIn(tests, "#[test]") >= 6, "non-vacuity: undo tests must ship at least 6 cases");
  contains(tests, "fn lookup_rejects_foreign_user_bot_and_session()", "foreign-scope test ships");
  contains(tests, "fn consume_succeeds_once_and_second_undo_is_rejected()", "double-undo test ships");
  contains(tests, "fn revert_plan_is_reverse_lifo_and_carries_only_before_states()", "LIFO test ships");
  contains(tests, "fn nil_identity_and_empty_ids_are_rejected_before_storage()", "nil/empty test ships");

  var foreign = span(tests, "fn lookup_rejects_foreign_user_bot_and_session()",
    "#[test]\nfn consume_succeeds_once_and_second_undo_is_rejected()");
  contains(foreign, "foreign_user", "foreign user is exercised");
  contains(foreign, "foreign_bot", "foreign bot is exercised");
  contains(foreign, "foreign_session", "foreign session is exercised");
  contains(foreign, "Err(UndoError::ScopeMismatch)", "foreign scope is denied");
  contains(foreign, "store.consume(foreign)", "foreign scope is denied on consume too");

  var doubleUndo = span(tests, "fn consume_succeeds_once_and_second_undo_is_rejected()",
    "#[test]\nfn operations_without_a_before_state_cannot_form_a_batch()");
  contains(doubleUndo, "let consumed = store.consume(&batch_key)?;", "first undo succeeds");
  contains(doubleUndo, "consumed.revert_plan().len(), 1", "first undo yields a plan");
  contains(doubleUndo, "Err(UndoError::AlreadyUndone)", "second undo is rejected");
  ordered(doubleUndo, "let consumed = store.consume(&batch_key)?;",
    "Err(UndoError::AlreadyUndone)", "the rejection must follow the successful consume");

  var lifo = span(tests, "fn revert_plan_is_reverse_lifo_and_carries_only_before_states()",
    "#[test]\nfn latest_undo_targets_only_the_newest_active_batch()");
  contains(lifo, 'assert_eq!(plan[0].op_id, "op-2");', "newest op is reverted first");
  contains(lifo, 'assert_eq!(plan[1].op_id, "op-1");', "oldest op is reverted last");
  ordered(lifo, 'assert_eq!(plan[0].op_id, "op-2");', 'assert_eq!(plan[1].op_id, "op-1");',
    "the plan must be reverse insertion order");
});

// (b) The production store shares the consent gate's audit table and uses
// distinct outcome markers plus a conditional UPDATE so two concurrent undos
// cannot both consume one batch. The card derives data-batch-id from an
// explicit batch id and never mistakes a consent request id for one.
h.staticTest("#2448 UndoStore persists to consent_audit with outcome markers and a conditional single-use consume; the UI sets data-batch-id from an explicit batch id, never a request id", function () {
  var src = h.readRepo(UNDO_STORE);
  var gate = h.readRepo(CONSENT_GATE);
  var ui = h.readRepo(CONSENT_UI);

  h.assert(src.length > 8000, "undo store source must be non-trivial");
  h.assert(gate.length > 12000, "consent gate source must be non-trivial");
  h.assert(ui.length > 14000, "consent card source must be non-trivial");
  contains(src, "use botschema::consent_audit;", "store re-exports the shared audit table");
  contains(src, "pub const OUTCOME_ACTIVE: &str = \"undo_batch_active\";", "active outcome marker ships");
  contains(src, "pub const OUTCOME_UNDONE: &str = \"undo_batch_undone\";", "undone outcome marker ships");
  contains(src, "pub trait UndoStore {", "single-use contract ships");
  contains(src, "fn consume(&self, key: &BatchKey) -> Result<UndoBatch, UndoError>;",
    "consume is part of the contract");
  contains(src, "fn record(", "record is part of the contract");

  var pg = span(src, "impl UndoStore for PgUndoStore {", "    fn find(&self, key: &BatchKey)");
  contains(pg, "diesel::insert_into(consent_audit::table)", "batch is persisted into consent_audit");
  contains(pg, ".values(&row)", "typed audit row is inserted");
  contains(pg, "outcome: OUTCOME_ACTIVE.to_string()", "insert marks the row active");
  contains(pg, "UndoRecord::new(batch.clone())", "typed envelope is stored in the request column");
  contains(pg, "user_id: Some(key.scope.user_id)", "audit row carries the owning user");
  ordered(pg, "UndoRecord::new(batch.clone())", "diesel::insert_into(consent_audit::table)",
    "the envelope must be built before the insert");

  var consumeSrc = src.slice(src.indexOf("impl UndoStore for PgUndoStore {"));
  h.assert(consumeSrc.length > 2000, "non-vacuity: pg store impl must be non-trivial");
  var consumeAt = consumeSrc.indexOf("fn consume(&self, key: &BatchKey)");
  h.assert(consumeAt !== -1, "pg consume must ship");
  var consume = consumeSrc.slice(consumeAt);
  h.assert(consume.length > 400, "non-vacuity: pg consume body must be non-trivial");
  contains(consume, "Conditional UPDATE is the single-use guard",
    "single-use rationale ships at the call site");
  contains(consume, "diesel::update(", "consume performs an UPDATE, not an INSERT");
  contains(consume, ".filter(dsl::id.eq(row_id))", "UPDATE is keyed by the exact audit row");
  contains(consume, ".filter(dsl::outcome.eq(OUTCOME_ACTIVE))", "UPDATE is conditional on active");
  contains(consume, ".set(dsl::outcome.eq(OUTCOME_UNDONE))", "UPDATE flips to the undone marker");
  contains(consume, "if changed == 0 {", "zero changed rows is handled");
  contains(consume, "return Err(UndoError::AlreadyUndone);", "second consume fails typed");
  ordered(consume, ".filter(dsl::outcome.eq(OUTCOME_ACTIVE))",
    ".set(dsl::outcome.eq(OUTCOME_UNDONE))", "the active filter must precede the state flip");
  ordered(consume, ".set(dsl::outcome.eq(OUTCOME_UNDONE))", "if changed == 0 {",
    "the row-count check must follow the conditional update");
  h.assert(countIn(src, "UndoError::AlreadyUndone") >= 2,
    "both store implementations must reject a second undo");

  contains(gate, "#[path = \"undo/mod.rs\"]", "undo model is mounted in the consent gate");
  contains(gate, "pub mod undo;", "undo module is public");
  contains(gate, "pub fn shared_service()", "undo store shares the process-wide pool");
  var marker = span(gate,
    "let supplied = request.detail.pointer(\"/params/batch_id\")", "let encoded =");
  contains(marker, "serde_json::Value::as_str", "batch id must be a string");
  contains(marker, "supplied.filter(|id| !id.trim().is_empty())", "empty batch ids are dropped");
  contains(marker, "payload[\"batch_id\"]", "batch id is carried in the marker payload");
  ordered(marker, "request.detail.pointer(\"/params/batch_id\")", "payload[\"batch_id\"]",
    "the explicit param must be read before it is emitted");

  contains(ui, "function batchIdFrom(req) {", "batch-id resolver ships");
  contains(ui, "window.GBConsentBatchId = batchIdFrom;", "resolver is exposed for the renderer");
  var batchId = span(ui, "function batchIdFrom(req) {", "window.GBConsentBatchId = batchIdFrom;");
  contains(batchId, "if (!req || !isConfirmGate(req)) return \"\";", "only confirm-gate cards carry a batch id");
  contains(batchId, "req.batch_id || (d.params || d).batch_id", "only an explicit batch id is accepted");
  contains(batchId, "typeof id === \"string\" && id.trim()", "the batch id is validated");
  contains(batchId, "return \"\";", "a missing batch id yields no attribute");
  absent(batchId, "request_id", "a consent request id must never become a batch id");
  contains(ui, 'card.setAttribute("data-request-id", String(req.request_id));',
    "request id remains a separate attribute");
  contains(ui, "var b = batchIdFrom(req);", "card resolves the batch id");
  contains(ui, 'card.setAttribute("data-batch-id", b);', "card attaches the batch id");
  ordered(ui, "var b = batchIdFrom(req);", 'card.setAttribute("data-batch-id", b);',
    "the attribute must be derived from the resolver, not from the request id");
});

// (c) The transfer handler resolves a validated local target while keeping
// the owner-or-admin rule and a sanitized, generic rejection. The shared-link
// read path remains read-only and gates source bytes by role and download flag.
h.staticTest("#2452 transfer_ownership ships with validated target, owner/admin authorization and sanitized rejection; shared-link read-only guards remain", function () {
  var src = h.readRepo(TRANSFER);
  var shared = h.readRepo(SHARED_LINK);

  h.assert(src.length > 10000, "transfer source must be non-trivial");
  h.assert(shared.length > 12000, "shared-link source must be non-trivial");
  contains(src, "//! #2452 — deterministic ownership transfer for collaboration resources.",
    "transfer module names the shipped slice");
  contains(src, "pub async fn transfer_ownership(", "transfer handler ships");
  contains(src, "const MAX_TARGET_LEN: usize = 254;", "target length is bounded");
  contains(src, "use botsecurity_core::validation::{validate_email, validate_uuid};",
    "shared validators are imported");

  var handler = span(src, "pub async fn transfer_ownership(", "#[cfg(test)]\nmod tests {");
  contains(handler, "sanitize_resource(&req.resource_type, &req.resource_id)", "resource shape is validated");
  contains(handler, "normalize_target_key(&req.new_owner_id)", "target key is normalized");
  contains(handler, "owner_or_admin(&mut conn, &user, &req.resource_type, &req.resource_id)",
    "unchanged owner-or-admin rule is enforced");
  contains(handler, "find_user(&mut conn, &key)", "target resolves in the local users table");
  contains(handler, "caller_identity(&user)", "caller comes from verified claims");
  contains(handler, "authorize_transfer(true, &caller, &target)", "pure authorization core is applied");
  contains(handler, "conn.transaction::<(), diesel::result::Error, _>(", "ACL swap is atomic");
  contains(handler, "DELETE FROM resource_permissions", "old owner grant is removed");
  contains(handler, "INSERT INTO resource_permissions", "new owner grant is written");
  contains(handler, "record_activity(", "transfer is audited");
  contains(handler, '"transfer"', "transfer activity verb ships");
  ordered(handler, "sanitize_resource(&req.resource_type, &req.resource_id)",
    "normalize_target_key(&req.new_owner_id)", "resource is validated before the target");
  ordered(handler, "owner_or_admin(&mut conn, &user, &req.resource_type, &req.resource_id)",
    "find_user(&mut conn, &key)", "authorization precedes target disclosure");
  ordered(handler, "authorize_transfer(true, &caller, &target)", "record_activity(",
    "the transfer is audited only after authorization");

  var normalize = span(src, "pub(crate) fn normalize_target_key(",
    "/// Build the caller identity");
  contains(normalize, "value.len() > MAX_TARGET_LEN", "overlong targets fail closed");
  contains(normalize, "validate_uuid(value).is_ok()", "uuid targets use the shared validator");
  contains(normalize, "validate_email(value).is_ok()", "email targets use the shared validator");
  contains(normalize, "Err(TransferRejection::InvalidTarget)", "invalid targets are typed rejections");

  var authorize = span(src, "pub(crate) fn authorize_transfer(",
    "/// Resolve the normalized key");
  contains(authorize, "if !privileged {", "privilege is required");
  contains(authorize, "Err(TransferRejection::NotAllowed)", "non-owner is denied");
  contains(authorize, "is_self_transfer(caller, target)", "self-transfer is checked");
  contains(authorize, "Err(TransferRejection::SelfTransfer)", "self-transfer is denied");
  ordered(authorize, "if !privileged {", "is_self_transfer(caller, target)",
    "privilege is checked before self-transfer");

  var rejection = span(src, "fn rejection(", "/// `POST /api/collab/permissions/transfer`");
  contains(rejection, "log_and_sanitize_str(", "server-side cause is sanitized");
  contains(rejection, '"collab_routes::transfer_ownership"', "sanitizer is scoped to this handler");
  contains(rejection, 'warn!("collab transfer rejected', "rejection is logged server-side");
  contains(rejection, '"error-collab-invalid-new-owner"', "client sees a generic localized body");
  ordered(rejection, "log_and_sanitize_str(", '"error-collab-invalid-new-owner"',
    "sanitize before returning the generic body");

  h.assert(countIn(src, "#[test]") >= 6, "non-vacuity: transfer tests must ship at least 6 cases");
  contains(src, "fn malformed_and_injection_targets_fail_closed()", "injection-target test ships");
  contains(src, "fn authorization_never_accepts_a_non_owner_or_a_self_transfer()",
    "authorization test ships");
  contains(src, "x'; DROP TABLE users; --", "SQL-injection target is exercised");

  contains(shared, "pub struct SharedDocument {", "shared document response ships");
  contains(shared, "pub read_only: bool,", "read-only marker ships");
  contains(shared, "read_only: true,", "every shared view is read-only");
  contains(shared, "fn role_allows_source(role: &str) -> bool {", "role gate ships");
  contains(shared, '"commenter" | "editor" | "owner"', "only source-capable roles qualify");
  contains(shared, "fn link_is_expired(", "expiry gate ships as a pure helper");
  contains(shared, "fn client_ip(addr: Option<&SocketAddr>)", "audit IP comes from the socket");
  var body = span(shared, "fn body_for_link<'a>(", "/// Unknown, expired, revoked");
  contains(body, "is_valid_token(token)", "bad tokens fail closed");
  contains(body, "!link_is_expired(expires_at, now)", "expired links fail closed");
  contains(body, "allow_download.unwrap_or(true)", "disabled downloads fail closed");
  contains(body, "role_allows_source(role)", "role capability is enforced");
  contains(body, 'if allowed { source } else { "" }', "denied links return an empty body");
  ordered(body, "allow_download.unwrap_or(true)", "role_allows_source(role)",
    "both download policy and role feed the body decision");
  contains(shared, 'ConnectInfo(addr): ConnectInfo<SocketAddr>', "connect info is extracted");
  contains(shared, "let content = body_for_link(", "handler uses the gated body");
  contains(shared, "record_shared_view_with_ip(", "the view is audited with a socket IP");
  h.assert(countIn(shared, "#[test]") >= 10, "non-vacuity: shared-link tests must ship at least 10 cases");
  contains(shared, "fn viewer_link_does_not_release_the_editable_source()", "viewer role test ships");
  contains(shared, "fn download_disabled_commenter_link_releases_no_source()",
    "download-disabled test ships");
});

// (d) The collab DTO tests live in dto/dto_tests.rs (the child module Rust
// actually loads) and pin allow_download, editable-role and grantee-domain
// behavior as deterministic unit tests.
h.staticTest("#2452 dto_tests.rs ships the allow_download / role / domain DTO tests under the loaded dto module", function () {
  var src = h.readRepo(DTO_TESTS);

  h.assert(src.length > 3000, "DTO test source must be non-trivial");
  contains(src, "//! #2452 — deterministic request-DTO tests for the collab routes.",
    "DTO tests name the shipped slice");
  contains(src, "mod dto_tests;", "the module path Rust loads is documented");
  contains(src, "use super::{LinkBody, sanitize_resource, valid_editable_role, valid_grantee_type};",
    "the real DTO helpers are imported");

  h.assert(countIn(src, "#[test]") >= 5, "non-vacuity: at least five DTO tests must ship");
  var names = [
    "fn link_body_without_a_download_policy_keeps_legacy_behaviour()",
    "fn link_body_with_downloads_disabled_is_recorded_explicitly()",
    "fn link_expiry_fields_are_independently_optional()",
    "fn only_editable_roles_pass_the_dto_gate()",
    "fn grantee_types_are_limited_to_user_group_and_domain()",
    "fn resource_validation_accepts_known_shapes_and_refuses_malformed_types()"
  ];
  names.forEach(function (name) {
    contains(src, name, "DTO test must ship: " + name);
  });

  var legacy = span(src, names[0], "#[test]\n" + names[1]);
  contains(legacy, "assert_eq!(body.allow_download, None);", "absent policy is None");
  contains(legacy, "assert!(body.downloads_allowed());", "absent policy keeps legacy behavior");

  var disabled = span(src, names[1], "#[test]\n" + names[2]);
  contains(disabled, '"allow_download": false', "explicit disable is parsed");
  contains(disabled, "assert_eq!(disabled.allow_download, Some(false));", "disable is recorded");
  contains(disabled, "assert!(!disabled.downloads_allowed());", "disable is enforced");
  contains(disabled, '"allow_download": true', "explicit enable is parsed");
  contains(disabled, "assert_eq!(enabled.allow_download, Some(true));", "enable is recorded");
  contains(disabled, "assert!(enabled.downloads_allowed());", "enable is enforced");

  var roles = span(src, names[3], "#[test]\n" + names[4]);
  contains(roles, '["viewer", "commenter", "editor"]', "editable roles are enumerated");
  contains(roles, "assert!(valid_editable_role(role));", "editable roles pass the gate");
  contains(roles, '["", "owner", "admin", "VIEWER", "viewer; drop"]', "unsafe roles are enumerated");
  contains(roles, "assert!(!valid_editable_role(role));", "unsafe roles fail the gate");

  var domains = span(src, names[4], "#[test]\n" + names[5]);
  contains(domains, '["user", "group", "domain"]', "allowed grantee types are enumerated");
  contains(domains, "assert!(valid_grantee_type(value));", "allowed grantee types pass");
  contains(domains, '["", "users", "email", "user;DROP", "Owner"]', "malformed grantee types are enumerated");
  contains(domains, "assert!(!valid_grantee_type(value));", "malformed grantee types fail");
  ordered(domains, '["user", "group", "domain"]', "assert!(valid_grantee_type(value));",
    "the domain gate must use the enumerated allow-list");
});

h.note("FOLLOW-UP #2448/#2452: source assertions only — the scoped undo model, the consent-audit-backed single-use store, the explicit batch-id UI wiring, the deterministic ownership-transfer handler with its read-only shared-link guards, and the loaded collab DTO tests. No runtime verification is implied; the driver owns issue closure.");
