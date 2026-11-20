/**
 * Shared harness for the offline test-floor static audit (#2544).
 *
 * The scanner is deliberately lexical: it counts `#[test]` / `#[tokio::test]`
 * attributes in .rs files, reads Cargo manifests and CI YAML, and reports what
 * is OBSERVED. It never imports or executes production code, never runs cargo
 * and never writes a baseline/ratchet file (those are owned by other changes).
 */
"use strict";

var fs = require("fs");
var path = require("path");

// webtest-test-floor -> tests -> suite -> ui -> botui -> workspace root
var ROOT = path.resolve(__dirname, "..", "..", "..", "..", "..");
var SERVER_DIR = path.join(ROOT, "botserver");
var CRATES_DIR = path.join(SERVER_DIR, "crates");
var BOTTEST_DIR = path.join(ROOT, "bottest");
var INTEGRATION_DIR = path.join(BOTTEST_DIR, "tests", "integration");
var SKIP_DIRS = { target: true, work: true, node_modules: true };

var PASS = 0;
var FAIL = 0;
var RESULTS = [];
var DEFERRED = [];
var BUGS = [];
var MODEL = null;

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

function defer(label, detail) {
  DEFERRED.push({ label: label, detail: detail });
}

function bug(id, detail, evidence) {
  BUGS.push({ id: id, detail: detail, evidence: evidence || [] });
}

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}

function equal(actual, expected, label) {
  assert(actual === expected, (label || "value") + ": expected " + expected + ", got " + actual);
}

function read(relPath) {
  return fs.readFileSync(path.join(ROOT, relPath), "utf8");
}

function exists(relPath) {
  return fs.existsSync(path.join(ROOT, relPath));
}

function findLine(relPath, needle) {
  var lines = read(relPath).split(/\r?\n/);
  for (var i = 0; i < lines.length; i++) {
    if (lines[i].indexOf(needle) >= 0) return i + 1;
  }
  return 0;
}

// ── Lexical Rust scanner ────────────────────────────────────────────────

var TEST_ATTR_RE = /#\s*\[\s*(?:tokio\s*::\s*)?test\b/g;
var TOKIO_ATTR_RE = /#\s*\[\s*tokio\s*::\s*test\b/g;
var IGNORE_ATTR_RE = /#\s*\[\s*ignore\b/g;

function countMatches(text, re) {
  var found = text.match(re);
  return found ? found.length : 0;
}

function walkRs(dir, out) {
  var entries;
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch (e) {
    return;
  }
  entries.forEach(function (entry) {
    var abs = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (!SKIP_DIRS[entry.name]) walkRs(abs, out);
    } else if (entry.isFile() && /\.rs$/.test(entry.name)) {
      out.push(abs);
    }
  });
}

function scanDir(absDir) {
  var files = [];
  walkRs(absDir, files);
  var c = { rsFiles: files.length, tests: 0, tokioTests: 0, ignored: 0, filesWithTests: 0 };
  files.forEach(function (abs) {
    var text;
    try {
      text = fs.readFileSync(abs, "utf8");
    } catch (e) {
      return;
    }
    var n = countMatches(text, TEST_ATTR_RE);
    c.tests += n;
    c.tokioTests += countMatches(text, TOKIO_ATTR_RE);
    c.ignored += countMatches(text, IGNORE_ATTR_RE);
    if (n > 0) c.filesWithTests++;
  });
  return c;
}

function listCrates() {
  var names = fs.readdirSync(CRATES_DIR, { withFileTypes: true })
    .filter(function (e) {
      return e.isDirectory() && fs.existsSync(path.join(CRATES_DIR, e.name, "Cargo.toml"));
    })
    .map(function (e) { return e.name; })
    .sort();
  names.push("botserver", "bottest");
  return names;
}

// ── CRM / Campaign integration scan ─────────────────────────────────────

function scanIntegration() {
  var files = [];
  walkRs(INTEGRATION_DIR, files);
  var rel = files.map(function (abs) { return path.relative(ROOT, abs).split(path.sep).join("/"); });
  function mentions(term) {
    return rel.filter(function (r) {
      var text = fs.readFileSync(path.join(ROOT, r), "utf8");
      return text.toLowerCase().indexOf(term.toLowerCase()) >= 0 &&
        countMatches(text, TEST_ATTR_RE) > 0;
    }).sort();
  }
  function named(term) {
    return rel.filter(function (r) {
      return path.basename(r).toLowerCase().indexOf(term.toLowerCase()) >= 0;
    }).sort();
  }
  return {
    rsFiles: rel.length,
    crmMention: mentions("CRM"),
    campaignMention: mentions("Campaign"),
    crmNamed: named("crm"),
    campaignNamed: named("campaign"),
  };
}

