"use strict";
/**
 * 08d_consent_vm_wait.js - While a Virtual Machine card waits in the open
 * conversation.
 *
 * - No "Pensando…" row for the turn that waits: its run already says
 *   "Aguardando sua permissão".
 * - A new message starts a new turn and the server then withdraws the open
 *   request, so sending asks first, in the shell's confirm style above the
 *   composer. "Voltar ao pedido" keeps the text and moves focus to the card.
 */
(function () {
    var A = window.GBAssistant;

    function waitingCard() {
        var card = A.openConsentCard && A.openConsentCard();
        return card && card.dataset.blocking === "1" ? card : null;
    }

    /* The card belongs to the current turn: no message of the person after it. */
    function waitingInTurn() {
        var card = waitingCard();
        if (!card) { return null; }
        var users = document.querySelectorAll("#gbMessages .gb-msg-user");
        var last = users.length ? users[users.length - 1] : null;
        return !last || (last.compareDocumentPosition(card) & Node.DOCUMENT_POSITION_FOLLOWING) ? card : null;
    }

    var baseShow = A.showThinking;
    A.showThinking = function () {
        if (waitingInTurn()) { A.hideThinking(); return; }
        return baseShow.apply(this, arguments);
    };

    var baseShown = A.onConsentCardShown;
    A.onConsentCardShown = function (card) {
        if (card.dataset.blocking === "1") { A.hideThinking(); }
        if (baseShown) { return baseShown.apply(this, arguments); }
    };

    /* ---- Sending while the card waits ---- */
    function visibleComposer() {
        var all = document.querySelectorAll(".gb-composer-inner, .gb-composer-card");
        for (var i = 0; i < all.length; i++) {
            if (all[i].offsetParent !== null) { return all[i]; }
        }
        return all[0] || null;
    }

    function focusComposer() {
        var composer = visibleComposer();
        var input = composer && composer.querySelector("textarea");
        if (input) { input.focus(); }
    }

    function toCard(card) {
        if (!card.isConnected || card.classList.contains("gb-consent-resolved")) { focusComposer(); return; }
        (card.lastElementChild || card).scrollIntoView({ block: "nearest" });
        var first = card.querySelector(".gb-consent-btn:not(:disabled)");
        (first || card).focus();
    }

    function closeConfirm() {
        var open = document.querySelector(".gb-confirm-vm-send");
        if (!open) { return; }
        var hadFocus = open.contains(document.activeElement);
        open.remove();
        if (hadFocus) { focusComposer(); }
    }

    function confirmSend(card, onSend) {
        document.querySelectorAll(".gb-confirm").forEach(function (c) { c.remove(); });
        var box = document.createElement("div");
        box.className = "gb-confirm gb-confirm-vm-send";
        box.setAttribute("role", "alertdialog");
        box.setAttribute("aria-labelledby", "gbVmSendHead");
        box.setAttribute("aria-describedby", "gbVmSendText");
        box.innerHTML =
            '<div class="gb-confirm-head" id="gbVmSendHead">' +
                '<svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">' +
                '<path d="M12 3l9 16H3z"/><path d="M12 10v4M12 17h.01"/></svg><span></span></div>' +
            '<p class="gb-confirm-text" id="gbVmSendText"></p>' +
            '<div class="gb-confirm-actions">' +
                '<button type="button" class="gb-block-btn" data-act="back"></button>' +
                '<button type="button" class="gb-block-btn gb-block-btn--acc" data-act="send"></button>' +
            '</div>';
        box.querySelector(".gb-confirm-head span").textContent = A.t("assistant.vm-send-head", "Confirmação");
        box.querySelector(".gb-confirm-text").textContent = A.t("assistant.vm-send-confirm",
            "Enviar agora cancela o pedido de permissão aberto. Enviar mesmo assim?");
        var back = box.querySelector('[data-act="back"]');
        var go = box.querySelector('[data-act="send"]');
        back.textContent = A.t("assistant.vm-send-back", "Voltar ao pedido");
        go.textContent = A.t("assistant.vm-send-go", "Enviar");
        back.addEventListener("click", function () { box.remove(); toCard(card); });
        go.addEventListener("click", function () { box.remove(); focusComposer(); onSend(); });
        box.addEventListener("keydown", function (e) {
            if (e.key !== "Escape") { return; }
            e.preventDefault();
            e.stopPropagation();
            box.remove();
            focusComposer();
        });
        var composer = visibleComposer();
        if (composer && composer.parentNode) { composer.parentNode.insertBefore(box, composer); }
        else { document.body.appendChild(box); }
        back.focus();
    }

    var baseGuard = A.preSendGuard;
    A.preSendGuard = function (onApproved) {
        var card = waitingCard();
        var next = function () {
            if (baseGuard) { baseGuard(onApproved); } else { onApproved(); }
        };
        if (!card) { next(); return; }
        confirmSend(card, next);
    };

    /* The card was answered or withdrawn meanwhile: the question is moot. */
    var basePending = A.onConsentPendingChanged;
    A.onConsentPendingChanged = function () {
        if (basePending) { basePending.apply(this, arguments); }
        if (!waitingCard()) { closeConfirm(); }
    };
})();
