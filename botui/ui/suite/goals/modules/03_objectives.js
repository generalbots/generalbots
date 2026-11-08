/* Goals app — module 3/6: objectives list and details (split from goals/goals.js for #1731) */

(function (app) {
  "use strict";

  const { state, escapeHtml, formatDate, humanizeText, openGoalsPanel } = app;

  // =============================================================================
  // OBJECTIVES
  // =============================================================================

  async function loadObjectives() {
    try {
      const response = await fetch("/api/goals/objectives");
      if (response.ok) {
        const data = await response.json();
        // #1323 — the API returns a bare array (Json<Vec<Objective>>); accept
        // both the array and the wrapped { objectives: [...] } shape so the
        // list never silently renders empty.
        state.objectives = Array.isArray(data) ? data : (data.objectives || []);
        // The app hides its views until at least one objective exists; the old
        // HTMX:afterSwap hook never fired because the list is JSON-driven.
        const appEl = document.querySelector(".goals-app");
        if (appEl) appEl.classList.toggle("has-objectives", state.objectives.length > 0);
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
          <button class="btn-icon" onclick="editObjective('${objective.id}')" title="Edit" aria-label="Edit objective">
            <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 20h9"></path><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"></path></svg>
          </button>
          <button class="btn-icon" onclick="deleteObjective('${objective.id}')" title="Delete" aria-label="Delete objective">
            <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"></path><path d="M10 11v6"></path><path d="M14 11v6"></path><path d="M9 6V4a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2v2"></path></svg>
          </button>
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
          <p>${escapeHtml(humanizeText(objective.description) || "No description")}</p>
        </div>
        <div class="detail-section">
          <label>Owner</label>
          <p>${escapeHtml(humanizeText(objective.owner_name) || "Unassigned")}</p>
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
          <span class="kr-title">${escapeHtml(humanizeText(kr.title))}</span>
          <span class="kr-progress">${escapeHtml(humanizeText(kr.current_value) || "0")} / ${escapeHtml(humanizeText(kr.target_value) || "100")}</span>
        </div>
        <div class="progress-bar small">
          <div class="progress-fill" style="width: ${((kr.current_value || 0) / (kr.target_value || 100)) * 100}%"></div>
        </div>
      </div>
    `,
      )
      .join("");
  }

  Object.assign(app, {
    loadObjectives,
    renderObjectives,
    selectObjective,
    renderObjectiveDetails,
    renderKeyResults,
  });
})(window.__gbGoals = window.__gbGoals || {});
