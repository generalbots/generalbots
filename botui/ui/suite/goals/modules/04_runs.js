/* Goals app — module 4/6: goal runs (#1706) (split from goals/goals.js for #1731) */

(function (app) {
  "use strict";

  const { state, escapeHtml, humanizeText, showNotification } = app;

  // =============================================================================
  // GOAL RUNS (#1706)
  // =============================================================================

  const RUNS_POLL_MS = 10000;
  let runsPollTimer = null;
  let runsRequestSeq = 0;

  function startRunsPolling() {
    stopRunsPolling();
    runsPollTimer = setInterval(loadGoalRuns, RUNS_POLL_MS);
  }

  function stopRunsPolling() {
    if (runsPollTimer) {
      clearInterval(runsPollTimer);
      runsPollTimer = null;
    }
  }

  async function loadGoalRuns() {
    const container = document.getElementById("goal-runs-list");
    if (!container) return;

    // Runs are listed per objective, so fan out over the objectives currently
    // loaded in the sidebar and merge the results.
    const seq = ++runsRequestSeq;
    try {
      const objectives = state.objectives.length
        ? state.objectives
        : await fetch("/api/goals/objectives")
            .then((r) => (r.ok ? r.json() : []))
            .then((d) => (Array.isArray(d) ? d : d.objectives || []));
      if (!state.objectives.length) state.objectives = objectives;

      const batches = await Promise.all(
        objectives.map((obj) =>
          fetch(`/api/goals/runs?objective_id=${encodeURIComponent(obj.id)}`)
            .then((r) => (r.ok ? r.json() : { runs: [] }))
            .then((d) => {
              const runs = Array.isArray(d) ? d : d.runs || [];
              return runs.map((run) => ({ ...run, objective_title: obj.title }));
            })
            .catch(() => []),
        ),
      );
      if (seq !== runsRequestSeq) return; // a newer refresh won the race
      const runs = batches.flat().sort((a, b) => {
        if (a.status === b.status) return b.checkpoints - a.checkpoints;
        const rank = { running: 0, paused: 1, completed: 2, stopped: 3, failed: 4 };
        return (rank[a.status] ?? 9) - (rank[b.status] ?? 9);
      });
      renderGoalRuns(runs);
    } catch (e) {
      console.error("Failed to load goal runs:", e);
      container.innerHTML =
        '<p class="empty-message">Could not load goal runs.</p>';
    }
  }

  function renderGoalRuns(runs) {
    const container = document.getElementById("goal-runs-list");
    if (!container) return;

    if (!runs.length) {
      container.innerHTML =
        '<p class="empty-message">No goal runs yet. Runs are created by the agent and API (POST /api/goals/runs).</p>';
      return;
    }

    container.innerHTML = runs
      .map((run) => {
        // #1979 — values are authored by API callers/agents and can arrive as
        // objects or JSON strings; the card shows them as words, not JSON.
        const last = humanizeText(run.last_value) || "—";
        const target = humanizeText(run.target_value) || "—";
        const progress = Number(run.progress_percent || 0);
        const canPause = run.status === "running";
        const canResume = run.status === "paused";
        const canStop = run.status === "running" || run.status === "paused";
        return `
        <article class="run-card" data-run-id="${run.id}">
          <header class="run-head">
            <span class="run-objective">${escapeHtml(run.objective_title || "Objective")}</span>
            <span class="run-status run-status-${escapeHtml(run.status)}">${escapeHtml(run.status)}</span>
          </header>
          <div class="run-progress">
            <div class="progress-bar">
              <div class="progress-fill" style="width: ${Math.min(100, Math.max(0, progress))}%"></div>
            </div>
            <span class="run-progress-text">${Math.round(progress)}%</span>
          </div>
          <div class="run-meta">
            <span>checkpoints <strong>${run.checkpoints}</strong></span>
            <span>value <strong>${escapeHtml(last)}</strong> / ${escapeHtml(target)}</span>
            <span>every <strong>${run.interval_seconds}s</strong></span>
            <span class="run-id" title="${run.id}">${String(run.id).slice(0, 8)}</span>
          </div>
          <div class="run-actions">
            ${canPause ? `<button class="btn-run btn-run-pause" onclick="pauseGoalRun('${run.id}')">Pause</button>` : ""}
            ${canResume ? `<button class="btn-run btn-run-resume" onclick="resumeGoalRun('${run.id}')">Resume</button>` : ""}
            ${canStop ? `<button class="btn-run btn-run-stop" onclick="stopGoalRun('${run.id}')">Stop</button>` : ""}
          </div>
        </article>`;
      })
      .join("");
  }

  async function runAction(runId, action) {
    try {
      const response = await fetch(
        `/api/goals/runs/${encodeURIComponent(runId)}/${action}`,
        { method: "POST" },
      );
      if (!response.ok) {
        const payload = await response.json().catch(() => ({}));
        showNotification(
          payload.error || `Could not ${action} the goal run`,
          "error",
        );
        return;
      }
      showNotification(
        { pause: "Goal run paused", resume: "Goal run resumed", stop: "Goal run stopped" }[action] ||
          "Goal run updated",
        "success",
      );
      await loadGoalRuns();
    } catch (e) {
      console.error(`Failed to ${action} goal run:`, e);
      showNotification(`Could not ${action} the goal run`, "error");
    }
  }

  function pauseGoalRun(runId) {
    return runAction(runId, "pause");
  }

  function resumeGoalRun(runId) {
    return runAction(runId, "resume");
  }

  function stopGoalRun(runId) {
    return runAction(runId, "stop");
  }

  Object.assign(app, {
    startRunsPolling,
    stopRunsPolling,
    loadGoalRuns,
    renderGoalRuns,
    pauseGoalRun,
    resumeGoalRun,
    stopGoalRun,
  });
})(window.__gbGoals = window.__gbGoals || {});
