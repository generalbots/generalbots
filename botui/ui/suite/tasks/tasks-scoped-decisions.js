// DECISION HANDLING
// =============================================================================

function selectDecision(element, value) {
  // Remove selected from all options
  document.getElementById("window-tasks").querySelectorAll(".decision-option").forEach((opt) => {
    opt.classList.remove("selected");
  }, 100);

  // Add selected to clicked option
  element.classList.add("selected");

  // Store selected value
  TasksState.selectedDecision = value;

  addAgentLog("info", `[DECISION] Selected: ${value}`);
}

function submitDecision() {
  const selectedOption = document.getElementById("window-tasks").querySelector(".decision-option.selected");
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
        const decisionSection = document.getElementById("window-tasks").querySelector(
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

      const decisionSection = document.getElementById("window-tasks").querySelector(
        ".decision-required-section",
      );
      if (decisionSection) {
        decisionSection.style.opacity = "0.5";
        setTimeout(() => {
          decisionSection.style.display = "none";
        }, 500);
      }

      // Update step status
      const activeStep = document.getElementById("window-tasks").querySelector(".step-item.active");
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
    }, 100);
}

function showDecisionRequired(decision) {
  addAgentLog("warning", `[ALERT] Decision required: ${decision.title}`);
  showToast(`Decision required: ${decision.title}`, "warning");
}

// =============================================================================
// PROGRESS LOG
// =============================================================================

function toggleProgressLog() {
  const stepList = document.getElementById("window-tasks").querySelector(".step-list");
  const toggle = document.getElementById("window-tasks").querySelector(".progress-log-toggle");

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

  const stepItems = document.getElementById("window-tasks").querySelectorAll(".step-item");
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
  }, 100);
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
  const pauseBtn = document.getElementById("window-tasks").querySelector(".agent-activity-btn:last-child");
  if (pauseBtn) {
    pauseBtn.textContent = TasksState.agentLogPaused ? "Resume" : "Pause";
  }
  addAgentLog(
    "info",
    TasksState.agentLogPaused ? "[SYSTEM] Log paused" : "[SYSTEM] Log resumed",
  );
}

// =============================================================================
// =============================================================================
// TASK EDIT MODAL (issue #878)
// =============================================================================

function editTask(taskId) {
  if (!taskId) return;

  fetch(`/api/tasks/${taskId}`, {
    headers: { "Content-Type": "application/json" },
  })
    .then((response) => response.json())
    .then((data) => {
      const task = data.task || data;
      if (!task) {
        showToast("Task not found", "error");
        return;
      }

      TasksState.editingTaskId = task.id || taskId;
      document.getElementById("task-edit-id").value = task.id || taskId;
      document.getElementById("task-edit-title").value = task.title || "";
      document.getElementById("task-edit-description").value =
        task.description || "";
      document.getElementById("task-edit-priority").value = priorityToIndex(
        task.priority,
      );
      document.getElementById("task-edit-due").value = task.due_date
        ? toDatetimeLocal(task.due_date)
        : "";
      document.getElementById("task-edit-assignee").value =
        task.assignee_id || "";
      document.getElementById("task-edit-parent").value = task.parent_id || "";

      const done = isDoneStatus(task.status);
      document.getElementById("task-edit-complete-btn").style.display = done
        ? "none"
        : "";
      document.getElementById("task-edit-reopen-btn").style.display = done
        ? ""
        : "none";

      document.getElementById("task-edit-modal").style.display = "flex";
    })
    .catch((error) => {
      showToast("Failed to load task", "error");
      console.error("[TASK] editTask error:", error);
    });
}

function closeTaskEditor() {
  const modal = document.getElementById("task-edit-modal");
  if (modal) modal.style.display = "none";
  TasksState.editingTaskId = null;
}