// ── Exemptions / CI / coverage ──────────────────────────────────────────

function readExemptions() {
  var rel = "botserver/test-exemptions.toml";
  if (!exists(rel)) return [];
  var crates = [];
  read(rel).split(/\r?\n/).forEach(function (line) {
    var m = line.match(/^\s*crate\s*=\s*"([^"]+)"/);
    if (m) crates.push(m[1]);
  });
  return crates.sort();
}

function measureAll() {
  var crates = listCrates();
  var perCrate = {};
  crates.forEach(function (name) {
    var dir = name === "botserver" ? SERVER_DIR
      : name === "bottest" ? BOTTEST_DIR
      : path.join(CRATES_DIR, name);
    perCrate[name] = scanDir(dir);
  });
  var zero = crates.filter(function (name) { return perCrate[name].tests === 0; });
  var tested = crates.filter(function (name) { return perCrate[name].tests > 0; });
  var serverManifest = read("botserver/Cargo.toml");
  var exemptions = readExemptions();
  var unexempted = zero.filter(function (name) { return exemptions.indexOf(name) < 0; });
  var ci = read(".github/workflows/ci.yml");
  return {
    crates: crates,
    perCrate: perCrate,
    zero: zero,
    tested: tested,
    libSection: /^\[lib\]\s*$/m.test(serverManifest),
    serverLibFile: exists("botserver/src/lib.rs"),
    serverMainFile: exists("botserver/src/main.rs"),
    exemptions: exemptions,
    unexempted: unexempted,
    integration: scanIntegration(),
    ciBinCommand: ci.indexOf("cargo test -p botserver --bin botserver") >= 0,
    ciCoverage: ci.indexOf("unit-coverage") >= 0 && ci.indexOf("coverage-plan") >= 0,
    agentsDocumentsBin: read("AGENTS.md").indexOf("cargo test -p botserver --bin botserver") >= 0,
    coverageBaseline: exists("issues/coverage/baseline.json"),
  };
}

function all() {
  if (MODEL === null) MODEL = measureAll();
  return MODEL;
}

function report() {
  var m = all();
  var totalTests = m.crates.reduce(function (s, n) { return s + m.perCrate[n].tests; }, 0);
  log("");
  log("=".repeat(68));
  log("TEST FLOOR OFFLINE STATIC AUDIT (#2544)");
  log("=".repeat(68));
  log("Crates scanned (botserver/crates/* + botserver + bottest): " + m.crates.length);
  log("Crates with >=1 #[test]/#[tokio::test]: " + m.tested.length);
  log("Crates with ZERO tests: " + m.zero.length + " -> " + m.zero.join(", "));
  log("Zero-test crates with an exemption: " + m.exemptions.join(", "));
  log("Zero-test crates WITHOUT exemption: " + m.unexempted.length + " -> " + m.unexempted.join(", "));
  log("Total test attributes observed: " + totalTests);
  log("botserver lib target: " + (m.libSection || m.serverLibFile ? "YES" : "NO") +
      " (src/main.rs=" + m.serverMainFile + ", src/lib.rs=" + m.serverLibFile + ")");
  log("Integration .rs files: " + m.integration.rsFiles +
      " | CRM-mention+test: " + m.integration.crmMention.length +
      " | Campaign-mention+test: " + m.integration.campaignMention.length);
  log("Named integration files -> CRM: " + m.integration.crmNamed.length +
      " | Campaign: " + m.integration.campaignNamed.length);
  log("-".repeat(68));
  RESULTS.forEach(function (r) {
    log("[" + r.status + "] " + r.name + (r.error ? " -- " + r.error : ""));
  });
  log("-".repeat(68));
  log("REAL BUGS / GAPS REPORTED (not patched by this audit): " + BUGS.length);
  BUGS.forEach(function (b) {
    log("  [BUG] " + b.id + " :: " + b.detail);
    b.evidence.forEach(function (e) { log("      " + e); });
  });
  log("-".repeat(68));
  log("DEFERRED TO CI / RUNTIME (NOT measured offline, no PASS claimed): " + DEFERRED.length);
  DEFERRED.forEach(function (d) { log("  [CI-ONLY] " + d.label + " :: " + d.detail); });
  log("=".repeat(68));
  log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  log("=".repeat(68));
  process.exitCode = FAIL > 0 ? 1 : 0;
}

function finish() {
  report();
}

module.exports = {
  ROOT, SERVER_DIR, CRATES_DIR, BOTTEST_DIR, INTEGRATION_DIR,
  log, test, defer, bug, assert, equal, read, exists, findLine,
  countMatches, scanDir, all, finish,
};
