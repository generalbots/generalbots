/**
 * #2448 — deterministic static coverage for the SHIPPED explicit
 * confirm/cancel consent gate: a class-scoped Rust predicate marks destructive
 * requests, the chat renderer shows only Confirm/Cancel, Confirm stages an
 * allow_once one-shot resume and Cancel denies with no continuation.
 *
 * All assertions are literal substrings of production source / locale
 * catalogs. Nothing is executed: no server, no browser, no cargo, no network.
 * NON-VACUITY: each case first proves the real functions/constants ship and
 * extracts non-empty source spans between literal markers; ordering, exact
 * counts and scoped absence checks are strict. Removing the gate, the class
 * scoping, the marker flag, the confirm-only branch, or the cancel no-execute
 * branch makes the corresponding case FAIL.
 */
"use strict";

var h = require("./00_harness");

var GATE = "botserver/src/core/bot/consent_gate.rs";
var PIPE = "botserver/src/core/bot/pipeline/api_call/handle.rs";
var CARD = "botui/ui/suite/chat/modules/40_consent_cards.js";
var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];

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

h.staticTest("#2448 confirm gate is a class-scoped delete-only predicate, flagged once in the marker and labelled confirm_pending in the pipeline", function () {
  var gate = h.readRepo(GATE);
  var pipe = h.readRepo(PIPE);
  h.assert(gate.length > 0 && pipe.length > 0, "gate and pipeline sources must be non-empty");

  // Non-vacuity: predicate, marker builder and Rust test module all ship.
  contains(gate, "pub fn destructive_confirm_required(action_class: &str) -> bool {",
    "class-scoped predicate ships");
  contains(gate, "pub fn marker_content(request: &botconsent::PendingRequest) -> String {",
    "marker builder ships");
  contains(gate, "#[cfg(test)]", "Rust test module ships");

  // The predicate is a single class equality — no other class is destructive.
  var predicate = span(gate, "pub fn destructive_confirm_required(action_class: &str) -> bool {",
    "/// Upper bound on");
  contains(predicate, 'action_class == "delete"', "only the delete class requires the gate");
  h.assert(predicate.indexOf("||") === -1 && predicate.indexOf("matches!") === -1,
    "predicate must be a single class equality, found <" + predicate + ">");

  // Destructive markers carry the flag exactly once, inside the predicate guard.
  var marker = span(gate, "pub fn marker_content(request: &botconsent::PendingRequest) -> String {",
    "#[cfg(test)]");
  contains(marker, "if destructive_confirm_required(&request.action_class) {",
    "marker gates the flag on the predicate");
  ordered(marker, "if destructive_confirm_required(&request.action_class) {",
    'payload["confirm_required"] = serde_json::json!(true);',
    "flag assignment must be inside the predicate guard");
  h.assertEqual(countIn(marker, 'payload["confirm_required"]'), 1,
    "confirm_required flag is set exactly once");
  contains(marker, "if let Some(preview) = consent_preview(&request.detail) {",
    "optional preview ships with the gate");

  // The 2 new Rust tests pin class scoping and the delete-only marker flag.
  var tests = gate.slice(gate.indexOf("#[cfg(test)]"));
  h.assertEqual(countIn(tests, "fn destructive_confirm_gate_is_class_scoped() {"), 1,
    "class-scope test declared once");
  h.assertEqual(countIn(tests, "fn marker_flags_confirm_gate_only_for_destructive_class() {"), 1,
    "marker-flag test declared once");
  contains(tests, 'assert!(destructive_confirm_required("delete"));',
    "test asserts delete requires the gate");
  contains(tests, '"read", "create", "update", "share", "publish", "payment", "", "unknown",',
    "test enumerates non-destructive classes");
  contains(tests, "!destructive_confirm_required(class),",
    "test asserts other classes stay legacy");
  contains(tests, 'assert_eq!(destructive["confirm_required"], serde_json::json!(true));',
    "test asserts the destructive marker flag");
  contains(tests, 'read.get("confirm_required").is_none(),',
    "test asserts the read marker has no flag");
  contains(tests, 'read.get("preview").is_none(),',
    "test asserts the read marker has no destructive preview");

  // The pipeline reuses the same predicate for the permission activity label.
  contains(pipe,
    'destructive_confirm_required(action_class) { "confirm_pending" } else { "pending" }',
    "pending activity is labelled confirm_pending only for the gate");
  contains(pipe, "permission_summary(&name, pending_label,",
    "pending label feeds the permission activity summary");
  h.assert(countIn(pipe, "permission_summary(&name, pending_label,") >= 2,
    "label feeds both started and needs_input activities");
});

