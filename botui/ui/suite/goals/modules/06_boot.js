/* Goals app — module 6/6: dashboard stats, window API and boot (split from goals/goals.js for #1731) */

(function (app) {
  "use strict";

  const {
    init,
    switchGoalsView,
    toggleGoalsPanel,
    openGoalsPanel,
    closeGoalsPanel,
    selectObjective,
    showCreateObjectiveModal,
    closeCreateObjectiveModal,
    createObjective,
    editObjective,
    updateObjective,
    deleteObjective,
    addKeyResult,
    loadGoalRuns,
    pauseGoalRun,
    resumeGoalRun,
    stopGoalRun,
  } = app;

  // =============================================================================
  // EXPORT TO WINDOW
  // =============================================================================

  window.switchGoalsView = switchGoalsView;
  window.toggleGoalsPanel = toggleGoalsPanel;
  window.openGoalsPanel = openGoalsPanel;
  window.closeGoalsPanel = closeGoalsPanel;
  window.selectObjective = selectObjective;
  window.showCreateObjectiveModal = showCreateObjectiveModal;
  window.closeCreateObjectiveModal = closeCreateObjectiveModal;
  // #2032 — the New Objective fragment calls closeModal() after a
  // successful create and from its Cancel/× buttons; the global was
  // never defined, so those buttons did nothing.
  window.closeModal = closeCreateObjectiveModal;
  window.createObjective = createObjective;
  window.editObjective = editObjective;
  window.updateObjective = updateObjective;
  window.deleteObjective = deleteObjective;
  window.addKeyResult = addKeyResult;
  window.loadGoalRuns = loadGoalRuns;
  window.pauseGoalRun = pauseGoalRun;
  window.resumeGoalRun = resumeGoalRun;
  window.stopGoalRun = stopGoalRun;
  window.loadGoalsStats = loadGoalsStats;

  function loadGoalsStats() {
    fetch("/api/goals/dashboard")
      .then(function (r) { return r.json(); })
      .then(function (data) {
        if (!data) return;
        var totalObj = document.getElementById("total-objectives");
        var totalKrs = document.getElementById("total-krs");
        var avg = document.getElementById("avg-progress");
        var atRisk = document.getElementById("at-risk-count");
        if (totalObj) totalObj.textContent = String(data.total_objectives != null ? data.total_objectives : "--");
        if (totalKrs) totalKrs.textContent = String(data.total_key_results != null ? data.total_key_results : "--");
        if (avg) avg.textContent = (data.average_progress != null ? Math.round(data.average_progress) : "--") + "%";
        if (atRisk) atRisk.textContent = String(data.at_risk_objectives != null ? data.at_risk_objectives : "--");
        if (typeof updateProgressRing === "function" && data.average_progress != null) {
          updateProgressRing(Math.round(data.average_progress));
        }
      })
      .catch(function (err) {
        console.error("goals stats load failed", err);
      });
  }

  // #2032 — the sidebar selector and the header badge used to be frozen at
  // 2025 in the markup, and the create form sent a bare `Q2`, so the two could
  // never match. Both now derive the year from the clock and speak the same
  // `q2-2026` form the API stores.
  function initPeriodSelector() {
    var select = document.getElementById("period-select");
    var now = new Date();
    var year = now.getFullYear();
    var quarter = Math.floor(now.getMonth() / 3) + 1;
    var badge = document.getElementById("current-period");
    if (badge) { badge.textContent = "Q" + quarter + " " + year; }
    if (!select) { return; }
    var periods = [
      ["q1", "Q1"], ["q2", "Q2"], ["q3", "Q3"], ["q4", "Q4"],
      ["h1", "H1 (Half Year)"], ["h2", "H2 (Half Year)"], ["annual", "Annual"],
    ];
    select.innerHTML = periods.map(function (period) {
      var selected = period[0] === "q" + quarter ? " selected" : "";
      return '<option value="' + period[0] + "-" + year + '"' + selected + ">" +
        period[1] + " " + year + "</option>";
    }).join("");
  }
  window.initGoalsPeriod = initPeriodSelector;
  initPeriodSelector();

  document.body.addEventListener("objectiveCreated", function () { loadGoalsStats(); });
  document.body.addEventListener("objectiveUpdated", function () { loadGoalsStats(); });
  document.body.addEventListener("objectiveDeleted", function () { loadGoalsStats(); });

  // =============================================================================
  // INITIALIZE
  // =============================================================================

  if (document.readyState === "loading") {
    (function(){ var __cb = init; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
  } else {
    init();
  }
  loadGoalsStats();
})(window.__gbGoals = window.__gbGoals || {});
