
function selectDecision(element, value) {
  // Remove selected from all options
  document.querySelectorAll(".decision-option").forEach((opt) => {
    opt.classList.remove("selected");
  });

  // Add selected to clicked option
  element.classList.add("selected");

  // Store selected value
  TasksState.selectedDecision = value;

  addAgentLog("info", `[DECISION] Selected: ${value}`);
}

function submitDecision() {
  const selectedOption = document.querySelector(".decision-option.selected");
  if (!selectedOption) {
    showToast("Please select an option", "warning");
    return;
  }

  const value = TasksState.selectedDecision;
  const taskId = TasksState.selectedTaskId;

  addAgentLog("accent", `[AGENT] Applying decision: ${value}`);
  addAgentLog("info", `[TASK] Resuming task #${taskId}...`);

  // In real app, send to API
  fetch(`/api/tasks/${taskId}/decide`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ decision: value }),
  })
    .then((response) => response.json())
    .then((result) => {
      if (result.success) {
        showToast("Decision applied successfully", "success");
        addAgentLog("success", `[OK] Decision applied, task resuming`);

        // Hide decision section (in real app, would update via HTMX)
        const decisionSection = document.querySelector(
          ".decision-required-section",
        );
        if (decisionSection) {
          decisionSection.style.display = "none";
        }
      } else {
        showToast("Failed to apply decision", "error");
        addAgentLog(
          "error",
          `[ERROR] Failed to apply decision: ${result.error}`,
        );
      }
    })
    .catch((error) => {
      // For demo, simulate success
      showToast("Decision applied successfully", "success");
      addAgentLog("success", `[OK] Decision applied, task resuming`);

      const decisionSection = document.querySelector(
        ".decision-required-section",
      );
      if (decisionSection) {
        decisionSection.style.opacity = "0.5";
        setTimeout(() => {
          decisionSection.style.display = "none";
        }, 500);
      }

      // Update step status
      const activeStep = document.querySelector(".step-item.active");
      if (activeStep) {
        activeStep.classList.remove("active");
        activeStep.classList.add("completed");
        activeStep.querySelector(".step-icon").textContent = "✓";
        activeStep.querySelector(".step-detail").textContent =
          "Completed with merge strategy";

        const nextStep = activeStep.nextElementSibling;
        if (nextStep && nextStep.classList.contains("pending")) {
          nextStep.classList.remove("pending");
          nextStep.classList.add("active");
          nextStep.querySelector(".step-icon").textContent = "●";
          nextStep.querySelector(".step-time").textContent = "Now";
        }
      }
    });
}

function showDecisionRequired(decision) {
  addAgentLog("warning", `[ALERT] Decision required: ${decision.title}`);
  showToast(`Decision required: ${decision.title}`, "warning");
}

// =============================================================================
// PROGRESS LOG
// =============================================================================

function toggleProgressLog() {
  const stepList = document.querySelector(".step-list");
  const toggle = document.querySelector(".progress-log-toggle");

  if (stepList.style.display === "none") {
    stepList.style.display = "flex";
    toggle.textContent = "Collapse";
  } else {
    stepList.style.display = "none";
    toggle.textContent = "Expand";
  }
}

function updateStepProgress(taskId, step) {
  if (taskId !== TasksState.selectedTaskId) return;

  const stepItems = document.querySelectorAll(".step-item");
  stepItems.forEach((item, index) => {
    if (index < step.index) {
      item.classList.remove("active", "pending");
      item.classList.add("completed");
      item.querySelector(".step-icon").textContent = "✓";
    } else if (index === step.index) {
      item.classList.remove("completed", "pending");
      item.classList.add("active");
      item.querySelector(".step-icon").textContent = "●";
      item.querySelector(".step-name").textContent = step.name;
      item.querySelector(".step-detail").textContent = step.detail;
      item.querySelector(".step-time").textContent = "Now";
    } else {
      item.classList.remove("completed", "active");
      item.classList.add("pending");
      item.querySelector(".step-icon").textContent = "○";
    }
  });
}

