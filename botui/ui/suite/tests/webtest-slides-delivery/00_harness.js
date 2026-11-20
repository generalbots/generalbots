/**
 * Slides delivery mode — offline test harness (#2432).
 * Run with: node botui/ui/suite/tests/webtest-slides-delivery.js
 *
 * Shared harness: counters, assertions, VM module loading and report. Mirrors
 * webtest-slides-diagram/00_harness.js. The eight real shipped delivery IIFEs
 * are loaded into ONE sandbox so they share `window.GBSlidesDelivery` exactly
 * as they do in slides.html. Only genuine exported helpers are exercised;
 * DOM-bound navigation/status primitives are replaced with controllable stubs
 * so the shipped pure logic can be driven offline. DOM-only render paths are
 * covered by labelled STATIC source assertions in 10_delivery.js.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");

var MODULE_RELS = [
  "slides/modules/51_delivery_state.js",
  "slides/modules/52_delivery_console.js",
  "slides/modules/53_delivery_shows.js",
  "slides/modules/54_delivery_sections.js",
  "slides/modules/55_delivery_captions.js",
  "slides/modules/56_delivery_rehearsal.js",
  "slides/modules/57_delivery_handouts.js",
  "slides/modules/58_delivery_recording.js"
];

var PASS = 0;
var FAIL = 0;
var RESULTS = [];

function test(name, fn) {
  try {
    fn();
    PASS++;
    RESULTS.push({ name: name, status: "PASS" });
  } catch (e) {
    FAIL++;
    RESULTS.push({ name: name, status: "FAIL", error: e.message });
  }
}

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}

function readFile(relPath) {
  return fs.readFileSync(path.join(SUITE_DIR, relPath), "utf8");
}

function loadModules() {
  var clock = 0;
  var uid = 0;
  var store = {};
  var tickerId = 0;
  var byId = {};
  var state = {
    slideCount: 12,
    current: 0,
    jumps: [],
    statuses: [],
    renderCalls: 0
  };

  function element(tag) {
    var el = {
      tagName: String(tag || "").toUpperCase(),
      children: [],
      style: {},
      dataset: {},
      attributes: {},
      textContent: "",
      value: "",
      className: "",
      id: "",
      parentNode: null,
      setAttribute: function (name, value) {
        el.attributes[name] = String(value);
        if (name === "id") el.id = String(value);
      },
      getAttribute: function (name) {
        return Object.prototype.hasOwnProperty.call(el.attributes, name)
          ? el.attributes[name] : null;
      },
      appendChild: function (child) {
        el.children.push(child);
        child.parentNode = el;
        return child;
      },
      removeChild: function (child) {
        var at = el.children.indexOf(child);
        if (at >= 0) el.children.splice(at, 1);
        child.parentNode = null;
        return child;
      },
      addEventListener: function () {},
      removeEventListener: function () {},
      querySelector: function () { return null; },
      querySelectorAll: function () { return []; },
      focus: function () {}
    };
    return el;
  }

  var body = element("body");
  body.appendChild = function (child) {
    body.children.push(child);
    child.parentNode = body;
    if (child.id) byId[child.id] = child;
    return child;
  };
  body.removeChild = function (child) {
    var at = body.children.indexOf(child);
    if (at >= 0) body.children.splice(at, 1);
    child.parentNode = null;
    if (child.id) delete byId[child.id];
    return child;
  };

  var documentStub = {
    body: body,
    head: element("head"),
    activeElement: null,
    createElement: function (tag) { return element(tag); },
    getElementById: function (id) { return byId[id] || null; },
    querySelectorAll: function () { return []; },
    addEventListener: function () {},
    removeEventListener: function () {}
  };

  var windowStub = {
    gbT: function (key, fallback) { return fallback; },
    getSlidesPresentationId: function () { return "demo-deck"; },
    localStorage: {
      getItem: function (key) {
        return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null;
      },
      setItem: function (key, value) { store[key] = String(value); },
      removeItem: function (key) { delete store[key]; }
    },
    setInterval: function () { tickerId += 1; return tickerId; },
    clearInterval: function () {}
  };

  var sandbox = {
    console: { warn: function () {}, log: function () {} },
    Date: { now: function () { return clock; } },
    window: windowStub,
    document: documentStub
  };
  var context = vm.createContext(sandbox);
  MODULE_RELS.forEach(function (rel) {
    vm.runInContext(readFile(rel), context, { filename: rel });
  });

  var D = windowStub.GBSlidesDelivery;
  // Deterministic ids and observable, DOM-free navigation/status primitives.
  // The real logic of every tested helper remains the shipped implementation.
  D.uid = function (prefix) { uid += 1; return (prefix || "delivery") + "_" + uid; };
  D.slideCount = function () { return state.slideCount; };
  D.currentSlide = function () { return state.current; };
  D.jump = function (index) {
    state.jumps.push(index);
    state.current = index;
    return true;
  };
  D.status = function (key, fallback, params) {
    state.statuses.push({ key: key, fallback: fallback, params: params });
    return fallback;
  };
  D.console.render = function () { state.renderCalls += 1; };

  function resetDom() {
    byId = {};
    body.children = [];
  }

  function reset() {
    clock = 0;
    state.slideCount = 12;
    state.current = 0;
    resetDom();
    if (D.recording.isActive()) D.recording.stop();
    D.console.pause();
    D.console.reset();
    D.rehearsal.pause();
    D.rehearsal.reset();
    store = {};
    uid = 0;
    state.jumps = [];
    state.statuses = [];
    state.renderCalls = 0;
  }

  reset();

  return {
    D: D,
    state: state,
    reset: reset,
    setClock: function (ms) { clock = ms; },
    advanceClock: function (ms) { clock += ms; },
    source: function (rel) { return readFile(rel); },
    storage: {
      raw: function () {
        return Object.prototype.hasOwnProperty.call(store, D.storeKey())
          ? store[D.storeKey()] : null;
      },
      setRaw: function (text) { store[D.storeKey()] = text; }
    }
  };
}

var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function report() {
  var rule = "=".repeat(64);
  gbLog.debug("\n" + rule);
  gbLog.debug("SLIDES DELIVERY — OFFLINE TEST RESULTS");
  gbLog.debug(rule);
  RESULTS.forEach(function (r) {
    gbLog.debug("[" + r.status + "] " + r.name + (r.error ? " — " + r.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("MODE: runtime (vm sandbox, eight real shipped modules, stubbed " +
    "window/document/gbT) + labelled STATIC source assertions");
  gbLog.debug(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

function finish() {
  report();
}

module.exports = {
  test: test,
  assert: assert,
  readFile: readFile,
  loadModules: loadModules,
  finish: finish,
  SUITE_DIR: SUITE_DIR,
  MODULE_RELS: MODULE_RELS
};
