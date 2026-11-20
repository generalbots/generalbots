/**
 * Offline harness for #2511 office-translation verification.
 *
 * Mirrors webtest-docs-lang/00_harness.js: ordered async-capable tests,
 * assertions, real-source vm loaders and deterministic browser stubs. The REAL
 * shipped files are read from botui/ui/suite and executed - no copies, mocks of
 * the code under test, or fabricated PASS results.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var FIXED_NOW = 1767225600000;
var PASS = 0;
var FAIL = 0;
var RESULTS = [];
var QUEUE = [];

function log(message) {
  console.log(message);
}

/** Queue a test; the returned promise (if any) is awaited by finish(). */
function test(name, fn) {
  QUEUE.push({ name: name, fn: fn });
}

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}

function assertEqual(actual, expected, msg) {
  if (actual !== expected) {
    throw new Error((msg || "values differ") + " (actual=" + JSON.stringify(actual) +
      ", expected=" + JSON.stringify(expected) + ")");
  }
}

function readFile(relPath) {
  return fs.readFileSync(path.join(SUITE_DIR, relPath), "utf8");
}

function fileExists(relPath) {
  return fs.existsSync(path.join(SUITE_DIR, relPath));
}

/** Deterministic clock and uid source so shipped helpers are reproducible. */
function fixedClock() {
  function FixedDate(value) {
    return new Date(value == null ? FIXED_NOW : value);
  }
  FixedDate.now = function () { return FIXED_NOW; };
  FixedDate.parse = Date.parse;
  FixedDate.UTC = Date.UTC;
  return FixedDate;
}

function fixedUidMath() {
  var stub = {};
  Object.getOwnPropertyNames(Math).forEach(function (name) {
    var value = Math[name];
    stub[name] = typeof value === "function" ? value.bind(Math) : value;
  });
  stub.random = function () { return 0.5; };
  return stub;
}

function memoryStorage() {
  var store = {};
  return {
    getItem: function (key) {
      return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null;
    },
    setItem: function (key, value) { store[key] = String(value); },
    removeItem: function (key) { delete store[key]; },
    _dump: function () { return JSON.parse(JSON.stringify(store)); }
  };
}

function loadScripts(relPaths, sandbox) {
  var context = vm.createContext(sandbox);
  relPaths.forEach(function (relPath) {
    vm.runInContext(readFile(relPath), context, { filename: relPath });
  });
  return context;
}

/** Minimal article used by both the translate module and the language modules. */
function makeArticle(innerHTML, docId) {
  return {
    innerHTML: innerHTML,
    textContent: String(innerHTML || "").replace(/<[^>]+>/g, ""),
    dataset: { docId: docId || "current" },
    events: [],
    dispatchEvent: function (event) {
      this.events.push({ type: event.type, bubbles: !!event.bubbles });
      return true;
    }
  };
}

/**
 * Execute the REAL shipped docs translation module in a vm sandbox.
 * Every browser dependency is an inert stub; network calls are captured with
 * their full JSON body so apply and undo-history seeding can be observed.
 */
