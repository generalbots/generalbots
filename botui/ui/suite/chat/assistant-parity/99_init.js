/* Boot parity modules; the gate module loads after first paint. */
"use strict";
(function () {
  var P = window.GBChatAssistantParity = window.GBChatAssistantParity || {};
  function safe(n) {
    try { if (P[n] && P[n].mount) P[n].mount(); }
    catch (e) { window.gbLogError ? gbLogError(e, "parity:" + n) : console.error("parity:" + n, e); }
  }
  function init() {
    ["blocks", "workshop", "modelDock", "driveActions", "attachPanel", "consentVm",
      "vmWait", "vmAllBar", "agentSubscribe"].forEach(safe);
    safe("agentGate");
    if (!P._gateLoading && typeof Promise === "function") {
      P._gateLoading = 1;
      var s = document.createElement("script");
      s.src = "/suite/chat/assistant-parity/08_agent_gate.js?v=1";
      s.onload = s.onerror = function () { safe("agentGate"); };
      (document.head || document.body).appendChild(s);
    }
    return P;
  }
  P.init = init;
  document.readyState === "loading" ? document.addEventListener("DOMContentLoaded", init, { once: true }) : init();
})();
