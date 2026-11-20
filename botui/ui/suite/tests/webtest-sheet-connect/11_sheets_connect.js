/**
 * Sheets connectivity — connector registry and query facade RUNTIME cases
 * (#2492). addStep/move/remove are private to the shipped query IIFE, so they
 * are driven through the real DOM handlers that its renderer wires up.
 */
"use strict";

var h = require("./00_harness");
var env = h.env;
var W = env.window;
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var deep = h.deep;

var C = W.SheetConnectCore;
var K = W.SheetConnectConnectors;
var Q = W.SheetConnectQuery;

function ctx() {
  var value = {
    statuses: [],
    renders: 0,
    state: function () { return C.state(); },
    setStatus: function (message, isError) {
      value.statuses.push({ message: message, isError: Boolean(isError) });
    },
    rerender: function () { value.renders++; },
  };
  return value;
}

function findByI18n(root, key) {
  return root.querySelectorAll("[data-i18n]").filter(function (node) {
    return node.getAttribute("data-i18n") === key;
  })[0];
}

function findByAriaKey(root, key) {
  return root.querySelectorAll("[data-i18n-aria-label]").filter(function (node) {
    return node.getAttribute("data-i18n-aria-label") === key;
  });
}

function renderQuery(container, c) {
  container.textContent = "";
  Q.render(container, c);
}

// ------------------------------------------------------------- connectors
test("connectors expose the six gallery types and two Vault types", function () {
  deep(K.TYPES.map(function (t) { return t.value; }), C.CONNECTOR_TYPES);
  deep(K.VAULT_TYPES, ["postgres", "mysql"]);
  deep(K.newDraft(), {
    name: "", type: "csv", url: "", param: "page", path: "",
    sheet: "", credentialRef: "", host: "",
  });
  equal(K.newDraft("rest").type, "rest");
});

test("connectors validate name, type, URL and host allowlist", function () {
  var state = C.emptyState();
  var empty = K.validateConnector({}, state);
  equal(empty.ok, false);
  assert(empty.errors.some(function (e) { return e.key === "sheet-connect-validation-name"; }));
  assert(empty.errors.some(function (e) { return e.key === "sheet-connect-validation-type"; }));

  var good = { name: "Local CSV", type: "csv", url: "https://data.example.com/feed" };
  equal(K.validateConnector(good, state).ok, true);
  var badUrl = K.validateConnector({ name: "X", type: "csv", url: "not-a-url" }, state);
  equal(badUrl.ok, false);
  assert(badUrl.errors.some(function (e) { return e.key === "sheet-connect-validation-url"; }));

  var blocked = K.validateConnector({ name: "X", type: "rest", url: "https://evil.example.org/p" }, state);
  equal(blocked.ok, false);
  equal(blocked.errors.length, 2);
  equal(blocked.errors[0].key, "sheet-connect-allowlist-required");
  equal(blocked.errors[0].host, "evil.example.org");
  equal(blocked.errors[1].key, "sheet-connect-host-blocked");
  state.allowlist.push("evil.example.org");
  equal(K.validateConnector({ name: "X", type: "rest", url: "https://evil.example.org/p" }, state).ok, true);
});

test("connectors accept vault references and reject secret values", function () {
  var state = C.emptyState();
  var vault = { name: "DB", type: "postgres", credentialRef: "vault://secret/gbo/db" };
  equal(K.validateConnector(vault, state).ok, true);
  var literal = { name: "DB", type: "postgres", credentialRef: "plaintext" };
  equal(K.validateConnector(literal, state).ok, false);
  var withSecret = {
    name: "DB", type: "mysql", credentialRef: "vault://secret/gbo/db", password: "plaintext",
  };
  var result = K.validateConnector(withSecret, state);
  equal(result.ok, false);
  assert(result.errors.some(function (e) { return e.key === "sheet-connect-validation-secret-value"; }));
  equal(K.validateConnector({ name: "File", type: "drive" }, state).ok, false);
  equal(K.validateConnector({ name: "File", type: "drive", path: "/a.csv" }, state).ok, true);
});

test("connectors configOf stores only the fields each type needs", function () {
  deep(K.configOf({ type: "rest", url: " https://data.example.com/f ", param: "page" }), {
    url: "https://data.example.com/f", param: "page", host: "data.example.com",
  });
  deep(K.configOf({ type: "postgres", credentialRef: "vault://secret/gbo/db" }), {
    credentialRef: "vault://secret/gbo/db",
  });
  deep(K.configOf({ type: "gsheet", path: "/sheet", sheet: "Tab1" }), {
    path: "/sheet", sheet: "Tab1",
  });
});

