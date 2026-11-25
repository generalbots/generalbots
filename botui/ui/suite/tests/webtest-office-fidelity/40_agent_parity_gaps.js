/**
 * [KNOWN BUG] #2448 — source-backed guards for the four still-failing
 * acceptance criteria (browser UAT on v7.1.2, 2026-11-24):
 *   1. "Build a 5-slide deck ... apply the theme ... add a chart from this
 *      sheet" — the catalog ships only CRUD verbs (slides.element.add,
 *      slides.theme.set, sheet.chart.create, docs.track_changes.*). There is no
 *      VM-backed deck builder: no `deck`/`build` verb, no deck native tool in
 *      the agent-VM dispatch, and the only VM route needs a UI click because
 *      DISABLED_MESSAGE says the VM is switched off until Agent (VM) is turned
 *      on in the chat controls.
 *   2. "Comment on clause 4.2 and request review" — docs.comment.add is an
 *      anchored comment (position/length), but no review-request/reviewer/
 *      notification/mention/assignee verb exists in the catalog and its param
 *      list has no reviewer field, so no reviewer can be notified from chat.
 *   3. "Destructive verbs require an explicit confirmation and show a diff
 *      first" — the marker originally had no diff field. A params-derived
 *      optional `preview` has since been added to marker_content, but the chat
 *      card (40_consent_cards.js) still renders only detail/risk and Allow/
 *      Deny buttons: no diff region and no confirm/cancel pair. This guard
 *      passes while EITHER half is missing and flips to FAIL only when both
 *      the marker and the renderer carry a real diff-first gate.
 *   6. "Parity test fails CI when a toolbar action has no verb" —
 *      office_parity_tests.rs checks a hand-maintained EXPECTED snapshot with
 *      no toolbar data-verb/data-action enumeration at all; the toolbar parity
 *      CI step is continue-on-error (non-blocking); and the unit-tests job that
 *      runs the Rust test is gated to github.com, not the ALM CI.
 *
 * NON-VACUITY: every case first proves the shipped source exists and carries
 * its real function/constant/step markers (and that the catalog/directory scan
 * found entries). The gap assertions are then scoped absence checks, so no
 * missing, empty, truncated or garbled file can satisfy a case by accident.
 * When a real fix lands an asserted-in gap disappears, its knownBug fn throws,
 * the case becomes FAIL and this marker must be removed and replaced by a
 * runtime test. No user-facing string, no server, no browser, no cargo.
 */
"use strict";

var h = require("./00_harness");
var fs = h.fs;
var path = h.path;

var COMMANDS_DIR = "botserver/src/apps/commands";
var SLIDES_TABLE = COMMANDS_DIR + "/app_commands_extra/table/sheet_slides_and_canvas.rs";
var CREATE_UTIL = COMMANDS_DIR + "/app_commands_extra/table/office_create_and_utilities.rs";
var REVIEW_OPS = COMMANDS_DIR + "/app_commands_extra/table/office_review_ops.rs";
var PARITY_TESTS = COMMANDS_DIR + "/app_commands_extra/table/office_parity_tests.rs";
var NATIVE_TOOLS = "botserver/src/core/bot/pipeline/native_tools/mod.rs";
var AGENT_TURN = "botserver/src/core/bot/pipeline/agent_turn.rs";
var CONSENT_GATE = "botserver/src/core/bot/consent_gate.rs";
var CONSENT_CARD = "botui/ui/suite/chat/modules/40_consent_cards.js";
var CI = ".github/workflows/ci.yml";

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  h.assert(
    h.readRepo(relPath).indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath
  );
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

/** True when EVERY token is absent from a span (the documented defect). */
function hasNone(span, tokens) {
  return tokens.every(function (token) { return span.indexOf(token) === -1; });
}

/** Assert every token is absent from a span (the documented defect). */
function absent(span, tokens, what) {
  h.assert(
    hasNone(span, tokens),
    (what || "gap") + " must still be absent: found a token from <" + tokens.join(", ") +
      "> :: <" + span.slice(0, 200) + ">"
  );
}

