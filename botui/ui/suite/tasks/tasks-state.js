/* =============================================================================
   TASKS APP JAVASCRIPT
   Automated Intelligent Task Management Interface
   ============================================================================= */

// =============================================================================
// STATE MANAGEMENT
// =============================================================================

// Prevent duplicate declaration when script is reloaded via HTMX
if (typeof TasksState === "undefined") {
  var TasksState = {
    selectedTaskId: null, // No task selected initially
    currentFilter: "complete",
    tasks: [],
    wsConnection: null,
    agentLogPaused: false,
    selectedItemType: "task", // task, goal, pending, scheduler, monitor
    loadingTaskId: null, // Prevent multiple simultaneous loads
  };
}

// =============================================================================
// INITIALIZATION
// =============================================================================

(function(){ var __cb = function () {
  // Only init if tasks app is visible
  if (document.querySelector(".tasks-app")) {
    initTasksApp();
  }
}; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();

// Reinitialize when tasks page is loaded via HTMX
document.body.addEventListener("htmx:afterSwap", function (evt) {
  // Check if tasks app was just loaded
  if (evt.detail.target && evt.detail.target.id === "main-content") {
    if (document.querySelector(".tasks-app")) {
      console.log(
        "[Tasks] Detected tasks app loaded via HTMX, initializing...",
      );
      initTasksApp();
    }
  }
});

function initTasksApp() {
  // Only init WebSocket if not already connected
  if (
    !TasksState.wsConnection ||
    TasksState.wsConnection.readyState !== WebSocket.OPEN
  ) {
    initWebSocket();
  } else {
    console.log("[Tasks] WebSocket already connected, skipping init");
  }
  setupEventListeners();
  setupKeyboardShortcuts();
  setupIntentInputHandlers();
  setupHtmxListeners();
  scrollAgentLogToBottom();
  console.log("[Tasks] Initialized");
}

// Helper to find pending manifest by normalized ID
function findPendingManifest(taskId) {
  if (!taskId) return null;
  const normalizedId = String(taskId).toLowerCase().trim();

  // Lookup using normalized ID (all storage now uses normalized keys)
  if (pendingManifestUpdates.has(normalizedId)) {
    return {
      key: normalizedId,
      manifest: pendingManifestUpdates.get(normalizedId),
    };
  }
  return null;
}

function setupHtmxListeners() {
  // Listen for HTMX content swaps to apply pending manifest updates
  document.body.addEventListener("htmx:afterSwap", function (evt) {
    const target = evt.detail.target;
    if (
      target &&
      (target.id === "task-detail-content" ||
        target.closest("#task-detail-content"))
    ) {
      console.log(
        "[HTMX] Task detail content loaded, checking for pending manifest updates",
        "\n  selectedTaskId:",
        TasksState.selectedTaskId,
        "\n  pending keys:",
        Array.from(pendingManifestUpdates.keys()),
      );
      // Check if there's a pending manifest update for the selected task
      const pending = findPendingManifest(TasksState.selectedTaskId);
      if (pending) {
        console.log(
          "[HTMX] Applying pending manifest for task:",
          TasksState.selectedTaskId,
          "from key:",
          pending.key,
        );
        setTimeout(() => {
          renderManifestProgress(
            TasksState.selectedTaskId,
            pending.manifest,
            0,
          );
        }, 50);
      } else {
        console.log("[HTMX] No pending manifest found for selected task");
      }
    }
  });
}

function setupIntentInputHandlers() {
  const input = document.getElementById("quick-intent-input");
  const btn = document.getElementById("quick-intent-btn");

  if (input && btn) {
    input.addEventListener("keypress", function (e) {
      if (e.key === "Enter" && input.value.trim()) {
        e.preventDefault();
        htmx.trigger(btn, "click");
      }
    });
  }

  document.body.addEventListener("htmx:beforeRequest", function (e) {
    if (e.detail.elt.id === "quick-intent-btn") {
      const resultDiv = document.getElementById("intent-result");
      resultDiv.innerHTML = `
        <div class="result-card">
          <div class="result-message">Processing your request...</div>
          <div class="result-progress">
            <div class="result-progress-bar" style="width: 30%"></div>
          </div>
        </div>
      `;
    }
  });

  document.body.addEventListener("htmx:afterRequest", function (e) {
    if (e.detail.elt.id === "quick-intent-btn") {
      const resultDiv = document.getElementById("intent-result");
      try {
        const response = JSON.parse(e.detail.xhr.responseText);

        // Handle async task creation (status 202 Accepted)
        if (response.status === "running" && response.task_id) {
          // Clear input immediately
          document.getElementById("quick-intent-input").value = "";

          // Select the task to show progress in detail panel
          setTimeout(() => {
            selectTask(response.task_id);
          }, 500);

          // Clear result div - progress is shown in floating panel
          resultDiv.innerHTML = "";

          // Trigger task list refresh to show new task
          htmx.trigger(document.body, "taskCreated");

          // Start polling for task status
          startTaskPolling(response.task_id);

          return;
        }

        // Handle completed task (legacy sync response)
        if (response.success) {
          let html = `<div class="result-card">
            <div class="result-message result-success">✓ ${response.message || "Done!"}</div>`;

          if (response.app_url) {
            html += `<a href="${response.app_url}" class="result-link" target="_blank">
              Open App →
            </a>`;
          }

          if (response.task_id) {
            html += `<div style="margin-top:8px;color:#666;font-size:13px;">Task ID: ${response.task_id}</div>`;
          }

          html += `</div>`;
          resultDiv.innerHTML = html;

          document.getElementById("quick-intent-input").value = "";
          htmx.trigger(document.body, "taskCreated");
        } else {
          resultDiv.innerHTML = `<div class="result-card">
            <div class="result-message result-error">✗ ${response.error || response.message || "Something went wrong"}</div>
          </div>`;
        }
      } catch (err) {
        resultDiv.innerHTML = `<div class="result-card">
          <div class="result-message result-error">✗ Failed to process response</div>
        </div>`;
      }
    }
  });

  // Save intent text before submit for progress display
  if (input) {
    input.addEventListener("input", function () {
      input.setAttribute("data-last-intent", input.value);
    });
  }
}

// Task polling for async task creation
if (typeof activePollingTaskId === "undefined") {
  var activePollingTaskId = null;
  var pollingInterval = null;
}

function startTaskPolling(taskId) {
  // Stop any existing polling
  stopTaskPolling();

  activePollingTaskId = taskId;
  let pollCount = 0;
  const maxPolls = 180; // 3 minutes at 1 second intervals

  console.log(`[POLL] Starting polling for task ${taskId}`);

  pollingInterval = setInterval(async () => {
    pollCount++;

    if (pollCount > maxPolls) {
      console.log(`[POLL] Max polls reached for task ${taskId}`);
      stopTaskPolling();
      errorFloatingProgress("Task timed out");
      return;
    }

    try {
      const response = await fetch(`/api/tasks/${taskId}`, {
        headers: {
          Accept: "application/json",
        },
      });
      if (!response.ok) {
        console.error(`[POLL] Failed to fetch task status: ${response.status}`);
        return;
      }

      const task = await response.json();
      console.log(
        `[POLL] Task ${taskId} status: ${task.status}, progress: ${task.progress || 0}%`,
      );

      // Update progress
      const progress = task.progress || 0;
      const currentStep = task.current_step || 0;
      const totalSteps = task.total_steps || 100;
      const message = task.status || "Processing...";
      updateFloatingProgressBar(
        currentStep,
        totalSteps,
        message,
        "poll",
        null,
        null,
      );

      // Check if task is complete
      // Update the task card in-place without refreshing entire list
      updateTaskCardFromPoll(taskId, task);

      if (task.status === "completed" || task.status === "complete") {
        stopTaskPolling();
        completeFloatingProgress(task);
        updateFilterCounts(); // Just update counts, not full list
        showToast("Task completed successfully!", "success");
      } else if (task.status === "failed" || task.status === "error") {
        stopTaskPolling();
        errorFloatingProgress(task.error || "Task failed");
        updateFilterCounts(); // Just update counts, not full list
        showToast(task.error || "Task failed", "error");
      }
    } catch (err) {
      console.error(`[POLL] Error polling task ${taskId}:`, err);
    }
  }, 1000); // Poll every 1 second
}

function stopTaskPolling() {
  if (pollingInterval) {
    clearInterval(pollingInterval);
    pollingInterval = null;
  }
  activePollingTaskId = null;
}

// =============================================================================
// WEBSOCKET CONNECTION
// =============================================================================

// Global singleton WebSocket - shared across all task views
// This ensures only ONE WebSocket connection exists for task progress
if (typeof window._taskProgressWsConnection === "undefined") {
  window._taskProgressWsConnection = null;
  window._taskProgressWsHandlers = new Set();
}

function initWebSocket() {
  // Use global singleton to prevent multiple connections
  // Check if global connection already exists and is open/connecting
  if (window._taskProgressWsConnection) {
    const state = window._taskProgressWsConnection.readyState;
    if (state === WebSocket.OPEN || state === WebSocket.CONNECTING) {
      console.log(
        "[Tasks WS] Global WebSocket already connected/connecting, reusing",
      );
      // Just update local reference
      TasksState.wsConnection = window._taskProgressWsConnection;
      return;
    }
  }

  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  const wsUrl = `${protocol}//${window.location.host}/ws/task-progress`;

  console.log("[Tasks WS] Creating SINGLETON WebSocket connection to:", wsUrl);

  try {
    const ws = new WebSocket(wsUrl);
    window._taskProgressWsConnection = ws;
    TasksState.wsConnection = ws;

    ws.onopen = function () {
      console.log("[Tasks WS] WebSocket connected successfully (singleton)");
      addAgentLog("info", "[SYSTEM] Connected to task orchestrator");
    };

    ws.onmessage = function (event) {
      console.log("[Tasks WS] Raw message received:", event.data);
      try {
        const data = JSON.parse(event.data);
        console.log("[Tasks WS] Parsed message:", data.type, data);
        handleWebSocketMessage(data);

        // Also forward to any registered handlers (e.g., ProgressPanel)
        window._taskProgressWsHandlers.forEach((handler) => {
          try {
            handler(data);
          } catch (e) {
            console.error("[Tasks WS] Handler error:", e);
          }
        });
      } catch (e) {
        console.error("[Tasks WS] Failed to parse message:", e, event.data);
      }
    };

    ws.onclose = function (event) {
      console.log(
        "[Tasks WS] WebSocket disconnected, code:",
        event.code,
        "reason:",
        event.reason,
      );
      window._taskProgressWsConnection = null;
      TasksState.wsConnection = null;
      setTimeout(initWebSocket, 5000);
    };

    ws.onerror = function (error) {
      console.error("[Tasks WS] WebSocket error:", error);
    };
  } catch (e) {
    console.error("[Tasks WS] Failed to create WebSocket:", e);
  }
}

// Register a handler to receive WebSocket messages (for ProgressPanel etc.)
function registerTaskProgressHandler(handler) {
  window._taskProgressWsHandlers.add(handler);
  console.log(
    "[Tasks WS] Registered handler, total:",
    window._taskProgressWsHandlers.size,
  );
}

// Unregister a handler
function unregisterTaskProgressHandler(handler) {
  window._taskProgressWsHandlers.delete(handler);
  console.log(
    "[Tasks WS] Unregistered handler, total:",
    window._taskProgressWsHandlers.size,
  );
}
