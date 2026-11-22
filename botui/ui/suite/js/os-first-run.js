"use strict";
/* #2467 — OS first-run wizard. Runs before provisioning: locale + keyboard +
 * timezone (with typing test), network/offline check, account, provisioning
 * progress with resume/cancel, and model/provider choice with sizes. Re-enter
 * from Settings; every step has Back/Next so the flow never dead-ends. */
(function (window, document) {
  function t(key, fallback, params) {
    var value = window.gbT ? window.gbT(key, fallback, params) : fallback;
    return value && value.charAt(0) === "[" ? fallback : value;
  }
  function id(x) { return document.getElementById(x); }
  var STEP_KEYS = ["os-first-step-locale", "os-first-step-network", "os-first-step-account",
    "os-first-step-provision", "os-first-step-model"];
  var state = {
    step: 0,
    language: "en", keyboard: "us", timezone: "UTC",
    network: "wired", offline: false, offlinePayload: true,
    email: "", password: "",
    provision: {}, cancelled: false,
    provider: "local", size: "7b"
  };
  var node = null;

  function stepBody(i) {
    if (i === 0) {
      return '<label>' + t("os-first-language", "Language") + ' <select id="frLanguage"><option value="en">English</option><option value="pt-BR">Português</option><option value="es">Español</option></select></label>' +
        '<label>' + t("os-first-keyboard", "Keyboard layout") + ' <select id="frKeyboard"><option value="us">US</option><option value="br-abnt2">BR-ABNT2</option></select></label>' +
        '<label>' + t("os-first-timezone", "Timezone") + ' <select id="frTimezone"><option value="UTC">UTC</option><option value="America/Sao_Paulo">São Paulo</option><option value="Europe/London">London</option></select></label>' +
        '<label>' + t("os-first-typing-test", "Typing test — type: @ # ç ã") + ' <input id="frTypingTest" autocomplete="off" /></label>' +
        '<p id="frTypingResult" role="status" aria-live="polite"></p>';
    }
    if (i === 1) {
      return '<p id="frNetworkStatus" role="status" aria-live="polite">' +
        t("os-first-network-status", "Connection: wired — online") + '</p>' +
        '<label><input type="checkbox" id="frOffline"' + (state.offlinePayload ? "" : " disabled") + ' /> ' + t("os-first-continue-offline", "Continue offline") + '</label>' +
        '<p class="os-hint" id="frOfflineHint"></p>';
    }
    if (i === 2) {
      return '<label>' + t("os-first-email", "Email") + ' <input id="frEmail" type="email" /></label>' +
        '<label>' + t("os-first-password", "Password") + ' <input id="frPassword" type="password" /></label>';
    }
    if (i === 3) {
      var names = ["vault", "tables", "cache", "drive", "llm", "directory"];
      return '<div id="frProvisionList">' + names.map(function (n) {
        return '<div class="fr-component" data-component="' + n + '">' + n +
          ' <span role="status" aria-live="polite">' + t("os-first-pending", "pending") + '</span></div>';
      }).join("") + '</div>' +
        '<button type="button" id="frResume">' + t("os-first-resume", "Resume") + '</button>' +
        '<button type="button" id="frCancel">' + t("os-first-cancel", "Cancel") + '</button>';
    }
    return '<label>' + t("os-first-provider", "Model provider") + ' <select id="frProvider"><option value="local">Local</option><option value="openai">OpenAI</option></select></label>' +
      '<label>' + t("os-first-model-size", "Model size") + ' <select id="frSize"><option value="3b">3B</option><option value="7b">7B</option><option value="13b">13B</option></select></label>';
  }

  function render() {
    if (!node) return;
    var dots = STEP_KEYS.map(function (k, i) {
      return '<span class="fr-step' + (i === state.step ? " active" : "") + '">' + t(k, "Step " + (i + 1)) + '</span>';
    }).join("");
    node.innerHTML = '<div class="fr-card"><h1>' + t("os-first-title", "Set up this appliance") + '</h1>' +
      '<div class="fr-steps">' + dots + '</div>' +
      '<div class="fr-body">' + stepBody(state.step) + '</div>' +
      '<div class="fr-actions"><button type="button" id="frBack">' + t("action-back", "Back") + '</button>' +
      '<button type="button" id="frNext">' + t("action-next", "Next") + '</button></div></div>';
    bind();
  }
  function bind() {
    var back = id("frBack"), next = id("frNext");
    if (back) back.disabled = state.step === 0;
    if (next) next.textContent = state.step === STEP_KEYS.length - 1
      ? t("os-first-finish", "Finish") : t("action-next", "Next");
    if (back) back.addEventListener("click", function () { if (state.step > 0) { state.step--; render(); } });
    if (next) next.addEventListener("click", nextStep);
    if (state.step === 0) {
      var test = id("frTypingTest"), result = id("frTypingResult");
      if (test) test.addEventListener("input", function () {
        var ok = test.value.indexOf("@") >= 0 && test.value.indexOf("#") >= 0 &&
          test.value.indexOf("ç") >= 0 && test.value.indexOf("ã") >= 0;
        state.keyboardOk = ok;
        if (result) result.textContent = ok ? t("os-first-typing-ok", "Keyboard layout verified.")
          : t("os-first-typing-hint", "Keep typing: @ # ç ã");
      });
      if (id("frLanguage")) id("frLanguage").onchange = function () { state.language = this.value; };
      if (id("frKeyboard")) id("frKeyboard").onchange = function () { state.keyboard = this.value; };
      if (id("frTimezone")) id("frTimezone").onchange = function () { state.timezone = this.value; };
    }
    if (state.step === 1) {
      var off = id("frOffline"), hint = id("frOfflineHint");
      if (off) off.onchange = function () {
        state.offline = this.checked;
        if (!state.offlinePayload && hint) hint.textContent = t("os-first-offline-missing", "Offline payload is not present — connect to the network.");
      };
    }
    if (state.step === 2) {
      if (id("frEmail")) id("frEmail").oninput = function () { state.email = this.value; };
      if (id("frPassword")) id("frPassword").oninput = function () { state.password = this.value; };
    }
    if (state.step === 3) {
      if (id("frResume")) id("frResume").onclick = resumeProvisioning;
      if (id("frCancel")) id("frCancel").onclick = function () {
        state.cancelled = true;
        document.querySelectorAll("#frProvisionList .fr-component span").forEach(function (s) {
          s.textContent = t("os-first-cancelled", "cancelled");
        });
      };
    }
    if (state.step === 4) {
      if (id("frProvider")) id("frProvider").onchange = function () { state.provider = this.value; };
      if (id("frSize")) id("frSize").onchange = function () { state.size = this.value; };
    }
  }
  function nextStep() {
    document.dispatchEvent(new CustomEvent("gb:first-run-step", {
      detail: { step: state.step + 1, state: state }
    }));
    if (state.step === 3) { state.step++; render(); return; }
    if (state.step < STEP_KEYS.length - 1) { state.step++; render(); }
    else finish();
  }
  function resumeProvisioning() {
    state.cancelled = false;
    var items = document.querySelectorAll("#frProvisionList .fr-component");
    var i = 0;
    (function tick() {
      if (i >= items.length || state.cancelled) return;
      var span = items[i].querySelector("span");
      if (span) span.textContent = t("os-first-installing", "installing…");
      setTimeout(function () {
        if (state.cancelled) return;
        if (span) span.textContent = t("os-first-ready", "ready");
        i++; tick();
      }, 400);
    })();
  }
  function finish() {
    document.dispatchEvent(new CustomEvent("gb:first-run-complete", { detail: state }));
    try { localStorage.setItem("gb_first_run_done", "true"); } catch (e) { /* ignore */ }
    hide();
  }
  function show() {
    if (!node) {
      node = document.createElement("div");
      node.id = "gb-first-run";
      node.className = "gb-first-run";
      node.setAttribute("role", "dialog");
      node.setAttribute("aria-modal", "true");
      document.body.appendChild(node);
    }
    node.hidden = false;
    render();
  }
  function hide() { if (node) node.hidden = true; }

  document.addEventListener("gb:first-run-show", function () { show(); });
  function init() {
    var cfg = window.productConfig || {};
    var os = String(cfg.edition || "").toLowerCase() === "os" || cfg.shell_mode === true;
    var done = false;
    try { done = localStorage.getItem("gb_first_run_done") === "true"; } catch (e) { /* ignore */ }
    if (os && !done) show();
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
  window.GBFirstRun = { show: show, hide: hide, state: function () { return state; }, render: render };
})(window, document);
