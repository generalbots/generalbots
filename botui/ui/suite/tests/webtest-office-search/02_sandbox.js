/**
 * Deterministic sandbox factory for the two shipped #2505 browser IIFEs.
 *
 * Both modules are loaded UNMODIFIED into one vm context. Stubs cover
 * window/document/gbT/localStorage/fetch/Promise/timers, a fixed Date.now and
 * all sinks the modules call (deep links, chat input, modal prompts, tours).
 * Fixtures only feed production code and observe what it does.
 */
"use strict";

var h = require("./00_harness");
var dom = require("./01_dom");

var MODULES = ["js/command-palette.js", "js/gb-search.js"];
var FIXED_NOW = Date.UTC(2026, 0, 15, 12, 0, 0);

function FixedDate() {}
FixedDate.now = function () { return FIXED_NOW; };
FixedDate.prototype.toISOString = function () { return "2026-01-15T12:00:00.000Z"; };

/** Synchronous Promise-like: shipped .then/.catch chains settle inside an assertion. */
function SyncThenable(value, rejected) {
  this.value = value;
  this.rejected = !!rejected;
}
function settle(value) {
  while (value instanceof SyncThenable && !value.rejected) value = value.value;
  return value instanceof SyncThenable ? value : new SyncThenable(value);
}
SyncThenable.prototype.then = function (onOk, onErr) {
  if (this.rejected) {
    if (typeof onErr !== "function") return new SyncThenable(this.value, true);
    return settle(onErr(this.value));
  }
  try {
    return settle(onOk(this.value));
  } catch (error) {
    if (typeof onErr === "function") return settle(onErr(error));
    return new SyncThenable(error, true);
  }
};
SyncThenable.prototype.catch = function (onErr) {
  if (!this.rejected) return this;
  return settle(onErr(this.value));
};

function GbEvent(type) {
  this.type = type;
  this.preventDefaultCalls = 0;
  this.stopPropagationCalls = 0;
  this.stopImmediatePropagationCalls = 0;
}
GbEvent.prototype.preventDefault = function () { this.preventDefaultCalls++; };
GbEvent.prototype.stopPropagation = function () { this.stopPropagationCalls++; };
GbEvent.prototype.stopImmediatePropagation = function () { this.stopImmediatePropagationCalls++; };

function defaultCatalog() {
  return [
    {
      id: "docs", title: "Docs", category: "office", icon: "<path/>", color: "#22c55e",
      keywords: "writer document text", description: "Office document editor",
      commands: [
        { name: "docs-new", label: "New document", summary: "Creates a blank document",
          deep_link: "docs?new=1" },
        { name: "docs-open", label: "Open document", summary: "Opens a document by id",
          deep_link: "docs?id={doc_id}" },
        { name: "docs-optional", label: "Scoped document", summary: "Opens with optional ref",
          deep_link: "docs?id={doc_id}&ref={record?}" },
      ],
    },
    {
      id: "slides", title: "Slides", category: "office", icon: "<circle/>", color: "#f59e0b",
      keywords: "presentation deck", description: "Office presentation editor",
      commands: [{ name: "slides-new", label: "New deck", summary: "Creates a blank deck",
        deep_link: "slides?new=1" }],
    },
    {
      id: "sheet", title: "Sheet", category: "office", icon: "<rect/>", color: "#38bdf8",
      keywords: "spreadsheet cell", description: "Office spreadsheet editor",
      commands: [{ name: "sheet-api", label: "Recalculate", summary: "Recalculates formulas",
        method: "POST", path: "/api/sheet/recalculate" }],
    },
    {
      id: "drive", title: "Drive", category: "files", icon: "<path/>", color: "#a78bfa",
      keywords: "file storage workspace", description: "File storage workspace",
      commands: [],
    },
    {
      id: "vibe-extra", title: "Vibe tool", category: "tools", toolwindow: true,
      commands: [],
    },
  ];
}

function defaultSearchResults() {
  return [
    { app: "people", type: "person", id: "p-1", title: "Ada Lovelace",
      subtitle: "ada@example.test", url: "/suite/people/people.html" },
    { app: "drive", type: "file", id: "f-1", title: "Q3 variance.xlsx",
      subtitle: "/finance", url: "/suite/drive/drive.html" },
  ];
}

function registryFromCatalog(catalog) {
  return catalog.map(function (app) {
    return {
      id: app.id, title: app.title, category: app.category, icon: app.icon,
      color: app.color, keywords: app.keywords, description: app.description,
      hxGet: app.hxGet,
      toolwindow: app.toolwindow,
    };
  });
}

