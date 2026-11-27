"use strict";
/* Assistant parity — "Requires Agent Mode" hint in classic chat.
 *
 * The Assistant shell's 05b_model_picker.js disables subscription entries when
 * the agent runtime is unavailable. Classic chat already has its own model
 * picker (02_model_dock.js) and its own subscription lock (07_agent_subscribe.js),
 * so this module only adds the missing short hint with a link to Settings. It
 * never opens a second WebSocket and keeps its state in memory (no localStorage).
 */
(function () {
  var P = (window.GBChatAssistantParity = window.GBChatAssistantParity || {});
  var SERVER_STATE = {
    ready: "ready", wsl_missing: "needs_wsl", distro_missing: "not_installed",
    runtime_not_installed: "not_installed", unsupported: "unsupported"
  };
  var state = null;
  var mounted = false;

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }
  function botId() {
    return (window.ChatState && window.ChatState.currentBotId) ||
      window.__BOT_NAME__ || "default";
  }
  function headers() {
    var result = { Accept: "application/json" };
    try {
      var token = typeof window.getChatAccessToken === "function" ? window.getChatAccessToken() : "";
      if (token) result.Authorization = "Bearer " + token;
    } catch (error) { /* anonymous chat */ }
    return result;
  }
  function panel() {
    var node = document.getElementById("gbParityAgentGate");
    if (node) return node;
    node = document.createElement("div");
    node.id = "gbParityAgentGate";
    node.className = "gb-parity-agent-gate";
    node.setAttribute("role", "status");
    node.setAttribute("aria-live", "polite");
    node.hidden = true;
    node.innerHTML = '<span class="gb-parity-agent-gate-text"></span> ' +
      '<button type="button" class="gb-parity-agent-gate-open"></button>';
    var form = document.getElementById("chatForm");
    if (form && form.parentNode) form.parentNode.insertBefore(node, form);
    else document.body.appendChild(node);
    return node;
  }
  function paint() {
    var node = document.getElementById("gbParityAgentGate");
    if (!node) return;
    var text = node.querySelector(".gb-parity-agent-gate-text");
    var open = node.querySelector(".gb-parity-agent-gate-open");
    if (text && !text.textContent) {
      text.textContent = t("chat-agent-gate-hint",
        "This model needs Agent Mode, which is not available on this computer.");
    }
    if (open && !open.textContent) {
      open.textContent = t("chat-agent-gate-open", "Open Settings");
    }
    node.hidden = !needsHint();
  }
  function selectedModel() {
    var picker = document.getElementById("modelPicker");
    return picker ? String(picker.value || "auto") : "auto";
  }
  function agentSelected() {
    var value = selectedModel();
    if (value.indexOf("sub:") === 0) return true;
    var picker = document.getElementById("modelPicker");
    if (!picker) return false;
    var groups = picker.querySelectorAll("optgroup.gb-model-agent-group");
    for (var i = 0; i < groups.length; i++) {
      var option = groups[i].querySelector("option");
      if (option && option.value === value) return true;
    }
    return false;
  }
  function agentToggleOn() {
    var gate = window.GBChatControls;
    if (!gate || typeof gate.getState !== "function") return false;
    try { return !!gate.getState().agent; } catch (error) { return false; }
  }
  function needsHint() {
    if (state === null || state === "ready") return false;
    return agentSelected() || agentToggleOn();
  }
  function fetchState() {
    var runtime = window.GBSettingsAgentRuntime;
    if (runtime && typeof runtime.fetch === "function") {
      return runtime.fetch().then(function (status) {
        state = status ? (SERVER_STATE[status.state] || status.state) : null;
        paint();
        return state;
      });
    }
    return fetch("/api/product", { cache: "no-store", credentials: "same-origin", headers: headers() })
      .then(function (response) { return response.ok ? response.json() : null; })
      .then(function (data) {
        state = data ? (SERVER_STATE[data.agent_runtime_state] || data.agent_runtime_state || null) : null;
        paint();
        return state;
      })
      .catch(function () { paint(); return state; });
  }
  function openSettings() {
    var runtime = window.GBSettingsAgentRuntime;
    if (runtime && typeof runtime.openSection === "function") { runtime.openSection(); return; }
    if (typeof window.openDeepLink === "function") {
      window.openDeepLink("settings", { section: "agent-runtime" });
      return;
    }
    window.location.href = "/settings#agent-runtime";
  }
  P.agentRuntimeReady = function () { return state === null || state === "ready"; };
  P.agentGate = {
    mount: function () {
      if (mounted) return P;
      mounted = true;
      panel();
      document.addEventListener("click", function (event) {
        if (event.target && event.target.closest &&
            event.target.closest(".gb-parity-agent-gate-open")) openSettings();
      });
      var picker = document.getElementById("modelPicker");
      if (picker) picker.addEventListener("change", paint);
      window.addEventListener("gb-chat-ready", paint);
      window.addEventListener("gb:auth:login", fetchState);
      fetchState();
      return P;
    },
    refresh: fetchState,
    ready: P.agentRuntimeReady,
    state: function () { return state; },
    bot: botId,
    reason: function () {
      return state === null || state === "ready"
        ? "" : t("chat-agent-gate-required", "Requires Agent Mode");
    }
  };
})();