function saveTaskEdit() {
  const taskId =
    TasksState.editingTaskId ||
    document.getElementById("task-edit-id").value;
  if (!taskId) return;

  const title = document.getElementById("task-edit-title").value.trim();
  const description = document
    .getElementById("task-edit-description")
    .value.trim();
  const priorityIdx = document.getElementById("task-edit-priority").value;
  const due = document.getElementById("task-edit-due").value;
  const assignee = document.getElementById("task-edit-assignee").value.trim();
  const parent = document.getElementById("task-edit-parent").value.trim();

  const payload = {
    title: title || undefined,
    description: description ? description : null,
    priority: priorityIdx === "" ? null : parseInt(priorityIdx, 10),
    assignee_id: assignee ? assignee : null,
    due_date: due ? new Date(due).toISOString() : null,
    parent_id: parent ? parent : null,
  };

  fetch(`/api/tasks/${taskId}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  })
    .then((response) => response.json())
    .then((result) => {
      if (result.error) {
        showToast("Failed to save task: " + result.error, "error");
        return;
      }
      showToast("Task saved", "success");
      closeTaskEditor();
      htmx.trigger(document.body, "taskCreated");
      if (TasksState.selectedTaskId === taskId) {
        loadTaskDetails(taskId);
      }
    })
    .catch((error) => {
      showToast("Failed to save task", "error");
      console.error("[TASK] saveTaskEdit error:", error);
    });
}

function completeTaskFromEditor() {
  const taskId =
    TasksState.editingTaskId ||
    document.getElementById("task-edit-id").value;
  if (taskId) completeTask(taskId);
}

function reopenTaskFromEditor() {
  const taskId =
    TasksState.editingTaskId ||
    document.getElementById("task-edit-id").value;
  if (taskId) reopenTask(taskId);
}

function deleteTaskFromEditor() {
  const taskId =
    TasksState.editingTaskId ||
    document.getElementById("task-edit-id").value;
  if (taskId) deleteTask(taskId);
}

function completeTask(taskId) {
  if (!taskId) return;

  fetch(`/api/tasks/${taskId}/complete`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
  })
    .then((response) => response.json())
    .then((result) => {
      if (result.error) {
        showToast("Failed to complete task: " + result.error, "error");
        return;
      }
      showToast("Task completed", "success");
      closeTaskEditor();
      htmx.trigger(document.body, "taskCreated");
      if (TasksState.selectedTaskId === taskId) {
        loadTaskDetails(taskId);
      }
    })
    .catch((error) => {
      showToast("Failed to complete task", "error");
      console.error("[TASK] completeTask error:", error);
    });
}

function reopenTask(taskId) {
  if (!taskId) return;

  fetch(`/api/tasks/${taskId}/reopen`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
  })
    .then((response) => response.json())
    .then((result) => {
      if (result.error) {
        showToast("Failed to reopen task: " + result.error, "error");
        return;
      }
      showToast("Task reopened", "success");
      closeTaskEditor();
      htmx.trigger(document.body, "taskCreated");
      if (TasksState.selectedTaskId === taskId) {
        loadTaskDetails(taskId);
      }
    })
    .catch((error) => {
      showToast("Failed to reopen task", "error");
      console.error("[TASK] reopenTask error:", error);
    });
}

function deleteTask(taskId) {
  if (!taskId) return;
  if (!confirm("Delete this task? This cannot be undone.")) {
    return;
  }

  fetch(`/api/tasks/${taskId}`, {
    method: "DELETE",
    headers: { "Content-Type": "application/json" },
  })
    .then((response) => response.json())
    .then((result) => {
      if (result.error) {
        showToast("Failed to delete task: " + result.error, "error");
        return;
      }
      showToast("Task deleted", "success");
      closeTaskEditor();
      if (TasksState.selectedTaskId === taskId) {
        deselectTask();
      }
      htmx.trigger(document.body, "taskCreated");
    })
    .catch((error) => {
      showToast("Failed to delete task", "error");
      console.error("[TASK] deleteTask error:", error);
    });
}

function isDoneStatus(status) {
  return ["done", "complete", "completed", "resolved"].includes(status);
}

function priorityToIndex(priority) {
  // schema priority is an int (0 low .. 3 critical); legacy data may store
  // text ('low'/'medium'/'high'/'urgent'). Normalize to a 0..3 index.
  if (typeof priority === "number" && !Number.isNaN(priority)) {
    return String(Math.max(0, Math.min(3, priority)));
  }
  const map = {
    low: "0",
    medium: "1",
    normal: "1",
    high: "2",
    critical: "3",
    urgent: "3",
  };
  return map[String(priority || "").toLowerCase()] || "1";
}

function toDatetimeLocal(iso) {
  if (!iso) return "";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "";
  const pad = (n) => String(n).padStart(2, "0");
  return (
    `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}` +
    `T${pad(d.getHours())}:${pad(d.getMinutes())}`
  );
}

// TASK ACTIONS
// =============================================================================
