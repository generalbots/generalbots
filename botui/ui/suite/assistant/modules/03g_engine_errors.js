"use strict";
/**
 * 03g_engine_errors.js - Engine refusal cards shown in the chat.
 *
 * The server sends `error_code` + `error_params` on the error frame when an
 * engine cannot fulfil a request. This module renders a sentence the person
 * can act on and, where possible, a button that takes them to the right place.
 */
(function () {
    var A = window.GBAssistant;
    function esc(s) { return String(s == null ? "" : s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;"); }
    function pt() { return !!(A.locale && A.locale.startsWith("pt")); }

    function tpl(s, params) {
        if (!params) { return s; }
        return s.replace(/\{(\w+)\}/g, function (m, k) { return params[k] !== undefined ? params[k] : m; });
    }

    /* ---- Error definitions ---- */
    var ERRORS = {
        agent_not_linked: {
            pt: "Sua conta do {label} não está conectada. Conecte em Configurações › Modelo › Agente por assinatura.",
            en: "Your {label} account is not connected. Connect it in Settings › Model › Subscription agent.",
            action: "settings_acp", refresh: true
        },
        agent_switch_off: {
            pt: "O {label} é um agente e precisa do interruptor Agente ligado nesta conversa.",
            en: "{label} is an agent and needs the Agent switch on in this conversation.",
            action: "agent_on"
        },
        agent_unavailable: {
            pt: "Os agentes por assinatura não estão disponíveis neste computador. Escolha outro modelo no menu.",
            en: "Subscription agents are not available on this computer. Choose another model in the menu.",
            action: "model_menu"
        },
        agent_model_unavailable: {
            pt: "O modelo {model} não está disponível no {label}. Escolha outro no menu de modelos ou ligue-o em Configurações › Modelo › Agente por assinatura.",
            en: "The model {model} is not available on {label}. Choose another in the model menu or switch it on in Settings › Model › Subscription agent.",
            action: "settings_acp", refresh: true
        },
        agent_workspace_failed: {
            pt: "A Máquina Virtual do agente não pôde ser iniciada. Veja o estado em Máquina Virtual e tente de novo.",
            en: "The agent's Virtual Machine could not be started. Check its state in Virtual Machine and try again.",
            action: "machine_view"
        },
        account_model_off: {
            pt: "O modelo {model} não está ligado para {label}. Ligue-o em Configurações › Modelo › Servidor ou chave própria ou escolha outro modelo.",
            en: "The model {model} is not switched on for {label}. Switch it on in Settings › Model › Own server or key, or choose another model.",
            action: "settings_own_key", refresh: true
        },
        free_model_unavailable: {
            pt: "O modelo gratuito está indisponível no momento. Tente de novo mais tarde ou escolha outro motor em Configurações › Modelo.",
            en: "The free model is unavailable right now. Try again later or choose another engine in Settings › Model.",
            action: "settings_model"
        }
    };

    var ACTION_LABELS = {
        settings_acp:     ["Abrir Configurações", "Open Settings"],
        settings_own_key: ["Abrir Configurações", "Open Settings"],
        settings_model:   ["Abrir Configurações", "Open Settings"],
        agent_on:         ["Ligar Agente", "Turn Agent on"],
        model_menu:       ["Abrir menu de modelos", "Open model menu"],
        machine_view:     ["Ver Máquina Virtual", "View Virtual Machine"]
    };

    function doAction(action) {
        if (action === "settings_acp") {
            if (typeof A.openSettings === "function") { A.openSettings("model"); }
            setTimeout(function () {
                var btn = document.querySelector('.gb-of-mode[data-mode="acp"]');
                if (btn && !btn.disabled) { btn.click(); }
            }, 120);
        } else if (action === "settings_own_key") {
            if (typeof A.openSettings === "function") { A.openSettings("model"); }
            setTimeout(function () {
                var btn = document.querySelector('.gb-of-mode[data-mode="own_key"]');
                if (btn && !btn.disabled) { btn.click(); }
            }, 120);
        } else if (action === "agent_on") {
            if (typeof A.setAgentEnabled === "function") { A.setAgentEnabled(true); }
        } else if (action === "settings_model") {
            if (typeof A.openSettings === "function") { A.openSettings("model"); }
        } else if (action === "model_menu") {
            var picker = document.querySelector(".gb-model-btn");
            if (picker) { picker.click(); }
        } else if (action === "machine_view") {
            if (typeof A.switchView === "function") { A.switchView("machine"); }
        }
    }

    /**
     * A.renderEngineError(code, params)
     * Returns true if the code was handled; false to fall through to the
     * existing renderErrorCode path.
     */
    A.renderEngineError = function (code, params) {
        var def = ERRORS[code];
        if (!def) { return false; }

        A.showConversation();
        A.hideThinking();
        if (typeof A._finalizeStreaming === "function") { A._finalizeStreaming(); }

        var msg = tpl(pt() ? def.pt : def.en, params || {});
        var msgs = document.getElementById("gbMessages");
        var el = document.createElement("div");
        el.className = "gb-msg-assistant gb-msg-error";

        var card = document.createElement("div");
        card.className = "gb-error-card gb-engine-error-card";
        card.setAttribute("role", "alert");
        card.innerHTML =
            '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>' +
            '<span>' + esc(msg) + '</span>';
        el.appendChild(card);

        var bar = document.createElement("div");
        bar.className = "gb-error-actions";

        /* Retry */
        var retryBtn = document.createElement("button");
        retryBtn.className = "gb-btn-retry";
        retryBtn.textContent = A.t("assistant.retry", "Tentar novamente");
        retryBtn.addEventListener("click", function () {
            var all = msgs.querySelectorAll(".gb-msg-user");
            var last = all.length ? all[all.length - 1] : null;
            if (!last) { return; }
            var text = (last.dataset.originalContent || "").trim();
            if (!text) { return; }
            el.remove();
            A.lastCompletedContent = "";
            A.lastCompletedTime = 0;
            A.showThinking();
            if (!A.sendMessage(text)) {
                A.hideThinking();
                if (typeof A.renderErrorCode === "function") { A.renderErrorCode("model_unreachable"); }
            }
        });
        bar.appendChild(retryBtn);

        /* Action button */
        if (def.action) {
            var actionLabel = ACTION_LABELS[def.action];
            var actionBtn = document.createElement("button");
            actionBtn.className = "gb-btn-settings";
            actionBtn.textContent = actionLabel ? (pt() ? actionLabel[0] : actionLabel[1]) : "";
            actionBtn.addEventListener("click", function () {
                doAction(def.action);
            });
            bar.appendChild(actionBtn);
        }
        el.appendChild(bar);

        msgs.appendChild(el);
        if (!A.isUserScrolledUp()) { A.scrollToBottom(true); }

        /* Refresh model data for codes that may have made the menu stale. */
        if (def.refresh && typeof A.loadModels === "function") {
            A.loadModels().then(function () {
                if (typeof A.renderModelPickers === "function") { A.renderModelPickers(); }
            });
        }
        return true;
    };
})();
