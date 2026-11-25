/**
 * [STATIC] deterministic coverage for the Sheet toolbar theming fix: the first
 * toolbar row must follow the office skin, not the light product theme's
 * near-white surface (Docs already did).
 *
 * Every case reads real shipped CSS as text and FAILS if any part of the guard
 * disappears: no runtime claim, no server, no browser, no cargo. Each case
 * first proves the surrounding rule exists (non-vacuity), then asserts literal
 * substrings and their ordering, so a truncated, empty or garbled file can
 * never satisfy a case by accident.
 *
 * Covered:
 *   (a) suite.css paints every office toolbar with a `--surface-raised`
 *       `!important` declaration; `.sheet-app.sheet-app` re-scopes that token
 *       (and `--surface`) to the dark office palette so the declaration
 *       resolves to the office skin rather than near-white.
 *   (b) both `.sheet-toolbar-row-main` and `.sheet-toolbar-row-format` read
 *       their background from `--surface-raised`.
 *   (c) the themed row override has no hardcoded near-white fill, and the
 *       base first-row rule contributes no light background of its own.
 */
"use strict";

var h = require("./00_harness");

var SUITE_CSS = "botui/ui/suite/css/suite.css";
var THEME_CSS = "botui/ui/suite/sheet/css/99_theme.css";
var TOOLBAR_CSS = "botui/ui/suite/sheet/css/01_toolbar.css";

var TOOLBAR_SELECTORS =
  ".docs-toolbar, .sheet-toolbar, .slides-toolbar, .editor-toolbar, .paper-toolbar {";
var ROW_OVERRIDE_START = ".sheet-app.sheet-app .sheet-toolbar-row-main,";
var ROW_OVERRIDE_END = ".sheet-app.sheet-app .sheet-name-input {";

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  var src = h.readRepo(relPath);
  h.assert(src.indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath);
}

/** Literal substring assertion on an already-read source span. */
function spanHas(span, needle, msg) {
  h.assert(span.indexOf(needle) !== -1,
    (msg || "span must contain") + " <" + needle + "> :: " + span.slice(0, 200));
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
    h.assert(at !== -1, (what || "span") + " must contain <" + marker + ">");
    h.assert(at > last, (what || "span") + " order broken at <" + marker + "> :: " + at + " <= " + last);
    last = at;
  });
}

/** Assert every token is absent from a span. */
function absent(span, tokens, what) {
  tokens.forEach(function (token) {
    h.assert(span.indexOf(token) === -1,
      (what || "span") + " must not contain <" + token + "> :: " + span.slice(0, 200));
  });
}

// (a) + (b) suite.css wins the toolbar background with !important, so only the
//     token *values* can change its resolved colour. 99_theme.css scopes those
//     tokens on the doubled .sheet-app selector, and applies the same
//     --surface-raised source to both toolbar rows.
h.staticTest("Sheet toolbar and both rows resolve the suite.css !important surface to the office skin", function () {
  var suite = between(SUITE_CSS, TOOLBAR_SELECTORS,
    ".docs-toolbar .btn-icon, .sheet-toolbar .btn-icon, .slides-toolbar .btn-icon,");
  spanHas(suite, "background: var(--surface-raised, var(--surface)) !important;",
    "office toolbars are painted with the surface tokens under !important");
  spanHas(suite, "border-bottom: 1px solid var(--border-color, var(--border)) !important;",
    "office toolbar separator is !important too");

  var scope = between(THEME_CSS, ".sheet-app.sheet-app {", ".sheet-app.sheet-app .sheet-toolbar {");
  spanHas(scope, "#theme", "suite.css toolbar theming rationale is documented");
  ordered(scope, [
    "--surface: var(--palette-0f172a);",
    "--surface-raised: var(--palette-111827);",
    "--surface-hover: var(--palette-1e293b);"
  ], "scoped office surface tokens");

  // Non-vacuity: both row classes ship in the base toolbar stylesheet.
  contains(TOOLBAR_CSS, ".sheet-toolbar-row {", "shared row base is shipped");
  contains(TOOLBAR_CSS, ".sheet-toolbar-row-main", "first row class is shipped");
  contains(TOOLBAR_CSS, ".sheet-toolbar-row-format", "format row class is shipped");

  var rows = between(THEME_CSS, ROW_OVERRIDE_START, ROW_OVERRIDE_END);
  ordered(rows, [
    ".sheet-app.sheet-app .sheet-toolbar-row-main,",
    ".sheet-app.sheet-app .sheet-toolbar-row-format {",
    "background: var(--surface-raised, var(--palette-111827));"
  ], "themed row override");
});

// (c) The override is token-driven, not a literal light fill, and the base
//     first-row rule has no background of its own to reintroduce near-white.
h.staticTest("No hardcoded near-white background remains on the sheet's first toolbar row", function () {
  var rows = between(THEME_CSS, ROW_OVERRIDE_START, ROW_OVERRIDE_END);
  spanHas(rows, "background: var(--surface-raised, var(--palette-111827));",
    "row background is the themed office surface token");
  absent(rows, ["#fff", "#ffffff", "white", "--palette-ffffff", "--palette-f1f5f9", "--palette-f8fafc"],
    "hardcoded near-white fill in the themed row override");

  var main = between(TOOLBAR_CSS, ".sheet-toolbar-row-main {", ".sheet-toolbar-row-format {");
  spanHas(main, "border-bottom: 1px solid var(--sentient-border, var(--palette-cbd5e1));",
    "first-row base rule is the shipped separator rule");
  absent(main, ["background:", "background-color:"], "hardcoded first-row background in base CSS");
});
