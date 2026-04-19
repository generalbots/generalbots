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

document.addEventListener("DOMContentLoaded", function () {
  // Only init if tasks app is visible
  if (document.querySelector(".tasks-app")) {
    initTasksApp();
  }
});

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

function handleWebSocketMessage(data) {
  console.log("[Tasks WS] handleWebSocketMessage called with type:", data.type);

  switch (data.type) {
    case "connected":
      console.log("[Tasks WS] Connected to task progress stream");
      addAgentLog("info", "[SYSTEM] Task progress stream connected");
      break;

    case "task_started":
      console.log("[Tasks WS] TASK_STARTED:", data.message);
      addAgentLog("accent", `[TASK] Started: ${data.message}`);
      // Update terminal in detail panel
      updateDetailTerminal(data.task_id, data.message, "started");
      // Add new task card to the list without full refresh
      if (data.task_id) {
        addTaskCardToList(data.task_id, data.message || "New Task", "running");
        selectTask(data.task_id);
      }
      break;

    case "task_progress":
      console.log(
        "[Tasks WS] TASK_PROGRESS - step:",
        data.step,
        "message:",
        data.message,
      );
      addAgentLog("info", `[${data.step}] ${data.message}`);

      // Auto-select this task if none selected
      if (data.task_id && !TasksState.selectedTaskId) {
        console.log(
          "[Tasks WS] Auto-selecting task from progress:",
          data.task_id,
        );
        TasksState.selectedTaskId = data.task_id;
        loadTaskDetails(data.task_id);
      }

      // Update STATUS section with current action
      updateStatusFromProgress(data.message, data.step);

      // Update terminal in detail panel with real data
      updateDetailTerminal(
        data.task_id,
        data.message,
        data.step,
        data.activity,
      );
      // Update progress bar in detail panel
      updateDetailProgress(
        data.task_id,
        data.current_step,
        data.total_steps,
        data.progress,
      );
      break;

    case "task_completed":
      console.log("[Tasks WS] TASK_COMPLETED:", data.message);
      addAgentLog("success", `[COMPLETE] ${data.message}`);

      // Extract app_url from details if present
      let appUrl = null;
      if (data.details && data.details.startsWith("app_url:")) {
        appUrl = data.details.substring(8);
        addAgentLog("success", `🚀 App URL: ${appUrl}`);
        showAppUrlNotification(appUrl);
      }

      // Update terminal with completion
      updateDetailTerminal(
        data.task_id,
        data.message,
        "complete",
        data.activity,
      );
      updateDetailProgress(
        data.task_id,
        data.total_steps,
        data.total_steps,
        100,
      );

      onTaskCompleted(data, appUrl);

      // Play completion sound
      playCompletionSound();

      // Update task card status in-place, then refresh list once
      if (data.task_id) {
        updateTaskCardStatus(data.task_id, "completed");
        setTimeout(() => {
          loadTaskDetails(data.task_id);
          // Trigger list refresh (throttled to 2s so won't flicker)
          if (typeof htmx !== "undefined") {
            htmx.trigger(document.body, "taskCreated");
          }
        }, 500);
      }
      break;

    case "task_error":
      console.log("[Tasks WS] TASK_ERROR:", data.error || data.message);
      addAgentLog("error", `[ERROR] ${data.error || data.message}`);
      updateDetailTerminal(data.task_id, data.error || data.message, "error");
      onTaskFailed(data, data.error);
      // Refresh task details to show error
      if (data.task_id) {
        setTimeout(() => loadTaskDetails(data.task_id), 500);
      }
      break;

    case "task_update":
      updateTaskCard(data.task);
      if (data.task && data.task.id === TasksState.selectedTaskId) {
        updateTaskDetail(data.task);
      }
      break;

    case "step_progress":
      updateStepProgress(data.taskId, data.step);
      break;

    case "agent_log":
      addAgentLog(data.level, data.message);
      break;

    case "decision_required":
      showDecisionRequired(data.decision);
      break;

    case "llm_stream":
      // Don't show raw LLM stream in terminal - it contains HTML/code garbage
      // Progress is shown via manifest_update events instead
      console.log("[Tasks WS] LLM streaming...");
      break;

    case "llm_generating":
      console.log("[Tasks WS] LLM_GENERATING:", data.message);
      addAgentLog("info", `[AI] ${data.message}`);
      // Update STATUS section with AI progress
      updateStatusFromProgress(data.message, "llm_generating");
      // Update terminal
      updateDetailTerminal(data.task_id, data.message, "llm_generating");
      break;

    case "llm_complete":
      console.log("[Tasks WS] LLM_COMPLETE:", data.message);
      addAgentLog("success", `[AI] ${data.message}`);
      // Update STATUS section
      updateStatusFromProgress(data.message, "llm_complete");
      // Update terminal
      updateDetailTerminal(data.task_id, `✓ ${data.message}`, "success");
      break;

    case "manifest_update":
      console.log(
        "[Tasks WS] *** MANIFEST_UPDATE RECEIVED ***",
        "\n  task_id:",
        data.task_id,
        "\n  selected:",
        TasksState.selectedTaskId,
        "\n  has details:",
        !!data.details,
        "\n  details length:",
        data.details?.length,
      );
      // Visual indicator in console
      console.warn(
        "[MANIFEST] Processing manifest_update for task:",
        data.task_id,
      );
      // Auto-select task if none selected or if this is a new running task
      if (data.task_id && !TasksState.selectedTaskId) {
        console.log("[Tasks WS] Auto-selecting task:", data.task_id);
        TasksState.selectedTaskId = data.task_id;
        loadTaskDetails(data.task_id);
      }
      // Update the progress log section with manifest data
      if (data.details) {
        try {
          const manifestData = JSON.parse(data.details);
          console.warn(
            "[MANIFEST] Parsed successfully:",
            "\n  sections:",
            manifestData.sections?.length,
            "\n  status:",
            manifestData.status,
            "\n  section names:",
            manifestData.sections
              ?.map((s) => s.name + ":" + s.status)
              .join(", "),
          );
          // Always render for the task, even if not selected (store for later)
          try {
            renderManifestProgress(data.task_id, manifestData, 0, true);
          } catch (renderError) {
            console.error(
              "[Tasks WS] Error in renderManifestProgress:",
              renderError,
              "\n  stack:",
              renderError.stack,
            );
          }
        } catch (e) {
          console.error(
            "[Tasks WS] Failed to parse manifest:",
            e,
            "\n  details preview:",
            data.details?.substring(0, 500),
          );
        }
      } else {
        console.warn(
          "[Tasks WS] manifest_update received but no details field",
        );
      }
      break;

    default:
      console.log(
        "[Tasks WS] Unhandled message type:",
        data.type,
        "\n  step:",
        data.step,
        "\n  message:",
        data.message,
      );
      break;
  }
}

// Store pending manifest updates for tasks whose elements aren't loaded yet
if (typeof pendingManifestUpdates === "undefined") {
  var pendingManifestUpdates = new Map();
}

