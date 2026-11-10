"use strict";

/**
 * Operational view controller for the Analytics app (#2116).
 *
 * The Operations tab is lazy: panels are only fetched when the tab is opened
 * (and then every 60 s while it stays open), so the product dashboard keeps
 * its original request pattern. `showOpsPanel` is used by the SLO table to
 * jump from an objective to the panel that explains it.
 */

(function () {
    var OPS_REFRESH_MS = 60000;
    var currentWindow = "60";
    var timer = null;

    function panels() {
        return Array.prototype.slice.call(
            document.querySelectorAll(".ops-panel-slot"),
        );
    }

    function isOperationsVisible() {
        var pane = document.getElementById("analytics-operations-pane");
        return !!pane && pane.classList.contains("active");
    }

    function urlFor(panel) {
        var base = panel.getAttribute("data-ops-url") || "";
        if (panel.getAttribute("data-ops-windowed") === "1") {
            return base + "?window=" + encodeURIComponent(currentWindow);
        }
        return base;
    }

    function loadPanels() {
        if (!window.htmx) return;
        panels().forEach(function (panel) {
            var url = urlFor(panel);
            panel.setAttribute("hx-get", url);
            htmx.ajax("GET", url, { target: panel, swap: "innerHTML" });
        });
    }

    window.setOpsWindow = function (value) {
        currentWindow = value || "60";
        if (isOperationsVisible()) loadPanels();
    };

    window.refreshOps = function () {
        if (isOperationsVisible()) loadPanels();
    };

    window.showAnalyticsTab = function (name) {
        var target = name === "operations" ? "operations" : "product";
        document.querySelectorAll(".analytics-tab").forEach(function (tab) {
            tab.classList.toggle("active", tab.dataset.tab === target);
        });
        var product = document.getElementById("analytics-product-pane");
        var operations = document.getElementById("analytics-operations-pane");
        if (product) product.classList.toggle("active", target === "product");
        if (operations) {
            operations.classList.toggle("active", target === "operations");
        }
        if (target === "operations") {
            loadPanels();
            startTimer();
        } else {
            stopTimer();
        }
    };

    window.showOpsPanel = function (panelId) {
        window.showAnalyticsTab("operations");
        var panel = document.getElementById(panelId + "-panel");
        if (panel && panel.scrollIntoView) {
            panel.scrollIntoView({ behavior: "smooth", block: "start" });
        }
    };

    function startTimer() {
        stopTimer();
        timer = window.setInterval(function () {
            if (isOperationsVisible()) loadPanels();
        }, OPS_REFRESH_MS);
        window.__analyticsOpsTimer = timer;
    }

    function stopTimer() {
        if (timer !== null) {
            window.clearInterval(timer);
            timer = null;
        }
        window.__analyticsOpsTimer = null;
    }

    // The analytics app is an HTMX fragment: it may be injected more than once
    // per session, so a timer left behind by a torn-down instance is cleared.
    if (window.__analyticsOpsTimer) {
        window.clearInterval(window.__analyticsOpsTimer);
        window.__analyticsOpsTimer = null;
    }
})();
