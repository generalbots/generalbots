/**
 * [BASELINE-OBSERVE] characterizations for #2495.
 *
 * These assert the counts the shipped tree ACTUALLY has today, so any change
 * to the boot graph is surfaced for review. They are observations, not a
 * ratchet: no baseline file is read or written. When production legitimately
 * changes the graph, these exact numbers must be updated in the same PR.
 *
 * BASELINE REFRESH (2026-11-25): relocked after shipped UI work added two
 * desktop scripts (assistant parity + boot flags), six chat scripts (theme,
 * consent/i18n modules) and one chat stylesheet; no new deferred/blocking
 * regression beyond the observed counts.
 * BASELINE REFRESH (2026-11-24): relocked after legitimate production HTML
 * growth since the previous lock (25b27a776). The +1 file / +5 scripts /
 * +3 stylesheets delta is fully accounted for by shipped pages:
 *   - new settings/partials/16_protection.html  (+1 file, +2 stylesheets)
 *   - docs/docs.html                            (+1 script, +1 stylesheet)
 *   - docs/shared.html                          (+1 script)
 *   - partials/tasks.html                       (+1 script)
 *   - settings/index.html                       (+1 script)
 *   - tasks/tasks.html                          (+1 script)
 * No per-file entry-page baseline changed (desktop/chat/projector counts are
 * unchanged), so this is observation drift only, not a budget regression.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var all = h.all;
var measure = h.measure;

var DESKTOP = "desktop.html";
var CHAT = "partials/chat.html";
var PROJECTOR = "chat/projector.html";

test("[BASELINE-OBSERVE] 215 shipped suite HTML files scanned, tests/ excluded", function () {
  var a = all();
  equal(a.files.length, 215, "HTML file count");
  assert(a.files.every(function (rel) { return rel.indexOf("tests/") !== 0 && rel.indexOf("tests/") < 0; }),
    "tests/ leaked into the scan set");
});

test("[BASELINE-OBSERVE] desktop.html: 104 script tags, 103 src, 103 blocking, 1 inline, 0 defer/async", function () {
  var d = measure(DESKTOP);
  equal(d.scripts, 104, "scripts");
  equal(d.scriptSrc, 103, "scriptSrc");
  equal(d.blockingScriptSrc, 103, "blockingScriptSrc");
  equal(d.inlineScript, 1, "inlineScript");
  equal(d.deferAsync, 0, "deferAsync");
});

test("[BASELINE-OBSERVE] desktop.html: 43 stylesheet links", function () {
  equal(measure(DESKTOP).stylesheet, 43, "stylesheet");
});

test("[BASELINE-OBSERVE] partials/chat.html: 62 script tags, 61 src, 48 blocking, 1 inline, 13 defer/async", function () {
  var c = measure(CHAT);
  equal(c.scripts, 62, "scripts");
  equal(c.scriptSrc, 61, "scriptSrc");
  equal(c.blockingScriptSrc, 48, "blockingScriptSrc");
  equal(c.inlineScript, 1, "inlineScript");
  equal(c.deferAsync, 13, "deferAsync");
});

test("[BASELINE-OBSERVE] partials/chat.html: 24 stylesheet links", function () {
  equal(measure(CHAT).stylesheet, 24, "stylesheet");
});

test("[BASELINE-OBSERVE] chat/projector.html: 2 script tags, 2 blocking, 3 stylesheets", function () {
  var p = measure(PROJECTOR);
  equal(p.scripts, 2, "scripts");
  equal(p.scriptSrc, 2, "scriptSrc");
  equal(p.blockingScriptSrc, 2, "blockingScriptSrc");
  equal(p.inlineScript, 0, "inlineScript");
  equal(p.deferAsync, 0, "deferAsync");
  equal(p.stylesheet, 3, "stylesheet");
});

test("[BASELINE-OBSERVE] suite totals: 1179 scripts, 1126 src, 1113 blocking, 53 inline, 514 stylesheets, 13 defer/async", function () {
  var t = all().total;
  equal(t.scripts, 1179, "scripts");
  equal(t.scriptSrc, 1126, "scriptSrc");
  equal(t.blockingScriptSrc, 1113, "blockingScriptSrc");
  equal(t.inlineScript, 53, "inlineScript");
  equal(t.stylesheet, 514, "stylesheet");
  equal(t.deferAsync, 13, "deferAsync");
});

test("[BASELINE-OBSERVE] entry-page graph is a subset of the suite-wide graph", function () {
  var a = all();
  var d = measure(DESKTOP);
  var c = measure(CHAT);
  var p = measure(PROJECTOR);
  assert(d.blockingScriptSrc + c.blockingScriptSrc + p.blockingScriptSrc <= a.total.blockingScriptSrc,
    "entry-page blocking scripts exceed the suite total");
  assert(d.stylesheet + c.stylesheet + p.stylesheet <= a.total.stylesheet,
    "entry-page stylesheets exceed the suite total");
  assert(d.scripts === d.scriptSrc + d.inlineScript && c.scripts === c.scriptSrc + c.inlineScript,
    "script tag decomposition is inconsistent");
});

test("[BASELINE-OBSERVE] issue-era figures (86/32 desktop, 81/19 chat) drifted from today's 103/43 and 48/24", function () {
  var d = measure(DESKTOP);
  var c = measure(CHAT);
  assert(d.blockingScriptSrc !== 86 || d.stylesheet !== 32,
    "desktop counts unexpectedly match the 2025 issue snapshot exactly");
  assert(c.blockingScriptSrc !== 81 || c.stylesheet !== 19,
    "chat counts unexpectedly match the 2025 issue snapshot exactly");
  assert(d.blockingScriptSrc === 103 && d.stylesheet === 43 &&
         c.blockingScriptSrc === 48 && c.stylesheet === 24,
    "current measured counts changed — update this observation deliberately");
});