function renderManifestProgress(
  taskId,
  manifest,
  retryCount = 0,
  forceStore = false,
) {
  // Normalize task IDs for comparison (both to lowercase string)
  const normalizedTaskId = String(taskId).toLowerCase().trim();
  const normalizedSelectedId = TasksState.selectedTaskId
    ? String(TasksState.selectedTaskId).toLowerCase().trim()
    : null;

  console.warn(
    "[MANIFEST] *** renderManifestProgress ***",
    "\n  taskId:",
    taskId,
    "\n  selectedTaskId:",
    TasksState.selectedTaskId,
    "\n  normalized match:",
    normalizedTaskId === normalizedSelectedId,
    "\n  sections:",
    manifest?.sections?.length,
    "\n  section statuses:",
    manifest?.sections?.map((s) => `${s.name}:${s.status}`).join(", "),
    "\n  retryCount:",
    retryCount,
  );

  // Always store the manifest for this task (use normalized ID for consistent lookup)
  pendingManifestUpdates.set(normalizedTaskId, manifest);

  // Only render UI if this is the selected task (use normalized comparison)
  if (normalizedSelectedId !== normalizedTaskId) {
    console.log(
      "[Manifest] Storing manifest but not rendering - not selected task",
      "\n  taskId:",
      normalizedTaskId,
      "\n  selectedId:",
      normalizedSelectedId,
    );
    return;
  }

  // Try multiple selectors to find the progress log element
  let progressLog = document.getElementById(`progress-log-${taskId}`);
  console.log(
    "[MANIFEST] Looking for progress-log-" + taskId + ", found:",
    !!progressLog,
  );
  if (!progressLog) {
    progressLog = document.querySelector(".taskmd-progress-content");
    console.log(
      "[MANIFEST] Looking for .taskmd-progress-content, found:",
      !!progressLog,
    );
  }

  if (!progressLog) {
    console.warn(
      "[MANIFEST] No progress log element found, retry:",
      retryCount,
    );
    // If task is selected but element not yet loaded, retry after a delay
    if (retryCount < 5) {
      pendingManifestUpdates.set(normalizedTaskId, manifest);
      setTimeout(
        () => {
          const pending = pendingManifestUpdates.get(normalizedTaskId);
          const currentSelectedNormalized = TasksState.selectedTaskId
            ? String(TasksState.selectedTaskId).toLowerCase().trim()
            : null;
          if (pending && currentSelectedNormalized === normalizedTaskId) {
            renderManifestProgress(taskId, pending, retryCount + 1);
          }
        },
        150 * (retryCount + 1),
      );
    }
    return;
  }

  // Clear pending update (use normalized ID)
  pendingManifestUpdates.delete(normalizedTaskId);

  if (!manifest || !manifest.sections) {
    console.log("[Manifest] No sections in manifest, skipping render");
    return;
  }

  const totalSteps = manifest.progress?.total || 60;

  console.warn(
    "[MANIFEST] Rendering progress tree:",
    "\n  totalSteps:",
    totalSteps,
    "\n  sections:",
    manifest.sections.length,
    "\n  progressLog element:",
    progressLog?.id || progressLog?.className,
  );

  // Update STATUS section if exists
  updateStatusSection(manifest);

  // Check if tree exists - if not, create it; if yes, update incrementally
  // Clear any "progress-empty" placeholder first
  const emptyPlaceholder = progressLog.querySelector(".progress-empty");
  if (emptyPlaceholder) {
    console.log("[Manifest] Removing progress-empty placeholder");
    emptyPlaceholder.remove();
  }

  let tree = progressLog.querySelector(".taskmd-tree");
  console.log("[Manifest] Existing tree found:", !!tree);

  // Check if we need to rebuild the tree (structure changed significantly)
  let shouldRebuild = !tree;
  if (tree && manifest.sections) {
    const existingSections = tree.querySelectorAll(".tree-section");
    const existingChildren = tree.querySelectorAll(".tree-child");
    const existingItems = tree.querySelectorAll(".tree-item");
    const newChildCount = manifest.sections.reduce(
      (sum, s) => sum + (s.children?.length || 0),
      0,
    );
    const newItemCount = manifest.sections.reduce((sum, s) => {
      let count = (s.items?.length || 0) + (s.item_groups?.length || 0);
      for (const child of s.children || []) {
        count += (child.items?.length || 0) + (child.item_groups?.length || 0);
      }
      return sum + count;
    }, 0);

    // Check if section names match (IDs may change but names should be stable)
    const existingNames = new Set(
      Array.from(existingSections).map(
        (el) => el.querySelector(".tree-name")?.textContent,
      ),
    );
    const newNames = new Set(manifest.sections.map((s) => s.name));
    const namesMatch =
      existingNames.size === newNames.size &&
      [...existingNames].every((n) => newNames.has(n));

    // Rebuild if:
    // - section count changed
    // - children appeared where there were none
    // - items appeared where there were none
    // - section names don't match (structure completely different)
    if (
      existingSections.length !== manifest.sections.length ||
      (existingChildren.length === 0 && newChildCount > 0) ||
      (existingItems.length === 0 && newItemCount > 0) ||
      !namesMatch
    ) {
      console.log(
        "[Manifest] Structure changed significantly, REBUILDING tree",
        "\n  existing sections:",
        existingSections.length,
        "-> new:",
        manifest.sections.length,
        "\n  existing children:",
        existingChildren.length,
        "-> new:",
        newChildCount,
        "\n  existing items:",
        existingItems.length,
        "-> new:",
        newItemCount,
        "\n  names match:",
        namesMatch,
      );
      shouldRebuild = true;
    }
  }

  try {
    if (shouldRebuild) {
      // Full rebuild - create the tree structure from scratch
      console.log("[Manifest] === REBUILDING TREE FROM SCRATCH ===");
      const treeHTML = buildProgressTreeHTML(manifest, totalSteps);
      console.log(
        "[Manifest] Tree HTML length:",
        treeHTML.length,
        "sections:",
        manifest.sections.length,
        "with children:",
        manifest.sections.filter((s) => s.children?.length > 0).length,
      );
      progressLog.innerHTML = treeHTML;
      // Auto-expand all sections and children for visibility
      progressLog
        .querySelectorAll(".tree-section, .tree-child")
        .forEach((el) => {
          el.classList.add("expanded");
        });
      const expandedCount = progressLog.querySelectorAll(".expanded").length;
      const childCount = progressLog.querySelectorAll(".tree-child").length;
      const itemCount = progressLog.querySelectorAll(".tree-item").length;
      console.log(
        "[Manifest] Tree rebuilt:",
        expandedCount,
        "expanded,",
        childCount,
        "children,",
        itemCount,
        "items",
      );
      // Auto-scroll to running item
      scrollToRunningItem(progressLog);
    } else {
      // Incremental update - only update what changed (no flicker)
      console.log("[Manifest] Updating tree in place");
      updateProgressTreeInPlace(tree, manifest, totalSteps);
      console.log("[Manifest] Tree updated successfully");
      // Auto-scroll to running item
      scrollToRunningItem(progressLog);
    }
  } catch (treeError) {
    console.error(
      "[Manifest] Error building/updating tree:",
      treeError,
      "\n  stack:",
      treeError.stack,
    );
  }

  // Update terminal stats
  updateTerminalStats(taskId, manifest);
}

// Auto-scroll progress log to show the currently running item
function scrollToRunningItem(progressLog) {
  if (!progressLog) return;

  // Find the running item (highest priority) or running section
  const runningItem = progressLog.querySelector(".tree-item.running");
  const runningSection = progressLog.querySelector(".tree-section.running");
  const runningChild = progressLog.querySelector(".tree-child.running");

  const targetElement = runningItem || runningChild || runningSection;

  if (targetElement) {
    // Scroll the target into view within the progress log container
    targetElement.scrollIntoView({
      behavior: "smooth",
      block: "center",
      inline: "nearest",
    });
  }
}

