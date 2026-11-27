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

    /** Match a destination host to a known account or provider catalogue name. */
    function providerDisplayName(host) {
        if (!host) { return ""; }
        var h = host.toLowerCase();
        var accounts = A.accounts || [];
        for (var ai = 0; ai < accounts.length; ai++) {
            var ah = (accounts[ai].host || "").toLowerCase();
            if (ah && h.indexOf(ah) >= 0) { return accounts[ai].name || accounts[ai].id || ""; }
        }
        var catalogue = A._modelProviders ? A._modelProviders.list : [];
        for (var ci = 0; ci < catalogue.length; ci++) {
            try {
                var pu = new URL(catalogue[ci].url);
                if (h === pu.hostname || h.indexOf(pu.hostname) >= 0) { return catalogue[ci].name; }
            } catch (e) { /* skip */ }
        }
        return "";
    }

    /* ---- Status line ---- */
    A.renderStatus = function () {
        var textEl = document.getElementById("gbStatusText");
        if (!textEl) { return; }

        var active = A.selectedModel || A.defaultModel || "";
        var short = typeof A.shortModelName === "function" ? A.shortModelName(active) : active;
        if (!short) { short = active; }

        var isExternal = false;
        var label = "";

        if (/^sub:/.test(active)) {
            isExternal = true;
            var provider = active.slice(4).split("/")[0];
            var modelId = active.indexOf("/") > 4 ? active.slice(active.indexOf("/", 4) + 1) : "";
            var meta = A._agentMeta ? A._agentMeta(provider) : null;
            var agentName = meta ? meta.name : provider;
            if (modelId && modelId !== "default") {
                /* sub:<p>/<model>: "Agent name · model label" */
                var sub = (A.subscriptions || []).filter(function (x) { return x.provider === provider; })[0];
                var mObj = sub && (sub.models || []).filter(function (x) { return x.id === modelId; })[0];
                var mLabel = typeof A.agentModelLabel === "function" && mObj ? A.agentModelLabel(mObj) : modelId;
                label = agentName + " \xb7 " + mLabel;
            } else {
                /* sub:<p>: "Agent name · agente por assinatura" */
                label = agentName + " \xb7 " + A.engineName("acp", "detail").toLowerCase();
            }
        } else if (/^acct:/.test(active)) {
            isExternal = true;
            var acctMatch = /^acct:([^/]+)\//.exec(active);
            var acctId = acctMatch ? acctMatch[1] : "";
            var acctEntry = (A.accounts || []).filter(function (a) { return a.id === acctId; })[0];
            var acctName = acctEntry ? (acctEntry.name || acctId) : acctId;
            label = short + " \xb7 " + acctName;
        } else {
            var dest = A.destination;
            if (!dest || !dest.scope || dest.scope === "unknown") {
                textEl.textContent = short;
                textEl.className = "gb-status-text";
                return;
            }
            var scope = dest.scope;
            var host = dest.host || "";
            if (scope === "machine") {
                label = short + " \xb7 " + A.t("assistant.on-this-computer", "on this computer");
            } else if (scope === "network") {
                label = short + " \xb7 " + A.t("assistant.on-your-network", "on your network") +
                    (host ? " (" + host + ")" : "");
            } else if (scope === "external") {
                isExternal = true;
                var providerLabel = providerDisplayName(host);
                if (A.engineMode === "free") {
                    label = short + " \xb7 " + A.engineName("free", "group").toLowerCase();
                } else if (providerLabel) {
                    label = short + " \xb7 " + providerLabel;
                } else {
                    label = short + " \xb7 " + A.t("assistant.external", "externo") +
                        (host ? ": " + host : "");
                }
            } else {
                label = short;
            }
        }

        if (isExternal) {
            textEl.innerHTML =
                '<svg class="gb-status-external-icon" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>' +
                '<span class="gb-status-external">' + escText(label) + '</span>';
        } else {
            textEl.textContent = label;
            textEl.className = "gb-status-text";
        }

        /* Short free-model reminder near the composer when the free model is active. */
        renderFreeReminder();
    };

    function renderFreeReminder() {
        var D = A._freeDisclosure;
        var existing = document.querySelectorAll(".gb-free-bar");
        var show = D && D.isFreeActive();
        if (!show) {
            existing.forEach(function (el) { el.remove(); });
            return;
        }
        if (existing.length) { return; }
        var composers = document.querySelectorAll(".gb-composer-inner, .gb-composer-card");
        for (var ci = 0; ci < composers.length; ci++) {
            var bar = document.createElement("div");
            bar.className = "gb-free-bar";
            bar.setAttribute("role", "status");
            bar.innerHTML = D.shortHtml();
            var link = bar.querySelector("[data-open-model-settings]");
            if (link) {
                link.addEventListener("click", function (e) {
                    e.preventDefault();
                    if (typeof A.openSettings === "function") { A.openSettings("model"); }
                });
            }
            composers[ci].parentNode.insertBefore(bar, composers[ci]);
        }
    }
})();
