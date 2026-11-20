/**
 * Mobile chat offline harness (#2516): vm sandbox, stubs and reporting.
 *
 * Creates a fresh, deterministic realm for each scenario. Timers, the clock,
 * animation frames and DOM events are host-controlled so shipped code can be
 * stepped deterministically. The DOM model itself lives in 05_dom.js.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var REPO_ROOT = path.resolve(SUITE_DIR, "..", "..", "..");
var FIXED_NOW = Date.UTC(2026, 0, 15, 12, 0, 0);
var FIXED_UID = "2516-fixed-session";
var GAP = 16;

var MODULES = [
  "chat/messages/10_scroll.js",
  "chat/messages/15_virtualize.js",
  "chat/messages/20_persist.js",
  "chat/messages/70_stream.js",
];

function readFile(rel) {
  return fs.readFileSync(path.join(SUITE_DIR, rel), "utf8");
}
function repoFile(rel) {
  return fs.readFileSync(path.join(REPO_ROOT, rel), "utf8");
}

/* ---------------------------------------------------------------- counters */
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
    RESULTS.push({ name: name, kind: kind, status: "FAIL", error: error && error.message });
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
    (message || "value") + ": expected " + expected + ", got " + actual);
}
function sourceHas(file, pattern, message) {
  assert(new RegExp(pattern).test(readFile(file)), message || file + " lacks " + pattern);
}
function sourceLacks(file, pattern, message) {
  assert(!new RegExp(pattern).test(readFile(file)), message || file + " still has " + pattern);
}

