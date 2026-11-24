/**
 * [UAT:DB-03] "Quais foram os 10 produtos mais vendidos no último mês?"
 * Steps exercised: (1) running the query renders a product/quantity result table;
 * (2) re-opening the SQL console shows the exact query that produced it.
 * Drives the real database modules (04_actions executeQuery/showQueryBuilder,
 * 02_render sanitise-based table rendering) with a fetch/DOM stub.
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

function flush() {
  return new Promise(function (resolve) { setTimeout(resolve, 0); });
}

function makeDatabase() {
  var calls = [];
  var doc = h.makeDocument({ readyState: "complete" });
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

  var SQL = "SELECT product, quantity FROM sales ORDER BY quantity DESC LIMIT 10";
  function fetchStub(url, opts) {
    calls.push({ url: url, opts: opts || {} });
    var payload = /\/query$/.test(url)
      ? {
          columns: ["product", "quantity"],
          rows: [
            { product: "Notebook", quantity: 128 },
            { product: "Monitor", quantity: 94 },
          ],
        }
      : {};
    return Promise.resolve({
      ok: true,
      json: function () { return Promise.resolve(payload); },
    });
  }

  var sandbox = env.baseSandbox(doc, env.makeStore(), { fetch: fetchStub });
  env.loadAll(sandbox, DB_MODULES);
  return { doc: doc, sandbox: sandbox, calls: calls, sql: SQL };
}

h.test("[UAT:DB-03] run the sales query and show the SQL that produced it", function () {
  var ctx = makeDatabase();
  h.assert(typeof ctx.sandbox.DBApp.executeQuery === "function", "DBApp.executeQuery not loaded");

  ctx.doc.getElementById("db-sql-input").value = ctx.sql;
  return ctx.sandbox.DBApp.executeQuery().then(function () {
    // Step 1 — the answer is a table with product and quantity, not prose.
    var queryCall = ctx.calls.filter(function (c) { return /\/query$/.test(c.url); })[0];
    h.assert(!!queryCall, "no query request was issued");
    h.assert(JSON.parse(queryCall.opts.body).query === ctx.sql, "SQL was not sent verbatim");

    var result = ctx.doc.getElementById("db-sql-result").innerHTML;
    h.assert(result.indexOf("Query returned 2 row(s).") !== -1, "result summary missing: " + result);
    h.assert(result.indexOf("<th>product</th>") !== -1, "product column missing: " + result);
    h.assert(result.indexOf("<th>quantity</th>") !== -1, "quantity column missing: " + result);
    h.assert(result.indexOf("Notebook") !== -1 && result.indexOf("128") !== -1, "top product row missing: " + result);
    h.assert(result.indexOf("Monitor") !== -1 && result.indexOf("94") !== -1, "second product row missing: " + result);

    return flush();
  }).then(function () {
    // Step 2 — the SQL console reopens with the same query visible.
    ctx.sandbox.DBApp.showQueryBuilder();
    h.assert(
      ctx.doc.getElementById("db-sql-console").style.display === "flex",
      "SQL console did not open on request",
    );
    h.assert(
      ctx.doc.getElementById("db-sql-input").value === ctx.sql,
      "the query used is not shown when requested",
    );
  });
});
