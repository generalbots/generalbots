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
        "vibe-assistant": { elId: "vibeChatOverlay", title: "Vibe Assistant", display: "flex" },
        // The canvas is a drawing surface (720px min-height) — it needs a
        // real workspace, not the half-height accessory default.
        "vibe-canvas": { elId: "vibeCanvas", title: "Project Canvas", display: "flex", size: { w: "min(1000px, 92vw)", h: "min(780px, 88vh)" } },
        "vibe-graph": { elId: "vibeGraphPanel", title: "Knowledge Graph", display: "flex" },
        "vibe-metrics": { elId: "vibeMetricsPanel", title: "Metrics", display: "flex" },
        "vibe-newproject": { elId: "vibeNewProjectModal", title: "New Project", display: "flex" },
        "vibe-members": { elId: "vibeMembersModal", title: "Project Members", display: "flex", popup: true },
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
            // The panel may have been parked hidden after a close: bring it
            // back to its natural display so the window is never blank.
            var activeEl = document.getElementById(spec.elId);
            if (activeEl) {
                activeEl.style.display = activeEl.getAttribute("data-vibe-panel-display") || spec.display || "flex";
            }
            WM.focusWindow(id);
            return;
        }
        var el = document.getElementById(spec.elId);
        if (!el) return;
        var body = WM.openToolWindowBody(id, spec.title, {
            popup: !!spec.popup,
            ownerId: "vibe",
        });
        if (!body) return;
        el.classList.add("vibe-tool-relocated");
        if (spec.popup) {
            el.classList.add("vibe-popup-panel");
            // The partial ships with inline position:fixed (modal overlay); a
            // popup must flow inside its window instead, and inline styles
            // beat CSS so set them here.
            el.style.position = "static";
            el.style.inset = "auto";
            el.style.width = "auto";
            el.style.height = "auto";
        }
        el.style.display = el.getAttribute("data-vibe-panel-display") || spec.display || "flex";
        body.appendChild(el);
        // Panels with a bespoke footprint (e.g. the drawing canvas) override
        // the half-height accessory default.
        if (spec.size) {
            var winEl = document.getElementById("window-" + id);
            if (winEl) {
                if (spec.size.w) winEl.style.width = spec.size.w;
                if (spec.size.h) winEl.style.height = spec.size.h;
            }
        }
    }

    // The run dock partial is fetched fresh, so it needs no stash.
    function openRunDock() {
        if (WM.getWindow("vibe-run")) {
            WM.focusWindow("vibe-run");
            return;
        }
        WM.openToolWindow("vibe-run", "Run Dock", RUN_DOCK_PARTIAL, {}, { ownerId: "vibe" });
    }

    // No accessory opens at startup: the Vibe window is toolbar + project
    // list only. Accessory panels stay parked (hidden) inside the vibe
    // window until their toolbar button opens them as floating tool windows
    // on demand (run dock, canvas, graph, metrics, assistant, dialogs…).
    (function init() {
        var hidden = {
            "vibeChatOverlay": "flex",
            "vibeCanvas": "flex",
            "vibeGraphPanel": "flex",
            "vibeMetricsPanel": "flex",
            "vibeNewProjectModal": "flex",
            "vibeMembersModal": "flex",
        };
        Object.keys(hidden).forEach(function (elId) {
            var el = document.getElementById(elId);
            if (!el) return;
            el.setAttribute("data-vibe-panel-display", hidden[elId]);
            el.style.display = "none";
        });

        // Run Dock is fetched on demand only; clear any stale host wiring.
        var host = document.getElementById("vibeRunDockHost");
        if (host) {
            host.removeAttribute("hx-get");
            host.removeAttribute("hx-trigger");
            host.innerHTML = "";
        }
    })();

    // Window-chrome ✕: stash moved panels (preserve DOM), hide relocated
    // modals so a later open() can show them again. Closing the Vibe window
    // itself closes every accessory tool window (they are children of vibe).
    document.addEventListener("gb-window-close", function (e) {
        var id = e.detail && e.detail.id;
        if (id === "vibe") {
            Object.keys(MOVED).forEach(function (childId) {
                if (WM.getWindow(childId)) WM.close(childId);
            });
            if (WM.getWindow("vibe-run")) WM.close("vibe-run");
            return;
        }
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
        // The Vibe assistant IS the shared Chat window — same bot chat, same
        // context. There is no separate assistant window (removed 2026-08-26).
        openAssistant: function () {
            if (WM && typeof WM.openDeepLink === "function") {
                WM.openDeepLink("chat", {}, { ownerId: "vibe" });
            } else if (window.VibeShell && window.VibeShell.toolbar) {
                window.VibeShell.toolbar.openChat();
            }
        },
        openCanvas: function () { openMoved("vibe-canvas"); },
        openGraph: function () { openMoved("vibe-graph"); },
        openMetrics: function () { openMoved("vibe-metrics"); },
        openNewProject: function () { openMoved("vibe-newproject"); },
        openMembers: function () { openMoved("vibe-members"); },
    };
})();
