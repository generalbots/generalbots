/**
 * Vibe Run Dock (#806, #807) — live run card, approval resume UX,
 * budget/metering panel, pipeline strip, grounding sources, sessions
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
        budgetCents: 0,
    };

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
        // dock never lingers on "PLANNING"/"ACTIVE" after completion.
        if (["completed", "failed", "cancelled"].indexOf(String(state.run.state)) !== -1) {
            state.phase = defaultPhase(state.run.state);
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
        renderBudget();
        renderApproval();
    }

    function renderBudget() {
        var card = q("vibeRunCard");
        if (!card) return;
        var spend = 0;
        if (state.metrics && typeof state.metrics.total_cost === "number") {
            spend = state.metrics.total_cost;
        }
        var budgetCents = (state.run && state.run.budget_cents) ? state.run.budget_cents : state.budgetCents;
        setText("vibeBudgetSpend", "$" + spend.toFixed(2) + " spent");
        if (budgetCents > 0) {
            setText("vibeBudgetCap", "of " + money(budgetCents));
            var pct = Math.min(100, Math.round((spend * 100) / budgetCents));
            var fill = q("vibeBudgetFill");
            if (fill) {
                fill.style.width = pct + "%";
                fill.classList.toggle("near-cap", pct > 80);
            }
            var spendEl = q("vibeBudgetSpend");
            if (spendEl) spendEl.classList.toggle("near-cap", pct > 80);
        } else {
            setText("vibeBudgetCap", "— no budget cap");
            var f = q("vibeBudgetFill");
            if (f) f.style.width = "0%";
        }
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
        stopPolling();
        state.pollTimer = setInterval(loadAll, 1500);
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

    function pollRun() {
        api("/api/vibe/run/" + state.runId).then(function (data) {
            if (!data || !data.run_id) return;
            var changed = !state.run || String(data.state) !== String(state.run.state);
            state.run = data;
            renderRunCard();
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
                } else if (data.state === "failed" || data.state === "cancelled") {
                    uiMsg("⛔ Run " + data.state + ".");
                    stopPolling();
                }
            }
            api("/api/vibe/metrics/" + state.runId).then(function (m) {
                if (m && m.success && m.metrics) {
                    state.metrics = m.metrics;
                    renderBudget();
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
        state.tickTimer = setInterval(function () {
            if (!state.run || !state.run.created_at) return;
            var start = new Date(state.run.created_at).getTime();
            if (isNaN(start)) return;
            var s = Math.max(0, Math.floor((Date.now() - start) / 1000));
            setText("vibeRunElapsed", s + "s");
        }, 1000);
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
        var budgetInput = q("vibeRunBudget");
        var budget = budgetInput && parseFloat(budgetInput.value);
        var budgetCents = isFinite(budget) && budget > 0 ? Math.round(budget * 100) : 0;
        state.budgetCents = budgetCents;
        uiMsg("🔄 Starting Vibe run: " + intent.substring(0, 80) + "…");
        // Attach the currently selected project (if any) so the agent edits
        // the right workspace and the deploy pipeline targets the right VM.
        var pid = null;
        if (typeof currentProjectId !== "undefined" && currentProjectId) pid = currentProjectId;
        var pname = null;
        if (typeof currentProject !== "undefined" && currentProject) pname = currentProject;
        var body = {
            intent: intent,
            bot_id: (typeof vibeBotId !== "undefined" && vibeBotId && vibeBotId !== "default") ? vibeBotId : null,
            use_case: state.useCase,
            budget_cents: budgetCents,
            // #919 — default to manual approval for destructive tools; the
            // server only honors auto_approve for administrators.
            auto_approve: opts.auto_approve === true,
            project_id: pid,
            project_name: pname,
        };
        return api("/api/vibe/run", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(body),
        }).then(function (data) {
            if (data.success && data.run_id) {
                setText("vibeRunDockState", "ACTIVE");
                q("vibeRunDockState").className = "vibe-chip state-running";
                focus(data.run_id);
                return { ok: true, run_id: data.run_id };
            }
            var err = data.error || ("HTTP " + (data.http_status || "?"));
            uiMsg("⚠️ Vibe run API unavailable (" + err + ") — falling back to autotask flow.");
            setText("vibeRunDockState", "IDLE");
            q("vibeRunDockState").className = "vibe-chip state-pending";
            throw new Error(err);
        });
    }

    function retry(fn, key) {
        var n = (retry.count[key] = (retry.count[key] || 0) + 1);
        if (n < 30) setTimeout(fn, 1000);
    }
    retry.count = {};

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
        });
    }

    /* ------------------------------------------------- teams */

    function createTeam() {
        var name = q("vibeTeamName").value.trim();
        var objective = q("vibeTeamObjective").value.trim();
        var lines = q("vibeTeamMembers").value.split("\n")
            .map(function (l) { return l.trim(); })
            .filter(Boolean);
        if (!name || !objective || !lines.length) {
            uiMsg("⚠️ Team run needs a name, objective and at least one member.");
            return;
        }
        var members = lines.map(function (l) {
            var parts = l.split("|");
            return {
                name: (parts[0] || "member").trim(),
                task: (parts[1] || objective).trim(),
            };
        });
        uiMsg("🚀 Starting team run '" + name + "' with " + members.length + " member(s)…");
        api("/api/vibe/teams", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ name: name, objective: objective, members: members }),
        }).then(function (data) {
            if (data.success) {
                q("vibeTeamName").value = "";
                q("vibeTeamObjective").value = "";
                q("vibeTeamMembers").value = "";
                uiMsg("✅ Team run started (team " + shortRunId(data.team_id) + ").");
                startTeamPolling();
            } else {
                uiMsg("⚠️ Team start failed: " + ((data.error) || "unknown"));
            }
        });
    }

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
        if (typeof updateVibe1 === "function") {
            if (eventData.progress === 100 || /complete|done|evolved/i.test(step)) {
                updateVibe1("done");
            } else {
                updateVibe1("working");
            }
        }
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
        var approve = q("vibeApproveBtn");
        if (approve) approve.addEventListener("click", approveRun);
        var deny = q("vibeDenyBtn");
        if (deny) deny.addEventListener("click", denyRun);
        var teamBtn = q("vibeTeamCreateBtn");
        if (teamBtn) teamBtn.addEventListener("click", createTeam);
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

    init();

    window.VibeRun = {
        start: start,
        focus: focus,
        onProgress: onProgress,
        approve: approveRun,
        deny: denyRun,
    };
})();