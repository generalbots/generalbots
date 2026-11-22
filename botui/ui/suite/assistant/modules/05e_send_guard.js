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

    A.sendMessage = function (text, files, messageType) {
        var model = A.selectedModel || A.defaultModel || "";
        if (isLocal(model) || !A.externalTarget || !A.confirmExternal) {
            return realSend.call(A, text, files, messageType);
        }
        var list = accepted();
        /* Fast path: resolve synchronously when possible. externalTarget
           returns a Promise, but for sub: values it resolves immediately. */
        var sent = false;
        A.externalTarget(model).then(function (target) {
            if (!target || !target.host || list.indexOf(target.host) >= 0) {
                sent = realSend.call(A, text, files, messageType);
                if (!sent) { A.hideThinking(); }
                return;
            }
            /* Show confirm card; on accept, remember and send. */
            A.hideThinking();
            var anchor = document.querySelector(".gb-composer-inner") ||
                document.querySelector(".gb-composer-card");
            A.confirmExternal(target, anchor, function () {
                A.showThinking();
                sent = realSend.call(A, text, files, messageType);
                if (!sent) { A.hideThinking(); }
            });
        });
        /* The caller (doSend in 05_composer.js) needs a truthy return to
           keep the thinking indicator. If the target resolves asynchronously
           (acct: fetches the account host), return true optimistically; the
           confirm callback will show/hide thinking as needed. */
        return true;
    };
})();
