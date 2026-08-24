"use strict";
/**
 * Vibe Shell — canvas task-card flow (issue #1177).
 * An ADDITIVE layer on the draw surface: canonical stage cards
 * plan → build → verify → merge → deploy wired to the live run data the
 * existing modules already consume (GET /api/vibe/runs, /api/vibe/run/:id,
 * /api/vibe/pipeline/:use_case). The manual design layer
 * (#vibeDesignSurface / #vibeSteps) is never modified.
 *
 * Card click emits CustomEvent "gb-vibe-card-click" with detail {step,
 * run_id, state} for a trace panel to consume later.
 */
(function () {
    "use strict";

    var S = window.VibeShell;
    var STEPS = ["plan", "build", "verify", "merge", "deploy"];
    var STEP_LABELS = { plan: "Plan", build: "Build", verify: "Verify", merge: "Merge", deploy: "Deploy" };
    var POLL_MS = 5000;

    var state = { runId: null, runState: null, pipelineStages: null, timer: null };

    function api(path) {
        return vibeAuthFetch(path).then(function (r) {
            return r.json().catch(function () { return null; });
        });
    }

    function layer() { return document.getElementById("vibeShellFlowLayer"); }
    function svg() { return document.getElementById("vibeShellFlowSvg"); }

    function ensureLayer() {
        if (layer()) return layer();
        var canvas = document.getElementById("vibeCanvas");
        if (!canvas) return null;
        var host = document.createElement("div");
        host.id = "vibeShellFlowLayer";
        host.className = "vibe-shell-flow";
        host.setAttribute("aria-label", "Pipeline task cards");
        var svgEl = document.createElementNS("http://www.w3.org/2000/svg", "svg");
        svgEl.id = "vibeShellFlowSvg";
        svgEl.classList.add("vibe-shell-flow-svg");
        svgEl.setAttribute("aria-hidden", "true");
        host.appendChild(svgEl);
        canvas.appendChild(host);
        return host;
    }

    /* Deterministic status derivation from the authoritative run state. */
    function deriveStatuses(runState) {
        var s = String(runState || "").toLowerCase();
        var statuses = {};
        STEPS.forEach(function (step) { statuses[step] = "pending"; });
        if (!s) return statuses;
        if (s === "completed") {
            STEPS.forEach(function (step) { statuses[step] = "done"; });
        } else if (s === "failed") {
            statuses.plan = "done";
            statuses.build = "failed";
        } else if (s === "cancelled") {
            statuses.plan = "done";
            statuses.build = "paused";
        } else if (s === "awaiting_approval") {
            statuses.plan = "done";
            statuses.build = "done";
            statuses.verify = "active";
        } else { /* pending | running */
            statuses.plan = s === "pending" ? "active" : "done";
            if (s !== "pending") statuses.build = "active";
        }
        refineFromPipeline(statuses);
        return statuses;
    }

    /* When the backend pipeline definition is available and a run is in
       flight, advance the markers along the real stage order instead of
       assuming only plan→build progress. */
    function refineFromPipeline(statuses) {
        if (!state.pipelineStages || !state.pipelineStages.length) return;
        var matched = [];
        state.pipelineStages.forEach(function (st) {
            var kind = String(st.kind || "").toLowerCase();
            var name = String(st.name || st.id || "").toLowerCase();
            var text = kind + " " + name;
            if (/classif|plan/.test(text)) matched.push("plan");
            else if (/compile|build/.test(text)) matched.push("build");
            else if (/execut|verif|test/.test(text)) matched.push("verify");
            else if (/merge|publish/.test(text)) matched.push("merge");
            else if (/deploy/.test(text)) matched.push("deploy");
        });
        if (matched.length < 2) return;
        if (String(state.runState || "").toLowerCase() !== "running") return;
        for (var i = 0; i < matched.length - 1; i++) {
            statuses[matched[i]] = "done";
        }
        var last = matched[matched.length - 1];
        if (statuses[last] === "pending") statuses[last] = "active";
    }

    function render() {
        var host = ensureLayer();
        if (!host) return;
        var statuses = deriveStatuses(state.runState);
        var existing = host.querySelectorAll(".vibe-shell-flow-card");
        if (existing.length !== STEPS.length) {
            host.querySelectorAll(".vibe-shell-flow-card").forEach(function (n) { n.remove(); });
            STEPS.forEach(function (step, index) {
                host.appendChild(buildCard(step, index));
            });
        }
        STEPS.forEach(function (step) {
            var card = host.querySelector('[data-flow-step="' + step + '"]');
            if (!card) return;
            card.setAttribute("data-status", statuses[step]);
            var badge = card.querySelector(".vibe-shell-flow-state");
            if (badge) badge.textContent = statuses[step];
        });
        requestAnimationFrame(drawArrows);
    }

    function buildCard(step, index) {
        var card = document.createElement("button");
        card.type = "button";
        card.className = "vibe-shell-flow-card";
        card.setAttribute("data-flow-step", step);
        card.innerHTML =
            '<span class="vibe-shell-flow-index">' + (index + 1) + "</span>" +
            '<span class="vibe-shell-flow-title">' + STEP_LABELS[step] + "</span>" +
            '<span class="vibe-shell-flow-state">pending</span>' +
            '<span class="vibe-anchor left"></span><span class="vibe-anchor right"></span>';
        card.addEventListener("click", function () {
            document.dispatchEvent(new CustomEvent("gb-vibe-card-click", {
                detail: { step: step, run_id: state.runId, state: state.runState },
                bubbles: true,
            }));
        });
        return card;
    }

    function anchorPoint(card, side) {
        var hostRect = layer().getBoundingClientRect();
        var rect = card.getBoundingClientRect();
        return {
            x: Math.round((side === "right" ? rect.right : rect.left) - hostRect.left),
            y: Math.round(rect.top + rect.height / 2 - hostRect.top),
        };
    }

    function drawArrows() {
        var host = layer();
        var svgEl = svg();
        if (!host || !svgEl) return;
        var cards = STEPS.map(function (step) {
            return host.querySelector('[data-flow-step="' + step + '"]');
        }).filter(Boolean);
        if (cards.length < 2) return;
        var w = Math.max(1, host.clientWidth);
        var h = Math.max(1, host.clientHeight);
        svgEl.setAttribute("viewBox", "0 0 " + w + " " + h);
        var lines = "";
        for (var i = 0; i < cards.length - 1; i++) {
            var from = anchorPoint(cards[i], "right");
            var to = anchorPoint(cards[i + 1], "left");
            var midX = Math.round((from.x + to.x) / 2);
            lines += '<path class="vibe-shell-flow-arrow" d="M' + from.x + " " + from.y +
                " C " + midX + " " + from.y + ", " + midX + " " + to.y + ", " + to.x + " " + to.y + '"/>';
        }
        svgEl.innerHTML = lines;
    }

    function adoptRun(runId) {
        if (!runId || runId === state.runId) return;
        state.runId = String(runId);
        state.runState = null;
        render();
    }

    function pickActiveRun(runs) {
        if (!Array.isArray(runs) || !runs.length) return null;
        var active = runs.find(function (r) {
            var st = String(r.state || "").toLowerCase();
            return st === "running" || st === "awaiting_approval" || st === "pending";
        });
        return active || runs[0] || null;
    }

    function poll() {
        api("/api/vibe/runs?limit=8").then(function (runs) {
            var chosen = pickActiveRun(runs);
            if (chosen && chosen.run_id && !state.runId) adoptRun(chosen.run_id);
            if (!state.runId) { render(); return; }
            return api("/api/vibe/run/" + encodeURIComponent(state.runId)).then(function (run) {
                if (!run || !run.run_id) return;
                if (String(run.state) !== String(state.runState)) {
                    state.runState = run.state;
                    render();
                }
            });
        }).catch(function () { });
        if (!state.pipelineStages) {
            api("/api/vibe/pipeline/software_development").then(function (data) {
                if (data && data.success && data.pipeline && Array.isArray(data.pipeline.stages)) {
                    state.pipelineStages = data.pipeline.stages;
                    render();
                }
            }).catch(function () { });
        }
    }

    function start() {
        ensureLayer();
        render();
        poll();
        document.addEventListener("gb:vibe-run", function (e) {
            if (e.detail && e.detail.run_id) adoptRun(e.detail.run_id);
        });
        window.addEventListener("resize", drawArrows);
        if (window.GBAppLifecycle && typeof window.GBAppLifecycle.interval === "function") {
            state.timer = window.GBAppLifecycle.interval("vibe", poll, POLL_MS);
        } else {
            state.timer = setInterval(poll, POLL_MS);
        }
    }

    function stop() {
        if (state.timer) {
            clearInterval(state.timer);
            state.timer = null;
        }
        var host = layer();
        if (host) host.remove();
    }

    window.VibeShell.canvasFlow = { start: start, stop: stop, refresh: render };
})();
