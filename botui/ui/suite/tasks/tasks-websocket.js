
function handleWebSocketMessage(data) {
  gbLog.debug("[Tasks WS] handleWebSocketMessage called with type:", data.type);

  switch (data.type) {
    case "connected":
      gbLog.debug("[Tasks WS] Connected to task progress stream");
      addAgentLog("info", "[SYSTEM] Task progress stream connected");
      break;

    case "task_started":
      gbLog.debug("[Tasks WS] TASK_STARTED:", data.message);
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
      gbLog.debug(
        "[Tasks WS] TASK_PROGRESS - step:",
        data.step,
        "message:",
        data.message,
      );
      addAgentLog("info", `[${data.step}] ${data.message}`);

      // Auto-select this task if none selected
      if (data.task_id && !TasksState.selectedTaskId) {
        gbLog.debug(
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
      gbLog.debug("[Tasks WS] TASK_COMPLETED:", data.message);
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
      gbLog.debug("[Tasks WS] TASK_ERROR:", data.error || data.message);
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
      gbLog.debug("[Tasks WS] LLM streaming...");
      break;

    case "llm_generating":
      gbLog.debug("[Tasks WS] LLM_GENERATING:", data.message);
      addAgentLog("info", `[AI] ${data.message}`);
      // Update STATUS section with AI progress
      updateStatusFromProgress(data.message, "llm_generating");
      // Update terminal
      updateDetailTerminal(data.task_id, data.message, "llm_generating");
      break;

    case "llm_complete":
      gbLog.debug("[Tasks WS] LLM_COMPLETE:", data.message);
      addAgentLog("success", `[AI] ${data.message}`);
      // Update STATUS section
      updateStatusFromProgress(data.message, "llm_complete");
      // Update terminal
      updateDetailTerminal(data.task_id, `✓ ${data.message}`, "success");
      break;

    case "manifest_update":
      gbLog.debug(
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
        gbLog.debug("[Tasks WS] Auto-selecting task:", data.task_id);
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
      gbLog.debug(
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
  forceStore = false
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
    gbLog.debug(
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
  gbLog.debug(
    "[MANIFEST] Looking for progress-log-" + taskId + ", found:",
    !!progressLog,
  );
  if (!progressLog) {
    progressLog = document.getElementById("window-tasks").querySelector(".taskmd-progress-content");
    gbLog.debug(
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
    gbLog.debug("[Manifest] No sections in manifest, skipping render");
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
    gbLog.debug("[Manifest] Removing progress-empty placeholder");
    emptyPlaceholder.remove();
  }

  let tree = progressLog.querySelector(".taskmd-tree");
  gbLog.debug("[Manifest] Existing tree found:", !!tree);

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
      gbLog.debug(
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
      gbLog.debug("[Manifest] === REBUILDING TREE FROM SCRATCH ===");
      const treeHTML = buildProgressTreeHTML(manifest, totalSteps);
      gbLog.debug(
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
        }, 100);
      const expandedCount = progressLog.querySelectorAll(".expanded").length;
      const childCount = progressLog.querySelectorAll(".tree-child").length;
      const itemCount = progressLog.querySelectorAll(".tree-item").length;
      gbLog.debug(
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
      gbLog.debug("[Manifest] Updating tree in place");
      updateProgressTreeInPlace(tree, manifest, totalSteps);
      gbLog.debug("[Manifest] Tree updated successfully");
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
    }, 100);
  }
}

// Update STATUS section from task_progress messages
function updateStatusFromProgress(message, step) {
  const statusContent = document.getElementById("window-tasks").querySelector(".taskmd-status-content");
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
  const statusContent = document.getElementById("window-tasks").querySelector(".taskmd-status-content");
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
