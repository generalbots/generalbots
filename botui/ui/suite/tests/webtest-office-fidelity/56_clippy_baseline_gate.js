/**
 * #2543 — deterministic static coverage for the restored clippy baseline CI
 * gate: the [clippy] 0/0/0 table in botserver/clippy-baseline.toml, the
 * preservation of that table (and its documentation) across ratchet
 * regeneration, and the CI job that pipes `cargo clippy --message-format=json`
 * into scripts/check_clippy_baseline.py.
 *
 * All assertions are literal substrings of production source. Nothing is
 * executed: no server, no browser, no cargo, no network. NON-VACUITY: each case
 * first proves the real artifacts ship (the baseline file, the two scripts,
 * the workflow) and extracts a non-empty source span between literal markers;
 * ordering and absence checks are strict. The cases FAIL if the [clippy] table
 * or its zeros are removed, if write_baseline stops preserving unknown TOML
 * sections, if the double-regeneration self-test is deleted, or if the CI job
 * stops feeding clippy JSON into the checker.
 */
"use strict";

var h = require("./00_harness");

var BASELINE = "botserver/clippy-baseline.toml";
var CLIPPY_SCRIPT = "scripts/check_clippy_baseline.py";
var RATCHET_SCRIPT = "scripts/check_rust_error_ratchet.py";
var CI = ".github/workflows/ci.yml";

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

h.staticTest("#2543 botserver/clippy-baseline.toml ships a documented [clippy] table with unwrap_used/expect_used/panic = 0", function () {
  var baseline = h.readRepo(BASELINE);

  // Non-vacuity: this is the real ratchet baseline, not an arbitrary TOML.
  contains(baseline, "issue = 2543", "ratchet baseline file ships");
  contains(baseline, "[rules]", "textual [rules] table ships");
  contains(baseline, "[clippy]", "blocking [clippy] table ships");

  // The documented header explains that these are real clippy diagnostics,
  // counted only in non-test production files, and are shrink-only.
  var header = span(baseline, "# Blocking clippy baseline (issue #2543).", "[clippy]");
  contains(header, "Blocking clippy baseline (issue #2543).", "clippy header names #2543");
  contains(header, "cargo clippy --message-format=json", "header documents the real diagnostic source");
  contains(header, "not textual proxies", "header distinguishes clippy from [rules] proxies");
  contains(header, "non-test production files under `botserver/src` and", "header scopes to production files");
  contains(header, "Shrink-only: lower a number after fixing sites; never raise one by hand.",
    "header documents the shrink-only rule");

  // The [clippy] ceiling table is isolated from [rules]/[files.*] and holds
  // the three real diagnostic codes, each pinned to zero exactly once.
  var clippy = span(baseline, "[clippy]", "[rule_notes]");
  contains(clippy, "unwrap_used = 0", "unwrap_used ceiling is zero");
  contains(clippy, "expect_used = 0", "expect_used ceiling is zero");
  contains(clippy, "panic = 0", "panic ceiling is zero");
  h.assertEqual(countIn(clippy, "unwrap_used"), 1, "unwrap_used declared exactly once in [clippy]");
  h.assertEqual(countIn(clippy, "expect_used"), 1, "expect_used declared exactly once in [clippy]");
  h.assertEqual(countIn(clippy, "panic"), 1, "panic declared exactly once in [clippy]");
  absent(clippy, "unwrap_used = 1", "unwrap_used must never be raised by hand");
  absent(clippy, "expect_used = 1", "expect_used must never be raised by hand");
});

