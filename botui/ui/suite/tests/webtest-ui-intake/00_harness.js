/**
 * Mobile native-intake offline static audit (#2510): counters, file access,
 * assertion helpers and the report. No browser, no server, no production write.
 *
 * Kinds reported here:
 *   [OBSERVE]           mechanical observation of the shipped source text
 *   [BASELINE-OBSERVE]  characterization lock of today's observed state
 *                       (a regression detector — NOT a baseline/ratchet file)
 *   [STATIC]            source-only assertion; never a runtime PASS claim
 *   KNOWN BUG           observed defect with file:line, reported, not patched
 *   GAP                 criterion not shippable/provable offline; never PASS
 */
"use strict";

var fs = require("fs");
var path = require("path");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var UI_DIR = path.resolve(SUITE_DIR, "..");
var REPO_ROOT = path.resolve(UI_DIR, "..", "..");
var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];

var DRIVE_HTML = "drive/drive.html";
var DRIVE_EVENTS = "drive/modules/07_events.js";
var MANIFEST = "manifest.webmanifest";
var SERVICE_WORKER = "sw.js";
var SHARE_PAGE = "drive/share-intake.html";
var CHAT_HTML = "partials/chat.html";
var DESKTOP_HTML = "desktop.html";
var SW_REGISTRATION = "desktop-inline/05_offline_shell.js";

function read(absolute) {
  return fs.readFileSync(absolute, "utf8");
}
function suiteFile(relative) {
  return read(path.join(SUITE_DIR, relative));
}
function uiFile(relative) {
  return read(path.join(UI_DIR, relative));
}
function repoFile(relative) {
  return read(path.join(REPO_ROOT, relative));
}
function localeFile(locale, relative) {
  return read(path.join(REPO_ROOT, "botlib", "locales", locale, relative));
}

/* ---------------------------------------------------------------- counters */
var PASS = 0;
var FAIL = 0;
var RESULTS = [];
var GAPS = [];

function record(name, kind, fn) {
  try {
    fn();
    PASS++;
    RESULTS.push({ name: name, kind: kind, status: "PASS" });
  } catch (error) {
    FAIL++;
    RESULTS.push({ name: name, kind: kind, status: "FAIL", error: error && error.message });
  }
}
function test(name, fn) { record("[OBSERVE] " + name, "OBSERVE", fn); }
function baseline(name, fn) { record("[BASELINE-OBSERVE] " + name, "BASELINE-OBSERVE", fn); }
function staticTest(name, fn) { record("[STATIC] " + name, "STATIC", fn); }
function knownBug(name, fn) { record("KNOWN BUG — " + name, "KNOWN BUG", fn); }
function gap(code, criterion, why) {
  GAPS.push({ code: code, criterion: criterion, why: why });
}
function assert(condition, message) {
  if (!condition) throw new Error(message || "Assertion failed");
}
function equal(actual, expected, message) {
  assert(actual === expected,
    (message || "value") + ": expected " + expected + ", got " + actual);
}
function includes(haystack, needle, message) {
  assert(String(haystack).indexOf(needle) !== -1,
    message || "missing " + needle);
}
function lineOf(text, index) {
  return text.slice(0, index).split("\n").length;
}

/* ------------------------------------------------------------------ report */
var gbLog = globalThis.gbLog || { debug: function (line) { process.stdout.write(line + "\n"); } };
function note(line) { gbLog.debug("    " + line); }
function finish() {
  var rule = "=".repeat(70);
  var counts = { OBSERVE: 0, "BASELINE-OBSERVE": 0, STATIC: 0, "KNOWN BUG": 0 };
  RESULTS.forEach(function (result) {
    if (result.status === "PASS" && counts[result.kind] !== undefined) counts[result.kind]++;
  });
  gbLog.debug("\n" + rule);
  gbLog.debug("MOBILE INTAKE — OFFLINE STATIC AUDIT RESULTS (#2510)");
  gbLog.debug(rule);
  RESULTS.forEach(function (result) {
    gbLog.debug("[" + result.status + "] " + result.name +
      (result.error ? " — " + result.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("OBSERVE: " + counts.OBSERVE +
    " | BASELINE-OBSERVE: " + counts["BASELINE-OBSERVE"] +
    " | STATIC: " + counts.STATIC +
    " | KNOWN BUG (documented): " + counts["KNOWN BUG"]);
  gbLog.debug("GAPS (not claimed): " + GAPS.length);
  GAPS.forEach(function (entry) {
    gbLog.debug("  [STATIC][GAP] " + entry.code + ": " + entry.criterion);
    gbLog.debug("                 " + entry.why);
  });
  gbLog.debug("MODE: read-only scan of shipped sources; no browser, no runtime claim");
  gbLog.debug("      [BASELINE-OBSERVE] = characterization lock, not a baseline file");
  gbLog.debug("      [STATIC][GAP] = criterion not shipped/provable offline; never PASS");
  gbLog.debug("      KNOWN BUG = file:line evidence, reported only, never patched here");
  gbLog.debug(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

module.exports = {
  SUITE_DIR: SUITE_DIR,
  UI_DIR: UI_DIR,
  REPO_ROOT: REPO_ROOT,
  LOCALES: LOCALES,
  DRIVE_HTML: DRIVE_HTML,
  DRIVE_EVENTS: DRIVE_EVENTS,
  MANIFEST: MANIFEST,
  SERVICE_WORKER: SERVICE_WORKER,
  SHARE_PAGE: SHARE_PAGE,
  CHAT_HTML: CHAT_HTML,
  DESKTOP_HTML: DESKTOP_HTML,
  SW_REGISTRATION: SW_REGISTRATION,
  suiteFile: suiteFile,
  uiFile: uiFile,
  repoFile: repoFile,
  localeFile: localeFile,
  test: test,
  baseline: baseline,
  staticTest: staticTest,
  knownBug: knownBug,
  gap: gap,
  assert: assert,
  equal: equal,
  includes: includes,
  lineOf: lineOf,
  note: note,
  finish: finish,
};
