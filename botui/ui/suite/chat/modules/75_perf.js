/**
 * Chat performance budgets (#1680).
 *
 * Records the three budgets the workspace promises:
 * - run-card first paint < 150 ms
 * - log frame latency p95 < 250 ms
 * - reconnect resume < 1 s
 *
 * Values stay in a bounded ring buffer and are exposed for the automated
 * browser tests (`window.GBPerf.metrics()`), never sent to a third party.
 */
(function () {
    "use strict";

    var MAX = 200;
    var samples = [];
    var reconnectStarted = null;
    var firstCardAt = null;
    var resuming = false;

    function now() {
        return window.performance && performance.now ? performance.now() : Date.now();
    }

    function push(kind, value, detail) {
        samples.push({ kind: kind, value: value, at: Date.now(), detail: detail || null });
        if (samples.length > MAX) samples.shift();
    }

    function percentile(values, p) {
        if (!values.length) return 0;
        var sorted = values.slice().sort(function (a, b) { return a - b; });
        var index = Math.min(sorted.length - 1, Math.ceil((p / 100) * sorted.length) - 1);
        return sorted[index];
    }

    window.addEventListener("gb-run-card-mounted", function (event) {
        var started = event.detail && event.detail.startedAt;
        if (!started) return;
        firstCardAt = now();
        push("first_paint", Math.max(0, firstCardAt - started));
    });

    window.addEventListener("gb-run-frame", function (event) {
        if (resuming) return;
        var frame = event.detail && event.detail.frame;
        if (!frame || !frame.ts) return;
        var sent = Date.parse(frame.ts);
        if (!sent) return;
        push("log_latency", Math.max(0, Date.now() - sent));
    });

    window.addEventListener("gb-chat-connected", function () {
        reconnectStarted = now();
        // A resume replays frames recorded earlier; they are not live latency.
        resuming = true;
    });

    window.addEventListener("run_resume_done", function () {
        resuming = false;
        if (reconnectStarted !== null) {
            push("reconnect_resume", now() - reconnectStarted);
            reconnectStarted = null;
        }
    });

    window.GBPerf = {
        mark: push,
        metrics: function () {
            var byKind = {};
            samples.forEach(function (sample) {
                (byKind[sample.kind] = byKind[sample.kind] || []).push(sample.value);
            });
            return {
                first_paint_ms: byKind.first_paint || [],
                log_latency_ms: byKind.log_latency || [],
                reconnect_resume_ms: byKind.reconnect_resume || [],
                log_latency_p95: percentile(byKind.log_latency || [], 95),
                samples: samples.slice(),
            };
        },
        clear: function () {
            samples = [];
        },
    };
})();
