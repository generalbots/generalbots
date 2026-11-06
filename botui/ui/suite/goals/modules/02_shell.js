/* Goals app — module 2/6: init, events, view switching and details panel (split from goals/goals.js for #1731) */

(function (app) {
  "use strict";

  // =============================================================================
  // INITIALIZATION
  // =============================================================================

  const { state } = app;

  // Linked lazily — provided by later modules, resolved on first call.
  const loadObjectives = (...args) => app.loadObjectives(...args);
  const selectObjective = (...args) => app.selectObjective(...args);
  const loadGoalRuns = (...args) => app.loadGoalRuns(...args);
  const startRunsPolling = (...args) => app.startRunsPolling(...args);
  const stopRunsPolling = (...args) => app.stopRunsPolling(...args);

  function init() {
    loadObjectives();
    bindEvents();
    console.log("Goals module initialized");
  }

  function bindEvents() {
    // View toggle buttons
    document.querySelectorAll(".view-btn").forEach((btn) => {
      btn.addEventListener("click", function () {
        const view = this.dataset.view;
        if (view) {
          switchGoalsView(view);
        }
      });
    });

    // Objective cards
    document.addEventListener("click", (e) => {
      const card = e.target.closest(".objective-card");
      if (card) {
        const objectiveId = card.dataset.id;
        if (objectiveId) {
          selectObjective(objectiveId);
        }
      }
    });
  }

  // =============================================================================
  // VIEW SWITCHING
  // =============================================================================

  function switchGoalsView(view) {
    state.currentView = view;

    // Update button states
    document.querySelectorAll(".goals-app .view-btn").forEach((btn) => {
      btn.classList.toggle("active", btn.dataset.view === view);
    });

    // Update view panels (the fragment's containers are .view-container,
    // not .goals-view — the old selector left every view hidden).
    document.querySelectorAll(".goals-app .view-container").forEach((panel) => {
      panel.classList.toggle("active", panel.id === `${view}-view`);
    });

    // Load view-specific data if using HTMX
    if (typeof htmx !== "undefined") {
      const viewContainer = document.getElementById("goals-content");
      if (viewContainer) {
        htmx.ajax("GET", `/api/ui/goals/${view}`, { target: viewContainer });
      }
    }

    // Goal runs poll while the view is visible so checkpoints advance on
    // screen; every other view stops the timer.
    if (view === "runs") {
      loadGoalRuns();
      startRunsPolling();
    } else {
      stopRunsPolling();
    }
  }

  // =============================================================================
  // DETAILS PANEL
  // =============================================================================

  function toggleGoalsPanel() {
    const panel = document.getElementById("details-panel");
    if (panel) {
      panel.classList.toggle("collapsed");
    }
  }

  function openGoalsPanel() {
    const panel = document.getElementById("details-panel");
    if (panel) {
      panel.classList.remove("collapsed");
    }
  }

  function closeGoalsPanel() {
    const panel = document.getElementById("details-panel");
    if (panel) {
      panel.classList.add("collapsed");
    }
  }

  Object.assign(app, {
    init,
    bindEvents,
    switchGoalsView,
    toggleGoalsPanel,
    openGoalsPanel,
    closeGoalsPanel,
  });
})(window.__gbGoals = window.__gbGoals || {});
