/**
 * Slides animation/transition timeline — offline test harness (#2431).
 * Run with: node botui/ui/suite/tests/webtest-slides-anim.js
 *
 * Shared counters, assertions, a vm loader for the six REAL shipped modules and
 * a minimal DOM/window stub. The modules are plain IIFEs; loading them into one
 * context in slides.html order lets them resolve each other's namespaces the
 * same way the browser does. No production module is edited or re-implemented.
 * Mirrors webtest-slides-diagram/00_harness.js.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");

// Exact load order from slides.html (14 → 15 → 16b → 16c → 16 → 17).
var MODULE_RELS = [
  "slides/modules/14_anim_state.js",
  "slides/modules/15_anim_engine.js",
  "slides/modules/16b_anim_motion.js",
  "slides/modules/16c_anim_editor.js",
  "slides/modules/16_anim_timeline.js",
  "slides/modules/17_anim_transitions.js"
];

var FIXED_NOW = 1700000000000;

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

/* Small DOM node double. It implements only the surface the shipped modules
 * actually touch: attributes, dataset, class/text, children, listeners and a
 * style object that behaves like a CSSStyleDeclaration when `style` is set. */
function makeNode(tag, ns) {
  var node = {
    tagName: tag,
    namespaceURI: ns || null,
    className: "",
    id: "",
    textContent: "",
    hidden: false,
    disabled: false,
    type: "",
    value: "",
    checked: false,
    min: "",
    max: "",
    dataset: {},
    style: {},
    _attrs: {},
    _children: [],
    _listeners: {},
    firstChild: null,
    lastChild: null,
    appendChild: function (child) {
      this._children.push(child);
      child.parentNode = this;
      if (!this.firstChild) this.firstChild = child;
      this.lastChild = child;
      return child;
    },
    setAttribute: function (key, value) {
      var text = String(value);
      this._attrs[key] = text;
      if (key === "id") this.id = text;
      if (key === "style") {
        this.style = {};
        text.split(";").forEach(function (pair) {
          var at = pair.indexOf(":");
          if (at > 0) this.style[pair.slice(0, at).trim()] = pair.slice(at + 1).trim();
        }, this);
      }
    },
    getAttribute: function (key) {
      return Object.prototype.hasOwnProperty.call(this._attrs, key) ? this._attrs[key] : null;
    },
    addEventListener: function (type, fn) {
      if (!this._listeners[type]) this._listeners[type] = [];
      this._listeners[type].push(fn);
    },
    removeEventListener: function (type, fn) {
      var list = this._listeners[type] || [];
      var at = list.indexOf(fn);
      if (at >= 0) list.splice(at, 1);
    },
    querySelectorAll: function () { return []; },
    querySelector: function () { return null; },
    getBoundingClientRect: function () { return { left: 0, top: 0, width: 100, height: 100 }; },
    focus: function () {}
  };
  return node;
}

function findNode(root, predicate) {
  if (predicate(root)) return root;
  var kids = root._children || [];
  for (var i = 0; i < kids.length; i++) {
    var found = findNode(kids[i], predicate);
    if (found) return found;
  }
  return null;
}

/* One fresh sandbox per call: state is captured at module load time, so tests
 * that mutate the deck ask for a new env instead of trying to reset it. */
function loadModules(opts) {
  var settings = opts || {};
  var storage = {};
  var body = makeNode("body");
  // Monotonic, per-sandbox value: freshEffect() ids stay reproducible across
  // fresh sandboxes but remain unique within one authored timeline.
  var randomTick = 0;
  var windowStub = {
    gbT: settings.gbT || function (key, fallback) { return fallback; },
    localStorage: {
      getItem: function (key) {
        return Object.prototype.hasOwnProperty.call(storage, key) ? storage[key] : null;
      },
      setItem: function (key, value) { storage[key] = String(value); }
    },
    getSlidesPresentationId: function () { return "harness-deck"; },
    matchMedia: function () { return { matches: !!settings.matchMediaMatches }; },
    addEventListener: function () {},
    requestAnimationFrame: null,
    cancelAnimationFrame: function () {}
  };
  var sandbox = {
    console: { warn: function () {}, log: function () {} },
    Date: { now: function () { return FIXED_NOW; } },
    Math: {
      max: Math.max, min: Math.min, round: Math.round, floor: Math.floor,
      sin: Math.sin, pow: Math.pow, abs: Math.abs,
      random: function () { randomTick += 1; return 0.1 + (randomTick % 80) / 100; }
    },
    window: windowStub,
    document: {
      body: body,
      activeElement: null,
      getElementById: function () { return null; },
      createElement: function (tag) { return makeNode(tag); },
      createElementNS: function (ns, tag) { return makeNode(tag, ns); },
      addEventListener: function () {}
    }
  };
  var context = vm.createContext(sandbox);
  MODULE_RELS.forEach(function (rel) {
    vm.runInContext(readFile(rel), context, { filename: rel });
  });
  return {
    A: windowStub.SlidesAnimation,
    E: windowStub.SlidesAnimationEngine,
    M: windowStub.SlidesAnimationMotion,
    Editor: windowStub.SlidesAnimationEditor,
    Timeline: windowStub.SlidesAnimationTimeline,
    Transitions: windowStub.SlidesAnimationTransitions,
    window: windowStub,
    document: sandbox.document,
    storage: storage,
    node: makeNode,
    findNode: findNode,
    source: function (rel) { return readFile(rel); }
  };
}

var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function report() {
  var rule = "=".repeat(64);
  gbLog.debug("\n" + rule);
  gbLog.debug("SLIDES ANIMATION/TRANSITION TIMELINE — OFFLINE TEST RESULTS");
  gbLog.debug(rule);
  RESULTS.forEach(function (r) {
    gbLog.debug("[" + r.status + "] " + r.name + (r.error ? " — " + r.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("MODE: runtime (vm sandbox, real shipped modules, stubbed window/document/gbT)");
  gbLog.debug("      + STATIC source assertions for DOM-only render helpers");
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
