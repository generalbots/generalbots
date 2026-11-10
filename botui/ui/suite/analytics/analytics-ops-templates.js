"use strict";

/**
 * Operational view of the Analytics app (#2116).
 *
 * Injects the Operations pane next to the existing product dashboard. Every
 * panel loads from its own analytics endpoint through HTMX; this file only
 * builds the markup, so a panel that fails renders its server-side "no data"
 * state instead of a client-side placeholder.
 */
(function () {
    var slot = document.getElementById("analyticsOpsSlot");
    if (!slot) return;

    var panels = [
        {
            id: "ops-slo-panel",
            url: "/api/ui/analytics/ops/slo",
            windowed: false,
            label: "Loading SLO burn...",
        },
        {
            id: "ops-traffic-panel",
            url: "/api/ui/analytics/ops/traffic",
            windowed: true,
            label: "Loading traffic...",
        },
        {
            id: "ops-endpoints-panel",
            url: "/api/ui/analytics/ops/endpoints",
            windowed: true,
            label: "Loading endpoint counters...",
        },
        {
            id: "ops-queue-panel",
            url: "/api/ui/analytics/ops/queue",
            windowed: true,
            label: "Loading queues...",
        },
        {
            id: "ops-cost-panel",
            url: "/api/ui/analytics/ops/cost",
            windowed: true,
            label: "Loading LLM cost...",
        },
    ];

    var runbooks = [
        ["Deploy & rollback", "runbook-deploy-rollback.md"],
        ["Restore PostgreSQL/MinIO", "runbook-restore-backup.md"],
        ["Rotate Vault secrets", "runbook-rotate-secrets.md"],
        ["Drain/disable a tenant", "runbook-disable-tenant.md"],
        ["LLM provider outage", "runbook-llm-outage.md"],
        ["Disk full / stuck queue", "runbook-disk-and-queue.md"],
    ];

    var base =
        "https://github.com/generalbots/generalbots/blob/main/botbook/src/10-configuration-deployment/operations/";

    var panelHtml = panels
        .map(function (panel) {
            var url = panel.url + (panel.windowed ? "?window=60" : "");
            return (
                '<div class="ops-panel-slot" id="' +
                panel.id +
                '" data-ops-windowed="' +
                (panel.windowed ? "1" : "0") +
                '" data-ops-url="' +
                panel.url +
                '" hx-get="' +
                url +
                '" hx-trigger="ops-load" hx-swap="innerHTML">' +
                '<div class="chart-loading"><div class="spinner"></div><span>' +
                panel.label +
                "</span></div></div>"
            );
        })
        .join("");

    var runbookHtml = runbooks
        .map(function (entry) {
            return (
                '<li><a href="' +
                base +
                entry[1] +
                '" target="_blank" rel="noopener">' +
                entry[0] +
                "</a></li>"
            );
        })
        .join("");

    slot.outerHTML =
        '<section class="analytics-ops" id="analytics-operations">' +
        '<div class="ops-toolbar">' +
        '<label for="opsWindow">Window</label>' +
        '<select id="opsWindow" class="time-selector" onchange="setOpsWindow(this.value)">' +
        '<option value="60" selected>Last hour</option>' +
        '<option value="360">Last 6 hours</option>' +
        '<option value="1440">Last 24 hours</option>' +
        '<option value="10080">Last 7 days</option>' +
        "</select>" +
        '<button class="btn-refresh" onclick="refreshOps()" title="Refresh operational panels">' +
        '<svg width="18" height="18" viewBox="0 0 24 24" fill="none">' +
        '<path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>' +
        '<path d="M21 3v5h-5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>' +
        "</svg></button>" +
        '<span class="ops-meta">Panels read live analytics endpoints; empty panels mean the source has no data yet.</span>' +
        "</div>" +
        panelHtml +
        '<div class="ops-panel ops-runbooks">' +
        '<div class="chart-header"><h3>Runbooks</h3>' +
        '<span class="ops-meta">one per alert · stored in botbook</span></div>' +
        "<ul>" +
        runbookHtml +
        "</ul></div>" +
        "</section>";
})();
