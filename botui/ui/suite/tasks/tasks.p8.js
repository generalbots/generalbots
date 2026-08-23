
function showToast(message, type = "info", duration = 4000, onClick = null) {
  let container = document.getElementById("toast-container");
  if (!container) {
    container = document.createElement("div");
    container.id = "toast-container";
    container.style.cssText = `
            position: fixed;
            bottom: 24px;
            right: 24px;
            z-index: 10000;
            display: flex;
            flex-direction: column;
            gap: 8px;
        `;
    document.body.appendChild(container);
  }

  const toast = document.createElement("div");
  const bgColors = {
    success: "rgba(34, 197, 94, 0.95)",
    error: "rgba(239, 68, 68, 0.95)",
    warning: "rgba(245, 158, 11, 0.95)",
    info: "rgba(59, 130, 246, 0.95)",
  };

  const icons = {
    success: "✓",
    error: "✕",
    warning: "⚠",
    info: "ℹ",
  };

  toast.style.cssText = `
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 12px 16px;
        background: ${bgColors[type] || bgColors.info};
        border-radius: 10px;
        color: white;
        font-size: 14px;
        font-weight: 500;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
        animation: slideIn 0.3s ease;
    `;

  toast.innerHTML = `
        <span style="font-size: 16px;">${icons[type] || icons.info}</span>
        <span>${message}</span>
    `;

  if (onClick) {
    toast.style.cursor = "pointer";
    toast.addEventListener("click", onClick);
  }

  container.appendChild(toast);

  setTimeout(() => {
    toast.style.animation = "fadeOut 0.3s ease forwards";
    setTimeout(() => toast.remove(), 300);
  }, duration);
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

function debounce(func, wait) {
  let timeout;
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout);
      func(...args);
    };
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}

function formatStatus(status) {
  const statusMap = {
    complete: "Complete",
    running: "Running",
    awaiting: "Awaiting",
    paused: "Paused",
    blocked: "Blocked",
  };
  return statusMap[status] || status;
}

function formatTime(seconds) {
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) {
    const mins = Math.floor(seconds / 60);
    return `${mins}m`;
  }
  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  return `${hours}h ${mins}m`;
}

// =============================================================================
// GLOBAL STYLES FOR TOAST ANIMATIONS
// =============================================================================

if (typeof taskStyleElement === "undefined") {
  var taskStyleElement = document.createElement("style");
  taskStyleElement.textContent = `
    @keyframes slideIn {
        from {
            opacity: 0;
            transform: translateX(20px);
        }
        to {
            opacity: 1;
            transform: translateX(0);
        }
    }

    @keyframes fadeOut {
        from {
            opacity: 1;
            transform: translateX(0);
        }
        to {
            opacity: 0;
            transform: translateX(20px);
        }
    }

    @keyframes slideInRight {
        from {
            opacity: 0;
            transform: translateX(100px);
        }
        to {
            opacity: 1;
            transform: translateX(0);
        }
    }

    @keyframes slideOutRight {
        from {
            opacity: 1;
            transform: translateX(0);
        }
        to {
            opacity: 0;
            transform: translateX(100px);
        }
    }
`;
  document.head.appendChild(taskStyleElement);
}

// =============================================================================
// GOALS, PENDING INFO, SCHEDULERS, MONITORS
// =============================================================================

// Select a goal and show its details
window.selectGoal = function (goalId) {
  TasksState.selectedItemType = "goal";
  window.selectedTaskId = goalId;

  document.querySelectorAll(".task-item, .task-card").forEach((el) => {
    el.classList.remove("selected");
  });
  const selectedEl = document.querySelector(`[data-goal-id="${goalId}"]`);
  if (selectedEl) {
    selectedEl.classList.add("selected");
  }

  document.getElementById("task-detail-empty").style.display = "none";
  document.getElementById("task-detail-content").style.display = "block";

  // Hide other sections, show goal section
  hideAllDetailSections();
  document.getElementById("goal-progress-section").style.display = "block";

  fetch(`/api/goals/${goalId}`)
    .then((response) => response.json())
    .then((goal) => {
      document.getElementById("detail-title").textContent =
        goal.goal_text || "Untitled Goal";
      document.getElementById("detail-status-text").textContent =
        goal.status || "active";
      document.getElementById("detail-priority-text").textContent = "Goal";
      document.getElementById("detail-description").textContent =
        goal.goal_text || "";

      const percent =
        goal.target_value > 0
          ? Math.round((goal.current_value / goal.target_value) * 100)
          : 0;
      document.getElementById("goal-progress-fill").style.width = `${percent}%`;
      document.getElementById("goal-current-value").textContent =
        goal.current_value || 0;
      document.getElementById("goal-target-value").textContent =
        goal.target_value || 0;
      document.getElementById("goal-percent").textContent = percent;
      document.getElementById("goal-last-action").textContent = goal.last_action
        ? `Last action: ${goal.last_action}`
        : "No actions yet";
    })
    .catch((err) => console.error("Failed to load goal:", err));
};

