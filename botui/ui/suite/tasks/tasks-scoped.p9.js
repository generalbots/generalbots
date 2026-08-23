window.selectPendingInfo = function (pendingId) {
  TasksState.selectedItemType = "pending";
  window.selectedTaskId = pendingId;

  document.getElementById("window-tasks").querySelectorAll(".task-item, .task-card").forEach((el) => {
    el.classList.remove("selected");
  }, 100);
  const selectedEl = document.getElementById("window-tasks").querySelector(`[data-pending-id="${pendingId}"]`);
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

  document.getElementById("window-tasks").querySelectorAll(".task-item, .task-card").forEach((el) => {
    el.classList.remove("selected");
  }, 100);
  const selectedEl = document.getElementById("window-tasks").querySelector(
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

  document.getElementById("window-tasks").querySelectorAll(".task-item, .task-card").forEach((el) => {
    el.classList.remove("selected");
  }, 100);
  const selectedEl = document.getElementById("window-tasks").querySelector(
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
if (tasksWindow) {
  tasksWindow.addEventListener("htmx:afterRequest", function (event) {
  if (event.detail.elt.id === "pending-fill-form" && event.detail.successful) {
    htmx.trigger(document.body, "taskCreated");
    document.getElementById("pending-fill-value").value = "";
    addAgentLog("success", "[OK] Pending info filled successfully");
  }
}, 100);
}

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
setTimeout(updateFilterCounts, 100);
if (tasksWindow) {
  tasksWindow.addEventListener("taskCreated", updateFilterCounts);
}

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

function submitNewIntent() {
  var form = document.getElementById("new-intent-form");
  if (!form) return;

  var intentInput = form.querySelector('[name="intent"]');
  if (!intentInput) return;

  var intent = intentInput.value;
  if (intent && intent.trim()) {
    var quickInput = document.getElementById("quick-intent-input");
    if (quickInput) {
      quickInput.value = intent;
    }
    var quickBtn = document.getElementById("quick-intent-btn");
    if (quickBtn && typeof htmx !== "undefined") {
      htmx.trigger(quickBtn, "click");
    }
    closeNewIntentModal();
  }
}

function skipDecision() {
  closeDecisionModal();
}

// =============================================================================
// TASK STATS LOADING
// =============================================================================

function loadTaskStats() {
  fetch("/api/tasks/stats/json")
    .then(function (response) {
      if (!response.ok) {
        throw new Error("Failed to fetch stats");
      }
      return response.json();
    })
    .then(function (stats) {
      var mappings = [
        { key: "complete", id: "count-complete" },
        { key: "completed", id: "count-complete" },
        { key: "active", id: "count-active" },
        { key: "awaiting", id: "count-awaiting" },
        { key: "paused", id: "count-paused" },
        { key: "blocked", id: "count-blocked" },
        { key: "time_saved", id: "time-saved-value" },
        { key: "total", id: "count-all" },
      ];

      mappings.forEach(function (mapping) {
        if (stats[mapping.key] !== undefined) {
          var el = document.getElementById(mapping.id);
          if (el) {
            el.textContent = stats[mapping.key];
          }
        }
      }, 100);
    })
    .catch(function (e) {
      console.warn("Failed to load stats:", e);
    }, 100);
}

// =============================================================================
// SPLITTER DRAG FUNCTIONALITY
// =============================================================================

(function initSplitter() {
  var splitter = document.getElementById("tasks-splitter");
  var main = document.getElementById("window-tasks").querySelector(".tasks-main");
  var leftPanel = document.getElementById("window-tasks").querySelector(".tasks-list-panel");

  if (!splitter || !main || !leftPanel) return;

  var isDragging = false;
  var startX = 0;
  var startWidth = 0;

  splitter.addEventListener("mousedown", function (e) {
    isDragging = true;
    startX = e.clientX;
    startWidth = leftPanel.offsetWidth;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    e.preventDefault();
  }, 100);

if (tasksWindow) {
  tasksWindow.addEventListener("mousemove", function (e) {
    if (!isDragging) return;

    var diff = e.clientX - startX;
    var newWidth = Math.max(200, Math.min(600, startWidth + diff));
    leftPanel.style.flex = "0 0 " + newWidth + "px";
    leftPanel.style.width = newWidth + "px";
  }, 100);
}

if (tasksWindow) {
  tasksWindow.addEventListener("mouseup", function () {
    if (isDragging) {
      isDragging = false;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    }
  }, 100);
}
})();

// =============================================================================
// HTMX TASK CREATION HANDLER
// =============================================================================

if (tasksWindow) {
  tasksWindow.addEventListener("htmx:afterRequest", function (evt) {
  if (!evt.detail.pathInfo) return;
  if (evt.detail.pathInfo.requestPath !== "/api/autotask/create") return;

  var xhr = evt.detail.xhr;
  var intentResult = document.getElementById("intent-result");
  if (!intentResult) return;

  if (xhr && xhr.status === 202) {
    try {
      var response = JSON.parse(xhr.responseText);
      if (response.success && response.task_id) {
        console.log("[TASK] Created task:", response.task_id);

        intentResult.innerHTML =
          '<span class="intent-success">✓ Task created - running...</span>';
        intentResult.style.display = "block";

        var quickInput = document.getElementById("quick-intent-input");
        if (quickInput) {
          quickInput.value = "";
        }

        selectTask(response.task_id);

        setTimeout(function () {
          intentResult.style.display = "none";
        }, 2000);
      } else {
        var msg = response.message || "Failed to create task";
        intentResult.innerHTML =
          '<span class="intent-error">✗ ' + escapeHtml(msg) + "</span>";
        intentResult.style.display = "block";
      }
    } catch (e) {
      console.warn("Failed to parse create response:", e);
      intentResult.innerHTML =
        '<span class="intent-error">✗ Failed to parse response</span>';
      intentResult.style.display = "block";
    }
  } else if (xhr && xhr.status >= 400) {
    try {
      var errorResponse = JSON.parse(xhr.responseText);
      var errorMsg =
        errorResponse.error || errorResponse.message || "Error creating task";
      intentResult.innerHTML =
        '<span class="intent-error">✗ ' + escapeHtml(errorMsg) + "</span>";
    } catch (e) {
      intentResult.innerHTML =
        '<span class="intent-error">✗ Error: ' + xhr.status + "</span>";
    }
    intentResult.style.display = "block";
  }
}, 100);
}

// =============================================================================
// FILTER PILL CLICK HANDLER
// =============================================================================

document.getElementById("window-tasks").querySelectorAll(".filter-pill").forEach(function (pill) {
  pill.addEventListener("click", function () {
    document.getElementById("window-tasks").querySelectorAll(".filter-pill").forEach(function (p) {
      p.classList.remove("active");
    }, 100);
    this.classList.add("active");
  }, 100);
}, 100);

// =============================================================================
// HTMX TASK LIST REFRESH HANDLER
// =============================================================================

if (tasksWindow) {
  tasksWindow.addEventListener("htmx:afterSwap", function (e) {
  if (e.detail.target && e.detail.target.id === "task-list") {
    loadTaskStats();
    var taskList = document.getElementById("task-list");
    var emptyState = document.getElementById("empty-state");
    if (taskList && emptyState) {
      var hasTasks = taskList.querySelector(".task-card");
      emptyState.style.display = hasTasks ? "none" : "flex";
    }
  }
}, 100);
}
