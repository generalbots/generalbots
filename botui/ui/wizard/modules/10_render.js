/* First-run wizard — render pipeline and per-step bindings. */
(function () {
  "use strict";

  var App = window.GBWizardApp;
  var W = App.W, S = App.S, B = App.B, esc = App.esc, t = App.t;
  var el = App.el, state = App.state, draft = App.draft;

  /* Forwarders resolve the functions defined in sibling modules at call time. */

  function renderComponents() { return App.renderComponents.apply(App, arguments); }
  function bindInstall() { return App.bindInstall.apply(App, arguments); }
  function bindFinish() { return App.bindFinish.apply(App, arguments); }
  function say(text) { return App.say(text); }
  function persistDraft() { return App.persistDraft(); }

  function renderRail() {
    var label = document.getElementById("railSectionLabel");
    if (label) { label.textContent = "// " + t("rail.section-label"); }
    el.rail.innerHTML = W.STEPS.map(function (step, index) {
      var n = index + 1;
      var num = (n < 10 ? "0" : "") + n;
      var current = n === state.step;
      var done = n < state.step || (state.provisioning.ready && n <= 5);
      return '<li class="rail-step' + (done ? " done" : "") + '"' +
        (current ? ' aria-current="step"' : "") + ">" +
        '<span class="rail-step-name">' + esc(t(step.key)) + "</span>" +
        '<span class="rail-step-num" aria-hidden="true">' +
          (done ? '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>' : num) +
        "</span></li>";
    }).join("");
  }

  function renderFooter() {
    var last = state.step === state.totalSteps;
    el.back.disabled = state.step === 1 || state.provisioning.running;
    el.next.innerHTML = esc(last ? t("wizard.open-assistant") : t("wizard.next")) +
      (last ? "" : ' <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" style="vertical-align:-2px"><path d="M5 12h14M13 6l6 6-6 6"/></svg>');
    el.next.disabled = !canAdvance();
    el.progress.style.width = Math.round((state.step / state.totalSteps) * 100) + "%";
    el.footLabel.textContent = t("wizard.foot-step", { current: state.step, total: state.totalSteps });
    el.counter.innerHTML = '<span class="wiz-sl">//</span>' + esc(t("wizard.step-counter", { current: state.step, total: state.totalSteps }));
  }

  function canAdvance() {
    if (state.provisioning.running) { return false; }
    if (state.step === 7) { return state.provisioning.ready || state.skipped; }
    if (state.step === 8) { return state.provisioning.ready || state.skipped || !state.context.first_run; }
    return S.validate(state.step, draft).ok;
  }

  function syncChrome() {
    document.title = t("wizard.title");
    if (el.langLabel) { el.langLabel.textContent = t("wizard.language"); }
    if (el.back) { el.back.textContent = t("wizard.back"); }
    if (el.skip) { el.skip.textContent = t("wizard.skip"); }
    if (el.modalCancel) { el.modalCancel.textContent = t("wizard.cancel"); }
    if (el.modal && el.modal.hidden) {
      el.modalTitle.textContent = t("wizard.confirm");
      el.modalConfirm.textContent = t("wizard.confirm");
    }
  }

  function render() {
    syncChrome();
    renderRail();
    renderFooter();
    el.heading.textContent = S.heading(state.step);
    el.body.innerHTML = S.render(state.step, draft, state.context);
    if (el.stepError) { el.stepError.hidden = true; el.stepError.textContent = ""; }
    if (state.step === 7) { renderComponents(); }
    bindStep();
    el.heading.focus();
    say(t("wizard.step-say", { current: state.step, total: state.totalSteps, title: S.heading(state.step) }));
  }

  function bindStep() {
    if (state.step === 1) {
      var name = document.getElementById("displayName");
      if (name) { name.addEventListener("input", function () { draft.display_name = name.value.trim(); }); }
      if (App.bindAccentPicker) { App.bindAccentPicker(); }
    }

    if (state.step === 2) {
      el.body.querySelectorAll("[data-open-url]").forEach(function (a) {
        a.addEventListener("click", function (e) { e.preventDefault(); B.openUrl(a.getAttribute("data-open-url")); });
      });
    }

    if (state.step === 3) {
      var path = document.getElementById("workspacePath");
      var size = document.getElementById("workspaceSize");
      if (path) {
        path.addEventListener("change", function () { draft.workspace.path = path.value.trim(); checkWorkspace(); });
        path.addEventListener("input", function () { draft.workspace.path = path.value.trim(); });
      }
      if (size) { size.addEventListener("change", function () { draft.workspace.size = size.value; persistDraft(); }); }
      var browse = document.getElementById("browseFolderBtn");
      if (browse) {
        browse.addEventListener("click", function () {
          B.pickFolder().then(function (folder) {
            if (folder) { path.value = folder; draft.workspace.path = folder; checkWorkspace(); }
          });
        });
      }
      checkWorkspace();
    }

    if (state.step === 4) { bindServices(); }
    if (state.step === 5) { bindPrivacy(); }
    if (state.step === 6) { bindNetwork(); }
    if (state.step === 7) { bindInstall(); }
    if (state.step === 8) { bindFinish(); }
  }

  function checkWorkspace() {
    var status = document.getElementById("workspaceStatus");
    var free = document.getElementById("workspaceFree");
    if (!status) { return; }
    var path = draft.workspace.path;
    if (!path) { status.textContent = t("workspace.choose"); return; }
    status.textContent = t("workspace.checking");
    B.checkWorkspace(path).then(function (result) {
      draft.workspace.blocked = !result.writable;
      draft.workspace.block_reason = result.writable ? "" : t("workspace.not-writable");
      status.textContent = result.writable ? t("workspace.writable") : t("workspace.not-writable");
      status.className = "hint " + (result.writable ? "ok" : "err");
      if (free) { free.textContent = t("workspace.free", { size: W.formatBytes(result.free_bytes) }); }
      renderFooter();
      persistDraft();
    }).catch(function (e) {
      status.textContent = String(e && e.message ? e.message : e);
      status.className = "hint err";
    });
  }

  function setServiceEnabled(id, on) {
    draft.services[id] = draft.services[id] || {};
    draft.services[id].enabled = on;
    var card = document.querySelector('[data-service="' + id + '"]');
    var body = document.getElementById("serviceBody-" + id);
    var badge = card ? card.querySelector(".badge") : null;
    var saved = draft.services[id];
    if (card) { card.classList.toggle("off", !on); }
    if (body) { body.hidden = !on; }
    if (badge) {
      badge.textContent = !on ? t("services.off") : (saved.client_id ? t("services.attention") : t("services.not-connected"));
      badge.className = "badge " + (on && saved.client_id ? "warn" : "");
    }
    persistDraft();
  }

  function bindServices() {
    Array.prototype.forEach.call(document.querySelectorAll("[data-service-toggle]"), function (toggle) {
      toggle.addEventListener("change", function () {
        setServiceEnabled(toggle.getAttribute("data-service-toggle"), toggle.checked);
      });
    });
    Array.prototype.forEach.call(document.querySelectorAll("[data-service-id]"), function (input) {
      input.addEventListener("input", function () {
        var id = input.getAttribute("data-service-id");
        draft.services[id] = draft.services[id] || {};
        draft.services[id].client_id = input.value.trim();
      });
    });
    Array.prototype.forEach.call(document.querySelectorAll("[data-service-secret]"), function (input) {
      input.addEventListener("input", function () {
        var id = input.getAttribute("data-service-secret");
        draft.services[id] = draft.services[id] || {};
        draft.services[id].client_secret = input.value;
      });
    });
    Array.prototype.forEach.call(document.querySelectorAll("[data-copy]"), function (button) {
      button.addEventListener("click", function () {
        var value = button.getAttribute("data-copy");
        if (navigator.clipboard) {
          navigator.clipboard.writeText(value).then(function () { say(t("nav.copied-redirect")); });
        }
      });
    });
    Array.prototype.forEach.call(document.querySelectorAll("[data-console]"), function (button) {
      button.addEventListener("click", function () { B.openUrl(button.getAttribute("data-console")); });
    });
  }

  function bindPrivacy() {
    var reveal = document.getElementById("revealVaultBtn");
    if (reveal) { reveal.addEventListener("click", function () { B.openFolder("vault"); }); }
  }

  function bindNetwork() {
    Array.prototype.forEach.call(document.querySelectorAll('input[name="networkMode"]'), function (input) {
      input.addEventListener("change", function () {
        draft.network = draft.network || { mode: "offline" };
        draft.network.mode = input.value === "offline" ? "offline" : "p2p";
        persistDraft();
      });
    });
  }

  App.renderRail = renderRail;
  App.renderFooter = renderFooter;
  App.canAdvance = canAdvance;
  App.syncChrome = syncChrome;
  App.render = render;
  App.bindStep = bindStep;
  App.checkWorkspace = checkWorkspace;
  App.bindServices = bindServices;
  App.setServiceEnabled = setServiceEnabled;
  App.bindPrivacy = bindPrivacy;
  App.bindNetwork = bindNetwork;
})();
