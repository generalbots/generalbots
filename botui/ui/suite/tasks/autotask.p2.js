
function renderIntentDetail(intent) {
  const panel = document.getElementById("intent-detail-panel");
  if (!panel) return;

  const statusIcons = {
    active: "⚡",
    complete: "✓",
    paused: "⏸",
    blocked: "⚠",
    awaiting: "⏳",
  };

  const healthClass =
    intent.health >= 80 ? "good" : intent.health >= 50 ? "warning" : "bad";

  panel.innerHTML = `
    <div class="detail-view">
      <!-- Header -->
      <div class="detail-header">
        <button class="btn-back" onclick="closeDetailPanel()">
          <span>◀</span>
        </button>
        <h2 class="detail-title">${escapeHtml(intent.title || intent.intent)}</h2>
        <div class="detail-status">
          <span class="status-icon">${statusIcons[intent.status] || "⚡"}</span>
          <span class="status-label">${intent.status_display || intent.status}</span>
        </div>
        <button class="btn-pause" onclick="togglePause('${intent.id}')">
          <span class="pause-icon">${intent.status === "paused" ? "▶" : "⏸"}</span>
        </button>
      </div>

      <!-- Progress Bar -->
      <div class="detail-progress">
        <div class="progress-bar large">
          <div class="progress-fill" style="width: ${intent.progress || 0}%"></div>
        </div>
        <div class="progress-stats">
          <span class="progress-steps">${intent.current_step || 0}/${intent.total_steps || 0} Steps</span>
          <span class="progress-percent">${intent.progress || 0}%</span>
        </div>
      </div>

      ${
        intent.decision_required
          ? `
      <!-- Decision Required Panel -->
      <div class="decision-panel">
        <div class="decision-header">
          <span class="decision-badge">DECISION REQUIRED</span>
          <span class="decision-title">${escapeHtml(intent.decision_title || "Decision needed")}</span>
        </div>
        <div class="decision-body">
          <div class="decision-actions">
            <button class="btn-decision-primary" onclick="openDecisionModal('${intent.id}')">
              Make Decision
            </button>
            <button class="btn-decision-secondary" onclick="viewDecisionContext('${intent.id}')">
              View Context Details
            </button>
          </div>
          <div class="decision-context">
            <div class="context-label">Context:</div>
            <p class="context-text">${escapeHtml(intent.decision_context || "")}</p>
          </div>
        </div>
      </div>
      `
          : ""
      }

      <!-- Status Section -->
      <div class="detail-section">
        <div class="section-header">
          <span class="section-title">STATUS</span>
          <span class="section-runtime">Runtime: ${intent.runtime || "0 min"}</span>
        </div>
        <div class="status-current-task">
          <span class="task-name">${escapeHtml(intent.current_task_name || "Processing...")}</span>
          <span class="task-estimate">Estimated: ${intent.estimated_time || "calculating..."}</span>
        </div>
        <div class="status-steps-preview">
          ${renderStepsPreview(intent.steps || [])}
        </div>
      </div>

      <!-- Progress Log Section -->
      <div class="detail-section">
        <div class="section-header">
          <span class="section-title">PROGRESS LOG</span>
        </div>
        <div class="progress-log" id="progress-log-${intent.id}">
          ${renderProgressLog(intent.logs || [])}
        </div>
      </div>

      <!-- Terminal Activity Section -->
      <div class="detail-section terminal-section">
        <div class="section-header">
          <span class="section-title">TERMINAL (LIVE AGENT ACTIVITY)</span>
          <span class="terminal-stats">
            Processed: <strong>${intent.processed_count || 0}</strong> data points /
            Processing speed: <strong>${intent.processing_speed || "~8 sources/s"}</strong> /
            Estimated completion: <strong>${intent.estimated_completion || "calculating"}</strong>
          </span>
        </div>
        <div class="terminal-output" id="terminal-${intent.id}">
          <div class="terminal-line">Initializing agent...</div>
        </div>
        <div class="terminal-cursor"></div>
      </div>
    </div>
  `;
}

function renderStepsPreview(steps) {
  if (!steps || steps.length === 0) {
    return '<div class="step-item pending"><span class="step-dot"></span><span class="step-name">No steps yet</span></div>';
  }

  return steps
    .slice(0, 3)
    .map(
      (step) => `
    <div class="step-item ${step.status || "pending"}">
      <span class="step-dot"></span>
      <span class="step-name">${escapeHtml(step.name)}</span>
      ${step.note ? `<span class="step-note">${escapeHtml(step.note)}</span>` : ""}
    </div>
  `,
    )
    .join("");
}