/* ------------------------------------------------------------------ engine */
function createEngine() {
  var store = {};
  var events = [];
  var timers = {};
  var timerSeq = 0;
  var raf = {};
  var rafSeq = 0;
  var now = FIXED_NOW;
  var windowEvents = {};
  var vvEvents = {};
  var resizeObserved = [];
  var turnCalls = [];

  function clock() { return now; }
  function setNow(value) { now = value; }
  function advance(ms) {
    now += ms;
    var due = Object.keys(timers).filter(function (id) {
      return timers[id] && timers[id].at <= now;
    }).sort(function (a, b) { return timers[a].at - timers[b].at || +a - +b; });
    due.forEach(function (id) {
      var entry = timers[id];
      if (!entry) return;
      if (entry.kind === "interval") {
        entry.at += entry.every;
        entry.fn();
      } else {
        delete timers[id];
        entry.fn();
      }
    });
  }
  function flushRaf() {
    var queue = raf;
    raf = {};
    Object.keys(queue).sort(function (a, b) { return +a - +b; }).forEach(function (id) {
      queue[id]();
    });
  }
  function activeTimers() {
    return Object.keys(timers).filter(function (id) { return !!timers[id]; }).length;
  }
  function activeRaf() { return Object.keys(raf).length; }

  var noop = function () {};
  var sandbox = {
    console: { log: noop, warn: noop, error: noop, debug: noop },
    __gbNow: now,
    __gbGap: GAP,
    localStorage: {
      get length() { return Object.keys(store).length; },
      key: function (i) { return Object.keys(store)[i] || null; },
      getItem: function (key) {
        return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null;
      },
      setItem: function (key, value) { store[key] = String(value); },
      removeItem: function (key) { delete store[key]; },
      clear: function () { store = {}; },
    },
    Event: function (type, options) {
      this.type = type;
      this.detail = options && options.detail;
      this.target = null;
      this.defaultPrevented = false;
      this.preventDefault = function () { this.defaultPrevented = true; };
    },
    CustomEvent: function (type, options) {
      this.type = type;
      this.detail = options && options.detail;
      this.target = null;
      this.defaultPrevented = false;
      this.preventDefault = function () { this.defaultPrevented = true; };
    },
    setTimeout: function (fn, ms) {
      var id = ++timerSeq;
      timers[id] = { fn: fn, at: now + (ms || 0), kind: "timeout" };
      return id;
    },
    clearTimeout: function (id) { delete timers[id]; },
    setInterval: function (fn, ms) {
      var id = ++timerSeq;
      timers[id] = { fn: fn, at: now + (ms || 0), kind: "interval", every: ms || 0 };
      return id;
    },
    clearInterval: function (id) { delete timers[id]; },
    requestAnimationFrame: function (fn) {
      var id = ++rafSeq;
      raf[id] = fn;
      return id;
    },
    cancelAnimationFrame: function (id) { delete raf[id]; },
    Date: Date,
  };
  Object.defineProperty(sandbox, "__gbNow", {
    get: function () { return now; },
    set: function (value) { now = value; },
    configurable: true,
  });

  /* Fixed-clock helpers used by stream/persist; identity render helpers keep
   * shipped streaming code on its plain-text branch. */
  sandbox.gbT = function (key, fallback) {
    return fallback === undefined ? key : fallback;
  };
  sandbox.gbLogError = function () {};
  sandbox.gbStoreGet = function (key, fallback) {
    var raw = sandbox.localStorage.getItem(key);
    if (raw === null) return fallback;
    try { return JSON.parse(raw); } catch (e) { return fallback; }
  };
  sandbox.gbFormatDate = function () { return "15 Jan 2026"; };
  sandbox.gbFormatTime = function () { return "12:00"; };
  sandbox.gbFormatDateTime = function () { return "15 Jan 2026, 12:00"; };
  sandbox.escapeHtml = function (value) { return String(value); };
  sandbox.stripMarkdownBlocks = function (value) { return value; };
  sandbox.convertNumberedLists = function (value) { return value; };
  sandbox.renderInlineMarkdown = function (value) { return value; };
  sandbox.renderMentionInMessage = function (value) { return value; };
  sandbox.renderDeepLinks = function (value) { return value; };
  sandbox.renderVmFileLinks = function (value) { return value; };
  sandbox.renderThinkingSection = function () { return ""; };
  sandbox.GBSanitize = { html: function (value) { return value; } };
  sandbox.TTSState = { autoSpeak: false };
  sandbox.ChatState = {
    currentSessionId: FIXED_UID,
    isUserAtBottom: true,
    isStreaming: false,
    streamingMessageId: "",
    currentStreamingContent: "",
    currentReasoning: "",
    streamingBuffer: "",
    suppressPersist: false,
    appendTarget: null,
    pendingResponseAt: 0,
    renderInterval: 200,
    lastRenderTime: 0,
    lastCompletedTurn: null,
    pendingToolExec: null,
    _gbAnnouncedTools: null,
    _gbStreamStartAnnounced: false,
  };

  sandbox.ResizeObserver = function (cb) {
    this.observe = function (el) { resizeObserved.push({ op: "observe", el: el }); };
    this.unobserve = function (el) { resizeObserved.push({ op: "unobserve", el: el }); };
  };
  sandbox.MutationObserver = function () {
    this.observe = function () {};
    this.disconnect = function () {};
  };

  sandbox.window = {
    document: null,
    localStorage: sandbox.localStorage,
    gbT: sandbox.gbT,
    gbLogError: sandbox.gbLogError,
    ChatState: sandbox.ChatState,
    ResizeObserver: sandbox.ResizeObserver,
    MutationObserver: sandbox.MutationObserver,
    getComputedStyle: function () {
      return { rowGap: GAP + "px", gap: GAP + "px" };
    },
    requestAnimationFrame: sandbox.requestAnimationFrame,
    cancelAnimationFrame: sandbox.cancelAnimationFrame,
    innerHeight: 844,
    innerWidth: 390,
    visualViewport: {
      height: 844,
      offsetTop: 0,
      addEventListener: function (type, fn) {
        (vvEvents[type] || (vvEvents[type] = [])).push(fn);
      },
      removeEventListener: function () {},
    },
    addEventListener: function (type, fn) {
      (windowEvents[type] || (windowEvents[type] = [])).push(fn);
    },
    removeEventListener: function (type, fn) {
      var list = windowEvents[type] || [];
      var i = list.indexOf(fn);
      if (i >= 0) list.splice(i, 1);
    },
    dispatchEvent: function (event) {
      events.push(event);
      (windowEvents[event.type] || []).slice().forEach(function (fn) { fn(event); });
      return true;
    },
    GBTurn: {
      start: function () { turnCalls.push("start"); },
      visible: function () { turnCalls.push("visible"); },
      thinking: function () { turnCalls.push("thinking"); },
      touch: function () { turnCalls.push("touch"); },
      complete: function () { turnCalls.push("complete"); },
      forceHide: function () { turnCalls.push("forceHide"); },
      stale: function () { return false; },
    },
  };

  var context = vm.createContext(sandbox);
  /* Fixed clock inside the vm realm so any shipped Date.now() is deterministic. */
  vm.runInContext("Date.now = function () { return __gbNow; };", context);

  var engine = {
    context: context,
    sandbox: sandbox,
    window: sandbox.window,
    ChatState: sandbox.ChatState,
    store: store,
    events: events,
    windowEvents: windowEvents,
    vvEvents: vvEvents,
    resizeObserved: resizeObserved,
    turnCalls: turnCalls,
    clock: clock,
    setNow: setNow,
    advance: advance,
    flushRaf: flushRaf,
    activeTimers: activeTimers,
    activeRaf: activeRaf,
    fireWindow: function (type, detail) {
      sandbox.window.dispatchEvent(new sandbox.Event(type, { detail: detail }));
    },
    fireVv: function (type) {
      (vvEvents[type] || []).slice().forEach(function (fn) {
        fn(new sandbox.Event(type));
      });
    },
    fixedNow: FIXED_NOW,
    fixedUid: FIXED_UID,
    gap: GAP,
  };
  return engine;
}

