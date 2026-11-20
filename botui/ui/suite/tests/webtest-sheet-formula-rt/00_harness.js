/**
 * Sheet formula round-trip offline harness (#2572).
 *
 * Loads the nine REAL shipped function IIFEs into ONE Node vm sandbox with
 * stubbed window/document/gbT/localStorage and a fixed clock + sheet uid.
 * Nothing is reimplemented: every RUNTIME assertion calls code exported by the
 * production modules. Deferred Rust/CI criteria are labelled [STATIC] source
 * assertions; observed defects are labelled KNOWN BUG and never patched here.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var REPO_ROOT = path.resolve(SUITE_DIR, "..", "..", "..");
var MODULES_REL = "sheet/js/modules-sheet-advanced/";
var FIXED_SHEET_ID = "sheet-2572-fixed";
var FIXED_NOW = Date.UTC(2026, 0, 15, 12, 0, 0);

/* Dependency order: 73 defines the error taxonomy used by 76a, 78 composes
 * every namespace, 79 is the settings module the evaluator reads at call time. */
var ORDER = [
  "73_functions_errors.js",
  "74_functions_financial.js",
  "75_functions_date.js",
  "76a_functions_base.js",
  "76_functions_stat_text.js",
  "77_functions_catalog.js",
  "78a_functions_parser.js",
  "78_functions_evaluator.js",
  "79_functions_settings.js",
];

function readFile(relPath) {
  return fs.readFileSync(path.join(SUITE_DIR, relPath), "utf8");
}

function source(fileName) {
  return readFile(MODULES_REL + fileName);
}

function repoFile(relPath) {
  return fs.readFileSync(path.join(REPO_ROOT, relPath), "utf8");
}

function createEngine() {
  var store = {};
  var events = [];
  var i18nLoads = [];
  var lang = "en";
  var sheetId = FIXED_SHEET_ID;
  var sandbox = {
    console: { log: function () {}, warn: function () {}, error: function () {} },
    localStorage: {
      getItem: function (key) { return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null; },
      setItem: function (key, value) { store[key] = String(value); },
      removeItem: function (key) { delete store[key]; },
      clear: function () { store = {}; },
    },
    CustomEvent: function (type, options) {
      this.type = type;
      this.detail = options && options.detail;
    },
  };
  sandbox.document = {
    documentElement: { getAttribute: function () { return lang; } },
    getElementById: function () { return null; },
    createElement: function () { return { style: {}, setAttribute: function () {}, appendChild: function () {} }; },
    addEventListener: function () {},
  };
  sandbox.window = {
    document: sandbox.document,
    gbT: function (key, fallback) { return fallback === undefined ? key : fallback; },
    SheetCore: { currentSheetId: function () { return sheetId; } },
    SheetI18n: {
      getLocale: function () { return lang; },
      load: function (locale) { i18nLoads.push(locale); lang = locale; },
    },
    dispatchEvent: function (event) { events.push(event); },
  };
  sandbox.gbT = sandbox.window.gbT;
  sandbox.window.localStorage = sandbox.localStorage;

  var context = vm.createContext(sandbox);
  /* Fixed clock inside the vm realm: no shipped module reads wall time, but a
   * patched Date.now keeps any future addition deterministic and host-safe. */
  vm.runInContext("Date.now = function () { return " + FIXED_NOW + "; };", context);
  ORDER.forEach(function (fileName) {
    vm.runInContext(source(fileName), context, { filename: fileName });
  });

  var w = sandbox.window;
  function withPersistedSettings(patch, fn) {
    var settings = w.SheetFunctionsSettings;
    var before = settings.state();
    var next = settings.normalize(Object.assign({}, before, patch));
    settings.save(next);
    try {
      return fn();
    } finally {
      settings.save(before);
    }
  }
  function withSheetId(value, fn) {
    var previous = sheetId;
    sheetId = value;
    try {
      return fn();
    } finally {
      sheetId = previous;
    }
  }
  function makeDate(year, monthIndex, day) {
    return vm.runInContext(
      "new Date(Date.UTC(" + year + "," + monthIndex + "," + day + "))", context);
  }
  return {
    context: context,
    sandbox: sandbox,
    window: w,
    store: store,
    events: events,
    i18nLoads: i18nLoads,
    errors: w.SheetFormulaErrors,
    financial: w.SheetFunctionsFinancial,
    date: w.SheetFunctionsDate,
    base: w.SheetFunctionsBase,
    statText: w.SheetFunctionsStatText,
    catalog: w.SheetFunctionsCatalog,
    parser: w.SheetFunctionsParser,
    evaluator: w.SheetFunctionsEvaluator,
    settings: w.SheetFunctionsSettings,
    withPersistedSettings: withPersistedSettings,
    withSheetId: withSheetId,
    makeDate: makeDate,
    fixedSheetId: FIXED_SHEET_ID,
  };
}

