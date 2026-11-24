/**
 * [UAT:GOALS-02] "Create a goal to read 12 books this year"
 * Steps exercised: (1) the goal is created with target 12 and current 0;
 * (2) marking one book done renders 1/12 and the updated percentage.
 */
"use strict";

var env = require("./00_env");
var h = env.h;

var GOALS_MODULES = [
  "goals/modules/01_state_utils.js",
  "goals/modules/02_shell.js",
  "goals/modules/03_objectives.js",
  "goals/modules/04_runs.js",
  "goals/modules/05_crud.js",
  "goals/modules/06_boot.js",
];

function makeGoals() {
  var calls = [];
  var doc = h.makeDocument({ readyState: "complete" });
  function fetchStub(url, opts) {
    calls.push({ url: url, opts: opts || {} });
    var payload = /\/dashboard$/.test(url)
      ? { total_objectives: 0, total_key_results: 0, average_progress: 0, at_risk_objectives: 0 }
      : [];
    return Promise.resolve({
      ok: true,
      json: function () { return Promise.resolve(payload); },
    });
  }
  var sandbox = env.baseSandbox(doc, env.makeStore(), { fetch: fetchStub });
  env.loadAll(sandbox, GOALS_MODULES);
  return { app: sandbox.__gbGoals, calls: calls };
}

h.test("[UAT:GOALS-02] create a 12-book goal and show 1/12 progress", function () {
  var ctx = makeGoals();
  h.assert(typeof ctx.app.createKeyResult === "function", "goals CRUD not loaded");

  // Step 1 — target 12, unit expressed in the title, current 0.
  ctx.app.createKeyResult("obj-books", {
    title: "Read 12 books this year",
    target_value: 12,
    current_value: 0,
  });
  var post = ctx.calls.filter(function (c) {
    return c.opts.method === "POST" && /\/key-results$/.test(c.url);
  })[0];
  h.assert(!!post, "no key-result creation request issued");
  var body = JSON.parse(post.opts.body);
  h.assert(body.target_value === 12, "target is not 12: " + post.opts.body);
  h.assert(body.current_value === 0, "current is not 0: " + post.opts.body);

  // Step 2 — one book done moves the bar to 1/12 (8.33%).
  var html = ctx.app.renderKeyResults([
    { title: "Read 12 books this year", current_value: 1, target_value: 12 },
  ]);
  h.assert(html.indexOf("1 / 12") !== -1, "progress counts missing: " + html);
  h.assert(/width:\s*8\.3/.test(html), "progress bar is not ~8.33%: " + html);
});
