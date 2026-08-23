if (window.GBAppLifecycle) GBAppLifecycle.begin("tasks");
/* =============================================================================
   AUTO TASK JAVASCRIPT - Sentient Theme
   Intelligent Self-Executing Task Interface
   ============================================================================= */

// =============================================================================
// STATE MANAGEMENT
// =============================================================================

const AutoTaskState = {
  currentFilter: "active",
  selectedIntentId: null,
  intents: [],
  compiledPlan: null,
  pendingDecisions: [],
  pendingApprovals: [],
  refreshInterval: null,
  wsConnection: null,
  progressWsConnection: null,
  activeTaskProgress: {},
  llmOutputStream: [],
};

// =============================================================================
// INITIALIZATION
// =============================================================================

(function(){ var __cb = function () {
  initAutoTask();
}; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();

function initAutoTask() {
  // Initialize WebSocket for real-time updates
  initWebSocket();

  // Initialize task progress WebSocket
  initTaskProgressWebSocket();

  // Start auto-refresh
  startAutoRefresh();

  // Setup event listeners
  setupEventListeners();

  // Load initial stats
  updateStats();

  // Setup keyboard shortcuts
  setupKeyboardShortcuts();

  // Initialize floating panel (hidden by default)
  initFloatingPanel();

  console.log("AutoTask Sentient initialized");
}

// =============================================================================
// FLOATING PANEL
// =============================================================================

function initFloatingPanel() {
  const panel = document.getElementById("floating-progress");
  if (panel) {
    panel.style.display = "none";
  }
}

function showFloatingPanel(taskId, taskName) {
  const panel = document.getElementById("floating-progress");
  if (!panel) return;

  panel.style.display = "block";
  panel.dataset.taskId = taskId;

  const nameEl = document.getElementById("floating-task-name");
  if (nameEl) nameEl.textContent = taskName || "Processing...";

  updateFloatingProgress(0, 0, 0, "Initializing...");
}

function updateFloatingProgress(progress, current, total, statusText) {
  const fill = document.getElementById("floating-progress-fill");
  const pct = document.getElementById("floating-percentage");
  const steps = document.getElementById("floating-status-steps");
  const status = document.getElementById("floating-status-text");

  if (fill) fill.style.width = `${progress}%`;
  if (pct) pct.textContent = `${progress}%`;
  if (steps) steps.textContent = `${current}/${total}`;
  if (status) status.textContent = statusText;
}

function addFloatingLog(icon, message, type = "info") {
  const log = document.getElementById("floating-log");
  if (!log) return;

  const entry = document.createElement("div");
  entry.className = `floating-log-entry ${type}`;
  entry.innerHTML = `
    <span class="log-icon">${icon}</span>
    <span class="log-message">${escapeHtml(message)}</span>
    <span class="log-time">${formatTime(new Date())}</span>
  `;
  log.appendChild(entry);
  log.scrollTop = log.scrollHeight;
}

function addFloatingTerminalOutput(text) {
  const terminal = document.getElementById("floating-terminal");
  if (!terminal) return;

  const line = document.createElement("div");
  line.className = "llm-output";
  line.textContent = text;
  terminal.appendChild(line);
  terminal.scrollTop = terminal.scrollHeight;

  // Keep only last 50 lines
  while (terminal.children.length > 50) {
    terminal.removeChild(terminal.firstChild);
  }
}

function minimizeFloatingPanel() {
  const panel = document.getElementById("floating-progress");
  if (panel) {
    panel.classList.toggle("minimized");
  }
}

function closeFloatingPanel() {
  const panel = document.getElementById("floating-progress");
  if (panel) {
    panel.style.display = "none";
  }
}

function completeFloatingPanel(message) {
  updateFloatingProgress(100, 0, 0, message || "Complete!");
  addFloatingLog("✅", message || "Task completed successfully", "success");

  // Auto-hide after delay
  setTimeout(() => {
    closeFloatingPanel();
  }, 5000);
}

// =============================================================================
// WEBSOCKET CONNECTION
// =============================================================================

function initWebSocket() {
  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  const wsUrl = `${protocol}//${window.location.host}/ws/autotask`;

  try {
    AutoTaskState.wsConnection = new WebSocket(wsUrl);

    AutoTaskState.wsConnection.onopen = function () {
      console.log("AutoTask WebSocket connected");
    };

    AutoTaskState.wsConnection.onmessage = function (event) {
      handleWebSocketMessage(JSON.parse(event.data));
    };

    AutoTaskState.wsConnection.onclose = function () {
      console.log("AutoTask WebSocket disconnected, reconnecting...");
      setTimeout(initWebSocket, 5000);
    };

    AutoTaskState.wsConnection.onerror = function (error) {
      console.error("AutoTask WebSocket error:", error);
    };
  } catch (e) {
    console.warn("WebSocket not available, using polling");
  }
}

function initTaskProgressWebSocket() {
  // Use the singleton WebSocket from tasks.js instead of creating a duplicate connection
  // This prevents the "2 receivers" problem where manifest_update events go to one
  // WebSocket while the browser UI is listening on a different one

  console.log("[AutoTask] Using singleton WebSocket for task progress");

  // Create handler for task progress messages
  const handler = function (data) {
    handleTaskProgressMessage(data);
  };

  // Store handler reference for cleanup
  AutoTaskState._progressHandler = handler;

  // Register with the global singleton WebSocket from tasks.js
  if (typeof registerTaskProgressHandler === "function") {
    registerTaskProgressHandler(handler);
    console.log("[AutoTask] Registered with singleton WebSocket");
  } else {
    // Fallback: wait for tasks.js to load and retry
    console.log("[AutoTask] Waiting for tasks.js singleton to be available...");
    setTimeout(initTaskProgressWebSocket, 500);
  }
}

function handleTaskProgressMessage(data) {
  // Note: ProgressPanel now registers its own handler with the singleton,
  // so we don't need to forward messages manually here

  console.log("[AutoTask] Task progress:", data.type, data.task_id);

  switch (data.type) {
    case "connected":
      console.log("Connected to task progress stream");
      break;
    case "task_started":
      onTaskStarted(data);
      break;
    case "task_progress":
      onTaskProgress(data);
      break;
    case "task_completed":
      onTaskProgressCompleted(data);
      break;
    case "task_error":
      onTaskProgressError(data);
      break;
    case "llm_stream":
      onLLMStream(data);
      break;
    default:
      console.log("Unknown progress event:", data.type);
  }
}

function onTaskStarted(data) {
  AutoTaskState.activeTaskProgress[data.task_id] = {
    started: new Date(),
    totalSteps: data.total_steps,
    currentStep: 0,
    progress: 0,
    logs: [],
  };

  // Show floating panel
  showFloatingPanel(data.task_id, data.message);
  addFloatingLog("🚀", data.message, "started");

  // Also update detail panel if viewing this intent
  if (AutoTaskState.selectedIntentId === data.task_id) {
    updateDetailPanelProgress(data);
  }
}

function onTaskProgress(data) {
  const taskProgress = AutoTaskState.activeTaskProgress[data.task_id];
  if (taskProgress) {
    taskProgress.currentStep = data.current_step;
    taskProgress.progress = data.progress;
    taskProgress.logs.push({
      time: new Date(),
      step: data.step,
      message: data.message,
      details: data.details,
    });
  }

  // Update floating panel
  updateFloatingProgress(
    data.progress,
    data.current_step,
    data.total_steps,
    data.message,
  );

  const stepIcons = {
    llm_request: "🤖",
    llm_response: "✨",
    parse_structure: "📋",
    create_tables: "🗄️",
    tables_synced: "✅",
    write_files: "📝",
    write_file: "📄",
    write_designer: "🎨",
    write_tools: "🔧",
    sync_site: "🔄",
  };
  const icon = stepIcons[data.step] || "📌";
  addFloatingLog(icon, data.message, "progress");

  // Update detail panel if viewing this intent
  if (AutoTaskState.selectedIntentId === data.task_id) {
    updateDetailPanelProgress(data);
    addTerminalLine(data.message);
  }
}

function onTaskProgressCompleted(data) {
  const taskProgress = AutoTaskState.activeTaskProgress[data.task_id];
  if (taskProgress) {
    taskProgress.progress = 100;
    taskProgress.completed = new Date();
  }

  completeFloatingPanel(data.message);

  // Refresh intent list
  setTimeout(() => {
    refreshIntents();
    updateStats();
  }, 1000);
}

function onTaskProgressError(data) {
  addFloatingLog("❌", data.error, "error");
  updateFloatingProgress(
    AutoTaskState.activeTaskProgress[data.task_id]?.progress || 0,
    0,
    0,
    `Error: ${data.error}`,
  );
}

function onLLMStream(data) {
  // Stream LLM output to terminal
  addFloatingTerminalOutput(data.text);
  if (AutoTaskState.selectedIntentId === data.task_id) {
    addTerminalLine(data.text, true);
  }
}

function addTerminalLine(text, isLLM = false) {
  const terminal = document.getElementById("window-tasks").querySelector(
    `#terminal-${AutoTaskState.selectedIntentId}`,
  );
  if (!terminal) return;

  const line = document.createElement("div");
  line.className = `terminal-line${isLLM ? " highlight" : ""}`;
  line.textContent = text;
  terminal.appendChild(line);
  terminal.scrollTop = terminal.scrollHeight;
}

function updateDetailPanelProgress(data) {
  // Update progress bar in detail panel
  const progressFill = document.getElementById("window-tasks").querySelector(
    ".detail-progress .progress-fill",
  );
  const progressSteps = document.getElementById("window-tasks").querySelector(
    ".detail-progress .progress-steps",
  );
  const progressPct = document.getElementById("window-tasks").querySelector(
    ".detail-progress .progress-percent",
  );

  if (progressFill) progressFill.style.width = `${data.progress}%`;
  if (progressSteps)
    progressSteps.textContent = `${data.current_step}/${data.total_steps} Steps`;
  if (progressPct) progressPct.textContent = `${data.progress}%`;
}

// =============================================================================
// INTENT SELECTION & DETAIL PANEL
// =============================================================================

function selectIntent(intentId) {
  // Update selected state
  document.getElementById("window-tasks").querySelectorAll(".intent-card").forEach((card) => {
    card.classList.remove("selected");
  });

  const selectedCard = document.getElementById("window-tasks").querySelector(
    `.intent-card[data-intent-id="${intentId}"]`,
  );
  if (selectedCard) {
    selectedCard.classList.add("selected");
  }

  AutoTaskState.selectedIntentId = intentId;

  // Load detail panel
  loadIntentDetail(intentId);
}

function loadIntentDetail(intentId) {
  const panel = document.getElementById("intent-detail-panel");
  if (!panel) return;

  // Show loading state
  panel.innerHTML = `
    <div class="loading-state">
      <div class="spinner"></div>
      <span>Loading intent details...</span>
    </div>
  `;

  // Fetch intent details and manifest in parallel
  Promise.all([
    fetch(`/api/autotask/${intentId}`).then((r) => r.json()),
    fetch(`/api/autotask/${intentId}/manifest`)
      .then((r) => r.json())
      .catch(() => null),
  ])
    .then(([taskData, manifestData]) => {
      renderIntentDetail(taskData);

      // If manifest exists, initialize ProgressPanel
      if (manifestData && manifestData.success && manifestData.manifest) {
        if (typeof ProgressPanel !== "undefined") {
          ProgressPanel.manifest = manifestData.manifest;
          ProgressPanel.init(intentId);
          ProgressPanel.render();
        }
      }
    })
    .catch((error) => {
      console.error("Failed to load intent detail:", error);
      panel.innerHTML = `
        <div class="detail-placeholder">
          <span class="placeholder-icon">⚠️</span>
          <p>Failed to load intent details</p>
        </div>
      `;
    });
}
