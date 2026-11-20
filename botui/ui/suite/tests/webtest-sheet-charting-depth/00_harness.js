/**
 * Charting-depth offline harness (#2490): ONE vm sandbox, real shipped IIFEs.
 *
 * Host globals are stubbed (window/document/gbT/localStorage/sessionStorage/
 * setTimeout); Date.now and Math.random are patched INSIDE the vm realm so
 * local chart ids and any timed work are deterministic. No production file is
 * modified and no browser is required.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var REPO_ROOT = path.resolve(SUITE_DIR, "..", "..", "..");
var MODULES_REL = "sheet/js/modules-sheet-advanced/";
var FIXED_NOW = Date.UTC(2026, 0, 15, 12, 0, 0);
var FIXED_RANDOM = 0.42;

/* Dependency order: 34b/34c/35/36 grab namespaces at load time. */
var ORDER = [
  "31_charting_data.js",
  "32_charting_palette.js",
  "33_charting_svg.js",
  "34a_charting_types_cartesian.js",
  "34b_charting_types_hierarchy.js",
  "34c_charting_types_variants.js",
  "35_charting_overlays.js",
  "36_charting_render.js",
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

/* Minimal Fluent-style interpolation: shipped catalogs use {name} placeholders
 * (see botlib/locales/en/sheet-charting.ftl:50). This is a translation STUB,
 * not a reimplementation of chart logic. */
function translate(key, fallback, params) {
  var text = fallback === undefined ? key : fallback;
  if (params && typeof params === "object") {
    Object.keys(params).forEach(function (name) {
      text = String(text).split("{" + name + "}").join(String(params[name]));
    });
  }
  return text;
}

function createEngine() {
  var local = {};
  var scheduled = [];
  var handlers = {};
  var bodyHandlers = {};
  var lang = "en";
  var sheetId = "sheet-2490-fixed";
  var activeIndex = 0;
  var grid = { cells: {} };

  var sandbox = {
    console: { log: function () {}, warn: function () {}, error: function () {} },
    localStorage: {
      getItem: function (key) { return Object.prototype.hasOwnProperty.call(local, key) ? local[key] : null; },
      setItem: function (key, value) { local[key] = String(value); },
      removeItem: function (key) { delete local[key]; },
      clear: function () { local = {}; },
    },
    sessionStorage: {
      getItem: function (key) { return key === "sheet_active_index" ? String(activeIndex) : null; },
      setItem: function (key, value) { if (key === "sheet_active_index") activeIndex = parseInt(value, 10) || 0; },
    },
    /* Capture, do not run: applyAll() touches the DOM and is a browser concern. */
    setTimeout: function (fn, ms) { scheduled.push({ fn: fn, ms: ms }); return scheduled.length; },
  };
  sandbox.document = {
    readyState: "complete",
    documentElement: { getAttribute: function () { return lang; } },
    getElementById: function () { return null; },
    createElement: function (tag) {
      return { tagName: tag, style: {}, className: "", children: [],
        setAttribute: function () {}, appendChild: function () {}, addEventListener: function () {} };
    },
    addEventListener: function (type, fn) { handlers[type] = fn; },
    body: {
      addEventListener: function (type, fn) { bodyHandlers[type] = fn; },
      querySelectorAll: function () { return []; },
    },
  };
  sandbox.window = {
    document: sandbox.document,
    gbT: translate,
    localStorage: sandbox.localStorage,
    sessionStorage: sandbox.sessionStorage,
    SheetCore: {
      getGrid: function () { return grid; },
      wsIndex: function () { return activeIndex; },
      currentSheetId: function () { return sheetId; },
    },
    SheetI18n: {
      getLocale: function () { return lang; },
      load: function (locale) { lang = locale; },
      t: translate,
    },
    dispatchEvent: function () {},
  };
  sandbox.gbT = translate;
  sandbox.window.localStorage = sandbox.localStorage;

  var context = vm.createContext(sandbox);
  vm.runInContext(
    "Date.now = function () { return " + FIXED_NOW + "; };" +
    "Math.random = function () { return " + FIXED_RANDOM + "; };",
    context);
  ORDER.forEach(function (fileName) {
    vm.runInContext(source(fileName), context, { filename: fileName });
  });

  var w = sandbox.window;

  function setGrid(next) { grid = next; return engine; }
  function makeMap(pairs) {
    return vm.runInContext("new Map(" + JSON.stringify(pairs) + ")", context);
  }
  function withSheetId(value, fn) {
    var previous = sheetId;
    var previousInitial = w.__SHEET_INITIAL_ID;
    sheetId = value;
    w.__SHEET_INITIAL_ID = value;
    try { return fn(); } finally {
      sheetId = previous;
      if (previousInitial === undefined) delete w.__SHEET_INITIAL_ID;
      else w.__SHEET_INITIAL_ID = previousInitial;
    }
  }
  function withIndex(value, fn) {
    var previous = activeIndex;
    activeIndex = value;
    try { return fn(); } finally { activeIndex = previous; }
  }
  function withTranslator(fn, callback) {
    var previous = w.gbT;
    w.gbT = fn;
    try { return callback(); } finally { w.gbT = previous; }
  }
  function withFormat(fn, callback) {
    var previous = w.gbFormatNumber;
    if (fn) w.gbFormatNumber = fn; else delete w.gbFormatNumber;
    try { return callback(); } finally {
      if (previous === undefined) delete w.gbFormatNumber; else w.gbFormatNumber = previous;
    }
  }

  /* Same ctx shape production renderSvg builds (36_charting_render.js:77-80). */
  function ctxFor(def, width, height) {
    var canvasW = width || 480;
    var canvasH = height || 300;
    var model = w.SheetChartingData.model(def);
    var geo = w.SheetChartingSvg.geometry(canvasW, canvasH);
    return {
      def: def || {},
      model: model,
      colors: w.SheetChartingPalette.colors(model.series, def && def.options && def.options.palette),
      geo: geo,
      w: canvasW,
      h: canvasH,
      t: w.SheetCharting.t,
      scale: w.SheetChartingSvg.verticalScale(0, Math.max(1, model.max * 1.1), geo, 4),
    };
  }

  /* Same overlay map production renderSvg builds (36_charting_render.js:84-91). */
  function mapFor(ctx) {
    var geo = ctx.geo;
    var count = Math.max(1, ctx.model.labels.length);
    return {
      x: function (index) { return geo.x(index, count); },
      y: function (value) { return ctx.scale.y(value); },
      left: geo.pad.l,
      right: geo.pad.l + geo.plotW,
      top: geo.pad.t,
      lastX: geo.pad.l + geo.plotW,
    };
  }

  function seriesDef(series, labels, options, analysis) {
    var def = {
      chart_type: "line",
      labels: labels,
      datasets: series.map(function (item) {
        return { label: item.label || "", color: item.color || "", data: item.data };
      }),
      options: options || {},
    };
    if (analysis) def.analysis = analysis;
    return def;
  }

  var engine = {
    context: context,
    sandbox: sandbox,
    window: w,
    data: w.SheetChartingData,
    palette: w.SheetChartingPalette,
    svg: w.SheetChartingSvg,
    types: w.SheetChartingTypes,
    renderers: w.SheetChartingTypes.renderers,
    overlays: w.SheetChartingOverlays,
    charting: w.SheetCharting,
    scheduled: scheduled,
    handlers: handlers,
    bodyHandlers: bodyHandlers,
    local: local,
    setGrid: setGrid,
    makeMap: makeMap,
    withSheetId: withSheetId,
    withIndex: withIndex,
    withTranslator: withTranslator,
    withFormat: withFormat,
    ctxFor: ctxFor,
    mapFor: mapFor,
    seriesDef: seriesDef,
    translate: translate,
    fixedNow: FIXED_NOW,
    fixedRandom: FIXED_RANDOM,
  };
  return engine;
}

var engine = createEngine();
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

function equal(actual, expected, message) {
  assert(actual === expected,
    (message || "values differ") + " — expected " + expected + ", got " + actual);
}

function deep(actual, expected, message) {
  var a = JSON.stringify(actual);
  var b = JSON.stringify(expected);
  assert(a === b, (message || "structures differ") + " — expected " + b + ", got " + a);
}

function sourceHas(fileName, pattern, message) {
  assert(new RegExp(pattern).test(source(fileName)), message);
}

function sourceLacks(fileName, pattern, message) {
  assert(!new RegExp(pattern).test(source(fileName)), message);
}

var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function note(line) { gbLog.debug("    " + line); }

function finish() {
  var rule = "=".repeat(68);
  var counts = { RUNTIME: 0, STATIC: 0, "KNOWN BUG": 0 };
  RESULTS.forEach(function (result) {
    if (result.status === "PASS" && counts[result.kind] !== undefined) counts[result.kind]++;
  });
  gbLog.debug("\n" + rule);
  gbLog.debug("SHEET CHARTING DEPTH — OFFLINE TEST RESULTS (#2490)");
  gbLog.debug(rule);
  RESULTS.forEach(function (result) {
    gbLog.debug("[" + result.status + "] " + result.name + (result.error ? " — " + result.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("RUNTIME PASS: " + counts.RUNTIME + " | STATIC PASS: " + counts.STATIC +
    " | KNOWN BUG (documented): " + counts["KNOWN BUG"]);
  gbLog.debug("MODE: one vm sandbox; eight real shipped IIFEs; stubbed window/document/" +
    "gbT/localStorage/sessionStorage/setTimeout + fixed clock and Math.random inside the vm");
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
  FIXED_RANDOM: FIXED_RANDOM,
  test: test,
  staticTest: staticTest,
  knownBug: knownBug,
  assert: assert,
  equal: equal,
  deep: deep,
  readFile: readFile,
  source: source,
  repoFile: repoFile,
  sourceHas: sourceHas,
  sourceLacks: sourceLacks,
  note: note,
  finish: finish,
};
