"use strict";
/**
 * Vibe Shell — shared state (#1177, classic mode removed per #1189).
 * Toolbar is the only presentation: slim command bar, shared desktop apps
 * (Terminal/Browser/Chat), floating specialist palettes and a canvas
 * task-card flow. The body class `vibe-toolbar-mode` is applied
 * unconditionally so the embedded legacy panes never render.
 */
"use strict";

window.VibeShell = window.VibeShell || {
    PALETTE_POS_PREFIX: "gb.vibe.shell.palette.",
    mode: "toolbar",

    /* Compatibility stubs — classic mode no longer exists. */
    normalize: function () {
        return "toolbar";
    },

    setMode: function () {
        this.mode = "toolbar";
        document.body.classList.add("vibe-toolbar-mode");
        return this.mode;
    },

    isToolbar: function () {
        return true;
    },

    detectToolbar: function () {
        this.setMode("toolbar");
        return Promise.resolve(true);
    },

    projectId: function () {
        if (typeof window.currentProjectId !== "undefined" && window.currentProjectId) {
            return window.currentProjectId;
        }
        // Restore the selection across reloads: applyProjectSelection
        // persists it, but window.currentProjectId itself is lost on a
        // fresh page load, which silently disabled Run/Deploy/Terminal
        // after every reload ("Select a project first" with one selected).
        try {
            return sessionStorage.getItem("gb-vibe-project-id") || null;
        } catch (e) {
            return null;
        }
    },

    projectName: function () {
        if (typeof window.currentProject !== "undefined" && window.currentProject) {
            return String(window.currentProject);
        }
        try {
            var n = sessionStorage.getItem("gb-vibe-project-name");
            if (n) return n;
        } catch (e) {
            /* storage unavailable */
        }
        return "vibe";
    },
};
