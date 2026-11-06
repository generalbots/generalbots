/* Goals app — module 5/6: objective/key-result CRUD (split from goals/goals.js for #1731) */

(function (app) {
  "use strict";

  const { state, showNotification, loadObjectives, closeGoalsPanel } = app;

  // =============================================================================
  // CRUD OPERATIONS
  // =============================================================================

  function showCreateObjectiveModal() {
    const modal = document.getElementById("create-objective-modal");
    if (modal) {
      if (modal.showModal) {
        modal.showModal();
      } else {
        modal.classList.add("open");
      }
    } else {
      // Fallback: create a simple prompt-based flow
      const title = prompt("Enter objective title:");
      if (title) {
        createObjective({ title });
      }
    }
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

  function editObjective(objectiveId) {
    const objective = state.objectives.find((o) => o.id === objectiveId);
    if (!objective) return;

    const newTitle = prompt("Edit objective title:", objective.title);
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
    if (!confirm("Delete this objective? This cannot be undone.")) return;

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

  function addKeyResult(objectiveId) {
    const title = prompt("Enter key result title:");
    if (!title) return;

    const targetValue = prompt("Enter target value:", "100");
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
