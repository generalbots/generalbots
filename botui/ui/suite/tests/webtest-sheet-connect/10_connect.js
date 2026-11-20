/**
 * Sheets connectivity — RUNTIME cases against the shipped pure helpers (#2492).
 * Core state, deterministic feed, parameter substitution and the query-step
 * chain (the four invariants named in the issue) are exercised directly.
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
var S = W.SheetConnectSteps;

function connector(id, type) {
  return {
    id: id, name: "Feed " + id, type: type || "csv",
    config: { url: "https://data.example.com/feed" }, createdAt: "2026-01-01T00:00:00.000Z",
  };
}

function targetWithConnector() {
  var target = C.emptyState();
  target.connectors = [connector("c1")];
  target.activeConnectorId = "c1";
  return target;
}

// ---------------------------------------------------------------- core: data
test("core exports the three shipped vocabularies", function () {
  deep(C.CONNECTOR_TYPES, ["csv", "rest", "postgres", "mysql", "drive", "gsheet"]);
  deep(C.STEP_TYPES, ["select", "filter", "rename", "type", "unpivot", "group"]);
  deep(C.FREQUENCIES, ["daily", "weekly", "monthly"]);
});

test("core emptyState is local-only and deterministic", function () {
  var a = C.emptyState();
  var b = C.emptyState();
  deep(a, b);
  equal(a.version, 1);
  equal(a.localOnly, true);
  equal(a.allowlist[0], "data.example.com");
  equal(a.schedule.enabled, true);
  equal(a.schedule.frequency, "daily");
  equal(a.schedule.hour, 6);
  equal(a.schedule.timezone, "UTC");
  equal(a.schedule.lastStatus, "never");
  equal(a.schedule.lastRefreshAt, null);
  equal(a.snapshot, null);
  equal(a.offline, false);
  equal(a.simulateFailure, false);
});

test("core defaultParameters ship the three documented bindings", function () {
  var p = C.defaultParameters();
  equal(p.length, 3);
  deep(p.map(function (item) { return item.key; }), ["dateFrom", "dateTo", "region"]);
  equal(p[0].defaultValue, "2026-03-01");
  equal(p[1].defaultValue, "2026-03-31");
  equal(p[2].defaultValue, "");
  deep(p, C.defaultParameters());
});

test("core oneOf and text clamp values", function () {
  equal(C.oneOf("daily", ["daily", "weekly"], "monthly"), "daily");
  equal(C.oneOf("bad", ["daily", "weekly"], "monthly"), "monthly");
  equal(C.text(null), "");
  equal(C.text("abcdef", 3), "abc");
  equal(C.text(12345), "12345");
});

// ------------------------------------------------------------ core: normalize
test("core normalize of null equals emptyState", function () {
  deep(C.normalize(null), C.emptyState());
  deep(C.normalize("bad"), C.emptyState());
});

test("core normalize clamps arrays and lowercases the allowlist", function () {
  var raw = C.emptyState();
  raw.connectors = new Array(50).fill(0).map(function (_, i) { return { id: "c" + i, type: "rest" }; });
  raw.connectors.push(null);
  raw.steps = new Array(70).fill(0).map(function (_, i) { return { id: "s" + i, type: "filter" }; });
  raw.allowlist = ["DATA.EXAMPLE.COM", "", "OTHER.example.org"];
  var normalized = C.normalize(raw);
  equal(normalized.connectors.length, 40);
  equal(normalized.steps.length, 60);
  deep(normalized.allowlist, ["data.example.com", "other.example.org"]);
  equal(normalized.connectors[0].type, "rest");
  equal(normalized.steps[0].enabled, true);
});

test("core normalize clamps schedule fields", function () {
  var raw = C.emptyState();
  raw.schedule = {
    frequency: "yearly", hour: 99, timezone: "", enabled: true,
    notifyOnFailure: false, lastRefreshAt: "t1", nextRefreshAt: "t2",
    lastStatus: "unknown", lastError: "boom",
  };
  var s = C.normalize(raw).schedule;
  equal(s.frequency, "daily");
  equal(s.hour, 23);
  equal(s.timezone, "UTC");
  equal(s.notifyOnFailure, false);
  equal(s.lastStatus, "never");
  equal(s.lastRefreshAt, "t1");
  equal(s.lastError, "boom");
  raw.schedule.hour = -4;
  equal(C.normalize(raw).schedule.hour, 0);
});

test("core normalize filters unnamed parameters and restores defaults", function () {
  var raw = C.emptyState();
  raw.parameters = [{ key: "" }, { key: "region", kind: "date", value: "x" }];
  var normalized = C.normalize(raw);
  equal(normalized.parameters.length, 1);
  equal(normalized.parameters[0].key, "region");
  equal(normalized.parameters[0].kind, "date");
  raw.parameters = [{ key: "" }];
  deep(C.normalize(raw).parameters, C.defaultParameters());
});

test("core sheetId falls back and honours injected sheet context", function () {
  env.reset();
  equal(C.sheetId(), "current");
  equal(C.storageKey(), "gb-sheet-connect:current");
  W.SheetCore = { currentSheetId: function () { return "sheet-9"; } };
  equal(C.sheetId(), "sheet-9");
  delete W.SheetCore;
  W.__LOADED_SHEET = { id: "drive-3" };
  equal(C.sheetId(), "drive-3");
  delete W.__LOADED_SHEET;
  equal(C.sheetId(), "current");
});

test("core state/save/mutate round-trip through storage", function () {
  env.reset();
  deep(C.state(), C.emptyState());
  var next = C.emptyState();
  next.activeConnectorId = "c7";
  equal(C.save(next), true);
  equal(C.state().activeConnectorId, "c7");
  var returned = C.mutate(function (state) { state.audit.push({ id: "a1" }); return state.audit.length; });
  equal(returned, 1);
  equal(C.state().audit.length, 1);
  equal(env.store.keys().length, 1);
});

test("core state survives corrupt storage by returning emptyState", function () {
  env.reset();
  env.store.setItem(C.storageKey(), "{not json");
  deep(C.state(), C.emptyState());
});

// ---------------------------------------------------------------- feed: hosts
test("feed hostOf strips scheme, credentials, port and case", function () {
  equal(F.hostOf("https://data.example.com/feed"), "data.example.com");
  equal(F.hostOf("HTTPS://user:pw@Data.Example.COM:8443/a/b"), "data.example.com");
  equal(F.hostOf("data.example.com:9000/path"), "data.example.com");
  equal(F.hostOf(""), "");
  equal(F.hostOf(null), "");
});

test("feed simulatedFeed keeps rows, columns and lineage parallel", function () {
  var table = F.simulatedFeed(connector("c1"));
  deep(table.columns, ["date", "region", "amount", "category"]);
  equal(table.rows.length, 6);
  equal(table.lineage.length, table.rows.length);
  table.rows.forEach(function (row, r) {
    equal(row.length, table.columns.length);
    equal(table.lineage[r].length, table.columns.length);
    table.lineage[r].forEach(function (cell, c) {
      equal(cell.stepId, "source");
      equal(cell.stepType, "source");
      equal(cell.sourceField, table.columns[c]);
    });
  });
  equal(table.sourceLabel, "Feed c1");
  equal(F.simulatedFeed(null).sourceLabel, "local");
});

test("feed activeConnector resolves the workbook selection", function () {
  var state = C.emptyState();
  equal(F.activeConnector(state), null);
  state.connectors = [connector("c1"), connector("c2")];
  state.activeConnectorId = "c2";
  equal(F.activeConnector(state).id, "c2");
  state.activeConnectorId = "missing";
  equal(F.activeConnector(state), null);
});

// ------------------------------------------------------- feed: param/definition
test("feed parameter substitution replaces known tokens only", function () {
  var params = [
    { key: "region", value: "south" },
    { key: "blank", value: "" },
  ];
  equal(F.paramValue(params, "region"), "south");
  equal(F.paramValue(params, "missing"), "");
  equal(F.applyParameters("region={param:region}", params), "region=south");
  equal(F.applyParameters("x={param:blank}&y={param:missing}", params), "x={param:blank}&y={param:missing}");
  equal(F.applyParameters(null, params), "");
});

test("feed sanitizeConfig keeps vault refs and drops literal secrets", function () {
  var clean = F.sanitizeConfig({
    url: "https://data.example.com/feed",
    credentialRef: "vault://secret/gbo/db",
    password: "literal-password",
    api_key: "vault://secret/gbo/api",
    long: new Array(600).fill("x").join(""),
  });
  equal(clean.url, "https://data.example.com/feed");
  equal(clean.credentialRef, "vault://secret/gbo/db");
  equal(clean.api_key, "vault://secret/gbo/api");
  equal("password" in clean, false);
  equal("long" in clean, false);
});

test("feed definition and mockExport describe a local-only preview", function () {
  var state = targetWithConnector();
  var definition = F.definition(state);
  equal(definition.version, 1);
  equal(definition.kind, "sheet-connect-local-preview");
  equal(definition.localOnly, true);
  equal(definition.connector.id, "c1");
  equal(definition.schedule.frequency, "daily");
  deep(definition.parameters.map(function (p) { return p.key; }), ["dateFrom", "dateTo", "region"]);
  equal(F.mockExport(state).definition.localOnly, true);
  equal(F.mockExport(state).snapshot, null);
  equal(F.mockExport(state).audit.length, 0);
});

test("feed noCredentialScan passes vault refs and fails literal values", function () {
  var safe = targetWithConnector();
  safe.connectors[0].config.credentialRef = "vault://secret/gbo/db";
  equal(F.noCredentialScan(safe).pass, true);
  var unsafe = targetWithConnector();
  unsafe.connectors[0].config.password = "plaintext-secret";
  var result = F.noCredentialScan(unsafe);
  equal(result.pass, false);
  assert(result.findings.length > 0);
  assert(/^workbook:/.test(result.findings[0]));
});

// -------------------------------------------------------- feed: refresh timing
test("feed nextRefresh applies frequency and hour deterministically", function () {
  var daily = { hour: 6, frequency: "daily" };
  equal(F.nextRefresh("2026-03-10T10:30:00.000Z", daily), "2026-03-11T06:00:00.000Z");
  equal(F.nextRefresh("2026-03-10T02:00:00.000Z", daily), "2026-03-10T06:00:00.000Z");
  equal(F.nextRefresh("2026-03-10T10:00:00.000Z", { hour: 6, frequency: "weekly" }), "2026-03-17T06:00:00.000Z");
  equal(F.nextRefresh("2026-03-10T10:00:00.000Z", { hour: 6, frequency: "monthly" }), "2026-04-10T06:00:00.000Z");
  equal(F.nextRefresh("bad", daily), null);
});

test("feed stamp is BASE_TIME plus one day per audit entry", function () {
  var state = C.emptyState();
  equal(F.stamp(state), "2026-03-01T06:00:00.000Z");
  state.audit = [{}, {}];
  equal(F.stamp(state), "2026-03-03T06:00:00.000Z");
});

test("feed refreshNow without a connector reports the shipped error key", function () {
  var result = F.refreshNow(C.emptyState());
  equal(result.ok, false);
  equal(result.errorKey, "sheet-connect-refresh-no-connector");
  assert(result.errorFallback.length > 0);
});

test("feed refreshNow success builds a snapshot and audit record", function () {
  var state = targetWithConnector();
  var result = F.refreshNow(state);
  equal(result.ok, true);
  equal(result.rows, 6);
  equal(result.time, "2026-03-01T06:00:00.000Z");
  equal(state.snapshot.rowCount, 6);
  equal(state.snapshot.connectorId, "c1");
  equal(state.snapshot.stepsApplied, 0);
  equal(state.snapshot.stale, false);
  equal(state.snapshot.lineage.length, 6);
  equal(state.schedule.lastStatus, "success");
  equal(state.schedule.lastError, "");
  equal(state.schedule.nextRefreshAt, "2026-03-02T06:00:00.000Z");
  equal(state.audit.length, 1);
  equal(state.audit[0].status, "success");
  equal(state.audit[0].id, "audit-0");
});

test("feed refreshNow failure transitions the schedule to failed", function () {
  var state = targetWithConnector();
  state.simulateFailure = true;
  var result = F.refreshNow(state);
  equal(result.ok, false);
  equal(state.schedule.lastStatus, "failed");
  assert(state.schedule.lastError.length > 0);
  equal(state.schedule.lastRefreshAt, result.time);
  assert(state.schedule.nextRefreshAt);
  equal(state.audit[0].status, "failed");
  equal(state.audit[0].rows, 0);
  equal(state.snapshot, null);
});

test("feed refreshes are deterministic across identical workbooks", function () {
  var a = F.refreshNow(targetWithConnector());
  var b = F.refreshNow(targetWithConnector());
  equal(a.time, b.time);
  deep(a.rows, b.rows);
});

// ----------------------------------------------------------------- steps: pure
test("steps columnsList trims and caps the requested columns", function () {
  deep(S.columnsList(" date , region ,, amount "), ["date", "region", "amount"]);
  deep(S.columnsList(""), []);
  equal(S.columnsList(new Array(40).fill("c").join(",")).length, 30);
});

test("steps compare implements all five shipped operators", function () {
  equal(S.compare("North", "north", "eq"), true);
  equal(S.compare("a", "b", "neq"), true);
  equal(S.compare("supplies", "supp", "contains"), true);
  equal(S.compare(120, 100, "gt"), true);
  equal(S.compare(5, 10, "lt"), true);
  equal(S.compare("abc", "abb", "gt"), true);
  equal(S.compare("a", "b", "unknown"), false);
});

test("steps select keeps requested columns and their lineage", function () {
  var table = F.simulatedFeed(null);
  var out = S.applyStep(table, { id: "st", type: "select", config: { columns: "region,amount" } });
  deep(out.columns, ["region", "amount"]);
  deep(out.rows[0], ["north", 120.5]);
  equal(out.lineage[0][0].sourceField, "region");
  equal(S.applyStep(table, { id: "st", type: "select", config: { columns: "missing" } }), table);
});

test("steps filter honours operators and parameter tokens", function () {
  var table = F.simulatedFeed(null);
  var eq = S.applyStep(table, { id: "f1", type: "filter", config: { field: "region", op: "eq", value: "north" } });
  equal(eq.rows.length, 2);
  var gt = S.applyStep(table, { id: "f2", type: "filter", config: { field: "amount", op: "gt", value: "100" } });
  equal(gt.rows.length, 3);
  var bound = S.applyStep(table,
    { id: "f3", type: "filter", config: { field: "region", op: "eq", value: "{param:region}" } },
    [{ key: "region", value: "south" }]);
  equal(bound.rows.length, 2);
  deep(bound.rows[0], ["2026-03-02", "south", 80, "travel"]);
});

test("steps rename and type stamp the lineage with the step id", function () {
  var table = F.simulatedFeed(null);
  var renamed = S.applyStep(table, { id: "rn", type: "rename", config: { from: "amount", to: "total" } });
  deep(renamed.columns, ["date", "region", "total", "category"]);
  equal(renamed.lineage[0][2].stepId, "rn");
  var typed = S.applyStep(table, { id: "ty", type: "type", config: { field: "amount", to: "number" } });
  equal(typed.lineage[0][2].stepId, "ty");
  var manual = { columns: ["amount"], rows: [["12.5"]], lineage: [[{ stepId: "source", sourceField: "amount" }]] };
  var coerced = S.applyStep(manual, { id: "ty2", type: "type", config: { field: "amount", to: "number" } });
  equal(coerced.rows[0][0], 12.5);
});

test("steps unpivot expands columns and records per-cell lineage", function () {
  var table = F.simulatedFeed(null);
  var out = S.applyStep(table, {
    id: "up", type: "unpivot",
    config: { columns: "amount", categoryHeader: "series", valueHeader: "value" },
  });
  deep(out.columns, ["date", "region", "category", "series", "value"]);
  equal(out.rows.length, 6);
  deep(out.rows[0], ["2026-03-01", "north", "supplies", "amount", 120.5]);
  equal(out.lineage[0][3].stepId, "up");
  equal(out.lineage[0][4].sourceField, "amount");
});

test("steps group aggregates sums and counts with parallel lineage", function () {
  var table = F.simulatedFeed(null);
  var sum = S.applyStep(table, { id: "gr", type: "group", config: { by: "region", aggField: "amount", agg: "sum" } });
  deep(sum.columns, ["region", "total"]);
  deep(sum.rows, [["north", 371.25], ["south", 390], ["east", 105.6]]);
  equal(sum.lineage[0][0].stepId, "gr");
  equal(sum.lineage[0][1].stepType, "group");
  var count = S.applyStep(table, { id: "gc", type: "group", config: { by: "region", aggField: "amount", agg: "count" } });
  deep(count.columns, ["region", "count"]);
  deep(count.rows, [["north", 2], ["south", 2], ["east", 2]]);
});

test("steps skip disabled and unknown steps without transforming", function () {
  var table = F.simulatedFeed(null);
  equal(S.applyStep(table, { id: "x", type: "filter", enabled: false, config: {} }), table);
  equal(S.applyStep(table, { id: "x", type: "unknown", config: {} }), table);
});

test("steps applyAll runs the ordered chain and counts applied steps", function () {
  var state = C.emptyState();
  state.steps = [
    { id: "s1", type: "filter", enabled: true, config: { field: "amount", op: "gt", value: "100" } },
    { id: "s2", type: "select", enabled: true, config: { columns: "region,amount" } },
    { id: "s3", type: "group", enabled: true, config: { by: "region", aggField: "amount", agg: "sum" } },
    { id: "s4", type: "select", enabled: false, config: { columns: "region" } },
  ];
  var result = S.applyAll(state, connector("c1"));
  equal(result.applied, 3);
  deep(result.table.columns, ["region", "total"]);
  deep(result.table.rows, [["north", 371.25], ["south", 310]]);
  var again = S.applyAll(state, connector("c1"));
  equal(again.applied, 3);
  deep(again.table, result.table);
});

test("steps preview applies only the first N enabled steps", function () {
  var state = C.emptyState();
  state.steps = [
    { id: "s1", type: "filter", enabled: true, config: { field: "region", op: "eq", value: "north" } },
    { id: "s2", type: "select", enabled: true, config: { columns: "region,amount" } },
  ];
  var one = S.preview(state, connector("c1"), 1);
  equal(one.rows.length, 2);
  deep(one.columns, ["date", "region", "amount", "category"]);
  var both = S.preview(state, connector("c1"), 2);
  deep(both.columns, ["region", "amount"]);
  equal(both.rows.length, 2);
});
