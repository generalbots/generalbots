"use strict";
/**
 * 05b_model_picker.js - Model picker button and dropdown menu.
 *
 * Extracted from 05_composer.js for file-size compliance (< 450 lines).
 */

(function () {
    var A = window.GBAssistant;

    function esc(s) { return String(s == null ? "" : s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;"); }

    /* ---- Model name helpers ---- */
    A.shortModelName = function (model) {
        if (!model) { return ""; }
        var s = model;
        var slash = s.lastIndexOf("/");
        if (slash >= 0) { s = s.substring(slash + 1); }
        var bslash = s.lastIndexOf("\\");
        if (bslash >= 0) { s = s.substring(bslash + 1); }
        if (s.toLowerCase().endsWith(".gguf")) { s = s.substring(0, s.length - 5); }
        s = s.replace(/[-_](Q\d[_.]\w{1,4})$/i, "");
        var colon = s.indexOf(":");
        if (colon > 0) { s = s.substring(0, colon); }
        return s;
    };

    A.fullModelName = function (model) {
        if (!model) { return ""; }
        var s = model;
        var slash = s.lastIndexOf("/");
        if (slash >= 0) { s = s.substring(slash + 1); }
        var bslash = s.lastIndexOf("\\");
        if (bslash >= 0) { s = s.substring(bslash + 1); }
        return s;
    };

    var shortModelName = A.shortModelName;

    /* ---- Render / update ---- */
    A.renderModelPickers = function () {
        var pickers = document.querySelectorAll(".gb-model-picker");
        for (var p = 0; p < pickers.length; p++) { replaceWithButton(pickers[p]); }
        var btns = document.querySelectorAll(".gb-model-btn");
        for (var b = 0; b < btns.length; b++) { updateBtnLabel(btns[b]); }
    };

    function updateBtnLabel(btn) {
        var label = btn.querySelector(".gb-model-btn-label");
        if (label) {
            var raw = A.selectedModel || A.defaultModel || "";
            label.textContent = shortModelName(raw);
            btn.title = A.fullModelName(raw);
        }
    }

    function replaceWithButton(select) {
        if (select.dataset.replaced) { return; }
        select.dataset.replaced = "1";
        var btn = document.createElement("button");
        btn.className = "gb-model-btn";
        btn.type = "button";
        var short = shortModelName(A.selectedModel || A.defaultModel || "");
        btn.innerHTML = '<span class="gb-model-btn-label">' + (short || "Model") + '</span>' +
            '<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="6 9 12 15 18 9"/></svg>';
        btn.addEventListener("click", function (e) { e.stopPropagation(); toggleModelMenu(btn); });
        select.style.display = "none";
        select.parentNode.insertBefore(btn, select.nextSibling);
    }

    /* ---- Dropdown ---- */
    var activeModelMenu = null;

    function toggleModelMenu(anchor) {
        if (activeModelMenu) { activeModelMenu.remove(); activeModelMenu = null; return; }
        var menu = document.createElement("div");
        menu.className = "gb-model-menu";
        activeModelMenu = menu;

        var dest = A.destination || {};
        var scopeLabel = "";
        if (dest.scope === "machine") { scopeLabel = A.t("assistant.on-this-computer", "on this computer"); }
        else if (dest.scope === "network") { scopeLabel = A.t("assistant.on-your-network", "on your network") + (dest.host ? " (" + dest.host + ")" : ""); }
        else if (dest.scope === "external") { scopeLabel = A.t("assistant.external", "external") + (dest.host ? ": " + dest.host : ""); }

        var currentShort = shortModelName(A.defaultModel || "");
        var isSelected = !A.selectedModel || A.selectedModel === A.defaultModel;
        var item0 = document.createElement("button");
        item0.className = "gb-model-menu-item" + (isSelected ? " active" : "");
        item0.innerHTML = '<span class="gb-model-menu-name">' + esc(currentShort) + '</span>' +
            (scopeLabel ? '<span class="gb-model-menu-meta">' + esc(scopeLabel) + '</span>' : '');
        item0.addEventListener("click", function () { selectModel(A.defaultModel); });
        menu.appendChild(item0);

        var distinctModels = [];
        var seen = {};
        if (A.defaultModel) { seen[A.defaultModel] = true; }
        for (var i = 0; i < A.tiers.length; i++) {
            var m = A.tiers[i].model;
            if (!seen[m] && !A.tiers[i].inherits_default) { seen[m] = true; distinctModels.push(A.tiers[i]); }
        }
        for (var j = 0; j < distinctModels.length; j++) {
            var tier = distinctModels[j];
            var isAct = A.selectedModel === tier.model;
            var item = document.createElement("button");
            item.className = "gb-model-menu-item" + (isAct ? " active" : "");
            item.innerHTML = '<span class="gb-model-menu-name">' + esc(shortModelName(tier.model)) + '</span>';
            (function (model) { item.addEventListener("click", function () { selectModel(model); }); })(tier.model);
            menu.appendChild(item);
        }

        var div = document.createElement("div");
        div.className = "gb-model-menu-divider";
        menu.appendChild(div);
        var action = document.createElement("button");
        action.className = "gb-model-menu-action";
        action.innerHTML = '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.6 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09"/></svg>' +
            '<span>' + esc(A.t("settings.model-switch", "Trocar modelo...")) + '</span>';
        action.addEventListener("click", function () {
            closeModelMenu();
            A.switchView("settings");
            setTimeout(function () {
                var tab = document.querySelector('.gb-set-tab[data-section="model"]');
                if (tab) { tab.click(); }
            }, 80);
        });
        menu.appendChild(action);

        anchor.style.position = "relative";
        anchor.appendChild(menu);
        document.addEventListener("click", closeModelMenu, { once: true });
    }

    function closeModelMenu() {
        if (activeModelMenu) { activeModelMenu.remove(); activeModelMenu = null; }
    }

    function selectModel(model) {
        A.selectedModel = model;
        try { localStorage.setItem("gb-model-preference", model); } catch (e) { /* */ }
        A.renderModelPickers();
        A.renderStatus();
        closeModelMenu();
    }
})();
