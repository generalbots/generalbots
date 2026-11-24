"use strict";
/**
 * 08e_confirm_dock.js - Confirm cards stay where the person can answer them.
 *
 * - While the card that asks about an engine picked in the menu is open,
 *   sending does not go out on the engine chosen before: the text and its
 *   attachments stay in the composer, the card says the message waits for
 *   the answer and takes the focus. "Continuar" sends the held message on
 *   the engine just chosen (through the remaining send checks); "Cancelar"
 *   or Escape keeps the text and sends nothing. The focus returns to the
 *   composer either way.
 * - When the view switches between the empty state and a conversation, each
 *   open `.gb-confirm` card moves above the composer that now shows instead
 *   of staying in the hidden one.
 *
 * Loaded after 05e and 08d so this check runs before theirs.
 */
(function () {
    var A = window.GBAssistant;

    function tr(ptT, enT) { return (A.locale || "").startsWith("pt") ? ptT : enT; }

    function focusComposer() {
        var composer = A.visibleComposer && A.visibleComposer();
        var input = composer && composer.querySelector("textarea");
        if (input) { input.focus(); }
    }

    /* Each open card goes right above the visible composer. */
    function dockConfirms() {
        var composer = A.visibleComposer && A.visibleComposer();
        if (!composer || !composer.parentNode) { return; }
        document.querySelectorAll(".gb-confirm").forEach(function (card) {
            if (card.offsetParent !== null && card.nextElementSibling === composer) { return; }
            var hadFocus = card.contains(document.activeElement) ? document.activeElement : null;
            composer.parentNode.insertBefore(card, composer);
            if (hadFocus) { hadFocus.focus(); }
        });
    }
    A.dockConfirms = dockConfirms;

    ["showConversation", "showEmptyState"].forEach(function (name) {
        var base = A[name];
        if (typeof base !== "function") { return; }
        A[name] = function () {
            var out = base.apply(this, arguments);
            dockConfirms();
            return out;
        };
    });

    function openPick() {
        var card = document.querySelector('.gb-confirm[data-kind="pick"]');
        return card && card.isConnected ? card : null;
    }

    /* The send the open card holds: the composer's own send, with the text
       and attachments it read when the person pressed Enter. */
    var held = null;

    function answered(card, go) {
        var send = go && held && held.card === card ? held.send : null;
        held = null;
        setTimeout(function () {
            focusComposer();
            if (send) { A.preSendGuard(function () { send(); focusComposer(); }); }
        }, 0);
    }

    function holdSend(card, onApproved) {
        held = { card: card, send: onApproved };
        dockConfirms();
        if (!card.querySelector(".gb-confirm-note")) {
            var note = document.createElement("p");
            note.className = "gb-confirm-note";
            note.setAttribute("role", "status");
            note.textContent = tr("Sua mensagem ainda não foi enviada: ao continuar, ela segue com o modelo escolhido.",
                "Your message has not been sent yet: if you continue, it goes with the engine you picked.");
            card.insertBefore(note, card.querySelector(".gb-confirm-actions"));
            card.querySelectorAll(".gb-confirm-actions button").forEach(function (btn) {
                btn.addEventListener("click", function () { answered(card, btn.classList.contains("gb-block-btn--acc")); });
            });
            card.addEventListener("keydown", function (e) {
                if (e.key === "Escape") { answered(card, false); }
            });
        }
        card.classList.remove("gb-confirm--ask");
        void card.offsetWidth;
        card.classList.add("gb-confirm--ask");
        (card.querySelector(".gb-block-btn--acc") || card).focus();
    }

    var baseGuard = A.preSendGuard;
    A.preSendGuard = function (onApproved) {
        var card = openPick();
        if (card) { holdSend(card, onApproved); return; }
        if (typeof baseGuard === "function") { baseGuard(onApproved); } else { onApproved(); }
    };
})();
