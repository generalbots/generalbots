/**
 * Deterministic sandbox factory for the #2515 shipped mobile IIFEs.
 *
 * Both real IIFEs are loaded UNMODIFIED into one vm context; each self-gates on
 * its own app root. Stubs cover window/document/gbT/localStorage/matchMedia/
 * visualViewport/PointerEvent/getComputedStyle plus a queueable timer clock,
 * a fixed Date.now and a deterministic uid. Fixtures only feed the shipped
 * code and observe what it does.
 */
"use strict";

var h = require("./00_harness");
var dom = require("./01_dom");

var MODULES = ["docs/modules/19_mobile.js", "slides/modules/99_mobile.js"];
var FIXED_NOW = Date.UTC(2026, 0, 15, 12, 0, 0);

function FixedDate() {}
FixedDate.now = function () { return FIXED_NOW; };
FixedDate.prototype.toISOString = function () { return "2026-01-15T12:00:00.000Z"; };

function docsApp(doc) {
  var app = dom.el("div", { id: "docs-app" });
  doc.body.appendChild(app);
  var main = dom.el("div", { class: "docs-main" });
  app.appendChild(main);
  var toggle = dom.el("button", { id: "docsSidebarToggle" });
  app.appendChild(toggle);
  var close = dom.el("button", { id: "docsSidebarClose" });
  main.appendChild(close);
  var backdrop = dom.el("div", { id: "docsSidebarBackdrop" });
  main.appendChild(backdrop);
  var scroller = dom.el("div", { id: "docs-content" });
  main.appendChild(scroller);
  var article = dom.el("article", { class: "docs-doc-view", contenteditable: "true" });
  scroller.appendChild(article);
  [app, toggle, close, backdrop, scroller].forEach(function (node) {
    doc._nodes[node.id] = node;
  });
  return { app: app, main: main, toggle: toggle, close: close, backdrop: backdrop,
    scroller: scroller, article: article };
}

function slidesApp(doc, opts) {
  var app = dom.el("div", { id: "slides-app" });
  doc.body.appendChild(app);
  var toolbarRow = dom.el("div", { class: "slides-toolbar-row-main" });
  app.appendChild(toolbarRow);
  var toolbarLeft = dom.el("div", { class: "toolbar-left" });
  toolbarRow.appendChild(toolbarLeft);
  var main = dom.el("div", { class: "slides-main" });
  app.appendChild(main);
  var sidebar = dom.el("div", { class: "slides-sidebar" });
  main.appendChild(sidebar);
  var tabs = dom.el("div", { class: "sidebar-tabs" });
  sidebar.appendChild(tabs);
  var refs = { app: app, toolbarRow: toolbarRow, toolbarLeft: toolbarLeft, main: main,
    sidebar: sidebar, tabs: tabs, scroll: null, canvas: null };
  if (opts.scroll !== false) {
    var scroll = dom.el("div", { class: "sl-canvas-scroll" });
    scroll.clientWidth = opts.scrollWidth === undefined ? 504 : opts.scrollWidth;
    main.appendChild(scroll);
    refs.scroll = scroll;
    if (opts.canvas !== false) {
      var canvas = dom.el("div", { class: "sl-canvas" });
      scroll.appendChild(canvas);
      refs.canvas = canvas;
    }
  }
  return refs;
}

