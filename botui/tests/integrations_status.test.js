"use strict";
/* #1616 — connected-services card must never present a status-load failure as
 * a connection state. Zero-dependency test (run: node tests/integrations_status.test.js)
 *
 * Loads the real state + render modules with a tiny element shim and asserts
 * both branches required by the issue:
 *   - status request succeeds  -> real rows / "0 connected services"
 *   - status request fails     -> "Status unavailable" + retry, with no
 *                                 "could not be loaded" connection wording
 */

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const MODULES = path.join(__dirname, "..", "ui", "suite", "integrations", "modules");

let passed = 0;
let failed = 0;

function check(condition, label) {
  if (condition) passed++;
  else {
    failed++;
    console.error("FAIL " + label);
  }
}

function element() {
  return { innerHTML: "", textContent: "", attributes: {}, setAttribute: function (k, v) { this.attributes[k] = v; }, removeAttribute: function (k) { delete this.attributes[k]; } };
}

function makeRoot() {
  const nodes = {
    "[data-connected-view]": element(),
    "[data-connected-status]": element(),
    "[data-global-live]": element(),
  };
  return {
    nodes: nodes,
    querySelector: function (selector) { return nodes[selector] || null; },
  };
}

function loadModules() {
  const window = {};
  const sandbox = {
    window: window,
    document: { querySelector: function () { return null; } },
    console: console,
    Map: Map,
    Set: Set,
    Promise: Promise,
    JSON: JSON,
    Object: Object,
    Array: Array,
    String: String,
    Number: Number,
    Date: Date,
    Intl: Intl,
    setTimeout: function () {},
  };
  for (const file of ["01_state.js", "02_render.js"]) {
    vm.runInNewContext(fs.readFileSync(path.join(MODULES, file), "utf8"), sandbox, {
      filename: path.join(MODULES, file),
    });
  }
  return window.GBIntegrationsCatalog;
}

const api = loadModules();

// --- Success branch: real rows render a real connection state.
let root = makeRoot();
let state = api.createState(root);
state.connectedStatus = "ready";
state.connectors = [{
  id: "conn-1",
  display_name: "Work Gmail",
  provider_slug: "gmail",
  auth_kind: "oauth2",
  status: "connected",
  granted_scopes: ["gmail.readonly"],
  visibility: "private",
  credential_version: 2,
  expires_at: null,
}];
api.renderConnected(root);
check(root.nodes["[data-connected-status]"].textContent === "1 connected service",
  "success branch reports the real count");
check(root.nodes["[data-connected-view]"].innerHTML.indexOf("Work Gmail") !== -1,
  "success branch renders the connection row");
check(root.nodes["[data-connected-view]"].innerHTML.indexOf("gmail.readonly") !== -1,
  "success branch shows the granted scope");

// Empty success is a real state, not an error.
root = makeRoot();
state = api.createState(root);
state.connectors = [];
api.renderConnected(root);
check(root.nodes["[data-connected-status]"].textContent === "0 connected services",
  "empty success reports zero, not a failure");
check(root.nodes["[data-connected-view]"].innerHTML.indexOf("No connected services yet") !== -1,
  "empty success offers the explore action");

// --- Failure branch: status request does not answer.
root = makeRoot();
state = api.createState(root);
state.connectors = [];
api.renderConnectedUnavailable(root);
const unavailable = root.nodes["[data-connected-view]"].innerHTML;
check(root.nodes["[data-connected-status]"].textContent === "Status unavailable",
  "load failure reports status unavailable");
check(unavailable.indexOf("Status unavailable") !== -1, "card headline is the failure, not a connection");
check(unavailable.indexOf('data-action="retry-connected"') !== -1, "failure offers a retry action");
check(unavailable.indexOf("could not be loaded") === -1,
  "no 'could not be loaded' string is presented as a connection state");
check(unavailable.indexOf("not a disconnected account") !== -1,
  "copy explicitly distinguishes failure from disconnected");

// Error branch with a cause stays honest and still offers retry.
root = makeRoot();
state = api.createState(root);
state.connectors = [];
api.renderConnectedError(root, "HTTP 504");
const errored = root.nodes["[data-connected-view]"].innerHTML;
check(errored.indexOf("HTTP 504") !== -1, "reported cause is shown");
check(errored.indexOf('data-action="retry-connected"') !== -1, "error branch offers retry");
check(errored.indexOf("could not be loaded") === -1, "error branch never claims a connection state");

console.log(`integrations_status: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
