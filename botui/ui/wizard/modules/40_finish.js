/* First-run wizard — Finish step bindings (admin credentials, account linking). */
(function () {
  "use strict";

  var App = window.GBWizardApp;
  var t = App.t;
  var esc = App.esc;
  var state = App.state, draft = App.draft;
  var B = App.B;

  /* Forwarders resolve the functions defined in sibling modules at call time. */

  function generatePassword() { return App.generatePassword(); }
  function say(text) { return App.say(text); }
  function bindFinishAccount() { return App.bindFinishAccount(); }

  function capabilityRow(label, capability) {
    var available = !!(capability && capability.available);
    var detail = "";
    if (!available && capability) {
      detail = capability.remediation || capability.detail || "";
    } else if (available && capability && capability.detail) {
      detail = capability.detail;
    }
    var text = t(available ? "finish.media-available" : "finish.media-unavailable");
    if (detail) { text += " — " + detail; }
    return '<div class="summary-row"><span>' + esc(label) + "</span>" +
      '<span class="note ' + (available ? "ok" : "err") + '">' + esc(text) + "</span></div>";
  }

  function renderMediaCapabilities(payload) {
    var rows = document.getElementById("mediaCapabilitiesRows");
    var badge = document.getElementById("mediaCapabilitiesBadge");
    var note = document.getElementById("mediaCapabilitiesNote");
    if (!rows) { return; }
    var capabilities = (payload && payload.capabilities) ? payload.capabilities : (payload || {});
    rows.innerHTML = capabilityRow(t("finish.media-barcode"), capabilities.barcode) +
      capabilityRow(t("finish.media-ocr"), capabilities.ocr);
    var allAvailable = !!(capabilities.barcode && capabilities.barcode.available) &&
      !!(capabilities.ocr && capabilities.ocr.available);
    var live = !!(payload && payload.service_running);
    if (badge) {
      badge.textContent = t(live ? "finish.media-live" : "finish.media-host");
      badge.className = "badge " + (allAvailable ? "ok" : "warn");
    }
    if (note) {
      note.textContent = t(live ? "finish.media-live-note" : "finish.media-host-note");
      note.className = "note " + (allAvailable ? "ok" : "warn");
    }
  }

  function loadMediaCapabilities() {
    var rows = document.getElementById("mediaCapabilitiesRows");
    if (!rows) { return; }
    rows.innerHTML = '<p class="note">' + esc(t("finish.media-checking")) + "</p>";
    B.mediaCapabilities().then(renderMediaCapabilities).catch(function (e) {
      var rows = document.getElementById("mediaCapabilitiesRows");
      if (rows) {
        rows.innerHTML = '<p class="note err">' +
          esc(t("finish.media-error", { reason: e && e.message ? e.message : e })) + "</p>";
      }
    });
  }

  function bindFinish() {
    var email = document.getElementById("adminEmail");
    var password = document.getElementById("adminPassword");
    if (email) {
      email.addEventListener("input", function () { draft.admin.email = email.value.trim(); });
    }
    var togglePassword = document.getElementById("toggleAdminPasswordBtn");
    if (togglePassword) {
      togglePassword.addEventListener("click", function () {
        var field = document.getElementById("adminPassword");
        if (!field) return;
        var reveal = field.type === "password";
        field.type = reveal ? "text" : "password";
        togglePassword.setAttribute("aria-pressed", reveal ? "true" : "false");
        togglePassword.textContent = reveal
          ? (window.WizardI18n ? WizardI18n.t("finish.hide-password") : "Hide password")
          : (window.WizardI18n ? WizardI18n.t("finish.show-password") : "Show password");
      });
    }

    var copy = document.getElementById("copyAdminBtn");
    if (copy) {
      copy.addEventListener("click", function () {
        var value = "email: " + draft.admin.email + "\npassword: " + draft.admin.password;
        if (navigator.clipboard) {
          navigator.clipboard.writeText(value).then(function () { say(t("nav.copied-admin")); });
        }
      });
    }
    var regenerate = document.getElementById("regenerateAdminBtn");
    if (regenerate) {
      regenerate.addEventListener("click", function () {
        draft.admin.password = generatePassword();
        if (password) { password.value = draft.admin.password; }
        say(t("nav.new-password"));
      });
    }
    var status = document.getElementById("finishStatus");
    if (status && !state.provisioning.ready && !state.skipped) {
      status.textContent = t("finish.start-prev");
    }
    bindFinishAccount();
    loadMediaCapabilities();
  }

  App.bindFinish = bindFinish;
  App.renderMediaCapabilities = renderMediaCapabilities;
  App.loadMediaCapabilities = loadMediaCapabilities;
})();
