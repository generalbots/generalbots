"use strict";
/**
 * Vibe Shell — bootstrap (#1177; classic mode removed per #1189).
 * Loads after 00..40 and activates the toolbar shell unconditionally:
 * command bar, floating specialist palettes, external-git controls and the
 * canvas task-card flow. No mode resolution remains.
 */
(function () {
    "use strict";

    var S = window.VibeShell;

    function wm() {
        return typeof window.WindowManager !== "undefined" ? window.WindowManager : null;
    }

    /* WindowManager.close removes the window DOM outright; palettes need to
       park their roots first. Wrap close once and re-emit as an event
       instead of altering WM behavior elsewhere. */
    function wrapClose() {
        var mgr = wm();
        if (!mgr || mgr.__vibeShellCloseWrapped) return;
        var original = mgr.close.bind(mgr);
        mgr.close = function (id) {
            original(id);
            document.dispatchEvent(new CustomEvent("gb-shell-window-closed", {
                detail: { id: id },
            }));
        };
        mgr.__vibeShellCloseWrapped = true;
    }

    function purgeStaleStash() {
        /* A freshly injected partial recreates every palette root; parked
           roots from a previous generation are garbage by now. */
        var stashHost = document.getElementById("vibeShellStash");
        if (stashHost) stashHost.innerHTML = "";
    }

    function boot() {
        /* Runs once per partial injection: a reopened Vibe window brings
           fresh DOM, so the shell always rebuilds for it. */
        purgeStaleStash();
        S.setMode("toolbar");
        wrapClose();
        S.toolbar.build();
        /* The single command row is the whole surface: its buttons open
           every accessory (terminal, browser, chat, run dock, canvas,
           graph, metrics, dialogs) as floating windows on demand. The
           shell's duplicate palette buttons, the auto-rendered phases
           canvas and the toolbar git controls are intentionally NOT
           started — committing happens in the git dialog popup. */
    }

    document.addEventListener("gb-shell-window-closed", function (e) {
        if (e.detail && e.detail.id && S.palettes &&
            typeof S.palettes.handleWindowClosed === "function") {
            S.palettes.handleWindowClosed(e.detail.id);
        }
    });

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", boot);
    } else {
        boot();
    }
})();
