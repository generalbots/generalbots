
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
  const progressBar = document.getElementById("window-tasks").querySelector(".result-progress-bar");
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
  document.getElementById("window-tasks").querySelectorAll(".status-pill").forEach((pill) => {
    pill.addEventListener("click", function (e) {
      e.preventDefault();
      const filter = this.dataset.filter;
      setActiveFilter(filter, this);
    }, 100);
  }, 100);

  // Search input
  const searchInput = document.getElementById("window-tasks").querySelector(".topbar-search-input");
  if (searchInput) {
    searchInput.addEventListener(
      "input",
      debounce(function (e) {
        searchTasks(e.target.value);
      }, 300),
    );
  }

  // Nav items
  document.getElementById("window-tasks").querySelectorAll(".topbar-nav-item").forEach((item) => {
    item.addEventListener("click", function () {
      document
        .querySelectorAll(".topbar-nav-item")
        .forEach((i) => i.classList.remove("active"));
      this.classList.add("active");
    }, 100);
  }, 100);

  // Progress log toggle
  const logToggle = document.getElementById("window-tasks").querySelector(".progress-log-toggle");
  if (logToggle) {
    logToggle.addEventListener("click", toggleProgressLog);
  }
}

function setupKeyboardShortcuts() {
if (tasksWindow) {
  tasksWindow.addEventListener("keydown", function (e) {
    // Escape: Deselect task
    if (e.key === "Escape") {
      deselectTask();
    }

    // Cmd/Ctrl + K: Focus search
    if ((e.metaKey || e.ctrlKey) && e.key === "k") {
      e.preventDefault();
      document.getElementById("window-tasks").querySelector(".topbar-search-input")?.focus();
    }

    // Arrow keys: Navigate tasks
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      navigateTasks(e.key === "ArrowDown" ? 1 : -1);
    }

    // Enter: Submit decision if in decision mode
    if (
      e.key === "Enter" &&
      document.getElementById("window-tasks").querySelector(".decision-option.selected")
    ) {
      submitDecision();
    }

    // 1-5: Quick filter
    if (e.key >= "1" && e.key <= "5" && !e.target.matches("input, textarea")) {
      const pills = document.getElementById("window-tasks").querySelectorAll(".status-pill");
      const index = parseInt(e.key) - 1;
      if (pills[index]) {
        pills[index].click();
      }
    }
  }, 100);
  }
}

// =============================================================================
// TASK SELECTION & FILTERING
// =============================================================================

function selectTask(taskId) {
  TasksState.selectedTaskId = taskId;

  // Update selected state in list
  document.getElementById("window-tasks").querySelectorAll(".task-card").forEach((card) => {
    card.classList.toggle("selected", card.dataset.taskId == taskId);
  }, 100);

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
  document.getElementById("window-tasks").querySelectorAll(".task-card").forEach((card) => {
    card.classList.remove("selected");
  }, 100);
}

function navigateTasks(direction) {
  const cards = Array.from(document.getElementById("window-tasks").querySelectorAll(".task-card"));
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
  cards[newIndex].scrollIntoView({ behavior: "smooth", block: "nearest" }, 100);
}

function setActiveFilter(filter, button) {
  TasksState.currentFilter = filter;

  // Update active pill
  document.getElementById("window-tasks").querySelectorAll(".status-pill").forEach((pill) => {
    pill.classList.remove("active");
  }, 100);
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
  const cards = document.getElementById("window-tasks").querySelectorAll(".task-card");
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
  }, 100);
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
        }, 100);
    } else {
      console.error("[LOAD] HTMX not available");
      TasksState.loadingTaskId = null;
    }
  }, 100);
}

function updateTaskCard(task) {
  const card = document.getElementById("window-tasks").querySelector(`[data-task-id="${task.id}"]`);
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
  const detailTitle = document.getElementById("window-tasks").querySelector(".task-detail-title");
  if (detailTitle) detailTitle.textContent = task.title;
}

// Update task card from polling without full list refresh
function updateTaskCardFromPoll(taskId, task) {
  const card = document.getElementById("window-tasks").querySelector(`[data-task-id="${taskId}"]`);
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
        <button class="task-action-btn" title="Edit" onclick="event.stopPropagation(); editTask('${taskId}')">✏️</button>
        <button class="task-action-btn" title="Complete" onclick="event.stopPropagation(); completeTask('${taskId}')">✓</button>
        <button class="task-action-btn" title="Delete" onclick="event.stopPropagation(); deleteTask('${taskId}')">🗑</button>
      </div>
    </div>
  `;

  // Insert at the top of the list
  taskList.insertAdjacentHTML("afterbegin", cardHtml);
}

// Update just the status of a task card
function updateTaskCardStatus(taskId, status) {
  const card = document.getElementById("window-tasks").querySelector(`[data-task-id="${taskId}"]`);
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