// Update STATUS section from task_progress messages
function updateStatusFromProgress(message, step) {
  const statusContent = document.querySelector(".taskmd-status-content");
  if (!statusContent) return;

  // Update current action text
  const actionText = statusContent.querySelector(
    ".status-current .status-text",
  );
  if (actionText && message) {
    actionText.textContent = message;
  }

  // Update the status dot to show activity
  const statusDot = statusContent.querySelector(".status-dot");
  if (statusDot) {
    statusDot.classList.add("active");
  }
}

function updateStatusSection(manifest) {
  const statusContent = document.querySelector(".taskmd-status-content");
  if (!statusContent) return;

  // Update current action text only if changed
  const actionText = statusContent.querySelector(
    ".status-current .status-text",
  );
  const currentAction =
    manifest.status?.current_action ||
    manifest.current_status?.current_action ||
    "Processing...";
  if (actionText && actionText.textContent !== currentAction) {
    actionText.textContent = currentAction;
  }

  // Update runtime text only
  const runtimeEl = statusContent.querySelector(".status-main .status-time");
  const runtime =
    manifest.status?.runtime_display || manifest.runtime || "Not started";
  if (runtimeEl) {
    // Only update text content, preserve indicator
    const indicator = runtimeEl.querySelector(".status-indicator");
    if (!indicator) {
      runtimeEl.innerHTML = `Runtime: ${runtime} <span class="status-indicator"></span>`;
    } else {
      runtimeEl.firstChild.textContent = `Runtime: ${runtime} `;
    }
  }

  // Update estimated text only
  const estimatedEl = statusContent.querySelector(
    ".status-current .status-time",
  );
  const estimated =
    manifest.status?.estimated_display ||
    (manifest.estimated_seconds
      ? `${manifest.estimated_seconds} sec`
      : "calculating...");
  if (estimatedEl) {
    const gear = estimatedEl.querySelector(".status-gear");
    if (!gear) {
      estimatedEl.innerHTML = `Estimated: ${estimated} <span class="status-gear">⚙</span>`;
    } else {
      estimatedEl.firstChild.textContent = `Estimated: ${estimated} `;
    }
  }
}

function buildProgressTreeHTML(manifest, totalSteps) {
  // Detailed logging to debug children/items
  const totalChildren = manifest.sections?.reduce(
    (sum, s) => sum + (s.children?.length || 0),
    0,
  );
  const totalItems = manifest.sections?.reduce((sum, s) => {
    let count = (s.items?.length || 0) + (s.item_groups?.length || 0);
    for (const c of s.children || []) {
      count += (c.items?.length || 0) + (c.item_groups?.length || 0);
    }
    return sum + count;
  }, 0);

  console.warn(
    "[BUILD_TREE] *** buildProgressTreeHTML ***",
    "\n  sections:",
    manifest.sections?.length,
    "\n  totalChildren:",
    totalChildren,
    "\n  totalItems:",
    totalItems,
    "\n  totalSteps:",
    totalSteps,
  );

  let html = '<div class="taskmd-tree">';

  for (const section of manifest.sections) {
    // Normalize status - backend sends "Running", "Completed", etc.
    const rawStatus = section.status || "Pending";
    const statusClass = rawStatus.toLowerCase();
    // ALWAYS expand all sections by default for visibility
    const shouldExpand = true;
    const globalCurrent =
      section.progress?.global_current || section.progress?.current || 0;

    const sectionChildCount = section.children?.length || 0;
    const sectionItemCount =
      (section.items?.length || 0) + (section.item_groups?.length || 0);

    console.log(
      "[BUILD_TREE] Section:",
      section.name,
      "| status:",
      rawStatus,
      "| children:",
      sectionChildCount,
      "| items:",
      sectionItemCount,
    );

    html += `
      <div class="tree-section ${statusClass}${shouldExpand ? " expanded" : ""}" data-section-id="${section.id}">
        <div class="tree-row tree-level-0" onclick="this.parentElement.classList.toggle('expanded')">
          <span class="tree-name">${escapeHtml(section.name)}</span>
          <span class="tree-step-badge">Step ${globalCurrent}/${totalSteps}</span>
          <span class="tree-status ${statusClass}">${rawStatus}</span>
          <span class="tree-section-dot ${statusClass}"></span>
        </div>
        <div class="tree-children">`;

    // Children (e.g., "Database Schema Design" under "Database & Models")
    if (section.children && section.children.length > 0) {
      console.log(
        "[BUILD_TREE]   -> Adding",
        section.children.length,
        "children to section",
        section.name,
      );
      for (const child of section.children) {
        const childRawStatus = child.status || "Pending";
        const childStatus = childRawStatus.toLowerCase();
        // ALWAYS expand all children by default for visibility
        const childShouldExpand = true;

        const childItemCount =
          (child.item_groups?.length || 0) + (child.items?.length || 0);
        console.log(
          "[BUILD_TREE]     Child:",
          child.name,
          "| status:",
          childRawStatus,
          "| items:",
          childItemCount,
        );

        html += `
          <div class="tree-child ${childStatus}${childShouldExpand ? " expanded" : ""}" data-child-id="${child.id}">
            <div class="tree-row tree-level-1" onclick="this.parentElement.classList.toggle('expanded')">
              <span class="tree-item-dot ${childStatus}"></span>
              <span class="tree-name">${escapeHtml(child.name)}</span>
              <span class="tree-step-badge">Step ${child.progress?.current || 0}/${child.progress?.total || 1}</span>
              <span class="tree-status ${childStatus}">${childRawStatus}</span>
            </div>
            <div class="tree-items">`;

        // Items within child (e.g., "email, password_hash, email_verified")
        const childItems = [
          ...(child.item_groups || []),
          ...(child.items || []),
        ];
        if (childItems.length > 0) {
          console.log(
            "[BUILD_TREE]       -> Adding",
            childItems.length,
            "items to child",
            child.name,
          );
        }
        for (const item of childItems) {
          html += buildItemHTML(item);
        }

        html += `</div></div>`;
      }
    }

    // Section-level items (items directly under section, not in children)
    const sectionItems = [
      ...(section.item_groups || []),
      ...(section.items || []),
    ];
    for (const item of sectionItems) {
      html += buildItemHTML(item);
    }

    html += `</div></div>`;
  }

  html += "</div>";

  // Final verification
  const hasChildren = html.includes("tree-child");
  const hasItems = html.includes("tree-item");
  console.warn(
    "[BUILD_TREE] *** Tree HTML built ***",
    "\n  length:",
    html.length,
    "\n  hasChildren:",
    hasChildren,
    "\n  hasItems:",
    hasItems,
  );

  return html;
}

function buildItemHTML(item) {
  const status = item.status?.toLowerCase() || "pending";
  const checkIcon = status === "completed" ? "✓" : "";
  const duration = item.duration_seconds
    ? item.duration_seconds >= 60
      ? `Duration: ${Math.floor(item.duration_seconds / 60)} min`
      : `Duration: ${item.duration_seconds} sec`
    : "";
  const name = item.name || item.display_name || "";

  return `
    <div class="tree-item ${status}" data-item-id="${item.id || name}">
      <span class="tree-item-dot ${status}"></span>
      <span class="tree-item-name">${escapeHtml(name)}</span>
      <span class="tree-item-duration">${duration}</span>
      <span class="tree-item-check ${status}">${checkIcon}</span>
    </div>`;
}

