/* Goals app — module 5/6: objective/key-result CRUD (split from goals/goals.js for #1731) */

(function (app) {
  "use strict";

  const { state, showNotification, loadObjectives, closeGoalsPanel } = app;

  // =============================================================================
  // CRUD OPERATIONS
  // =============================================================================

  // #2032 — dialogs go through the suite modal when it is present; the native
  // prompt/confirm looked broken inside the shell and cannot be themed.
  // GBModal.* answers through callbacks in the shell and returns the value in
  // the native fallback, so both shapes are bridged into one promise.
  function gbConfirm(message) {
    return new Promise(function (resolve) {
      if (window.GBModal && typeof window.GBModal.confirm === "function") {
        var settled = false;
        var result = window.GBModal.confirm(
          message,
          "Confirm",
          function () { settled = true; resolve(true); },
          function () { settled = true; resolve(false); },
        );
        if (!settled && typeof result === "boolean") { resolve(result); }
        return;
      }
      resolve(window.confirm(message));
    });
  }

  function gbAsk(message, fallback) {
    return new Promise(function (resolve) {
      if (window.GBModal && typeof window.GBModal.prompt === "function") {
        var settled = false;
        var result = window.GBModal.prompt(
          message,
          fallback,
          "Input",
          function (value) { settled = true; resolve(value == null ? null : String(value)); },
        );
        if (!settled && result !== undefined) {
          resolve(result == null ? null : String(result));
        }
        return;
      }
      resolve(window.prompt(message, fallback));
    });
  }

  function showCreateObjectiveModal() {
    const modal = document.getElementById("create-objective-modal");
    if (modal) {
      if (modal.showModal) {
        modal.showModal();
      } else {
        modal.classList.add("open");
      }
      return;
    }
    // The app ships the HTMX fragment, not a <dialog>: load it into the modal
    // container so the create flow always has a real form (#2032).
    const host = document.getElementById("goals-modal");
    if (host && window.htmx && typeof window.htmx.ajax === "function") {
      window.htmx.ajax("GET", "/api/ui/goals/new-objective", {
        target: "#goals-modal",
        swap: "innerHTML",
      });
      return;
    }
    gbAsk("Enter objective title:").then(function (title) {
      if (title) { createObjective({ title, period: currentPeriodValue() }); }
    });
  }

  function currentPeriodValue() {
    const select = document.getElementById("period-select");
    return (select && select.value) || "annual-" + new Date().getFullYear();
  }

  function closeCreateObjectiveModal() {
    const modal = document.getElementById("create-objective-modal");
    if (modal) {
      if (modal.close) {
        modal.close();
      } else {
        modal.classList.remove("open");
      }
    }
    // The HTMX fragment is swapped into #goals-modal, not into the dialog.
    const host = document.getElementById("goals-modal");
    if (host) {
      host.innerHTML = "";
      host.classList.remove("open");
    }
  }

  async function createObjective(data) {
    try {
      const response = await fetch("/api/goals/objectives", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });
      if (response.ok) {
        showNotification("Objective created", "success");
        loadObjectives();
        closeCreateObjectiveModal();
      } else {
        showNotification("Failed to create objective", "error");
      }
    } catch (e) {
      console.error("Failed to create objective:", e);
      showNotification("Failed to create objective", "error");
    }
  }

  async function editObjective(objectiveId) {
    const objective = state.objectives.find((o) => o.id === objectiveId);
    if (!objective) return;

    const newTitle = await gbAsk("Edit objective title:", objective.title);
    if (newTitle && newTitle !== objective.title) {
      updateObjective(objectiveId, { title: newTitle });
    }
  }

  async function updateObjective(objectiveId, data) {
    try {
      const response = await fetch(`/api/goals/objectives/${objectiveId}`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });
      if (response.ok) {
        showNotification("Objective updated", "success");
        loadObjectives();
      } else {
        showNotification("Failed to update objective", "error");
      }
    } catch (e) {
      console.error("Failed to update objective:", e);
      showNotification("Failed to update objective", "error");
    }
  }

  async function deleteObjective(objectiveId) {
    if (!(await gbConfirm("Delete this objective? This cannot be undone."))) return;

    try {
      const response = await fetch(`/api/goals/objectives/${objectiveId}`, {
        method: "DELETE",
      });
      if (response.ok) {
        showNotification("Objective deleted", "success");
        state.selectedObjective = null;
        closeGoalsPanel();
        loadObjectives();
      } else {
        showNotification("Failed to delete objective", "error");
      }
    } catch (e) {
      console.error("Failed to delete objective:", e);
      showNotification("Failed to delete objective", "error");
    }
  }

  async function addKeyResult(objectiveId) {
    const title = await gbAsk("Enter key result title:");
    if (!title) return;

    const targetValue = await gbAsk("Enter target value:", "100");
    if (!targetValue) return;

    createKeyResult(objectiveId, {
      title,
      target_value: parseFloat(targetValue) || 100,
      current_value: 0,
    });
  }

  async function createKeyResult(objectiveId, data) {
    try {
      const response = await fetch(
        `/api/goals/objectives/${objectiveId}/key-results`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(data),
        },
      );
      if (response.ok) {
        showNotification("Key result added", "success");
        loadObjectives();
      } else {
        showNotification("Failed to add key result", "error");
      }
    } catch (e) {
      console.error("Failed to create key result:", e);
      showNotification("Failed to add key result", "error");
    }
  }

  Object.assign(app, {
    showCreateObjectiveModal,
    closeCreateObjectiveModal,
    createObjective,
    editObjective,
    updateObjective,
    deleteObjective,
    addKeyResult,
    createKeyResult,
  });
})(window.__gbGoals = window.__gbGoals || {});
