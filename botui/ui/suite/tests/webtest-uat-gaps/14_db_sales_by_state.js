/**
 * [UAT:DB-08] "Quanto vendemos por estado em 2025?"
 * Steps exercised: (1) running the GROUP BY query renders a table grouped by
 * state; (2) every group is rendered once and the totals add up to the sum of
 * the groups.
 * Step 1's natural-language-to-SQL is LLM-driven; the SQL console is driven
 * here with the real DBApp.executeQuery.
 * Drives the real database modules (04_actions.executeQuery, 01_state.sanitize).
 */
"use strict";

var env = require("./00_env");
var h = env.h;

var DB_MODULES = [
  "database/modules/01_state.js",
  "database/modules/02_render.js",
  "database/modules/03_actions.js",
  "database/modules/04_actions.js",
];

var GROUPS = [
  { state: "SP", total: 120 },
  { state: "RJ", total: 90 },
  { state: "MG", total: 60 },
];

function makeDatabase() {
  var calls = [];
  var doc = h.makeDocument({ readyState: "complete" });
  // A real browser's textContent -> innerHTML behaviour, used by DB.sanitize.
  var originalCreate = doc.createElement;
  doc.createElement = function (tag) {
    var el = originalCreate(tag);
    var text = "";
    Object.defineProperty(el, "textContent", {
      get: function () { return text; },
      set: function (value) {
        text = String(value == null ? "" : value);
        el.innerHTML = text;
      },
      configurable: true,
    });
    return el;
  };
  function fetchStub(url, opts) {
    calls.push({ url: url, opts: opts || {} });
    var payload = /\/query$/.test(url)
      ? { columns: ["state", "total"], rows: GROUPS }
      : {};
    return Promise.resolve({
      ok: true,
      json: function () { return Promise.resolve(payload); },
    });
  }
  var sandbox = env.baseSandbox(doc, env.makeStore(), { fetch: fetchStub });
  env.loadAll(sandbox, DB_MODULES);
  return { doc: doc, sandbox: sandbox, calls: calls };
}

h.test("[UAT:DB-08] group 2025 sales by state and reconcile the totals", function () {
  var ctx = makeDatabase();
  h.assert(typeof ctx.sandbox.DBApp.executeQuery === "function", "DBApp.executeQuery not loaded");

  var sql = "SELECT state, SUM(total) AS total FROM sales WHERE year = 2025 GROUP BY state";
  ctx.doc.getElementById("db-sql-input").value = sql;
  return ctx.sandbox.DBApp.executeQuery().then(function () {
    var queryCall = ctx.calls.filter(function (c) { return /\/query$/.test(c.url); })[0];
    h.assert(!!queryCall, "no query request was issued");
    var body = JSON.parse(queryCall.opts.body);
    h.assert(body.query === sql, "SQL was not sent verbatim: " + body.query);
    h.assert(/GROUP BY state/.test(body.query) && /2025/.test(body.query), "wrong grouping/period");

    var result = ctx.doc.getElementById("db-sql-result").innerHTML;
    h.assert(result.indexOf("Query returned 3 row(s).") !== -1, "not all groups were rendered: " + result);
    h.assert(result.indexOf("<th>state</th>") !== -1, "state column missing: " + result);
    h.assert(result.indexOf("<th>total</th>") !== -1, "total column missing: " + result);
    GROUPS.forEach(function (group) {
      h.assert(
        result.indexOf("<td>" + group.state + "</td>") !== -1 &&
          result.indexOf("<td>" + group.total + "</td>") !== -1,
        "group " + group.state + "/" + group.total + " missing from the result: " + result,
      );
    });

    // Step 2 - the sum of the rendered group totals equals the whole period.
    var totals = result.match(/<td>\d+<\/td>/g) || [];
    var rendered = totals.map(function (cell) {
      return Number(cell.replace(/<\/?td>/g, ""));
    });
    var sum = rendered.reduce(function (a, b) { return a + b; }, 0);
    h.assert(sum === 270, "rendered group totals do not add up to 270: " + sum);
  });
});