test("connectors create persists a valid connector and rejects an invalid one", function () {
  env.reset();
  var c = ctx();
  var fields = { name: " Local CSV ", type: "csv", url: "https://data.example.com/feed" };
  var created = K.create(fields, null, c);
  equal(created.ok, true);
  equal(C.state().connectors.length, 1);
  assert(/^conn-/.test(created.connector.id));
  equal(created.connector.name, "Local CSV");
  equal(C.state().activeConnectorId, created.connector.id);
  equal(c.renders, 1);
  assert(c.statuses[0].message.indexOf("{sheet-connect-connector-saved}") === 0);

  env.reset();
  var c2 = ctx();
  equal(K.create({}, null, c2).ok, false);
  equal(C.state().connectors.length, 0);
  equal(c2.renders, 0);
  equal(c2.statuses[0].isError, true);
});

test("connectors edit, activate and remove keep one active connector", function () {
  env.reset();
  var c = ctx();
  K.create({ name: "A", type: "csv", url: "https://data.example.com/a" }, null, c);
  env.setRandom(0.5);
  K.create({ name: "B", type: "csv", url: "https://data.example.com/b" }, null, c);
  var ids = C.state().connectors.map(function (item) { return item.id; });
  equal(ids.length, 2);
  assert(ids[0] !== ids[1]);

  K.activate(ids[1], c);
  equal(C.state().activeConnectorId, ids[1]);
  K.create({ name: "B2", type: "csv", url: "https://data.example.com/b" }, ids[1], c);
  equal(C.state().connectors.length, 2);
  equal(C.state().connectors[1].name, "B2");

  K.remove(ids[1], c);
  equal(C.state().connectors.length, 1);
  equal(C.state().activeConnectorId, ids[0]);
  K.remove(ids[0], c);
  equal(C.state().activeConnectorId, null);
  env.setRandom(0);
});

test("connectors host approval is lowercased and idempotent", function () {
  env.reset();
  var c = ctx();
  equal(K.approve("HTTPS://Data.Example.ORG/x", c), true);
  deep(C.state().allowlist, ["data.example.com", "data.example.org"]);
  equal(K.approve("data.example.org", c), false);
  equal(C.state().allowlist.length, 2);
  K.unapprove("data.example.org", c);
  deep(C.state().allowlist, ["data.example.com"]);
  equal(K.approve("", c), false);
  equal(c.statuses[c.statuses.length - 1].isError, true);
});

// ------------------------------------------------------------------- query
test("query exposes the six step types and their default configs", function () {
  deep(Q.TYPES.map(function (t) { return t.value; }), C.STEP_TYPES);
  deep(Q.DEFAULTS, {
    select: { columns: "date,region,amount" },
    filter: { field: "amount", op: "gt", value: "100" },
    rename: { from: "amount", to: "total" },
    type: { field: "amount", to: "number" },
    unpivot: { columns: "amount", categoryHeader: "series", valueHeader: "value" },
    group: { by: "region", aggField: "amount", agg: "sum" },
  });
});

test("query reorder moves steps inside bounds without touching invalid moves", function () {
  var steps = [{ id: "a" }, { id: "b" }, { id: "c" }];
  equal(Q.reorder(steps, "c", -1), true);
  deep(steps.map(function (s) { return s.id; }), ["a", "c", "b"]);
  equal(Q.reorder(steps, "a", 1), true);
  deep(steps.map(function (s) { return s.id; }), ["c", "a", "b"]);
  equal(Q.reorder(steps, "c", -1), false);
  equal(Q.reorder(steps, "b", 1), false);
  deep(steps.map(function (s) { return s.id; }), ["c", "a", "b"]);
});

test("query add/move/remove handlers mutate workbook state through ctx", function () {
  env.reset();
  var c = ctx();
  var container = env.document.createElement("div");
  renderQuery(container, c);
  var addFilter = findByI18n(container, "sheet-connect-step-filter");
  assert(addFilter);
  addFilter.dispatch("click");
  assert(/^step-/.test(C.state().steps[0].id));
  equal(C.state().steps[0].enabled, true);
  deep(C.state().steps[0].config, Q.DEFAULTS.filter);
  assert(c.statuses[0].message.indexOf("{sheet-connect-step-saved}") === 0);

  renderQuery(container, c);
  env.setRandom(0.5);
  findByI18n(container, "sheet-connect-step-select").dispatch("click");
  var ids = C.state().steps.map(function (s) { return s.id; });
  equal(ids.length, 2);
  assert(ids[0] !== ids[1]);

  renderQuery(container, c);
  findByAriaKey(container, "sheet-connect-step-up")[1].dispatch("click");
  equal(C.state().steps[0].id, ids[1]);
  equal(c.statuses[c.statuses.length - 1].message, "{sheet-connect-step-moved}");

  renderQuery(container, c);
  findByAriaKey(container, "sheet-connect-step-delete")[0].dispatch("click");
  equal(C.state().steps.length, 1);
  equal(c.statuses[c.statuses.length - 1].message, "{sheet-connect-step-removed}");
  env.setRandom(0);
});
