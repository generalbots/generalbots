"use strict";
/**
 * 04b_status.js - Status line rendering for the sidebar bottom.
 * Split from 04_sidebar.js to stay under the 450-line limit.
 */

(function () {
    var A = window.GBAssistant;

    function escText(s) {
        var d = document.createElement("span"); d.textContent = s; return d.innerHTML;
    }

    /* ---- Status line ---- */
    A.renderStatus = function () {
        var textEl = document.getElementById("gbStatusText");
        if (!textEl) { return; }

        /* Find the display name of the current model */
        var modelName = A.defaultModel || A.selectedModel || "";
        /* Shorten long model IDs: after the last slash or colon */
        var short = modelName;
        var slashIdx = short.lastIndexOf("/");
        if (slashIdx >= 0) { short = short.substring(slashIdx + 1); }
        var colonIdx = short.indexOf(":");
        if (colonIdx > 0) { short = short.substring(0, colonIdx); }
        if (!short) { short = modelName; }

        var dest = A.destination;
        if (!dest || !dest.scope || dest.scope === "unknown") {
            textEl.textContent = short;
            textEl.className = "gb-status-text";
            return;
        }

        var scope = dest.scope;
        var host = dest.host || "";
        var label = "";
        var isExternal = false;

        if (scope === "machine") {
            label = short + " · " + A.t("assistant.on-this-computer", "on this computer");
        } else if (scope === "network") {
            label = short + " · " + A.t("assistant.on-your-network", "on your network") +
                (host ? " (" + host + ")" : "");
        } else if (scope === "external") {
            isExternal = true;
            label = short + " · " + A.t("assistant.external", "external") +
                (host ? ": " + host : "");
        } else {
            label = short;
        }

        if (isExternal) {
            textEl.innerHTML =
                '<svg class="gb-status-external-icon" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>' +
                '<span class="gb-status-external">' + escText(label) + '</span>';
        } else {
            textEl.textContent = label;
            textEl.className = "gb-status-text";
        }
    };
})();
