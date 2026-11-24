/**
 * [UAT:DB-01] "Show me the tables in the database and the row counts"
 * Steps exercised: (1) the schema list shows each table with its row count;
 * (2) selecting a table opens its first rows in the grid;
 * (3) sorting a column reloads it in the requested order and draws the arrow.
 * Drives the real database modules (03_actions loadSchema/loadTableData,
 * 02_render renderTableList/renderGrid, 04_actions selectTable/sortBy).
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

  var rows = [
    { id: 3, email: "carol@example.com" },
    { id: 1, email: "alice@example.com" },
    { id: 2, email: "bob@example.com" },
  ];
  function payloadFor(url) {
    if (/\/schema$/.test(url)) {
      return {
        tables: [
          {
            name: "customers",
            column_count: 2,
            row_count: 120,
            columns: [
              { name: "id", data_type: "UUID", is_pk: true },
              { name: "email", data_type: "VARCHAR(255)" },
            ],
          },
          { name: "orders", column_count: 3, row_count: 42, columns: [] },
        ],
      };
    }
    if (/\/table\/customers\/data/.test(url)) {
      var sorted = rows.slice();
      var match = url.match(/[?&]sort=([^&]+)/);
      var desc = /[?&]sort_order=desc/.test(url);
      if (match) {
        sorted.sort(function (a, b) {
          return a.email.localeCompare(b.email) * (desc ? -1 : 1);
        });
      }
      return { rows: sorted, total: sorted.length, pk_column: "id" };
    }
    return {};
  }
  function fetchStub(url, opts) {
    calls.push({ url: url, opts: opts || {} });
    var payload = payloadFor(url);
    return Promise.resolve({
      ok: true,
      json: function () { return Promise.resolve(payload); },
    });
  }

  var sandbox = env.baseSandbox(doc, env.makeStore(), { fetch: fetchStub });
  env.loadAll(sandbox, DB_MODULES);
  return { doc: doc, sandbox: sandbox, calls: calls };
}

h.test("[UAT:DB-01] list tables with counts, open rows and sort a column", function () {
  var ctx = makeDatabase();
  h.assert(typeof ctx.sandbox.DBApp.selectTable === "function", "DBApp not loaded");

  return ctx.sandbox.DBApp.refreshSchema().then(function () {
    // Step 1 — both tables are listed with their real row counts.
    var list = ctx.doc.getElementById("db-table-list").innerHTML;
    h.assert(list.indexOf("customers") !== -1, "customers missing from table list: " + list);
    h.assert(list.indexOf("120 rows") !== -1, "customer row count missing: " + list);
    h.assert(list.indexOf("42 rows") !== -1, "order row count missing: " + list);

    // Step 2 — selecting a table fetches its data and renders the first rows.
    ctx.sandbox.DBApp.selectTable("customers");
    return flush().then(function () {
      h.assert(
        ctx.doc.getElementById("db-current-table").textContent === "customers",
        "selected table not shown in the header",
      );
      var grid = ctx.doc.getElementById("db-grid-container").innerHTML;
      h.assert(grid.indexOf("alice@example.com") !== -1, "first rows not rendered: " + grid);
      h.assert(
        ctx.doc.getElementById("db-row-count").textContent === "3 rows",
        "row count not updated after opening the table",
      );

      // Step 3 — the first sort orders ascending, the second descending.
      ctx.sandbox.DBApp.sortBy("email");
      return flush();
    }).then(function () {
      var asc = ctx.calls.filter(function (c) { return /\/table\/customers\/data/.test(c.url); }).pop();
      h.assert(/sort=email/.test(asc.url), "sort column not sent: " + asc.url);
      h.assert(/sort_order=asc/.test(asc.url), "ascending order not requested: " + asc.url);
      var ascGrid = ctx.doc.getElementById("db-grid-container").innerHTML;
      h.assert(ascGrid.indexOf("alice@example.com") < ascGrid.indexOf("bob@example.com"), "ascending rows not reordered");
      h.assert(ascGrid.indexOf("&#9650;") !== -1, "ascending sort arrow missing");

      ctx.sandbox.DBApp.sortBy("email");
      return flush();
    }).then(function () {
      var desc = ctx.calls.filter(function (c) { return /\/table\/customers\/data/.test(c.url); }).pop();
      h.assert(/sort_order=desc/.test(desc.url), "descending order not requested: " + desc.url);
      var descGrid = ctx.doc.getElementById("db-grid-container").innerHTML;
      h.assert(descGrid.indexOf("carol@example.com") < descGrid.indexOf("alice@example.com"), "descending rows not reordered");
      h.assert(descGrid.indexOf("&#9660;") !== -1, "descending sort arrow missing");
    });
  });
});
