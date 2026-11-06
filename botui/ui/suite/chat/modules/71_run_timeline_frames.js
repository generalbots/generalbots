/**
 * Run timeline (#1668) — frame dispatch, resume and reset.
 * Split from 71_run_timeline.js for #1731; publishes the public
 * window.RunTimeline API once all three files are loaded.
 */
(function () {
    "use strict";

    var RT = window.RunTimeline;
    var S = RT._state;
    var sessionKey = RT._sessionKey;
    var createCard = RT._createCard;
    var applyTakeover = RT._applyTakeover;
    var row = RT._row;
    var appendLog = RT._appendLog;
    var finish = RT._finish;
    var customName = RT._customName;
    var customPayload = RT._customPayload;
    var writeLastSeq = RT._writeLastSeq;
    var readLastSeq = RT._readLastSeq;

    function handleFrame(frame) {
        if (!frame) return;
        // Replayed frames come back as AG-UI envelopes (`event_type`); the
        // live path uses `type`. Normalize so both render identically.
        if (!frame.type && frame.event_type) {
            frame = Object.assign({}, frame, { type: frame.event_type });
        }
        if (!frame.type) return;
        var runId = frame.run_id || frame.runId || "";
        if (frame.seq) writeLastSeq(frame.seq);
        if (runId && frame.seq && S.seenSeq[runId + ":" + frame.seq]) return;
        if (runId && frame.seq) S.seenSeq[runId + ":" + frame.seq] = true;

        var entry = runId ? S.runs[runId] : null;
        if (!entry && S.activeRunId) entry = S.runs[S.activeRunId] || null;
        var type = frame.type;

        // #1677 — takeover segments (live broadcast or replay) restore the
        // controls after a reload, on any device.
        if (type === "CUSTOM") {
            var custom = frame.data || {};
            if (custom.name === "gb.takeover") {
                var payload = custom.payload || {};
                if (entry) {
                    applyTakeover(entry.id, payload.state || "taken_over", payload.surface || "terminal");
                } else {
                    S.pendingTakeover[runId || "pending"] = {
                        state: payload.state || "taken_over",
                        surface: payload.surface || "terminal",
                    };
                }
                return;
            }
            // Every OTHER custom frame must continue to the dispatcher below;
            // the takeover branch used to return unconditionally, so
            // `gb.command_output` (live terminal lines) never reached the log.
        }

        // A run that touched the terminal or the browser exposes takeover.
        if ((type === "terminal_output" || type === "browser_ready") && entry && entry.takeover) {
            entry.surface = type === "browser_ready" ? "browser" : "terminal";
            entry.takeover.hidden = false;
        }

        if (type === "RUN_STARTED") {
            // A new run on the same thread means the previous waiting card was
            // answered or approved; close it instead of leaving it spinning.
            Object.keys(S.runs).forEach(function (key) {
                var previous = S.runs[key];
                if (previous.waiting && !previous.terminal) {
                    finish(previous, "succeeded", {});
                }
            });
            if (!entry && runId) createCard(runId, Object.assign({}, frame.data || {}, {
                startedAt: Date.parse(frame.ts) || Date.now(),
            }));
            return;
        }
        if (type === "RUN_FINISHED") {
            var state = (frame.data && frame.data.state) || "succeeded";
            finish(entry, state, frame.data || {});
            return;
        }
        if (type === "RUN_ERROR") {
            if (entry) {
                var errorRow = row(entry, "error:" + runId, (frame.data && frame.data.message) || "Run failed", "error");
                errorRow.classList.add("run-error");
            }
            finish(entry, "failed", frame.data || {});
            return;
        }
        if (!entry && type === "STEP_STARTED" && runId) {
            entry = createCard(runId, { title: "Working…" });
        }
        if (!entry) return;

        if (type === "STEP_STARTED") {
            var stepId = (frame.data && frame.data.step_id) || "step";
            var stepName = (frame.data && frame.data.name) || stepId;
            entry.summary.textContent = stepName;
            // The only step the pipeline emits is "Answering", and the summary
            // already shows it. A row with the same word printed the status
            // twice ("Answering … Answering", #1693).
            if (stepName.toLowerCase() !== "answering") {
                row(entry, "step:" + stepId, stepName, "running");
            }
            return;
        }
        if (type === "STEP_FINISHED") {
            var doneStep = (frame.data && frame.data.step_id) || "step";
            row(entry, "step:" + doneStep, doneStep, (frame.data && frame.data.status) || "ok");
            return;
        }
        if (type === "TOOL_CALL_START") {
            var callId = (frame.data && frame.data.call_id) || "call";
            var toolName = (frame.data && frame.data.name) || callId;
            // Say what the VM is doing while it runs, instead of a static
            // "Answering" until the tool returns (#1693).
            entry.summary.textContent = "Running " + toolName + "…";
            row(entry, "tool:" + callId, toolName, "running");
            return;
        }
        if (type === "TOOL_CALL_RESULT") {
            var resultId = (frame.data && frame.data.call_id) || "call";
            var status = (frame.data && frame.data.status) || "ok";
            row(entry, "tool:" + resultId, "Tool result", status);
            entry.summary.textContent = status === "error" ? "Tool failed" : "Answering…";
            var result = frame.data && frame.data.result;
            if (result && typeof result === "object") {
                var lines = Array.isArray(result.lines) ? result.lines : null;
                if (lines) {
                    lines.forEach(function (line) {
                        appendLog(entry, line, "stdout");
                    });
                }
            }
            return;
        }
        if (type === "SUBAGENT_STARTED") {
            var child = (frame.data && frame.data.child_run_id) || "child";
            row(entry, "subagent:" + child, (frame.data && frame.data.agent_name) || "Sub-agent", "running");
            return;
        }
        if (type === "SUBAGENT_FINISHED") {
            var childDone = (frame.data && frame.data.child_run_id) || "child";
            row(entry, "subagent:" + childDone, "Sub-agent", (frame.data && frame.data.state) || "succeeded");
            return;
        }
        if (type === "CUSTOM") {
            var name = customName(frame);
            var payload = customPayload(frame);
            if (name === "gb.command_output") {
                appendLog(entry, payload.line || "", payload.stream || "stdout");
            } else if (name === "gb.tool_progress") {
                // #1693 — the VM reports start/finish so the card says what it
                // is doing during a long run instead of a static "Answering".
                var progressTool = payload.tool || "tool";
                if (payload.phase === "start") {
                    entry.summary.textContent = "Running " + progressTool + "…";
                    row(entry, "progress:" + progressTool, progressTool +
                        (payload.detail ? " · " + payload.detail : ""), "running");
                } else {
                    entry.summary.textContent = "Answering…";
                    row(entry, "progress:" + progressTool, progressTool + " finished",
                        payload.ok === false ? "error" : "ok");
                }
            } else if (name === "gb.question") {
                row(entry, "question:" + (payload.question_id || "q"), "Waiting for your answer", "waiting");
                entry.card.classList.add("waiting");
                entry.waiting = true;
                entry.summary.textContent = "Waiting for input";
            } else if (name === "gb.surface") {
                row(entry, "surface:" + (payload.surface_id || "s"), "Showing a result", "ok");
            } else if (name === "gb.plan") {
                row(entry, "plan", "Plan proposed", "waiting");
                entry.waiting = true;
            }
        }
    }

    function resume() {
        // A fresh page has no cards: replay from the beginning so the run
        // summaries are restored together with the conversation. A reconnect
        // with cards already mounted resumes from the last seen frame.
        var hasCards = document.querySelectorAll(".run-card").length > 0;
        var last = hasCards ? readLastSeq() : 0;
        var ws = window.ChatState && window.ChatState.ws;
        if (!ws || ws.readyState !== WebSocket.OPEN) return;
        S.restoring = true;
        window.setTimeout(function () { S.restoring = false; }, 2500);
        ws.send(JSON.stringify({
            type: "run_resume",
            session_id: sessionKey(),
            last_seq: last,
        }));
    }

    function reset() {
        S.runs = Object.create(null);
        S.seenSeq = Object.create(null);
        S.activeRunId = null;
        S.logBuffer = [];
        if (S.timer) {
            clearInterval(S.timer);
            S.timer = null;
        }
    }

    window.addEventListener("gb-chat-connected", resume);

    RT.handleFrame = handleFrame;
    RT.resume = resume;
    RT.reset = reset;
    RT.activeRunId = function () { return S.activeRunId; };
})();
