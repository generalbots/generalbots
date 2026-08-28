/**
 * Vibe Run Dock (#806, #807) — live run card, approval resume UX,
 * multi-agent TODO board, pipeline strip, grounding sources, sessions
 * resume/fork and team-run chips. Polling-based: /api/vibe/run|metrics|
 * events|pipeline|sessions|teams (backend channels confirmed live; the
 * WS vibe_progress broadcast has no subscriber, so events are optional).
 */
(function () {
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

    function esc(s) {
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

    /* ------------------------------------------------- repatriation */

    function syncRunDock() {
        var dock = q("vibeRunDockState");
        if (!dock || !state.run) return;
        var label = {
            pending: "ACTIVE",
            running: "ACTIVE",
            awaiting_approval: "APPROVAL",
            completed: "COMPLETED",
            failed: "FAILED",
            cancelled: "CANCELLED",
        }[String(state.run.state)] || "IDLE";
        dock.textContent = label;
        dock.className = "vibe-chip " + chipState(state.run.state);
    }

    function renderRunCard() {
        if (!state.run) return;
        var card = q("vibeRunCard");
        if (card) card.classList.add("visible");
        setText("vibeRunId", "RUN " + shortRunId(state.runId));
        setText("vibeRunIntent", state.run.intent || "—");
        setText("vibeRunToolCalls", (state.run.tool_call_count || 0) + " tool calls");
        setText("vibeRunOutcome", "");
        var stateEl = q("vibeRunState");
        if (stateEl) {
            stateEl.textContent = state.run.state;
            stateEl.className = "vibe-chip " + chipState(state.run.state);
        }
        // Terminal states freeze the phase and the run-dock ribbon so the
        // dock never lingers on "PLANNING"/"ACTIVE" after completion — and
        // the elapsed clock stops at the real duration.
        if (["completed", "failed", "cancelled"].indexOf(String(state.run.state)) !== -1) {
            state.phase = defaultPhase(state.run.state);
            freezeElapsed();
        }
        syncRunDock();
        var phaseEl = q("vibeRunPhase");
        if (phaseEl) {
            phaseEl.textContent = String(state.phase || defaultPhase(state.run.state)).toUpperCase();
        }
        var fill = q("vibeRunProgress");
        if (fill) fill.style.width = Math.max(2, Math.min(100, state.progress)) + "%";
        if (state.run.error) {
            setText("vibeRunOutcome", "❌ " + state.run.error);
        } else if (state.run.state === "completed") {
            setText("vibeRunOutcome", "✅ completed");
        }
        renderTodoBoard();
        renderApproval();
    }

    function renderTodoBoard() {
        var list = q("vibeTodoList");
        if (!list) return;
        var items = [];
        if (state.pipeline && Array.isArray(state.pipeline.stages)) {
            items = state.pipeline.stages.map(function (stage) {
                var key = stageKindTool(stage.kind) || stage.id;
                var done = !!state.stageDone[stage.id] || !!state.stageDone[key];
                var active = state.stageActive === stage.id;
                return { label: stage.name || stage.id, state: done ? "done" : (active ? "active" : "pending") };
            });
        }
        if (!items.length && state.events.length) {
            items = state.events.slice(-8).map(function (event) {
                return { label: event.tool_name || event.event_type || event.step || "agent task", state: "done" };
            });
        }
        if (!items.length) {
            items = [{ label: "Plan project change", state: "pending" }, { label: "Implement with agent", state: "pending" }, { label: "Verify result", state: "pending" }];
        }
        list.innerHTML = items.map(function (item) {
            var icon = item.state === "done" ? "✓" : item.state === "active" ? "▶" : "○";
            return '<div class="vibe-todo-item ' + esc(item.state) + '"><span class="vibe-todo-icon">' + icon + '</span><span>' + esc(item.label) + '</span></div>';
        }).join("");
    }

    function loadPipeline() {
        if (state.loadedPipeline) return;
        state.loadedPipeline = true;
        var useCase = (state.run && state.run.use_case) || state.useCase;
        api("/api/vibe/pipeline/" + useCase).then(function (data) {
            if (data.success && data.pipeline && data.pipeline.stages) {
                state.pipeline = data.pipeline;
                renderPipelineStrip();
            }
        });
    }

    function renderPipelineStrip() {
        if (!state.pipeline) return;
        var sec = q("vibePipelineSection");
        if (sec) sec.style.display = "block";
        var strip = q("vibePipelineStrip");
        if (!strip) return;
        strip.innerHTML = state.pipeline.stages.map(function (st) {
            var tool = stageKindTool(st.kind);
            var cls = "pending";
            if (state.stageActive === st.id) cls = "active";
            else if (tool && state.stageDone[tool]) cls = "done";
            return '<span class="vibe-stage-chip ' + cls + '">' + esc(st.name || st.id) + "</span>";
        }).join("");
    }

    /* ------------------------------------------------- approval */

    function renderApproval() {
        var card = q("vibeApprovalCard");
        if (!card) return;
        if (!state.run || state.run.state !== "awaiting_approval") {
            card.classList.remove("visible");
            return;
        }
        card.classList.add("visible");
        var tools = q("vibeApprovalTools");
        if (tools) {
            var named = state.events
                .filter(function (e) { return e && e.tool_name; })
                .map(function (e) { return e.tool_name; });
            named = Array.from(new Set(named)).slice(-3);
            tools.innerHTML =
                "This run requires approval for " +
                (state.run.tool_call_count || 0) +
                " tool call(s)." +
                (named.length ? "<br/>Recent tools: <b>" + esc(named.join(", ")) + "</b>" : "");
        }
    }

    function approveRun() {
        var btn = q("vibeApproveBtn");
        if (btn) {
            btn.disabled = true;
            btn.textContent = "Resuming…";
        }
        api("/api/vibe/run/" + state.runId + "/approve", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: "{}",
        }).then(function (data) {
            var msg = data && data.message ? String(data.message) : "";
            // The backend guards terminal states: a late approval still
            // records the pending tool calls but reports the run already
            // finished. Reflect that instead of leaving the dock stuck on
            // "Resuming…" forever.
            if (msg.indexOf("already finished") !== -1) {
                resetApproveBtn();
                uiMsg("✅ Approval recorded — run already finished.");
                pollRun();
                return;
            }
            uiMsg("✅ Approval sent — resuming run.");
            waitForResume();
        });
    }

    function resetApproveBtn() {
        var btn = q("vibeApproveBtn");
        if (btn) {
            btn.disabled = false;
            btn.textContent = "✓ Approve & Resume";
        }
    }

    function denyRun() {
        api("/api/vibe/run/" + state.runId + "/cancel", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ reason: "Denied by user" }),
        }).then(function () {
            uiMsg("✕ Run denied and cancelled.");
            pollRun();
        });
    }

    function waitForResume() {
        var tries = 0;
        var t = setInterval(function () {
            tries++;
            api("/api/vibe/run/" + state.runId).then(function (data) {
                if (!data || !data.run_id) return;
                if (String(data.state).toLowerCase() !== "awaiting_approval") {
                    clearInterval(t);
                    state.run = data;
                    state.phase = state.phase === "waiting approval" ? "acting" : state.phase;
                    renderRunCard();
                } else if (tries >= 40) {
                    clearInterval(t);
                    resetApproveBtn();
                }
            });
        }, 750);
    }

    /* ------------------------------------------------- sources */

    function renderSources() {
        api("/api/vibe/run/" + state.runId + "/grounding").then(function (data) {
            var items = [];
            if (data && data.success && Array.isArray(data.sources) && data.sources.length) {
                items = data.sources;
            } else {
                items = fallbackSources();
            }
            if (!items.length) return;
            q("vibeSourcesSection").style.display = "block";
            var list = q("vibeSourcesList");
            if (!list) return;
            list.innerHTML = items.map(function (s) {
                var label = typeof s === "string" ? s : (s.label || s.name || s.source || s.type || "source");
                var detail = typeof s === "object" && s !== null ? (s.detail || s.url || s.path || "") : "";
                return '<div class="vibe-src-item"><span>📎</span><span>' + esc(label) + (detail ? " · " + esc(detail) : "") + "</span></div>";
            }).join("");
        });
    }

    function fallbackSources() {
        var items = [];
        var named = state.events.filter(function (e) { return e && e.tool_name; })
            .map(function (e) { return e.tool_name; });
        Array.from(new Set(named)).slice(0, 6).forEach(function (t) {
            items.push({ label: "tool: " + t });
        });
        if (state.run && state.run.intent) {
            items.push({ label: "intent", detail: state.run.intent.substring(0, 80) });
        }
        return items;
    }

    /* ------------------------------------------------- polling */

    function startPolling() {
        if (window.GBAppLifecycle) GBAppLifecycle.begin("vibe");
        stopPolling();
        state.pollTimer = (window.GBAppLifecycle ? GBAppLifecycle.interval("vibe", loadAll, 1500) : setInterval(loadAll, 1500));
    }

    function stopPolling() {
        if (state.pollTimer) {
            clearInterval(state.pollTimer);
            state.pollTimer = null;
        }
    }

    function loadAll() {
        if (!state.runId) return;
        pollRun();
    }

    // #1190 — animated execution overlay over the Vibe window while a run is
    // executing; hidden on terminal states.
    function syncExecOverlay() {
        var overlay = q("vibeExecOverlay");
        if (!overlay) return;
        var active = state.run && ["running", "pending"].indexOf(String(state.run.state)) !== -1;
        overlay.style.display = active ? "flex" : "none";
        overlay.classList.toggle("hidden", !active);
    }

    function pollRun() {
        api("/api/vibe/run/" + state.runId).then(function (data) {
            if (!data || !data.run_id) return;
            var changed = !state.run || String(data.state) !== String(state.run.state);
            state.run = data;
            renderRunCard();
            syncExecOverlay();
            if (changed) {
                // Keep the RUNS list truthful: the dock polls the focused run,
                // but the list only refreshed on focus/start — so a run that
                // just completed would stay as a stale "RUNNING 0 calls" entry.
                loadRuns();
                if (data.state === "awaiting_approval") {
                    uiMsg("⏸ Run is waiting for approval — see the Run Dock.");
                } else if (data.state === "completed") {
                    uiMsg("✅ Run completed.");
                    stopPolling();
                    // #1271 — a chat message that changed the app should end
                    // with the result visible: open the project's app in the
                    // Browser window on the dev VM (skipped for deploy runs,
                    // which ship to production and are verified there).
                    var runProjectId = data.project_id;
                    var isDeploy = String(data.pipeline_mode || "").indexOf("deploy") !== -1;
                    if (runProjectId && !isDeploy) {
                        if (window.VibeShell && window.VibeShell.toolbar &&
                            typeof window.VibeShell.toolbar.openProjectApp === "function") {
                            window.VibeShell.toolbar.openProjectApp(runProjectId);
                        }
                    }
                } else if (data.state === "failed" || data.state === "cancelled") {
                    uiMsg("⛔ Run " + data.state + ".");
                    stopPolling();
                }
            }
            api("/api/vibe/metrics/" + state.runId).then(function (m) {
                if (m && m.success && m.metrics) {
                    state.metrics = m.metrics;
                }
            });
            if (!state.loadedPipeline) loadPipeline();
            if (data.state === "running" || data.state === "awaiting_approval") {
                api("/api/vibe/events/" + state.runId).then(function (ev) {
                    if (ev && Array.isArray(ev.events)) {
                        if (ev.events.length !== state.events.length) {
                            state.events = ev.events;
                            renderSources();
                            renderApproval();
                        }
                    } else if (Array.isArray(ev)) {
                        state.events = ev;
                    }
                });
            }
        });
    }

    function startTicker() {
        if (state.tickTimer) clearInterval(state.tickTimer);
        var tick = function () {
            if (!state.run || !state.run.created_at) return;
            // A finished run must freeze the clock, not keep counting (#run-timer).
            var runState = String(state.run.state || "").toLowerCase();
            if (["completed", "failed", "cancelled"].indexOf(runState) !== -1) {
                freezeElapsed();
                return;
            }
            var start = new Date(state.run.created_at).getTime();
            if (isNaN(start)) return;
            var s = Math.max(0, Math.floor((Date.now() - start) / 1000));
            setText("vibeRunElapsed", s + "s");
        };
        state.tickTimer = window.GBAppLifecycle
            ? GBAppLifecycle.interval("vibe", tick, 1000)
            : setInterval(tick, 1000);
    }

    function stopTicker() {
        if (state.tickTimer) {
            clearInterval(state.tickTimer);
            state.tickTimer = null;
        }
    }

    // Final elapsed for a terminal run: prefer the backend's completed_at so
    // the frozen value is the true duration, not "now minus start".
    function freezeElapsed() {
        stopTicker();
        if (!state.run || !state.run.created_at) return;
        var start = new Date(state.run.created_at).getTime();
        var end = state.run.completed_at ? new Date(state.run.completed_at).getTime() : NaN;
        if (isNaN(start)) return;
        if (isNaN(end)) end = Date.now();
        setText("vibeRunElapsed", Math.max(0, Math.floor((end - start) / 1000)) + "s");
    }

    /* ------------------------------------------------- focus */

    function focus(runId) {
        if (!runId) return;
        state.runId = String(runId);
        state.run = null;
        state.metrics = null;
        state.events = [];
        state.progress = 0;
        state.phase = "planning";
        state.stageDone = {};
        state.stageActive = null;
        var card = q("vibeRunCard");
        if (card) card.classList.remove("visible");
        var approval = q("vibeApprovalCard");
        if (approval) approval.classList.remove("visible");
        var sources = q("vibeSourcesSection");
        if (sources) sources.style.display = "none";
        setText("vibeRunId", "RUN " + shortRunId(state.runId));
        renderRunCard();
        startPolling();
        startTicker();
        loadRuns();
        loadSessions();
    }

    /* ------------------------------------------------- run create */

    function start(intent, opts) {
        opts = opts || {};
        uiMsg("🔄 Starting Vibe run: " + intent.substring(0, 80) + "…");
        // Attach the currently selected project (if any) so the agent edits
        // the right workspace and the deploy pipeline targets the right VM.
        var pid = null;
        if (typeof currentProjectId !== "undefined" && currentProjectId) pid = currentProjectId;
        var pname = null;
        if (typeof currentProject !== "undefined" && currentProject) pname = currentProject;
        var activeBotName =
            (typeof vibeBotName !== "undefined" && vibeBotName) ||
            window.__INITIAL_BOT_NAME__ ||
            "default";
        var body = {
            intent: intent,
            // The default bot is resolved server-side from the authenticated
            // session. Sending its UUID as an explicit target incorrectly
            // triggers cross-bot RBAC and blocks legitimate project edits.
            bot_id: activeBotName !== "default" && typeof vibeBotId !== "undefined" && vibeBotId
                ? vibeBotId
                : null,
            use_case: state.useCase,
            // #919 — default to manual approval for destructive tools; the
            // server only honors auto_approve for administrators.
            auto_approve: opts.auto_approve === true,
            // "deploy" routes the run through the approval-gated deploy
            // pipeline (publish to production); omitted/other values test in
            // development. Run = dev, Deploy = prod — the only two paths.
            pipeline_mode: opts.pipeline_mode || null,
            project_id: pid,
            project_name: pname,
        };
        return api("/api/vibe/run", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(body),
        }).then(function (data) {
            if (data.success && data.run_id) {
                // The dock state chip may not exist yet (the Run Dock window
                // is only opened after the run is created). Guard it — an
                // unguarded null deref here rejected the promise and turned
                // a successful run creation into "START FAILED".
                var dockState = q("vibeRunDockState");
                if (dockState) {
                    dockState.textContent = "ACTIVE";
                    dockState.className = "vibe-chip state-running";
                }
                focus(data.run_id);
                return { ok: true, run_id: data.run_id };
            }
            var err = data.error || ("HTTP " + (data.http_status || "?"));
            uiMsg("⚠️ Vibe run API unavailable (" + err + ") — falling back to autotask flow.");
            var dockIdle = q("vibeRunDockState");
            if (dockIdle) {
                dockIdle.textContent = "IDLE";
                dockIdle.className = "vibe-chip state-pending";
            }
            throw new Error(err);
        });
    }

    function retry(fn, key) {
        var n = (retry.count[key] = (retry.count[key] || 0) + 1);
        if (n < 30) setTimeout(fn, 1000);
    }
    retry.count = {};

    /* ------------------------------------------------- preview */

    /* #1192 — run the project's OWN custom app from its workspace instead of a
       bundled template. Falls back to the deployed preview URL when the
       workspace has no source. */
    function workspaceServeUrl(projectId) {
        if (!projectId) return Promise.resolve(null);
        return api("/api/vibe/projects/" + encodeURIComponent(projectId) + "/files")
            .then(function (data) {
                var files = (data && data.files) || [];
                var hasIndex = files.some(function (f) { return f === "index.html"; });
                if (!hasIndex) return null;
                var token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || "";
                var base = window.location.origin + "/api/vibe/projects/" + encodeURIComponent(projectId) + "/serve/index.html";
                return token ? base + "?token=" + encodeURIComponent(token) : base;
            })
            .catch(function () { return null; });
    }

    // Handle of the popup tab currently showing the live project preview.
    // Stop must close it back (pressing ■ tears down what ▶ launched).
    var previewWindowRef = null;

    function closePreview() {
        if (previewWindowRef && !previewWindowRef.closed) {
            try { previewWindowRef.close(); } catch (e) { /* cross-origin: ignore */ }
            uiMsg("⛔ Preview closed.");
        }
        previewWindowRef = null;
    }

    function previewProject() {
        var projectId = typeof currentProjectId !== "undefined" ? currentProjectId : null;
        if (!projectId) {
            uiMsg("⚠️ Select a project before opening Preview App.");
            return;
        }
        // Open synchronously so browser popup blockers do not swallow the
        // preview tab while the authenticated API request is in flight.
        previewWindowRef = window.open("about:blank", "_blank");
        var previewWindow = previewWindowRef;
        if (!previewWindow) {
            uiMsg("⚠️ Allow pop-ups to open the project preview.");
            return;
        }
        previewWindow.document.body.innerHTML = "<p style='font-family:system-ui;padding:24px'>Resolving project preview…</p>";
        workspaceServeUrl(projectId).then(function (url) {
            if (url) return url;
            return api("/api/vibe/projects/" + encodeURIComponent(projectId)).then(function (projectData) {
                var project = projectData && projectData.project;
                var env = (project && project.environment) || "development";
                return api("/api/vibe/projects/" + encodeURIComponent(projectId) + "/preview?env=" + encodeURIComponent(env));
            }).then(function (data) {
                if (typeof data === "string") return data;
                var payload = data && data.data ? data.data : data;
                var url = payload && payload.preview_url;
                if (!url || (String(url).indexOf("http://") !== 0 && String(url).indexOf("https://") !== 0)) {
                    throw new Error("No live preview is available yet. Deploy the project first.");
                }
                return url;
            });
        }).then(function (url) {
            previewWindow.location.href = url;
            uiMsg("🌐 Preview opened: " + url);
            return null;
        }).catch(function (err) {
            previewWindow.close();
            uiMsg("⚠️ Preview unavailable: " + err);
        });
    }

    /* ------------------------------------------------- sessions */

    function loadSessions() {
        var list = q("vibeSessionsList");
        if (!list) {
            retry(loadSessions, "sessions");
            return;
        }
        api("/api/vibe/sessions").then(function (data) {
            var sessions = (data && data.success && Array.isArray(data.sessions)) ? data.sessions : [];
            if (!sessions.length) {
                list.innerHTML = '<div class="vibe-rd-empty">No sessions yet.</div>';
                return;
            }
            list.innerHTML = sessions.slice(0, 6).map(function (s) {
                var runState = s.run ? s.run.state : "idle";
                var st = "Session " + String(s.session_id).substring(0, 4);
                return '<div class="vibe-session-item">' +
                    '<span class="vibe-chip ' + chipState(runState) + '">' + esc(runState) + "</span>" +
                    '<span class="meta" title="' + esc(s.intent) + '">' + esc(s.intent || st) + "</span>" +
                    '<button type="button" data-resume="' + esc(s.session_id) + '">Resume</button>' +
                    '<button type="button" data-fork="' + esc(s.session_id) + '">Fork</button>' +
                    "</div>";
            }).join("");
            list.querySelectorAll("[data-resume]").forEach(function (b) {
                b.addEventListener("click", function () {
                    resumeSession(b.getAttribute("data-resume"));
                });
            });
            list.querySelectorAll("[data-fork]").forEach(function (b) {
                b.addEventListener("click", function () {
                    forkSession(b.getAttribute("data-fork"));
                });
            });
        });
    }

    function resumeSession(sessionId) {
        uiMsg("🔄 Resuming session " + shortRunId(sessionId) + "…");
        api("/api/vibe/sessions/" + sessionId + "/resume", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: "{}",
        }).then(function (data) {
            if (data && data.success && data.run_id) {
                focus(data.run_id);
                uiMsg("✅ Session resumed — run " + shortRunId(data.run_id) + " active.");
            } else {
                uiMsg("⚠️ Resume failed: " + ((data && data.error) || "unknown"));
            }
        });
    }

    function forkSession(sessionId) {
        api("/api/vibe/sessions/" + sessionId + "/fork", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: "{}",
        }).then(function (data) {
            if (data && data.success) {
                uiMsg("🍴 Session forked.");
                loadSessions();
            } else {
                uiMsg("⚠️ Fork failed: " + ((data && data.error) || "unknown"));
            }
        });
    }

    /* ------------------------------------------------- runs list */

    function loadRuns() {
        api("/api/vibe/runs?limit=8").then(function (data) {
            var list = q("vibeRunsList");
            if (!list) return;
            var runs = Array.isArray(data) ? data : [];
            if (!runs.length) {
                list.innerHTML = '<div class="vibe-rd-empty">No runs yet.</div>';
                return;
            }
            list.innerHTML = runs.map(function (r) {
                return '<div class="vibe-run-item" style="cursor:pointer" data-run="' + esc(r.run_id) + '">' +
                    '<span class="vibe-chip ' + chipState(r.state) + '">' + esc(r.state) + "</span>" +
                    '<span class="meta" title="' + esc(r.intent) + '">' + esc(r.intent || "run") + "</span>" +
                    '<span class="meta" style="flex:0">' + (r.tool_call_count || 0) + " calls</span>" +
                    "</div>";
            }).join("");
            list.querySelectorAll("[data-run]").forEach(function (el) {
                el.addEventListener("click", function () {
                    focus(el.getAttribute("data-run"));
                });
            });
            if (!state.runId) {
                var active = runs.find(function (run) {
                    var runState = String(run.state || "").toLowerCase();
                    return runState === "running" || runState === "awaiting_approval";
                });
                if (active && active.run_id) focus(active.run_id);
            }
        });
    }

    /* ------------------------------------------------- teams */

    function startTeamPolling() {
        if (state.teamTimer) clearInterval(state.teamTimer);
        loadTeams();
        state.teamTimer = setInterval(loadTeams, 3000);
    }

    function loadTeams() {
        api("/api/vibe/teams").then(function (data) {
            var list = q("vibeTeamsList");
            if (!list) return;
            var teams = (data && data.success && Array.isArray(data.teams)) ? data.teams : [];
            if (!teams.length) {
                list.innerHTML = '<div class="vibe-rd-empty">No teams yet.</div>';
                return;
            }
            var anyRunning = false;
            list.innerHTML = teams.map(function (t) {
                if (t.status === "running") anyRunning = true;
                var chips = (t.members || []).map(function (m) {
                    var cls = m.state === "done" || m.state === "completed" ? "done"
                        : m.state === "failed" ? "failed"
                        : m.state === "pending" ? "" : "working";
                    var runTag = m.run_id ? " · " + shortRunId(m.run_id) : "";
                    return '<span class="vibe-member-chip ' + cls + '">' +
                        esc(m.name) + " · " + esc(m.state || "?") + runTag + "</span>";
                }).join("");
                return '<div class="vibe-team-card">' +
                    '<div class="vibe-team-head"><span>' + esc(t.name) + "</span>" +
                    '<span class="vibe-chip state-running" style="background:transparent;border:1px solid var(--border);color:var(--text-muted)">' + esc(t.status || "?") + "</span></div>" +
                    '<div class="vibe-member-chips">' + chips + "</div>" +
                    "</div>";
            }).join("");
            if (!anyRunning && state.teamTimer) {
                clearInterval(state.teamTimer);
                state.teamTimer = null;
            }
        });
    }

    /* ------------------------------------------------- progress events (WS) */

    function onProgress(eventData, raw) {
        if (!eventData) return;
        var step = eventData.step || eventData.event_type || "";
        var evRunId = eventData.run_id;
        if (evRunId && state.runId && String(evRunId) !== String(state.runId)) return;

        if (typeof eventData.progress === "number") {
            state.progress = Math.max(state.progress, eventData.progress);
        } else if (eventData.current_step && eventData.total_steps) {
            state.progress = Math.max(state.progress, Math.round((eventData.current_step / eventData.total_steps) * 100));
        }

        var phase = phaseForStep(step);
        if (phase) state.phase = phase;

        if (String(step).indexOf("pipeline:") === 0) {
            var stageId = String(step).replace("pipeline:", "");
            if (state.pipeline) {
                var st = state.pipeline.stages.find(function (s) { return s.id === stageId; }) ||
                    state.pipeline.stages.find(function (s) { return s.kind && stageKindTool(s.kind) === stageId; });
                if (st) state.stageDone[st.id] = true;
                state.stageActive = null;
                renderPipelineStrip();
            }
        }
        renderRunCard();
        if (String(step).indexOf("team:member:") === 0) {
            startTeamPolling();
        }
    }

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
        if (!document.documentElement.dataset.vibeRunApprovalWired) {
            document.documentElement.dataset.vibeRunApprovalWired = "true";
            document.addEventListener("click", function (event) {
                var target = event.target;
                if (!target || typeof target.closest !== "function") return;
                if (target.closest("#vibeApproveBtn")) {
                    approveRun();
                } else if (target.closest("#vibeDenyBtn")) {
                    denyRun();
                }
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
            if (e.detail && e.detail.run_id) focus(e.detail.run_id);
        });
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
        var st = state.run && state.run.state;
        if (state.runId && st === "awaiting_approval") {
            approveRun();
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
        var projectName = typeof currentProject !== "undefined" && currentProject ? String(currentProject) : "selected project";
        var text = "Run and verify the selected project " + projectName;
        // Auto-approval like freebuff: Run executes tools without waiting for
        // manual approval gates (server only honors it for admins).
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

    // VB6 "Break": the run holds at its next checkpoint (manual approval
    // gate). The server keeps the run in flight; the transport shows PAUSED
    // and Play resumes (or approves the pending gate).
    function pause() {
        if (!state.runId || !state.run) {
            updateRibbonStatus("NO RUN TO PAUSE", "hint");
            return;
        }
        if (String(state.run.state) === "awaiting_approval") {
            updateRibbonStatus("PAUSED — WAITING FOR APPROVAL", "paused");
            if (window.VibeWindows) window.VibeWindows.openRunDock();
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
        var projectName = typeof currentProject !== "undefined" && currentProject ? String(currentProject) : "selected project";
        var text = "Deploy the project " + projectName + " to production";
        uiMsg("🚀 Deploying " + projectName + " to production…");
        // The Deploy click itself is the approval to publish: carry
        // auto_approve so the approval-gated deploy pipeline (commit/publish/
        // domain stages) does not hang waiting for a signal nobody sends.
        // The server only honors auto_approve for administrators.
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
        return ["pending", "running", "awaiting_approval"].indexOf(String(state.run.state)) !== -1;
    }

    function stop() {
        if (!state.runId) {
            updateRibbonStatus("NO RUN TO STOP", "hint");
            return;
        }
        state.paused = false;
        denyRun();
        closePreview();
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
        focus: focus,
        onProgress: onProgress,
        approve: approveRun,
        deny: denyRun,
        preview: previewProject,
        play: play,
        deploy: deploy,
        pause: pause,
        stop: stop,
    };
    window.VibeTransport = {
        play: play,
        deploy: deploy,
        pause: pause,
        stop: stop,
        hasActiveRun: hasActiveRun,
    };
})();
