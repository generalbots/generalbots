
function showApprovalNotification(approval) {
  showToast(`Approval required: ${approval.title}`, "warning", 10000);
  updateStats();
}

// =============================================================================
// SIMULATION
// =============================================================================

function renderSimulationResult(result) {
  const container = document.getElementById("simulation-content");

  const statusIcon = result.success ? "✅" : "⚠️";
  const statusText = result.success
    ? "Simulation Successful"
    : "Simulation Found Issues";

  let html = `
        <div class="simulation-result">
            <div class="simulation-header">
                <div class="simulation-status status-${result.success}">
                    <span class="status-icon">${statusIcon}</span>
                    <span class="status-text">${statusText}</span>
                </div>
                <div class="simulation-confidence">
                    Confidence: ${Math.round(result.confidence * 100)}%
                </div>
            </div>

            <div class="impact-overview">
                <h4>Impact Assessment</h4>
                <div class="impact-grid">
                    <div class="impact-card">
                        <span class="impact-icon">💾</span>
                        <span class="impact-label">Data Impact</span>
                        <span class="impact-value">${result.impact.data_impact.records_modified} records modified</span>
                    </div>
                    <div class="impact-card">
                        <span class="impact-icon">💰</span>
                        <span class="impact-label">Cost Impact</span>
                        <span class="impact-value">$${result.impact.cost_impact.total_estimated_cost.toFixed(2)}</span>
                    </div>
                    <div class="impact-card">
                        <span class="impact-icon">⏱️</span>
                        <span class="impact-label">Time Impact</span>
                        <span class="impact-value">${formatDuration(result.impact.time_impact.estimated_duration_seconds)}</span>
                    </div>
                    <div class="impact-card risk-${result.impact.security_impact.risk_level.toLowerCase()}">
                        <span class="impact-icon">🔒</span>
                        <span class="impact-label">Security Impact</span>
                        <span class="impact-value">${result.impact.security_impact.risk_level}</span>
                    </div>
                </div>
            </div>

            <div class="step-outcomes">
                <h4>Step-by-Step Predictions</h4>
                <div class="outcomes-list">
                    ${result.step_outcomes
                      .map(
                        (step) => `
                        <div class="outcome-item ${step.would_succeed ? "success" : "warning"}">
                            <span class="outcome-icon">${step.would_succeed ? "✅" : "⚠️"}</span>
                            <span class="outcome-name">${step.step_name}</span>
                            <span class="outcome-probability">${Math.round(step.success_probability * 100)}% success</span>
                        </div>
                    `,
                      )
                      .join("")}
                </div>
            </div>

            ${
              result.side_effects.length > 0
                ? `
                <div class="side-effects">
                    <h4>⚠️ Potential Side Effects</h4>
                    <div class="side-effects-list">
                        ${result.side_effects
                          .map(
                            (effect) => `
                            <div class="side-effect-item severity-${effect.severity.toLowerCase()}">
                                <span class="effect-description">${effect.description}</span>
                                ${effect.mitigation ? `<span class="effect-mitigation">Mitigation: ${effect.mitigation}</span>` : ""}
                            </div>
                        `,
                          )
                          .join("")}
                    </div>
                </div>
            `
                : ""
            }

            ${
              result.recommendations.length > 0
                ? `
                <div class="recommendations">
                    <h4>💡 Recommendations</h4>
                    <div class="recommendations-list">
                        ${result.recommendations
                          .map(
                            (rec) => `
                            <div class="recommendation-item">
                                <span class="rec-description">${rec.description}</span>
                                ${rec.action ? `<button class="btn-apply-rec" onclick="applyRecommendation('${rec.id}')">${rec.action}</button>` : ""}
                            </div>
                        `,
                          )
                          .join("")}
                    </div>
                </div>
            `
                : ""
            }

            <div class="simulation-actions">
                <button class="btn-secondary" onclick="closeSimulationModal()">
                    <span>↩️</span> Back
                </button>
                <button class="btn-primary" onclick="proceedAfterSimulation('${result.task_id}')" ${result.impact.risk_score > 0.8 ? "disabled" : ""}>
                    <span>🚀</span> Proceed with Execution
                </button>
            </div>
        </div>
    `;

  container.innerHTML = html;
}

