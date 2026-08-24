"use strict";
/**
 * Vibe Shell — shared state and mode detection (issue #1177).
 * The shell has two presentation modes:
 *   classic — the stock Vibe layout (ribbon + inline dialogs). Untouched.
 *   toolbar — slim command bar, shared desktop apps, floating specialist
 *             palettes and a canvas task-card flow.
 * The mode persists in localStorage under gb.vibe.shell.mode and can be
 * forced per load with ?shell=toolbar. Everything the shell does is gated
 * behind mode !== "classic", so classic remains byte-for-byte inert.
 */
"use strict";

window.VibeShell = window.VibeShell || {
    MODE_KEY: "gb.vibe.shell.mode",
    PALETTE_POS_PREFIX: "gb.vibe.shell.palette.",
    mode: "classic",

    normalize: function (value) {
        return value === "toolbar" ? "toolbar" : "classic";
    },

    getStoredMode: function () {
        try {
            return this.normalize(localStorage.getItem(this.MODE_KEY));
        } catch (ignore) {
            return "classic";
        }
    },

    setMode: function (mode, persist) {
        this.mode = this.normalize(mode);
        if (persist) {
            try {
                localStorage.setItem(this.MODE_KEY, this.mode);
            } catch (ignore) {
                /* Storage disabled — in-memory selection stays valid. */
            }
        }
        document.body.classList.toggle("vibe-toolbar-mode", this.mode === "toolbar");
        return this.mode;
    },

    isToolbar: function () {
        return this.mode === "toolbar";
    },

    /**
     * Resolution order:
     *   1. ?shell=toolbar / ?shell=classic query override (also persisted);
     *   2. GET /api/settings/ui flag (ui.vibe_toolbar_mode === true);
     *   3. stored preference;
     *   4. default classic (inert).
     */
    detectToolbar: function () {
        var self = this;
        var qs = new URLSearchParams(window.location.search);
        var forced = qs.get("shell");
        if (forced === "toolbar" || forced === "classic") {
            self.setMode(forced, true);
            return Promise.resolve(self.mode === "toolbar");
        }
        var stored = self.getStoredMode();
        var timeout = new Promise(function (resolve) {
            setTimeout(function () { resolve(null); }, 2500);
        });
        var probe = fetch("/api/settings/ui", { headers: { Accept: "application/json" } })
            .then(function (r) { return r.ok ? r.json() : null; })
            .then(function (data) {
                if (!data) return null;
                var ui = data.ui || data.settings || data;
                if (typeof ui.vibe_toolbar_mode === "boolean") return ui.vibe_toolbar_mode;
                return null;
            })
            .catch(function () { return null; });
        return Promise.race([probe, timeout]).then(function (flag) {
            if (flag !== null) return self.setMode(flag ? "toolbar" : "classic", false) === "toolbar";
            return self.setMode(stored, false) === "toolbar";
        });
    },

    projectId: function () {
        return typeof window.currentProjectId !== "undefined" ? window.currentProjectId : null;
    },

    projectName: function () {
        if (typeof window.currentProject !== "undefined" && window.currentProject) {
            return String(window.currentProject);
        }
        return "vibe";
    },
};
