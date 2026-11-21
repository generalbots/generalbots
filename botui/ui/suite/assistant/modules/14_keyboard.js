"use strict";
/**
 * 14_keyboard.js - Global keyboard shortcuts for the assistant shell.
 *
 * Ctrl+Shift+O / Ctrl+N   New conversation
 * Ctrl+K                   Focus search
 * Ctrl+B                   Toggle sidebar
 * Ctrl+,                   Open Settings
 * Esc                      Close topmost layer / stop generation
 * Up arrow (empty input)   Edit last message
 */
(function () {
    var A = window.GBAssistant;

    function isMod(e) {
        return e.ctrlKey || e.metaKey;
    }

    /* Open layers, topmost last: windows, menus, panels and drawers. */
    var LAYERS = 'dialog[open], [role="dialog"], [role="alertdialog"], [role="menu"], ' +
        ".gb-model-menu, .gb-user-menu, .gb-drv-ctx, .gb-attach-panel, .gb-lightbox, .gb-quota-drawer";
    function topLayer() {
        var open = Array.prototype.filter.call(document.querySelectorAll(LAYERS), function (el) {
            return !el.hidden && el.getClientRects().length > 0;
        });
        return open[open.length - 1] || null;
    }

    /* Esc closes the topmost layer and does nothing else. An open <dialog>
       and the user menu are closed here; every other layer closes itself
       through its own Esc handler, so the event is left untouched for it. */
    function closeTopLayer(e) {
        var layer = topLayer();
        if (!layer) return false;
        if (layer.tagName === "DIALOG") { e.preventDefault(); layer.close(); }
        else if (layer.classList.contains("gb-user-menu")) { e.preventDefault(); document.dispatchEvent(new MouseEvent("click")); }
        return true;
    }

    document.addEventListener("keydown", function (e) {
        /* Ignore when typing in an input/textarea (unless it is Esc). */
        var tag = (e.target.tagName || "").toLowerCase();
        var isInput = tag === "input" || tag === "textarea" || e.target.isContentEditable;

        /* Ctrl+Shift+O or Ctrl+N — new conversation */
        if (isMod(e) && e.shiftKey && e.key.toUpperCase() === "O") {
            e.preventDefault();
            var btn = document.getElementById("gbNewChat");
            if (btn) { btn.click(); }
            return;
        }
        if (isMod(e) && !e.shiftKey && e.key.toLowerCase() === "n") {
            /* Only intercept when no input is focused (Ctrl+N can mean
               new line in some text areas). */
            if (!isInput) {
                e.preventDefault();
                var newBtn = document.getElementById("gbNewChat");
                if (newBtn) { newBtn.click(); }
            }
            return;
        }

        /* Ctrl+K — focus search */
        if (isMod(e) && !e.shiftKey && e.key.toLowerCase() === "k") {
            e.preventDefault();
            var search = document.getElementById("gbSearchInput");
            if (search) {
                /* Expand sidebar if collapsed */
                var sidebar = document.getElementById("gb-sidebar");
                if (sidebar && sidebar.classList.contains("collapsed")) {
                    var toggle = document.getElementById("gbSidebarToggle");
                    if (toggle) { toggle.click(); }
                }
                search.focus();
                search.select();
            }
            return;
        }

        /* Ctrl+B — toggle sidebar (works even when an input is focused;
           the browser's "bold" shortcut is not meaningful here). */
        if (isMod(e) && !e.shiftKey && e.key.toLowerCase() === "b") {
            e.preventDefault();
            var sideToggle = document.getElementById("gbSidebarToggle");
            if (sideToggle) { sideToggle.click(); }
            return;
        }

        /* Ctrl+, — open Settings */
        if (isMod(e) && e.key === ",") {
            e.preventDefault();
            if (A.currentView === "settings") {
                A.switchView("chat");
            } else {
                A.switchView("settings");
            }
            return;
        }

        /* Esc — layered close:
           1. Close the topmost open window, menu, panel or drawer
           2. Stop generation (if streaming)
           3. Close Drive / Settings view */
        if (e.key === "Escape") {
            if (e.defaultPrevented || closeTopLayer(e)) { return; }
            /* Then stopping generation */
            if (A.isStreaming) {
                e.preventDefault();
                A.cancelGeneration();
                return;
            }
            /* Attachment panel (13_attachpanel.js manages its own Esc
               handler; we do not duplicate it here.) */
            /* Drive or Settings: return to chat */
            if (A.currentView === "drive" || A.currentView === "settings") {
                e.preventDefault();
                A.switchView("chat");
                return;
            }
            /* The sidebar's own Esc handler (04_sidebar.js) handles the
               narrow slide-over. No action needed here. */
            return;
        }

        /* Up arrow in an empty composer — edit last user message */
        if (e.key === "ArrowUp" && !e.shiftKey && !isMod(e) && isInput) {
            var input = e.target;
            var val = (input.value || "").trim();
            if (val === "" && typeof A.editLastMessage === "function") {
                e.preventDefault();
                A.editLastMessage();
            }
        }
    });

    /* After send, after closing a panel and after "Nova conversa",
       focus the composer. This is handled by composing code
       (05_composer.js, 04_sidebar.js) but we add an explicit
       "focus composer" utility other modules can call. */
    A.focusComposer = function () {
        var inp = document.getElementById("gbInputDock") ||
                  document.getElementById("gbInputEmpty");
        if (inp) { inp.focus(); }
    };
})();
