/* Split from tasks-toasts.js for #2305 (450-line limit). */
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

  var apply = function (goal) {
      if (!goal) return;
      var titleEl = document.getElementById("detail-title");
      var statusEl = document.getElementById("detail-status-text");
      var priorityEl = document.getElementById("detail-priority-text");
      var descEl = document.getElementById("detail-description");
      if (titleEl) titleEl.textContent = goal.goal_text || tDetail("tasks-detail-untitled-goal", "Untitled Goal");
      if (statusEl) statusEl.textContent = goal.status || tDetail("label-active", "active");
      if (priorityEl) priorityEl.textContent = dLabel("Goal");
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
      if (lastActionEl) lastActionEl.textContent = goal.last_action ? (dLabel("Last action") + ": " + goal.last_action) : tDetail("tasks-detail-no-actions", "No actions yet");

      if (!titleEl || !statusEl) {
        renderTaskDetailFallback(goal.goal_text || tDetail("tasks-detail-untitled-goal", "Untitled Goal"), [
          [dLabel("Status"), goal.status || tDetail("label-active", "active")],
          [dLabel("Type"), dLabel("Goal")],
          [dLabel("Progress"), (goal.current_value || 0) + " / " + (goal.target_value || 0) + " (" + percent + "%)"],
          [dLabel("Last action"), goal.last_action || tDetail("tasks-detail-no-actions", "No actions yet")],
          [dLabel("Description"), goal.goal_text || ""]
        ]);
      }
  };

  fetch("/api/goals/" + encodeURIComponent(goalId))
    .then(function (response) {
      if (!response.ok) throw new Error("goal endpoint unavailable");
      return response.json();
    })
    .then(function (goal) { apply(normalizeGoal(goal)); })
    .catch(function () {
      // The live backend exposes objectives under /api/goals/objectives/:id.
      fetch("/api/goals/objectives/" + encodeURIComponent(goalId))
        .then(function (response) {
          if (!response.ok) throw new Error("objective endpoint unavailable");
          return response.json();
        })
        .then(function (goal) { apply(normalizeGoal(goal)); })
        .catch(function () { renderGoalDetailFailure(goalId); });
    });
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
      if (titleEl) titleEl.textContent = pending.field_label || dLabel("Pending Info");
      if (statusEl) statusEl.textContent = dLabel("Pending");
      if (priorityEl) priorityEl.textContent = pending.app_name || "";
      if (descEl) descEl.textContent = pending.reason || "";

      var reasonEl = document.getElementById("pending-reason");
      var idEl = document.getElementById("pending-fill-id");
      var labelEl = document.getElementById("pending-fill-label");
      var valueEl = document.getElementById("pending-fill-value");
      if (reasonEl) reasonEl.textContent = pending.reason || dLabel("Required for app functionality");
      if (idEl) idEl.value = pending.id;
      if (labelEl) labelEl.textContent = pending.field_label;
      if (valueEl) valueEl.type = pending.field_type === "secret" ? "password" : "text";

      if (!titleEl || !statusEl) {
        renderTaskDetailFallback(pending.field_label || dLabel("Pending Info"), [
          [dLabel("Status"), dLabel("Pending")],
          [dLabel("App"), pending.app_name || ""],
          [dLabel("Field"), pending.field_label || ""],
          [dLabel("Type"), pending.field_type || "text"],
          [dLabel("Reason"), pending.reason || dLabel("Required for app functionality")]
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
      if (statusEl) statusEl.textContent = scheduler.status || tDetail("label-active", "active");
      if (priorityEl) priorityEl.textContent = dLabel("Scheduler");
      if (descEl) descEl.textContent = scheduler.description || "";

      var cronEl = document.getElementById("scheduler-cron");
      var nextEl = document.getElementById("scheduler-next");
      var fileEl = document.getElementById("scheduler-file");
      if (cronEl) cronEl.textContent = scheduler.cron || "-";
      if (nextEl) nextEl.textContent = scheduler.next_run
        ? (dLabel("Next run") + ": " + (typeof gbFormatDateTime === "function" ? gbFormatDateTime(new Date(scheduler.next_run)) : scheduler.next_run))
        : dLabel("Next run") + ": -";
      if (fileEl) fileEl.textContent = scheduler.file ? (dLabel("File") + ": " + scheduler.file) : dLabel("File") + ": -";

      if (!titleEl || !statusEl) {
        renderTaskDetailFallback(scheduler.name || schedulerName, [
          [dLabel("Status"), scheduler.status || tDetail("label-active", "active")],
          [dLabel("Type"), dLabel("Scheduler")],
          [dLabel("Cron"), scheduler.cron || "-"],
          [dLabel("Next run"), scheduler.next_run ? (typeof gbFormatDateTime === "function" ? gbFormatDateTime(new Date(scheduler.next_run)) : scheduler.next_run) : "-"],
          [dLabel("File"), scheduler.file || "-"],
          [dLabel("Description"), scheduler.description || ""]
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
      if (statusEl) statusEl.textContent = monitor.status || tDetail("label-active", "active");
      if (priorityEl) priorityEl.textContent = dLabel("Monitor");
      if (descEl) descEl.textContent = monitor.description || "";

      var targetEl = document.getElementById("monitor-target");
      var intervalEl = document.getElementById("monitor-interval");
      var lastCheckEl = document.getElementById("monitor-last-check");
      var lastValueEl = document.getElementById("monitor-last-value");
      if (targetEl) targetEl.textContent = monitor.target ? (dLabel("Target") + ": " + monitor.target) : dLabel("Target") + ": -";
      if (intervalEl) intervalEl.textContent = monitor.interval ? (dLabel("Interval") + ": " + monitor.interval) : dLabel("Interval") + ": -";
      if (lastCheckEl) lastCheckEl.textContent = monitor.last_check
        ? (dLabel("Last check") + ": " + (typeof gbFormatDateTime === "function" ? gbFormatDateTime(new Date(monitor.last_check)) : monitor.last_check))
        : dLabel("Last check") + ": -";
      if (lastValueEl) lastValueEl.textContent = monitor.last_value ? (dLabel("Last value") + ": " + monitor.last_value) : dLabel("Last value") + ": -";

      if (!titleEl || !statusEl) {
        renderTaskDetailFallback(monitor.name || monitorName, [
          [dLabel("Status"), monitor.status || tDetail("label-active", "active")],
          [dLabel("Type"), dLabel("Monitor")],
          [dLabel("Target"), monitor.target || "-"],
          [dLabel("Interval"), monitor.interval || "-"],
          [dLabel("Last check"), monitor.last_check ? (typeof gbFormatDateTime === "function" ? gbFormatDateTime(new Date(monitor.last_check)) : monitor.last_check) : "-"],
          [dLabel("Last value"), monitor.last_value || "-"],
          [dLabel("Description"), monitor.description || ""]
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

// #2307 — detail-panel helpers. gbT keeps every user-facing label localised;
// the fallbacks are only used when the catalog has not been bundled yet.
function tDetail(key, fallback) {
  return window.gbT ? window.gbT(key, fallback) : fallback;
}

// #2307 — authored label map: every user-facing label in the detail handlers
// resolves through the catalogue; unknown text is returned unchanged.
var DETAIL_LABEL_KEYS = {
  "Goal": "tasks-detail-type-goal",
  "Pending": "tasks-detail-status-pending",
  "Pending Info": "tasks-detail-pending-title",
  "Required for app functionality": "tasks-detail-required-reason",
  "Scheduler": "tasks-detail-type-scheduler",
  "Monitor": "tasks-detail-type-monitor",
  "Last action": "tasks-detail-last-action",
  "Next run": "tasks-detail-next-run",
  "File": "tasks-detail-file",
  "Target": "label-target",
  "Interval": "tasks-detail-interval",
  "Last check": "tasks-detail-last-check",
  "Last value": "tasks-detail-last-value",
  "Status": "label-status",
  "Type": "label-type",
  "Description": "label-description",
  "Progress": "tasks-detail-progress",
  "App": "tasks-detail-app",
  "Field": "tasks-detail-field",
  "Reason": "tasks-detail-reason",
  "Cron": "tasks-detail-cron"
};

function dLabel(text) {
  var key = DETAIL_LABEL_KEYS[text];
  return key ? tDetail(key, text) : text;
}

function normalizeGoal(data) {
  if (!data || typeof data !== "object") return null;
  return {
    goal_text: data.goal_text || data.title || tDetail("tasks-detail-untitled-goal", "Untitled Goal"),
    status: data.status || tDetail("label-active", "active"),
    current_value: Number(data.current_value || data.progress || 0),
    target_value: Number(data.target_value || 100),
    last_action: data.last_action || ""
  };
}

function renderGoalDetailFailure(goalId) {
  var statusEl = document.getElementById("detail-status-text");
  var descEl = document.getElementById("detail-description");
  if (statusEl) statusEl.textContent = tDetail("tasks-detail-unavailable", "Unavailable");
  if (descEl) {
    descEl.textContent = tDetail("tasks-detail-load-failed", "Unable to load goal details") + " (" + goalId + ")";
  }
}

// #2307 — the detail sections are rendered by window.selectGoal() etc. Wire
// any card that carries the matching data-* attribute so a click actually
// opens the panel instead of leaving the handlers orphaned.
(function installDetailSelection() {
  document.addEventListener("click", function (event) {
    var el = event.target && event.target.closest
      ? event.target.closest("[data-goal-id],[data-pending-id],[data-scheduler-name],[data-monitor-name]")
      : null;
    if (!el) return;
    if (el.getAttribute("data-goal-id")) window.selectGoal(el.getAttribute("data-goal-id"));
    else if (el.getAttribute("data-pending-id")) window.selectPendingInfo(el.getAttribute("data-pending-id"));
    else if (el.getAttribute("data-scheduler-name")) window.selectScheduler(el.getAttribute("data-scheduler-name"));
    else if (el.getAttribute("data-monitor-name")) window.selectMonitor(el.getAttribute("data-monitor-name"));
  });
})();