/** Repo-relative .rs files under a directory, in deterministic order. */
function rustSources(relDir) {
  var root = path.join(h.REPO_DIR, relDir);
  var found = [];
  (function walk(dir) {
    fs.readdirSync(dir, { withFileTypes: true })
      .sort(function (a, b) { return a.name < b.name ? -1 : a.name > b.name ? 1 : 0; })
      .forEach(function (entry) {
        var full = path.join(dir, entry.name);
        if (entry.isDirectory()) walk(full);
        else if (entry.name.slice(-3) === ".rs") {
          found.push(path.relative(h.REPO_DIR, full).split(path.sep).join("/"));
        }
      });
  })(root);
  return found;
}

/** Every verb name declared by cmd(app, name, ...) and bound by bind(app, name, ...). */
function catalogNames(relDir) {
  var declared = /\bcmd\(\s*"[^"]+",\s*"([^"]+)"/g;
  var bound = /\bbind\(\s*"[^"]+",\s*"([^"]+)"/g;
  var names = [];
  rustSources(relDir).forEach(function (relPath) {
    var src = h.readRepo(relPath);
    [declared, bound].forEach(function (re) {
      re.lastIndex = 0;
      var match = re.exec(src);
      while (match !== null) {
        names.push({ name: match[1], file: relPath });
        match = re.exec(src);
      }
    });
  });
  return names;
}

// 1. Deck authoring is CRUD-only: there is no single VM-backed deck builder.
h.knownBug("no VM-backed deck builder verb/tool; the VM route still needs a UI toggle", function () {
  contains(SLIDES_TABLE, '"slides.element.add"', "slides element-add verb is shipped");
  contains(SLIDES_TABLE, '"slides.theme.set"', "slides theme verb is shipped");
  contains(CREATE_UTIL, '"sheet.chart.create"', "sheet chart verb is shipped");
  contains(REVIEW_OPS, '"docs.track_changes.list"', "Docs track-changes verbs are shipped");
  contains(NATIVE_TOOLS, "pub async fn try_run(", "native tool dispatch is shipped");
  contains(NATIVE_TOOLS, 'if name == "terminal_run" || name == "vm_exec" {',
    "the only workspace native tools are shipped");
  contains(AGENT_TURN, "pub const DISABLED_MESSAGE: &str =", "VM-off message is shipped");

  var verbs = catalogNames(COMMANDS_DIR);
  h.assert(verbs.length > 0, "catalog scan is not vacuous");
  var builder = verbs.filter(function (entry) { return /deck|build/i.test(entry.name); });
  h.assertEqual(builder.length, 0, "no catalog verb may build a whole deck");

  var dispatch = between(NATIVE_TOOLS, "pub async fn try_run(", "pub(super) async fn send(");
  absent(dispatch, ["deck", "deck_build", "build_deck", "slides.build"],
    "VM-backed deck tool in the native dispatch");

  contains(AGENT_TURN,
    "The VM is switched off for this conversation. Turn on Agent (VM) in the chat controls to run commands or generate files.",
    "the VM gate still requires a UI click");
});

// 2. Anchored comments exist, but there is no review/notification verb and no
//    reviewer field on the comment add, so "request review" cannot notify.
h.knownBug("anchored comment add has no review-request verb or reviewer notification", function () {
  contains(REVIEW_OPS, '"docs.comment.add"', "anchored comment verb is shipped");
  contains(REVIEW_OPS, '"position"', "comment anchors to a character offset");
  contains(REVIEW_OPS, '"length"', "comment anchors to a character range");

  var names = catalogNames(COMMANDS_DIR);
  h.assert(names.length > 0, "catalog scan is not vacuous");
  var reviewVerb = /\bnotify\b|notification|request_review|review_request|reviewer|\bmention\b|assignee|assigned_to|(^|\.)review(\.|$)/i;
  var hits = names.filter(function (entry) { return reviewVerb.test(entry.name); });
  h.assertEqual(hits.length, 0, "no catalog verb requests review or notifies a reviewer");

  var add = between(REVIEW_OPS, '"docs.comment.add",', "    cmd(");
  absent(add, ["reviewer", "notify", "notification", "mention", "assignee", "assigned_to",
    "request_review", "review_request"], "reviewer field on docs.comment.add");
});

// 3. The marker now carries a bounded preview and the chat card renders it,
//    but the card is still a permission GRANT (allow once/run/hour/always/
//    deny) with no explicit confirm/cancel that resumes a staged mutation.
//    This case flips to FAIL when that resolution gate lands.
h.knownBug("destructive consent preview is read-only: no explicit confirm/cancel resolution gate", function () {
  contains(CONSENT_GATE, "pub fn marker_content(", "marker renderer is shipped");
  contains(CONSENT_GATE, 'payload["preview"] = preview;', "marker carries a bounded preview");
  contains(CONSENT_GATE, "GB-CONSENT-REQUEST:{encoded}", "marker protocol is shipped");
  contains(CONSENT_CARD, "function buildPreview(preview)", "card preview renderer is shipped");
  contains(CONSENT_CARD, "gb-consent-preview", "card preview region is shipped");
  contains(CONSENT_CARD, 'data-decision="allow_once"', "grant-based decisions are shipped");

  var card = between(CONSENT_CARD, "  function buildCard(req) {", "  window.GBBuildConsentCard = buildCard;");
  var hasExplicitGate = ["action-confirm", "action-cancel", "decision=confirm", "decision=cancel"]
    .some(function (token) { return card.indexOf(token) !== -1; });
  h.assert(
    !hasExplicitGate,
    "criterion 3 must still lack an explicit confirm/cancel resolution gate"
  );
});

// 6. The Rust parity test is a hand-maintained snapshot with no toolbar scan;
//    the Python toolbar guard is non-blocking and the Rust job is github-only.
h.knownBug("toolbar parity is a hand-maintained snapshot with no blocking CI gate", function () {
  contains(PARITY_TESTS, "const EXPECTED: &[(&str, &str, &str, &str)] = &[",
    "hand-maintained snapshot is shipped");
  contains(PARITY_TESTS,
    "fn office_review_verbs_are_registered_with_stable_names_and_paths()",
    "snapshot test is shipped");
  contains(PARITY_TESTS,
    "fn office_review_summaries_carry_scope_confirmation_and_undo()",
    "summary test is shipped");
  contains(PARITY_TESTS, "fn destructive_office_review_verbs_require_confirmation()",
    "destructive test is shipped");
  contains(CI, "python3 scripts/check_office_toolbar_parity.py", "toolbar guard step is shipped");
  contains(CI, "cargo test -p botserver --bin botserver", "Rust unit-test step is shipped");

  absent(h.readRepo(PARITY_TESTS),
    ["data-verb", "data-action", "scan_toolbar", "botui/ui/suite", "rglob", "read_dir",
      "include_str!"],
    "toolbar action enumeration in office_parity_tests.rs");

  var ci = h.readRepo(CI);
  var stepStart = ci.indexOf('- name: "Guard: office toolbar action -> catalog verb parity (#2448)"');
  h.assert(stepStart !== -1, "toolbar parity CI step is missing");
  var step = ci.slice(stepStart, ci.indexOf("# #2459 —", stepStart));
  h.assert(step.length > 0, "toolbar parity CI step span is empty");
  h.assert(step.indexOf("continue-on-error: true") !== -1,
    "toolbar parity must still be non-blocking in CI");

  var unitStart = ci.indexOf("  unit-tests:");
  h.assert(unitStart !== -1, "unit-tests job is missing");
  var unit = ci.slice(unitStart, ci.indexOf("  # #2074 —", unitStart));
  h.assert(unit.length > 0, "unit-tests job span is empty");
  h.assert(unit.indexOf("if: ${{ github.server_url == 'https://github.com' }}") !== -1,
    "Rust unit tests must still be skipped outside github.com CI");
});

h.note(
  "FOLLOW-UP #2448 criteria 1/2/3/6: add a VM-backed deck builder, a review-request " +
  "notification verb, a real diff-first consent card, and a blocking toolbar->verb CI gate."
);