/** Build a fresh fixture and execute the real shipped IIFEs in it. */
function create(opts) {
  opts = opts || {};
  var doc = dom.makeDocument();

  var chatInput = dom.el("input", { id: "messageInput" });
  var searchInput = dom.el("input", { id: "searchInput" });
  doc._nodes.messageInput = chatInput;
  doc._nodes.searchInput = searchInput;
  doc.body.appendChild(chatInput);
  doc.body.appendChild(searchInput);

  var store = { "gb-access-token": "fixed-token" };
  var storage = {
    getItem: function (key) {
      return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null;
    },
    setItem: function (key, value) { store[key] = String(value); },
    removeItem: function (key) { delete store[key]; },
    clear: function () { Object.keys(store).forEach(function (key) { delete store[key]; }); },
  };

  var fetchCalls = [];
  var gbTCalls = [];
  var timers = [];
  var timerDelays = [];
  var timerId = 0;
  var deepLinks = [];
  var launches = [];
  var tourStarts = [];
  var prompts = [];
  var openedWindows = [];
  var windowListeners = {};
  var windowDispatched = [];

  var catalog = opts.catalog === undefined ? defaultCatalog() : opts.catalog;
  var searchResults = opts.searchResults === undefined ? defaultSearchResults() : opts.searchResults;
  var searchStatus = opts.searchStatus === undefined ? 200 : opts.searchStatus;
  var tourIds = opts.tourIds === undefined ? [] : opts.tourIds;

  function responseFor(url) {
    if (url.indexOf("/api/apps/catalog") === 0) {
      return { ok: true, status: 200, json: function () {
        return new SyncThenable({ apps: catalog });
      } };
    }
    if (url.indexOf("/api/ui/search") === 0) {
      var ok = searchStatus >= 200 && searchStatus < 300;
      return { ok: ok, status: searchStatus, json: function () {
        return new SyncThenable({ results: searchResults });
      } };
    }
    return { ok: false, status: 404, json: function () {
      return new SyncThenable({ results: [] });
    } };
  }

  var win = {
    document: doc,
    localStorage: storage,
    APPS_REGISTRY: opts.appsRegistry === undefined ? registryFromCatalog(catalog) : opts.appsRegistry,
    gbT: function (key, fallback, params) {
      gbTCalls.push({ key: key, fallback: fallback, params: params });
      return fallback === undefined ? key : fallback;
    },
    gbLogError: function () {},
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
    WindowManager: {
      launchFromMenu: function (appId, title, hxGet) {
        launches.push({ appId: appId, title: title, hxGet: hxGet });
      },
      openDeepLink: function (app, params, options) {
        deepLinks.push({ api: "WindowManager", app: app, params: params, options: options });
      },
    },
    openDeepLink: function (app, params) {
      deepLinks.push({ api: "window", app: app, params: params });
    },
    GBTour: {
      definitionIds: function () { return tourIds.slice(); },
      start: function (appId, options) { tourStarts.push({ appId: appId, options: options }); },
    },
    open: function (url, target, features) {
      openedWindows.push({ url: url, target: target, features: features });
      return null;
    },
  };

  var sandbox = {
    window: win,
    document: doc,
    localStorage: storage,
    gbT: win.gbT,
    fetch: function (url, options) {
      fetchCalls.push({ url: url, options: options || {} });
      return new SyncThenable(responseFor(url));
    },
    setTimeout: function (fn, delay) {
      timers.push({ fn: fn, id: ++timerId });
      timerDelays.push(delay || 0);
      return timerId;
    },
    clearTimeout: function (id) {
      for (var i = timers.length - 1; i >= 0; i--) {
        if (timers[i].id === id) timers.splice(i, 1);
      }
    },
    Date: FixedDate,
    Event: GbEvent,
    Promise: Promise,
    console: { log: function () {}, warn: function () {}, error: function () {} },
    GBModal: {
      prompt: function (text, callback) {
        prompts.push(text);
        callback(opts.promptValue === undefined ? "doc-42" : opts.promptValue);
      },
    },
  };

  var context = h.loadModules(MODULES, sandbox);

  var fx = {
    ctx: context,
    sandbox: sandbox,
    doc: doc,
    win: win,
    chatInput: chatInput,
    searchInput: searchInput,
    fetchCalls: fetchCalls,
    gbTCalls: gbTCalls,
    timers: timers,
    timerDelays: timerDelays,
    deepLinks: deepLinks,
    launches: launches,
    tourStarts: tourStarts,
    prompts: prompts,
    openedWindows: openedWindows,
    windowDispatched: windowDispatched,
  };

  fx.event = function (props) {
    var event = new GbEvent(props && props.type);
    Object.keys(props || {}).forEach(function (key) { event[key] = props[key]; });
    return event;
  };
  fx.flushTimers = function () {
    var count = 0;
    while (timers.length && count < 20) {
      var timer = timers.shift();
      count++;
      timer.fn();
    }
    return count;
  };
  fx.ctrlK = function () {
    var event = fx.event({
      type: "keydown", key: "k", ctrlKey: true, metaKey: false, shiftKey: false,
    });
    doc.dispatchEvent(event);
    return event;
  };
  fx.overlay = function () { return win.GBCommandPalette && win.GBCommandPalette.overlay; };
  fx.items = function () {
    return fx.overlay() ? fx.overlay().querySelectorAll(".gb-palette-item") : [];
  };
  fx.typeQuery = function (value) {
    var input = fx.overlay().querySelector("#gb-palette-input");
    input.value = value;
    input.dispatchEvent(fx.event({ type: "input", target: input }));
    return input;
  };
  fx.searchQuery = function (value) {
    searchInput.value = value;
    searchInput.dispatchEvent(fx.event({ type: "input", target: searchInput }));
  };
  fx.mountSearch = function () { win.GBUniversalSearch.mount(searchInput); };
  fx.searchPanel = function () { return doc.getElementById("gb-search-panel"); };
  return fx;
}

module.exports = { MODULES, FIXED_NOW, SyncThenable, GbEvent, create };