// Incremental update - only change what's different (prevents flicker)
function updateProgressTreeInPlace(tree, manifest, totalSteps) {
  for (const section of manifest.sections) {
    let sectionEl = tree.querySelector(`[data-section-id="${section.id}"]`);

    // If section not found by ID, try to find by name (backend may regenerate IDs)
    if (!sectionEl) {
      const allSections = tree.querySelectorAll(".tree-section");
      for (const el of allSections) {
        const nameEl = el.querySelector(":scope > .tree-row .tree-name");
        if (nameEl && nameEl.textContent === section.name) {
          sectionEl = el;
          // Update the data-section-id to the new ID for future lookups
          sectionEl.setAttribute("data-section-id", section.id);
          console.log(
            "[Manifest] Found section by name, updated ID:",
            section.name,
            "->",
            section.id,
          );
          break;
        }
      }
    }

    // If section still doesn't exist, create it dynamically (new section arrived!)
    if (!sectionEl) {
      console.log("[Manifest] Creating new section:", section.name);
      const rawStatus = section.status || "Pending";
      const statusClass = rawStatus.toLowerCase();
      const globalCurrent =
        section.progress?.global_current || section.progress?.current || 0;

      const sectionHtml = `
        <div class="tree-section ${statusClass} expanded" data-section-id="${section.id}">
          <div class="tree-row tree-level-0" onclick="this.parentElement.classList.toggle('expanded')">
            <span class="tree-name">${escapeHtml(section.name)}</span>
            <span class="tree-step-badge">Step ${globalCurrent}/${totalSteps}</span>
            <span class="tree-status ${statusClass}">${rawStatus}</span>
            <span class="tree-section-dot ${statusClass}"></span>
          </div>
          <div class="tree-children"></div>
        </div>`;

      tree.insertAdjacentHTML("beforeend", sectionHtml);
      sectionEl = tree.querySelector(`[data-section-id="${section.id}"]`);
    }

    const rawStatus = section.status || "Pending";
    const statusClass = rawStatus.toLowerCase();
    const globalCurrent =
      section.progress?.global_current || section.progress?.current || 0;
    const isExpanded = sectionEl.classList.contains("expanded");

    // ALWAYS keep sections expanded for visibility
    const shouldExpand = true;
    const newClasses = `tree-section ${statusClass}${shouldExpand ? " expanded" : ""}`;
    if (sectionEl.className !== newClasses) {
      sectionEl.className = newClasses;
    }

    // Update step badge text only if changed
    const stepBadge = sectionEl.querySelector(
      ":scope > .tree-row .tree-step-badge",
    );
    const stepText = `Step ${globalCurrent}/${totalSteps}`;
    if (stepBadge && stepBadge.textContent !== stepText) {
      stepBadge.textContent = stepText;
    }

    // Update status text and class only if changed
    const statusEl = sectionEl.querySelector(":scope > .tree-row .tree-status");
    if (statusEl) {
      if (statusEl.textContent !== rawStatus) {
        statusEl.textContent = rawStatus;
      }
      const statusClasses = `tree-status ${statusClass}`;
      if (statusEl.className !== statusClasses) {
        statusEl.className = statusClasses;
      }
    }

    // Update section dot
    const sectionDot = sectionEl.querySelector(
      ":scope > .tree-row .tree-section-dot",
    );
    if (sectionDot) {
      const dotClasses = `tree-section-dot ${statusClass}`;
      if (sectionDot.className !== dotClasses) {
        sectionDot.className = dotClasses;
      }
    }

    // Update children
    if (section.children) {
      for (const child of section.children) {
        updateChildInPlace(sectionEl, child);
      }
    }

    // Update section-level items
    const childrenContainer = sectionEl.querySelector(".tree-children");
    if (childrenContainer) {
      updateItemsInPlace(childrenContainer, [
        ...(section.item_groups || []),
        ...(section.items || []),
      ]);
    }
  }
}

function updateChildInPlace(sectionEl, child) {
  let childEl = sectionEl.querySelector(`[data-child-id="${child.id}"]`);

  // If child not found by ID, try to find by name (backend may regenerate IDs)
  if (!childEl) {
    const allChildren = sectionEl.querySelectorAll(".tree-child");
    for (const el of allChildren) {
      const nameEl = el.querySelector(":scope > .tree-row .tree-name");
      if (nameEl && nameEl.textContent === child.name) {
        childEl = el;
        // Update the data-child-id to the new ID for future lookups
        childEl.setAttribute("data-child-id", child.id);
        console.log(
          "[Manifest] Found child by name, updated ID:",
          child.name,
          "->",
          child.id,
        );
        break;
      }
    }
  }

  // If child still doesn't exist in DOM, create it (and auto-expand new children!)
  if (!childEl) {
    const childrenContainer = sectionEl.querySelector(".tree-children");
    if (!childrenContainer) return;

    const rawStatus = child.status || "Pending";
    const statusClass = rawStatus.toLowerCase();
    // NEW: Always expand newly created children so they're visible immediately
    const childHasItems =
      (child.item_groups?.length || 0) + (child.items?.length || 0) > 0;
    const shouldExpand = true; // Always expand new children for visibility

    console.log(
      "[Manifest] Creating new child:",
      child.name,
      "status:",
      rawStatus,
      "expanded:",
      shouldExpand,
    );

    const childHtml = `
      <div class="tree-child ${statusClass}${shouldExpand ? " expanded" : ""}" data-child-id="${child.id}">
        <div class="tree-row tree-level-1" onclick="this.parentElement.classList.toggle('expanded')">
          <span class="tree-item-dot ${statusClass}"></span>
          <span class="tree-name">${escapeHtml(child.name)}</span>
          <span class="tree-step-badge">Step ${child.progress?.current || 0}/${child.progress?.total || 1}</span>
          <span class="tree-status ${statusClass}">${rawStatus}</span>
        </div>
        <div class="tree-items"></div>
      </div>`;

    childrenContainer.insertAdjacentHTML("beforeend", childHtml);
    childEl = sectionEl.querySelector(`[data-child-id="${child.id}"]`);

    // Add items to the newly created child
    const itemsContainer = childEl.querySelector(".tree-items");
    if (itemsContainer) {
      const allItems = [...(child.item_groups || []), ...(child.items || [])];
      for (const item of allItems) {
        itemsContainer.insertAdjacentHTML("beforeend", buildItemHTML(item));
      }
    }
    return;
  }

  const rawStatus = child.status || "Pending";
  const statusClass = rawStatus.toLowerCase();
  const isExpanded = childEl.classList.contains("expanded");

  // ALWAYS keep children expanded for visibility
  const shouldExpand = true;
  const newClasses = `tree-child ${statusClass}${shouldExpand ? " expanded" : ""}`;
  if (childEl.className !== newClasses) {
    childEl.className = newClasses;
  }

  // Update step badge
  const stepBadge = childEl.querySelector(
    ":scope > .tree-row .tree-step-badge",
  );
  const stepText = `Step ${child.progress?.current || 0}/${child.progress?.total || 1}`;
  if (stepBadge && stepBadge.textContent !== stepText) {
    stepBadge.textContent = stepText;
  }

  // Update status
  const statusEl = childEl.querySelector(":scope > .tree-row .tree-status");
  if (statusEl) {
    if (statusEl.textContent !== rawStatus) {
      statusEl.textContent = rawStatus;
    }
    const statusClasses = `tree-status ${statusClass}`;
    if (statusEl.className !== statusClasses) {
      statusEl.className = statusClasses;
    }
  }

  // Update child dot
  const childDot = childEl.querySelector(":scope > .tree-row .tree-item-dot");
  if (childDot) {
    const dotClasses = `tree-item-dot ${statusClass}`;
    if (childDot.className !== dotClasses) {
      childDot.className = dotClasses;
    }
  }

  // Update items within child
  const itemsContainer = childEl.querySelector(".tree-items");
  if (itemsContainer) {
    updateItemsInPlace(itemsContainer, [
      ...(child.item_groups || []),
      ...(child.items || []),
    ]);
  }
}