function renderProgressLog(logs) {
  if (!logs || logs.length === 0) {
    return '<div class="log-entry"><div class="log-entry-header"><span class="log-icon pending">●</span><span class="log-title">Waiting for activity...</span></div></div>';
  }

  return logs
    .map(
      (log) => `
    <div class="log-entry ${log.expanded ? "expanded" : ""}">
      <div class="log-entry-header" onclick="toggleLogEntry(this)">
        <span class="log-icon ${log.status || "pending"}">●</span>
        <span class="log-title">${escapeHtml(log.title)}</span>
        <span class="log-expand">▶</span>
        ${log.step_label ? `<span class="log-step-badge">${escapeHtml(log.step_label)}</span>` : ""}
      </div>
      <div class="log-entry-body">
        <div class="log-sub-entries">
          ${(log.sub_entries || [])
            .map(
              (sub) => `
            <div class="log-sub-entry">
              <span class="sub-icon ${sub.status || "complete"}">●</span>
              <span class="sub-text">${escapeHtml(sub.text)}</span>
              <span class="sub-duration">Duration: ${sub.duration || "< 1 min"}</span>
              <span class="sub-check">${sub.status === "complete" ? "✓" : ""}</span>
            </div>
          `,
            )
            .join("")}
        </div>
      </div>
    </div>
  `,
    )
    .join("");
}

function toggleLogEntry(header) {
  const entry = header.closest(".log-entry");
  if (entry) {
    entry.classList.toggle("expanded");
  }
}

function closeDetailPanel() {
  // Clean up ProgressPanel if active
  if (typeof ProgressPanel !== "undefined") {
    ProgressPanel.destroy();
  }

  AutoTaskState.selectedIntentId = null;

  document.querySelectorAll(".intent-card").forEach((card) => {
    card.classList.remove("selected");
  });

  const panel = document.getElementById("intent-detail-panel");
  if (panel) {
    panel.innerHTML = `
      <div class="detail-placeholder">
        <span class="placeholder-icon">📋</span>
        <p>Select an intent to view details</p>
      </div>
    `;
  }
}

function viewDetailedIntent(intentId) {
  selectIntent(intentId);
}

function togglePause(intentId) {
  const intent = AutoTaskState.intents.find((i) => i.id === intentId);
  if (!intent) return;

  const action = intent.status === "paused" ? "resume" : "pause";

  fetch(`/api/autotask/${intentId}/${action}`, { method: "POST" })
    .then((response) => response.json())
    .then((result) => {
      if (result.success) {
        showToast(`Intent ${action}d`, "success");
        refreshIntents();
        if (AutoTaskState.selectedIntentId === intentId) {
          loadIntentDetail(intentId);
        }
      } else {
        showToast(`Failed to ${action} intent`, "error");
      }
    })
    .catch((error) => {
      console.error(`Failed to ${action} intent:`, error);
      showToast(`Failed to ${action} intent`, "error");
    });
}

function formatTime(date) {
  return date.toLocaleTimeString("en-US", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  });
}

