/**
 * Vibe floating tool windows (VB6/Adobe-style, no modals).
 *
 * The Vibe main window is only commands + project list. Every accessory —
 * Assistant chat, Run Dock, Project Canvas, Knowledge Graph, Metrics and the
 * New-Project / Members dialogs — floats in its own draggable tool window,
 * exactly like VB6/Adobe dockable tool windows over the main IDE.
 *
 * Tool windows are same-document: WindowManager injects the body HTML into the
 * document, so the existing vibe modules (vibe-run.js, vibe-canvas.js,
 * vibe-websocket.js…) keep binding with document.getElementById(). This file
 * runs at script-eval time (before DOMContentLoaded) so the relocation is
 * complete before any module init runs.
 *
 * Closing a tool window removes its DOM node, which would destroy the moved
 * panels. To keep them alive, closed panels are stashed into a hidden
 * container in the main vibe window and re-parented into a fresh window body
 * when reopened — node identity is preserved so listeners survive.
 */
(function () {
    "use strict";

    var WM = window.WindowManager;
    if (!WM) return;

    // App launched isolated (own tab, no desktop shell): keep single-window.
    if (/[?&]isolated=1/.test(window.location.search)) return;

    // Panels that are MOVED (their DOM must be preserved across close/reopen)
    // vs. dialogs whose content is rebuilt on every open.
    var MOVED = {
        "vibe-assistant": { elId: "vibeChatOverlay", title: "Vibe Assistant" },
        "vibe-canvas": { elId: "vibeCanvas", title: "Project Canvas" },
        "vibe-graph": { elId: "vibeGraphPanel", title: "Knowledge Graph" },
        "vibe-metrics": { elId: "vibeMetricsPanel", title: "Metrics" },
        "vibe-newproject": { elId: "vibeNewProjectModal", title: "New Project" },
        "vibe-members": { elId: "vibeMembersModal", title: "Project Members" },
    };

    var RUN_DOCK_PARTIAL = "/suite/partials/vibe-run-panel.html?v=4";

    function stashHost() {
        var host = document.getElementById("vibeHiddenPanels");
        if (!host) {
            host = document.createElement("div");
            host.id = "vibeHiddenPanels";
            host.style.display = "none";
            var vibeWindow = document.getElementById("vibeWindow");
            (vibeWindow || document.body).appendChild(host);
        }
        return host;
    }

    // Open (or focus) a floating tool window hosting a moved panel.
    function openMoved(id) {
        var spec = MOVED[id];
        if (!spec) return;
        if (WM.getWindow(id)) {
            // Re-launching the vibe app injects a fresh partial: drop any
            // stale copy already living in this tool window before focusing.
            var wmBody = document.getElementById("window-body-" + id);
            var fresh = document.getElementById(spec.elId);
            if (wmBody && fresh && wmBody.contains(fresh) === false) {
                var stale = wmBody.querySelector("#" + spec.elId);
                if (stale) stale.remove();
                fresh.classList.add("vibe-tool-relocated");
                wmBody.appendChild(fresh);
            }
            WM.focusWindow(id);
            return;
        }
        var el = document.getElementById(spec.elId);
        if (!el) return;
        var body = WM.openToolWindowBody(id, spec.title);
        if (!body) return;
        el.classList.add("vibe-tool-relocated");
        body.appendChild(el);
    }

    // The run dock partial is fetched fresh, so it needs no stash.
    function openRunDock() {
        if (WM.getWindow("vibe-run")) {
            WM.focusWindow("vibe-run");
            return;
        }
        WM.openToolWindow("vibe-run", "Run Dock", RUN_DOCK_PARTIAL);
    }

    // Relocate everything on first boot.
    (function init() {
        // Assistant chat.
        var chat = document.getElementById("vibeChatOverlay");
        if (chat) {
            var body = WM.openToolWindowBody("vibe-assistant", MOVED["vibe-assistant"].title);
            if (body) {
                var stale = body.querySelector("#vibeChatOverlay");
                if (stale && stale !== chat) stale.remove();
                chat.classList.add("vibe-tool-relocated");
                body.appendChild(chat);
            }
        }

        // Run Dock (partial fetch, no in-window host anymore).
        var host = document.getElementById("vibeRunDockHost");
        if (host) {
            host.removeAttribute("hx-get");
            host.removeAttribute("hx-trigger");
            host.innerHTML = "";
        }
        openRunDock();

        // Canvas, graph, metrics, modals.
        openMoved("vibe-canvas");
        openMoved("vibe-graph");
        openMoved("vibe-metrics");
        openMoved("vibe-newproject");
        openMoved("vibe-members");
    })();

    // Window-chrome ✕: stash moved panels (preserve DOM), hide relocated
    // modals so a later open() can show them again.
    document.addEventListener("gb-window-close", function (e) {
        var id = e.detail && e.detail.id;
        if (MOVED[id]) {
            var el = document.getElementById(MOVED[id].elId);
            if (el) {
                stashHost().appendChild(el);
                el.style.display = "none";
            }
        }
    });

    window.VibeWindows = {
        openRunDock: openRunDock,
        openAssistant: function () { openMoved("vibe-assistant"); },
        openCanvas: function () { openMoved("vibe-canvas"); },
        openGraph: function () { openMoved("vibe-graph"); },
        openMetrics: function () { openMoved("vibe-metrics"); },
        openNewProject: function () { openMoved("vibe-newproject"); },
        openMembers: function () { openMoved("vibe-members"); },
    };
})();