function updateItemsInPlace(container, items) {
  if (!container || !items) return;

  for (const item of items) {
    const itemId = item.id || item.name || item.display_name;
    const itemName = item.name || item.display_name;
    let itemEl = container.querySelector(`[data-item-id="${itemId}"]`);

    // If item not found by ID, try to find by name (backend may regenerate IDs)
    if (!itemEl && itemName) {
      const allItems = container.querySelectorAll(".tree-item");
      for (const el of allItems) {
        const nameEl = el.querySelector(".tree-item-name");
        if (nameEl && nameEl.textContent === itemName) {
          itemEl = el;
          // Update the data-item-id to the new ID for future lookups
          itemEl.setAttribute("data-item-id", itemId);
          break;
        }
      }
    }

    if (!itemEl) {
      // New item - append it
      container.insertAdjacentHTML("beforeend", buildItemHTML(item));
      continue;
    }

    const rawStatus = item.status || "Pending";
    const status = rawStatus.toLowerCase();

    // Update item class
    const newClasses = `tree-item ${status}`;
    if (itemEl.className !== newClasses) {
      itemEl.className = newClasses;
    }

    // Update dot
    const dot = itemEl.querySelector(".tree-item-dot");
    if (dot) {
      const dotClasses = `tree-item-dot ${status}`;
      if (dot.className !== dotClasses) {
        dot.className = dotClasses;
      }
    }

    // Update check
    const check = itemEl.querySelector(".tree-item-check");
    if (check) {
      const checkClasses = `tree-item-check ${status}`;
      if (check.className !== checkClasses) {
        check.className = checkClasses;
      }
      const checkText = status === "completed" ? "✓" : "";
      if (check.textContent !== checkText) {
        check.textContent = checkText;
      }
    }

    // Update duration
    const durationEl = itemEl.querySelector(".tree-item-duration");
    if (durationEl && item.duration_seconds) {
      const durationText =
        item.duration_seconds >= 60
          ? `Duration: ${Math.floor(item.duration_seconds / 60)} min`
          : `Duration: ${item.duration_seconds} sec`;
      if (durationEl.textContent !== durationText) {
        durationEl.textContent = durationText;
      }
    }
  }
}

function updateTerminalStats(taskId, manifest) {
  const processedEl = document.getElementById(`terminal-processed-${taskId}`);
  if (processedEl && manifest.terminal?.stats?.processed) {
    processedEl.textContent = manifest.terminal.stats.processed;
  }

  const etaEl = document.getElementById(`terminal-eta-${taskId}`);
  if (etaEl && manifest.terminal?.stats?.eta) {
    etaEl.textContent = manifest.terminal.stats.eta;
  }
}

