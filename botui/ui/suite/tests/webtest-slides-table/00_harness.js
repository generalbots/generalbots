/**
 * Slides table & media authoring — offline test harness (#2487).
 * Run with: node botui/ui/suite/tests/webtest-slides-table.js
 *
 * Self-contained plain-Node harness mirroring webtest-sheet-a11y/00_harness.js.
 * The real shipped modules are evaluated in a vm sandbox with stubbed
 * window/document/gbT and a fake host; no production file is modified.
 *
 * Two modules hide their helpers inside an IIFE. To reach those genuine
 * function bodies (not copies) the harness appends ONE export statement to an
 * in-memory copy of the source. 10_table.js proves that removing that single
 * statement reproduces the shipped source byte-for-byte, so no production
 * logic is fabricated or rewritten.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var MODEL_REL = "slides/modules/41_stm_table_model.js";
var EDITOR_REL = "slides/modules/42_stm_table_editor.js";
var BULK_REL = "slides/modules/24_bulk_ops.js";

var EDITOR_MARKER = 'if (document.readyState === "loading")';
var EDITOR_EXPORT =
  "window.__stmEditorInternals = { finishEdit: finishEdit, cellCoords: cellCoords, " +
  "currentCellModel: currentCellModel, selectCell: selectCell, applyMutation: applyMutation };";
var BULK_MARKER = "window.GBSlidesBulk = { open: open, restore: restore };";
var BULK_EXPORT =
  "window.__stmBulkInternals = { selected: selected, thumbFor: thumbFor, label: label, " +
  "snapshot: snapshot, restoreOrder: restoreOrder, move: move, duplicate: duplicate, " +
  "remove: remove, hide: hide, renderRows: renderRows };";

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

function silentConsole() {
  return { log: function () {}, warn: function () {}, error: function () {} };
}

/* Append a single export line inside the IIFE; never touch a function body. */
function instrument(source, marker, exportStatement) {
  if (source.split(marker).length !== 2) throw new Error("instrumentation marker is not unique");
  if (source.indexOf(exportStatement) !== -1) throw new Error("export already present in shipped source");
  return source.replace(marker, exportStatement + "\n  " + marker);
}

function stripInstrumentation(instrumented, marker, exportStatement) {
  return instrumented.replace(exportStatement + "\n  " + marker, marker);
}

/* Minimal DOM node with the operations the bulk helpers actually call. */
function fakeNode(attrs) {
  var node = {
    attrs: attrs || {},
    parentNode: null,
    nextSibling: null,
    children: [],
    tag: "",
    className: "",
    _text: "",
    getAttribute: function (key) {
      return Object.prototype.hasOwnProperty.call(node.attrs, key) ? node.attrs[key] : null;
    },
    setAttribute: function (key, value) { node.attrs[key] = String(value); },
    removeAttribute: function (key) { delete node.attrs[key]; },
    appendChild: function (child) {
      if (child.parentNode) child.parentNode.removeChild(child);
      node.children.push(child);
      child.parentNode = node;
      resync();
      return child;
    },
    insertBefore: function (child, ref) {
      if (child.parentNode) child.parentNode.removeChild(child);
      var at = ref ? node.children.indexOf(ref) : -1;
      if (at < 0) at = node.children.length;
      node.children.splice(at, 0, child);
      child.parentNode = node;
      resync();
      return child;
    },
    removeChild: function (child) {
      var at = node.children.indexOf(child);
      if (at >= 0) node.children.splice(at, 1);
      if (child.parentNode === node) child.parentNode = null;
      resync();
      return child;
    },
    cloneNode: function () { return fakeNode(Object.assign({}, node.attrs)); },
    addEventListener: function () {}
  };
  Object.defineProperty(node, "textContent", {
    get: function () { return node._text; },
    set: function (value) {
      node._text = String(value);
      if (value === "") node.children.slice().forEach(function (child) { node.removeChild(child); });
    }
  });
  function resync() {
    node.children.forEach(function (child, index) {
      child.nextSibling = node.children[index + 1] || null;
    });
  }
  return node;
}

function fakeDocument() {
  return {
    readyState: "loading",
    addEventListener: function () {},
    createElement: function (tag) { var node = fakeNode(); node.tag = tag; return node; },
    body: fakeNode()
  };
}

/* The 41_stm_table_model.js module only binds window.SlidesTableMedia on load;
 * DOM is referenced lazily by tableRender() which these tests never call. */
function loadTableModel() {
  var source = readFile(MODEL_REL);
  var sandbox = { window: {}, document: {}, console: silentConsole() };
  vm.runInContext(source, vm.createContext(sandbox), { filename: MODEL_REL });
  return { stm: sandbox.window.SlidesTableMedia, source: source };
}

/* The 42 editor binds document listeners at load unless readyState is
 * "loading"; the stub keeps bind() dormant while exposing the real functions. */
