/**
 * [BASELINE-OBSERVE] characterizations for #2495.
 *
 * These assert the counts the shipped tree ACTUALLY has today, so any change
 * to the boot graph is surfaced for review. They are observations, not a
 * ratchet: no baseline file is read or written. When production legitimately
 * changes the graph, these exact numbers must be updated in the same PR.
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

test("[BASELINE-OBSERVE] 214 shipped suite HTML files scanned, tests/ excluded", function () {
  var a = all();
  equal(a.files.length, 214, "HTML file count");
  assert(a.files.every(function (rel) { return rel.indexOf("tests/") !== 0 && rel.indexOf("tests/") < 0; }),
    "tests/ leaked into the scan set");
});

test("[BASELINE-OBSERVE] desktop.html: 102 script tags, 101 src, 101 blocking, 1 inline, 0 defer/async", function () {
  var d = measure(DESKTOP);
  equal(d.scripts, 102, "scripts");
  equal(d.scriptSrc, 101, "scriptSrc");
  equal(d.blockingScriptSrc, 101, "blockingScriptSrc");
  equal(d.inlineScript, 1, "inlineScript");
  equal(d.deferAsync, 0, "deferAsync");
});

test("[BASELINE-OBSERVE] desktop.html: 43 stylesheet links", function () {
  equal(measure(DESKTOP).stylesheet, 43, "stylesheet");
});

test("[BASELINE-OBSERVE] partials/chat.html: 56 script tags, 55 src, 42 blocking, 1 inline, 13 defer/async", function () {
  var c = measure(CHAT);
  equal(c.scripts, 56, "scripts");
  equal(c.scriptSrc, 55, "scriptSrc");
  equal(c.blockingScriptSrc, 42, "blockingScriptSrc");
  equal(c.inlineScript, 1, "inlineScript");
  equal(c.deferAsync, 13, "deferAsync");
});

test("[BASELINE-OBSERVE] partials/chat.html: 23 stylesheet links", function () {
  equal(measure(CHAT).stylesheet, 23, "stylesheet");
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

test("[BASELINE-OBSERVE] suite totals: 1160 scripts, 1104 src, 1091 blocking, 56 inline, 510 stylesheets, 13 defer/async", function () {
  var t = all().total;
  equal(t.scripts, 1160, "scripts");
  equal(t.scriptSrc, 1104, "scriptSrc");
  equal(t.blockingScriptSrc, 1091, "blockingScriptSrc");
  equal(t.inlineScript, 56, "inlineScript");
  equal(t.stylesheet, 510, "stylesheet");
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

test("[BASELINE-OBSERVE] issue-era figures (86/32 desktop, 81/19 chat) drifted from today's 101/43 and 42/22", function () {
  var d = measure(DESKTOP);
  var c = measure(CHAT);
  assert(d.blockingScriptSrc !== 86 || d.stylesheet !== 32,
    "desktop counts unexpectedly match the 2025 issue snapshot exactly");
  assert(c.blockingScriptSrc !== 81 || c.stylesheet !== 19,
    "chat counts unexpectedly match the 2025 issue snapshot exactly");
  assert(d.blockingScriptSrc === 101 && d.stylesheet === 43 &&
         c.blockingScriptSrc === 42 && c.stylesheet === 23,
    "current measured counts changed — update this observation deliberately");
});
