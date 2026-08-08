/**
 * Vibe Bridge (#753) — cross-surface control and deeplinks.
 * Routes `?vibe=<project>&run_id=<id>` URL params to the Vibe app window,
 * exposes window.VibeB for chat/launcher surfaces, and forwards
 * app params to the Vibe partial via a custom event.
 */
(function () {
    "use strict";

    function parseQuery() {
        var qs = new URLSearchParams(window.location.search);
        return {
            project: qs.get("vibe") || "",
            run_id: qs.get("run_id") || "",
            open: qs.get("vibe") !== null || qs.get("run_id") !== null
        };
    }

    function openVibe(params) {
        var p = params || {};
        if (!window.openDeepLink) return false;
        var q = {};
        if (p.project) q.project = String(p.project);
        if (p.run_id) q.run_id = String(p.run_id);
        window.openDeepLink("vibe", q);
        window.__gbAppParams__ = Object.assign({}, window.__gbAppParams__ || {}, q);
        var evt = new CustomEvent("gb:vibe-params", { detail: q });
        window.dispatchEvent(evt);
        return true;
    }

    function consumeUrlParams() {
        var q = parseQuery();
        if (!q.open) return;
        setTimeout(function () {
            openVibe({ project: q.project || null, run_id: q.run_id || null });
        }, 600);
    }

    window.VibeB = {
        open: openVibe,
        openProject: function (projectId) { return openVibe({ project: projectId }); },
        openRun: function (runId) { return openVibe({ run_id: runId }); },
        hasParams: function () { return parseQuery().open; }
    };

    window.addEventListener("DOMContentLoaded", consumeUrlParams);
    if (document.readyState !== "loading") consumeUrlParams();
})();