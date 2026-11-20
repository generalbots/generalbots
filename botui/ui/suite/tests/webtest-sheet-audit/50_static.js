/**
 * [STATIC] source assertions for criteria that cannot be proven in a Node vm,
 * plus KNOWN BUG characterizations. No [STATIC] test claims runtime PASS and no
 * production defect is patched here. Source files are read only.
 */
"use strict";

var h = require("./00_harness");
var assert = h.assert;
var equal = h.equal;

h.staticTest("sheet.html loads all nine modules in the required dependency order", function () {
  var html = h.readSuite("sheet/sheet.html");
  var positions = h.ORDER.map(function (file) {
    var token = "modules-sheet-advanced/" + file;
    var index = html.indexOf(token);
    assert(index >= 0, "sheet.html does not load " + file);
    return index;
  });
  for (var i = 1; i < positions.length; i++) {
    assert(positions[i] > positions[i - 1],
      h.ORDER[i] + " must load after " + h.ORDER[i - 1] + " (parser before engine)");
  }
  assert(html.indexOf("/suite/sheet/css/09_sheet_audit.css") >= 0, "audit stylesheet linked");
  assert(html.indexOf('id="sheetAuditBtn"') >= 0, "toolbar entry present");
  assert(html.indexOf('data-i18n-title="sheet-audit-open"') >= 0, "button title is localized");
  assert(html.indexOf('data-i18n-aria-label="sheet-audit-open"') >= 0, "button aria-label is localized");
});

h.staticTest("all nine shipped modules stay below the 450-line limit", function () {
  h.ORDER.forEach(function (file) {
    var lines = h.source(file).split("\n").length;
    assert(lines < 450, file + " has " + lines + " lines");
  });
});

h.staticTest("all eight locale catalogs define an identical sheet-audit key set", function () {
  var locales = ["de", "en", "es", "fr", "ja", "ko", "pt-BR", "zh-CN"];
  var reference = null;
  locales.forEach(function (locale) {
    var text = h.readRoot("botlib/locales/" + locale + "/sheet-audit.ftl");
    var keys = text.split("\n").map(function (line) {
      var match = /^([a-z0-9][a-z0-9.-]*)\s*=/.exec(line);
      return match ? match[1] : null;
    }).filter(Boolean).sort();
    assert(keys.length >= 100, locale + " has only " + keys.length + " audit keys");
    if (!reference) reference = keys;
    else assert(JSON.stringify(keys) === JSON.stringify(reference), locale + " key set differs from en");
  });
  assert(reference.indexOf("sheet-audit-cause-div0") >= 0, "cause keys present");
  assert(reference.indexOf("sheet-audit-fix-cycle") >= 0, "fix keys present");
  assert(reference.indexOf("sheet-audit-tab-trace") >= 0, "tab keys present");
});

