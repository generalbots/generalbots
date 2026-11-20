/**
 * RUNTIME #2533: slow-device gating + baseline table display.
 * Executes shipped modules 88..89; the numbers are explicitly local estimates.
 */
"use strict";

var h = require("./00_harness");
var dom = require("./01_dom");
var f = require("./02_fixture");
var test = h.test;
var assert = h.assert;
var equal = h.assertEqual;
var deep = h.deepEqual;
var fixture = f.fixture;
var byClass = dom.byClass;

test("RUNTIME #2533: size threshold opens large fixtures read-only with an action", function () {
  var NS = fixture().NS;
  var D = NS.Degradation;
  var small = D.evaluate(5, 10);
  equal(small.readOnly, false, "5 MB is inside the 10 MB threshold");
  equal(small.bannerKey, "docs-large-edit-ready", "small banner is a catalog key");
  equal(small.actionKey, null, "no override action when editing is ready");
  var large = D.evaluate(20, 10);
  equal(large.readOnly, true, "20 MB opens read-only");
  equal(large.bannerKey, "docs-large-degraded-banner", "degraded banner is a catalog key");
  equal(large.actionKey, "docs-large-enable-anyway", "explicit actionable override key");
  equal(NS.tr(large.bannerKey, large.bannerFallback),
    "Editing will be slow — opened in read-only preview", "degraded banner text");
  equal(D.evaluate(20).thresholdMb, NS.DEFAULT_THRESHOLD_MB, "default threshold applied");
});

test("RUNTIME #2533: editing is gated until the explicit override is chosen", function () {
  var NS = fixture().NS;
  var state = NS.defaultState();
  var verdict = NS.Degradation.start(20, 10, state);
  equal(state.editingEnabled, false, "start disables editing for a large document");
  equal(NS.Degradation.editable(verdict, state), false, "large document is not editable yet");
  equal(NS.Degradation.enableAnyway(state), true, "override enables editing");
  equal(NS.Degradation.editable(verdict, state), true, "override makes the window editable");
  var ready = NS.Degradation.start(5, 10, state);
  equal(state.editingEnabled, true, "small document starts editable");
  equal(NS.Degradation.editable(ready, state), true, "small document remains editable");
});

test("RUNTIME #2533: baseline rows are deterministic formulas over block count", function () {
  var B = fixture().NS.Baseline;
  var five = B.estimate(5);
  deep(five, {
    size: "5 MB", blocks: 5120, heap: "30 MB", ttfp: "1.0 s",
    keystroke: "8 ms", autosave: "60 ms", wire: "5242880 B",
  }, "5 MB estimate row");
  var twenty = B.estimate(20);
  deep(twenty, {
    size: "20 MB", blocks: 20480, heap: "120 MB", ttfp: "2.8 s",
    keystroke: "14 ms", autosave: "180 ms", wire: "20971520 B",
  }, "20 MB estimate row");
  equal(B.rows().length, 3, "one row per declared size");
  equal(B.rows()[1].size, "10 MB", "middle size present");
  equal(B.localLabel(), "LOCAL ESTIMATE — not measured browser/server numbers",
    "table is labelled as a local estimate");
});

test("RUNTIME #2533: baseline render builds a scoped, i18n-keyed table", function () {
  var fx = fixture();
  var NS = fx.NS;
  var host = fx.doc.createElement("div");
  NS.Baseline.render(host);
  equal(host.children.length, 4, "title, note, table and deferred note");
  var title = host.children[0];
  equal(f.i18nKey(title), "docs-large-baseline-title", "title uses a catalog key");
  equal(title.textContent, "Baseline (local estimate)", "title fallback text");
  var note = host.children[1];
  equal(f.i18nKey(note), "docs-large-baseline-local", "estimate label uses a catalog key");
  equal(note.textContent, NS.Baseline.localLabel(), "label text matches the shipped helper");
  var table = host.children[2];
  equal(table.tagName, "TABLE", "table rendered");
  var headers = table.querySelectorAll("th");
  equal(headers.length, 6, "six declared columns");
  assert(headers.every(function (th) { return th.getAttribute("scope") === "col"; }),
    "every header is scoped to its column");
  var keys = headers.map(function (th) { return f.i18nKey(th); });
  deep(keys, [
    "docs-large-col-size", "docs-large-col-heap", "docs-large-col-ttfp",
    "docs-large-col-keystroke", "docs-large-col-autosave", "docs-large-col-wire",
  ], "column headers use the shipped catalog keys");
  var rows = table.querySelectorAll("tbody")[0].children;
  equal(rows.length, 3, "three size rows");
  equal(rows[0].querySelectorAll("td").length, 6, "six cells per row");
  equal(rows[2].querySelectorAll("td")[0].textContent, "20 MB", "last row is the largest size");
  var deferred = byClass(host, "gb-large-deferred")[0];
  assert(deferred && deferred.textContent.indexOf("Deferred:") === 0,
    "deferred work is stated, not claimed as shipped");
});
