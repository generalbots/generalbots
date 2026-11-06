/* First-run wizard — Finish step bindings (admin credentials, account linking). */
(function () {
  "use strict";

  var App = window.GBWizardApp;
  var t = App.t;
  var state = App.state, draft = App.draft;

  /* Forwarders resolve the functions defined in sibling modules at call time. */

  function generatePassword() { return App.generatePassword(); }
  function say(text) { return App.say(text); }
  function bindFinishAccount() { return App.bindFinishAccount(); }

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
  }

  App.bindFinish = bindFinish;
})();
