/* Vibe Run Dock — split from vibe/vibe-run.js for #1731 */

/**
 * Vibe Run Dock (#806, #807) — live run card, progress narration,
 * multi-agent TODO board, pipeline strip, grounding sources, sessions
 * resume/fork and team-run chips. Polling-based: /api/vibe/run|metrics|
 * events|pipeline|sessions|teams (backend channels confirmed live; the
 * WS vibe_progress broadcast has no subscriber, so events are optional).
 */
"use strict";


    var state = {
        runId: null,
        run: null,
        metrics: null,
        events: [],
        pipeline: null,
        progress: 0,
        phase: "planning",
        stageDone: {},
        stageActive: null,
        pollTimer: null,
        tickTimer: null,
        teamTimer: null,
        useCase: "software_development",
        loadedPipeline: false,
        paused: false,
    };

    // Runs already announced as finished, so pollRun + the runs-list watcher
    // never double-fire (chat-originated runs show up only in the list).
    var announcedFinished = {};

    // Last seen state per run id, so the runs-list watcher only announces
    // runs that TRANSITION to completed during this page session — never
    // long-finished history (which would re-target the Browser pointlessly).
    var seenRunStates = {};

    // #1408 — announce a terminal run so the shell can refresh the open
    // integrated apps (Browser preview, Editor tree, Canvas project.draw,
    // Database schema) once, per run.
    function announceRunFinished(run) {
        if (!run || !run.run_id) return;
        if (announcedFinished[String(run.run_id)]) return;
        announcedFinished[String(run.run_id)] = 1;
        document.dispatchEvent(new CustomEvent("gb:vibe-run-finished", {
            detail: {
                project: run.project_id || null,
                name: run.project_name || "",
                run_id: run.run_id,
            },
        }));
    }

    // Ribbon status lives in the Vibe main window (commands + project list).
    function updateRibbonStatus(text, kind) {
        var el = q("vibeRibbonStatus");
        if (!el) return;
        el.textContent = text || "";
        el.className = "vibe-ribbon-status" + (kind ? " " + kind : "");
        var runBtn = q("vibeRunBtn");
        var pauseBtn = q("vibePauseBtn");
        var stopBtn = q("vibeStopBtn");
        if (runBtn) runBtn.classList.toggle("active", state.runId && state.run && ["running", "pending"].indexOf(String(state.run.state)) !== -1 && !state.paused);
        if (pauseBtn) pauseBtn.classList.toggle("active", state.paused || (state.run && String(state.run.state) === "awaiting_approval"));
        if (stopBtn) stopBtn.classList.toggle("active", state.runId && state.run && ["running", "awaiting_approval", "pending"].indexOf(String(state.run.state)) !== -1);
    }

    function q(id) {
        return document.getElementById(id);
    }

    function vibeRunEsc(s) {
        var d = document.createElement("div");
        d.textContent = s == null ? "" : String(s);
        return d.innerHTML;
    }

    function money(cents) {
        return "$" + (cents / 100).toFixed(2);
    }

    function shortRunId(id) {
        return id ? "#" + String(id).substring(0, 4) : "—";
    }

    function setText(id, text) {
        var el = q(id);
        if (el) el.textContent = text;
    }

    function chipState(state) {
        var s = String(state || "pending").toLowerCase();
        var map = {
            pending: "state-pending",
            running: "state-running",
            awaiting_approval: "state-awaiting_approval",
            completed: "state-completed",
            failed: "state-failed",
            cancelled: "state-cancelled",
        };
        return map[s] || "state-pending";
    }

    function phaseForStep(step) {
        var s = String(step || "").toLowerCase();
        if (s.indexOf("pipeline:") === 0) return null;
        if (s.indexOf("team:member:") === 0) return null;
        if (s === "plan" || s === "planning" || s === "analyze") return "planning";
        if (s === "act" || s === "acting" || s === "execute" || s === "executing") return "acting";
        if (s === "verify" || s === "verifying" || s === "test" || s === "testing") return "verifying";
        if (s === "reflect" || s === "reflecting" || s === "review" || s === "reviewing") return "reflecting";
        if (s === "awaiting_approval" || s === "approval") return "waiting approval";
        if (s.indexOf("build") === 0) return "building";
        if (s.indexOf("deploy") === 0) return "deploying";
        return null;
    }

    function defaultPhase(runState) {
        var s = String(runState || "").toLowerCase();
        switch (s) {
            case "pending": return "planning";
            case "running": return "acting";
            case "awaiting_approval": return "waiting approval";
            case "completed": return "verified";
            case "failed": return "failed";
            case "cancelled": return "cancelled";
            default: return "planning";
        }
    }

    function stageKindTool(kind) {
        var k = String(kind || "").toLowerCase();
        if (k.indexOf("classify") === 0) return "classify_intent";
        if (k.indexOf("compile") === 0) return "compile_plan";
        if (k.indexOf("execute") === 0) return "execute_plan";
        return null;
    }

    function api(path, opts) {
        opts = opts || {};
        opts.headers = Object.assign({}, opts.headers || {});
        var token =
            (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) ||
            localStorage.getItem("token") ||
            localStorage.getItem("id_token") ||
            localStorage.getItem("gb-access-token") ||
            sessionStorage.getItem("gb-access-token") ||
            "";
        if (token) opts.headers.Authorization = "Bearer " + token;
        return fetch(path, opts).then(function (r) {
            return r.json().catch(function () {
                return { success: false, error: "HTTP " + r.status };
            });
        });
    }

    function uiMsg(text) {
        if (typeof vibeSafeMsg === "function") {
            vibeSafeMsg("system", text);
        } else if (typeof vibeAddMsg === "function") {
            vibeAddMsg("system", text);
        }
    }

    // #1396 — remember the last chat milestone per run in sessionStorage so a
    // botserver restart (which resets the server-side poller state) does not
    // re-send a milestone the user already saw.
    function rememberMilestone(runId, label) {
        try { sessionStorage.setItem("vibe-milestone-" + runId, label); } catch (ignore) { }
    }

    function isStaleMilestone(runId, label) {
        try { return sessionStorage.getItem("vibe-milestone-" + runId) === label; } catch (ignore) { return false; }
    }