// =============================================================================
// AGENT ACTIVITY LOG
// =============================================================================

function addAgentLog(level, message) {
  if (TasksState.agentLogPaused) return;

  const log = document.getElementById("agent-log");
  if (!log) return;

  const now = new Date();
  const timestamp = now.toTimeString().split(" ")[0].substring(0, 8);

  const line = document.createElement("div");
  line.className = `activity-line ${level}`;
  line.innerHTML = `
        <span class="activity-timestamp">${timestamp}</span>
        <span class="activity-message">${message}</span>
    `;

  // Insert at the top
  log.insertBefore(line, log.firstChild);

  // Limit log entries
  while (log.children.length > 100) {
    log.removeChild(log.lastChild);
  }
}

function scrollAgentLogToBottom() {
  const log = document.getElementById("agent-log");
  if (log) {
    log.scrollTop = 0; // Since newest is at top
  }
}

function clearAgentLog() {
  const log = document.getElementById("agent-log");
  if (log) {
    log.innerHTML = "";
    addAgentLog("info", "[SYSTEM] Log cleared");
  }
}

function toggleAgentLogPause() {
  TasksState.agentLogPaused = !TasksState.agentLogPaused;
  const pauseBtn = document.querySelector(".agent-activity-btn:last-child");
  if (pauseBtn) {
    pauseBtn.textContent = TasksState.agentLogPaused ? "Resume" : "Pause";
  }
  addAgentLog(
    "info",
    TasksState.agentLogPaused ? "[SYSTEM] Log paused" : "[SYSTEM] Log resumed",
  );
}

// =============================================================================
// TASK ACTIONS
// =============================================================================

function pauseTask(taskId) {
  addAgentLog("info", `[TASK] Pausing task #${taskId}...`);

  fetch(`/api/tasks/${taskId}/pause`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
  })
    .then((response) => response.json())
    .then((result) => {
      if (result.success) {
        showToast("Task paused", "success");
        addAgentLog("success", `[OK] Task #${taskId} paused`);
        htmx.trigger(document.body, "taskCreated");
        if (TasksState.selectedTaskId === taskId) {
          loadTaskDetails(taskId);
        }
      } else {
        showToast("Failed to pause task", "error");
        addAgentLog(
          "error",
          `[ERROR] Failed to pause task: ${result.error || result.message}`,
        );
      }
    })
    .catch((error) => {
      showToast("Failed to pause task", "error");
      addAgentLog("error", `[ERROR] Failed to pause task: ${error}`);
    });
}

function cancelTask(taskId) {
  if (!confirm("Are you sure you want to cancel this task?")) {
    return;
  }

  addAgentLog("info", `[TASK] Cancelling task #${taskId}...`);

  fetch(`/api/tasks/${taskId}/cancel`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
  })
    .then((response) => response.json())
    .then((result) => {
      if (result.success) {
        showToast("Task cancelled", "success");
        addAgentLog("success", `[OK] Task #${taskId} cancelled`);
        htmx.trigger(document.body, "taskCreated");
        if (TasksState.selectedTaskId === taskId) {
          loadTaskDetails(taskId);
        }
      } else {
        showToast("Failed to cancel task", "error");
        addAgentLog(
          "error",
          `[ERROR] Failed to cancel task: ${result.error || result.message}`,
        );
      }
    })
    .catch((error) => {
      showToast("Failed to cancel task", "error");
      addAgentLog("error", `[ERROR] Failed to cancel task: ${error}`);
    });
}

function showDetailedView(taskId) {
  addAgentLog("info", `[TASK] Opening detailed view for task #${taskId}...`);

  // For now, just reload the task details
  // In the future, this could open a modal or new page with more details
  loadTaskDetails(taskId);
  showToast("Detailed view loaded", "info");
}

// =============================================================================
// TASK LIFECYCLE
// =============================================================================

