"use strict";
/**
 * 09d_settings_shell.js - Settings section for switching shells.
 *
 * Assistant stays the product default. "Windowed suite" is a per-browser
 * preview that opens the classic suite window manager with Chat in a window.
 * The choice is stored locally and honoured by the desktop hand-off and by
 * the login redirect, so it survives a reload without changing the product
 * file for everyone.
 */
(function () {
    var A = window.GBAssistant;
    if (!A || typeof A.registerSettingsSection !== "function") { return; }

    function esc(value) {
        return String(value == null ? "" : value)
            .replace(/&/g, "&amp;").replace(/</g, "&lt;")
            .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
    }

    function openWindowedSuite() {
        try { localStorage.setItem("gb-shell-override", "desktop"); }
        catch (e) { /* storage disabled: the URL parameter still applies */ }
        window.location.replace("/suite/desktop.html?app=chat&shell=desktop");
    }

    A.registerSettingsSection({
        id: "shell",
        icon: "info",
        key: "settings.preview.windowed",
        render: function (panel) {
            var html = '<div class="gb-set-card">' +
                '<h2 class="gb-set-title">' +
                    esc(A.t("settings.preview.windowed-title", "Windowed suite (Preview)")) +
                '</h2>' +
                '<p class="gb-set-hint">' +
                    esc(A.t("settings.preview.windowed-desc",
                        "Open the classic suite window manager with Chat in a window. The assistant stays your default.")) +
                '</p>' +
                '<button type="button" id="gbSetOpenWindowed" class="gb-set-btn-sm gb-set-btn-accent">' +
                    esc(A.t("settings.preview.windowed-open", "Open windowed suite")) +
                '</button>' +
            '</div>';
            panel.innerHTML = html;
            var button = document.getElementById("gbSetOpenWindowed");
            if (button) { button.addEventListener("click", openWindowedSuite); }
        }
    });
})();
