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

test("[BASELINE-OBSERVE] 212 shipped suite HTML files scanned, tests/ excluded", function () {
  var a = all();
  equal(a.files.length, 212, "HTML file count");
  assert(a.files.every(function (rel) { return rel.indexOf("tests/") !== 0 && rel.indexOf("tests/") < 0; }),
    "tests/ leaked into the scan set");
});

test("[BASELINE-OBSERVE] desktop.html: 97 script tags, 96 src, 96 blocking, 1 inline, 0 defer/async", function () {
  var d = measure(DESKTOP);
  equal(d.scripts, 97, "scripts");
  equal(d.scriptSrc, 96, "scriptSrc");
  equal(d.blockingScriptSrc, 96, "blockingScriptSrc");
  equal(d.inlineScript, 1, "inlineScript");
  equal(d.deferAsync, 0, "deferAsync");
});

test("[BASELINE-OBSERVE] desktop.html: 37 stylesheet links", function () {
  equal(measure(DESKTOP).stylesheet, 37, "stylesheet");
});

test("[BASELINE-OBSERVE] partials/chat.html: 56 script tags, 55 src, 42 blocking, 1 inline, 13 defer/async", function () {
  var c = measure(CHAT);
  equal(c.scripts, 56, "scripts");
  equal(c.scriptSrc, 55, "scriptSrc");
  equal(c.blockingScriptSrc, 42, "blockingScriptSrc");
  equal(c.inlineScript, 1, "inlineScript");
  equal(c.deferAsync, 13, "deferAsync");
});

test("[BASELINE-OBSERVE] partials/chat.html: 22 stylesheet links", function () {
  equal(measure(CHAT).stylesheet, 22, "stylesheet");
});

test("[BASELINE-OBSERVE] chat/projector.html: 2 script tags, 2 blocking, 2 stylesheets", function () {
  var p = measure(PROJECTOR);
  equal(p.scripts, 2, "scripts");
  equal(p.scriptSrc, 2, "scriptSrc");
  equal(p.blockingScriptSrc, 2, "blockingScriptSrc");
  equal(p.inlineScript, 0, "inlineScript");
  equal(p.deferAsync, 0, "deferAsync");
  equal(p.stylesheet, 2, "stylesheet");
});

test("[BASELINE-OBSERVE] suite totals: 1125 scripts, 1069 src, 1056 blocking, 56 inline, 378 stylesheets, 13 defer/async", function () {
  var t = all().total;
  equal(t.scripts, 1125, "scripts");
  equal(t.scriptSrc, 1069, "scriptSrc");
  equal(t.blockingScriptSrc, 1056, "blockingScriptSrc");
  equal(t.inlineScript, 56, "inlineScript");
  equal(t.stylesheet, 378, "stylesheet");
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

test("[BASELINE-OBSERVE] issue-era figures (86/32 desktop, 81/19 chat) drifted from today's 96/37 and 42/22", function () {
  var d = measure(DESKTOP);
  var c = measure(CHAT);
  assert(d.blockingScriptSrc !== 86 || d.stylesheet !== 32,
    "desktop counts unexpectedly match the 2025 issue snapshot exactly");
  assert(c.blockingScriptSrc !== 81 || c.stylesheet !== 19,
    "chat counts unexpectedly match the 2025 issue snapshot exactly");
  assert(d.blockingScriptSrc === 96 && d.stylesheet === 37 &&
         c.blockingScriptSrc === 42 && c.stylesheet === 22,
    "current measured counts changed — update this observation deliberately");
});