var PASS = 0;
var FAIL = 0;
var RESULTS = [];

function record(name, kind, fn) {
  try {
    fn();
    PASS++;
    RESULTS.push({ name: name, kind: kind, status: "PASS" });
  } catch (error) {
    FAIL++;
    RESULTS.push({ name: name, kind: kind, status: "FAIL", error: error.message });
  }
}

function test(name, fn) { record(name, "RUNTIME", fn); }
function staticTest(name, fn) { record("[STATIC] " + name, "STATIC", fn); }
function knownBug(name, fn) { record("KNOWN BUG — " + name, "KNOWN BUG", fn); }

function assert(condition, message) {
  if (!condition) throw new Error(message || "Assertion failed");
}

function approx(actual, expected, tolerance, message) {
  assert(typeof actual === "number" && Number.isFinite(actual),
    message + ": expected a finite number, got " + String(actual));
  var delta = Math.abs(actual - expected);
  assert(delta <= (tolerance || 1e-9),
    message + ": expected " + expected + ", got " + actual + " (delta " + delta + ")");
}

function errCode(value) {
  return value && value.__gbError === true ? value.code : null;
}

function assertError(value, code, message) {
  assert(errCode(value) === code,
    message + ": expected " + code + ", got " + String(errCode(value)));
}

function sourceHas(fileName, pattern, message) {
  assert(new RegExp(pattern).test(source(fileName)), message);
}

function sourceLacks(fileName, pattern, message) {
  assert(!new RegExp(pattern).test(source(fileName)), message);
}

var engine = createEngine();
var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function note(line) {
  gbLog.debug("    " + line);
}

function finish() {
  var rule = "=".repeat(68);
  var counts = { RUNTIME: 0, STATIC: 0, "KNOWN BUG": 0 };
  RESULTS.forEach(function (result) {
    if (result.status === "PASS" && counts[result.kind] !== undefined) counts[result.kind]++;
  });
  gbLog.debug("\n" + rule);
  gbLog.debug("SHEET FORMULA ROUND-TRIP — OFFLINE TEST RESULTS (#2572)");
  gbLog.debug(rule);
  RESULTS.forEach(function (result) {
    gbLog.debug("[" + result.status + "] " + result.name + (result.error ? " — " + result.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("RUNTIME PASS: " + counts.RUNTIME + " | STATIC PASS: " + counts.STATIC +
    " | KNOWN BUG (documented): " + counts["KNOWN BUG"]);
  gbLog.debug("MODE: one vm sandbox; nine real shipped IIFEs; stubbed window/document/gbT/" +
    "localStorage/fixed clock/fixed uid");
  gbLog.debug("      [STATIC] = source assertions only, no runtime claim");
  gbLog.debug(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

module.exports = {
  engine: engine,
  ORDER: ORDER,
  SUITE_DIR: SUITE_DIR,
  REPO_ROOT: REPO_ROOT,
  FIXED_NOW: FIXED_NOW,
  test: test,
  staticTest: staticTest,
  knownBug: knownBug,
  assert: assert,
  approx: approx,
  errCode: errCode,
  assertError: assertError,
  readFile: readFile,
  repoFile: repoFile,
  source: source,
  sourceHas: sourceHas,
  sourceLacks: sourceLacks,
  note: note,
  finish: finish,
};
