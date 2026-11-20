/**
 * Shared harness for the offline timezone audit (#2389).
 *
 * Read-only. Rust sources are scanned, never compiled here; the only executed
 * code is the shipped suite JS loaded verbatim into a Node vm with stubbed
 * host globals. No production file is edited and no guard/baseline is written.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var TESTS_DIR = path.resolve(__dirname, "..");
var SUITE_DIR = path.resolve(TESTS_DIR, "..");
var UI_DIR = path.resolve(SUITE_DIR, "..");
var REPO_ROOT = path.resolve(UI_DIR, "..", "..");

var PASS = 0;
var FAIL = 0;
var RESULTS = [];
var GAPS = [];
var BUGS = [];
var METRICS = null;

function log(message) {
  console.log(message);
}

function test(name, fn) {
  try {
    fn();
    PASS++;
    RESULTS.push({ name: name, status: "PASS" });
  } catch (e) {
    FAIL++;
    RESULTS.push({ name: name, status: "FAIL", error: e && e.message ? e.message : String(e) });
  }
}

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}

function equal(actual, expected, label) {
  assert(actual === expected,
    (label || "value") + ": expected " + expected + ", got " + actual);
}

function readAbs(absPath) {
  return fs.readFileSync(absPath, "utf8");
}

function suiteFile(rel) {
  return readAbs(path.join(SUITE_DIR, rel));
}

function repoFile(rel) {
  return readAbs(path.join(REPO_ROOT, rel));
}

function suiteExists(rel) {
  return fs.existsSync(path.join(SUITE_DIR, rel));
}

function repoExists(rel) {
  return fs.existsSync(path.join(REPO_ROOT, rel));
}

/** Count regex matches without relying on a shared lastIndex. */
function count(text, re) {
  var pattern = new RegExp(re.source, re.global ? re.flags : re.flags + "g");
  var matches = text.match(pattern);
  return matches ? matches.length : 0;
}

var SKIP_DIRS = {
  target: true, ".git": true, node_modules: true, dist: true,
  "botserver-stack": true, work: true, ".opencode": true,
};

/** Walk *.rs below an absolute root, skipping build/vendor directories. */
function walkRs(absRoot) {
  var out = [];
  function visit(dir) {
    var entries;
    try {
      entries = fs.readdirSync(dir, { withFileTypes: true });
    } catch (e) {
      return;
    }
    entries.forEach(function (entry) {
      if (entry.isDirectory()) {
        if (!SKIP_DIRS[entry.name]) visit(path.join(dir, entry.name));
        return;
      }
      if (entry.isFile() && /\.rs$/.test(entry.name)) {
        out.push(path.join(dir, entry.name));
      }
    });
  }
  visit(absRoot);
  return out;
}

/**
 * Count LINES containing needle, matching the issue's `grep -rn ... | wc -l`
 * methodology (multiple matches on one line count once).
 */
function countInTree(absRoot, needle) {
  return walkRs(absRoot).reduce(function (sum, file) {
    var flags = needle.flags.replace(/g/g, "");
    var pattern = new RegExp(needle.source, flags);
    return sum + readAbs(file).split(/\r?\n/).filter(function (line) {
      return pattern.test(line);
    }).length;
  }, 0);
}

/** Read a JS array literal of quoted strings: `var NAME = [ ... ];`. */
function readStringArray(text, name) {
  var decl = new RegExp("(?:var|let|const)\\s+" + name + "\\s*=\\s*\\[([^\\]]*)\\]");
  var match = decl.exec(text);
  if (!match) return null;
  var literals = match[1].match(/'[^']*'|"[^"]*"/g) || [];
  return literals.map(function (item) { return item.slice(1, -1); });
}

/** Extract one top-level function by name using brace matching (verbatim). */
function extractFunction(text, name) {
  var start = text.indexOf("function " + name + "(");
  if (start < 0) return null;
  var open = text.indexOf("{", start);
  if (open < 0) return null;
  var depth = 0;
  for (var i = open; i < text.length; i++) {
    if (text[i] === "{") depth++;
    if (text[i] === "}") {
      depth--;
      if (depth === 0) return text.slice(start, i + 1);
    }
  }
  return null;
}

/**
 * Load the REAL shipped clock.js into a vm. document.readyState=loading keeps
 * its boot IIFE from touching the DOM; only the pure exported helpers are used.
 */
function makeClockEngine() {
  var source = suiteFile("clock/clock.js");
  var sandbox = {
    console: { log: function () {}, warn: function () {}, error: function () {} },
    Intl: Intl, Date: Date, Math: Math, String: String, Number: Number,
    setInterval: function () { return 0; }, clearInterval: function () {},
    setTimeout: function () { return 0; }, clearTimeout: function () {},
  };
  sandbox.document = {
    readyState: "loading",
    hidden: false,
    addEventListener: function () {},
    getElementById: function () { return null; },
  };
  sandbox.window = {
    document: sandbox.document,
    addEventListener: function () {},
    gbLogError: function () {},
  };
  var context = vm.createContext(sandbox);
  vm.runInContext(source, context, { filename: "suite/clock/clock.js" });
  return { app: sandbox.window.GBClockApp, sandbox: sandbox, context: context };
}

