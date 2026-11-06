/* First-run wizard — listeners and boot sequence (loads last). */
(function () {
  "use strict";

  var App = window.GBWizardApp;
  var W = App.W, B = App.B, I = App.I, esc = App.esc, t = App.t;
  var el = App.el, state = App.state, draft = App.draft, logLines = App.logLines;

  function resumeBanner() {
    if (!state.context.resume_step || state.context.resume_step <= 1) { return; }
    var banner = document.createElement("div");
    banner.className = "card";
    banner.style.marginBottom = "14px";
    banner.innerHTML = '<p class="note warn" style="margin:0">' + esc(t("nav.resume", { step: state.context.resume_step })) + '</p>';
    el.body.insertBefore(banner, el.body.firstChild);
  }

  el.modalCancel.addEventListener("click", App.closeModal);
  el.modalConfirm.addEventListener("click", function () {
    var action = App.modalAction;
    var value = el.modalInput.value;
    App.closeModal();
    if (action) { action(value); }
  });
  el.modal.addEventListener("keydown", function (event) {
    if (event.key === "Escape") { event.stopPropagation(); App.closeModal(); }
  });

  el.next.addEventListener("click", App.next);
  el.back.addEventListener("click", function () { App.goTo(state.step - 1); });
  el.skip.addEventListener("click", App.skip);

  document.addEventListener("keydown", function (event) {
    if (!el.modal.hidden) { return; }
    if (event.altKey && event.key === "ArrowRight") { event.preventDefault(); App.next(); }
    if (event.altKey && event.key === "ArrowLeft") { event.preventDefault(); App.goTo(state.step - 1); }
    var tag = (event.target && event.target.tagName) || "";
    var typing = tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || tag === "BUTTON";
    if (event.key === "Enter" && !typing) { event.preventDefault(); App.next(); }
  });

  B.onProgress(function (event) {
    if (event.id === "__done__") {
      App.handleDone();
      return;
    }
    if (event.state === "failed") {
      state.provisioning.running = false;
      state.provisioning.failed = event.message;
      logLines.push("[" + event.id + "] failed: " + event.message);
      App.showProvisionError(event.message);
      return;
    }
    logLines.push("[" + event.id + "] " + event.state + " — " + event.message);
    App.updateComponent(event);
    App.renderLog();
    if (event.state === "ready" && event.id === "botserver") { App.say(t("nav.assistant-ready")); }
  });

  I.populate(el.lang);
  window.addEventListener("wizard:localeChanged", function () {
    if (!state.context) { return; }
    App.render();
  });

  B.context().then(function (context) {
    state.context = context;
    if (!draft.workspace.path) { draft.workspace.path = state.context.workspace_path; }
    if (!draft.model.endpoint && draft.model.mode !== "local") {
      draft.model.endpoint = W.defaultModelEndpoint(draft.model.provider);
      draft.model.model = W.defaultModelName(draft.model.provider);
    }
    return B.loadDraft().then(function (saved) {
      if (saved) {
        draft.display_name = saved.display_name || draft.display_name;
        draft.model = Object.assign(draft.model, saved.model || {});
        draft.model.api_key = "";
        draft.workspace = Object.assign(draft.workspace, saved.workspace || {});
        draft.telemetry = !!saved.telemetry;
        draft.services = (saved.services && !Array.isArray(saved.services)) ? saved.services : {};
        Object.keys(draft.services).forEach(function (id) { draft.services[id].client_secret = ""; });
      }
      state.resumed = !!state.context.resume_step;
      state.provisioning.components = (state.context.completed_components || []).map(function (id) {
        return { id: id, state: "ready", message: "Ready", elapsed_ms: 0 };
      });
      App.ensureAdmin();
      state.step = state.context.resume_step || 1;
      el.root.setAttribute("data-state", "ready");
      if (el.note) {
        el.note.textContent = state.context.desktop_local
          ? t("nav.rail-desktop")
          : t("nav.rail-local");
      }
      App.render();
      if (state.resumed) { resumeBanner(); }
      if (state.context.first_run === false) {
        el.skip.hidden = true;
        state.provisioning.ready = true;
      }
    });
  }).catch(function (e) {
    el.body.innerHTML = '<p class="note err">' + esc(t("nav.state-error", { error: e && e.message ? e.message : e })) + "</p>";
    el.root.setAttribute("data-state", "error");
  });
})();
