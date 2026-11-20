/**
 * Shared harness for the offline static audit of issue #2528.
 *
 * This harness reads files only. It never imports production JS except through
 * an explicit vm sandbox in 30_vm.js, never writes a baseline/ratchet file and
 * never claims a runtime, network, RSS or browser measurement it cannot make.
 *
 * The source set is the shipped Sheet UI (botui/ui/suite/sheet/**) and the
 * botserver spreadsheet crates (botsheet, botsheet-core) plus the shared upload
 * policy and server layer registrations. tests/ and target/ are excluded.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var REPO_DIR = path.resolve(__dirname, "..", "..", "..", "..", "..");
var SUITE_DIR = path.resolve(__dirname, "..", "..");
var SHEET_DIR = path.join(SUITE_DIR, "sheet");
var RUST_ROOTS = [
  "botserver/crates/botsheet",
  "botserver/crates/botsheet-core",
];

var PASS = 0;
var FAIL = 0;
var RESULTS = [];
var FINDINGS = [];
var SRC_CACHE = Object.create(null);
var FILE_CACHE = Object.create(null);

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
    RESULTS.push({
      name: name,
      status: "FAIL",
      error: e && e.message ? e.message : String(e),
    });
  }
}

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}

function equal(actual, expected, label) {
  assert(
    actual === expected,
    (label || "value") + ": expected <" + expected + ">, got <" + actual + ">"
  );
}

/** Record an observed deviation from the issue text without changing code. */
function finding(name, detail) {
  FINDINGS.push({ name: name, detail: detail });
}

function exists(relPath) {
  return fs.existsSync(path.join(REPO_DIR, relPath));
}

function readRepo(relPath) {
  return fs.readFileSync(path.join(REPO_DIR, relPath), "utf8");
}

/** Cached source read; returns "" when the path is absent. */
function src(relPath) {
  if (!Object.prototype.hasOwnProperty.call(SRC_CACHE, relPath)) {
    SRC_CACHE[relPath] = exists(relPath) ? readRepo(relPath) : "";
  }
  return SRC_CACHE[relPath];
}

/** Cached, sorted, recursive file list. Skips build/vendor/test trees. */
function walk(relRoot, suffix) {
  var cacheKey = relRoot + "|" + suffix;
  if (Object.prototype.hasOwnProperty.call(FILE_CACHE, cacheKey)) {
    return FILE_CACHE[cacheKey];
  }
  var out = [];
  var absRoot = path.join(REPO_DIR, relRoot);
  var skipped = { tests: true, target: true, node_modules: true, vendor: true, ".git": true };
  function visit(dir, prefix) {
    var entries;
    try {
      entries = fs.readdirSync(dir, { withFileTypes: true });
    } catch (e) {
      return;
    }
    entries.forEach(function (entry) {
      var rel = prefix ? prefix + "/" + entry.name : entry.name;
      if (entry.isDirectory()) {
        if (!skipped[entry.name]) visit(path.join(dir, entry.name), rel);
      } else if (entry.isFile() && (!suffix || entry.name.slice(-suffix.length) === suffix)) {
        out.push(relRoot + "/" + rel);
      }
    });
  }
  visit(absRoot, "");
  FILE_CACHE[cacheKey] = out.sort();
  return FILE_CACHE[cacheKey];
}

function sheetJsFiles() {
  return walk("botui/ui/suite/sheet", ".js");
}

function rustFiles() {
  var out = [];
  RUST_ROOTS.forEach(function (root) {
    out = out.concat(walk(root, ".rs"));
  });
  return out.sort();
}

function countMatches(text, regex) {
  var matches = text.match(regex);
  return matches ? matches.length : 0;
}

/** Files in `files` whose content matches `regex`. */
function filesMatching(files, regex) {
  return files.filter(function (rel) {
    regex.lastIndex = 0; // global regexes keep state across .test() calls
    return regex.test(src(rel));
  });
}

/** Fail unless `rel` contains every needle (each checked as a literal string). */
function containsAll(rel, needles, label) {
  var text = src(rel);
  assert(text.length > 0, "missing source file: " + rel);
  needles.forEach(function (needle) {
    assert(
      text.indexOf(needle) >= 0,
      (label || rel) + " does not contain: " + needle
    );
  });
}

/** Fail if `rel` contains any needle. Used for honest gap characterizations. */
function containsNone(rel, needles, label) {
  var text = src(rel);
  needles.forEach(function (needle) {
    assert(
      text.indexOf(needle) < 0,
      (label || rel) + " unexpectedly contains: " + needle
    );
  });
}

/** Load ordered shipped IIFEs verbatim into one vm context. */
function loadModules(relPaths, sandbox) {
  var context = vm.createContext(sandbox);
  relPaths.forEach(function (relPath) {
    vm.runInContext(readRepo(relPath), context, { filename: relPath });
  });
  return context;
}

function report() {
  var js = sheetJsFiles();
  var rs = rustFiles();
  log("");
  log("=".repeat(72));
  log("SHEET LARGE .XLSX LOAD PATH OFFLINE STATIC AUDIT (#2528)");
  log("=".repeat(72));
  log("Scanned shipped sheet JS modules (tests/ excluded): " + js.length);
  log("Scanned botsheet + botsheet-core Rust files: " + rs.length);
  log("This is a static/vm audit: no RSS, timing, byte-size or browser claim is made.");
  log("-".repeat(72));
  RESULTS.forEach(function (r) {
    log("[" + r.status + "] " + r.name + (r.error ? " -- " + r.error : ""));
  });
  if (FINDINGS.length) {
    log("-".repeat(72));
    FINDINGS.forEach(function (f) {
      log("[FINDING] " + f.name + " :: " + f.detail);
    });
  }
  log("=".repeat(72));
  log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  log("=".repeat(72));
  process.exitCode = FAIL > 0 ? 1 : 0;
}

function finish() {
  report();
}

module.exports = {
  fs, path, vm, REPO_DIR, SUITE_DIR, SHEET_DIR, RUST_ROOTS,
  log, test, assert, equal, finding, exists, readRepo, src, walk,
  sheetJsFiles, rustFiles, countMatches, filesMatching,
  containsAll, containsNone, loadModules, finish,
};
