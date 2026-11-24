/**
 * [UAT:GOALS-01] "Defina uma meta de correr 30 km por mês e acompanhe"
 * Steps exercised: (1) a 30 km goal is created with current 0;
 * (2) registering 5 km renders the 5/30 bar at ~16%.
 * Exercises the real goals modules (05_crud.createKeyResult,
 * 03_objectives.renderKeyResults) with a fetch/DOM stub.
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

h.test("[UAT:GOALS-01] create a 30 km goal at zero and show 5/30 progress", function () {
  var ctx = makeGoals();
  h.assert(typeof ctx.app.createKeyResult === "function", "goals CRUD not loaded");

  // Step 1 — the goal is created with target 30 and current 0.
  ctx.app.createKeyResult("obj-run", {
    title: "Run 30 km per month",
    target_value: 30,
    current_value: 0,
  });
  var post = ctx.calls.filter(function (c) {
    return c.opts.method === "POST" && /\/key-results$/.test(c.url);
  })[0];
  h.assert(!!post, "no key-result creation request issued");
  var body = JSON.parse(post.opts.body);
  h.assert(body.target_value === 30, "target is not 30: " + post.opts.body);
  h.assert(body.current_value === 0, "current is not 0: " + post.opts.body);

  // Step 2 — five kilometres produce a 5 / 30 bar at roughly 16%.
  var html = ctx.app.renderKeyResults([
    { title: "Run 30 km per month", current_value: 5, target_value: 30 },
  ]);
  h.assert(html.indexOf("5 / 30") !== -1, "progress counts missing: " + html);
  h.assert(/width:\s*16\./.test(html), "progress bar is not ~16%: " + html);
});
