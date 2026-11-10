"use strict";
/* #2100 — the Integrations Explore pane must never be blank.
 * Zero-dependency test (run: node tests/integrations_catalog.test.js)
 *
 * Loads the real state + render modules with a tiny element shim and asserts
 * the four ways the catalog used to read as empty:
 *   - Preview OFF narrowed to "must" and a payload without `priority` hid
 *     every provider (the desktop default, where preview is off);
 *   - an empty payload and a failed request were not told apart;
 *   - a 404 (build/profile without the route) read as "no providers";
 *   - an exception during initialization left the pane blank.
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
  return {
    innerHTML: "",
    textContent: "",
    id: "node-id",
    attributes: {},
    classList: { add() {}, remove() {} },
    setAttribute: function (k, v) { this.attributes[k] = v; },
    removeAttribute: function (k) { delete this.attributes[k]; },
    querySelector: function () { return null; }
  };
}

function makeRoot() {
  const nodes = {
    "[data-catalog-view]": element(),
    "[data-category-filters]": element(),
    "[data-category-panel]": element(),
    "[data-explore-notice]": element(),
    "[data-result-status]": element(),
    "[data-catalog-health]": element(),
    "[data-catalog-health-text]": element(),
    "[data-connected-view]": element(),
    "[data-connected-status]": element(),
    "[data-global-live]": element(),
    '[data-category][aria-selected="true"]': element()
  };
  return {
    nodes: nodes,
    dataset: {},
    querySelector: function (selector) { return nodes[selector] || null; },
    querySelectorAll: function () { return []; }
  };
}

function loadModules(previewOn) {
  const window = {};
  if (typeof previewOn === "boolean") {
    window.GBPreviewMode = { isOn: function () { return previewOn; } };
  }
  const sandbox = {
    window: window,
    document: { querySelector: function () { return null; } },
    location: { pathname: "/suite/desktop.html", search: "" },
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
    URL: URL
  };
  for (const file of ["01_state.js", "02_render.js"]) {
    vm.runInNewContext(fs.readFileSync(path.join(MODULES, file), "utf8"), sandbox, {
      filename: path.join(MODULES, file),
    });
  }
  return window.GBIntegrationsCatalog;
}

function provider(id, extra) {
  return Object.assign({
    id: id,
    name: id.toUpperCase(),
    category: "productivity",
    strategy: "integrate",
    status: "partial",
    priority: "must",
    auth: { method: "api_key" },
    actions: [],
    action_count: 0,
    llm_available: true
  }, extra || {});
}

// --- Default filter follows the payload, not the build.
let api = loadModules(false);
check(api.preferredStatusFilter([provider("gmail")]) === "must",
  "preview OFF keeps the production-ready default when the payload carries priorities");
check(api.preferredStatusFilter([provider("cursor", { priority: "nice" })]) === "all",
  "preview OFF falls back to all when no provider is priority=must");
const noPriority = [provider("gmail")];
delete noPriority[0].priority;
check(api.normalizeProvider(noPriority[0]).priority === "standard",
  "a payload without priority normalizes to standard (the field can be absent)");
check(api.preferredStatusFilter([api.normalizeProvider(noPriority[0])]) === "all",
  "a catalog without priorities never narrows to must");

api = loadModules(true);
check(api.preferredStatusFilter([provider("cursor", { priority: "nice" })]) === "all",
  "preview ON keeps the full catalog");

// --- A filter that hides everything says so and offers a way back.
api = loadModules(false);
let root = makeRoot();
let state = api.createState(root);
state.providers = [provider("cursor", { priority: "nice" })];
state.providerMap = new Map([["cursor", state.providers[0]]]);
state.categories = [{ value: "productivity", label: "Productivity", count: 1 }];
state.statusFilter = "must";
api.renderExplore(root);
const filteredOut = root.nodes["[data-catalog-view]"].innerHTML;
check(filteredOut.indexOf("No providers in this filter") !== -1,
  "an availability filter that hides every provider is named");
check(filteredOut.indexOf('data-action="show-all-providers"') !== -1,
  "the hidden-catalog state offers Show all");
check(filteredOut.indexOf("1 providers") !== -1,
  "the Show all action states how many providers it reveals");
check(root.nodes["[data-result-status]"].textContent.indexOf("production-ready") !== -1,
  "the count names the active availability filter");

// --- An empty catalog is distinguishable from a failed request.
root = makeRoot();
state = api.createState(root);
state.providers = [];
state.categories = [];
api.renderExplore(root);
const empty = root.nodes["[data-catalog-view]"].innerHTML;
check(empty.indexOf("The catalog is empty") !== -1, "an empty payload says the catalog is empty");
check(empty.indexOf('data-action="retry-catalog"') !== -1, "an empty payload offers a refresh");

// --- A missing route names the build, not an empty catalog.
root = makeRoot();
state = api.createState(root);
const missing = new Error("Request failed with status 404");
missing.status = 404;
api.renderCatalogError(root, missing);
const notFound = root.nodes["[data-catalog-view]"].innerHTML;
check(notFound.indexOf("does not expose the catalog API") !== -1,
  "a 404 names the missing route instead of reporting an empty catalog");
check(notFound.indexOf("/api/apps/integrations/catalog") !== -1, "the failing endpoint is shown");
check(notFound.indexOf('data-action="retry-catalog"') !== -1, "the 404 state offers a retry");
check(notFound.indexOf("no providers") === -1, "the 404 state never claims the catalog has no providers");

// --- Network and server failures keep the retry path and stay honest.
root = makeRoot();
state = api.createState(root);
api.renderCatalogError(root, new Error("Failed to fetch"));
check(root.nodes["[data-catalog-view]"].innerHTML.indexOf("unreachable") !== -1,
  "a network failure reports an unreachable service");

root = makeRoot();
state = api.createState(root);
const serverError = new Error("Request failed with status 503");
serverError.status = 503;
api.renderCatalogError(root, serverError);
check(root.nodes["[data-catalog-view]"].innerHTML.indexOf("HTTP 503") !== -1,
  "a server failure shows the HTTP status");

// --- No workspace context is a state, not a status failure.
root = makeRoot();
state = api.createState(root);
api.renderConnectedNoContext(root);
check(root.nodes["[data-connected-status]"].textContent.indexOf("No active bot") !== -1,
  "a missing workspace context is reported as such");
check(root.nodes["[data-connected-view]"].innerHTML.indexOf("No active bot/workspace context") !== -1,
  "the no-context card explains itself");
check(root.nodes["[data-connected-view]"].innerHTML.indexOf("Status unavailable") === -1,
  "the no-context card is not presented as a failure");

// --- An exception during init renders an actionable card, never a blank pane.
root = makeRoot();
state = api.createState(root);
api.renderCatalogUnavailable(root, new Error("GBIntegrationsCatalog is undefined"));
const crashed = root.nodes["[data-catalog-view]"].innerHTML;
check(crashed.indexOf("Integrations could not start") !== -1, "an init failure is reported");
check(crashed.indexOf("GBIntegrationsCatalog is undefined") !== -1, "the init failure names its cause");
check(crashed.indexOf('data-action="retry-catalog"') !== -1, "the init failure offers a reload");
check(root.nodes["[data-catalog-health-text]"].textContent.indexOf("unavailable") !== -1,
  "the health badge stops claiming the catalog is loading");

console.log(`integrations_catalog: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
