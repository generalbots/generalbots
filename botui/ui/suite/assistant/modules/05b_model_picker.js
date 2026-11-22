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
        /* A linked subscription agent is "sub:<provider>". */
        if (/^sub:/.test(String(model))) {
            var id = String(model).slice(4);
            var hit = (A.linkedAgents || []).filter(function (x) { return x.id === id; })[0];
            return hit ? hit.name : id;
        }
        /* An account model is "acct:<provider>/<model>": show just the model. */
        var s = String(model).replace(/^acct:[^/]+\//, "");
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
        if (/^sub:/.test(String(model))) { return A.shortModelName(model); }
        var s = String(model).replace(/^acct:[^/]+\//, "");
        var slash = s.lastIndexOf("/");
        if (slash >= 0) { s = s.substring(slash + 1); }
        var bslash = s.lastIndexOf("\\");
        if (bslash >= 0) { s = s.substring(bslash + 1); }
        return s;
    };

    var shortModelName = A.shortModelName;

    /* ---- Web search switch (Settings > Modelo) ---- */
    A.webSearchAvailable = function () {
        return !!(A.productConfig && A.productConfig.web_search_available);
    };
    A.webSearchOn = function () {
        if (!A.webSearchAvailable()) { return false; }
        try { return localStorage.getItem("gb-web-search") === "1"; } catch (e) { return false; }
    };
    A.setWebSearch = function (on) {
        try { localStorage.setItem("gb-web-search", on ? "1" : "0"); } catch (e) { /* storage unavailable */ }
    };

    /* ---- Render / update ---- */
    A.renderModelPickers = function () {
        if (A.loadLinkedAgents && !A._agentsAsked) {
            A._agentsAsked = true;
            A.loadLinkedAgents().then(function () { A.renderModelPickers(); });
        }
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

        addAgentGroup(menu);
        addAccountGroups(menu);
        addSwitchEntries(menu, dest);

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

    /* ---- Switch entries: the models installed on this computer and the
       free hosted model. Picking one does the same switch as Settings >
       Modelo; the menu never changes the model silently. ---- */
    function tr(ptT, enT) { return A.locale && A.locale.startsWith("pt") ? ptT : enT; }

    function menuEntry(menu, name, meta, onPick) {
        var item = document.createElement("button");
        item.className = "gb-model-menu-item";
        item.innerHTML = '<span class="gb-model-menu-name">' + esc(name) + '</span>' +
            '<span class="gb-model-menu-meta">' + esc(meta) + '</span>';
        item.addEventListener("click", function (e) { e.stopPropagation(); onPick(); });
        var divider = menu.querySelector(".gb-model-menu-divider");
        if (divider) { menu.insertBefore(item, divider); } else { menu.appendChild(item); }
    }

    function addSwitchEntries(menu, dest) {
        if (dest.scope === "machine" || dest.scope === "network") {
            menuEntry(menu, tr("Modelo gratuito", "Free model"),
                tr("hospedado fora deste computador", "hosted outside this computer"),
                function () { switchTo({ mode: "free" }); });
        }
        A.apiFetch("/api/llm/local/models").then(function (r) { return r.ok ? r.json() : null; }).then(function (data) {
            if (!data || activeModelMenu !== menu) { return; }
            var current = A.fullModelName(A.defaultModel || "");
            (data.tiers || []).forEach(function (tier) {
                (tier.models || []).forEach(function (m) {
                    if (!m.downloaded || m.file === current) { return; }
                    menuEntry(menu, m.name, tr("neste computador", "on this computer"),
                        function () { switchTo({ local: m.id }); });
                });
            });
        }).catch(function () { /* the menu keeps what it has */ });
    }

    /* ---- Connected subscription agents (Settings > AI accounts): picking
       one answers through its own agent VM; the bot's default is untouched. ---- */
    function addAgentGroup(menu) {
        var agents = A.linkedAgents || [];
        if (!agents.length) { return; }
        addGroup(menu, { id: "agents", name: A.t("chat-model-agents", "Agents"), logo: "" },
            agents.map(function (x) { return { model: x.name, value: "sub:" + x.id }; }),
            function (value) {
                if (typeof A.setAgentEnabled === "function") { A.setAgentEnabled(true); }
                selectModel(value);
            });
    }

    /* ---- Provider accounts: one collapsible group per key, listing the
       models switched on in Settings > Modelo. The group holding the chosen
       model starts open. Accounts are loaded by 10i_accounts.js. ---- */
    function addAccountGroups(menu) {
        (A.accounts || []).forEach(function (acc) {
            if ((acc.models || []).length) { addGroup(menu, acc, acc.models, selectModel); }
        });
    }

    function addGroup(menu, acc, models, onPick) {
        var open = models.some(function (m) { return m.value === A.selectedModel; });
        var group = document.createElement("div");
        group.className = "gb-model-group" + (open ? " open" : "");
        var head = document.createElement("button");
        head.type = "button";
        head.className = "gb-model-group-head";
        head.setAttribute("aria-expanded", String(open));
        var svg = (A._providerLogos || {})[acc.logo || acc.id] || "";
        head.innerHTML = (svg ? '<span class="gb-model-group-logo">' + svg + '</span>' : '') +
            '<span class="gb-model-group-name">' + esc(acc.name || acc.id) + '</span>' +
            '<span class="gb-model-group-count">' + models.length + '</span>' +
            '<svg class="gb-model-group-caret" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="6 9 12 15 18 9"/></svg>';
        head.addEventListener("click", function (e) {
            e.stopPropagation();
            var isOpen = group.classList.toggle("open");
            head.setAttribute("aria-expanded", String(isOpen));
        });
        var list = document.createElement("div");
        list.className = "gb-model-group-list";
        models.forEach(function (m) {
            var item = document.createElement("button");
            item.className = "gb-model-menu-item" + (A.selectedModel === m.value ? " active" : "");
            item.innerHTML = '<span class="gb-model-menu-name">' + esc(m.model) + '</span>';
            item.addEventListener("click", function () { onPick(m.value); });
            list.appendChild(item);
        });
        group.appendChild(head);
        group.appendChild(list);
        var divider = menu.querySelector(".gb-model-menu-divider");
        if (divider) { menu.insertBefore(group, divider); } else { menu.appendChild(group); }
    }

    function setPickersBusy(on) {
        var btns = document.querySelectorAll(".gb-model-btn");
        for (var i = 0; i < btns.length; i++) {
            btns[i].disabled = on;
            var label = btns[i].querySelector(".gb-model-btn-label");
            if (label && on) { label.textContent = tr("Trocando o modelo...", "Switching the model..."); }
        }
    }

    function switchTo(target) {
        closeModelMenu();
        var shared = A._modelShared || {};
        function run() {
            setPickersBusy(true);
            var req = target.local
                ? A.apiFetch("/api/llm/local/activate", { method: "POST", body: JSON.stringify({ model_id: target.local }) })
                : A.apiFetch("/api/llm/config", { method: "PUT", body: JSON.stringify({ mode: target.mode }) });
            req.then(function (r) {
                return r.json().catch(function () { return { error: "http_" + r.status }; })
                    .then(function (d) { return r.ok && !d.error ? "" : (d.error || "http_" + r.status); });
            }).catch(function () { return "unreachable"; }).then(function (failure) {
                return A.loadModels().then(function () {
                    setPickersBusy(false);
                    A.renderModelPickers();
                    if (typeof A.renderStatus === "function") { A.renderStatus(); }
                    if (failure && shared.showFailure) { shared.showFailure(failure); }
                });
            });
        }
        if (target.mode === "free" && shared.inlineConfirm) {
            shared.inlineConfirm(shared.t("confirmExt").replace("%h", shared.t("freeHost")), run);
        } else {
            run();
        }
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
