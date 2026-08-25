/**
 * Vibe specialist broadcast sync (#1189).
 *
 * The Vibe main window and the standalone specialist pages (Knowledge Graph,
 * Metrics, Members, Deploy, DB) are separate desktop windows sharing one
 * BroadcastChannel. Project selection in any of them propagates to all of
 * them, so the specialists always follow the project the user is working on.
 */
(function () {
    "use strict";

    var CHANNEL = "vibe-specialists";
    var channel = null;

    try {
        channel = new BroadcastChannel(CHANNEL);
    } catch (e) {
        return; // BroadcastChannel unsupported (older browsers): no sync.
    }

    function currentProjectId() {
        return (typeof window.currentProjectId !== "undefined" && window.currentProjectId)
            ? String(window.currentProjectId)
            : null;
    }

    // Local project selection → announce to every specialist window.
    document.addEventListener("gb:vibe-project", function (e) {
        var id = e.detail && (e.detail.id || e.detail.project);
        if (!id) return;
        channel.postMessage({ type: "project", id: String(id) });
    });

    channel.onmessage = function (ev) {
        var msg = ev.data || {};
        if (msg.type !== "project" || !msg.id) return;
        if (msg.id === currentProjectId()) return;
        if (typeof currentProjectId !== "undefined") window.currentProjectId = msg.id;
        document.dispatchEvent(new CustomEvent("gb:vibe-project", { detail: { id: msg.id, project: msg.id } }));
    };
})();
