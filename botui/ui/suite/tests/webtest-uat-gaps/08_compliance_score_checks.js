/**
 * [UAT:BIZ-06] "What is our compliance score and what is failing?"
 * Steps exercised: (1) the overall score and failing-check count render from
 * real check data; (2) the Checks tab lists the failing controls by id.
 * Drives the real compliance app (_compliance loadAll/switchTab/renderChecks).
 */
"use strict";

var env = require("./00_env");
var h = env.h;

function flush() {
  return new Promise(function (resolve) { setTimeout(resolve, 0); });
}

var CHECKS = [
  { id: "AC-2", name: "Access control review", framework: "SOC2", status: "failed" },
  { id: "AU-6", name: "Audit log retention", framework: "LGPD", status: "failed" },
  { id: "AC-1", name: "Inventory maintained", framework: "SOC2", status: "passed" },
  { id: "SC-1", name: "Boundary protection", framework: "SOC2", status: "passed" },
];

function makeCompliance() {
  var calls = [];
  var doc = h.makeDocument({ readyState: "complete" });
  // The app wires switchTab through tab clicks, so expose real tab elements.
  var tabs = ["dashboard", "checks"].map(function (name) {
    var el = h.makeElement("tab-" + name);
    el.dataset.tab = name;
    // The real DOM binds `this` to the element in addEventListener callbacks.
    var originalAdd = el.addEventListener;
    el.addEventListener = function (type, fn) {
      originalAdd.call(el, type, fn.bind(el));
    };
    return el;
  });
  var originalQueryAll = doc.querySelectorAll;
  doc.querySelectorAll = function (selector) {
    if (selector === ".compliance-tab") return tabs;
    return originalQueryAll(selector);
  };
  function fetchStub(url, opts) {
    calls.push({ url: url, opts: opts || {} });
    var payload = /\/checks$/.test(url) ? CHECKS : [];
    var text = JSON.stringify(payload);
    return Promise.resolve({
      ok: true,
      text: function () { return Promise.resolve(text); },
      json: function () { return Promise.resolve(payload); },
    });
  }
  var sandbox = env.baseSandbox(doc, env.makeStore(), {
    fetch: fetchStub,
    // compliance.html runs inside the desktop shell, where this global exists.
    escapeHtml: function (value) {
      return String(value == null ? "" : value)
        .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    },
  });
  env.loadAll(sandbox, ["compliance/compliance.js"]);
  return { doc: doc, sandbox: sandbox, calls: calls, tabs: tabs };
}

h.test("[UAT:BIZ-06] show the compliance score and the failing controls", function () {
  var ctx = makeCompliance();
  h.assert(
    ctx.sandbox._compliance && typeof ctx.sandbox._compliance.loadAll === "function",
    "compliance app did not initialise",
  );

  // The app loads its data itself on boot; wait for those fetches to settle.
  return flush().then(function () {
    return flush();
  }).then(function () {
    h.assert(
      ctx.calls.some(function (c) { return /\/api\/compliance\/checks$/.test(c.url); }),
      "checks were never loaded",
    );
    // Step 1 — score and failing count are computed from the returned checks.
    h.assert(
      ctx.doc.getElementById("stat-score").textContent === "50%",
      "score is not 2/4 = 50%: " + ctx.doc.getElementById("stat-score").textContent,
    );
    h.assert(
      String(ctx.doc.getElementById("stat-failed").textContent) === "2",
      "failing count is not 2: " + ctx.doc.getElementById("stat-failed").textContent,
    );
    var score = ctx.doc.getElementById("score-display").innerHTML;
    h.assert(score.indexOf("50%") !== -1, "score circle missing the value: " + score);
    h.assert(/compliance-score-circle yellow/.test(score), "50% score not flagged yellow: " + score);

    // Step 2 — clicking the Checks tab activates it and lists the controls.
    var checksTab = ctx.tabs.filter(function (tab) { return tab.dataset.tab === "checks"; })[0];
    checksTab.click();
    h.assert(
      ctx.doc.getElementById("panel-checks").classList.contains("active"),
      "Checks tab did not become active",
    );
    var body = ctx.doc.getElementById("checks-tbody").innerHTML;
    h.assert(body.indexOf("AC-2") !== -1, "control AC-2 missing from Checks tab: " + body);
    h.assert(body.indexOf("AU-6") !== -1, "control AU-6 missing from Checks tab: " + body);
    h.assert(body.indexOf("failed") !== -1, "failing status missing from Checks tab: " + body);
  });
});