/* ------------------------------------------------------------------ report */
var gbLog = globalThis.gbLog || { debug: function (line) { process.stdout.write(line + "\n"); } };
function note(line) { gbLog.debug("    " + line); }
function finish() {
  var rule = "=".repeat(70);
  var counts = { RUNTIME: 0, STATIC: 0, "KNOWN BUG": 0 };
  RESULTS.forEach(function (result) {
    if (result.status === "PASS" && counts[result.kind] !== undefined) counts[result.kind]++;
  });
  gbLog.debug("\n" + rule);
  gbLog.debug("MOBILE CHAT — OFFLINE TEST RESULTS (#2516)");
  gbLog.debug(rule);
  RESULTS.forEach(function (result) {
    gbLog.debug("[" + result.status + "] " + result.name +
      (result.error ? " — " + result.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("RUNTIME PASS: " + counts.RUNTIME + " | STATIC PASS: " + counts.STATIC +
    " | KNOWN BUG (documented): " + counts["KNOWN BUG"]);
  gbLog.debug("MODE: one vm sandbox per scenario; four real shipped modules; " +
    "stubbed window/document/gbT/localStorage/fixed clock/uid");
  gbLog.debug("      [STATIC] = source assertions only, no runtime claim");
  gbLog.debug("      KNOWN BUG = observed behaviour, reported only, never patched here");
  gbLog.debug(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

module.exports = {
  SUITE_DIR: SUITE_DIR,
  REPO_ROOT: REPO_ROOT,
  MODULES: MODULES,
  FIXED_NOW: FIXED_NOW,
  FIXED_UID: FIXED_UID,
  GAP: GAP,
  readFile: readFile,
  repoFile: repoFile,
  createEngine: createEngine,
  test: test,
  staticTest: staticTest,
  knownBug: knownBug,
  assert: assert,
  equal: equal,
  sourceHas: sourceHas,
  sourceLacks: sourceLacks,
  note: note,
  finish: finish,
};
