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
        // Choke point: a BARE open (no project, no run_id) while the user
        // has a remembered close (gb.vibe.closed=1) is a no-op. Every caller
        // (bridge boot, chat-init deeplinks, launcher) goes through here, so
        // even stale cached callers cannot pop the bar back open once the
        // user closed it. Explicit project/run opens carry intent and pass.
        if (!p.project && !p.run_id) {
            try {
                if (localStorage.getItem("gb.vibe.closed") === "1") {
                    return false;
                }
            } catch (e) {
                /* storage unavailable — keep the default open behavior */
            }
        }
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
        // Respect a remembered close (gb.vibe.closed=1) for bare ?vibe deep
        // links: a reload of ?vibe must not pop the bar back open once the
        // user closed it. Explicit project/run deep links (?vibe=<id>,
        // ?run_id=<id>) carry clear intent and still open the bar.
        try {
            if (!q.project && !q.run_id &&
                localStorage.getItem("gb.vibe.closed") === "1") {
                return;
            }
        } catch (e) {
            /* storage unavailable — keep the default open behavior */
        }
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