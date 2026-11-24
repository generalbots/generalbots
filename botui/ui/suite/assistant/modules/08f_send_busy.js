"use strict";
/**
 * 08f_send_busy.js - Sending while the assistant is still answering.
 *
 * A message sent during a turn would be lost, so the composer used to drop
 * Enter with only a screen-reader notice. Now the person always sees why
 * and chooses, in the shell's confirm style above the composer:
 * - a permission card waits ("Salvar no Drive", a Virtual Machine request):
 *   sending cancels the turn, the server withdraws the open request, then
 *   the message goes;
 * - the answer is still being written: wait, or stop it and send.
 * At the end of a turn with cards open, the pending list is read again so a
 * request the server withdrew does not stay marked in the sidebar.
 */
(function () {
    var A = window.GBAssistant;
    function t(pt, en) { return A.locale && A.locale.indexOf("pt") === 0 ? pt : en; }

    function focusComposer() {
        var composer = A.visibleComposer && A.visibleComposer();
        var input = composer && composer.querySelector("textarea");
        if (input) { input.focus(); }
    }

    function stopTurn() {
        if (typeof A.cancelGeneration === "function") { A.cancelGeneration(); }
        if (A.isStreaming && typeof A._finalizeStreaming === "function") { A._finalizeStreaming(); }
        A.isStreaming = false;
    }

    function ask(opts) {
        document.querySelectorAll(".gb-confirm").forEach(function (c) { c.remove(); });
        var box = document.createElement("div");
        box.className = "gb-confirm gb-confirm-busy-send";
        box.setAttribute("role", "alertdialog");
        box.setAttribute("aria-labelledby", "gbBusySendHead");
        box.setAttribute("aria-describedby", "gbBusySendText");
        box.innerHTML =
            '<div class="gb-confirm-head" id="gbBusySendHead">' +
                '<svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">' +
                '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/></svg><span></span></div>' +
            '<p class="gb-confirm-text" id="gbBusySendText"></p>' +
            '<div class="gb-confirm-actions">' +
                '<button type="button" class="gb-block-btn" data-act="back"></button>' +
                '<button type="button" class="gb-block-btn gb-block-btn--acc" data-act="send"></button>' +
            '</div>';
        box.querySelector(".gb-confirm-head span").textContent = opts.head;
        box.querySelector(".gb-confirm-text").textContent = opts.text;
        var back = box.querySelector('[data-act="back"]');
        var go = box.querySelector('[data-act="send"]');
        back.textContent = opts.back;
        go.textContent = opts.go;
        function done() { box.remove(); clearInterval(watch); }
        back.addEventListener("click", function () { done(); if (opts.onBack) { opts.onBack(); } else { focusComposer(); } });
        go.addEventListener("click", function () { done(); focusComposer(); opts.onGo(); });
        box.addEventListener("keydown", function (e) {
            if (e.key !== "Escape") { return; }
            e.preventDefault();
            e.stopPropagation();
            done();
            focusComposer();
        });
        /* The answer finished meanwhile: nothing to choose any more. */
        var watch = setInterval(function () {
            if (!box.isConnected) { clearInterval(watch); return; }
            if (!A.isStreaming && !opts.card) { done(); }
        }, 500);
        var composer = A.visibleComposer && A.visibleComposer();
        if (composer && composer.parentNode) { composer.parentNode.insertBefore(box, composer); }
        else { document.body.appendChild(box); }
        back.focus();
        A.announce(opts.text);
    }

    function toCard(card) {
        if (!card.isConnected || card.classList.contains("gb-consent-resolved")) { focusComposer(); return; }
        (card.lastElementChild || card).scrollIntoView({ block: "nearest" });
        var first = card.querySelector(".gb-consent-btn:not(:disabled)");
        (first || card).focus();
    }

    /* Called by doSend (05_composer.js) instead of dropping the message. */
    A.onSendWhileBusy = function (send) {
        var card = A.openConsentCard && A.openConsentCard();
        if (card) {
            ask({
                card: card,
                head: A.t("assistant.vm-send-head", "Confirmação"),
                text: A.t("assistant.vm-send-confirm", "Enviar agora cancela o pedido de permissão aberto. Enviar mesmo assim?"),
                back: A.t("assistant.vm-send-back", "Voltar ao pedido"),
                go: A.t("assistant.vm-send-go", "Enviar"),
                onBack: function () { toCard(card); },
                onGo: function () { stopTurn(); send(); }
            });
            return;
        }
        ask({
            head: t("A resposta ainda está sendo escrita", "The answer is still being written"),
            text: t("Aguarde a resposta terminar, ou pare a resposta para enviar esta mensagem agora.",
                "Wait for the answer to finish, or stop it to send this message now."),
            back: t("Continuar esperando", "Keep waiting"),
            go: t("Parar e enviar", "Stop and send"),
            onGo: function () { stopTurn(); send(); }
        });
    };

    var baseEnd = A.endStream;
    A.endStream = function () {
        var r = baseEnd ? baseEnd.apply(this, arguments) : undefined;
        var open = A.pendingConsentThreads ? Object.keys(A.pendingConsentThreads()) : [];
        if (open.length && A.reviveConsentCards) { A.reviveConsentCards(); }
        return r;
    };
})();
