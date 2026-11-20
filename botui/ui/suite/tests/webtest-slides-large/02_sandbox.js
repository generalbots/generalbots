/**
 * Deterministic vm sandbox for two real shipped Slides modules.
 *
 * MODULES are loaded UNMODIFIED:
 *   01_state.js  constants + SlideCanvas shell (no auto-run DOM binding)
 *   13_delta.js  incremental-save / gb-doc-delta op application
 *
 * Stubs cover window/document/fetch/timers/console with a synchronous
 * thenable so every promise chain settles inside the same test tick. No
 * production logic is reimplemented: fixtures only feed real shipped code and
 * observe the calls it makes.
 */
"use strict";

var h = require("./00_harness");

var MODULES = [
  "slides/modules/01_state.js",
  "slides/modules/13_delta.js",
];

/** A thenable that settles synchronously; `failed` propagates to .catch(). */
function thenable(value, failed) {
  var isFailed = failed === true;
  return {
    then: function (fn, onReject) {
      if (isFailed) {
        return typeof onReject === "function" ? thenable(onReject(value)) : thenable(value, true);
      }
      if (typeof fn !== "function") return this;
      try {
        return thenable(fn(value));
      } catch (error) {
        return thenable(error, true);
      }
    },
    catch: function (onReject) {
      return isFailed && typeof onReject === "function" ? thenable(onReject(value)) : this;
    },
  };
}

function defaultResponse() {
  return {
    ok: true,
    status: 200,
    json: function () { return thenable({ ok: true }); },
  };
}

/**
 * Fresh sandbox. `options.response` may simulate an HTTP failure;
 * `options.nodes` seeds document.getElementById results.
 */
function fixture(options) {
  options = options || {};
  var calls = { fetch: [], refresh: [], toast: [], registered: [], listeners: [] };
  var response = options.response || defaultResponse();
  var nodes = options.nodes || {};
  var sandbox = {
    console: { log: function () {}, error: function () {} },
    setTimeout: function () { return 0; },
    clearTimeout: function () {},
    fetch: function (url, opts) {
      calls.fetch.push({ url: url, opts: opts || {} });
      return thenable(response);
    },
    document: {
      getElementById: function (id) {
        return Object.prototype.hasOwnProperty.call(nodes, id) ? nodes[id] : null;
      },
      querySelector: function () { return null; },
      addEventListener: function (name) { calls.listeners.push(name); },
    },
    window: {
      __SLIDES_PRESENTATION_ID: "deck-fixture",
      getSlidesPresentationId: function () { return "deck-fixture"; },
      refreshSlideView: function (index) { calls.refresh.push(index); },
      GBToasts: {
        show: function (scope, message, kind) {
          calls.toast.push({ scope: scope, message: message, kind: kind });
        },
      },
      GBDocDelta: {
        register: function (kind, fn) {
          calls.registered.push({ kind: kind, fn: fn });
        },
      },
    },
  };
  var ctx = h.loadModules(MODULES, sandbox);
  return {
    ctx: ctx,
    win: sandbox.window,
    calls: calls,
    sandbox: sandbox,
  };
}

/** Parse the JSON body of the Nth recorded fetch call. */
function bodyAt(fx, index) {
  var raw = fx.calls.fetch[index].opts.body;
  return JSON.parse(raw);
}

module.exports = {
  MODULES,
  thenable,
  defaultResponse,
  fixture,
  bodyAt,
};
