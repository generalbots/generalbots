/**
 * Vibe Pipeline tabs (#806) — real stage data under each tab.
 * Clicking PLAN/BUILD/REVIEW/DEPLOY/MONITOR loads the pipeline
 * definition (/api/vibe/pipeline/:use_case) plus the latest runs
 * (/api/vibe/runs) into a stage panel under the canvas header.
 */
(function () {
    "use strict";

    var D = window.VibeDialogs;
    var stageCache = null;

    function stageMap() {
        return {
            plan: { label: "PLAN", tool: "classify_intent, compile_plan" },
            build: { label: "BUILD", tool: "execute_plan, file/write, shell/run" },
            review: { label: "REVIEW", tool: "verify, build_test" },
            deploy: { label: "DEPLOY", tool: "commit_push, publish, domain/bind" },
            monitor: { label: "MONITOR", tool: "logs, metrics" },
        };
    }

    function loadPipeline() {
        if (stageCache) return Promise.resolve(stageCache);
        return D.api("/api/vibe/pipeline/software_development").then(function (data) {
            stageCache = (data && (data.pipeline || data.stages)) || null;
            return stageCache;
        }).catch(function () {
            return null;
        });
    }

    function latestRuns() {
        return D.api("/api/vibe/runs").then(function (data) {
            return (data && data.runs) || [];
        }).catch(function () {
            return [];
        });
    }

    function renderStage(stage) {
        var panel = document.getElementById("vibeStagePanel");
        if (!panel) return;
        var tabs = document.querySelectorAll(".vibe-pipeline-tab");
        tabs.forEach(function (t) {
            t.classList.toggle("active", t.getAttribute("data-stage") === stage);
        });
        panel.classList.add("visible");

        loadPipeline().then(function (pipe) {
            latestRuns().then(function (runs) {
                var s = stageMap()[stage] || stageMap().build;
                var html = '<div style="display:flex;align-items:center;gap:10px;margin-bottom:8px;">' +
                    '<span class="vibe-status info">' + s.label + "</span>" +
                    '<span style="font-size:11px;color:var(--text-muted);">' + D.esc(s.tool) + "</span></div>";

                var stages = pipe;
                if (stages && stages.length) {
                    html += '<div class="vibe-stage-row done"><span class="idx">✓</span><span>' +
                        D.esc(s.label) + " stage — pipeline: " + D.esc(stages.map(function (x) { return x.name || x.id; }).join(" → ")) +
                        "</span></div>";
                }

                var mine = runs.filter(function (r) {
                    return r && (r.state || "").toLowerCase() ===
                        (stage === "monitor" ? "completed" : "running");
                }).slice(0, 3);
                if (!mine.length) mine = runs.slice(0, 2);

                if (mine.length) {
                    html += '<div class="vibe-stage-row"><span class="idx">↻</span><span style="color:var(--text);">Recent runs:</span></div>';
                    mine.forEach(function (r) {
                        var st = String(r.state || "?").toUpperCase();
                        var cls = st === "COMPLETED" ? "ok" : st === "FAILED" ? "err" : "warn";
                        html += '<div class="vibe-stage-row"><span class="idx">•</span>' +
                            '<span style="color:var(--text-secondary);">' + D.esc((r.intent || "").substring(0, 60) || String(r.run_id).substring(0, 8)) + "</span>" +
                            '<span class="vibe-status ' + cls + '">' + D.esc(st) + "</span></div>";
                    });
                }
                panel.innerHTML = html;
            });
        });
    }

    function bindTabs() {
        var container = document.querySelector(".vibe-pipeline");
        if (!container) return;
        container.addEventListener("click", function (e) {
            var tab = e.target.closest(".vibe-pipeline-tab");
            if (!tab) return;
            renderStage(tab.getAttribute("data-stage") || "build");
        });
    }

    document.addEventListener("gb:vibe-deeplink-loaded", function () {
        bindTabs();
    });

    (function () {
        var __cb = function () {
            bindTabs();
        };
        if (document.readyState === "loading") {
            document.addEventListener("DOMContentLoaded", __cb);
        } else {
            __cb();
        }
    })();

    window.VibePipeline = {
        render: renderStage,
        invalidate: function () {
            stageCache = null;
        },
    };
})();