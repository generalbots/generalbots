"use strict";

/**
 * Documents tab controller (#2507). Load order matters: this module runs
 * after analytics-ops.js so the existing tab switcher can be wrapped without
 * losing the Operations 60 s refresh timer.
 *
 * The date range is the dashboard's existing #timeRange selector; scope
 * selectors are local to the Documents tab. The API is intentionally absent
 * from this build — a 404 renders the explicit "no data available" state.
 */
(function () {
    var ns = window.AnalyticsDocs;
    if (!ns) return;

    var previousShowTab = window.showAnalyticsTab;
    var timer = null;
    var ticks = 0;
    var loading = false;

    function pane() {
        return document.getElementById("analytics-documents-pane");
    }

    function isVisible() {
        var element = pane();
        return !!element && element.classList.contains("active");
    }
    ns.isVisible = isVisible;

    function encode(value) {
        return encodeURIComponent(String(value == null ? "" : value));
    }

    function queryString() {
        var select = document.getElementById("timeRange");
        var range = select && ns.RANGES[select.value] ? select.value : "24h";
        var to = new Date();
        var from = new Date(to.getTime() - ns.RANGES[range] * 3600000);
        var parts = [
            "from=" + encode(from.toISOString()),
            "to=" + encode(to.toISOString()),
            "range=" + encode(range),
        ];
        ns.SCOPE_FIELDS.forEach(function (field) {
            var element = document.getElementById(field.id);
            if (element && element.value) parts.push(encode(field.param) + "=" + encode(element.value));
        });
        return parts.join("&");
    }

    function clearData() {
        ns.lastDisplay = {};
        ns.lastModel = null;
        ns.renderFreshness(null);
    }

    function load() {
        if (loading || !isVisible()) return;
        loading = true;
        ns.setStatus("loading");
        ns.markAllTables("loading");

        fetch(ns.ENDPOINT + "?" + queryString(), {
            headers: { Accept: "application/json" },
            credentials: "same-origin",
        })
            .then(function (response) {
                if (response.status === 404 || response.status === 204) return null;
                if (!response.ok) throw new Error("analytics request failed");
                return response.json();
            })
            .then(function (payload) {
                var model = ns.normalize(payload);
                if (!model) {
                    clearData();
                    ns.setStatus("empty");
                    ns.markAllTables("empty");
                    return;
                }
                ns.render(model);
            })
            .catch(function () {
                clearData();
                ns.setStatus("error");
                ns.markAllTables("error");
            })
            .then(function () {
                loading = false;
            });
    }
    ns.load = load;

    function stopTimer() {
        if (timer !== null) window.clearInterval(timer);
        timer = null;
        ticks = 0;
    }

    function startTimer() {
        stopTimer();
        timer = window.setInterval(function () {
            if (!isVisible()) {
                stopTimer();
                return;
            }
            ns.renderFreshness(ns.lastModel ? ns.lastModel.generatedAt : null);
            ticks += 1;
            if (ticks % 5 === 0) load();
        }, 60000);
    }

    function activate() {
        load();
        startTimer();
    }
    ns.activate = activate;

    function showTab(name) {
        var target = name === "documents" ? "documents" : name;
        if (typeof previousShowTab === "function") {
            previousShowTab(target === "documents" ? "product" : target);
        }
        Array.prototype.forEach.call(document.querySelectorAll(".analytics-tab"), function (tab) {
            var active = tab.getAttribute("data-tab") === target;
            tab.classList.toggle("active", active);
            tab.setAttribute("aria-selected", active ? "true" : "false");
        });
        ["product", "operations", "documents"].forEach(function (id) {
            var element = document.getElementById("analytics-" + id + "-pane");
            if (element) element.classList.toggle("active", id === target);
        });
        if (target === "documents") activate();
        else stopTimer();
    }
    window.showAnalyticsTab = showTab;

    function bindEvents() {
        if (window.__analyticsDocsBound) return;
        window.__analyticsDocsBound = true;

        document.addEventListener("change", function (event) {
            var target = event.target;
            if (!target) return;
            if (target.id === "timeRange" || target.getAttribute("data-docs-scope")) {
                if (isVisible()) load();
            }
        });

        document.addEventListener("click", function (event) {
            var target = event.target;
            if (!target || typeof target.closest !== "function") return;
            if (target.closest("#docsRefresh") && isVisible()) load();
            if (target.closest("#docsExport") && isVisible()) ns.exportCsv();
        });

        window.addEventListener("localeChanged", function () {
            if (!isVisible()) return;
            if (ns.lastModel) ns.render(ns.lastModel);
            else ns.markAllTables("empty");
        });
    }

    function init() {
        bindEvents();
        if (isVisible()) activate();
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", init);
    } else {
        init();
    }
})();
