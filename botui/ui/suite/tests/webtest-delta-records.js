/**
 * Offline verification for the shared gb-doc-delta record renderer.
 * Run with: node botui/ui/suite/tests/webtest-delta-records.js
 *
 * Loads the REAL shipped botui/ui/suite/js/delta-records.js verbatim in a Node
 * vm with a minimal DOM stub. The row label is derived from stored fields;
 * patch_field must recompute it so a patched non-title field (price, qty,
 * status) is not left stale on screen while the model already holds the value.
 * No production logic is reimplemented here.
 *
 * Regression: patch_field on a JSON-labelled row used to update dataset.fields
 * only, leaving textContent stale — the visible cell contradicted the model.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..");
var SRC = path.join(SUITE_DIR, "js", "delta-records.js");
var PASS = 0;
var FAIL = 0;

function test(name, fn) {
  try { fn(); PASS++; console.log("[PASS] " + name); }
  catch (e) { FAIL++; console.log("[FAIL] " + name + " :: " + e.message); }
}
function assert(cond, msg) { if (!cond) throw new Error(msg || "Assertion failed"); }
function assertEqual(actual, expected, msg) {
  if (actual !== expected) throw new Error((msg || "values differ") + " :: <" + actual + "> != <" + expected + ">");
}

function El(tag) {
  this.tagName = tag || "div";
  this.className = "";
  this.dataset = {};
  this.attributes = {};
  this._text = "";
  this.children = [];
  this.parentNode = null;
}
Object.defineProperty(El.prototype, "textContent", {
  get: function () { return this._text; },
  set: function (v) { this._text = String(v); }
});
El.prototype.setAttribute = function (k, v) {
  this.attributes[k] = v;
  if (k.indexOf("data-") === 0) {
    this.dataset[k.slice(5).replace(/-([a-z])/g, function (_, c) { return c.toUpperCase(); })] = v;
  }
};
El.prototype.getAttribute = function (k) { return this.attributes[k]; };
El.prototype.appendChild = function (c) { c.parentNode = this; this.children.push(c); return c; };
El.prototype.insertBefore = function (c, ref) {
  var i = this.children.indexOf(ref);
  if (i < 0) i = this.children.length;
  this.children.splice(i, 0, c);
  c.parentNode = this;
  return c;
};
El.prototype.remove = function () {
  if (this.parentNode) {
    var i = this.parentNode.children.indexOf(this);
    if (i >= 0) this.parentNode.children.splice(i, 1);
  }
};
El.prototype.querySelector = function (sel) {
  var m = /^\[data-record-id="(.*)"\]$/.exec(sel);
  if (!m) return null;
  for (var i = 0; i < this.children.length; i++) {
    if (this.children[i].getAttribute("data-record-id") === m[1]) return this.children[i];
  }
  return null;
};

function loadRenderer() {
  var document = {
    readyState: "complete",
    createElement: function (t) { return new El(t); },
    querySelector: function () { return null; },
    addEventListener: function () {}
  };
  var window = { GBDocDelta: { register: function () {} }, setTimeout: function () {} };
  var sandbox = { window: window, document: document, JSON: JSON, Math: Math, Object: Object, console: console };
  vm.createContext(sandbox);
  vm.runInContext(fs.readFileSync(SRC, "utf8"), sandbox);
  return window.GBDeltaRecords;
}

var R = loadRenderer();

function row(id, fields) {
  var c = new El("div");
  R.applyToContainer(c, { ops: [{ op: "upsert_record", id: id, fields: fields }] });
  return { container: c, el: c.querySelector('[data-record-id="' + id + '"]') };
}

test("upsert label stays byte-identical for a titled record", function () {
  var r = row("t1", { title: "Widget Pro", price: 19.99 });
  assertEqual(r.el.textContent, "Widget Pro");
});

test("upsert label stays byte-identical for a JSON-labelled record", function () {
  var r = row("j1", { sku: "X1", qty: 3 });
  assertEqual(r.el.textContent, '{"sku":"X1","qty":3}');
});

test("REGRESSION: patch_field refreshes the visible label of a JSON row", function () {
  var r = row("p1", { sku: "X1", qty: 3 });
  R.applyToContainer(r.container, { ops: [{ op: "patch_field", id: "p1", field: "qty", value: 5 }] });
  assertEqual(r.el.dataset.fields, '{"sku":"X1","qty":5}', "model updated");
  assertEqual(r.el.textContent, '{"sku":"X1","qty":5}', "visible label tracks model");
});

test("patch_field refreshes the visible label of a titled record", function () {
  var r = row("t2", { title: "Widget Pro", price: 19.99 });
  R.applyToContainer(r.container, { ops: [{ op: "patch_field", id: "t2", field: "title", value: "Widget Max" }] });
  assertEqual(r.el.textContent, "Widget Max");
});

test("patch_field on an unknown id is a no-op", function () {
  var c = new El("div");
  R.applyToContainer(c, { ops: [{ op: "patch_field", id: "missing", field: "qty", value: 5 }] });
  assertEqual(c.children.length, 0);
});

test("[UAT:DB-07] delete_record removes the row", function () {
  var r = row("d1", { title: "Gone" });
  R.applyToContainer(r.container, { ops: [{ op: "delete_record", id: "d1" }] });
  assert(r.container.querySelector('[data-record-id="d1"]') === null);
});

console.log("============================================================");
console.log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
console.log("============================================================");
process.exit(FAIL ? 1 : 0);
