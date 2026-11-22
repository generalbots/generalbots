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
    function pt() { return !!(A.locale && A.locale.startsWith("pt")); }

    function tpl(s, params) {
        if (!params) { return s; }
        return s.replace(/\{(\w+)\}/g, function (m, k) { return params[k] !== undefined ? params[k] : m; });
    }

    /* ---- A spent subscription plan (agent_limit_reached) ---- */

    /* When the plan comes back, in the person's own clock and language:
       "às 19:15", "amanhã às 19:15", "na quinta-feira às 19:15". "" when
       the server did not know it. */
    function whenBack(iso) {
        var at = iso ? new Date(iso) : null;
        if (!at || isNaN(at.getTime()) || at.getTime() <= Date.now()) { return ""; }
        var loc = pt() ? "pt-BR" : "en-US";
        var time = at.toLocaleTimeString(loc, { hour: pt() ? "2-digit" : "numeric", minute: "2-digit" });
        var now = new Date();
        var days = Math.round((new Date(at.getFullYear(), at.getMonth(), at.getDate()) -
            new Date(now.getFullYear(), now.getMonth(), now.getDate())) / 86400000);
        if (days === 0) { return (pt() ? "às " : "at ") + time; }
        if (days === 1) { return (pt() ? "amanhã às " : "tomorrow at ") + time; }
        if (days < 7) {
            var day = at.toLocaleDateString(loc, { weekday: "long" });
            if (!pt()) { return "on " + day + " at " + time; }
            return (/^(sábado|domingo)/.test(day) ? "no " : "na ") + day + " às " + time;
        }
        var date = at.toLocaleDateString(loc, { day: "numeric", month: "long" });
        return (pt() ? "em " : "on ") + date + (pt() ? " às " : " at ") + time;
    }
    A.engineLimitWhen = whenBack;

    function block(tag, text) {
        var el = document.createElement(tag);
        el.style.display = "block";
        if (text) { el.textContent = text; }
        return el;
    }

    /* The agent's own words; only https addresses become links, opened in
       the person's browser like every other external link of the shell. */
    function linkify(el, text) {
        var re = /https:\/\/[^\s<>"'()\[\]{}]+/g, last = 0, m;
        while ((m = re.exec(text))) {
            var url = m[0].replace(/[.,;:!?]+$/, "");
            if (m.index > last) { el.appendChild(document.createTextNode(text.slice(last, m.index))); }
            var a = document.createElement("a");
            a.href = url;
            a.target = "_blank";
            a.rel = "noopener noreferrer";
            a.textContent = url;
            el.appendChild(a);
            last = m.index + url.length;
        }
        if (last < text.length) { el.appendChild(document.createTextNode(text.slice(last))); }
    }

    function agentLimitBody(span, p) {
        var when = whenBack(p.reset);
        var vars = { label: p.label || "", when: when };
        if (!p.message) {
            span.textContent = tpl(pt()
                ? "Você atingiu o limite de uso do seu plano no {label}. " + (when ? "Ele volta {when}." : "Ele volta mais tarde; veja em Limites.") + " Enquanto isso, escolha outro modelo."
                : "You have reached your plan's usage limit on {label}. " + (when ? "It comes back {when}." : "It comes back later; see Limits.") + " Meanwhile, choose another model.", vars);
            return;
        }
        span.appendChild(block("strong", tpl(pt() ? "Limite do plano atingido no {label}" : "{label} plan limit reached", vars)));
        var body = block("span");
        linkify(body, String(p.message));
        span.appendChild(body);
        if (when) {
            var note = block("span", tpl(pt() ? "Volta {when} no seu horário." : "Back {when}, your time.", vars));
            note.style.opacity = "0.75";
            span.appendChild(note);
        }
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
        },
        agent_limit_reached: {
            build: agentLimitBody,
            actions: ["pick_model", "limits_view"], noRetry: true, limits: true
        }
    };

    var ACTION_LABELS = {
        settings_acp:     ["Abrir Configurações", "Open Settings"],
        settings_own_key: ["Abrir Configurações", "Open Settings"],
        settings_model:   ["Abrir Configurações", "Open Settings"],
        agent_on:         ["Ligar Agente", "Turn Agent on"],
        model_menu:       ["Abrir menu de modelos", "Open model menu"],
        machine_view:     ["Ver Máquina Virtual", "View Virtual Machine"],
        pick_model:       ["Escolher outro modelo", "Choose another model"],
        limits_view:      ["Ver limites", "See limits"]
    };

    /* The first matching control that is on screen (the composer's, not a
       hidden copy on another view). */
    function visible(selector) {
        var all = document.querySelectorAll(selector);
        for (var i = 0; i < all.length; i++) {
            if (all[i].offsetParent !== null) { return all[i]; }
        }
        return all[0] || null;
    }

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
        } else if (action === "model_menu" || action === "pick_model") {
            /* After this click has finished bubbling: the menu closes on any
               click outside it, and this one started on the card. */
            setTimeout(function () {
                var picker = visible(".gb-model-btn");
                if (picker) { picker.click(); }
            }, 0);
        } else if (action === "limits_view") {
            var drawer = document.getElementById("gbQuotaDrawer");
            var trigger = visible(".gb-usage-trigger");
            if (trigger && (!drawer || drawer.hidden)) { trigger.click(); return; }
            if (trigger) { return; }
            if (typeof A.switchView === "function") { A.switchView("settings"); }
            setTimeout(function () {
                var tab = document.querySelector('.gb-set-tab[data-section="limits"]');
                if (tab) { tab.click(); }
            }, 80);
        } else if (action === "machine_view") {
            if (typeof A.switchView === "function") { A.switchView("machine"); }
        }
    }

    /* An error frame with error_code arriving without is_complete (e.g. on
       a streaming frame) would enter the streaming path and lose its card.
       Promote it to complete so onBotMessage renders through the error path. */
    var origOnBot = A.onBotMessage;
    if (typeof origOnBot === "function") {
        A.onBotMessage = function (data) {
            if (data.error_code && !data.is_complete &&
                (data.is_error || data.status === "failed" || data.turn_status === "failed" || data.turn_status === "error")) {
                data = Object.assign({}, data, { is_complete: true });
            }
            return origOnBot(data);
        };
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

        var msgs = document.getElementById("gbMessages");
        var el = document.createElement("div");
        el.className = "gb-msg-assistant gb-msg-error";

        var card = document.createElement("div");
        card.className = "gb-error-card gb-engine-error-card";
        card.setAttribute("role", "alert");
        card.innerHTML =
            '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>';
        var textEl = document.createElement("span");
        if (def.build) { def.build(textEl, params || {}); } else { textEl.textContent = tpl(pt() ? def.pt : def.en, params || {}); }
        card.appendChild(textEl);
        el.appendChild(card);

        var bar = document.createElement("div");
        bar.className = "gb-error-actions";

        /* Retry (not offered when only the person can clear the cause) */
        if (!def.noRetry) {
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
        }

        /* Action buttons */
        (def.actions || (def.action ? [def.action] : [])).forEach(function (action) {
            var actionLabel = ACTION_LABELS[action];
            var actionBtn = document.createElement("button");
            actionBtn.className = "gb-btn-settings";
            actionBtn.textContent = actionLabel ? (pt() ? actionLabel[0] : actionLabel[1]) : "";
            actionBtn.addEventListener("click", function () {
                doAction(action);
            });
            bar.appendChild(actionBtn);
        });
        el.appendChild(bar);

        msgs.appendChild(el);
        if (!A.isUserScrolledUp()) { A.scrollToBottom(true); }

        /* Refresh model data for codes that may have made the menu stale. */
        if (def.refresh && typeof A.loadModels === "function") {
            A.loadModels().then(function () {
                if (typeof A.renderModelPickers === "function") { A.renderModelPickers(); }
            });
        }
        /* A spent plan: show the exhausted window in the engine panel. */
        if (def.limits && typeof A.llmLimits === "function") {
            A.llmLimits(true).then(function () {
                if (typeof A.syncEngineUsage === "function") { A.syncEngineUsage(); }
            });
        }
        return true;
    };
})();
