/* Boots the assistant-parity add-ons inside the classic chat app.
 * Each module mounts itself against chat's existing DOM; none of them
 * replaces the renderer, the WebSocket or the native model picker. */
"use strict";
(function () {
  var P = window.GBChatAssistantParity = window.GBChatAssistantParity || {};

  function safe(name) {
    try {
      if (P[name] && typeof P[name].mount === "function") P[name].mount();
    } catch (error) {
      if (typeof window.gbLogError === "function") window.gbLogError(error, "assistant-parity:" + name);
      else window.console && console.error("assistant-parity:" + name, error);
    }
  }

  function init() {
    safe("blocks");
    safe("workshop");
    safe("modelDock");
    safe("driveActions");
    safe("attachPanel");
    safe("consentVm");
    safe("vmWait");
    safe("vmAllBar");
    safe("agentSubscribe");
    return P;
  }

  P.init = init;

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init, { once: true });
  } else {
    init();
  }
})();
