"use strict";
/** #2305 — cancelGeneration split from 02_api.js to stay under 450 lines. */
(function () {
    var A = window.GBAssistant;
    A.cancelGeneration = function () {
        if (!A.ws || A.ws.readyState !== 1) { return; }
        A.ws.send(JSON.stringify({
            type: "run_cancel",
            session_id: A.currentSessionId || A.sessionId || "",
        }));
        /* Optimistic cancel: restore the composer immediately.  Late frames
           from the model are suppressed by the _cancelledAt guard in
           onBotMessage so they cannot re-lock the composer (#2562). */
        A._cancelledAt = Date.now();
        A.isStreaming = false;
        A.hideThinking();
        if (typeof A.onStreamEnd === "function") { A.onStreamEnd(); }
    };
})();