function loadEditor(modelStm) {
  var source = readFile(EDITOR_REL);
  var instrumented = instrument(source, EDITOR_MARKER, EDITOR_EXPORT);
  var stm = modelStm || {};
  var sandbox = {
    window: {
      SlidesTableMedia: stm,
      gbT: function (key, fallback) { return fallback; }
    },
    document: fakeDocument(),
    console: silentConsole()
  };
  vm.runInContext(instrumented, vm.createContext(sandbox), {
    filename: EDITOR_REL + " (single appended export; bodies verbatim)"
  });
  return {
    stm: stm,
    internals: sandbox.window.__stmEditorInternals,
    source: source,
    instrumented: instrumented
  };
}

/* Fake outline host: O.slides() is derived from the rail, as soSlides() does. */
function makeOutlineHost(indices) {
  var rail = fakeNode();
  var thumbs = (indices || [0, 1, 2]).map(function (index) {
    var thumb = fakeNode({ "data-index": String(index) });
    rail.appendChild(thumb);
    return thumb;
  });
  var store = {};
  var statuses = [];
  var confirms = [];
  var undos = [];
  var confirmResult = true;

  function slides() {
    return rail.children.map(function (thumb, position) {
      var raw = thumb.getAttribute("data-index") || thumb.getAttribute("data-slide");
      var index = parseInt(raw, 10);
      if (!isFinite(index)) index = position;
      return { index: index, thumb: thumb, elements: [], canvasElements: [] };
    });
  }

  var O = {
    $$: function (selector, host) { return host && host.__checked ? host.__checked : []; },
    $: function (selector) { return selector === "#sidebar-thumbs" ? rail : null; },
    slides: slides,
    t: function (key, fallback, params) {
      return String(fallback).replace(/\{(\w+)\}/g, function (all, name) {
        return params && params[name] !== undefined ? String(params[name]) : all;
      });
    },
    save: function (key, value) { store[key] = JSON.parse(JSON.stringify(value)); },
    load: function (key, fallback) {
      return store[key] === undefined
        ? JSON.parse(JSON.stringify(fallback))
        : JSON.parse(JSON.stringify(store[key]));
    },
    status: function (message) { statuses.push(message); },
    confirm: function (config) {
      confirms.push(config);
      return {
        then: function (onOk) { if (confirmResult) onOk(true); return this; }
      };
    },
    pushUndo: function (action) { undos.push(action); },
    el: function (tag, cls, text, key) {
      var node = fakeNode();
      node.tag = tag;
      node.className = cls || "";
      if (text !== undefined) node.textContent = text;
      if (key) node.setAttribute("data-i18n", key);
      return node;
    },
    openPanel: function () { return fakeNode(); },
    closePanels: function () {},
    updateUndoUI: function () {},
    undo: function () {},
    localBadge: function () { return fakeNode(); },
    setConfirmResult: function (value) { confirmResult = !!value; }
  };

  var host = fakeNode();
  host.__checked = [];
  host.check = function (checkedIndices) {
    host.__checked = checkedIndices.map(function (index) {
      var box = fakeNode({ "data-bulk-index": String(index) });
      box.type = "checkbox";
      return box;
    });
  };
  return {
    O: O, rail: rail, thumbs: thumbs, host: host, store: store,
    statuses: statuses, confirms: confirms, undos: undos, slides: slides
  };
}

function loadBulkOps(outline) {
  var source = readFile(BULK_REL);
  var instrumented = instrument(source, BULK_MARKER, BULK_EXPORT);
  var sandbox = {
    window: { GBSlidesOutline: outline },
    document: fakeDocument(),
    console: silentConsole()
  };
  vm.runInContext(instrumented, vm.createContext(sandbox), {
    filename: BULK_REL + " (single appended export; bodies verbatim)"
  });
  return {
    api: sandbox.window.GBSlidesBulk,
    internals: sandbox.window.__stmBulkInternals,
    source: source,
    instrumented: instrumented
  };
}

var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function report() {
  var rule = "=".repeat(64);
  gbLog.debug("\n" + rule);
  gbLog.debug("SLIDES TABLE/MEDIA — OFFLINE TEST RESULTS (#2487)");
  gbLog.debug(rule);
  RESULTS.forEach(function (result) {
    gbLog.debug("[" + result.status + "] " + result.name + (result.error ? " — " + result.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("MODE: runtime (vm sandbox; real shipped bodies, stubbed host)");
  gbLog.debug("NOT COVERED: native PPTX a:tbl export, video trim, captions, real-slide UAT");
  gbLog.debug(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

function finish() { report(); }

module.exports = {
  test: test,
  assert: assert,
  readFile: readFile,
  instrument: instrument,
  stripInstrumentation: stripInstrumentation,
  fakeNode: fakeNode,
  fakeDocument: fakeDocument,
  loadTableModel: loadTableModel,
  loadEditor: loadEditor,
  makeOutlineHost: makeOutlineHost,
  loadBulkOps: loadBulkOps,
  finish: finish,
  SUITE_DIR: SUITE_DIR,
  MODEL_REL: MODEL_REL,
  EDITOR_REL: EDITOR_REL,
  BULK_REL: BULK_REL,
  EDITOR_MARKER: EDITOR_MARKER,
  EDITOR_EXPORT: EDITOR_EXPORT,
  BULK_MARKER: BULK_MARKER,
  BULK_EXPORT: BULK_EXPORT
};
