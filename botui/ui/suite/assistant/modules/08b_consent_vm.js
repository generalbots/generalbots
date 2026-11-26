"use strict";
/**
 * 08b_consent_vm.js - Card for an agent action in the Virtual Machine.
 *
 * The server raises a blocking request (detail.vm): the tool call waits for
 * the decision, so no continuation message is sent. detail: { agent, kind,
 * text, cwd?, language?, packages?, background?, repair?, from?, folder?,
 * provider?, outside_vm, protected, offers[] }. The card sits in the answer
 * that waits for it (08_consent.js places it). Keyboard: focus moves to the
 * card when it appears, unless the person is typing (then it is announced
 * and Alt+P reaches it); Esc never denies.
 */

(function () {
    var A = window.GBAssistant;

    var DECISIONS = [
        ["allow_once", "assistant.vm-allow-once", "Permitir uma vez", "gb-consent-primary"],
        ["allow_conversation", "assistant.vm-allow-conversation", "Permitir nesta conversa", ""],
        ["allow_all", "assistant.vm-allow-all", "Permitir tudo", ""],
        ["deny", "assistant.vm-deny", "Negar", "gb-consent-deny"]
    ];
    var SHIELD = '<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>';

    function esc(s) {
        return String(s == null ? "" : s)
            .replace(/&/g, "&amp;").replace(/</g, "&lt;")
            .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
    }

    /* A subscription agent by its display name; "chat" is the assistant. */
    function agentMeta(agent) {
        return agent && agent !== "chat" && A._agentMeta ? A._agentMeta(agent) : null;
    }

    function agentName(agent) {
        var meta = agentMeta(agent);
        return meta ? meta.name : A.t("assistant.vm-agent-chat", "Assistente");
    }
    A.vmAgentName = agentName;

    /* The title names who asks: "O assistente", "O Claude Code". */
    function subject(agent) {
        var meta = agentMeta(agent);
        if (!meta) { return A.t("assistant.vm-agent-chat-subject", "O assistente"); }
        return A.t("assistant.vm-agent-subject", "O {name}").replace("{name}", meta.name);
    }

    function fact(key, fallback, value) {
        if (value == null || value === "" || (Array.isArray(value) && !value.length)) { return ""; }
        var text = Array.isArray(value) ? value.join(", ") : String(value);
        return '<div class="gb-consent-meta"><span>' + esc(A.t(key, fallback)) + '</span> ' + esc(text) + '</div>';
    }

    function title(d) {
        var t = A.t("assistant.vm-wants-" + d.kind, A.t("assistant.vm-wants-command", "{agent} quer executar um comando na Máquina Virtual"));
        return t.replace("{agent}", subject(d.agent)).replace("{provider}", d.provider || "");
    }

    /* Where the action runs or lands. */
    function where(d) {
        if (d.kind === "drive_save") {
            return fact("assistant.vm-dest", "Destino:", A.t("assistant.vm-where-drive", "seu Drive") + (d.folder ? " · " + d.folder : ""));
        }
        if (d.kind === "external_save") {
            return fact("assistant.vm-dest", "Destino:", A.t("assistant.vm-where-external", "drive externo ({provider})").replace("{provider}", d.provider || ""));
        }
        return fact("assistant.vm-where", "Onde:", A.t("assistant.vm-where-vm", "Máquina Virtual") + (d.cwd ? " · " + d.cwd : ""));
    }

    function note(d, offers) {
        if (d.protected) {
            return '<p class="gb-consent-note gb-consent-warn">' + esc(A.t("assistant.vm-protected", "Este pedido envolve as credenciais do agente: só pode ser permitido uma vez.")) + '</p>';
        }
        if (offers.indexOf("allow_all") !== -1) {
            return '<p class="gb-consent-note">' + esc(A.t("assistant.vm-all-note", "Permitir tudo: todas as ações deste agente nesta conversa, só dentro da Máquina Virtual.")) + '</p>';
        }
        if (d.outside_vm) {
            return '<p class="gb-consent-note">' + esc(A.t("assistant.vm-outside-note", "Sai da Máquina Virtual: “Permitir tudo” não vale para esta ação.")) + '</p>';
        }
        return "";
    }

    /* Builds the card; `resolve(card, requestId, decision)` posts the choice. */
    A.buildVmConsentCard = function (req, resolve, thread) {
        var d = req.detail || {};
        var offers = Array.isArray(d.offers) && d.offers.length ? d.offers : ["allow_once", "deny"];
        var card = document.createElement("div");
        var titleId = "gbConsentTitle-" + String(req.request_id).replace(/[^A-Za-z0-9_-]/g, "");
        card.className = "gb-consent-card gb-consent-vm" + (d.outside_vm ? " gb-consent-out" : "");
        card.dataset.requestId = req.request_id;
        card.dataset.threadId = req.thread_id || thread;
        card.dataset.blocking = "1";
        card.dataset.agent = d.agent || "chat";
        card.setAttribute("role", "group");
        card.setAttribute("aria-labelledby", titleId);
        card.tabIndex = -1;
        var buttons = DECISIONS.filter(function (x) { return offers.indexOf(x[0]) !== -1; }).map(function (x) {
            return '<button type="button" class="gb-consent-btn ' + x[3] + '" data-decision="' + x[0] + '">' + esc(A.t(x[1], x[2])) + '</button>';
        }).join("");
        card.innerHTML =
            '<div class="gb-consent-head">' +
                '<span class="gb-consent-icon">' + SHIELD + '</span>' +
                '<div class="gb-consent-headings">' +
                    '<div class="gb-consent-kicker">' + esc(A.t("assistant.vm-kicker", "Precisa da sua permissão")) + '</div>' +
                    '<div class="gb-consent-title" id="' + titleId + '">' + esc(title(d)) + '</div>' +
                    (d.repair ? '<div class="gb-consent-subtitle">' + esc(A.t("assistant.vm-repair", "Versão corrigida do programa")) + '</div>' : '') +
                '</div>' +
            '</div>' +
            '<div class="gb-consent-body">' +
                '<pre class="gb-consent-code" tabindex="0" aria-label="' + esc(title(d)) + '">' + esc(d.text || "") + '</pre>' +
                where(d) +
                fact("assistant.vm-language", "Linguagem:", d.language) +
                fact("assistant.vm-packages", "Instala da internet:", d.packages) +
                fact("assistant.vm-from", "Origem:", d.from) +
                (d.background ? '<div class="gb-consent-meta">' + esc(A.t("assistant.vm-background", "Roda em segundo plano")) + '</div>' : '') +
            '</div>' +
            '<div class="gb-consent-actions">' + buttons + '</div>' +
            note(d, offers);
        card.addEventListener("click", function (e) {
            var btn = e.target.closest(".gb-consent-btn");
            if (btn && !btn.disabled) { resolve(card, req.request_id, btn.getAttribute("data-decision")); }
        });
        card.addEventListener("keydown", function (e) {
            if (e.key !== "Escape" || card.classList.contains("gb-consent-resolved")) { return; }
            e.preventDefault();
            e.stopPropagation();
            focusComposer();
            A.announce(A.t("assistant.vm-still-open", "O pedido de permissão continua aberto."));
        });
        return card;
    };

    A.vmOutcomeText = function (decision) {
        if (decision === "allow_all") { return A.t("assistant.vm-allowed-all", "Permissão total nesta conversa"); }
        if (decision === "allow_conversation") { return A.t("assistant.vm-allowed-conversation", "Permitido nesta conversa"); }
        if (decision === "allow_once") { return A.t("assistant.vm-allowed-once", "Permitido uma vez"); }
        if (decision === "deny") { return A.t("assistant.vm-denied", "Negado por você: nada foi executado"); }
        return null;
    };

    function focusComposer() {
        var input = document.querySelector("#gbComposerDock textarea");
        if (input && input.offsetParent !== null) { input.focus(); }
    }

    /* ---- Focus and announcements ---- */
    /* Typing: text waiting in a field, or a key pressed in the last moment.
       Sending a message (Enter) is not typing. */
    var lastTyping = 0;
    document.addEventListener("input", function (e) {
        var tag = e.target && e.target.tagName;
        if (tag === "TEXTAREA" || tag === "INPUT") { lastTyping = Date.now(); }
    }, true);
    document.addEventListener("keydown", function (e) {
        if (e.key === "Enter" && !e.shiftKey && e.target && e.target.tagName === "TEXTAREA") { lastTyping = 0; }
    }, true);

    function typing() {
        var el = document.activeElement;
        var busy = el && (el.tagName === "TEXTAREA" || el.tagName === "INPUT") && String(el.value || "").trim() !== "";
        return busy || Date.now() - lastTyping < 2000;
    }

    /* The buttons come into view: the card's last line, nearest edge. */
    function reveal(card) {
        var last = card.lastElementChild || card;
        last.scrollIntoView({ block: "nearest" });
    }

    A.onConsentCardShown = function (card) {
        if (card.dataset.shown === "1" || card.dataset.blocking !== "1") { return; }
        card.dataset.shown = "1";
        var label = (card.querySelector(".gb-consent-title") || {}).textContent || "";
        if (typing()) {
            A.announce(A.t("assistant.vm-shown-typing", "{title}. Pressione Alt+P para responder.").replace("{title}", label));
            return;
        }
        card.focus({ preventScroll: true });
        reveal(card);
        A.announce(label);
    };

    /* Alt+P reaches the open card; Esc while a card waits does not stop the
       turn (stopping it would refuse the action without a word). */
    var escapeNow = false;
    window.addEventListener("keydown", function (e) {
        escapeNow = e.key === "Escape";
        if (escapeNow) { setTimeout(function () { escapeNow = false; }, 0); }
        if (!e.altKey || e.ctrlKey || e.metaKey || e.code !== "KeyP") { return; }
        var card = A.openConsentCard && A.openConsentCard();
        if (!card) { return; }
        e.preventDefault();
        reveal(card);
        var first = card.querySelector(".gb-consent-btn:not(:disabled)");
        (first || card).focus();
    }, true);

    var baseCancel = A.cancelGeneration;
    A.cancelGeneration = function () {
        var card = escapeNow && A.openConsentCard && A.openConsentCard();
        if (card) {
            card.focus({ preventScroll: true });
            reveal(card);
            A.announce(A.t("assistant.vm-esc-open", "Há um pedido de permissão aberto. Escolha uma opção; Negar não executa nada."));
            return;
        }
        if (baseCancel) { return baseCancel.apply(this, arguments); }
    };

    A.onVmConsentDecision = function (decision, hadFocus) {
        if (hadFocus) { focusComposer(); }
        if (A.onVmGrantChanged) { A.onVmGrantChanged(decision); }
    };
})();