h.staticTest("#2448 destructive cards render only Confirm/Cancel with gb-consent-confirm-gate and i18n keys in all 8 catalogs", function () {
  var src = h.readRepo(CARD);
  h.assert(src.length > 0, "consent card renderer must be non-empty");

  // Non-vacuity: gate predicate, card builder and public hook all ship.
  contains(src, "function isConfirmGate(r) {", "gate predicate ships in the renderer");
  contains(src, "function buildCard(req) {", "card builder ships");
  contains(src, "window.GBBuildConsentCard = buildCard;", "card builder hook ships");

  var predicate = span(src, "function isConfirmGate(r) {", "var lastConsentRequest = null;");
  contains(predicate, 'r.confirm_required === true || r.action_class === "delete"',
    "renderer recognises both the marker flag and the delete class");

  var actions = span(src, "var actions = confirmGate", "var card = document.createElement(\"div\");");
  var elseMarker = ": '<button type=\"button\" class=\"gb-consent-btn primary\" data-decision=\"allow_once\">";
  var elseIdx = actions.indexOf(elseMarker);
  h.assert(elseIdx !== -1, "non-destructive grant branch must exist");
  var confirmBranch = actions.slice(0, elseIdx);
  var grantBranch = actions.slice(elseIdx);

  // Destructive branch: exactly two buttons, Confirm + Cancel, no grants.
  h.assertEqual(countIn(confirmBranch, 'data-decision="'), 2,
    "destructive card renders exactly two buttons");
  contains(confirmBranch, 'data-decision="confirm"', "Confirm button ships in the gate branch");
  contains(confirmBranch, 'data-decision="cancel"', "Cancel button ships in the gate branch");
  contains(confirmBranch, 't("action-confirm", "Confirm")', "Confirm label is localised");
  contains(confirmBranch, 't("action-cancel", "Cancel")', "Cancel label is localised");
  ["allow_once", "allow_run", "allow_workspace", "always", "deny", "grant"]
    .forEach(function (forbidden) {
      absent(confirmBranch, forbidden, "destructive branch must not offer " + forbidden);
    });

  // The gate CSS class is tied to the predicate, not painted on every card.
  contains(src, 'card.className = "gb-consent-card" + (confirmGate ? " gb-consent-confirm-gate" : "");',
    "gate class is conditional on isConfirmGate");

  // Cancelled status is localised; all three gate keys ship in every catalog.
  contains(src, 't("status-cancelled", "Cancelled")', "cancelled status is localised");
  var keys = ["action-confirm", "action-cancel", "status-cancelled"];
  var seen = {};
  LOCALES.forEach(function (loc) {
    var rel = "botlib/locales/" + loc + "/common.ftl";
    var catalog = h.readRepo(rel);
    h.assert(catalog.length > 0, rel + " must be a non-empty catalog");
    keys.forEach(function (key) {
      var match = catalog.match(new RegExp("^" + key + "\\s*=\\s*(\\S.*)$", "m"));
      h.assert(match !== null, rel + " must define " + key + " on its own line");
      h.assert(match[1].trim().length > 0, rel + " " + key + " value must not be empty");
      seen[match[1].trim()] = true;
    });
  });
  h.assert(Object.keys(seen).length >= 3,
    "the three gate keys must carry real, per-locale translations");
});

