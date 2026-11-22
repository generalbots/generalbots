/* First-run wizard controller — core namespace, shared state and small helpers.
 * Split out of wizard.js; load order is fixed by index.html. */
(function () {
  "use strict";

  var App = window.GBWizardApp = window.GBWizardApp || {};

  var W = window.GBWizard;
  var S = window.GBWizardSteps;
  var B = window.GBWizardBridge;
  var I = window.WizardI18n;

  App.W = W;
  App.S = S;
  App.B = B;
  App.I = I;
  App.esc = S.esc;
  App.t = function (key, params) { return I.t(key, params); };
  App.modalAction = null;

  var el = {
    root: document.getElementById("wizardRoot"),
    rail: document.getElementById("railSteps"),
    note: document.getElementById("railNote"),
    counter: document.getElementById("stepCounter"),
    heading: document.getElementById("stepHeading"),
    stepError: document.getElementById("stepError"),
    body: document.getElementById("stepBody"),
    progress: document.getElementById("progressFill"),
    footLabel: document.getElementById("footLabel"),
    back: document.getElementById("backBtn"),
    next: document.getElementById("nextBtn"),
    skip: document.getElementById("skipBtn"),
    lang: document.getElementById("wizardLanguage"),
    langLabel: document.getElementById("wizardLanguageLabel"),
    status: document.getElementById("statusLive"),
    modal: document.getElementById("modalBackdrop"),
    modalTitle: document.getElementById("modalTitle"),
    modalText: document.getElementById("modalText"),
    modalField: document.getElementById("modalField"),
    modalFieldLabel: document.getElementById("modalFieldLabel"),
    modalInput: document.getElementById("modalInput"),
    modalCancel: document.getElementById("modalCancel"),
    modalConfirm: document.getElementById("modalConfirm")
  };
  App.el = el;

  App.state = W.state;
  App.draft = App.state.draft;
  var draft = App.draft;
  App.logLines = [];

  function say(text) { el.status.textContent = text; }

  function persistDraft() {
    var safe = W.clone(draft);
    safe.model.api_key = "";
    Object.keys(safe.services || {}).forEach(function (id) { safe.services[id].client_secret = ""; });
    B.saveDraft(safe).catch(function () {});
  }

  /* A uniform random index below `limit` (rejection sampling, no modulo bias). */
  function randomBelow(limit) {
    var buf = new Uint8Array(1);
    var ceiling = 256 - (256 % limit);
    for (;;) {
      if (window.crypto && window.crypto.getRandomValues) { window.crypto.getRandomValues(buf); }
      else { buf[0] = Math.floor(Math.random() * 256); }
      if (buf[0] < ceiling) { return buf[0] % limit; }
    }
  }

  /* 18 characters with at least one upper case letter, lower case letter,
     digit and symbol: the directory's default password rules refuse a
     password that lacks one of them. */
  function generatePassword() {
    var classes = ["ABCDEFGHJKLMNPQRSTUVWXYZ", "abcdefghijkmnopqrstuvwxyz", "23456789", "!@#$%&*"];
    var alphabet = classes.join("");
    var chars = classes.map(function (set) { return set.charAt(randomBelow(set.length)); });
    while (chars.length < 18) { chars.push(alphabet.charAt(randomBelow(alphabet.length))); }
    for (var i = chars.length - 1; i > 0; i -= 1) {
      var j = randomBelow(i + 1);
      var tmp = chars[i]; chars[i] = chars[j]; chars[j] = tmp;
    }
    return chars.join("");
  }

  function ensureAdmin() {
    if (!draft.admin.email) { draft.admin.email = "admin@localhost"; }
    if (!draft.admin.password) { draft.admin.password = generatePassword(); }
    if (!draft.admin.user_id) {
      if (window.crypto && window.crypto.randomUUID) {
        draft.admin.user_id = window.crypto.randomUUID();
      } else {
        draft.admin.user_id = "00000000-0000-4000-8000-" + String(Date.now()).slice(-12);
      }
    }
  }

  App.say = say;
  App.persistDraft = persistDraft;
  App.generatePassword = generatePassword;
  App.ensureAdmin = ensureAdmin;
})();
