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
        // ("vibe-assistant" removed — chat lives in the Chat app window.)
        "vibe-graph": { elId: "vibeGraphPanel", title: "Knowledge Graph", display: "flex", size: { w: "90vw", h: "90vh" } },
        // Fixed-size popups (product spec): New Project / Metrics / Members
        // open at a set footprint with ALL fields visible — no scrollbars,
        // no resizing; only the long members list scrolls internally.
        "vibe-metrics": { elId: "vibeMetricsPanel", title: "Metrics", display: "flex", size: { w: "520px", h: "420px" } },
        // New Project is a fixed-height popup (product spec): all fields
        // visible at once, taller than the content would otherwise need, and
        // NOT resizable — it must not grow a scrollbar nor a resize handle.
        // Height tracks the form content (Name + 3 kinds + env tier +
        // framework + footer) so there is no dead space below the fields.
        "vibe-newproject": { elId: "vibeNewProjectModal", title: "New Project", display: "flex", popup: true, size: { w: "540px", h: "480px" } },
        "vibe-members": { elId: "vibeMembersModal", title: "Project Members", display: "flex", popup: true, size: { w: "560px", h: "auto" } },
    };

    var RUN_DOCK_PARTIAL = "/suite/partials/vibe-run-panel.html?v=15";

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

    // Re-injecting the vibe partial (desktop relaunch, htmx body swap)
    // duplicates every panel id in the document. The stale copies become the
    // "ghost windows/metrics/graphs" reported by users: duplicated tool
    // content stacked inside old windows. Keep ONLY the last occurrence (the
    // fresh partial markup) and drop every earlier copy.
    function dedupeGhosts() {
        var ids = ["vibeGraphPanel", "vibeMetricsPanel",
                   "vibeNewProjectModal", "vibeMembersModal", "vibeDialogMask"];
        ids.forEach(function (id) {
            var all = Array.prototype.slice.call(document.querySelectorAll("#" + id));
            if (all.length <= 1) return;
            var keep = all[all.length - 1];
            all.slice(0, -1).forEach(function (stale) {
                if (typeof WM !== "undefined" && WM && keep && stale.compareDocumentPosition(keep) & Node.DOCUMENT_POSITION_FOLLOWING) {
                    // A later element exists — this one is a ghost copy.
                    stale.remove();
                }
            });
        });
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
        // The WindowManager body ships with an empty `.gb-window-content`
        // placeholder stretched to height:100%. A relocated panel is appended
        // as a SIBLING (not inside it), so the empty placeholder stacks under
        // the popup and doubles the body scroll height — showing a scrollbar
        // on fixed-size popups (New Project / Members). Hide the placeholder
        // (and its loading overlay): the relocated panel IS the content now.
        var gbContent = body.querySelector(".gb-window-content");
        if (gbContent) gbContent.style.display = "none";
        var gbLoading = body.querySelector(".gb-window-loading");
        if (gbLoading) gbLoading.style.display = "none";
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
        // 80% of the desktop height, docked at the top, so the TODO board,
        // the runner log and the sessions are all visible at once.
        var winEl = document.getElementById("window-vibe-run");
        if (winEl) {
            winEl.style.top = "10px";
            winEl.style.height = "80vh";
        }
    }

    // Closing every floating accessory when the user switches project: each
    // window shows project-scoped content (canvas project.draw, graph runs,
    // metrics, run dock, dialogs), so stale content from the previous
    // project must not linger. The main window itself stays open.
    function closeVibeSubwindows() {
        var subs = [
            "vibe-run", "canvas", "vibe-graph", "vibe-metrics",
            "vibe-newproject", "vibe-members",
            "vibe-tool-project", "vibe-tool-terminal", "vibe-tool-browser",
        ];
        subs.forEach(function (id) {
            if (WM.getWindow(id)) WM.close(id);
        });
        // Also close any other tool windows owned by the vibe app.
        (WM.openWindows || []).slice().forEach(function (w) {
            if (w.ownerId === "vibe" && w.id !== "vibe") WM.close(w.id);
        });
    }

    // No accessory opens at startup: the Vibe window is toolbar + project
    // list only. Accessory panels stay parked (hidden) inside the vibe
    // window until their toolbar button opens them as floating tool windows
    // on demand (run dock, canvas, graph, metrics, assistant, dialogs…).
    (function init() {
        // Kill ghost duplicates from any prior partial injection first,
        // then re-run shortly after to catch late htmx swaps (no timers).
        dedupeGhosts();
        requestAnimationFrame(function () { dedupeGhosts(); });
        requestAnimationFrame(function () {
            requestAnimationFrame(dedupeGhosts);
        });

        var hidden = {
            // No #vibeCanvas parked panel exists — the canvas is the official
            // deep-linked app, not a hidden panel inside the vibe window.
            // (No #vibeChatOverlay either — the runner chat was removed.)
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
            // #1271 — automatic dev-VM lifecycle: closing the vibe window
            // stops the selected project's dev VM (Run restarts it).
            var pid = window.currentProjectId || null;
            if (pid && window.VibeShell && window.VibeShell.toolbar &&
                typeof window.VibeShell.toolbar.stopDevVm === "function") {
                window.VibeShell.toolbar.stopDevVm(pid);
            }
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
        // CANVAS APP (#1191/#1271): the Canvas button opens the SHARED Canvas
        // App (the official whiteboard at /suite/canvas/canvas.html) via the
        // window-manager deep link. The old "Vibe Canvas" window is REMOVED
        // from vibe — there is no custom in-window canvas panel, only the
        // real Canvas App, reused everywhere (desktop launcher + vibe).
        openCanvas: function () {
            dedupeGhosts();
            var pid = (typeof currentProjectId !== "undefined" && currentProjectId) || "";
            WM.openDeepLink("canvas", pid ? { project: pid } : {}, { ownerId: "vibe" });
        },
        // Project properties/info dialog for the currently selected project.
        openProjectInfo: function () {
            dedupeGhosts();
            var pid = (typeof window.currentProjectId !== "undefined" && window.currentProjectId) || null;
            if (!pid) {
                if (window.VibeShell && window.VibeShell.toolbar && typeof window.VibeShell.toolbar.flashHint === "function") {
                    window.VibeShell.toolbar.flashHint("SELECT A PROJECT FIRST");
                }
                return;
            }
            var show = function (p) {
                if (typeof window.showProjectInfo === "function") {
                    window.showProjectInfo(p || { id: pid, name: String(window.currentProject || pid) });
                }
            };
            vibeAuthFetch("/api/vibe/projects")
                .then(function (r) {
                    if (!r.ok) throw new Error("Project lookup failed");
                    return r.json();
                })
                .then(function (d) {
                    var rows = (d && d.projects) || (d && d.data && d.data.projects) || (Array.isArray(d) ? d : []) || [];
                    var p = rows.find(function (row) {
                        return String(row.project_id || row.id) === String(pid);
                    });
                    show(p);
                })
                .catch(function () { show(null); });
        },
        // openMoved() only relocates the parked panel into its tool window;
        // the panel's own activator must run afterwards or the canvas stays
        // 1x1 and the body empty (graph pollution/empty-panel bug).
        openGraph: function () {
            openMoved("vibe-graph");
            // Always bring the Knowledge Graph to ~90% of the desktop, even
            // when the tool window already exists (openMoved applies size
            // only on first create). Force the window size every open.
            var winEl = document.getElementById("window-vibe-graph");
            if (winEl) {
                winEl.style.width = "90vw";
                winEl.style.height = "90vh";
                winEl.style.left = (window.innerWidth * 0.05) + "px";
                winEl.style.top = (window.innerHeight * 0.05) + "px";
            }
            if (window.VibeGraph) window.VibeGraph.togglePanel(true);
        },
        openMetrics: function () {
            openMoved("vibe-metrics");
            if (window.VibeMetrics) window.VibeMetrics.open();
        },
        openNewProject: function () { openMoved("vibe-newproject"); },
        closeVibeSubwindows: closeVibeSubwindows,
        openMembers: function () { openMoved("vibe-members"); },
    };
})();
