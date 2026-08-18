/**
 * Vibe Metrics panel — real run telemetry from /api/vibe/metrics.
 * Distinct from the Knowledge Graph panel: shows totals (runs, success,
 * tool calls, latency, cost) plus a per-use-case breakdown. Polls the
 * endpoint so the numbers stay fresh while runs are in flight.
 */
"use strict";

// Guard against double-declaration: the desktop window manager re-injects
// this script every time the Vibe app opens (HTMX body swap), so a top-level
// `const` would throw 'Identifier VibeMetrics has already been declared' on
// the second open. Assigning to window keeps a single singleton; init()
// re-binds the close button from the fresh DOM on each load.
window.VibeMetrics = window.VibeMetrics || {
    pollTimer: null,

    init: function () {
        const close = document.getElementById("vibeMetricsClose");
        if (close) close.addEventListener("click", () => this.close());
    },

    open: function () {
        const panel = document.getElementById("vibeMetricsPanel");
        if (!panel) return;
        // Close the knowledge graph so the two floating panels never stack.
        if (window.VibeGraph) VibeGraph.togglePanel(false);
        panel.style.display = "flex";
        this.refresh();
        if (this.pollTimer) clearInterval(this.pollTimer);
        this.pollTimer = setInterval(() => this.refresh(), 15000);
    },

    close: function () {
        const panel = document.getElementById("vibeMetricsPanel");
        if (panel) panel.style.display = "none";
        if (this.pollTimer) {
            clearInterval(this.pollTimer);
            this.pollTimer = null;
        }
    },

    refresh: function () {
        const body = document.getElementById("vibeMetricsBody");
        if (!body) return;
        vibeAuthFetch("/api/vibe/metrics")
            .then((r) => r.json())
            .then((data) => {
                const m = data && data.metrics ? data.metrics : null;
                if (!m) {
                    body.innerHTML =
                        '<div class="vibe-empty" style="padding: 20px;">No metrics available yet.</div>';
                    return;
                }
                body.innerHTML = this.render(m);
            })
            .catch((e) => {
                body.innerHTML =
                    '<div class="vibe-empty" style="padding: 20px;">Failed to load metrics: ' +
                    this.esc(e.message || e) +
                    "</div>";
            });
    },

    fmt: function (n) {
        if (typeof n !== "number" || !isFinite(n)) return "0";
        if (n >= 1000000) return (n / 1000000).toFixed(1) + "M";
        if (n >= 1000) return (n / 1000).toFixed(1) + "k";
        return Math.round(n).toString();
    },

    fmtCost: function (n) {
        if (typeof n !== "number" || !isFinite(n) || n === 0) return "$0.00";
        return "$" + n.toFixed(2);
    },

    esc: function (s) {
        return String(s)
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;");
    },

    render: function (m) {
        const total = m.total_runs || 0;
        const done = m.completed_runs || 0;
        const failed = m.failed_runs || 0;
        const rate = total > 0 ? Math.round((done / total) * 100) : 0;
        const cells = [
            { label: "Total runs", value: this.fmt(total), cls: "vm-total" },
            { label: "Completed", value: this.fmt(done), cls: "vm-ok" },
            { label: "Failed", value: this.fmt(failed), cls: "vm-bad" },
            { label: "Success", value: rate + "%", cls: "vm-rate" },
        ];
        const extras = [
            { label: "Tool calls", value: this.fmt(m.total_tool_calls || 0) },
            { label: "Avg latency", value: this.ms(m.avg_latency_ms) },
            { label: "LLM cost", value: this.fmtCost(m.total_cost) },
        ];
        let html =
            '<div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; margin-bottom: 10px;">';
        cells.forEach((c) => {
            html +=
                '<div style="background: var(--bg, #0d1117); border: 1px solid var(--border, #222); border-radius: 8px; padding: 10px 8px; text-align: center;">' +
                '<div style="font-size: 18px; font-weight: 700; color: ' +
                (c.cls === "vm-bad" ? "#f77" : "var(--accent, #84d669)") +
                ';">' +
                this.esc(c.value) +
                "</div>" +
                '<div style="font-size: 10px; color: var(--text-muted, #888); margin-top: 2px;">' +
                this.esc(c.label) +
                "</div></div>";
        });
        html += "</div>";
        html +=
            '<div style="display: flex; gap: 16px; font-size: 11px; color: var(--text-muted, #888); border-bottom: 1px solid var(--border, #222); padding-bottom: 8px; margin-bottom: 8px;">';
        extras.forEach((e) => {
            html +=
                "<span><strong style='color: var(--text, #eee);'>" +
                this.esc(e.value) +
                "</strong> " +
                this.esc(e.label) +
                "</span>";
        });
        html += "</div>";

        const byUseCase = m.by_use_case || {};
        const keys = Object.keys(byUseCase).sort();
        if (!keys.length) {
            html += '<div style="color: var(--text-muted, #888);">No runs recorded yet.</div>';
        } else {
            html +=
                '<div style="font-size: 11px; font-weight: 700; color: var(--text-muted, #888); text-transform: uppercase; margin: 8px 0 6px;">By use case</div>';
            keys.forEach((uc) => {
                const u = byUseCase[uc] || {};
                const ut = u.total_runs || 0;
                const ud = u.completed_runs || 0;
                const uf = u.failed_runs || 0;
                const pct = ut > 0 ? Math.round((ud / ut) * 100) : 0;
                html +=
                    '<div style="margin-bottom: 8px;">' +
                    '<div style="display: flex; justify-content: space-between; font-size: 11px; margin-bottom: 3px;">' +
                    "<span>" +
                    this.esc(uc.replace(/_/g, " ")) +
                    "</span>" +
                    "<span style='color: var(--text-muted, #888);'>" +
                    this.fmt(ud) +
                    " ok · " +
                    this.fmt(uf) +
                    " fail · " +
                    this.fmt(u.total_tool_calls || 0) +
                    " calls</span></div>" +
                    '<div style="height: 6px; border-radius: 3px; background: var(--border, #222); overflow: hidden;">' +
                    '<div style="height: 100%; width: ' +
                    pct +
                    "%; background: var(--accent, #84d669);\"></div></div></div>";
            });
        }
        return html;
    },

    ms: function (v) {
        if (typeof v !== "number" || !isFinite(v)) return "—";
        if (v >= 60000) return (v / 60000).toFixed(1) + "m";
        if (v >= 1000) return (v / 1000).toFixed(1) + "s";
        return Math.round(v) + "ms";
    },
};

function initVibeMetrics() {
    window.VibeMetrics.init();
}

if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initVibeMetrics);
} else {
    initVibeMetrics();
}
