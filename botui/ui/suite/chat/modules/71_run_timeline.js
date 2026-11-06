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
    var RT = (window.RunTimeline = window.RunTimeline || {});
    var S = (RT._state = {
        runs: Object.create(null),
        seenSeq: Object.create(null),
        activeRunId: null,
        // True while replaying stored frames right after a (re)connect. A run
        // that already finished before this page loaded is history: its card is
        // removed on RUN_FINISHED instead of being parked at the end of the
        // transcript, away from the turn it belongs to.
        restoring: false,
        // #1677 — takeover segments that arrived before the run card existed
        // (the WS broadcast can beat RUN_STARTED). Applied on card creation.
        pendingTakeover: Object.create(null),
        timer: null,
        logBuffer: [],
        flushScheduled: false,
    });

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
        if (!S.activeRunId) return;
        applyTakeover(S.activeRunId, state, surface || "terminal");
        reportTakeover(S.activeRunId, state, surface || "terminal");
    }

    // #1677 — ESC interrupts the run (Claude-Code-style): same message the
    // Stop button sends. An input or textarea keeps the key to itself.
    document.addEventListener("keydown", function (event) {
        if (event.key !== "Escape" || !S.activeRunId) return;
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
        var entry = S.runs[runId];
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
            // Open the surface the run actually touched. The old handler
            // always opened the browser panel, so taking over a terminal run
            // showed the wrong surface.
            var surface = entry.surface || "terminal";
            applyTakeover(runId, "taken_over", surface);
            reportTakeover(runId, "taken_over", surface);
            if (surface === "browser") {
                if (window.GBBrowserUse) window.GBBrowserUse.show();
            } else if (window.WindowManager && typeof window.WindowManager.openDeepLink === "function") {
                window.WindowManager.openDeepLink("terminal", {});
            } else {
                window.open("/suite/desktop.html?app=terminal", "_blank", "noopener");
            }
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
            restored: S.restoring,
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

        S.runs[runId] = entry;
        S.activeRunId = runId;
        var pending = S.pendingTakeover[runId];
        if (pending) {
            applyTakeover(runId, pending.state, pending.surface);
            delete S.pendingTakeover[runId];
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
        if (S.timer) return;
        S.timer = setInterval(function () {
            var any = false;
            Object.keys(S.runs).forEach(function (id) {
                var entry = S.runs[id];
                if (entry.terminal) return;
                any = true;
                entry.elapsed.textContent = elapsedLabel(entry.startedAt);
            });
            if (!any && S.timer) {
                clearInterval(S.timer);
                S.timer = null;
            }
        }, 250);
    }

    // Internals shared with the sibling files (same split for #1731).
    RT._createCard = createCard;
    RT._applyTakeover = applyTakeover;
    RT._readLastSeq = readLastSeq;
    RT._writeLastSeq = writeLastSeq;
    RT._elapsedLabel = elapsedLabel;
    RT._sessionKey = sessionKey;
    RT.reportTakeoverState = reportTakeoverState;
})();