function makeTranslateFixture(options) {
  options = options || {};
  var article = makeArticle(options.html || "<p>Hello <strong>world</strong></p>",
    options.docId || "contract-1");
  var modalHost = { innerHTML: "" };
  var applyButton = { disabled: false, textContent: "", onclick: null };
  var langSelect = { value: options.langValue || "Portuguese" };
  var calls = [];
  var statusCalls = [];
  var snapshots = [];
  var pageCountCalls = 0;

  var documentStub = {
    querySelector: function (selector) {
      if (selector === "article[data-doc-id]") return options.noArticle ? null : article;
      if (selector === "article[contenteditable]") return article;
      return null;
    },
    getElementById: function (id) {
      if (id === "docs-modal-container") return modalHost;
      if (id === "translate-apply") return applyButton;
      if (id === "translate-lang") return langSelect;
      return null;
    },
    createTreeWalker: function () { return { nextNode: function () { return null; } }; },
    createRange: function () { return {}; },
    createTextNode: function (value) { return { nodeValue: String(value) }; }
  };

  var windowStub = {
    gbT: function (key, fallback) { return fallback; },
    localStorage: memoryStorage(),
    DocsNav: {
      tr: function (key, fallback) { return fallback; },
      readStore: function () { return null; },
      writeStore: function () {},
      status: function () {}
    }
  };
  if (!options.noSnapshots) {
    windowStub.GBCollabVersions = {
      snapshot: function (payload) { snapshots.push(payload); }
    };
  }

  function responseFor(url, status) {
    if (url === "/api/docs/autosave") {
      if (options.autosaveFails) return Promise.reject(new Error("network down"));
      return Promise.resolve({ ok: true, status: 200, json: function () { return Promise.resolve({}); } });
    }
    if (url === "/api/docs/ai/translate") {
      var payload = typeof options.translateResponse === "function"
        ? options.translateResponse(calls.length) : options.translateResponse;
      if (payload == null) {
        payload = { content: "<p>Titulo <strong>mundo</strong></p>" };
      }
      if (payload.ok === false) {
        return Promise.resolve({ ok: false, status: payload.status || 503 });
      }
      return Promise.resolve({ ok: true, status: 200, json: function () { return Promise.resolve(payload); } });
    }
    return Promise.resolve({ ok: false, status: 404 });
  }

  var sandbox = {
    console: { error: function () {}, log: function () {}, warn: function () {} },
    window: windowStub,
    document: documentStub,
    Date: fixedClock(),
    Math: fixedUidMath(),
    NodeFilter: { SHOW_TEXT: 4 },
    Event: function (type, init) {
      this.type = type;
      this.bubbles = !!(init && init.bubbles);
    },
    fetch: function (url, init) {
      var body = null;
      try { body = JSON.parse(init.body); } catch (error) { body = null; }
      calls.push({ url: url, init: init, body: body });
      return responseFor(url, init);
    },
    getActiveArticle: function () { return options.noActiveArticle ? null : article; },
    setSaveStatus: function (message, isError) {
      statusCalls.push({ message: message, isError: !!isError });
    },
    updatePageCount: function () { pageCountCalls++; },
    gbLogError: function () {}
  };
  if (options.renderBlocks) sandbox.docsRenderBlocks = options.renderBlocks;

  var sources = ["docs/modules/17_translate.js"];
  if (options.withLangModules) {
    sources = [
      "docs/modules/25_lang_core.js",
      "docs/modules/25b_lang_data.js",
      "docs/modules/26_lang_spell.js",
      "docs/modules/27_lang_style.js",
      "docs/modules/29_lang_readability.js"
    ].concat(sources);
  }
  var context = loadScripts(sources, sandbox);

  return {
    ctx: context,
    window: windowStub,
    document: documentStub,
    article: article,
    modalHost: modalHost,
    applyButton: applyButton,
    langSelect: langSelect,
    calls: calls,
    statusCalls: statusCalls,
    snapshots: snapshots,
    pageCountCalls: function () { return pageCountCalls; }
  };
}

/** Load the shipped local-language modules (glossary, variables, variants). */
function makeLangFixture() {
  var windowStub = {
    gbT: function (key, fallback) { return fallback; },
    localStorage: memoryStorage(),
    DocsNav: {
      tr: function (key, fallback) { return fallback; },
      readStore: function () { return null; },
      writeStore: function () {},
      status: function () {}
    }
  };
  var article = makeArticle("<p>Contract</p>", "doc-lang-1");
  var documentStub = {
    querySelector: function (selector) {
      return selector === "article[contenteditable]" ? article : null;
    },
    getElementById: function () { return null; }
  };
  var sandbox = {
    console: { error: function () {}, log: function () {} },
    window: windowStub,
    document: documentStub,
    Date: fixedClock(),
    Math: fixedUidMath(),
    NodeFilter: { SHOW_TEXT: 4 }
  };
  var context = loadScripts([
    "docs/modules/25_lang_core.js",
    "docs/modules/25b_lang_data.js",
    "docs/modules/26_lang_spell.js",
    "docs/modules/27_lang_style.js",
    "docs/modules/29_lang_readability.js",
    "docs/modules/23_quickparts.js"
  ], sandbox);
  return { ctx: context, window: windowStub, document: documentStub, article: article, NS: windowStub.DocsLang };
}

function report() {
  log("\n" + "=".repeat(64));
  log("OFFICE TRANSLATION ACROSS THE TRIO - OFFLINE RESULTS (#2511)");
  log("=".repeat(64));
  RESULTS.forEach(function (result) {
    log("[" + result.status + "] " + result.name + (result.error ? " -- " + result.error : ""));
  });
  log("=".repeat(64));
  log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  log("=".repeat(64));
  process.exitCode = FAIL > 0 ? 1 : 0;
}

/** Run queued tests in registration order, then print the report. */
function finish() {
  (async function () {
    for (var i = 0; i < QUEUE.length; i++) {
      var job = QUEUE[i];
      try {
        await job.fn();
        PASS++;
        RESULTS.push({ name: job.name, status: "PASS" });
      } catch (error) {
        FAIL++;
        RESULTS.push({
          name: job.name,
          status: "FAIL",
          error: error && error.message ? error.message : String(error)
        });
      }
    }
    report();
  })();
}

module.exports = {
  fs,
  path,
  SUITE_DIR,
  FIXED_NOW,
  test,
  assert,
  assertEqual,
  readFile,
  fileExists,
  loadScripts,
  memoryStorage,
  fixedClock,
  fixedUidMath,
  makeArticle,
  makeTranslateFixture,
  makeLangFixture,
  log,
  finish
};
