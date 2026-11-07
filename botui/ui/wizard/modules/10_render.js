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
    el.rail.innerHTML = W.STEPS.map(function (step, index) {
      var n = index + 1;
      var current = n === state.step;
      var done = n < state.step || (state.provisioning.ready && n <= 5);
      return '<li class="rail-step' + (done ? " done" : "") + '"' +
        (current ? ' aria-current="step"' : "") + ">" +
        '<span class="num" aria-hidden="true">' + (done ? "✓" : n) + "</span>" +
        '<span class="name">' + esc(t(step.key)) + "</span></li>";
    }).join("");
  }

  function renderFooter() {
    var last = state.step === state.totalSteps;
    el.back.disabled = state.step === 1 || state.provisioning.running;
    el.next.textContent = last ? t("wizard.open-assistant") : t("wizard.next");
    el.next.disabled = !canAdvance();
    el.progress.style.width = Math.round((state.step / state.totalSteps) * 100) + "%";
    el.footLabel.textContent = t("wizard.foot-step", { current: state.step, total: state.totalSteps });
    el.counter.textContent = t("wizard.step-counter", { current: state.step, total: state.totalSteps });
  }

  function canAdvance() {
    if (state.provisioning.running) { return false; }
    if (state.step === 6) { return state.provisioning.ready || state.skipped; }
    if (state.step === 7) { return state.provisioning.ready || state.skipped || !state.context.first_run; }
    return S.validate(state.step, draft).ok;
  }

  function syncChrome() {
    document.title = t("wizard.title");
    if (el.langLabel) { el.langLabel.textContent = t("wizard.language"); }
    if (el.back) { el.back.textContent = t("wizard.back"); }
    if (el.skip) { el.skip.textContent = t("wizard.skip"); }
    if (el.modalCancel) { el.modalCancel.textContent = t("wizard.cancel"); }
  }

  function render() {
    syncChrome();
    renderRail();
    renderFooter();
    el.heading.textContent = S.heading(state.step);
    el.body.innerHTML = S.render(state.step, draft, state.context);
    if (el.stepError) { el.stepError.hidden = true; el.stepError.textContent = ""; }
    if (state.step === 5) { renderComponents(); }
    bindStep();
    el.heading.focus();
    say(t("wizard.step-say", { current: state.step, total: state.totalSteps, title: S.heading(state.step) }));
  }

  function bindStep() {
    if (state.step === 1) {
      var name = document.getElementById("displayName");
      if (name) { name.addEventListener("input", function () { draft.display_name = name.value.trim(); }); }
    }

    if (state.step === 2) {
      Array.prototype.forEach.call(document.querySelectorAll('[data-model-mode]'), function (card) {
        card.addEventListener("click", function () {
          var mode = card.getAttribute("data-model-mode");
          draft.model.mode = mode;
          if (mode === "claude-code" || mode === "chatgpt-plan") {
            draft.model.provider = mode;
            draft.model.endpoint = "";
            draft.model.model = W.defaultModelName(mode);
            draft.model.api_key = "";
            draft.model.tested = true;
          } else if (mode === "free") {
            draft.model.provider = "kilo";
            draft.model.endpoint = W.defaultModelEndpoint("kilo");
            draft.model.model = W.defaultModelName("kilo");
            draft.model.api_key = "";
            draft.model.tested = false;
          } else if (mode === "own-key" || mode === "hosted") {
            if (!W.defaultModelEndpoint(draft.model.provider)) {
              draft.model.provider = "openai";
            }
            draft.model.endpoint = W.defaultModelEndpoint(draft.model.provider);
            draft.model.model = W.defaultModelName(draft.model.provider);
            draft.model.tested = false;
          } else if (mode === "local") {
            draft.model.tested = true;
          }
          persistDraft();
          render();
        });
      });
      bindModelDetail();
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
    if (state.step === 6) { bindInstall(); }
    if (state.step === 7) { bindFinish(); }
  }

  function bindModelDetail() {
    var provider = document.getElementById("modelProvider");
    var endpoint = document.getElementById("modelEndpoint");
    var modelName = document.getElementById("modelName");
    var key = document.getElementById("modelKey");
    var test = document.getElementById("testModelBtn");
    if (provider) {
      provider.addEventListener("change", function () {
        draft.model.provider = provider.value;
        if (draft.model.mode === "free") {
          draft.model.endpoint = W.defaultModelEndpoint(provider.value);
          draft.model.model = W.defaultModelName(provider.value);
          draft.model.tested = false;
          document.getElementById("modelDetail").innerHTML = S.modelDetail(draft);
          bindModelDetail();
          renderFooter();
          persistDraft();
          return;
        }
        if (provider.value !== "custom") {
          endpoint.value = W.defaultModelEndpoint(provider.value);
          modelName.value = W.defaultModelName(provider.value);
        }
        draft.model.endpoint = endpoint.value;
        draft.model.model = modelName.value;
        draft.model.tested = false;
        document.getElementById("modelTestStatus").textContent = t("model.untested");
        renderFooter();
      });
    }
    if (endpoint) { endpoint.addEventListener("input", function () { draft.model.endpoint = endpoint.value.trim(); draft.model.tested = false; renderFooter(); }); }
    if (modelName) { modelName.addEventListener("input", function () { draft.model.model = modelName.value.trim(); draft.model.tested = false; renderFooter(); }); }
    if (key) { key.addEventListener("input", function () { draft.model.api_key = key.value; draft.model.tested = false; renderFooter(); }); }
    if (test) {
      test.addEventListener("click", function () {
        var status = document.getElementById("modelTestStatus");
        status.textContent = t("model.testing");
        status.className = "note";
        test.disabled = true;
        draft.model.endpoint = endpoint ? endpoint.value.trim() : "";
        draft.model.model = modelName ? modelName.value.trim() : "";
        draft.model.api_key = key ? key.value : "";
        B.testModel({
          mode: draft.model.mode,
          provider: draft.model.provider,
          endpoint: draft.model.endpoint,
          model: draft.model.model,
          api_key: draft.model.api_key
        }).then(function (result) {
          draft.model.tested = !!result.ok;
          status.textContent = result.message;
          status.className = "note " + (result.ok ? "ok" : "err");
          say(result.message);
        }).catch(function (e) {
          draft.model.tested = false;
          status.textContent = String(e && e.message ? e.message : e);
          status.className = "note err";
        }).then(function () {
          test.disabled = false;
          renderFooter();
          persistDraft();
        });
      });
    }
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
      draft.workspace.block_reason = result.reason || "";
      status.textContent = result.writable ? t("workspace.writable") : (result.reason || t("workspace.not-writable"));
      status.className = "hint " + (result.writable ? "ok" : "err");
      if (free) { free.textContent = t("workspace.free", { size: W.formatBytes(result.free_bytes) }); }
      renderFooter();
      persistDraft();
    }).catch(function (e) {
      status.textContent = String(e && e.message ? e.message : e);
      status.className = "hint err";
    });
  }

  function bindServices() {
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

  App.renderRail = renderRail;
  App.renderFooter = renderFooter;
  App.canAdvance = canAdvance;
  App.syncChrome = syncChrome;
  App.render = render;
  App.bindStep = bindStep;
  App.bindModelDetail = bindModelDetail;
  App.checkWorkspace = checkWorkspace;
  App.bindServices = bindServices;
  App.bindPrivacy = bindPrivacy;
})();
