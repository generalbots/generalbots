/* =============================================================================
   GOALS/OKR MODULE - Objectives & Key Results
   ============================================================================= */

(function () {
  "use strict";

  // =============================================================================
  // STATE
  // =============================================================================

  const state = {
    currentView: "dashboard",
    objectives: [],
    selectedObjective: null,
    filters: {
      status: "all",
      owner: "all",
      period: "current",
    },
  };

  // =============================================================================
  // INITIALIZATION
  // =============================================================================

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
    document.querySelectorAll(".view-btn").forEach((btn) => {
      btn.classList.toggle("active", btn.dataset.view === view);
    });

    // Update view panels
    document.querySelectorAll(".goals-view").forEach((panel) => {
      panel.classList.toggle("active", panel.id === `${view}-view`);
    });

    // Load view-specific data if using HTMX
    if (typeof htmx !== "undefined") {
      const viewContainer = document.getElementById("goals-content");
      if (viewContainer) {
        htmx.ajax("GET", `/api/ui/goals/${view}`, { target: viewContainer });
      }
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

  // =============================================================================
  // OBJECTIVES
  // =============================================================================

  async function loadObjectives() {
    try {
      const response = await fetch("/api/goals/objectives");
      if (response.ok) {
        const data = await response.json();
        state.objectives = data.objectives || [];
        renderObjectives();
      }
    } catch (e) {
      console.error("Failed to load objectives:", e);
    }
  }

  function renderObjectives() {
    const container = document.getElementById("objectives-list");
    if (!container) return;

    if (state.objectives.length === 0) {
      container.innerHTML = `
        <div class="empty-state">
          <span class="empty-icon">🎯</span>
          <h3>No objectives yet</h3>
          <p>Create your first objective to start tracking goals</p>
          <button class="btn-primary" onclick="showCreateObjectiveModal()">
            Create Objective
          </button>
        </div>
      `;
      return;
    }

    container.innerHTML = state.objectives
      .map(
        (obj) => `
      <div class="objective-card ${state.selectedObjective?.id === obj.id ? "selected" : ""}"
           data-id="${obj.id}">
        <div class="objective-header">
          <h3>${escapeHtml(obj.title)}</h3>
          <span class="status-badge ${obj.status}">${obj.status}</span>
        </div>
        <div class="objective-progress">
          <div class="progress-bar">
            <div class="progress-fill" style="width: ${obj.progress || 0}%"></div>
          </div>
          <span class="progress-text">${obj.progress || 0}%</span>
        </div>
        <div class="objective-meta">
          <span class="owner">${escapeHtml(obj.owner_name || "Unassigned")}</span>
          <span class="due-date">${formatDate(obj.end_date)}</span>
        </div>
      </div>
    `,
      )
      .join("");
  }

  function selectObjective(objectiveId) {
    const objective = state.objectives.find((o) => o.id === objectiveId);
    if (!objective) return;

    state.selectedObjective = objective;
    renderObjectives();
    renderObjectiveDetails(objective);
    openGoalsPanel();
  }

  function renderObjectiveDetails(objective) {
    const container = document.getElementById("objective-details");
    if (!container) return;

    container.innerHTML = `
      <div class="detail-header">
        <h2>${escapeHtml(objective.title)}</h2>
        <div class="detail-actions">
          <button class="btn-icon" onclick="editObjective('${objective.id}')" title="Edit">✏️</button>
          <button class="btn-icon" onclick="deleteObjective('${objective.id}')" title="Delete">🗑️</button>
        </div>
      </div>
      <div class="detail-body">
        <div class="detail-section">
          <label>Status</label>
          <span class="status-badge ${objective.status}">${objective.status}</span>
        </div>
        <div class="detail-section">
          <label>Progress</label>
          <div class="progress-bar large">
            <div class="progress-fill" style="width: ${objective.progress || 0}%"></div>
          </div>
          <span>${objective.progress || 0}%</span>
        </div>
        <div class="detail-section">
          <label>Description</label>
          <p>${escapeHtml(objective.description || "No description")}</p>
        </div>
        <div class="detail-section">
          <label>Owner</label>
          <p>${escapeHtml(objective.owner_name || "Unassigned")}</p>
        </div>
        <div class="detail-section">
          <label>Timeline</label>
          <p>${formatDate(objective.start_date)} - ${formatDate(objective.end_date)}</p>
        </div>
        <div class="detail-section">
          <label>Key Results</label>
          <div id="key-results-list">
            ${renderKeyResults(objective.key_results || [])}
          </div>
          <button class="btn-secondary btn-sm" onclick="addKeyResult('${objective.id}')">
            + Add Key Result
          </button>
        </div>
      </div>
    `;
  }

  function renderKeyResults(keyResults) {
    if (keyResults.length === 0) {
      return '<p class="empty-message">No key results defined</p>';
    }

    return keyResults
      .map(
        (kr) => `
      <div class="key-result-item">
        <div class="kr-header">
          <span class="kr-title">${escapeHtml(kr.title)}</span>
          <span class="kr-progress">${kr.current_value || 0} / ${kr.target_value || 100}</span>
        </div>
        <div class="progress-bar small">
          <div class="progress-fill" style="width: ${((kr.current_value || 0) / (kr.target_value || 100)) * 100}%"></div>
        </div>
      </div>
    `,
      )
      .join("");
  }

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

  // =============================================================================
  // UTILITIES
  // =============================================================================

  function escapeHtml(text) {
    if (!text) return "";
    const div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
  }

  function formatDate(dateString) {
    if (!dateString) return "Not set";
    try {
      const date = new Date(dateString);
      return date.toLocaleDateString();
    } catch {
      return dateString;
    }
  }

  function showNotification(message, type) {
    if (typeof window.showNotification === "function") {
      window.showNotification(message, type);
    } else if (typeof window.GBAlerts !== "undefined") {
      if (type === "success") window.GBAlerts.success("Goals", message);
      else if (type === "error") window.GBAlerts.error("Goals", message);
      else window.GBAlerts.info("Goals", message);
    } else {
      console.log(`[${type}] ${message}`);
    }
  }

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
  window.createObjective = createObjective;
  window.editObjective = editObjective;
  window.updateObjective = updateObjective;
  window.deleteObjective = deleteObjective;
  window.addKeyResult = addKeyResult;

  // =============================================================================
  // INITIALIZE
  // =============================================================================

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