/** Build a fresh phone/desktop fixture and execute the shipped IIFEs in it. */
function create(opts) {
  opts = opts || {};
  var apps = opts.apps || ["docs", "slides"];
  var doc = dom.makeDocument();
  doc.readyState = opts.readyState || "loading";
  var refs = {};
  if (apps.indexOf("docs") >= 0) refs.docs = docsApp(doc);
  if (apps.indexOf("slides") >= 0) refs.slides = slidesApp(doc, opts);
  /* Element fixtures are built with dom.el (no owner); link them now so the
   * shipped focus() calls update document.activeElement as in a real DOM. */
  dom.walk(doc.body, function (node) { node._ownerDoc = doc; });

  var store = {};
  var timers = [];
  var timerDelays = [];
  var timerId = 0;
  var gbTCalls = [];
  var matchMediaCalls = [];
  var pointerEvents = [];
  var windowDispatched = [];
  var windowListeners = {};
  var vvListeners = {};
  var uuidCounter = 0;
  var fx = {
    doc: doc,
    refs: refs,
    store: store,
    timers: timers,
    timerDelays: timerDelays,
    gbTCalls: gbTCalls,
    matchMediaCalls: matchMediaCalls,
    pointerEvents: pointerEvents,
    windowDispatched: windowDispatched,
    windowListeners: windowListeners,
    vvListeners: vvListeners,
    keyboardPx: opts.keyboardPx === undefined ? 320 : opts.keyboardPx,
    selectionText: "",
    caretRect: null,
    selection: null,
  };

  var visualViewport = opts.visualViewport === false ? null : {
    addEventListener: function (type, fn) {
      (vvListeners[type] || (vvListeners[type] = [])).push(fn);
    },
    dispatch: function (type, event) {
      (vvListeners[type] || []).slice().forEach(function (fn) { fn(event); });
    },
  };

  function PointerEvent(type, options) {
    options = options || {};
    this.type = type;
    this.bubbles = !!options.bubbles;
    this.pointerType = options.pointerType || "";
    this.pointerId = options.pointerId || 0;
    this.clientX = options.clientX || 0;
    this.clientY = options.clientY || 0;
    this.preventDefaultCalls = 0;
    this.stopPropagationCalls = 0;
    this.preventDefault = function () { this.preventDefaultCalls++; };
    this.stopPropagation = function () { this.stopPropagationCalls++; };
    pointerEvents.push(this);
  }

  function MouseEvent(type, options) {
    options = options || {};
    this.type = type;
    this.bubbles = !!options.bubbles;
    this.clientX = options.clientX || 0;
    this.clientY = options.clientY || 0;
  }

  var win = {
    document: doc,
    localStorage: {
      getItem: function (key) { return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null; },
      setItem: function (key, value) { store[key] = String(value); },
      removeItem: function (key) { delete store[key]; },
      clear: function () { Object.keys(store).forEach(function (key) { delete store[key]; }); },
    },
    crypto: {
      randomUUID: function () {
        uuidCounter++;
        return String(uuidCounter).padStart(8, "0") + "-0000-4000-8000-000000000000";
      },
    },
    matchMedia: function (query) {
      matchMediaCalls.push(query);
      if (opts.throwMatchMedia) throw new Error("matchMedia unavailable");
      var coarse = query.indexOf("pointer: coarse") >= 0;
      var narrow = query.indexOf("max-width: 760px") >= 0;
      return { matches: coarse ? opts.pointer !== "fine" : (narrow ? opts.width !== undefined ? opts.width <= 760 : true : false) };
    },
    getSelection: function () { return fx.selection; },
    visualViewport: visualViewport,
    gbT: function (key, fallback) {
      gbTCalls.push({ key: key, fallback: fallback });
      return fallback === undefined ? key : fallback;
    },
    SlideCanvas: opts.slideCanvas === false ? undefined : { scale: 1 },
    addEventListener: function (type, fn) {
      (windowListeners[type] || (windowListeners[type] = [])).push(fn);
    },
    removeEventListener: function (type, fn) {
      var list = windowListeners[type] || [];
      var index = list.indexOf(fn);
      if (index >= 0) list.splice(index, 1);
    },
    dispatchEvent: function (event) {
      windowDispatched.push(event);
      (windowListeners[event.type] || []).slice().forEach(function (fn) { fn(event); });
      return true;
    },
  };

  fx.setSelection = function (text, rect) {
    fx.selectionText = text || "";
    fx.caretRect = rect || null;
    fx.selection = {
      toString: function () { return fx.selectionText; },
      rangeCount: rect ? 1 : 0,
      getRangeAt: function () {
        return { getClientRects: function () { return rect ? [rect] : []; } };
      },
    };
    win.getSelection = function () { return fx.selection; };
    return fx;
  };
  fx.setSelection("", null);

  fx.event = function (options) {
    var event = options || {};
    event.preventDefaultCalls = 0;
    event.stopPropagationCalls = 0;
    event.preventDefault = function () { event.preventDefaultCalls++; };
    event.stopPropagation = function () { event.stopPropagationCalls++; };
    return event;
  };
  fx.flushOne = function () {
    var timer = timers.shift();
    if (!timer) return null;
    timer.fn();
    return timer;
  };
  fx.flushTimers = function (limit) {
    var count = 0;
    var cap = limit === undefined ? 50 : limit;
    while (timers.length && count < cap) {
      var timer = timers.shift();
      count++;
      timer.fn();
    }
    return count;
  };

  var sandbox = {
    window: win,
    document: doc,
    gbT: win.gbT,
    Date: FixedDate,
    console: { log: function () {}, warn: function () {}, error: function () {} },
    getComputedStyle: function () {
      return {
        getPropertyValue: function (name) {
          return name === "--gb-vv-bottom" ? fx.keyboardPx + "px" : "";
        },
      };
    },
    setTimeout: function (fn, delay) {
      timers.push({ fn: fn, delay: delay || 0, id: ++timerId });
      timerDelays.push(delay || 0);
      return timerId;
    },
    clearTimeout: function (id) {
      for (var i = timers.length - 1; i >= 0; i--) {
        if (timers[i].id === id) timers.splice(i, 1);
      }
    },
    PointerEvent: PointerEvent,
    MouseEvent: MouseEvent,
  };

  fx.ctx = h.loadModules(MODULES, sandbox);
  fx.sandbox = sandbox;
  fx.win = win;
  fx.keyboard = function (px) { fx.keyboardPx = px; return fx; };
  fx.hasWindowListener = function (type) { return !!(windowListeners[type] || []).length; };
  fx.hasDocListener = function (type) { return !!(doc._listeners[type] || []).length; };
  return fx;
}

module.exports = { MODULES, FIXED_NOW, create, docsApp, slidesApp, FixedDate };
