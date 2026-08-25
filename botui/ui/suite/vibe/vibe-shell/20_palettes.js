"use strict";
/**
 * Vibe Shell — floating specialist palettes (issue #1177).
 * Each palette is a desktop window (window.WindowManager) whose body hosts
 * the EXISTING panel root element, moved with appendChild so every id,
 * listener and render function keeps working:
 *
 *   KnowledgeGraph → #vibeGraphPanel   (+ VibeGraph.togglePanel(true))
 *   DB / Deploy    → #vibeDialogMask   (+ VibeDialogs.open('db'|'deploy'))
 *   Metrics        → #vibeMetricsPanel (+ VibeMetrics.open())
 *   Members        → #vibeMembersModal (+ VibeMembers.open())
 *
 * WindowManager.close removes DOM permanently, so on window close the
 * roots are parked into a hidden stash element and re-mounted on reopen.
 * Dragging/z-order come from the window manager itself; positions persist
 * per palette id in localStorage.
 */
(function () {
    "use strict";

    var S = window.VibeShell;
    var PALETTES = [
        { id: "vibe-palette-graph", root: "vibeGraphPanel", title: "Knowledge Graph", icon: "⛓️", kind: "panel", activate: function () { if (window.VibeGraph) window.VibeGraph.togglePanel(true); } },
        { id: "vibe-palette-db", root: "vibeDialogMask", title: "Database", icon: "🗄️", kind: "dialog", dialog: ["db", "Database Schema"], activate: function () { if (window.VibeDialogs) window.VibeDialogs.open("db", "Database Schema"); } },
        { id: "vibe-palette-deploy", root: "vibeDialogMask", title: "Deploy", icon: "🚀", kind: "dialog", dialog: ["deploy", "Deploy"], activate: function () { if (window.VibeDialogs) window.VibeDialogs.open("deploy", "Deploy"); } },
        { id: "vibe-palette-metrics", root: "vibeMetricsPanel", title: "Metrics", icon: "📊", kind: "panel", activate: function () { if (window.VibeMetrics) window.VibeMetrics.open(); } },
        { id: "vibe-palette-members", root: "vibeMembersModal", title: "Members", icon: "👥", kind: "modal", activate: function () { if (window.VibeMembers) window.VibeMembers.open(); } },
    ];

    function wm() {
        return typeof window.WindowManager !== "undefined" ? window.WindowManager : null;
    }

    function stash() {
        var host = document.getElementById("vibeShellStash");
        if (!host) {
            host = document.createElement("div");
            host.id = "vibeShellStash";
            host.style.display = "none";
            document.body.appendChild(host);
        }
        return host;
    }

    /* The db and deploy palettes share the single #vibeDialogMask root; only
       one of them can be mounted at a time. */
    function findDef(id) {
        for (var i = 0; i < PALETTES.length; i++) {
            if (PALETTES[i].id === id) return PALETTES[i];
        }
        return null;
    }

    function isMounted(def) {
        var root = document.getElementById(def.root);
        return !!(root && root.classList.contains("vibe-palette-mounted"));
    }

    function loadPosition(id) {
        try {
            var raw = localStorage.getItem(S.PALETTE_POS_PREFIX + id);
            if (!raw) return null;
            var pos = JSON.parse(raw);
            if (typeof pos.left === "number" && typeof pos.top === "number") return pos;
        } catch (ignore) { }
        return null;
    }

    function savePositions() {
        PALETTES.forEach(function (def) {
            var win = document.getElementById("window-" + def.id);
            if (!win || !win.parentNode) return;
            var left = parseInt(win.style.left || "0", 10);
            var top = parseInt(win.style.top || "0", 10);
            try {
                localStorage.setItem(S.PALETTE_POS_PREFIX + def.id, JSON.stringify({ left: left, top: top }));
            } catch (ignore) { }
        });
    }

    function applyPosition(def) {
        var win = document.getElementById("window-" + def.id);
        if (!win) return;
        var pos = loadPosition(def.id);
        if (pos) {
            win.style.left = Math.max(0, pos.left) + "px";
            win.style.top = Math.max(0, pos.top) + "px";
        }
        if (wm() && typeof wm().focus === "function") wm().focus(def.id);
    }

    function mount(def) {
        if (def.kind === "dialog") {
            /* Shared mask: if another dialog palette owns it, reuse its
               window and simply switch the dialog content. */
            var other = document.getElementById("window-vibe-palette-db") ||
                document.getElementById("window-vibe-palette-deploy");
            if (other && isMounted(findDef(other.id.replace("window-", "")))) {
                def.activate();
                if (wm()) wm().focus(other.id.replace("window-", ""));
                return;
            }
        }
        var mgr = wm();
        if (!mgr || typeof mgr.open !== "function") {
            def.activate();
            return;
        }
        mgr.open(def.id, def.icon + " " + def.title, "");
        var body = document.getElementById("window-body-" + def.id);
        var root = document.getElementById(def.root);
        if (!body || !root) {
            if (mgr.close) mgr.close(def.id);
            def.activate();
            return;
        }
        body.classList.add("vibe-shell-palette-body");
        root.classList.add("vibe-palette-mounted");
        body.appendChild(root);
        applyPosition(def);
        def.activate();
    }

    function open(id) {
        var def = findDef(id);
        if (!def) return;
        if (isMounted(def)) {
            if (wm()) wm().focus(def.id);
            def.activate();
            return;
        }
        mount(def);
    }

    function deactivate(def) {
        try {
            if (def.root === "vibeMetricsPanel" && window.VibeMetrics) window.VibeMetrics.close();
            else if (def.root === "vibeGraphPanel" && window.VibeGraph) window.VibeGraph.togglePanel(false);
            else if (def.root === "vibeMembersModal" && window.VibeMembers) window.VibeMembers.close();
            else if (def.kind === "dialog" && window.VibeDialogs) window.VibeDialogs.close();
        } catch (ignore) { }
    }

    /* Park palette roots back into the stash before the WM destroys them. */
    function handleWindowClosed(winId) {
        PALETTES.forEach(function (def) {
            if (def.id !== winId) return;
            var root = document.getElementById(def.root);
            if (root && root.classList.contains("vibe-palette-mounted")) {
                deactivate(def);
                root.classList.remove("vibe-palette-mounted");
                stash().appendChild(root);
            }
        });
        savePositions();
    }

    function buildButtons() {
        var host = document.getElementById("vibeShellPaletteButtons");
        if (!host || host.childElementCount) return;
        PALETTES.forEach(function (def) {
            var btn = document.createElement("button");
            btn.type = "button";
            btn.className = "vibe-shell-tb-btn vibe-shell-tb-palette";
            btn.dataset.paletteId = def.id;
            btn.innerHTML = '<span class="vibe-shell-tb-icon">' + def.icon + "</span>" +
                '<span class="vibe-shell-tb-label">' + def.title + "</span>";
            btn.addEventListener("click", function () { open(def.id); });
            host.appendChild(btn);
        });
    }

    document.addEventListener("mouseup", savePositions);

    window.VibeShell.palettes = {
        init: function () { buildButtons(); },
        open: open,
        handleWindowClosed: handleWindowClosed,
        defs: PALETTES,
    };
})();
