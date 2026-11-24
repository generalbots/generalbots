/**
 * Shared sandbox helpers for the UAT-gap step-level tests.
 * Reuses the webtest-apps runner so results print in the same format.
 */
"use strict";

var h = require("../webtest-apps/00_harness");

/** A string-valued localStorage stand-in backed by a plain object. */
function makeStore(seed) {
  var data = Object.assign({}, seed || {});
  return {
    data: data,
    api: {
      getItem: function (key) {
        return Object.prototype.hasOwnProperty.call(data, key) ? data[key] : null;
      },
      setItem: function (key, value) { data[key] = String(value); },
      removeItem: function (key) { delete data[key]; },
    },
  };
}

/** Common browser globals for the module-style suite apps under test. */
function baseSandbox(doc, store, extra) {
  var sandbox = {
    window: null,
    document: doc,
    localStorage: store ? store.api : makeStore().api,
    gbT: function (key, fallback) { return fallback === undefined ? key : fallback; },
    gbFormatDate: function () { return "stub-date"; },
    gbFormatDateTime: function () { return "stub-date-time"; },
    gbLogError: function () {},
    gbLog: {
      debug: function () {}, info: function () {},
      warn: function () {}, error: function () {},
    },
    console: {
      debug: function () {}, log: function () {},
      warn: function () {}, error: function () {},
    },
    Date: Date, Math: Math, JSON: JSON, Promise: Promise,
    navigator: { clipboard: { writeText: function () { return Promise.resolve(); } } },
    setTimeout: function () { return 0; },
    clearTimeout: function () {},
    setInterval: function () { return 0; },
    clearInterval: function () {},
  };
  sandbox.gbStoreGet = function (key, fallback) {
    var raw = sandbox.localStorage.getItem(key);
    if (raw === null) return fallback;
    try { return JSON.parse(raw); } catch (e) { return fallback; }
  };
  if (extra) {
    Object.keys(extra).forEach(function (key) { sandbox[key] = extra[key]; });
  }
  sandbox.window = sandbox;
  return sandbox;
}

/** Load scripts in order into one vm context (they share window.* state). */
function loadAll(sandbox, paths) {
  paths.forEach(function (relativePath) { h.loadScript(relativePath, sandbox); });
  return sandbox;
}

module.exports = {
  h: h,
  makeStore: makeStore,
  baseSandbox: baseSandbox,
  loadAll: loadAll,
};
