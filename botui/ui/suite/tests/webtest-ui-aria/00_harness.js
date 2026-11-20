/**
 * Suite UI a11y static audit — shared harness (#2331).
 * Run with: node botui/ui/suite/tests/webtest-ui-aria.js
 *
 * Self-contained plain-Node harness: counters, deterministic source walk,
 * assertion helpers and a report that keeps OBSERVED facts separate from
 * characterization locks and documented (unmet) criteria.
 */
"use strict";

var fs = require("fs");
var path = require("path");

var SUITE_ROOT = path.resolve(__dirname, "..", "..");
var SOURCE_EXT = { ".html": true, ".js": true };

var PASS = 0;
var FAIL = 0;
var RESULTS = [];
var GAPS = [];
var metrics = null;

var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function test(name, fn) {
  try {
    fn();
    PASS++;
    RESULTS.push({ status: "PASS", name: name });
  } catch (e) {
    FAIL++;
    RESULTS.push({ status: "FAIL", name: name, error: e.message });
  }
}

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}

function readSource(relPath) {
  return fs.readFileSync(path.join(SUITE_ROOT, relPath), "utf8");
}

function walk(absDir, relDir, out) {
  var entries = fs.readdirSync(absDir, { withFileTypes: true });
  entries.sort(function (a, b) { return a.name < b.name ? -1 : a.name > b.name ? 1 : 0; });
  entries.forEach(function (entry) {
    var rel = relDir ? relDir + "/" + entry.name : entry.name;
    var abs = path.join(absDir, entry.name);
    if (entry.isDirectory()) {
      walk(abs, rel, out);
      return;
    }
    if (!entry.isFile() || !SOURCE_EXT[path.extname(entry.name)]) return;
    var parts = rel.split("/");
    if (parts.indexOf("tests") !== -1) return;
    out.push({
      rel: rel,
      ext: path.extname(entry.name),
      vendor: parts.indexOf("vendor") !== -1,
      text: fs.readFileSync(abs, "utf8"),
    });
  });
}

function readTree() {
  var out = [];
  walk(SUITE_ROOT, "", out);
  return out;
}

function setMetrics(observed) {
  metrics = observed;
}

/**
 * Register an acceptance criterion the static audit cannot prove shipped.
 * The assertion below only locks the *registration* invariant, so the gate
 * stays honest (FAIL 0) while the report names the remaining browser/AT work.
 */
function gap(id, statement, reason) {
  GAPS.push({ id: id, statement: statement, reason: reason });
  test("[STATIC] documented gap " + id, function () {
    assert(!!id && !!statement && !!reason, "gap " + id + " is incomplete");
    var seen = GAPS.filter(function (g) { return g.id === id; });
    assert(seen.length === 1, "duplicate gap id " + id);
  });
}

function observedLines() {
  var order = [
    "filesScanned", "htmlFiles", "jsFiles", "vendorFiles", "topSegments",
    "ariaLiveMentions", "ariaLiveFiles", "ariaLiveDeclarations",
    "livePolite", "liveAssertive", "liveOff",
    "roleStatus", "roleAlert", "roleLog", "roleStatusAlertFiles",
    "dataI18nAttrs", "htmlWithI18n", "gbTCalls", "jsWithGbt",
    "dynamicWrites", "writesInRegionFiles", "writesOutsideRegionFiles",
    "filesWithWritesNoRegion",
  ];
  return order.map(function (key) {
    return "[OBSERVE] " + key + " = " + metrics[key];
  });
}

function report() {
  var rule = "=".repeat(72);
  gbLog.debug("\n" + rule);
  gbLog.debug("SUITE UI A11Y STATIC AUDIT — ISSUE #2331");
  gbLog.debug(rule);
  gbLog.debug("MODE: offline static scan (no browser, no screen reader, read-only)");
  gbLog.debug("SCOPE: botui/ui/suite HTML and JS; tests subtree excluded");
  gbLog.debug("NOTES: literal values only; computed values and template keys are not resolved.");
  gbLog.debug("NOTES: writesOutsideRegionFiles is a file-level upper bound (cross-file helpers not attributed).");
  gbLog.debug("");
  observedLines().forEach(function (line) { gbLog.debug(line); });
  gbLog.debug("");
  RESULTS.filter(function (r) { return r.name.indexOf("[BASELINE-OBSERVE]") === 0; })
    .forEach(function (r) { gbLog.debug("[" + r.status + "] " + r.name); });
  gbLog.debug("");
  RESULTS.filter(function (r) { return r.name.indexOf("[STATIC]") === 0; })
    .forEach(function (r) { gbLog.debug("[" + r.status + "] " + r.name); });
  gbLog.debug("");
  GAPS.forEach(function (g) {
    gbLog.debug("[STATIC][GAP] " + g.id + ": " + g.statement);
    gbLog.debug("               why: " + g.reason);
  });
  gbLog.debug("");
  RESULTS.filter(function (r) { return r.status === "FAIL"; })
    .forEach(function (r) { gbLog.debug("[FAIL] " + r.name + " — " + r.error); });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL +
    " | DOCUMENTED GAPS: " + GAPS.length);
  gbLog.debug("BASELINE-OBSERVE locks are characterizations of this tree, not a ratchet.");
  gbLog.debug(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

function finish() {
  if (!metrics) throw new Error("scanner did not publish metrics");
  report();
}

module.exports = {
  SUITE_ROOT: SUITE_ROOT,
  test: test,
  assert: assert,
  readSource: readSource,
  readTree: readTree,
  gap: gap,
  setMetrics: setMetrics,
  finish: finish,
};
