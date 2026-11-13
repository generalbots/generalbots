
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

  const isAlert = type === "error" || type === "warning";
  toast.setAttribute("role", isAlert ? "alert" : "status");
  toast.setAttribute("aria-live", isAlert ? "assertive" : "polite");
  toast.setAttribute("aria-atomic", "true");

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
        <span style="font-size: 16px;" aria-hidden="true">${icons[type] || icons.info}</span>
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

function gbConfirm(message, title) {
  const t = title || "Confirm";
  return new Promise((resolve) => {
    if (window.GBModal && typeof window.GBModal.confirm === "function") {
      try {
        window.GBModal.confirm(String(message), t, () => resolve(true), () => resolve(false));
        return;
      } catch (e) {
        /* fall through */
      }
    }
    const overlay = document.createElement("div");
    overlay.style.cssText = "position:fixed;inset:0;z-index:10050;display:flex;align-items:center;justify-content:center;background:rgba(0,0,0,.45)";
    overlay.innerHTML = '<div style="background:#1e293b;color:#f8fafc;border:1px solid #334155;border-radius:10px;padding:16px;min-width:280px;max-width:420px;font-size:14px;box-shadow:0 8px 24px rgba(0,0,0,.4)">' +
      '<div style="font-weight:600;margin-bottom:8px">' + String(t).replace(/</g, "&lt;") + '</div>' +
      '<div style="margin-bottom:14px;white-space:pre-wrap">' + String(message).replace(/</g, "&lt;") + '</div>' +
      '<div style="display:flex;gap:8px;justify-content:flex-end">' +
      '<button data-gb-cancel style="padding:6px 12px;border-radius:6px;border:1px solid #475569;background:#334155;color:#f8fafc;cursor:pointer">Cancel</button>' +
      '<button data-gb-ok style="padding:6px 12px;border-radius:6px;border:0;background:#3b82f6;color:#fff;cursor:pointer">OK</button>' +
      '</div></div>';
    const done = (v) => { overlay.remove(); resolve(v); };
    overlay.querySelector("[data-gb-ok]").addEventListener("click", () => done(true));
    overlay.querySelector("[data-gb-cancel]").addEventListener("click", () => done(false));
    overlay.addEventListener("click", (e) => { if (e.target === overlay) done(false); });
    document.body.appendChild(overlay);
  });
}

function gbAlert(message, title) {
  const t = title || "Notice";
  if (window.GBModal && typeof window.GBModal.alert === "function") {
    try { window.GBModal.alert(String(message), t); return; } catch (e) { /* fallback */ }
  }
  if (window.GBToasts && typeof window.GBToasts.show === "function") {
    window.GBToasts.show(t, String(message), "info");
    return;
  }
  if (typeof showToast === "function") showToast(String(message), "info");
}

