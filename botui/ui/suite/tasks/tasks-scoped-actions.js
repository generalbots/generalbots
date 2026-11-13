
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
    }, 100);
}

async function cancelTask(taskId) {
  const ok = typeof gbConfirm === "function"
    ? await gbConfirm("Are you sure you want to cancel this task?", "Cancel task")
    : (window.GBModal && typeof window.GBModal.confirm === "function"
      ? await new Promise((r) => window.GBModal.confirm("Are you sure you want to cancel this task?", "Cancel task", () => r(true), () => r(false)))
      : false);
  if (!ok) {
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
    }, 100);
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
    }, 100);
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
    <a href="${appUrl}" target="_blank" rel="noopener noreferrer" style="
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

// Goal/pending/scheduler/monitor detail selection is handled by tasks-toasts.js
// and tasks-scoped-selection.js with resilient DOM fallbacks.
