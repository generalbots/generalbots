"use strict";
/* GB6-18 (#1569) — the consent cell must show exactly what a connection shares
 * and who can use it. Zero-dependency test
 * (run: node tests/connector_consent.test.js)
 *
 * Loads the real integrations state + render modules and drives sharedCell,
 * the function behind the "Shared" column that sits next to Revoke. */

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
const api = window.GBIntegrationsCatalog;

// 1. Granted scopes and visibility are both shown.
let html = api.sharedCell({ granted_scopes: ["gmail.readonly", "files.metadata.read"], visibility: "private" });
check(html.indexOf("gmail.readonly") !== -1, "the granted scope is shown");
check(html.indexOf("files.metadata.read") !== -1, "every granted scope is shown");
check(html.indexOf("only you") !== -1, "private visibility is stated");
check(html.indexOf("<button") === -1, "the cell renders no hidden action of its own");

// 2. A branch-visible connection is labelled as such.
html = api.sharedCell({ granted_scopes: ["Files.Read"], visibility: "branch" });
check(html.indexOf("branch workspace") !== -1, "branch visibility is stated");

// 3. Long scope lists stay readable and are complete in the tooltip.
html = api.sharedCell({
  granted_scopes: ["a.read", "b.read", "c.read", "d.read", "e.read"],
  visibility: "private",
});
check(html.indexOf("a.read") !== -1 && html.indexOf("b.read") !== -1, "the first scopes are visible");
check(html.indexOf("+2") !== -1, "the overflow count is shown");
check(html.indexOf("Scopes: a.read, b.read, c.read, d.read, e.read") !== -1,
  "the tooltip carries the complete scope list");

// 4. A connection that reported no scopes says so instead of implying full access.
html = api.sharedCell({ granted_scopes: [], visibility: "" });
check(html.indexOf("No scopes recorded") !== -1, "missing scopes are stated honestly");
check(html.indexOf("Scopes:") === -1 || html.indexOf("No scopes recorded") !== -1,
  "an empty list never looks like a grant");

// 5. Rows always offer revoke next to the shared description.
function fakeRoot() {
  const nodes = {};
  return {
    nodes: nodes,
    querySelector: function (selector) {
      if (!nodes[selector]) {
        nodes[selector] = {
          innerHTML: "",
          textContent: "",
          setAttribute: function () {},
          querySelector: function () { return null; },
        };
      }
      return nodes[selector];
    },
  };
}
const root = fakeRoot();
const state = api.createState(root);
state.connectors = [{ id: "c1", display_name: "Work", provider_slug: "gmail", auth_kind: "oauth2", status: "connected", granted_scopes: ["gmail.readonly"], visibility: "private", credential_version: 1, expires_at: null }];
api.renderConnected(root);
const connectedHtml = root.nodes["[data-connected-view]"].innerHTML;
check(connectedHtml.indexOf('data-action="revoke-connection"') !== -1, "revoke is offered next to the shared description");
check(connectedHtml.indexOf("gmail.readonly") !== -1, "the connected row shows the granted scope");

console.log(`connector_consent: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
