/**
 * Vibe Ribbon (#806) — stage tabs (PLAN/BUILD/REVIEW/DEPLOY/MONITOR) each
 * reveal the big command buttons that belong to that phase. Replaces the
 * previous pipeline tabs which only showed a hardcoded tool string.
 */
(function () {
    "use strict";

    var D = window.VibeDialogs;
    var STAGES = ["plan", "build", "review", "deploy", "monitor"];

    function activate(stage) {
        if (!stage || STAGES.indexOf(stage) === -1) stage = "build";
        var tabs = document.querySelectorAll(".vibe-ribbon-tab");
        var groups = document.querySelectorAll(".vibe-ribbon-group");
        tabs.forEach(function (t) {
            t.classList.toggle("active", t.getAttribute("data-stage") === stage);
        });
        groups.forEach(function (g) {
            g.classList.toggle("active", g.getAttribute("data-group") === stage);
        });
        loadStatus();
    }

    function loadStatus() {
        var el = document.getElementById("vibeRibbonStatus");
        if (!el) return;
        el.textContent = "· · ·";
        D.api("/api/vibe/runs")
            .then(function (data) {
                // /api/vibe/runs returns a bare array (not {runs:[...]}).
                var runs = Array.isArray(data) ? data : (data && data.runs) || [];
                if (!runs.length) {
                    el.textContent = "no runs yet";
                    return;
                }
                var latest = runs[0];
                var st = String(latest.state || "?").toUpperCase();
                var intent = (latest.intent || "").trim().substring(0, 40);
                el.textContent = "last run " + st + (intent ? " · " + intent : "");
            })
            .catch(function () {
                el.textContent = "";
            });
    }

    function bind() {
        var ribbon = document.getElementById("vibeRibbon");
        if (!ribbon) return;
        ribbon.addEventListener("click", function (e) {
            var tab = e.target.closest(".vibe-ribbon-tab");
            if (!tab) return;
            activate(tab.getAttribute("data-stage"));
        });
        activate("build");
    }

    document.addEventListener("gb:vibe-deeplink-loaded", bind);
    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", bind);
    } else {
        bind();
    }

    window.VibePipeline = {
        activate: activate,
    };
})();
