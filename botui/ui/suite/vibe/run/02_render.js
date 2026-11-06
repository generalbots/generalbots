/* Vibe Run Dock — split from vibe/vibe-run.js for #1731 */

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
            // #1309 — terminal state is also a status-bar outcome, so the bar
            // never keeps spinning on a stale RUNNING text.
            var outcomeIcon = state.run.state === "completed" ? "\u2705 "
                : state.run.state === "failed" ? "\u274C " : "\u23F9 ";
            updateRibbonStatus(outcomeIcon + String(state.run.state).toUpperCase(),
                state.run.state === "completed" ? "ok" : "error");
        } else {
            // #1309 — narrate run progress on the Vibe status bar (mirrors
            // vibeRibbonStatus every 800ms) until the run completes: phase,
            // progress percent and tool-call count, e.g.
            // "ACTING · 45% · 12 tool calls".
            var pct = Math.max(2, Math.min(100, Math.round(state.progress)));
            var bits = [String(state.phase || defaultPhase(state.run.state)).toUpperCase()];
            if (state.progress > 0) bits.push(pct + "%");
            bits.push((state.run.tool_call_count || 0) + " tool calls");
            updateRibbonStatus(bits.join(" \u00B7 "), "running");
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
            list.innerHTML = '<div class="vibe-rd-empty">No tasks yet — start a run to see its plan stages.</div>';
            return;
        }
        list.innerHTML = items.map(function (item) {
            var icon = item.state === "done" ? "✓" : item.state === "active" ? "▶" : "○";
            return '<div class="vibe-todo-item ' + vibeRunEsc(item.state) + '"><span class="vibe-todo-icon">' + icon + '</span><span>' + vibeRunEsc(item.label) + '</span></div>';
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
            return '<span class="vibe-stage-chip ' + cls + '">' + vibeRunEsc(st.name || st.id) + "</span>";
        }).join("");
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
                return '<div class="vibe-src-item"><span aria-hidden="true"><svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48"/></svg></span><span>' + vibeRunEsc(label) + (detail ? " · " + vibeRunEsc(detail) : "") + "</span></div>";
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