function gbPrompt(message, defaultValue, title) {
  const t = title || "Input";
  const d = defaultValue == null ? "" : String(defaultValue);
  return new Promise((resolve) => {
    if (window.GBModal && typeof window.GBModal.prompt === "function") {
      try {
        window.GBModal.prompt(String(message), d, t, (v) => resolve(v == null ? null : String(v)));
        return;
      } catch (e) { /* fallback */ }
    }
    const overlay = document.createElement("div");
    overlay.style.cssText = "position:fixed;inset:0;z-index:10050;display:flex;align-items:center;justify-content:center;background:rgba(0,0,0,.45)";
    overlay.innerHTML = '<div style="background:#1e293b;color:#f8fafc;border:1px solid #334155;border-radius:10px;padding:16px;min-width:300px;max-width:420px;font-size:14px;box-shadow:0 8px 24px rgba(0,0,0,.4)">' +
      '<div style="font-weight:600;margin-bottom:8px">' + String(t).replace(/</g, "&lt;") + '</div>' +
      '<div style="margin-bottom:8px;white-space:pre-wrap">' + String(message).replace(/</g, "&lt;") + '</div>' +
      '<input data-gb-input style="width:100%;box-sizing:border-box;margin-bottom:14px;padding:8px;border-radius:6px;border:1px solid #475569;background:#0f172a;color:#f8fafc" />' +
      '<div style="display:flex;gap:8px;justify-content:flex-end">' +
      '<button data-gb-cancel style="padding:6px 12px;border-radius:6px;border:1px solid #475569;background:#334155;color:#f8fafc;cursor:pointer">Cancel</button>' +
      '<button data-gb-ok style="padding:6px 12px;border-radius:6px;border:0;background:#3b82f6;color:#fff;cursor:pointer">OK</button>' +
      '</div></div>';
    const input = overlay.querySelector("[data-gb-input]");
    input.value = d;
    const done = (v) => { overlay.remove(); resolve(v); };
    overlay.querySelector("[data-gb-ok]").addEventListener("click", () => done(String(input.value)));
    overlay.querySelector("[data-gb-cancel]").addEventListener("click", () => done(null));
    input.addEventListener("keydown", (e) => { if (e.key === "Enter") done(String(input.value)); if (e.key === "Escape") done(null); });
    overlay.addEventListener("click", (e) => { if (e.target === overlay) done(null); });
    document.body.appendChild(overlay);
    setTimeout(() => input.focus(), 0);
  });
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

function tStatus(key, fallback) {
  return window.gbT ? window.gbT(key, fallback) : fallback;
}

function formatStatus(status) {
  const statusMap = {
    complete: tStatus("tasks-status-complete", "Complete"),
    running: tStatus("tasks-status-running", "Running"),
    awaiting: tStatus("tasks-status-awaiting", "Awaiting"),
    paused: tStatus("tasks-status-paused", "Paused"),
    blocked: tStatus("tasks-status-blocked", "Blocked"),
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

function getTasksRoot() {
  return document.getElementById("window-tasks") || document;
}

function ensureTaskDetailVisible() {
  const empty = document.getElementById("task-detail-empty") || document.getElementById("detail-empty");
  const content = document.getElementById("task-detail-content");
  if (empty) empty.style.display = "none";
  if (content) content.style.display = "block";
  return content;
}

function escapeTasksText(value) {
  if (typeof escapeHtml === "function") return escapeHtml(String(value == null ? "" : value));
  return String(value == null ? "" : value)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function renderTaskDetailFallback(title, rows) {
  const content = ensureTaskDetailVisible();
  if (!content) return;
  const body = (rows || [])
    .filter(function (row) { return row && row[1] !== undefined && row[1] !== null && row[1] !== ""; })
    .map(function (row) {
      return '<div class="detail-row"><strong>' + escapeTasksText(row[0]) + ':</strong> <span>' + escapeTasksText(row[1]) + '</span></div>';
    })
    .join("");
  content.innerHTML =
    '<div class="task-detail-fallback">' +
    '<h3 class="detail-title">' + escapeTasksText(title || "Details") + '</h3>' +
    (body || '<p class="detail-empty-note">No additional details available.</p>') +
    '</div>';
}

// Select a goal and show its details
window.selectGoal = function (goalId) {
  if (typeof TasksState !== "undefined" && TasksState) TasksState.selectedItemType = "goal";
  window.selectedTaskId = goalId;

  var root = getTasksRoot();
  root.querySelectorAll(".task-item, .task-card").forEach(function (el) {
    el.classList.remove("selected");
  });
  var selectedEl = root.querySelector('[data-goal-id="' + goalId + '"]') || document.querySelector('[data-goal-id="' + goalId + '"]');
  if (selectedEl) selectedEl.classList.add("selected");

  ensureTaskDetailVisible();
  hideAllDetailSections();
  var goalSection = document.getElementById("goal-progress-section");
  if (goalSection) goalSection.style.display = "block";

  fetch("/api/goals/" + goalId)
    .then(function (response) { return response.json(); })
    .then(function (goal) {
      var titleEl = document.getElementById("detail-title");
      var statusEl = document.getElementById("detail-status-text");
      var priorityEl = document.getElementById("detail-priority-text");
      var descEl = document.getElementById("detail-description");
      if (titleEl) titleEl.textContent = goal.goal_text || "Untitled Goal";
      if (statusEl) statusEl.textContent = goal.status || "active";
      if (priorityEl) priorityEl.textContent = "Goal";
      if (descEl) descEl.textContent = goal.goal_text || "";

      var percent = goal.target_value > 0
        ? Math.round((goal.current_value / goal.target_value) * 100)
        : 0;
      var fillEl = document.getElementById("goal-progress-fill");
      var currentEl = document.getElementById("goal-current-value");
      var targetEl = document.getElementById("goal-target-value");
      var percentEl = document.getElementById("goal-percent");
      var lastActionEl = document.getElementById("goal-last-action");
      if (fillEl) fillEl.style.width = percent + "%";
      if (currentEl) currentEl.textContent = goal.current_value || 0;
      if (targetEl) targetEl.textContent = goal.target_value || 0;
      if (percentEl) percentEl.textContent = percent;
      if (lastActionEl) lastActionEl.textContent = goal.last_action ? ("Last action: " + goal.last_action) : "No actions yet";

      if (!titleEl || !statusEl) {
        renderTaskDetailFallback(goal.goal_text || "Untitled Goal", [
          ["Status", goal.status || "active"],
          ["Type", "Goal"],
          ["Progress", (goal.current_value || 0) + " / " + (goal.target_value || 0) + " (" + percent + "%)"],
          ["Last action", goal.last_action || "No actions yet"],
          ["Description", goal.goal_text || ""]
        ]);
      }
    })
    .catch(function (err) { console.error("Failed to load goal:", err); });
};

// Select a pending info item
window.selectPendingInfo = function (pendingId) {
  if (typeof TasksState !== "undefined" && TasksState) TasksState.selectedItemType = "pending";
  window.selectedTaskId = pendingId;

  var root = getTasksRoot();
  root.querySelectorAll(".task-item, .task-card").forEach(function (el) {
    el.classList.remove("selected");
  });
  var selectedEl = root.querySelector('[data-pending-id="' + pendingId + '"]') || document.querySelector('[data-pending-id="' + pendingId + '"]');
  if (selectedEl) selectedEl.classList.add("selected");

  ensureTaskDetailVisible();
  hideAllDetailSections();
  var pendingSection = document.getElementById("pending-fill-section");
  if (pendingSection) pendingSection.style.display = "block";

  fetch("/api/pending-info/" + pendingId)
    .then(function (response) { return response.json(); })
    .then(function (pending) {
      var titleEl = document.getElementById("detail-title");
      var statusEl = document.getElementById("detail-status-text");
      var priorityEl = document.getElementById("detail-priority-text");
      var descEl = document.getElementById("detail-description");
      if (titleEl) titleEl.textContent = pending.field_label || "Pending Info";
      if (statusEl) statusEl.textContent = "Pending";
      if (priorityEl) priorityEl.textContent = pending.app_name || "";
      if (descEl) descEl.textContent = pending.reason || "";

      var reasonEl = document.getElementById("pending-reason");
      var idEl = document.getElementById("pending-fill-id");
      var labelEl = document.getElementById("pending-fill-label");
      var valueEl = document.getElementById("pending-fill-value");
      if (reasonEl) reasonEl.textContent = pending.reason || "Required for app functionality";
      if (idEl) idEl.value = pending.id;
      if (labelEl) labelEl.textContent = pending.field_label;
      if (valueEl) valueEl.type = pending.field_type === "secret" ? "password" : "text";

      if (!titleEl || !statusEl) {
        renderTaskDetailFallback(pending.field_label || "Pending Info", [
          ["Status", "Pending"],
          ["App", pending.app_name || ""],
          ["Field", pending.field_label || ""],
          ["Type", pending.field_type || "text"],
          ["Reason", pending.reason || "Required for app functionality"]
        ]);
      }
    })
    .catch(function (err) { console.error("Failed to load pending info:", err); });
};

// Select a scheduler
window.selectScheduler = function (schedulerName) {
  if (typeof TasksState !== "undefined" && TasksState) TasksState.selectedItemType = "scheduler";
  window.selectedTaskId = schedulerName;

  var root = getTasksRoot();
  root.querySelectorAll(".task-item, .task-card").forEach(function (el) {
    el.classList.remove("selected");
  });
  var selectedEl = root.querySelector('[data-scheduler-name="' + schedulerName + '"]') || document.querySelector('[data-scheduler-name="' + schedulerName + '"]');
  if (selectedEl) selectedEl.classList.add("selected");

  ensureTaskDetailVisible();
  hideAllDetailSections();
  var schedulerSection = document.getElementById("scheduler-info-section");
  if (schedulerSection) schedulerSection.style.display = "block";

  fetch("/api/schedulers/" + encodeURIComponent(schedulerName))
    .then(function (response) { return response.json(); })
    .then(function (scheduler) {
      var titleEl = document.getElementById("detail-title");
      var statusEl = document.getElementById("detail-status-text");
      var priorityEl = document.getElementById("detail-priority-text");
      var descEl = document.getElementById("detail-description");
      if (titleEl) titleEl.textContent = scheduler.name || schedulerName;
      if (statusEl) statusEl.textContent = scheduler.status || "active";
      if (priorityEl) priorityEl.textContent = "Scheduler";
      if (descEl) descEl.textContent = scheduler.description || "";

      var cronEl = document.getElementById("scheduler-cron");
      var nextEl = document.getElementById("scheduler-next");
      var fileEl = document.getElementById("scheduler-file");
      if (cronEl) cronEl.textContent = scheduler.cron || "-";
      if (nextEl) nextEl.textContent = scheduler.next_run
        ? ("Next run: " + (typeof gbFormatDateTime === "function" ? gbFormatDateTime(new Date(scheduler.next_run)) : scheduler.next_run))
        : "Next run: -";
      if (fileEl) fileEl.textContent = scheduler.file ? ("File: " + scheduler.file) : "File: -";

      if (!titleEl || !statusEl) {
        renderTaskDetailFallback(scheduler.name || schedulerName, [
          ["Status", scheduler.status || "active"],
          ["Type", "Scheduler"],
          ["Cron", scheduler.cron || "-"],
          ["Next run", scheduler.next_run ? (typeof gbFormatDateTime === "function" ? gbFormatDateTime(new Date(scheduler.next_run)) : scheduler.next_run) : "-"],
          ["File", scheduler.file || "-"],
          ["Description", scheduler.description || ""]
        ]);
      }
    })
    .catch(function (err) { console.error("Failed to load scheduler:", err); });
};

// Select a monitor
window.selectMonitor = function (monitorName) {
  if (typeof TasksState !== "undefined" && TasksState) TasksState.selectedItemType = "monitor";
  window.selectedTaskId = monitorName;

  var root = getTasksRoot();
  root.querySelectorAll(".task-item, .task-card").forEach(function (el) {
    el.classList.remove("selected");
  });
  var selectedEl = root.querySelector('[data-monitor-name="' + monitorName + '"]') || document.querySelector('[data-monitor-name="' + monitorName + '"]');
  if (selectedEl) selectedEl.classList.add("selected");

  ensureTaskDetailVisible();
  hideAllDetailSections();
  var monitorSection = document.getElementById("monitor-info-section");
  if (monitorSection) monitorSection.style.display = "block";

  fetch("/api/monitors/" + encodeURIComponent(monitorName))
    .then(function (response) { return response.json(); })
    .then(function (monitor) {
      var titleEl = document.getElementById("detail-title");
      var statusEl = document.getElementById("detail-status-text");
      var priorityEl = document.getElementById("detail-priority-text");
      var descEl = document.getElementById("detail-description");
      if (titleEl) titleEl.textContent = monitor.name || monitorName;
      if (statusEl) statusEl.textContent = monitor.status || "active";
      if (priorityEl) priorityEl.textContent = "Monitor";
      if (descEl) descEl.textContent = monitor.description || "";

      var targetEl = document.getElementById("monitor-target");
      var intervalEl = document.getElementById("monitor-interval");
      var lastCheckEl = document.getElementById("monitor-last-check");
      var lastValueEl = document.getElementById("monitor-last-value");
      if (targetEl) targetEl.textContent = monitor.target ? ("Target: " + monitor.target) : "Target: -";
      if (intervalEl) intervalEl.textContent = monitor.interval ? ("Interval: " + monitor.interval) : "Interval: -";
      if (lastCheckEl) lastCheckEl.textContent = monitor.last_check
        ? ("Last check: " + (typeof gbFormatDateTime === "function" ? gbFormatDateTime(new Date(monitor.last_check)) : monitor.last_check))
        : "Last check: -";
      if (lastValueEl) lastValueEl.textContent = monitor.last_value ? ("Last value: " + monitor.last_value) : "Last value: -";

      if (!titleEl || !statusEl) {
        renderTaskDetailFallback(monitor.name || monitorName, [
          ["Status", monitor.status || "active"],
          ["Type", "Monitor"],
          ["Target", monitor.target || "-"],
          ["Interval", monitor.interval || "-"],
          ["Last check", monitor.last_check ? (typeof gbFormatDateTime === "function" ? gbFormatDateTime(new Date(monitor.last_check)) : monitor.last_check) : "-"],
          ["Last value", monitor.last_value || "-"],
          ["Description", monitor.description || ""]
        ]);
      }
    })
    .catch(function (err) { console.error("Failed to load monitor:", err); });
};

// Hide all detail sections
function hideAllDetailSections() {
  var ids = [
    "goal-progress-section",
    "pending-fill-section",
    "scheduler-info-section",
    "monitor-info-section"
  ];
  ids.forEach(function (id) {
    var el = document.getElementById(id);
    if (el) el.style.display = "none";
  });
}

// Fill pending info form submission
document.addEventListener("htmx:afterRequest", function (event) {
  if (!event || !event.detail || !event.detail.elt) return;
  if (event.detail.elt.id === "pending-fill-form" && event.detail.successful) {
    if (typeof htmx !== "undefined" && htmx.trigger) htmx.trigger(document.body, "taskCreated");
    var pendingValue = document.getElementById("pending-fill-value");
    if (pendingValue) pendingValue.value = "";
    if (typeof addAgentLog === "function") addAgentLog("success", "[OK] Pending info filled successfully");
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
