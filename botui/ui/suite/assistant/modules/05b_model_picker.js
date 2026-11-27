"use strict";
/**
 * 05b_model_picker.js - Model picker button and dropdown menu.
 *
 * Extracted from 05_composer.js for file-size compliance (< 450 lines).
 */

(function () {
    var A = window.GBAssistant;
    /* The CLI's own default model reads "Padrão do agente"; other labels are model names. */
    A.agentModelLabel = function (m) {
        if (m && m.id === "default") { return (A.locale || "").startsWith("pt") ? "Padrão do agente" : "Agent default"; }
        return m ? (m.label || m.id) : "";
    };

    function esc(s) { return String(s == null ? "" : s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;"); }

    /* ---- Model name helpers ---- */
    A.shortModelName = function (model) {
        if (!model) { return ""; }
        /* A connected agent: "sub:<provider>"; an account's model: "acct:<provider>/<model>" */
        if (/^sub:/.test(model)) { return A.subscriptionLabel(model); }
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

    /* "sub:<provider>/<model>" names the agent's model; "sub:<provider>" its own default. */
    A.subscriptionLabel = function (value) {
        var rest = String(value).slice(4), cut = rest.indexOf("/");
        var provider = cut >= 0 ? rest.slice(0, cut) : rest, modelId = cut >= 0 ? rest.slice(cut + 1) : "";
        var sub = (A.subscriptions || []).filter(function (x) { return x.provider === provider; })[0];
        if (modelId) {
            var m = sub && (sub.models || []).filter(function (x) { return x.id === modelId; })[0];
            return m ? (A.agentModelLabel(m)) : modelId;
        }
        if (sub) { return sub.label || provider; }
        var hit = (A.linkedAgents || []).filter(function (x) { return x.id === provider; })[0];
        return hit ? hit.name : provider;
    };

    A.fullModelName = function (model) {
        if (!model) { return ""; }
        var s = String(model).replace(/^acct:[^/]+\//, "");
        var slash = s.lastIndexOf("/");
        if (slash >= 0) { s = s.substring(slash + 1); }
        var bslash = s.lastIndexOf("\\");
        if (bslash >= 0) { s = s.substring(bslash + 1); }
        return s;
    };

    var shortModelName = A.shortModelName;

    /* ---- Render / update ---- */
    A.renderModelPickers = function () {
        closeModelMenu();
        if (A.loadLinkedAgents && !A._agentsAsked) {
            A._agentsAsked = true;
            A.loadLinkedAgents().then(function () { A.renderModelPickers(); });
        }
        var pickers = document.querySelectorAll(".gb-model-picker");
        for (var p = 0; p < pickers.length; p++) { replaceWithButton(pickers[p]); }
        var btns = document.querySelectorAll(".gb-model-btn");
        for (var b = 0; b < btns.length; b++) { updateBtnLabel(btns[b]); }
        if (A.syncEngineUsage) { A.syncEngineUsage(); }
    };

    function updateBtnLabel(btn) {
        var label = btn.querySelector(".gb-model-btn-label");
        if (label) {
            var raw = A.selectedModel || A.defaultModel || "";
            label.textContent = shortModelName(raw) || tr("Selecionar motor", "Select an engine");
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
        btn.setAttribute("aria-expanded", "false");
        btn.innerHTML = '<span class="gb-model-btn-label">' + esc(short || A.t("settings.model", "Model")) + '</span>' +
            '<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="6 9 12 15 18 9"/></svg>';
        btn.addEventListener("click", function (e) { e.stopPropagation(); toggleModelMenu(btn); });
        select.style.display = "none";
        select.parentNode.insertBefore(btn, select.nextSibling);
    }

    /* ---- Dropdown ---- */
    var activeModelMenu = null;

    function toggleModelMenu(anchor) {
        if (activeModelMenu) {
            var wasActive = activeModelMenu.previousElementSibling === anchor;
            closeModelMenu();
            if (wasActive) { return; }
        }
        var menu = document.createElement("div");
        menu.className = "gb-model-menu";
        anchor.setAttribute("aria-expanded", "true");
        activeModelMenu = menu;
        var heading = document.createElement("div");
        heading.className = "gb-engine-label gb-model-menu-title";
        heading.textContent = tr("Motores", "Engines");
        menu.appendChild(heading);

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
        item0.className += " gb-model-menu-current";
        item0.addEventListener("click", function () { selectModel(A.defaultModel); });
        if (currentShort) { menu.appendChild(item0); }

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
        /* Agents linked since the page opened appear without a reload. */
        if (A.loadLinkedAgents) {
            A.loadLinkedAgents().then(function () {
                if (activeModelMenu === menu && !menu.querySelector('[data-group="agents"]')) { addAgentGroup(menu); }
            });
        }
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

        var wrap = anchor.parentElement;
        wrap.classList.add("gb-model-anchor");
        anchor.after(menu);
        /* Keyboard-navigation hook (05f_model_menu_keys.js). Must fire
           after the menu is in the DOM so focus() works. */
        if (typeof A.onModelMenuOpen === "function") { A.onModelMenuOpen(menu, anchor); }
        document.addEventListener("click", closeModelMenuOutside);
        document.addEventListener("keydown", closeModelMenuOnEscape);
    }

    /* ---- Switch entries: the models installed on this computer and the
       free hosted model. Picking one does the same switch as Settings >
       Modelo; the menu never changes the model silently. ---- */
    function tr(ptT, enT) { return A.locale && A.locale.startsWith("pt") ? ptT : enT; }

    function menuEntry(menu, name, meta, onPick) {
        var item = document.createElement("button");
        item.className = "gb-model-menu-item gb-model-menu-free";
        item.innerHTML = '<span class="gb-model-menu-name">' + esc(name) + '</span>' +
            '<span class="gb-model-menu-meta">' + esc(meta) + '</span>';
        item.addEventListener("click", function (e) { e.stopPropagation(); onPick(); });
        var divider = menu.querySelector(".gb-model-menu-divider");
        if (divider) { menu.insertBefore(item, divider); } else { menu.appendChild(item); }
    }

    function addSwitchEntries(menu, dest) {
        if (dest.scope === "machine" || dest.scope === "network") {
            menuEntry(menu, A.engineName("free", "group"),
                tr("serviço Kilo (NVIDIA), nos EUA", "Kilo service (NVIDIA), in the US"),
                function () { switchTo({ mode: "free" }); });
        }
        A.apiFetch("/api/llm/local/models").then(function (r) { return r.ok ? r.json() : null; }).then(function (data) {
            if (!data || activeModelMenu !== menu) { return; }
            addLocalGroup(menu, data);
        }).catch(function () { /* the menu keeps what it has */ });
    }

    /* ---- The models installed on this computer, in one group. The one in
       use moves from the top of the menu into the group; picking another
       does the real local switch. ---- */
    var CHIP = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">' +
        '<rect x="6" y="6" width="12" height="12" rx="2"/><path d="M9 2v4M15 2v4M9 18v4M15 18v4M2 9h4M2 15h4M18 9h4M18 15h4"/></svg>';
    function addLocalGroup(menu, data) {
        var current = A.fullModelName(A.defaultModel || "");
        var inUse = !A.selectedModel || A.selectedModel === A.defaultModel;
        var items = [];
        (data.tiers || []).forEach(function (tier) {
            (tier.models || []).forEach(function (m) {
                if (!m.downloaded) { return; }
                var isCurrent = m.file === current;
                items.push({ model: m.name, value: isCurrent ? "" : m.id, active: isCurrent && inUse, current: isCurrent });
            });
        });
        if (!items.length) { return; }
        if (items.some(function (x) { return x.current; })) {
            var top = menu.querySelector(".gb-model-menu-current");
            if (top) { top.remove(); }
        }
        addGroup(menu, { id: "local", name: A.engineName("local", "group"), icon: CHIP }, items, function (id) {
            if (id) { switchTo({ local: id }); } else { selectModel(A.defaultModel); }
        });
    }

    /* ---- Connected subscription agents (Settings > Modelo > Agente). They
       run in the agent workspace, so picking one turns the Agent switch on. ---- */
    var AGENT_LOGO = { "claude-code": "anthropic", "codex-cli": "openai", "antigravity": "gemini" };
    function pickAgent(value) {
        selectModel(value);
    }
    function addAgentGroup(menu) {
        var runtimeMissing = typeof A.agentRuntimeReady === "function" && !A.agentRuntimeReady();
        var backendOff = runtimeMissing || !!(A.productConfig && A.productConfig.agent_backend_unavailable);
        /* /api/chat/models lists each linked agent with its switched-on models;
           an older server only says which agents are linked. */
        if (!Array.isArray(A.subscriptions)) {
            var agents = A.linkedAgents || [];
            if (!agents.length) { return; }
            addGroup(menu, { id: "agents", name: A.engineName("acp", "group"), logo: "" },
                agents.map(function (x) { return { model: x.name, value: "sub:" + x.id, disabled: backendOff }; }), pickAgent);
            return;
        }
        var subs = A.subscriptions.filter(function (x) { return x.linked !== false; });
        if (!subs.length) { return; }
        var chosen = /^sub:/.test(A.selectedModel || "");
        var group = addGroup(menu, { id: "agents", name: A.engineName("acp", "group") }, [], pickAgent, chosen, subs.length);
        subs.forEach(function (x) {
            var acc = { id: x.provider, name: x.label || x.provider, logo: AGENT_LOGO[x.provider] || x.provider };
            var models = (x.models || []).map(function (m) { return { model: A.agentModelLabel(m), value: "sub:" + x.provider + "/" + m.id, disabled: backendOff }; });
            if (!models.length) { models = [{ model: tr("Padrão do agente", "Agent default"), value: "sub:" + x.provider, disabled: backendOff }]; }
            addGroup(group, acc, models, pickAgent);
        });
    }

    /* ---- Provider accounts: one collapsible group per key, listing the
       models switched on in Settings > Modelo. The group holding the chosen
       model starts open. ---- */
    function addAccountGroups(menu) {
        var accounts = (A.accounts || []).filter(function (acc) { return (acc.models || []).length; });
        if (!accounts.length) { return; }
        var group = addGroup(menu, { id: "accounts", name: A.engineName("own_key", "group") }, [], selectModel,
            accounts.some(function (acc) { return acc.models.some(function (m) { return m.value === A.selectedModel; }); }),
            accounts.length);
        accounts.forEach(function (acc) { addGroup(group, acc, acc.models, selectModel); });
    }

    function addGroup(menu, acc, models, onPick, expanded, count) {
        function isActive(m) { return m.active !== undefined ? m.active : A.selectedModel === m.value; }
        var open = expanded === undefined ? models.some(isActive) : expanded;
        var group = document.createElement("div");
        group.className = "gb-model-group" + (open ? " open" : "");
        group.setAttribute("data-group", acc.id);
        var head = document.createElement("button");
        head.type = "button";
        head.className = "gb-model-group-head";
        head.setAttribute("aria-expanded", String(open));
        var svg = acc.icon || (A._providerLogos || {})[acc.logo || acc.id] || "";
        head.innerHTML = (svg ? '<span class="gb-model-group-logo">' + svg + '</span>' : '') +
            '<span class="gb-model-group-name">' + esc(acc.name || acc.id) + '</span>' +
            '<span class="gb-model-group-count">' + (count === undefined ? models.length : count) + '</span>' +
            '<svg class="gb-model-group-caret" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="6 9 12 15 18 9"/></svg>';
        head.addEventListener("click", function (e) {
            e.stopPropagation();
            var isOpen = group.classList.toggle("open");
            head.setAttribute("aria-expanded", String(isOpen));
        });
        var list = document.createElement("div");
        list.className = "gb-model-group-list";
        var runtimeOff = typeof A.agentRuntimeReady === "function" && !A.agentRuntimeReady();
        var backendReason = runtimeOff
            ? A.t("assistant.requires-agent-mode", "Requires Agent Mode")
            : (A.productConfig && A.productConfig.agent_backend_unavailable);
        models.forEach(function (m) {
            var item = document.createElement("button");
            item.className = "gb-model-menu-item" + (isActive(m) ? " active" : "");
            item.innerHTML = '<span class="gb-model-menu-name">' + esc(m.model) + '</span>';
            if (m.disabled) {
                item.disabled = true;
                var reason = backendReason || A.t("assistant.agent-backend-off", "Subscription agents unavailable on this computer");
                item.title = reason;
                item.classList.add("gb-model-menu-disabled");
                if (runtimeOff) {
                    var chip = document.createElement("span");
                    chip.className = "gb-model-menu-meta";
                    chip.textContent = tr("Requer Modo Agente", "Requires Agent Mode");
                    item.appendChild(chip);
                }
            } else {
                item.addEventListener("click", function () { onPick(m.value); });
            }
            list.appendChild(item);
        });
        if (runtimeOff && typeof A.openAgentRuntimeSettings === "function") {
            var link = document.createElement("button");
            link.className = "gb-model-menu-action";
            link.innerHTML = '<span>' + esc(A.t("assistant.requires-agent-mode", "Requires Agent Mode")) + '</span>';
            link.addEventListener("click", function () { closeModelMenu(); A.openAgentRuntimeSettings(); });
            list.appendChild(link);
        }
        group.appendChild(head);
        group.appendChild(list);
        var before = acc.id === "agents"
            ? (menu.querySelector('[data-group="accounts"]') || menu.querySelector(".gb-model-menu-free"))
            : null;
        var divider = before || menu.querySelector(".gb-model-menu-divider");
        if (divider) { menu.insertBefore(group, divider); } else { menu.appendChild(group); }
        return list;
    }

    function setPickersBusy(on) {
        var btns = document.querySelectorAll(".gb-model-btn");
        for (var i = 0; i < btns.length; i++) {
            btns[i].disabled = on;
            var label = btns[i].querySelector(".gb-model-btn-label");
            if (label && on) { label.textContent = tr("Trocando o modelo…", "Switching the model…"); }
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
        if (target.mode === "free" && A.confirmExternal) {
            A.confirmExternal({ name: tr("Modelo gratuito", "Free model"), host: shared.t ? shared.t("freeHost") : "", mode: "free" },
                visible(".gb-motor"), run);
        } else {
            run();
        }
    }

    /* The empty state and the conversation each have a composer; one shows. */
    function visible(selector) {
        return Array.from(document.querySelectorAll(selector)).filter(function (x) { return x.offsetParent; })[0] || null;
    }

    function closeModelMenu() {
        if (!activeModelMenu) { return; }
        var menu = activeModelMenu;
        var anchor = menu.previousElementSibling;
        activeModelMenu = null;
        document.removeEventListener("click", closeModelMenuOutside);
        document.removeEventListener("keydown", closeModelMenuOnEscape);
        if (anchor) { anchor.setAttribute("aria-expanded", "false"); }
        if (menu.parentNode) { menu.remove(); }
    }

    function closeModelMenuOutside(event) {
        if (activeModelMenu && !activeModelMenu.contains(event.target)) { closeModelMenu(); }
    }

    function closeModelMenuOnEscape(event) {
        if (event.key !== "Escape" || !activeModelMenu) { return; }
        var anchor = activeModelMenu.previousElementSibling;
        event.stopPropagation();
        closeModelMenu();
        if (anchor) { anchor.focus(); }
    }

    function selectModel(model) {
        var anchor = activeModelMenu ? activeModelMenu.previousElementSibling : null;
        closeModelMenu();
        A.pickEngine(model, anchor || visible(".gb-model-btn"));
    }

    /* Picks an engine for the chat from any surface (this menu, Settings >
       Modelo): the first send outside this computer asks next to `anchor`. */
    A.pickEngine = function (model, anchor, onApplied) {
        function apply() { applyModel(model); if (onApplied) { onApplied(); } }
        if (!A.externalTarget || !A.confirmExternal) { apply(); return; }
        A.externalTarget(model).then(function (target) { A.confirmExternal(target, anchor, apply); });
    };

    /* Exposed for the keyboard-navigation module (05f). */
    A.closeModelMenu = closeModelMenu;

    function applyModel(model) {
        A.selectedModel = model;
        try { localStorage.setItem("gb-model-preference", model); } catch (e) { /* */ }
        if (typeof A.syncAgentToggle === "function") { A.syncAgentToggle(); }
        A.renderModelPickers();
        A.renderStatus();
        closeModelMenu();
    }
})();