// =============================================================================
// MODAL FUNCTIONS
// =============================================================================

function showSimulationModal() {
  const modal = document.getElementById("simulation-modal");
  if (modal) {
    modal.style.display = "flex";
    document.body.classList.add("modal-open");
    // Show loading state
    document.getElementById("simulation-content").innerHTML = `
            <div class="loading-state">
                <div class="spinner"></div>
                <p>Running impact simulation...</p>
            </div>
        `;
  }
}

function closeSimulationModal() {
  const modal = document.getElementById("simulation-modal");
  if (modal) {
    modal.style.display = "none";
    document.body.classList.remove("modal-open");
  }
}

function showDecisionModal() {
  const modal = document.getElementById("decision-modal");
  if (modal) {
    modal.style.display = "flex";
    document.body.classList.add("modal-open");
    // Show loading state
    document.getElementById("decision-content").innerHTML = `
            <div class="loading-state">
                <div class="spinner"></div>
                <p>Loading decisions...</p>
            </div>
        `;
  }
}

function closeDecisionModal() {
  const modal = document.getElementById("decision-modal");
  if (modal) {
    modal.style.display = "none";
    document.body.classList.remove("modal-open");
  }
}

function showApprovalModal() {
  const modal = document.getElementById("approval-modal");
  if (modal) {
    modal.style.display = "flex";
    document.body.classList.add("modal-open");
    // Show loading state
    document.getElementById("approval-content").innerHTML = `
            <div class="loading-state">
                <div class="spinner"></div>
                <p>Loading approvals...</p>
            </div>
        `;
  }
}

function closeApprovalModal() {
  const modal = document.getElementById("approval-modal");
  if (modal) {
    modal.style.display = "none";
    document.body.classList.remove("modal-open");
  }
}

function closeAllModals() {
  closeSimulationModal();
  closeDecisionModal();
  closeApprovalModal();
}

// =============================================================================
// SIMULATION ACTIONS
// =============================================================================

function proceedAfterSimulation(taskId) {
  closeSimulationModal();

  if (!taskId) {
    showToast("No task ID provided", "error");
    return;
  }

  fetch(`/api/autotask/${taskId}/execute`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      confirmed: true,
    }),
  })
    .then((response) => response.json())
    .then((result) => {
      if (result.success) {
        showToast("Task execution started!", "success");
        refreshTasks();
      } else {
        showToast(result.error || "Failed to start execution", "error");
      }
    })
    .catch((error) => {
      console.error("Failed to proceed after simulation:", error);
      showToast("Failed to start execution", "error");
    });
}

function applyRecommendation(recId) {
  fetch(`/api/autotask/recommendations/${recId}/apply`, {
    method: "POST",
  })
    .then((response) => response.json())
    .then((result) => {
      if (result.success) {
        showToast("Recommendation applied", "success");
        // Re-run simulation to show updated results
        const taskId =
          document.querySelector(".simulation-result")?.dataset?.taskId;
        if (taskId) {
          simulateTask(taskId);
        }
      } else {
        showToast(result.error || "Failed to apply recommendation", "error");
      }
    })
    .catch((error) => {
      console.error("Failed to apply recommendation:", error);
      showToast("Failed to apply recommendation", "error");
    });
}

// =============================================================================
// TOAST NOTIFICATIONS
// =============================================================================

