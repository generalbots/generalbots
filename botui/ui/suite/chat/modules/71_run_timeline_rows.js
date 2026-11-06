/**
 * Run timeline (#1668) — rows, batched log output and run completion.
 * Split from 71_run_timeline.js for #1731. State lives on
 * window.RunTimeline._state; the frame dispatcher loads after this file.
 */
(function () {
    "use strict";

    var RT = window.RunTimeline;
    var S = RT._state;
    var MAX_LOG_LINES = 200;
    var elapsedLabel = RT._elapsedLabel;

    function row(entry, key, name, status) {
        var existing = entry.rows.querySelector('[data-row-key="' + key + '"]');
        if (existing) {
            existing.setAttribute("data-status", status);
            var badge = existing.querySelector(".run-row-status");
            if (badge) badge.textContent = status;
            var existingLabel = existing.querySelector(".run-row-name");
            if (existingLabel && name) existingLabel.textContent = name;
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
        S.logBuffer.push({ entry: entry, line: String(line || ""), stream: stream || "stdout" });
        if (!S.flushScheduled) {
            S.flushScheduled = true;
            window.requestAnimationFrame(flushLog);
        }
    }

    function flushLog() {
        S.flushScheduled = false;
        var batch = S.logBuffer;
        S.logBuffer = [];
        var byRun = Object.create(null);
        batch.forEach(function (item) {
            (byRun[item.entry.id] = byRun[item.entry.id] || []).push(item);
        });
        Object.keys(byRun).forEach(function (id) {
            var entry = S.runs[id];
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
        if (entry.restored && state === "succeeded") {
            // Replayed history, not a live turn: the transcript already holds
            // the answer and this card would sit detached at the bottom.
            entry.card.remove();
            delete S.runs[entry.id];
            if (S.activeRunId === entry.id) S.activeRunId = null;
            return;
        }
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
        if (S.activeRunId === entry.id) S.activeRunId = null;
    }

    function customName(data) {
        var payload = data && data.data ? data.data : {};
        return payload.name || "";
    }

    function customPayload(data) {
        var payload = data && data.data ? data.data : {};
        return payload.payload || {};
    }

    RT._row = row;
    RT._appendLog = appendLog;
    RT._finish = finish;
    RT._customName = customName;
    RT._customPayload = customPayload;
})();
