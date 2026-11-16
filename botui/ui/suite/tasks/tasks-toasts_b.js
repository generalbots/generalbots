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
