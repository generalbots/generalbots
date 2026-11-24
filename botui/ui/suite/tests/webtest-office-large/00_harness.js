/**
 * Core harness for the #2534 offline Office large-file readiness audit.
 *
 * Reads shipped source only. Records three outcome classes:
 *   PASS - a capability is present and its evidence anchor was found in source.
 *   GAP  - a capability is missing; an explicit anchor documents the absence.
 *   FAIL - a probe itself could not be verified (stale anchor / broken probe).
 *
 * A GAP is a determinate, honest finding and does NOT fail the run. A FAIL means
 * the audit's own assumptions broke and must be fixed. No server, browser,
 * cargo, RSS/timing or byte measurement is fabricated here.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var REPO_DIR = path.resolve(__dirname, "..", "..", "..", "..", "..");
var SUITE_DIR = path.resolve(__dirname, "..", "..");

var PASS = 0;
var FAIL = 0;
var GAP = 0;
var RESULTS = [];
var MATRIX = [];
var SRC_CACHE = Object.create(null);

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}

function equal(actual, expected, label) {
  assert(actual === expected,
    (label || "value") + ": expected <" + expected + ">, got <" + actual + ">");
}

function exists(relPath) {
  return fs.existsSync(path.join(REPO_DIR, relPath));
}

function src(relPath) {
  if (!Object.prototype.hasOwnProperty.call(SRC_CACHE, relPath)) {
    SRC_CACHE[relPath] = exists(relPath) ? fs.readFileSync(path.join(REPO_DIR, relPath), "utf8") : "";
  }
  return SRC_CACHE[relPath];
}

/**
 * Return a concrete file:line reference for the first line matching `pattern`.
 * Throws when the anchor is gone so a stale GAP/PASS is surfaced as FAIL rather
 * than silently reported. `pattern` is a regular-expression source string.
 */
function findLine(relPath, pattern) {
  var text = src(relPath);
  assert(text.length > 0, "evidence file missing: " + relPath);
  var re = new RegExp(pattern);
  var lines = text.split(/\r?\n/);
  for (var i = 0; i < lines.length; i += 1) {
    if (re.test(lines[i])) {
      return { file: relPath, line: i + 1, text: lines[i].trim() };
    }
  }
  throw new Error("evidence anchor not found in " + relPath + " for /" + pattern + "/");
}

function ref(evidence) {
  return evidence.file + ":" + evidence.line;
}

/** Count lines in `relPath` matching `pattern` (0 expected for absence claims). */
function countLines(relPath, pattern) {
  var text = src(relPath);
  if (!text) return 0;
  var re = new RegExp(pattern);
  return text.split(/\r?\n/).filter(function (line) { return re.test(line); }).length;
}

function record(kind, name, detail) {
  RESULTS.push({ status: kind, name: name, detail: detail || "" });
  if (kind === "PASS") PASS += 1;
  else if (kind === "GAP") GAP += 1;
  else FAIL += 1;
}

/**
 * Register one matrix cell. `verify` must assert any anchors it relies on;
 * the matrix cell is recorded only after verification succeeds.
 */
function matrixCell(app, capability, status, verify, evidence, note) {
  try {
    verify();
  } catch (error) {
    record("FAIL", app + " / " + capability + " (probe)", error.message);
    return;
  }
  MATRIX.push({
    app: app,
    capability: capability,
    status: status,
    evidence: evidence ? ref(evidence) : "-",
    quote: evidence ? evidence.text : "",
    note: note || ""
  });
  record(status, app + " / " + capability, (evidence ? ref(evidence) + " - " : "") + (note || ""));
}

/** Deterministic vm context; ordered shipped IIFEs run verbatim. */
function loadModules(relPaths, sandbox) {
  var context = vm.createContext(sandbox);
  relPaths.forEach(function (relPath) {
    vm.runInContext(src(relPath), context, { filename: relPath });
  });
  return context;
}

function runtime(name, fn) {
  try {
    fn();
    record("PASS", "[VM] " + name, "shipped code executed in stubbed vm");
  } catch (error) {
    record("FAIL", "[VM] " + name, error.message);
  }
}

function finish() {
  var rule = "=".repeat(78);
  console.log("\n" + rule);
  console.log("OFFICE LARGE-FILE / LOW-MEMORY READINESS (#2534) - OFFLINE STATIC AUDIT");
  console.log(rule);
  RESULTS.forEach(function (r) {
    console.log("[" + r.status + "] " + r.name + (r.detail ? " :: " + r.detail : ""));
  });

  console.log("\n" + "-".repeat(78));
  console.log("DATA-DRIVEN MATRIX (Docs / Sheets / Slides x capability)");
  console.log("-".repeat(78));
  console.log(
    "APP".padEnd(8) + "CAPABILITY".padEnd(20) + "STATUS".padEnd(7) +
    "EVIDENCE".padEnd(62) + "NOTE"
  );
  MATRIX.forEach(function (row) {
    console.log(
      row.app.padEnd(8) + row.capability.padEnd(20) + row.status.padEnd(7) +
      row.evidence.slice(0, 60).padEnd(62) + row.note.slice(0, 46)
    );
  });

  console.log("\n" + rule);
  console.log("TOTAL: " + (PASS + FAIL + GAP) + " | PASS: " + PASS +
    " | GAP: " + GAP + " | FAIL: " + FAIL);
  console.log("MODE: read-only scan of shipped office UI + Rust import guards; " +
    "public Docs large modules executed in a stubbed Node vm.");
  console.log("GAP = determinate missing capability with a file:line anchor; does not fail the run.");
  console.log("FAIL = stale evidence anchor or broken probe; must be fixed before trusting this audit.");
  console.log("NEEDS A REAL BROWSER / CI: open/parse timing, peak RSS, wire bytes, cgroup admission, " +
    "413/507 behaviour, progress+cancel interaction and 5-100 MB corpus. Not asserted here.");
  console.log(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

module.exports = {
  fs: fs, path: path, vm: vm, REPO_DIR: REPO_DIR, SUITE_DIR: SUITE_DIR,
  assert: assert, equal: equal, exists: exists, src: src,
  findLine: findLine, ref: ref, countLines: countLines,
  matrixCell: matrixCell, loadModules: loadModules, runtime: runtime,
  finish: finish,
  matrix: function () { return MATRIX.slice(); },
  counts: function () { return { PASS: PASS, GAP: GAP, FAIL: FAIL }; }
};
