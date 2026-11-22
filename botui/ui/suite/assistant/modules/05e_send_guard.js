"use strict";
/**
 * 05e_send_guard.js - Consent check before sending to an external destination.
 *
 * When the person picks an external model, 05d_engine_guard.js shows a
 * confirm card. But a selection restored from localStorage (or the default
 * engine being external) with no stored consent skips that check. This
 * module wraps A.sendMessage so the same confirm appears before the first
 * message leaves the computer when the destination was never accepted.
 */
(function () {
    var A = window.GBAssistant;
    var KEY = "gb-external-ok";

    function accepted() {
        try { return JSON.parse(localStorage.getItem(KEY) || "[]") || []; } catch (e) { return []; }
    }

    function isLocal(model) {
        return !/^(sub|acct):/.test(model || "") &&
            A.engineMode !== "free" &&
            ((A.destination || {}).scope === "machine" || (A.destination || {}).scope === "network");
    }

    var realSend = A.sendMessage;
    var guardCleared = false;

    /**
     * Pre-flight consent check called by the composer BEFORE it clears the
     * input and renders the user bubble. If consent is already stored or
     * the destination is local, onApproved fires synchronously. Otherwise
     * the confirm card appears and onApproved fires only on accept; on
     * decline nothing happens and the composer keeps the person's text
     * and attachments intact.
     */
    A.preSendGuard = function (onApproved) {
        var model = A.selectedModel || A.defaultModel || "";
        if (isLocal(model) || !A.externalTarget || !A.confirmExternal) {
            guardCleared = true;
            onApproved();
            return;
        }
        var list = accepted();
        A.externalTarget(model).then(function (target) {
            if (!target || !target.host || list.indexOf(target.host) >= 0) {
                guardCleared = true;
                onApproved();
                return;
            }
            var anchor = document.querySelector(".gb-composer-inner") ||
                document.querySelector(".gb-composer-card");
            A.confirmExternal(target, anchor, function () {
                guardCleared = true;
                onApproved();
            });
        });
    };

    A.sendMessage = function (text, files, messageType) {
        /* When preSendGuard already ran and approved, skip the duplicate
           check and forward directly to the real send. */
        if (guardCleared) {
            guardCleared = false;
            return realSend.call(A, text, files, messageType);
        }
        var model = A.selectedModel || A.defaultModel || "";
        if (isLocal(model) || !A.externalTarget || !A.confirmExternal) {
            return realSend.call(A, text, files, messageType);
        }
        var list = accepted();
        var sent = false;
        A.externalTarget(model).then(function (target) {
            if (!target || !target.host || list.indexOf(target.host) >= 0) {
                sent = realSend.call(A, text, files, messageType);
                if (!sent) { A.hideThinking(); }
                return;
            }
            A.hideThinking();
            var anchor = document.querySelector(".gb-composer-inner") ||
                document.querySelector(".gb-composer-card");
            A.confirmExternal(target, anchor, function () {
                A.showThinking();
                sent = realSend.call(A, text, files, messageType);
                if (!sent) { A.hideThinking(); }
            });
        });
        return true;
    };
})();
