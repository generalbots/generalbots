/**
 * Run timeline (#1668).
 *
 * One collapsible card per assistant turn. Frames come from the run event
 * contract (RUN_*, STEP_*, TOOL_CALL_*, CUSTOM gb.*). The card is built with
 * DOM nodes and textContent only: model output never reaches innerHTML.
 */
(function () {
    "use strict";

    var LAST_SEQ_PREFIX = "gb.run.last_seq.";
    var MAX_LOG_LINES = 200;
    var runs = Object.create(null);
    var seenSeq = Object.create(null);
    var activeRunId = null;
    var timer = null;
    var logBuffer = [];
    var flushScheduled = false;

    function sessionKey() {
        var id = (window.ChatState && window.ChatState.currentSessionId) || "global";
        return String(id);
    }

    function lastSeqKey() {
        return LAST_SEQ_PREFIX + sessionKey();
    }

    function readLastSeq() {
        try {
            return parseInt(localStorage.getItem(lastSeqKey()) || "0", 10) || 0;
        } catch (e) {
            return 0;
        }
    }

    function writeLastSeq(seq) {
        if (!seq) return;
        try {
            localStorage.setItem(lastSeqKey(), String(seq));
        } catch (e) { /* storage unavailable */ }
    }

    function messagesRoot() {
        return document.getElementById("messages");
    }

    function elapsedLabel(startedAt) {
        var seconds = Math.max(0, (Date.now() - startedAt) / 1000);
        if (seconds < 60) return seconds.toFixed(1) + "s";
        var minutes = Math.floor(seconds / 60);
        return minutes + "m " + Math.round(seconds % 60) + "s";
    }

    function createCard(runId, data) {
        var root = messagesRoot();
        if (!root) return null;

        var card = document.createElement("section");
        card.className = "run-card running";
        card.setAttribute("data-run-id", runId);
        card.setAttribute("aria-label", "Run progress");

        var head = document.createElement("header");
        head.className = "run-card-head";

        var dot = document.createElement("span");
        dot.className = "run-dot";
        dot.setAttribute("aria-hidden", "true");

        var title = document.createElement("span");
        title.className = "run-title";
        title.textContent = (data && data.title) || "Working…";

        var elapsed = document.createElement("span");
        elapsed.className = "run-elapsed";
        elapsed.textContent = "0.0s";

        var toggle = document.createElement("button");
        toggle.type = "button";
        toggle.className = "run-toggle";
        toggle.textContent = "Collapse";
        toggle.setAttribute("aria-expanded", "true");

        var stop = document.createElement("button");
        stop.type = "button";
        stop.className = "run-stop";
        stop.textContent = "Stop";

        head.appendChild(dot);
        head.appendChild(title);
        head.appendChild(elapsed);
        head.appendChild(toggle);
        head.appendChild(stop);

        var body = document.createElement("div");
        body.className = "run-body";

        var summary = document.createElement("div");
        summary.className = "run-summary";
        summary.textContent = "Starting…";

        var rows = document.createElement("div");
        rows.className = "run-rows";

        var log = document.createElement("pre");
        log.className = "run-log";
        log.setAttribute("role", "log");
        log.setAttribute("aria-live", "polite");
        log.hidden = true;

        body.appendChild(summary);
        body.appendChild(rows);
        body.appendChild(log);
        card.appendChild(head);
        card.appendChild(body);
        root.appendChild(card);

        var entry = {
            id: runId,
            card: card,
            title: title,
            elapsed: elapsed,
            summary: summary,
            rows: rows,
            log: log,
            toggle: toggle,
            stop: stop,
            steps: Object.create(null),
            tools: Object.create(null),
            startedAt: Date.now(),
            terminal: false,
            logLines: 0,
            collapsed: false,
        };

        toggle.addEventListener("click", function () {
            entry.collapsed = !entry.collapsed;
            card.classList.toggle("collapsed", entry.collapsed);
            toggle.textContent = entry.collapsed ? "Expand" : "Collapse";
            toggle.setAttribute("aria-expanded", entry.collapsed ? "false" : "true");
        });

        stop.addEventListener("click", function () {
            var ws = window.ChatState && window.ChatState.ws;
            if (ws && ws.readyState === WebSocket.OPEN) {
                ws.send(JSON.stringify({
                    type: "run_cancel",
                    session_id: sessionKey(),
                }));
            }
            stop.disabled = true;
            stop.textContent = "Stopping…";
        });

        runs[runId] = entry;
        activeRunId = runId;
        startTimer();
        return entry;
    }

    function startTimer() {
        if (timer) return;
        timer = setInterval(function () {
            var any = false;
            Object.keys(runs).forEach(function (id) {
                var entry = runs[id];
                if (entry.terminal) return;
                any = true;
                entry.elapsed.textContent = elapsedLabel(entry.startedAt);
            });
            if (!any && timer) {
                clearInterval(timer);
                timer = null;
            }
        }, 250);
    }

    function row(entry, key, name, status) {
        var existing = entry.rows.querySelector('[data-row-key="' + key + '"]');
        if (existing) {
            existing.setAttribute("data-status", status);
            var badge = existing.querySelector(".run-row-status");
            if (badge) badge.textContent = status;
            return existing;
        }
        var item = document.createElement("div");
        item.className = "run-row";
        item.setAttribute("data-row-key", key);
        item.setAttribute("data-status", status);

        var badge = document.createElement("span");
        badge.className = "run-row-status";
        badge.textContent = status;

        var label = document.createElement("span");
        label.className = "run-row-name";
        label.textContent = name;

        item.appendChild(badge);
        item.appendChild(label);
        entry.rows.appendChild(item);
        return item;
    }

    function appendLog(entry, line, stream) {
        if (!entry || entry.terminal) return;
        logBuffer.push({ entry: entry, line: String(line || ""), stream: stream || "stdout" });
        if (!flushScheduled) {
            flushScheduled = true;
            window.requestAnimationFrame(flushLog);
        }
    }

    function flushLog() {
        flushScheduled = false;
        var batch = logBuffer;
        logBuffer = [];
        var byRun = Object.create(null);
        batch.forEach(function (item) {
            (byRun[item.entry.id] = byRun[item.entry.id] || []).push(item);
        });
        Object.keys(byRun).forEach(function (id) {
            var entry = runs[id];
            if (!entry) return;
            entry.log.hidden = false;
            var fragment = document.createDocumentFragment();
            byRun[id].forEach(function (item) {
                if (entry.logLines >= MAX_LOG_LINES) return;
                var line = document.createElement("span");
                line.className = "run-log-line " + item.stream;
                line.textContent = item.line;
                fragment.appendChild(line);
                fragment.appendChild(document.createTextNode("\n"));
                entry.logLines += 1;
            });
            if (entry.logLines >= MAX_LOG_LINES && !entry.log.dataset.truncated) {
                entry.log.dataset.truncated = "1";
                var note = document.createElement("span");
                note.className = "run-log-truncated";
                note.textContent = "Output truncated. Download the full log from the run menu.";
                fragment.appendChild(note);
            }
            entry.log.appendChild(fragment);
            entry.log.scrollTop = entry.log.scrollHeight;
        });
    }

    function finish(entry, state, data) {
        if (!entry || entry.terminal) return;
        entry.terminal = true;
        entry.card.classList.remove("running", "waiting");
        entry.card.classList.add("state-" + state);
        entry.stop.hidden = true;
        entry.elapsed.textContent = data && data.duration_ms
            ? (data.duration_ms / 1000).toFixed(1) + "s"
            : elapsedLabel(entry.startedAt);
        var label = {
            succeeded: "Succeeded",
            failed: "Failed",
            cancelled: "Cancelled",
        }[state] || state;
        entry.summary.textContent = label;
        if (state === "succeeded") {
            entry.collapsed = true;
            entry.card.classList.add("collapsed");
            entry.toggle.textContent = "Expand";
            entry.toggle.setAttribute("aria-expanded", "false");
        } else {
            entry.card.classList.add("expanded");
            entry.stop.hidden = true;
        }
        if (activeRunId === entry.id) activeRunId = null;
    }

    function customName(data) {
        var payload = data && data.data ? data.data : {};
        return payload.name || "";
    }

    function customPayload(data) {
        var payload = data && data.data ? data.data : {};
        return payload.payload || {};
    }

    function handleFrame(frame) {
        if (!frame || !frame.type) return;
        var runId = frame.run_id || frame.runId || "";
        if (frame.seq) writeLastSeq(frame.seq);
        if (runId && frame.seq && seenSeq[runId + ":" + frame.seq]) return;
        if (runId && frame.seq) seenSeq[runId + ":" + frame.seq] = true;

        var entry = runId ? runs[runId] : null;
        var type = frame.type;

        if (type === "RUN_STARTED") {
            if (!entry && runId) createCard(runId, frame.data || {});
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
            row(entry, "step:" + stepId, (frame.data && frame.data.name) || stepId, "running");
            entry.summary.textContent = (frame.data && frame.data.name) || "Working…";
            return;
        }
        if (type === "STEP_FINISHED") {
            var doneStep = (frame.data && frame.data.step_id) || "step";
            row(entry, "step:" + doneStep, doneStep, (frame.data && frame.data.status) || "ok");
            return;
        }
        if (type === "TOOL_CALL_START") {
            var callId = (frame.data && frame.data.call_id) || "call";
            row(entry, "tool:" + callId, (frame.data && frame.data.name) || callId, "running");
            return;
        }
        if (type === "TOOL_CALL_RESULT") {
            var resultId = (frame.data && frame.data.call_id) || "call";
            var status = (frame.data && frame.data.status) || "ok";
            row(entry, "tool:" + resultId, "Tool result", status);
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
            } else if (name === "gb.question") {
                row(entry, "question:" + (payload.question_id || "q"), "Waiting for your answer", "waiting");
                entry.card.classList.add("waiting");
                entry.summary.textContent = "Waiting for input";
            } else if (name === "gb.surface") {
                row(entry, "surface:" + (payload.surface_id || "s"), "Showing a result", "ok");
            } else if (name === "gb.plan") {
                row(entry, "plan", "Plan proposed", "waiting");
            }
        }
    }

    function resume() {
        var last = readLastSeq();
        if (!last) return;
        var ws = window.ChatState && window.ChatState.ws;
        if (!ws || ws.readyState !== WebSocket.OPEN) return;
        ws.send(JSON.stringify({
            type: "run_resume",
            session_id: sessionKey(),
            last_seq: last,
        }));
    }

    function reset() {
        runs = Object.create(null);
        seenSeq = Object.create(null);
        activeRunId = null;
        logBuffer = [];
        if (timer) {
            clearInterval(timer);
            timer = null;
        }
    }

    window.addEventListener("gb-chat-connected", resume);

    window.RunTimeline = {
        handleFrame: handleFrame,
        resume: resume,
        reset: reset,
        activeRunId: function () { return activeRunId; },
    };
})();
