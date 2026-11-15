"use strict";
/**
 * 06_drive.js - Drive and Settings integration.
 *
 * Loads the existing Drive app (HTMX partial + module loader) into the
 * main area, visually calmed by the assistant shell's identity.
 * Similarly loads the Settings partial.
 */

(function () {
    var A = window.GBAssistant;
    var driveLoaded = false;
    var settingsLoaded = false;

    /* ------------------------------------------------------------------ */
    /*  View switching                                                     */
    /* ------------------------------------------------------------------ */

    A.switchView = function (view) {
        A.currentView = view;

        var chatView = document.getElementById("gbChatView");
        var driveView = document.getElementById("gbDriveView");
        var settingsView = document.getElementById("gbSettingsView");

        if (chatView) {
            chatView.classList.toggle("gb-view-active", view === "chat");
        }
        if (driveView) {
            driveView.classList.toggle("gb-view-active", view === "drive");
        }
        if (settingsView) {
            settingsView.classList.toggle("gb-view-active", view === "settings");
        }

        /* Update nav item highlights */
        var navItems = document.querySelectorAll(".gb-nav-item[data-view]");
        for (var i = 0; i < navItems.length; i++) {
            var v = navItems[i].getAttribute("data-view");
            navItems[i].classList.toggle("active", v === view);
        }

        /* Lazy-load Drive */
        if (view === "drive" && !driveLoaded) {
            loadDrive();
        }

        /* Lazy-load Settings */
        if (view === "settings" && !settingsLoaded) {
            loadSettings();
        }
    };

    /* ------------------------------------------------------------------ */
    /*  Load Drive                                                         */
    /* ------------------------------------------------------------------ */

    function loadDrive() {
        var container = document.getElementById("gbDriveContainer");
        if (!container) { return; }

        /* Fetch the Drive HTML partial */
        A.apiFetch("/suite/drive/drive.html")
            .then(function (r) {
                if (!r.ok) { throw new Error("Drive HTML " + r.status); }
                return r.text();
            })
            .then(function (html) {
                container.innerHTML = html;
                /* Load the Drive CSS */
                if (!document.querySelector('link[href*="drive/drive.css"]')) {
                    var link = document.createElement("link");
                    link.rel = "stylesheet";
                    link.href = "/suite/drive/drive.css";
                    document.head.appendChild(link);
                }
                /* Load the Drive module loader */
                var script = document.createElement("script");
                script.src = "/suite/drive/drive-module-loader.js?_=" + Date.now();
                document.body.appendChild(script);
                driveLoaded = true;
            })
            .catch(function (e) {
                container.innerHTML =
                    '<p style="padding:24px;color:var(--id-text);">' +
                    A.t("assistant.drive-error", "Could not load Drive.") +
                    "</p>";
                console.error("[assistant] Drive load failed:", e);
            });
    }

    /* ------------------------------------------------------------------ */
    /*  Load Settings                                                      */
    /* ------------------------------------------------------------------ */

    function loadSettings() {
        var container = document.getElementById("gbSettingsContainer");
        if (!container) { return; }

        A.apiFetch("/suite/settings/index.html")
            .then(function (r) {
                if (!r.ok) { throw new Error("Settings HTML " + r.status); }
                return r.text();
            })
            .then(function (html) {
                container.innerHTML = html;
                settingsLoaded = true;
            })
            .catch(function (e) {
                container.innerHTML =
                    '<p style="padding:24px;color:var(--id-text);">' +
                    A.t("assistant.settings-error", "Could not load Settings.") +
                    "</p>";
                console.error("[assistant] Settings load failed:", e);
            });
    }

    /* ------------------------------------------------------------------ */
    /*  Bind navigation buttons                                            */
    /* ------------------------------------------------------------------ */

    A.initNavigation = function () {
        var navItems = document.querySelectorAll(".gb-nav-item[data-view]");
        for (var i = 0; i < navItems.length; i++) {
            (function (item) {
                item.addEventListener("click", function () {
                    var view = item.getAttribute("data-view");
                    A.switchView(view);
                });
            })(navItems[i]);
        }

        /* Clicking the wordmark / brand goes back to chat */
        var brand = document.getElementById("gbWordmark");
        if (brand) {
            brand.style.cursor = "pointer";
            brand.addEventListener("click", function () {
                A.switchView("chat");
            });
        }
    };

    /* ------------------------------------------------------------------ */
    /*  Hide features not compiled                                         */
    /* ------------------------------------------------------------------ */

    A.filterFeatures = function () {
        var apps = A.apps || [];
        var hasDrive = apps.indexOf("drive") !== -1;
        var driveNav = document.getElementById("gbNavDrive");
        if (driveNav && !hasDrive) {
            driveNav.style.display = "none";
        }
    };
})();
