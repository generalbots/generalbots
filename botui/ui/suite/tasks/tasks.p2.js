
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