// Select a pending info item
window.selectPendingInfo = function (pendingId) {
  TasksState.selectedItemType = "pending";
  window.selectedTaskId = pendingId;

  document.querySelectorAll(".task-item, .task-card").forEach((el) => {
    el.classList.remove("selected");
  });
  const selectedEl = document.querySelector(`[data-pending-id="${pendingId}"]`);
  if (selectedEl) {
    selectedEl.classList.add("selected");
  }

  document.getElementById("task-detail-empty").style.display = "none";
  document.getElementById("task-detail-content").style.display = "block";

  hideAllDetailSections();
  document.getElementById("pending-fill-section").style.display = "block";

  fetch(`/api/pending-info/${pendingId}`)
    .then((response) => response.json())
    .then((pending) => {
      document.getElementById("detail-title").textContent =
        pending.field_label || "Pending Info";
      document.getElementById("detail-status-text").textContent = "Pending";
      document.getElementById("detail-priority-text").textContent =
        pending.app_name || "";
      document.getElementById("detail-description").textContent =
        pending.reason || "";

      document.getElementById("pending-reason").textContent =
        pending.reason || "Required for app functionality";
      document.getElementById("pending-fill-id").value = pending.id;
      document.getElementById("pending-fill-label").textContent =
        pending.field_label;
      document.getElementById("pending-fill-value").type =
        pending.field_type === "secret" ? "password" : "text";
    })
    .catch((err) => console.error("Failed to load pending info:", err));
};

// Select a scheduler
window.selectScheduler = function (schedulerName) {
  TasksState.selectedItemType = "scheduler";
  window.selectedTaskId = schedulerName;

  document.querySelectorAll(".task-item, .task-card").forEach((el) => {
    el.classList.remove("selected");
  });
  const selectedEl = document.querySelector(
    `[data-scheduler-name="${schedulerName}"]`,
  );
  if (selectedEl) {
    selectedEl.classList.add("selected");
  }

  document.getElementById("task-detail-empty").style.display = "none";
  document.getElementById("task-detail-content").style.display = "block";

  hideAllDetailSections();
  document.getElementById("scheduler-info-section").style.display = "block";

  fetch(`/api/schedulers/${encodeURIComponent(schedulerName)}`)
    .then((response) => response.json())
    .then((scheduler) => {
      document.getElementById("detail-title").textContent =
        scheduler.name || schedulerName;
      document.getElementById("detail-status-text").textContent =
        scheduler.status || "active";
      document.getElementById("detail-priority-text").textContent = "Scheduler";
      document.getElementById("detail-description").textContent =
        scheduler.description || "";

      document.getElementById("scheduler-cron").textContent =
        scheduler.cron || "-";
      document.getElementById("scheduler-next").textContent = scheduler.next_run
        ? `Next run: ${new Date(scheduler.next_run).toLocaleString()}`
        : "Next run: -";
      document.getElementById("scheduler-file").textContent = scheduler.file
        ? `File: ${scheduler.file}`
        : "File: -";
    })
    .catch((err) => console.error("Failed to load scheduler:", err));
};

