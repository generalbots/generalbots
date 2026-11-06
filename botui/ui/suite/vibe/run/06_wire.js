/* Vibe Run Dock — split from vibe/vibe-run.js for #1731 */

    /* ------------------------------------------------- wiring */

    function wire() {
        var toggle = q("vibeRunDockToggle");
        if (toggle) {
            toggle.addEventListener("click", function () {
                var dock = q("vibeRunDock");
                var collapsed = dock.classList.toggle("collapsed");
                q("vibeRunDockArrow").textContent = collapsed ? "▸" : "▾";
            });
        }
        // Collapsible sections (Sessions / Runs / Team runs).
        document.querySelectorAll("[data-rd-collapse]").forEach(function (h) {
            h.addEventListener("click", function () {
                var wrap = q(h.getAttribute("data-rd-collapse"));
                if (!wrap) return;
                var open = wrap.style.display !== "none";
                wrap.style.display = open ? "none" : "";
                var arrow = h.querySelector(".vibe-rd-arrow");
                if (arrow) arrow.textContent = open ? "▸" : "▾";
            });
        });
        var previewBtn = q("vibePreviewBtn");
        if (previewBtn) previewBtn.addEventListener("click", previewProject);
        document.addEventListener("gb:vibe-run", function (e) {
            if (e.detail && e.detail.run_id) runFocus(e.detail.run_id);
        });
        // #run-done — the Runner Log "open the app" deeplinks are injected
        // after completion (showDoneDeeplink); one delegated listener covers
        // every run card without re-binding per message.
        if (!document.documentElement.dataset.vibeRunDoneWired) {
            document.documentElement.dataset.vibeRunDoneWired = "true";
            document.addEventListener("click", function (event) {
                var link = event.target && typeof event.target.closest === "function"
                    ? event.target.closest("a[data-vibe-open-app], a[data-vibe-open-prod]")
                    : null;
                if (!link) return;
                event.preventDefault();
                var pid = link.getAttribute("data-vibe-open-app") || link.getAttribute("data-vibe-open-prod");
                if (!pid) return;
                if (link.hasAttribute("data-vibe-open-app")) {
                    if (window.VibeShell && window.VibeShell.toolbar &&
                        typeof window.VibeShell.toolbar.openProjectApp === "function") {
                        window.VibeShell.toolbar.openProjectApp(pid);
                    }
                } else {
                    // Same run-scoped guard as the auto-open above: if the
                    // tab already opened at completion, this is a no-op.
                    openProdTab(link.getAttribute("data-vibe-open-prod-name") || "", pid, state.runId);
                }
            });
        }
        loadRuns();
        loadSessions();
        loadTeams();
    }

    function init() {
        if (document.readyState === "loading") {
            document.addEventListener("DOMContentLoaded", wire);
        } else {
            wire();
        }
    }

    /* ------------------------------------------------ transport (VB6) */

    function transportStatus() {
        if (!state.runId || !state.run) return "IDLE";
        if (state.paused) return "PAUSED";
        var s = String(state.run.state || "").toUpperCase();
        return s === "PENDING" ? "ACTIVE" : s;
    }

    function play() {
        if (state.paused) {
            state.paused = false;
            uiMsg("▶ Run resumed.");
            updateRibbonStatus("RESUMED", "ok");
            pollRun();
            return;
        }
        if (state.runId && (st === "running" || st === "pending")) {
            updateRibbonStatus("RUNNING", "running");
            if (window.VibeWindows) window.VibeWindows.openRunDock();
            return;
        }
        // Idle / finished: a selected project can always be run directly.
        // With the runner chat removed, Run always executes the deterministic
        // project verification intent; prompts belong in the Chat app (@app).
        var projectName = typeof currentProject !== "undefined" && currentProject ? String(currentProject) : "";
        // Avoid the doubled "...selected project selected project" when no
        // project is currently selected (projectName is the verbatim fallback).
        var text = projectName
            ? "Run and verify the selected project " + projectName
            : "Run and verify the selected project";
        // Fully automated like freebuff: Run executes all tools directly,
        // no gates, decisions reported through the chat narration.
        start(text, { auto_approve: true }).then(function () {
            updateRibbonStatus("RUNNING", "running");
            // Surface the execution board when starting a fresh run: the run
            // card only exists inside the on-demand-fetched Run Dock, so a
            // run started from the ribbon stayed invisible until the user
            // manually opened the dock ("Play does nothing").
            if (window.VibeWindows && typeof window.VibeWindows.openRunDock === "function") {
                window.VibeWindows.openRunDock();
            }
            // Show the app in the Browser window IMMEDIATELY — waiting for
            // run completion (which can take minutes) left the user staring
            // at an empty desktop with no feedback ("can't see the app").
            // openProjectApp re-runs at completion and refreshes the result.
            var pid = typeof currentProjectId !== "undefined" ? currentProjectId : null;
            if (pid && window.VibeShell && window.VibeShell.toolbar &&
                typeof window.VibeShell.toolbar.openProjectApp === "function") {
                window.VibeShell.toolbar.openProjectApp(pid);
            }
        }).catch(function () {
            updateRibbonStatus("START FAILED", "error");
        });
    }

    // VB6 "Break": the run holds at its next checkpoint (tool-call budget
    // check). The server keeps the run in flight; the transport shows PAUSED
    // and Play resumes it.
    function pause() {
        if (!state.runId || !state.run) {
            updateRibbonStatus("NO RUN TO PAUSE", "hint");
            return;
        }
        state.paused = true;
        uiMsg("⏸ Paused — the run will hold at its next checkpoint.");
        updateRibbonStatus("PAUSED — HOLDS AT NEXT CHECKPOINT", "paused");
    }

    // Publish to production: same agent loop as Run, but the run executes the
    // approval-gated deploy pipeline (server resolves pipeline_mode="deploy"
    // to the deploy graph). Run stays the dev/test action; Deploy is the only
    // path that touches production.
    function deploy() {
        if (state.runId && hasActiveRun()) {
            uiMsg("⏳ A run is already active — stop it before deploying.");
            updateRibbonStatus("RUN IN PROGRESS", "hint");
            return;
        }
        var projectName = typeof currentProject !== "undefined" && currentProject ? String(currentProject) : "";
        var text = projectName
            ? "Deploy the project " + projectName + " to production"
            : "Deploy the selected project to production";
        uiMsg("🚀 Deploying " + projectName + " to production…");
        // Deploy is fully automated — no approval gates anywhere in the
        // pipeline; failures and decisions surface in the Vibe chat.
        start(text, { pipeline_mode: "deploy", auto_approve: true }).then(function () {
            updateRibbonStatus("DEPLOYING", "running");
            if (window.VibeWindows && typeof window.VibeWindows.openRunDock === "function") {
                window.VibeWindows.openRunDock();
            }
        }).catch(function () {
            updateRibbonStatus("DEPLOY FAILED", "error");
        });
    }

    function hasActiveRun() {
        if (!state.runId || !state.run) return false;
        return ["pending", "running"].indexOf(String(state.run.state)) !== -1;
    }

    function runStop() {
        if (!state.runId) {
            updateRibbonStatus("NO RUN TO STOP", "hint");
            return;
        }
        state.paused = false;
        api("/api/vibe/run/" + state.runId + "/cancel", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ reason: "Stopped by user" }),
        }).then(function () {
            uiMsg("\u23F9 Run stopped.");
            pollRun();
        });
        runClosePreview();
        updateRibbonStatus("STOPPED", "stop");
    }

    init();

    // Wire the run state into the ribbon status on every poll.
    var _origPollRun = pollRun;
    pollRun = function () {
        _origPollRun();
        updateRibbonStatus(state.paused ? "PAUSED" : transportStatus(), state.paused ? "paused" : undefined);
    };

    window.VibeRun = {
        start: start,
        focus: runFocus,
        onProgress: onProgress,
        preview: previewProject,
        play: play,
        deploy: deploy,
        pause: pause,
        stop: runStop,
    };
    window.VibeTransport = {
        play: play,
        deploy: deploy,
        pause: pause,
        stop: runStop,
        hasActiveRun: hasActiveRun,
    };