h.staticTest("no shipped audit module writes a hardcoded English literal into the DOM", function () {
  h.ORDER.forEach(function (file) {
    var text = h.source(file);
    assert(!/\.(textContent|innerText|innerHTML|title)\s*=\s*["'][A-Za-z]{4,}/.test(text),
      file + " assigns a literal user-facing string");
    assert(!/setAttribute\(\s*["'](title|placeholder|aria-label)["']\s*,\s*["'][A-Za-z]{4,}/.test(text),
      file + " sets a literal accessibility label");
    assert(!/\b(alert|confirm|notify|toast)\(\s*["'][A-Za-z]{4,}/.test(text),
      file + " shows a literal notification");
    assert(!/data-i18n=["'][a-z-]+["']\s*\+/.test(text),
      file + " concatenates a dynamic i18n key");
  });
});

h.staticTest("local-only boundary is stated in shipped source and UI copy", function () {
  ["25_audit_graph.js", "26_audit_eval.js", "27_audit_panel.js", "29_audit_overlay.js", "30_audit_boot.js"]
    .forEach(function (file) {
      h.sourceHas(file, "local-only", file + " must declare its local-only scope");
    });
  h.sourceHas("27_audit_panel.js", "no server graph or evaluator parity is implied",
    "the dialog states the backend parity boundary");
  h.sourceHas("29_audit_overlay.js", "Never writes",
    "the overlay documents that it is view-only");
});

h.staticTest("server graph exposure, evaluator parity and cross-sheet recalc are deferred", function () {
  h.sourceHas("27_audit_panel.js", "no server graph or evaluator parity is implied",
    "no server parity is claimed by the shipped UI");
  h.sourceHas("25_audit_graph.js", "cells already held in memory",
    "graph source is explicitly the in-memory workbook");
  h.sourceHas("28_audit_sections.js", "makeEvalContext",
    "evaluation context is local, not a server call");
  h.rootHas("botserver/crates/botsheet-core/src/dependency_graph/incremental.rs",
    "pub fn find_cycles_in", "the Rust dependency graph is a separate backend crate");
  h.sourceLacks("30_audit_boot.js", "fetch\\(", "boot does not call a server graph API");
});

h.staticTest("watch refresh uses rAF coalescing; <=200 ms timing needs browser UAT", function () {
  h.sourceHas("29_audit_overlay.js", "requestAnimationFrame", "watch refresh is frame-scheduled");
  h.sourceHas("29_audit_overlay.js", "watchQueued", "duplicate scheduling is suppressed");
  h.sourceLacks("29_audit_overlay.js", "setTimeout\\(.*200", "no unverified fixed 200 ms timer is shipped");
  h.sourceHas("29_audit_overlay.js", "MutationObserver", "grid mutations are observed");
  h.sourceHas("30_audit_boot.js", "gb-sheet-selection", "selection events trigger rerender");
});

h.staticTest("error inspector over 200k formulas needs browser/CI; budgets are explicit", function () {
  h.sourceHas("25_audit_graph.js", "MAX_GRAPH_CELLS = 250000",
    "graph can hold a 200k-cell workbook");
  h.sourceHas("25_audit_graph.js", "MAX_RANGE_CELLS = 4000", "a single range is bounded");
  h.sourceHas("25_audit_graph.js", "MAX_CYCLES = 40", "cycle report is bounded");
  h.sourceHas("28a_audit_sections2.js", "MAX_EVAL_CELLS = 400", "evaluation scan is bounded");
  h.sourceHas("26a_audit_engine.js", "MAX_RANGE", "range expansion is bounded in the evaluator");
});

h.staticTest("overlay and circular navigator are view-only; CDP verifies pixels", function () {
  h.sourceHas("29_audit_overlay.js", "createElementNS", "arrows are decorative SVG");
  h.sourceHas("29_audit_overlay.js", "aria-hidden", "arrows are hidden from assistive tech");
  h.sourceHas("28_audit_sections.js", "S.jumpTo = jumpTo", "circular rows navigate via selection");
  h.sourceHas("28_audit_sections.js", "setRange", "navigation selects a cell, never edits it");
  h.sourceLacks("29_audit_overlay.js", "loadedSheet\\s*=|\\.value\\s*=\\s*node", "overlay never assigns cell data");
});

h.staticTest("the harness is self-contained and sets a real exit code", function () {
  var runner = h.readSuite("tests/webtest-sheet-audit.js");
  assert(runner.indexOf('require("./webtest-sheet-audit/00_harness")') >= 0, "harness is required");
  ["10_graph.js", "20_eval.js", "30_reports.js", "40_ui.js", "50_static.js"].forEach(function (file) {
    assert(runner.indexOf('"./webtest-sheet-audit/' + file.replace(/\.js$/, '') + '"') >= 0,
      file + " is loaded by the runner");
  });
  assert(runner.indexOf("harness.finish()") >= 0, "report is finalized");
  assert(/process\.exitCode = FAIL > 0 \? 1 : 0/.test(h.readSuite("tests/webtest-sheet-audit/00_harness.js")),
    "FAIL results produce a non-zero exit code");
});

h.knownBug("Rust find_cycles_in reports cycle dependents as members", function () {
  var text = h.readRoot("botserver/crates/botsheet-core/src/dependency_graph/incremental.rs");
  assert(/Kahn-style cycle detection over a `referenced -> dependents` map/.test(text),
    "the algorithm description is present");
  assert(/in_degree\.entry\(\*d\)\.or_insert\(0\) \+= 1/.test(text),
    "incoming edges are counted from the dependents map");
  assert(/filter\(\|k\| !removed\.contains\(k\)\)/.test(text),
    "survivors after Kahn stripping are reported as cycles");
  h.sourceHas("25a_audit_reports.js", "color\\[neighbor\\] === 1",
    "the JS auditor uses DFS coloring instead and does not enshrine the false positive");
});

var e = h.createEngine();

h.knownBug("dividing by a non-numeric cell returns #DIV/0! instead of #VALUE!", function () {
  var book = h.makeBook();
  book.worksheets[0].data["0,0"] = { value: "hello", formula: "" };
  var ctx = e.sections.makeEvalContext(e.graph.normalizeSnapshot(book, 0), 0);
  var result = e.eval.evaluate("=1/A1", ctx);
  equal(result.error, "#DIV/0!", "characterize the defect: text denominator is treated as zero");
  h.sourceHas("26a_audit_engine.js", 'b === 0 \\|\\| b === null \\? err\\("#DIV/0!"\\)',
    "the zero/null check ignores the non-numeric case");
});

h.knownBug("arithmetic coerces text cells to zero instead of #VALUE!", function () {
  var book = h.makeBook();
  book.worksheets[0].data["0,0"] = { value: "hello", formula: "" };
  var ctx = e.sections.makeEvalContext(e.graph.normalizeSnapshot(book, 0), 0);
  equal(e.eval.evaluate("=A1+1", ctx).value, "1",
    "characterize the defect: local + coerces non-numeric text to zero");
});