function escapeHtml(text) {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

function handleWebSocketMessage(data) {
  switch (data.type) {
    case "task_update":
    case "intent_update":
      updateIntentInList(data.task || data.intent);
      break;
    case "step_progress":
      updateStepProgress(data.taskId, data.step, data.progress);
      break;
    case "decision_required":
      showDecisionNotification(data.decision);
      break;
    case "approval_required":
      showApprovalNotification(data.approval);
      break;
    case "task_completed":
    case "intent_completed":
      onIntentCompleted(data.task || data.intent);
      break;
    case "task_failed":
    case "intent_failed":
      onIntentFailed(data.task || data.intent, data.error);
      break;
    case "stats_update":
      updateStatsFromData(data.stats);
      break;
  }
}

function updateIntentInList(intent) {
  const card = document.querySelector(
    `.intent-card[data-intent-id="${intent.id}"]`,
  );
  if (card) {
    // Update progress
    const progressFill = card.querySelector(".progress-fill");
    const progressSteps = card.querySelector(".progress-steps");
    const progressPct = card.querySelector(".progress-percent");

    if (progressFill) progressFill.style.width = `${intent.progress}%`;
    if (progressSteps)
      progressSteps.textContent = `${intent.current_step}/${intent.total_steps} Steps`;
    if (progressPct) progressPct.textContent = `${intent.progress}%`;

    // Update status indicator
    const statusIndicator = card.querySelector(".intent-status-indicator");
    if (statusIndicator) {
      statusIndicator.className = `intent-status-indicator ${intent.status}`;
    }
  }
}

function onIntentCompleted(intent) {
  showToast(`Intent completed: ${intent.title || intent.id}`, "success");
  updateIntentInList(intent);
  updateStats();
}

function onIntentFailed(intent, error) {
  showToast(`Intent failed: ${intent.title || intent.id} - ${error}`, "error");
  updateIntentInList(intent);
  updateStats();
}

// =============================================================================
// EVENT LISTENERS
// =============================================================================

function setupEventListeners() {
  // Intent form submission
  const intentForm = document.getElementById("intent-form");
  if (intentForm) {
    intentForm.addEventListener("htmx:afterSwap", function (event) {
      if (event.detail.target.id === "compilation-result") {
        onCompilationComplete(event);
      }
    });
  }

  // Task list updates
  const taskList = document.getElementById("task-list");
  if (taskList) {
    taskList.addEventListener("htmx:afterSwap", function () {
      updateStats();
      highlightPendingItems();
    });
  }

  // Expand log entries on details open
  document.addEventListener(
    "toggle",
    function (event) {
      if (
        event.target.classList.contains("execution-log") &&
        event.target.open
      ) {
        const taskId = event.target.closest(".autotask-item")?.dataset.taskId;
        if (taskId) {
          loadExecutionLogs(taskId);
        }
      }
    },
    true,
  );
}

function setupKeyboardShortcuts() {
  document.addEventListener("keydown", function (e) {
    // Alt + N: Focus on intent input
    if (e.altKey && e.key === "n") {
      e.preventDefault();
      document.getElementById("intent-input")?.focus();
    }

    // Alt + R: Refresh tasks
    if (e.altKey && e.key === "r") {
      e.preventDefault();
      refreshTasks();
    }

    // Escape: Close any open modal
    if (e.key === "Escape") {
      closeAllModals();
    }

    // Alt + 1-4: Switch filters
    if (e.altKey && e.key >= "1" && e.key <= "4") {
      e.preventDefault();
      const filters = ["all", "running", "approval", "decision"];
      const index = parseInt(e.key) - 1;
      const tabs = document.querySelectorAll(".filter-tab");
      if (tabs[index]) {
        tabs[index].click();
      }
    }
  });
}

// =============================================================================
// AUTO REFRESH
// =============================================================================

function startAutoRefresh() {
  // Refresh every 5 seconds
  AutoTaskState.refreshInterval = setInterval(function () {
    if (!document.hidden) {
      updateStats();
    }
  }, 5000);
}

function stopAutoRefresh() {
  if (AutoTaskState.refreshInterval) {
    clearInterval(AutoTaskState.refreshInterval);
    AutoTaskState.refreshInterval = null;
  }
}

// =============================================================================
// STATS MANAGEMENT
// =============================================================================

function updateStats() {
  fetch("/api/autotask/stats")
    .then((response) => response.json())
    .then((stats) => {
      updateStatsFromData(stats);
    })
    .catch((error) => {
      console.error("Failed to fetch stats:", error);
    });
}

function updateStatsFromData(stats) {
  // Sentient filter counts
  const countComplete = document.getElementById("count-complete");
  const countActive = document.getElementById("count-active");
  const countAwaiting = document.getElementById("count-awaiting");
  const countPaused = document.getElementById("count-paused");
  const countBlocked = document.getElementById("count-blocked");
  const timeSaved = document.getElementById("time-saved");

  if (countComplete) countComplete.textContent = stats.completed || 0;
  if (countActive) countActive.textContent = stats.running || stats.active || 0;
  if (countAwaiting)
    countAwaiting.textContent = stats.pending_decision || stats.awaiting || 0;
  if (countPaused) countPaused.textContent = stats.paused || 0;
  if (countBlocked)
    countBlocked.textContent = stats.blocked || stats.failed || 0;
  if (timeSaved) timeSaved.textContent = stats.time_saved || "0 hrs this week";

  // Legacy support
  const statRunning = document.getElementById("stat-running");
  const statPending = document.getElementById("stat-pending");
  const statCompleted = document.getElementById("stat-completed");
  const statApproval = document.getElementById("stat-approval");

  if (statRunning) statRunning.textContent = stats.running || 0;
  if (statPending) statPending.textContent = stats.pending || 0;
  if (statCompleted) statCompleted.textContent = stats.completed || 0;
  if (statApproval) statApproval.textContent = stats.pending_approval || 0;
}

// =============================================================================