h.staticTest("#2448 Confirm stages allow_once one-shot resume; Cancel maps to deny with no sendMessage; legacy cards keep five grants", function () {
  var src = h.readRepo(CARD);
  var gate = h.readRepo(GATE);
  h.assert(src.length > 0 && gate.length > 0, "renderer and gate sources must be non-empty");

  // Non-vacuity: wire mapping, resolver and server one-shot consumer ship.
  contains(src, 'var CONFIRM_WIRE_DECISION = "allow_once", CANCEL_WIRE_DECISION = "deny";',
    "confirm/cancel wire mapping ships");
  contains(src, "function wireDecision(d) {", "decision wire adapter ships");
  contains(src, "function resolve(card, requestId, decision) {", "consent resolver ships");
  contains(gate, "botconsent::enforce::take_approved_match(service, user_id, app_id, action_class).await",
    "server consumes the staged one-shot approval");
  contains(gate, "approved_detail: Some(approved.detail),",
    "approved payload travels with the staged grant");

  var wire = span(src, "function wireDecision(d) {", "function isConfirmGate(r) {");
  contains(wire, 'd === "confirm" ? CONFIRM_WIRE_DECISION',
    "Confirm maps to the allow_once wire decision");
  contains(wire, 'd === "cancel" ? CANCEL_WIRE_DECISION',
    "Cancel maps to the deny wire decision");

  // Non-destructive cards keep all five legacy grant buttons.
  var actions = span(src, "var actions = confirmGate", "var card = document.createElement(\"div\");");
  var elseMarker = ": '<button type=\"button\" class=\"gb-consent-btn primary\" data-decision=\"allow_once\">";
  var grantBranch = actions.slice(actions.indexOf(elseMarker));
  h.assertEqual(countIn(grantBranch, 'data-decision="'), 5,
    "non-destructive card keeps exactly five grant buttons");
  ["allow_once", "allow_run", "allow_workspace", "always", "deny"].forEach(function (decision) {
    contains(grantBranch, 'data-decision="' + decision + '"',
      "grant button " + decision + " stays on legacy cards");
  });
  absent(grantBranch, 'data-decision="confirm"', "legacy cards must not expose the gate Confirm button");
  absent(grantBranch, 'data-decision="cancel"', "legacy cards must not expose the gate Cancel button");

  var resolve = span(src, "function resolve(card, requestId, decision) {",
    "// Renders one request into a message; idempotent per request id.");
  contains(resolve, "decision: wireDecision(decision)",
    "resolver wires the staged decision into the POST");
  ordered(resolve, 'if (decision === "cancel") {', '} else if (decision === "confirm") {',
    "Cancel branch must precede Confirm branch");

  // Cancel: deny, clear the staged request, report status, execute nothing.
  var cancel = span(resolve, 'if (decision === "cancel") {', '} else if (decision === "confirm") {');
  contains(cancel, "lastConsentRequest = null;", "Cancel clears the staged request");
  contains(cancel, 't("status-cancelled", "Cancelled")',
    "Cancel reports the cancelled status without resuming");
  absent(cancel, "sendMessage", "Cancel must send no continuation message");
  absent(cancel, "resumeAfterGrant", "Cancel must never take the grant-resume path");
  absent(cancel, "wireDecision(", "Cancel stages no further decision client-side");

  // Confirm: one-shot approval posted first, then the explicit resume message.
  var confirm = span(resolve, '} else if (decision === "confirm") {', "} else {");
  contains(confirm, 'typeof window.sendMessage === "function"',
    "Confirm resumes through sendMessage");
  contains(confirm,
    'window.sendMessage("Confirmed. Continue and run the previously requested command now with the same parameters.");',
    "Confirm sends the one-shot continuation");
  ordered(resolve, "body: JSON.stringify({ request_id: requestId, decision: wireDecision(decision) }),",
    '} else if (decision === "confirm") {',
    "staged decision must be posted before the resume branch");

  // The legacy else branch still resumes on grants; the gate branch never does.
  h.assertEqual(countIn(resolve, "resumeAfterGrant(decision);"), 1,
    "grant resume belongs only to the final legacy branch");
});

h.note("FOLLOW-UP #2448: these are literal source/catalog assertions only — the runtime " +
  "confirm/cancel dialog still needs a browser UAT; no server, browser or cargo was run.");