function escapeHtml(text) {
  if (!text) return "";
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

function updateActivityMetrics(activity) {
  if (!activity) return;

  const metricsEl = document.getElementById("floating-activity-metrics");
  if (!metricsEl) return;

  let html = "";

  if (activity.phase) {
    html += `<div class="metric-row"><span class="metric-label">Phase:</span> <span class="metric-value phase-${activity.phase}">${activity.phase.toUpperCase()}</span></div>`;
  }

  if (activity.items_processed !== undefined) {
    const total = activity.items_total ? `/${activity.items_total}` : "";
    html += `<div class="metric-row"><span class="metric-label">Processed:</span> <span class="metric-value">${activity.items_processed}${total} items</span></div>`;
  }

  if (activity.speed_per_min) {
    html += `<div class="metric-row"><span class="metric-label">Speed:</span> <span class="metric-value">~${activity.speed_per_min.toFixed(1)} items/min</span></div>`;
  }

  if (activity.eta_seconds) {
    const mins = Math.floor(activity.eta_seconds / 60);
    const secs = activity.eta_seconds % 60;
    const eta = mins > 0 ? `${mins}m ${secs}s` : `${secs}s`;
    html += `<div class="metric-row"><span class="metric-label">ETA:</span> <span class="metric-value">${eta}</span></div>`;
  }

  if (activity.bytes_processed) {
    const kb = (activity.bytes_processed / 1024).toFixed(1);
    html += `<div class="metric-row"><span class="metric-label">Generated:</span> <span class="metric-value">${kb} KB</span></div>`;
  }

  if (activity.tokens_used) {
    html += `<div class="metric-row"><span class="metric-label">Tokens:</span> <span class="metric-value">${activity.tokens_used.toLocaleString()}</span></div>`;
  }

  if (activity.files_created && activity.files_created.length > 0) {
    html += `<div class="metric-row"><span class="metric-label">Files:</span> <span class="metric-value">${activity.files_created.length} created</span></div>`;
  }

  if (activity.tables_created && activity.tables_created.length > 0) {
    html += `<div class="metric-row"><span class="metric-label">Tables:</span> <span class="metric-value">${activity.tables_created.length} synced</span></div>`;
  }

  if (activity.current_item) {
    html += `<div class="metric-row current-item"><span class="metric-label">Current:</span> <span class="metric-value">${activity.current_item}</span></div>`;
  }

  metricsEl.innerHTML = html;
}

function logFinalStats(activity) {
  if (!activity) return;

  addAgentLog("info", "─────────────────────────────────");
  addAgentLog("info", "GENERATION COMPLETE");

  if (activity.files_created && activity.files_created.length > 0) {
    addAgentLog("success", `Files created: ${activity.files_created.length}`);
    activity.files_created.forEach((f) => addAgentLog("info", `  • ${f}`));
  }

  if (activity.tables_created && activity.tables_created.length > 0) {
    addAgentLog("success", `Tables synced: ${activity.tables_created.length}`);
    activity.tables_created.forEach((t) => addAgentLog("info", `  • ${t}`));
  }

  if (activity.bytes_processed) {
    const kb = (activity.bytes_processed / 1024).toFixed(1);
    addAgentLog("info", `Total size: ${kb} KB`);
  }

  addAgentLog("info", "─────────────────────────────────");
}

// =============================================================================
// FLOATING PROGRESS PANEL
// =============================================================================

// Update terminal in the detail panel with real-time data
function updateDetailTerminal(taskId, message, step, activity) {
  // Try multiple selectors to find the terminal
  let terminalOutput = document.getElementById(`terminal-output-${taskId}`);

  if (!terminalOutput) {
    // Try the currently visible terminal output (any task)
    terminalOutput = document.querySelector(".taskmd-terminal-output");
  }

  if (!terminalOutput) {
    // Try generic terminal output
    terminalOutput = document.querySelector(".terminal-output-rich");
  }

  if (!terminalOutput) {
    console.log("[Terminal] No terminal element found for task:", taskId);
    return;
  }

  // Ensure message is a string
  const messageStr =
    typeof message === "string" ? message : JSON.stringify(message) || "";
  console.log(
    "[Terminal] Adding line to terminal:",
    messageStr.substring(0, 50),
  );
  addTerminalLine(terminalOutput, messageStr, step, activity);
}

// Simple markdown parser for terminal/LLM output
function parseMarkdown(text) {
  if (!text) return "";

  let html = text;

  // Escape HTML first
  html = html
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");

  // Code blocks (```language\ncode```) - must be before other replacements
  html = html.replace(/```(\w*)\n?([\s\S]*?)```/g, (match, lang, code) => {
    const langClass = lang ? ` data-lang="${lang}"` : "";
    return `<pre class="terminal-code"${langClass}><code>${code.trim()}</code></pre>`;
  });

  // Headers (# ## ###)
  html = html.replace(/^### (.+)$/gm, '<h3 class="terminal-h3">$1</h3>');
  html = html.replace(/^## (.+)$/gm, '<h2 class="terminal-h2">$1</h2>');
  html = html.replace(/^# (.+)$/gm, '<h1 class="terminal-h1">$1</h1>');

  // Bold (**text** or __text__)
  html = html.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>");
  html = html.replace(/__(.+?)__/g, "<strong>$1</strong>");

  // Italic (*text* or _text_)
  html = html.replace(/\*([^*]+)\*/g, "<em>$1</em>");
  html = html.replace(/_([^_]+)_/g, "<em>$1</em>");

  // Inline code (`code`)
  html = html.replace(
    /`([^`]+)`/g,
    '<code class="terminal-inline-code">$1</code>',
  );

  // Links [text](url)
  html = html.replace(
    /\[([^\]]+)\]\(([^)]+)\)/g,
    '<a href="$2" target="_blank" rel="noopener">$1</a>',
  );

  // Unordered lists (- item or * item)
  html = html.replace(/^[-*]\s+(.+)$/gm, '<li class="terminal-li">$1</li>');
  // Wrap consecutive li elements in ul
  html = html.replace(
    /(<li class="terminal-li">.*<\/li>\n?)+/g,
    (match) => `<ul class="terminal-ul">${match}</ul>`,
  );

  // Ordered lists (1. item)
  html = html.replace(/^\d+\.\s+(.+)$/gm, '<li class="terminal-oli">$1</li>');
  html = html.replace(
    /(<li class="terminal-oli">.*<\/li>\n?)+/g,
    (match) => `<ol class="terminal-ol">${match}</ol>`,
  );

  // Blockquotes (> text)
  html = html.replace(
    /^>\s+(.+)$/gm,
    '<blockquote class="terminal-quote">$1</blockquote>',
  );

  // Horizontal rule (--- or ***)
  html = html.replace(/^[-*]{3,}$/gm, '<hr class="terminal-hr">');

  // Checkmarks
  html = html.replace(/^✓\s*/gm, '<span class="check-mark">✓</span> ');
  html = html.replace(/^\[x\]/gim, '<span class="check-mark">✓</span>');
  html = html.replace(/^\[ \]/g, '<span class="check-empty">○</span>');

  // Line breaks - convert double newlines to paragraphs
  html = html.replace(/\n\n+/g, '</p><p class="terminal-p">');
  if (!html.startsWith("<")) {
    html = '<p class="terminal-p">' + html + "</p>";
  }

  return html;
}

// Format markdown-like text for terminal display (simple version for status messages)
function formatTerminalMarkdown(text) {
  if (!text) return "";

  // Headers (## Header)
  text = text.replace(
    /^##\s+(.+)$/gm,
    '<strong class="terminal-header">$1</strong>',
  );

  // Bold (**text**)
  text = text.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>");

  // Inline code (`code`)
  text = text.replace(/`([^`]+)`/g, "<code>$1</code>");

  // Code blocks (```code```)
  text = text.replace(
    /```([\s\S]*?)```/g,
    '<pre class="terminal-code"><code>$1</code></pre>',
  );

  // List items (- item)
  text = text.replace(/^-\s+(.+)$/gm, "  • $1");

  // Checkmarks
  text = text.replace(/^✓\s*/gm, '<span class="check-mark">✓</span> ');

  return text;
}

// Render full markdown content (for LLM output)
function renderMarkdownContent(container, markdown) {
  if (!container || !markdown) return;

  const content = document.createElement("div");
  content.className = "markdown-content";
  content.innerHTML = parseMarkdown(markdown);

  // Clear previous content and add new
  container.innerHTML = "";
  container.appendChild(content);
  container.scrollTop = container.scrollHeight;
}

// Update terminal with LLM markdown content
function updateTerminalWithMarkdown(taskId, markdown) {
  const terminalOutput = document.getElementById(`terminal-output-${taskId}`);
  if (terminalOutput) {
    renderMarkdownContent(terminalOutput, markdown);
  } else {
    const genericTerminal = document.querySelector(".taskmd-terminal-output");
    if (genericTerminal) {
      renderMarkdownContent(genericTerminal, markdown);
    }
  }
}

function addTerminalLine(terminal, message, step, activity) {
  const timestamp = new Date().toLocaleTimeString("en-US", { hour12: false });
  const isLlmStream = step === "llm_stream";
  const isLlmOutput =
    step === "llm_output" || (message && message.length > 200);

  // For large LLM output, render as full markdown
  if (isLlmOutput && message && message.length > 200) {
    renderMarkdownContent(terminal, message);
    return;
  }

  // Determine line type based on content
  const isHeader = message && message.startsWith("##");
  const isSuccess = message && message.startsWith("✓");
  const isError = step === "error";
  const isComplete = step === "complete";

  const stepClass = isError
    ? "error"
    : isComplete || isSuccess
      ? "success"
      : isHeader
        ? "progress"
        : isLlmStream
          ? "llm-stream"
          : "info";

  // Format the message with markdown
  const formattedMessage = formatTerminalMarkdown(message);

  const line = document.createElement("div");
  line.className = `terminal-line ${stepClass} current`;

  if (isLlmStream) {
    line.innerHTML = `<span class="llm-text">${formattedMessage}</span>`;
  } else if (isHeader) {
    line.innerHTML = formattedMessage;
  } else {
    line.innerHTML = `<span class="terminal-timestamp">${timestamp}</span>${formattedMessage}`;
  }

  // Remove 'current' class from previous lines
  terminal.querySelectorAll(".terminal-line.current").forEach((el) => {
    el.classList.remove("current");
  });

  terminal.appendChild(line);
  terminal.scrollTop = terminal.scrollHeight;

  // Keep only last 50 lines
  while (terminal.children.length > 50) {
    terminal.removeChild(terminal.firstChild);
  }
}

// Update progress bar in detail panel
function updateDetailProgress(taskId, current, total, percent) {
  const progressFill = document.querySelector(".progress-fill-rich");
  const progressLabel = document.querySelector(".progress-label-rich");
  const stepInfo = document.querySelector(".meta-estimated");

  const pct = percent || (total > 0 ? Math.round((current / total) * 100) : 0);

  if (progressFill) {
    progressFill.style.width = `${pct}%`;
  }
  if (progressLabel) {
    progressLabel.textContent = `Progress: ${pct}%`;
  }
  if (stepInfo) {
    stepInfo.textContent = `Step ${current}/${total}`;
  }
}

// Legacy functions kept for compatibility but now do nothing
function showFloatingProgress(taskName) {
  // Progress now shown in detail panel terminal
  console.log("[Tasks] Progress:", taskName);
}

function updateFloatingProgressBar(
  current,
  total,
  message,
  step,
  details,
  activity,
) {
  // Progress now shown in detail panel
  updateDetailProgress(null, current, total);
  if (message) {
    updateDetailTerminal(null, message, step, activity);
  }
}

function completeFloatingProgress(message, activity, appUrl) {
  // Completion now shown in detail panel
  console.log("[Tasks] Complete:", message);
}

function closeFloatingProgress() {
  // No floating panel to close
}

function minimizeFloatingProgress() {
  // No floating panel to minimize
}

function updateProgressUI(data) {
  if (data && data.current_step !== undefined) {
    updateDetailProgress(
      data.task_id,
      data.current_step,
      data.total_steps,
      data.progress,
    );
  }
}

// Legacy function - errors now shown in detail panel
function errorFloatingProgress(errorMessage) {
  updateDetailTerminal(null, errorMessage, "error");
}

function updateActivityMetrics(activity) {
  // Activity metrics are now shown in terminal output
  if (!activity) return;
  console.log("[Tasks] Activity update:", activity);
}

function logFinalStats(activity) {
  if (!activity) return;
  let stats = "Generation complete";
  if (activity.files_created)
    stats += ` - ${activity.files_created.length} files`;
  if (activity.bytes_processed)
    stats += ` - ${Math.round(activity.bytes_processed / 1024)}KB`;
  console.log("[Tasks]", stats);
}

function addLLMStreamOutput(text) {
  // Add LLM streaming output to the floating terminal
  const terminal = document.getElementById("floating-llm-terminal");
  if (!terminal) return;

  const line = document.createElement("div");
  line.className = "llm-output";
  line.textContent = text;
  terminal.appendChild(line);
  terminal.scrollTop = terminal.scrollHeight;

  // Keep only last 100 lines to prevent memory issues
  while (terminal.children.length > 100) {
    terminal.removeChild(terminal.firstChild);
  }
}

function updateProgressUI(data) {
  const progressBar = document.querySelector(".result-progress-bar");
  const resultDiv = document.getElementById("intent-result");

  if (data.total_steps && data.current_step) {
    const percent = Math.round((data.current_step / data.total_steps) * 100);

    if (progressBar) {
      progressBar.style.width = `${percent}%`;
    }

    if (resultDiv && data.message) {
      resultDiv.innerHTML = `
        <div class="result-card">
          <div class="result-message">${data.message}</div>
          <div class="result-progress">
            <div class="result-progress-bar" style="width: ${percent}%"></div>
          </div>
          <div style="margin-top:8px;font-size:12px;color:var(--sentient-text-muted);">
            Step ${data.current_step}/${data.total_steps} (${percent}%)
          </div>
        </div>
      `;
    }
  }
}

// =============================================================================
// EVENT LISTENERS
// =============================================================================

function setupEventListeners() {
  // Filter pills
  document.querySelectorAll(".status-pill").forEach((pill) => {
    pill.addEventListener("click", function (e) {
      e.preventDefault();
      const filter = this.dataset.filter;
      setActiveFilter(filter, this);
    });
  });

  // Search input
  const searchInput = document.querySelector(".topbar-search-input");
  if (searchInput) {
    searchInput.addEventListener(
      "input",
      debounce(function (e) {
        searchTasks(e.target.value);
      }, 300),
    );
  }

  // Nav items
  document.querySelectorAll(".topbar-nav-item").forEach((item) => {
    item.addEventListener("click", function () {
      document
        .querySelectorAll(".topbar-nav-item")
        .forEach((i) => i.classList.remove("active"));
      this.classList.add("active");
    });
  });

  // Progress log toggle
  const logToggle = document.querySelector(".progress-log-toggle");
  if (logToggle) {
    logToggle.addEventListener("click", toggleProgressLog);
  }
}

function setupKeyboardShortcuts() {
  document.addEventListener("keydown", function (e) {
    // Escape: Deselect task
    if (e.key === "Escape") {
      deselectTask();
    }

    // Cmd/Ctrl + K: Focus search
    if ((e.metaKey || e.ctrlKey) && e.key === "k") {
      e.preventDefault();
      document.querySelector(".topbar-search-input")?.focus();
    }

    // Arrow keys: Navigate tasks
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      navigateTasks(e.key === "ArrowDown" ? 1 : -1);
    }

    // Enter: Submit decision if in decision mode
    if (
      e.key === "Enter" &&
      document.querySelector(".decision-option.selected")
    ) {
      submitDecision();
    }

    // 1-5: Quick filter
    if (e.key >= "1" && e.key <= "5" && !e.target.matches("input, textarea")) {
      const pills = document.querySelectorAll(".status-pill");
      const index = parseInt(e.key) - 1;
      if (pills[index]) {
        pills[index].click();
      }
    }
  });
}

// =============================================================================
// TASK SELECTION & FILTERING
// =============================================================================

function selectTask(taskId) {
  TasksState.selectedTaskId = taskId;

  // Update selected state in list
  document.querySelectorAll(".task-card").forEach((card) => {
    card.classList.toggle("selected", card.dataset.taskId == taskId);
  });

  // Load task details (in real app, this would fetch from API)
  loadTaskDetails(taskId);

  // Check if we have a pending manifest update for this task
  const pending = findPendingManifest(taskId);
  if (pending) {
    console.log(
      "[selectTask] Found pending manifest for task:",
      taskId,
      "from key:",
      pending.key,
    );
    // Wait for detail content to load, then render manifest
    setTimeout(() => {
      renderManifestProgress(taskId, pending.manifest, 0, false);
    }, 300);
  }
}

function deselectTask() {
  TasksState.selectedTaskId = null;
  document.querySelectorAll(".task-card").forEach((card) => {
    card.classList.remove("selected");
  });
}

function navigateTasks(direction) {
  const cards = Array.from(document.querySelectorAll(".task-card"));
  if (cards.length === 0) return;

  const currentIndex = cards.findIndex((c) => c.classList.contains("selected"));
  let newIndex;

  if (currentIndex === -1) {
    newIndex = direction === 1 ? 0 : cards.length - 1;
  } else {
    newIndex = currentIndex + direction;
    if (newIndex < 0) newIndex = cards.length - 1;
    if (newIndex >= cards.length) newIndex = 0;
  }

  const taskId = cards[newIndex].dataset.taskId;
  selectTask(taskId);
  cards[newIndex].scrollIntoView({ behavior: "smooth", block: "nearest" });
}

function setActiveFilter(filter, button) {
  TasksState.currentFilter = filter;

  // Update active pill
  document.querySelectorAll(".status-pill").forEach((pill) => {
    pill.classList.remove("active");
  });
  button.classList.add("active");

  // Filter will be handled by HTMX, but we track state
  addAgentLog("info", `[FILTER] Showing ${filter} tasks`);
}

function searchTasks(query) {
  if (query.length > 0) {
    addAgentLog("info", `[SEARCH] Searching: "${query}"`);
  }

  // In real app, this would filter via API
  // For demo, we'll do client-side filtering
  const cards = document.querySelectorAll(".task-card");
  cards.forEach((card) => {
    const title =
      card.querySelector(".task-card-title")?.textContent.toLowerCase() || "";
    const subtitle =
      card.querySelector(".task-card-subtitle")?.textContent.toLowerCase() ||
      "";
    const matches =
      title.includes(query.toLowerCase()) ||
      subtitle.includes(query.toLowerCase());
    card.style.display = matches || query === "" ? "block" : "none";
  });
}

// =============================================================================
// TASK DETAILS
// =============================================================================

function loadTaskDetails(taskId) {
  if (!taskId) {
    console.warn("[LOAD] No task ID provided");
    return;
  }

  // Prevent multiple simultaneous loads of the same task
  if (TasksState.loadingTaskId === taskId) {
    console.log("[LOAD] Already loading task:", taskId);
    return;
  }

  addAgentLog("info", `[LOAD] Loading task #${taskId} details`);
  TasksState.loadingTaskId = taskId;

  // Show detail panel and hide empty state
  const emptyState = document.getElementById("detail-empty");
  const detailContent = document.getElementById("task-detail-content");

  if (!detailContent) {
    console.error("[LOAD] task-detail-content element not found");
    TasksState.loadingTaskId = null;
    return;
  }

  if (emptyState) emptyState.style.display = "none";
  detailContent.style.display = "block";

  // Fetch task details from API - use requestAnimationFrame to ensure DOM is ready
  requestAnimationFrame(() => {
    if (typeof htmx !== "undefined" && htmx.ajax) {
      htmx
        .ajax("GET", `/api/tasks/${taskId}`, {
          target: "#task-detail-content",
          swap: "innerHTML",
        })
        .then(() => {
          TasksState.loadingTaskId = null;
        })
        .catch(() => {
          TasksState.loadingTaskId = null;
        });
    } else {
      console.error("[LOAD] HTMX not available");
      TasksState.loadingTaskId = null;
    }
  });
}

function updateTaskCard(task) {
  const card = document.querySelector(`[data-task-id="${task.id}"]`);
  if (!card) return;

  // Update progress
  const progressFill = card.querySelector(".task-progress-fill");
  const progressPercent = card.querySelector(".task-progress-percent");
  const progressSteps = card.querySelector(".task-progress-steps");

  if (progressFill) progressFill.style.width = `${task.progress}%`;
  if (progressPercent) progressPercent.textContent = `${task.progress}%`;
  if (progressSteps)
    progressSteps.textContent = `${task.currentStep}/${task.totalSteps} steps`;

  // Update status badge
  const statusBadge = card.querySelector(".task-card-status");
  if (statusBadge) {
    statusBadge.className = `task-card-status ${task.status}`;
    statusBadge.textContent = formatStatus(task.status);
  }
}

function updateTaskDetail(task) {
  // Update detail panel with task data
  const detailTitle = document.querySelector(".task-detail-title");
  if (detailTitle) detailTitle.textContent = task.title;
}

// Update task card from polling without full list refresh
function updateTaskCardFromPoll(taskId, task) {
  const card = document.querySelector(`[data-task-id="${taskId}"]`);
  if (!card) return;

  // Don't update if the list is being swapped by HTMX
  const taskList = document.getElementById("task-list");
  if (
    taskList &&
    (taskList.classList.contains("htmx-swapping") ||
      taskList.classList.contains("htmx-settling"))
  ) {
    return;
  }

  // Update progress bar
  const progressFill = card.querySelector(".task-progress-fill");
  const progressPercent = card.querySelector(".task-progress-percent");
  if (progressFill && task.progress !== undefined) {
    progressFill.style.width = `${task.progress}%`;
  }
  if (progressPercent && task.progress !== undefined) {
    progressPercent.textContent = `${Math.round(task.progress)}%`;
  }

  // Update status badge
  const statusBadge = card.querySelector(".task-card-status");
  if (statusBadge && task.status) {
    const oldStatus = statusBadge.className
      .split(" ")
      .find((c) => c !== "task-card-status");
    if (oldStatus) statusBadge.classList.remove(oldStatus);
    statusBadge.classList.add(task.status);
    statusBadge.textContent = formatStatus(task.status);
  }
}

// Add a new task card to the list without full refresh
function addTaskCardToList(taskId, title, status) {
  const taskList = document.getElementById("task-list");
  if (!taskList) return;

  // Check if card already exists
  if (taskList.querySelector(`[data-task-id="${taskId}"]`)) {
    return;
  }

  // Don't insert if task list is currently being swapped by HTMX
  if (
    taskList.classList.contains("htmx-swapping") ||
    taskList.classList.contains("htmx-settling")
  ) {
    console.log("[TASK] Skipping card insert - list is being swapped");
    return;
  }

  const statusClass = status || "running";
  const statusText = formatStatus(status) || "Running";

  const cardHtml = `
    <div class="task-card ${statusClass}" data-task-id="${taskId}" onclick="selectTask('${taskId}')">
      <div class="task-card-header">
        <span class="task-card-icon">📋</span>
        <span class="task-card-title">${escapeHtml(title)}</span>
      </div>
      <div class="task-card-meta">
        <span class="task-card-status ${statusClass}">${statusText}</span>
        <span class="task-card-priority">medium</span>
      </div>
      <div class="task-card-progress">
        <div class="task-progress-bar">
          <div class="task-progress-fill" style="width: 0%"></div>
        </div>
        <span class="task-progress-percent">0%</span>
      </div>
      <div class="task-card-actions">
        <button class="task-action-btn" title="Star">★</button>
        <button class="task-action-btn" title="Delete">🗑</button>
      </div>
    </div>
  `;

  // Insert at the top of the list
  taskList.insertAdjacentHTML("afterbegin", cardHtml);
}

// Update just the status of a task card
function updateTaskCardStatus(taskId, status) {
  const card = document.querySelector(`[data-task-id="${taskId}"]`);
  if (!card) return;

  // Don't update if the list is being swapped by HTMX
  const taskList = document.getElementById("task-list");
  if (
    taskList &&
    (taskList.classList.contains("htmx-swapping") ||
      taskList.classList.contains("htmx-settling"))
  ) {
    return;
  }

  const statusBadge = card.querySelector(".task-card-status");
  if (statusBadge) {
    statusBadge.className = `task-card-status ${status}`;
    statusBadge.textContent = formatStatus(status);
  }

  // Update progress to 100% for completed
  if (status === "completed") {
    const progressFill = card.querySelector(".task-progress-fill");
    const progressPercent = card.querySelector(".task-progress-percent");
    if (progressFill) progressFill.style.width = "100%";
    if (progressPercent) progressPercent.textContent = "100%";
  }
}

// =============================================================================
// DECISION HANDLING
// =============================================================================

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
document.addEventListener("DOMContentLoaded", updateFilterCounts);
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
      });
    })
    .catch(function (e) {
      console.warn("Failed to load stats:", e);
    });
}

