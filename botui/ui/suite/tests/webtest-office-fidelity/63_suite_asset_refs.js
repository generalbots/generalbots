/**
 * #2934 — deterministic static guard for the Tasks app's local asset graph.
 *
 * Every /suite/... CSS/JS path referenced by any HTML, JS or CSS source file
 * under botui/ui/suite/tasks/ must resolve to a real file in the repository.
 * The reference count is asserted up front (non-vacuity) and every missing
 * path is reported by source file and asset in the failure message.
 *
 * A second case pins the #2934 endpoint fix: both shipped callers use the
 * registered HTMX stats route and no Tasks file keeps the dead JSON URL or
 * the removed stylesheet part.
 *
 * Source assertions only — no server, browser, cargo or i18n change. The
 * driver owns issue closure, deployment and the rendered-count UAT.
 */
"use strict";

var h = require("./00_harness");
var fs = require("fs");
var path = require("path");

var REPO = path.resolve(__dirname, "..", "..", "..", "..", "..");
var TASKS_DIR = "botui/ui/suite/tasks";
var ASSET_RE = /\/suite\/[A-Za-z0-9_.\/-]+\.(?:css|js)/g;

/** All Tasks HTML/JS/CSS source files, in a stable order. */
function tasksSources() {
  return fs.readdirSync(path.join(REPO, TASKS_DIR))
    .filter(function (name) { return /\.(html|js|css)$/.test(name); })
    .sort()
    .map(function (name) { return TASKS_DIR + "/" + name; });
}

/** Every local asset reference paired with the source file that declares it. */
function assetRefs() {
  var refs = [];
  tasksSources().forEach(function (rel) {
    var src = h.readRepo(rel);
    h.assert(src.length > 0, "non-empty Tasks source file: " + rel);
    var match;
    ASSET_RE.lastIndex = 0;
    while ((match = ASSET_RE.exec(src)) !== null) {
      refs.push({ file: rel, asset: match[0] });
    }
  });
  return refs;
}

h.staticTest("#2934 every local /suite/ CSS/JS asset referenced by the Tasks app exists in the repository", function () {
  var sources = tasksSources();
  h.assert(sources.indexOf(TASKS_DIR + "/tasks.html") !== -1,
    "non-vacuity: tasks.html must be part of the scanned Tasks sources");

  var refs = assetRefs();
  h.assert(refs.length >= 20,
    "non-vacuity: expected at least 20 local asset references in the Tasks app, found " + refs.length);

  var checked = {};
  var missing = [];
  refs.forEach(function (ref) {
    var repoPath = "botui/ui" + ref.asset;
    if (checked[repoPath]) return;
    checked[repoPath] = true;
    if (!h.fileExistsRepo(repoPath)) missing.push(ref.file + " -> " + repoPath);
  });
  h.assert(missing.length === 0,
    "missing /suite/ CSS/JS assets referenced by the Tasks app (" +
    Object.keys(checked).length + " distinct paths checked):\n  " + missing.join("\n  "));
});

h.staticTest("#2934 Tasks stats callers use /api/ui/tasks/stats and no dead JSON path or removed stylesheet survives", function () {
  var callers = [
    TASKS_DIR + "/tasks-toasts_b.js",
    TASKS_DIR + "/tasks-scoped-filters.js",
  ];
  callers.forEach(function (rel) {
    var src = h.readRepo(rel);
    h.assert(src.indexOf('fetch("/api/ui/tasks/stats")') !== -1,
      rel + " must fetch the registered stats route /api/ui/tasks/stats");
    h.assert(src.indexOf("/api/tasks/stats/json") === -1,
      rel + " must not keep the dead /api/tasks/stats/json URL");
    h.assert(src.indexOf("response.text()") !== -1,
      rel + " must read the HTMX HTML fragment with response.text()");
    h.assert(src.indexOf("parseTaskStatsFragment(") !== -1,
      rel + " must turn the stats fragment into real numbers");
  });

  var state = h.readRepo(TASKS_DIR + "/tasks-scoped-state.js");
  h.assert(state.indexOf("function parseTaskStatsFragment(html)") !== -1,
    "the shared stats-fragment parser must ship in tasks-scoped-state.js");
  h.assert(state.indexOf('doc.querySelector(".task-stats")') !== -1,
    "the parser must read the .task-stats fragment emitted by handle_ui_tasks_stats");

  tasksSources().forEach(function (rel) {
    var src = h.readRepo(rel);
    h.assert(src.indexOf("/api/tasks/stats/json") === -1,
      rel + " still references the dead /api/tasks/stats/json endpoint");
    h.assert(src.indexOf("tasks-scoped-log-detail-terminal.css") === -1,
      rel + " still references the removed tasks-scoped-log-detail-terminal.css");
  });

  var html = h.readRepo(TASKS_DIR + "/tasks.html");
  h.assert(html.indexOf("tasks-scoped-log-detail.css") !== -1,
    "tasks.html must still load the shipped tasks-scoped-log-detail.css");
});

h.note("FOLLOW-UP #2934: static source checks only — the Tasks asset graph and the stats-endpoint wiring; browser verification of rendered counts belongs to the UAT agent.");