function onTaskCompleted(data, appUrl) {
  const title = data.title || data.message || "Task";
  const taskId = data.task_id || data.id;

  // Add to bell notifications using global GBAlerts infrastructure
  if (window.GBAlerts) {
    window.GBAlerts.taskCompleted(title, appUrl);
  }

  if (appUrl) {
    showToast(`App ready! Click to open: ${appUrl}`, "success", 10000, () => {
      window.open(appUrl, "_blank");
    });
    addAgentLog("success", `[COMPLETE] Task #${taskId}: ${title}`);
    addAgentLog("success", `[URL] ${appUrl}`);
  } else {
    showToast(`Task completed: ${title}`, "success");
    addAgentLog("success", `[COMPLETE] Task #${taskId}: ${title}`);
  }

  if (data.task) {
    updateTaskCard(data.task);
  }
}

function showAppUrlNotification(appUrl) {
  // Create a prominent notification for the app URL
  let notification = document.getElementById("app-url-notification");
  if (!notification) {
    notification = document.createElement("div");
    notification.id = "app-url-notification";
    notification.style.cssText = `
      position: fixed;
      top: 80px;
      right: 24px;
      background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%);
      color: white;
      padding: 16px 24px;
      border-radius: 12px;
      box-shadow: 0 8px 32px rgba(34, 197, 94, 0.4);
      z-index: 10001;
      max-width: 400px;
      animation: slideInRight 0.5s ease;
    `;
    document.body.appendChild(notification);
  }

  notification.innerHTML = `
    <div style="font-weight: 600; margin-bottom: 8px;">🎉 App Created Successfully!</div>
    <div style="font-size: 13px; opacity: 0.9; margin-bottom: 12px;">Your app is ready to use</div>
    <a href="${appUrl}" target="_blank" style="
      display: inline-block;
      background: white;
      color: #16a34a;
      padding: 8px 16px;
      border-radius: 6px;
      text-decoration: none;
      font-weight: 600;
      font-size: 14px;
    ">Open App →</a>
    <button onclick="this.parentElement.remove()" style="
      position: absolute;
      top: 8px;
      right: 8px;
      background: none;
      border: none;
      color: white;
      cursor: pointer;
      font-size: 18px;
      opacity: 0.7;
    ">×</button>
  `;

  // Auto-hide after 30 seconds
  setTimeout(() => {
    if (notification.parentElement) {
      notification.style.animation = "slideOutRight 0.5s ease forwards";
      setTimeout(() => notification.remove(), 500);
    }
  }, 30000);
}

function playCompletionSound() {
  try {
    // Create a simple beep sound using Web Audio API
    const audioCtx = new (window.AudioContext || window.webkitAudioContext)();
    const oscillator = audioCtx.createOscillator();
    const gainNode = audioCtx.createGain();

    oscillator.connect(gainNode);
    gainNode.connect(audioCtx.destination);

    oscillator.frequency.value = 800;
    oscillator.type = "sine";
    gainNode.gain.setValueAtTime(0.3, audioCtx.currentTime);
    gainNode.gain.exponentialRampToValueAtTime(
      0.01,
      audioCtx.currentTime + 0.5,
    );

    oscillator.start(audioCtx.currentTime);
    oscillator.stop(audioCtx.currentTime + 0.5);

    // Play a second higher tone for success feel
    setTimeout(() => {
      const osc2 = audioCtx.createOscillator();
      const gain2 = audioCtx.createGain();
      osc2.connect(gain2);
      gain2.connect(audioCtx.destination);
      osc2.frequency.value = 1200;
      osc2.type = "sine";
      gain2.gain.setValueAtTime(0.3, audioCtx.currentTime);
      gain2.gain.exponentialRampToValueAtTime(0.01, audioCtx.currentTime + 0.3);
      osc2.start(audioCtx.currentTime);
      osc2.stop(audioCtx.currentTime + 0.3);
    }, 150);
  } catch (e) {
    console.log("[Tasks] Could not play completion sound:", e);
  }
}

function onTaskFailed(task, error) {
  showToast(`Task failed: ${task.title}`, "error");
  addAgentLog("error", `[FAILED] Task #${task.id}: ${error}`);
  updateTaskCard(task);
}

// =============================================================================
// TOAST NOTIFICATIONS
// =============================================================================