// =============================================================================
// SPLITTER DRAG FUNCTIONALITY
// =============================================================================

(function initSplitter() {
  var splitter = document.getElementById("tasks-splitter");
  var main = document.querySelector(".tasks-main");
  var leftPanel = document.querySelector(".tasks-list-panel");

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
  });

  document.addEventListener("mousemove", function (e) {
    if (!isDragging) return;

    var diff = e.clientX - startX;
    var newWidth = Math.max(200, Math.min(600, startWidth + diff));
    leftPanel.style.flex = "0 0 " + newWidth + "px";
    leftPanel.style.width = newWidth + "px";
  });

  document.addEventListener("mouseup", function () {
    if (isDragging) {
      isDragging = false;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    }
  });
})();

// =============================================================================
// HTMX TASK CREATION HANDLER
// =============================================================================

document.body.addEventListener("htmx:afterRequest", function (evt) {
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
});

// =============================================================================
// FILTER PILL CLICK HANDLER
// =============================================================================

document.querySelectorAll(".filter-pill").forEach(function (pill) {
  pill.addEventListener("click", function () {
    document.querySelectorAll(".filter-pill").forEach(function (p) {
      p.classList.remove("active");
    });
    this.classList.add("active");
  });
});

// =============================================================================
// HTMX TASK LIST REFRESH HANDLER
// =============================================================================

document.body.addEventListener("htmx:afterSwap", function (e) {
  if (e.detail.target && e.detail.target.id === "task-list") {
    loadTaskStats();
    var taskList = document.getElementById("task-list");
    var emptyState = document.getElementById("empty-state");
    if (taskList && emptyState) {
      var hasTasks = taskList.querySelector(".task-card");
      emptyState.style.display = hasTasks ? "none" : "flex";
    }
  }
});