h.staticTest("#2543 write_baseline preserves unknown TOML sections before writing; the double-regeneration preservation self-test is wired", function () {
  var ratchet = h.readRepo(RATCHET_SCRIPT);
  var clippy = h.readRepo(CLIPPY_SCRIPT);

  // Non-vacuity: both scripts and the shared preservation helpers ship.
  contains(ratchet, "from check_clippy_baseline import extract_unknown_sections, self_test_preservation",
    "ratchet imports the preservation helpers");
  contains(ratchet, "def write_baseline(", "ratchet write_baseline ships");
  contains(clippy, "def extract_unknown_sections(raw: str)", "section extractor ships");
  contains(clippy, "def self_test_preservation(check, write_baseline)", "preservation self-test helper ships");

  // write_baseline reads/preserves unknown tables BEFORE it writes the file.
  var writer = span(ratchet, "def write_baseline(", "def show_top(");
  contains(writer, "carry unknown tables (e.g. [clippy]) over verbatim.",
    "write_baseline documents verbatim preservation");
  contains(writer, "extract_unknown_sections(handle.read())", "write_baseline extracts unknown sections");
  contains(writer, "handle.write(content)", "write_baseline writes the regenerated content");
  ordered(writer, "extract_unknown_sections(handle.read())", "handle.write(content)",
    "unknown sections must be captured before the file is rewritten");

  // The preservation self-test is invoked from the ratchet's own self-test.
  contains(ratchet, "self_test_history(check)", "history self-test is wired into the ratchet");
  contains(ratchet, "self_test_preservation(check, write_baseline)",
    "preservation self-test is wired into the ratchet self-test");

  // The helper regenerates TWICE and checks [clippy] + its comment each time.
  var preservation = span(clippy, "def self_test_preservation(check, write_baseline)", "def normalize_rel(");
  contains(preservation, "for generation in (1, 2):", "preservation is tested over two regenerations");
  contains(preservation, "write_baseline(path, totals, {})", "self-test regenerates the baseline");
  contains(preservation, "\\n# doc for clippy\\n[clippy]\\nunwrap_used = 0",
    "self-test appends a documented [clippy] table");
  contains(preservation, "regeneration #{generation} preserves [clippy] and its comment",
    "each regeneration asserts [clippy] survival");
  contains(preservation, "\"# doc for clippy\" in text", "each regeneration asserts the comment survives");
  contains(preservation, "keeps [rules]", "each regeneration asserts generated [rules] survive");
});

h.staticTest("#2543 CI clippy-baseline job pipes cargo clippy JSON into check_clippy_baseline.py and the preservation self-test is wired", function () {
  var ci = h.readRepo(CI);

  // Non-vacuity: the workflow defines the named #2543 gate job.
  contains(ci, "name: Clippy error-handling baseline (#2543)", "clippy-baseline job ships in CI");
  var job = span(ci, "  clippy-baseline:\n", "  reindex:\n");

  // The job is configured (cache scope, advisory-lint rationale) and runs the
  // blocking checker on the exact JSON file it produces.
  contains(job, "name: Clippy error-handling baseline (#2543)", "job name is present in its own span");
  contains(job, "cache-scope: clippy-baseline", "job uses the clippy-baseline cache scope");
  contains(job, "cargo clippy --workspace --exclude botapp --exclude botdevice",
    "clippy runs over the workspace excluding the two named crates");
  contains(job, "--message-format=json > /tmp/clippy-baseline.json 2>/tmp/clippy-baseline.err",
    "clippy JSON diagnostics are written to the baseline JSON file");
  contains(job, "python3 scripts/check_clippy_baseline.py /tmp/clippy-baseline.json",
    "the checker consumes the clippy JSON file");
  ordered(job, "--message-format=json > /tmp/clippy-baseline.json",
    "python3 scripts/check_clippy_baseline.py /tmp/clippy-baseline.json",
    "JSON must be produced before the baseline checker reads it");

  // The ratchet self-test (which includes self_test_preservation) runs in the
  // same CI workflow, so a removed preservation check would fail the gate.
  contains(ci, "python3 scripts/check_rust_error_ratchet.py --self-test",
    "ratchet self-test is wired into CI");
  contains(ci, "python3 scripts/check_rust_error_ratchet.py\n",
    "ratchet production check is wired into CI");
});

h.note("FOLLOW-UP #2543: static coverage for the clippy baseline gate — the documented " +
  "[clippy] 0/0/0 table, verbatim preservation across regeneration (with double-regeneration " +
  "self-test), and the CI job piping clippy JSON into the checker; literal source assertions " +
  "only, no browser/server/cargo run.");
