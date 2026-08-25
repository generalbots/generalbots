"use strict";
/**
 * Vibe Shell — bootstrap (issue #1177).
 * Loads after 00..40, resolves the shell mode and activates only the
 * toolbar-mode features. In classic mode a discreet SHELL ⇄ chip is added
 * to the ribbon so the user can still switch modes; nothing else changes.
 */
(function () {
    "use strict";

    var S = window.VibeShell;

    function wm() {
        return typeof window.WindowManager !== "undefined" ? window.WindowManager : null;
    }

    /* WindowManager.close removes the window DOM outright; palettes need to
       park their roots first. Wrap close once (toolbar mode only) and
       re-emit as an event instead of altering WM behavior elsewhere. */
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
        S.detectToolbar().then(function (toolbar) {
            /* detectToolbar already applied the resolved mode + body class. */
            if (toolbar) {
                wrapClose();
                S.toolbar.build();
                S.palettes.init();
                S.git.mount();
                S.canvasFlow.start();
            } else {
                S.toolbar.registerClassicToggle();
            }
        });
    }

    document.addEventListener("gb-shell-window-closed", function (e) {
        if (e.detail && e.detail.id && S.isToolbar()) {
            S.palettes.handleWindowClosed(e.detail.id);
        }
    });

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", boot);
    } else {
        boot();
    }
})();
