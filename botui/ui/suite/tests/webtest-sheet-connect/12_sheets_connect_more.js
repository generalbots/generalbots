/**
 * Sheets connectivity — parameters, refresh, lineage and dialog-shell RUNTIME
 * cases plus labelled STATIC source assertions (#2492). Static checks are used
 * only for private/DOM-only/backend-deferred behaviour; they never fabricate a
 * runtime result. Split from 11_sheets_connect.js to stay under the 450-line
 * file limit.
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
var F = W.SheetConnectFeed;
var P = W.SheetConnectParams;
var R = W.SheetConnectRefresh;
var L = W.SheetConnectLineage;
var B = W.SheetConnect;

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

function feedState() {
  var state = C.emptyState();
  state.connectors = [{
    id: "c1", name: "Feed c1", type: "csv",
    config: { url: "https://data.example.com/feed" }, createdAt: "2026-01-01T00:00:00.000Z",
  }];
  state.activeConnectorId = "c1";
  return state;
}

function findByI18n(root, key) {
  return root.querySelectorAll("[data-i18n]").filter(function (node) {
    return node.getAttribute("data-i18n") === key;
  })[0];
}

// ------------------------------------------------------------------ params
test("params render binds current values into workbook state", function () {
  env.reset();
  var c = ctx();
  var container = env.document.createElement("div");
  P.render(container, c);
  var values = container.querySelectorAll("input").filter(function (node) {
    return node.dataset.i18nKey === "sheet-connect-param-value";
  });
  var defaults = container.querySelectorAll("input").filter(function (node) {
    return node.dataset.i18nKey === "sheet-connect-param-default";
  });
  equal(values.length, 3);
  equal(defaults.length, 3);
  assert(defaults.every(function (node) { return node.disabled === true; }));
  values[0].value = "2026-04-01";
  values[0].dispatch("change");
  equal(C.state().parameters[0].value, "2026-04-01");
  equal(c.statuses[c.statuses.length - 1].message, "{sheet-connect-param-saved}");
});

test("params reset restores every default through its shipped button", function () {
  env.reset();
  var c = ctx();
  var container = env.document.createElement("div");
  P.render(container, c);
  var button = findByI18n(container, "sheet-connect-param-reset");
  assert(button);
  button.dispatch("click");
  C.state().parameters.forEach(function (parameter) {
    equal(parameter.value, parameter.defaultValue);
  });
  equal(c.statuses[c.statuses.length - 1].message, "{sheet-connect-param-reset-done}");
  assert(c.renders > 0);
});

// ----------------------------------------------------------------- refresh
test("refresh time and status formatting are deterministic", function () {
  deep(R.FREQUENCIES.map(function (f) { return f.value; }), ["daily", "weekly", "monthly"]);
  equal(R.shortTime("2026-03-01T06:00:00.000Z"), "2026-03-01 06:00");
  equal(R.shortTime(null), "");
  equal(R.statusText("success"), "{sheet-connect-status-success}");
  equal(R.statusText("failed"), "{sheet-connect-status-failed}");
  equal(R.statusText("never"), "{sheet-connect-status-never}");
});

test("refresh facade reports failures without rerendering and success with it", function () {
  env.reset();
  var c = ctx();
  var failed = R.refresh(c);
  equal(failed.ok, false);
  equal(c.renders, 0);
  assert(c.statuses[0].isError);
  assert(c.statuses[0].message.indexOf("{sheet-connect-refresh-failed}") === 0);

  C.save(feedState());
  var good = R.refresh(c);
  equal(good.ok, true);
  equal(good.rows, 6);
  equal(c.renders, 1);
  assert(c.statuses[c.statuses.length - 1].message.indexOf("{sheet-connect-refresh-ok}") === 0);
  equal(C.state().snapshot.rowCount, 6);
});

test("refresh render marks failures with a live status region and audit table", function () {
  env.reset();
  var state = feedState();
  state.simulateFailure = true;
  F.refreshNow(state);
  C.save(state);
  var container = env.document.createElement("div");
  R.render(container, ctx());
  var notice = container.querySelector(".ss-connect-failure");
  assert(notice);
  equal(notice.getAttribute("role"), "status");
  equal(notice.getAttribute("aria-live"), "polite");
  var table = container.querySelector(".ss-connect-table");
  assert(table);
  equal(table.querySelectorAll("th").length, 4);
  equal(table.querySelectorAll("td").length, 4);
});

test("refresh render marks a cached snapshot as stale when offline", function () {
  env.reset();
  var state = feedState();
  F.refreshNow(state);
  C.save(state);
  C.mutate(function (target) { target.offline = true; });
  var container = env.document.createElement("div");
  R.render(container, ctx());
  assert(container.querySelector(".ss-connect-stale"));
});

// ----------------------------------------------------------------- lineage
test("lineage stepLabel resolves source, known and unknown step ids", function () {
  var state = feedState();
  state.steps = [{ id: "s1", type: "filter" }];
  equal(L.stepLabel(state, "source"), "{sheet-connect-lineage-source-step}");
  equal(L.stepLabel(state, null), "{sheet-connect-lineage-source-step}");
  equal(L.stepLabel(state, "missing-id"), "missing-id");
  equal(L.stepLabel(state, "s1"), "{sheet-connect-step-filter}");
});

test("lineage cell-click handler records the snapshot cell provenance", function () {
  // selectCell is private to 71_connect_lineage.js; drive it through the real
  // cell click callback that the shipped table renderer attaches.
  env.reset();
  var state = feedState();
  F.refreshNow(state);
  C.save(state);
  var c = ctx();
  var container = env.document.createElement("div");
  L.render(container, c);
  var cells = container.querySelector(".ss-connect-table").querySelectorAll("td");
  equal(cells.length, 24);
  cells[6].dispatch("click"); // row 1, col 2
  deep(C.state().selectedCell, {
    row: 1, col: 2, stepId: "source", sourceField: "amount",
  });
  assert(c.renders > 0);
});

test("lineage self-check reports pass and fail through ctx", function () {
  env.reset();
  var state = feedState();
  state.connectors[0].config.credentialRef = "vault://secret/gbo/db";
  C.save(state);
  var c = ctx();
  equal(L.runCheck(c).pass, true);
  equal(c.statuses[0].message, "{sheet-connect-check-pass}");
  C.mutate(function (target) { target.connectors[0].config.password = "plaintext"; });
  equal(L.runCheck(c).pass, false);
  equal(c.statuses[1].isError, true);
  assert(c.statuses[1].message.indexOf("{sheet-connect-check-fail}") === 0);
});

test("lineage render draws the snapshot grid and selected-cell panel", function () {
  env.reset();
  var state = feedState();
  F.refreshNow(state);
  C.save(state);
  var container = env.document.createElement("div");
  L.render(container, ctx());
  assert(container.querySelector(".ss-connect-table"));
  container.querySelector(".ss-connect-table").querySelectorAll("td")[0].dispatch("click");
  equal(C.state().selectedCell.row, 0);
  equal(C.state().selectedCell.col, 0);
  container.textContent = "";
  L.render(container, ctx());
  assert(container.querySelector(".ss-connect-lineage-panel"));
});

// -------------------------------------------------------------------- boot
test("boot facade opens, switches and closes the shared dialog", function () {
  B.switchTab("connectors");
  var root = B.open();
  equal(root.getAttribute("role"), "dialog");
  equal(root.getAttribute("aria-modal"), "true");
  var status = root.querySelector(".ss-connect-status");
  assert(status);
  equal(status.getAttribute("role"), "status");
  equal(status.getAttribute("aria-live"), "polite");
  equal(B.tabs.length, 5);

  B.switchTab("lineage");
  equal(B.activeTab(), "lineage");
  var selected = root.querySelector("#ss-connect-tab-lineage");
  assert(selected);
  equal(selected.getAttribute("aria-selected"), "true");
  equal(root.querySelector("#ss-connect-tab-query").getAttribute("aria-selected"), "false");

  B.close();
  equal(root.isConnected, false);
  B.toggle();
  assert(env.document.querySelector(".ss-connect-dialog").isConnected);
  B.close();
});

test("boot relocalize rewrites data-i18n nodes from the catalog stub", function () {
  var root = B.open();
  var node = root.querySelector("[data-i18n]");
  assert(node);
  var key = node.getAttribute("data-i18n");
  W.dispatch("gb-sheet-locale", {});
  equal(node.textContent, "{" + key + "}");
  B.close();
  B.switchTab("connectors");
});

// ------------------------------------------------- STATIC source assertions
test("STATIC: all ten shipped modules carry the #2492 provenance marker", function () {
  h.MODULES.forEach(function (name) {
    assert(h.readSource(name).indexOf("#2492") >= 0, name + " lacks the issue marker");
  });
});

test("STATIC: the preview never calls network, cron or server persistence", function () {
  var forbidden = ["fetch(", "XMLHttpRequest", "setInterval", "setTimeout", "axios", "WebSocket", "new Worker"];
  h.MODULES.forEach(function (name) {
    var source = h.readSource(name);
    forbidden.forEach(function (token) {
      assert(source.indexOf(token) < 0, name + " contains forbidden host call " + token);
    });
  });
});

test("STATIC: credential values are never written into a DOM text slot", function () {
  var connectors = h.readSource("67_connect_connectors.js");
  assert(!/\.textContent\s*=\s*[^\n]*credentialRef/.test(connectors));
  var feed = h.readSource("66a_connect_feed.js");
  assert(/vault:\/\//.test(feed));
  assert(/function sanitizeConfig/.test(feed));
});

test("STATIC: status regions are announced and DOM literals use i18n keys", function () {
  ["66_connect_core.js", "70_connect_refresh.js"].forEach(function (name) {
    var source = h.readSource(name);
    assert(source.indexOf('setAttribute("role", "status")') >= 0, name);
    assert(source.indexOf('setAttribute("aria-live", "polite")') >= 0, name);
  });
  h.MODULES.forEach(function (name) {
    var source = h.readSource(name);
    assert(!/\.textContent\s*=\s*["'][A-Za-z][A-Za-z ]{6,}["']/.test(source), name + " has a direct English DOM write");
  });
});