function showToast(message, type = "info") {
  // Get or create toast container
  let container = document.getElementById("toast-container");
  if (!container) {
    container = document.createElement("div");
    container.id = "toast-container";
    container.className = "toast-container";
    document.body.appendChild(container);
  }

  // Create toast element
  const toast = document.createElement("div");
  toast.className = `toast toast-${type}`;

  const icons = {
    success: "✅",
    error: "❌",
    warning: "⚠️",
    info: "ℹ️",
  };

  toast.innerHTML = `
        <span class="toast-icon">${icons[type] || icons.info}</span>
        <span class="toast-message">${message}</span>
        <button class="toast-close" onclick="this.parentElement.remove()">×</button>
    `;

  container.appendChild(toast);

  // Auto-remove after 5 seconds
  setTimeout(() => {
    toast.classList.add("toast-fade-out");
    setTimeout(() => {
      toast.remove();
    }, 300);
  }, 5000);
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

function formatDuration(seconds) {
  if (seconds < 60) {
    return `${Math.round(seconds)}s`;
  } else if (seconds < 3600) {
    const minutes = Math.floor(seconds / 60);
    const secs = Math.round(seconds % 60);
    return `${minutes}m ${secs}s`;
  } else {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
  }
}

function formatRelativeTime(dateString) {
  if (!dateString) return "N/A";

  const date = new Date(dateString);
  const now = new Date();
  const diffMs = date - now;
  const diffSeconds = Math.floor(diffMs / 1000);
  const diffMinutes = Math.floor(diffSeconds / 60);
  const diffHours = Math.floor(diffMinutes / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffMs < 0) {
    // Past
    const absDays = Math.abs(diffDays);
    const absHours = Math.abs(diffHours);
    const absMinutes = Math.abs(diffMinutes);

    if (absDays > 0) return `${absDays}d ago`;
    if (absHours > 0) return `${absHours}h ago`;
    if (absMinutes > 0) return `${absMinutes}m ago`;
    return "just now";
  } else {
    // Future
    if (diffDays > 0) return `in ${diffDays}d`;
    if (diffHours > 0) return `in ${diffHours}h`;
    if (diffMinutes > 0) return `in ${diffMinutes}m`;
    return "soon";
  }
}

// =============================================================================
// TASK LIFECYCLE HANDLERS
// =============================================================================

function onTaskCompleted(task) {
  showToast(`Task completed: ${task.title || task.id}`, "success");
  updateTaskInList(task);
  updateStats();
}

function onTaskFailed(task, error) {
  showToast(`Task failed: ${task.title || task.id} - ${error}`, "error");
  updateTaskInList(task);
  updateStats();
}

function highlightPendingItems() {
  // Highlight tasks requiring attention
  document.querySelectorAll(".autotask-item").forEach((item) => {
    const status = item.dataset.status;
    if (status === "pending-approval" || status === "pending-decision") {
      item.classList.add("attention-required");
    } else {
      item.classList.remove("attention-required");
    }
  });
}

function loadExecutionLogs(taskId) {
  const logContainer = document.querySelector(
    `[data-task-id="${taskId}"] .log-entries`,
  );
  if (!logContainer || logContainer.dataset.loaded === "true") {
    return;
  }

  logContainer.innerHTML = `
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading logs...</p>
    </div>
  `;

  fetch(`/api/autotask/${taskId}/logs`)
    .then((response) => response.json())
    .then((logs) => {
      if (!logs || logs.length === 0) {
        logContainer.innerHTML =
          "<p class='no-logs'>No execution logs yet.</p>";
      } else {
        logContainer.innerHTML = logs
          .map(
            (log) => `
            <div class="log-entry log-${log.level.toLowerCase()}">
              <span class="log-time">${new Date(log.timestamp).toLocaleTimeString()}</span>
              <span class="log-level">${log.level}</span>
              <span class="log-message">${log.message}</span>
            </div>
          `,
          )
          .join("");
      }
      logContainer.dataset.loaded = "true";
    })
    .catch((error) => {
      logContainer.innerHTML = `
        <div class="error-message">
          <span class="error-icon">❌</span>
          <p>Failed to load logs: ${error.message}</p>
        </div>
      `;
    });
}