/** Execute the REAL settings timezoneList() helper, extracted verbatim. */
function makeSettingsListEngine() {
  var source = suiteFile("settings/settings-actions.js");
  var fn = extractFunction(source, "timezoneList");
  assert(fn, "shipped timezoneList() helper not found in settings-actions.js");
  var sandbox = { Intl: Intl, Array: Array, console: console };
  var context = vm.createContext(sandbox);
  vm.runInContext(fn, context, { filename: "extracted:timezoneList" });
  return {
    zones: vm.runInContext("timezoneList()", context),
    sandbox: sandbox,
  };
}

/** Register an acceptance criterion the static audit cannot prove shipped. */
function gap(id, statement, reason) {
  GAPS.push({ id: id, statement: statement, reason: reason });
  test("[STATIC] documented gap " + id, function () {
    assert(!!id && !!statement && !!reason, "gap " + id + " is incomplete");
    equal(GAPS.filter(function (g) { return g.id === id; }).length, 1,
      "duplicate gap id " + id);
  });
}

/** Report a real defect with file:line evidence; never patched by this harness. */
function bug(id, statement, evidence) {
  BUGS.push({ id: id, statement: statement, evidence: evidence });
  test("[KNOWN BUG] reported " + id, function () {
    assert(!!id && !!statement && !!evidence, "bug " + id + " is incomplete");
    assert(/:[0-9]+/.test(evidence), "bug " + id + " lacks file:line evidence");
    equal(BUGS.filter(function (b) { return b.id === id; }).length, 1,
      "duplicate bug id " + id);
  });
}

function setMetrics(observed) {
  METRICS = observed;
}

/** Add runtime measurements to the existing metrics object without replacing it. */
function updateMetrics(patch) {
  assert(METRICS, "setMetrics must run before updateMetrics");
  Object.keys(patch).forEach(function (key) { METRICS[key] = patch[key]; });
}

var METRIC_ORDER = [
  "rustFixedOffsetZones", "timezoneModule", "chronoTzDep", "ianaTzDep",
  "zoneIanaVariant", "parseTz", "warnOnUnknown", "localDateHelper",
  "dateNaiveCrates", "dateNaiveBotserver", "dateNaiveRepo", "utcNowDateNaive",
  "uiSettingsFallbacks", "uiClockFallbacks", "uiAutomationZones",
  "uiLanguageStaticOptions", "uiSupportedValuesOf", "automationSilentUtcDefault",
  "runtimeIanaZones", "runtimeNyWinterOffset", "runtimeNySummerOffset",
  "runtimeUnknownZoneOffset",
];

function report() {
  var rule = "=".repeat(72);
  log("\n" + rule);
  log("SUITE TIMEZONE OFFLINE STATIC-AUDIT + RUNTIME HARNESS — ISSUE #2389");
  log(rule);
  log("MODE: read-only source scan + real shipped JS in a Node vm (no browser)");
  log("Rust is SCANNED only (cannot execute in Node); JS helpers are EXECUTED.");
  log("SCOPE: botui/ui/suite UI + botserver Rust; tests subtree excluded from scans.");
  log("LABELS: [BASELINE-OBSERVE] today's counts (regression detection, no ratchet)");
  log("        [RUNTIME] real shipped helper; [STATIC] source assertion only");
  log("        [STATIC][GAP] unmet criterion; [KNOWN BUG] reported, not patched");
  log("");
  METRIC_ORDER.forEach(function (key) {
    log("[OBSERVE] " + key + " = " + METRICS[key]);
  });
  log("");
  ["[BASELINE-OBSERVE]", "[RUNTIME]", "[STATIC]"].forEach(function (prefix) {
    RESULTS.filter(function (r) { return r.name.indexOf(prefix) === 0; })
      .forEach(function (r) {
        log("[" + r.status + "] " + r.name + (r.error ? " -- " + r.error : ""));
      });
  });
  RESULTS.filter(function (r) { return r.status === "FAIL"; })
    .forEach(function (r) { log("[FAIL] " + r.name + " -- " + r.error); });
  log("");
  GAPS.forEach(function (g) {
    log("[STATIC][GAP] " + g.id + ": " + g.statement);
    log("              why: " + g.reason);
  });
  BUGS.forEach(function (b) {
    log("[KNOWN BUG] " + b.id + ": " + b.statement);
    log("            evidence: " + b.evidence);
  });
  log(rule);
  log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL +
    " | GAPS: " + GAPS.length + " | BUGS: " + BUGS.length);
  log("BASELINE-OBSERVE locks characterize this tree; they are not a ratchet.");
  log(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

function finish() {
  if (!METRICS) throw new Error("scanner did not publish metrics");
  report();
}

module.exports = {
  TESTS_DIR: TESTS_DIR, SUITE_DIR: SUITE_DIR, UI_DIR: UI_DIR,
  REPO_ROOT: REPO_ROOT,
  log: log, test: test, assert: assert, equal: equal,
  readAbs: readAbs, suiteFile: suiteFile, repoFile: repoFile,
  suiteExists: suiteExists, repoExists: repoExists,
  count: count, walkRs: walkRs, countInTree: countInTree,
  readStringArray: readStringArray, extractFunction: extractFunction,
  makeClockEngine: makeClockEngine, makeSettingsListEngine: makeSettingsListEngine,
  gap: gap, bug: bug, setMetrics: setMetrics, updateMetrics: updateMetrics,
  finish: finish,
};