// Select a monitor
window.selectMonitor = function (monitorName) {
  TasksState.selectedItemType = "monitor";
  window.selectedTaskId = monitorName;

  document.querySelectorAll(".task-item, .task-card").forEach((el) => {
    el.classList.remove("selected");
  });
  const selectedEl = document.querySelector(
    `[data-monitor-name="${monitorName}"]`,
  );
  if (selectedEl) {
    selectedEl.classList.add("selected");
  }

  document.getElementById("task-detail-empty").style.display = "none";
  document.getElementById("task-detail-content").style.display = "block";

  hideAllDetailSections();
  document.getElementById("monitor-info-section").style.display = "block";

  fetch(`/api/monitors/${encodeURIComponent(monitorName)}`)
    .then((response) => response.json())
    .then((monitor) => {
      document.getElementById("detail-title").textContent =
        monitor.name || monitorName;
      document.getElementById("detail-status-text").textContent =
        monitor.status || "active";
      document.getElementById("detail-priority-text").textContent = "Monitor";
      document.getElementById("detail-description").textContent =
        monitor.description || "";

      document.getElementById("monitor-target").textContent = monitor.target
        ? `Target: ${monitor.target}`
        : "Target: -";
      document.getElementById("monitor-interval").textContent = monitor.interval
        ? `Interval: ${monitor.interval}`
        : "Interval: -";
      document.getElementById("monitor-last-check").textContent =
        monitor.last_check
          ? `Last check: ${new Date(monitor.last_check).toLocaleString()}`
          : "Last check: -";
      document.getElementById("monitor-last-value").textContent =
        monitor.last_value
          ? `Last value: ${monitor.last_value}`
          : "Last value: -";
    })
    .catch((err) => console.error("Failed to load monitor:", err));
};

// Hide all detail sections
function hideAllDetailSections() {
  document.getElementById("goal-progress-section").style.display = "none";
  document.getElementById("pending-fill-section").style.display = "none";
  document.getElementById("scheduler-info-section").style.display = "none";
  document.getElementById("monitor-info-section").style.display = "none";
}

// Fill pending info form submission
document.addEventListener("htmx:afterRequest", function (event) {
  if (event.detail.elt.id === "pending-fill-form" && event.detail.successful) {
    htmx.trigger(document.body, "taskCreated");
    document.getElementById("pending-fill-value").value = "";
    addAgentLog("success", "[OK] Pending info filled successfully");
  }
});

// Update counts for new filters
function updateFilterCounts() {
  fetch("/api/tasks/stats/json")
    .then((response) => response.json())
    .then((stats) => {
      if (stats.total !== undefined) {
        const el = document.getElementById("count-all");
        if (el) el.textContent = stats.total;
      }
      if (stats.completed !== undefined) {
        const el = document.getElementById("count-complete");
        if (el) el.textContent = stats.completed;
      }
      if (stats.active !== undefined) {
        const el = document.getElementById("count-active");
        if (el) el.textContent = stats.active;
      }
      if (stats.awaiting !== undefined) {
        const el = document.getElementById("count-awaiting");
        if (el) el.textContent = stats.awaiting;
      }
      if (stats.paused !== undefined) {
        const el = document.getElementById("count-paused");
        if (el) el.textContent = stats.paused;
      }
      if (stats.blocked !== undefined) {
        const el = document.getElementById("count-blocked");
        if (el) el.textContent = stats.blocked;
      }
      if (stats.time_saved !== undefined) {
        const el = document.getElementById("time-saved-value");
        if (el) el.textContent = stats.time_saved;
      }
    })
    .catch((e) => console.warn("Failed to load task stats:", e));
}

// Call updateFilterCounts on load
(function(){ var __cb = updateFilterCounts; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
document.body.addEventListener("taskCreated", updateFilterCounts);

// =============================================================================
// MODAL FUNCTIONS
// =============================================================================

function showNewIntentModal() {
  var modal = document.getElementById("new-intent-modal");
  if (modal) {
    modal.style.display = "flex";
  }
}

function closeNewIntentModal() {
  var modal = document.getElementById("new-intent-modal");
  if (modal) {
    modal.style.display = "none";
  }
}

function showDecisionModal(decision) {
  var questionEl = document.getElementById("decision-question");
  if (decision && questionEl) {
    var title = decision.title || "Decision Required";
    var description = decision.description || "";
    questionEl.innerHTML =
      "<h4>" +
      escapeHtml(title) +
      "</h4>" +
      "<p>" +
      escapeHtml(description) +
      "</p>";
  }
  var modal = document.getElementById("decision-modal");
  if (modal) {
    modal.style.display = "flex";
  }
}

function closeDecisionModal() {
  var modal = document.getElementById("decision-modal");
  if (modal) {
    modal.style.display = "none";
  }
}
