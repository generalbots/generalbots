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
    // #1677 — takeover segments that arrived before the run card existed
    // (the WS broadcast can beat RUN_STARTED). Applied on card creation.
    var pendingTakeover = Object.create(null);
    var timer = null;
    var logBuffer = [];
    var flushScheduled = false;

    function sessionKey() {
        var id = (window.ChatState && window.ChatState.currentSessionId) || "global";
        return String(id);
    }

    // #1677 — takeover events are mirrored to the run so hand-off and return
    // are visible to run subscribers and the model, without a frame the UI
    // can lose. The WS frame is the live path; nothing here is pending state.
    function reportTakeover(runId, state, surface) {
        var ws = window.ChatState && window.ChatState.ws;
        if (!ws || ws.readyState !== WebSocket.OPEN) return;
        ws.send(JSON.stringify({
            type: "run_takeover",
            bot_id: window.ChatState.currentBotId,
            user_id: window.ChatState.currentUserId,
            session_id: sessionKey(),
            run_id: runId,
            state: state,
            surface: surface || "terminal",
            timestamp: new Date().toISOString(),
        }));
    }

    // #1677 — takeover asked from a terminal/browser card without the run id
    // in scope (61_terminal_use.js). Uses the active run.
    function reportTakeoverState(state, surface) {
        if (!activeRunId) return;
        applyTakeover(activeRunId, state, surface || "terminal");
        reportTakeover(activeRunId, state, surface || "terminal");
    }

    // #1677 — ESC interrupts the run (Claude-Code-style): same message the
    // Stop button sends. An input or textarea keeps the key to itself.
    document.addEventListener("keydown", function (event) {
        if (event.key !== "Escape" || !activeRunId) return;
        var target = event.target;
        if (target) {
            var tag = String(target.tagName || "").toLowerCase();
            if (tag === "input" || tag === "textarea" || tag === "select" || target.isContentEditable) return;
        }
        if (function () {
            var overlay = document.querySelector("dialog[open], .settings-mcp-preview-overlay");
            return Boolean(overlay);
        }()) return;
        var ws = window.ChatState && window.ChatState.ws;
        if (ws && ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({ type: "run_cancel", session_id: sessionKey() }));
        }
    });

    // #1677 — reflect a takeover/hand-back segment on the run card. Used by
    // the local controls and by replicated frames (live broadcast or replay).
    function applyTakeover(runId, state, surface) {
        var entry = runs[runId];
        if (!entry || !entry.takeoverBar) return;
        if (state === "handed_back") {
            entry.takeoverBar.hidden = true;
            entry.takeoverBar.setAttribute("data-mode", "");
            if (entry.takeover) { entry.takeover.textContent = "Take over"; }
            return;
        }
        entry.takeoverBar.hidden = false;
        entry.takeoverBar.setAttribute("data-mode", "active");
        if (entry.takeover) { entry.takeover.textContent = "You control"; }
        var hint = entry.takeoverBar.querySelector(".run-takeover-hint");
        if (hint) {
            hint.textContent = (surface === "browser" ? "You control the browser. " : "You control the terminal. ") +
                "Hand input back when done — ESC interrupts the run.";
        }
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

        // #1677 — takeover: while the run holds a terminal or browser
        // session the user can take it over from the agent without
        // cancelling the run.
        var takeover = document.createElement("button");
        takeover.type = "button";
        takeover.className = "run-toggle";
        takeover.textContent = "Take over";
        takeover.hidden = true;

        head.appendChild(dot);
        head.appendChild(title);
        head.appendChild(elapsed);
        head.appendChild(toggle);
        head.appendChild(takeover);
        head.appendChild(stop);

        var body = document.createElement("div");
        body.className = "run-body";

        var summary = document.createElement("div");
        summary.className = "run-summary";
        summary.setAttribute("role", "status");
        summary.setAttribute("aria-live", "polite");
        summary.textContent = "Starting…";

        // #1677 — takeover controls, shown when the user takes over a live
        // terminal or browser session from the run.
        var takeoverBar = document.createElement("div");
        takeoverBar.className = "run-takeover";
        takeoverBar.hidden = true;

        var takeoverHint = document.createElement("span");
        takeoverHint.className = "run-takeover-hint";
        takeoverHint.textContent = "You control the session. Hand input back when done — ESC interrupts the run.";
        takeoverBar.appendChild(takeoverHint);

        // The two live surfaces a takeover can attach to. The Terminal app
        // attaches a real interactive pty for this workspace; the browser
        // panel is the consented computer-use pane.
        var openTerminal = document.createElement("button");
        openTerminal.type = "button";
        openTerminal.className = "run-toggle";
        openTerminal.textContent = "Terminal window";
        openTerminal.addEventListener("click", function () {
            if (window.WindowManager && typeof window.WindowManager.openDeepLink === "function") {
                window.WindowManager.openDeepLink("terminal", {});
            } else {
                window.open("/suite/desktop.html?app=terminal", "_blank", "noopener");
            }
        });
        takeoverBar.appendChild(openTerminal);

        var openBrowser = document.createElement("button");
        openBrowser.type = "button";
        openBrowser.className = "run-toggle";
        openBrowser.textContent = "Browser panel";
        openBrowser.addEventListener("click", function () {
            if (window.GBBrowserUse) window.GBBrowserUse.show();
        });
        takeoverBar.appendChild(openBrowser);

        var handBack = document.createElement("button");
        handBack.type = "button";
        handBack.className = "run-toggle";
        handBack.textContent = "Hand back";
        handBack.addEventListener("click", function () {
            applyTakeover(runId, "handed_back", "terminal");
            reportTakeover(runId, "handed_back", "terminal");
        });
        takeoverBar.appendChild(handBack);

        takeover.addEventListener("click", function () {
            applyTakeover(runId, "taken_over", "terminal");
            reportTakeover(runId, "taken_over", "terminal");
            if (window.GBBrowserUse) window.GBBrowserUse.show();
        });

        var rows = document.createElement("div");
        rows.className = "run-rows";

        var log = document.createElement("pre");
        log.className = "run-log";
        log.setAttribute("role", "log");
        log.setAttribute("aria-live", "polite");
        log.hidden = true;

        body.appendChild(summary);
        body.appendChild(takeoverBar);
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
            takeover: takeover,
            takeoverBar: takeoverBar,
            steps: Object.create(null),
            tools: Object.create(null),
            startedAt: (data && Number(data.startedAt)) || Date.now(),
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
        var pending = pendingTakeover[runId];
        if (pending) {
            applyTakeover(runId, pending.state, pending.surface);
            delete pendingTakeover[runId];
        }
        startTimer();
        // #1680 — first-paint sample for the performance gate.
        try {
            window.dispatchEvent(new CustomEvent("gb-run-card-mounted", {
                detail: { run_id: runId, startedAt: entry.startedAt },
            }));
        } catch (e) { /* CustomEvent unavailable */ }
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
        // #1677 — opt-in completion notification. Never fires per step, and
        // never requests permission on its own.
        try {
            if (
                localStorage.getItem("gb.run.notify") === "1" &&
                "Notification" in window &&
                window.Notification.permission === "granted" &&
                document.visibilityState !== "visible"
            ) {
                new window.Notification(entry.title.textContent || "Run finished", {
                    body: label,
                    tag: "gb-run-" + entry.id,
                });
            }
        } catch (e) { /* notifications unavailable */ }
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
        if (!frame) return;
        // Replayed frames come back as AG-UI envelopes (`event_type`); the
        // live path uses `type`. Normalize so both render identically.
        if (!frame.type && frame.event_type) {
            frame = Object.assign({}, frame, { type: frame.event_type });
        }
        if (!frame.type) return;
        var runId = frame.run_id || frame.runId || "";
        if (frame.seq) writeLastSeq(frame.seq);
        if (runId && frame.seq && seenSeq[runId + ":" + frame.seq]) return;
        if (runId && frame.seq) seenSeq[runId + ":" + frame.seq] = true;

        var entry = runId ? runs[runId] : null;
        if (!entry && activeRunId) entry = runs[activeRunId] || null;
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
                    pendingTakeover[runId || "pending"] = {
                        state: payload.state || "taken_over",
                        surface: payload.surface || "terminal",
                    };
                }
            }
            return;
        }

        // A run that touched the terminal or the browser exposes takeover.
        if ((type === "terminal_output" || type === "browser_ready") && entry && entry.takeover) {
            entry.takeover.hidden = false;
        }

        if (type === "RUN_STARTED") {
            // A new run on the same thread means the previous waiting card was
            // answered or approved; close it instead of leaving it spinning.
            Object.keys(runs).forEach(function (key) {
                var previous = runs[key];
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
        reportTakeoverState: reportTakeoverState,
    };
})();
