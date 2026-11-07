/* First-run wizard — step navigation, validation and handoff. */
(function () {
  "use strict";

  var App = window.GBWizardApp;
  var S = App.S, B = App.B, t = App.t;
  var el = App.el, state = App.state, draft = App.draft;

  /* Forwarders resolve the functions defined in sibling modules at call time. */

  function say(text) { return App.say(text); }
  function hostLocked() { return App.hostLocked(); }
  function startProvisioning() { return App.startProvisioning(); }
  function stopAccountTimer() { return App.stopAccountTimer(); }
  function render() { return App.render(); }
  function persistDraft() { return App.persistDraft(); }

  function goTo(step) {
    if (step < 1 || step > state.totalSteps) { return; }
    if (step !== 7) { stopAccountTimer(); }
    state.step = step;
    render();
    persistDraft();
  }

  function next() {
    if (state.step === state.totalSteps) { finish(); return; }
    var check = S.validate(state.step, draft);
    if (!check.ok) { say(check.reason); showInlineError(check.reason); return; }
    if (state.step === 5) {
      goTo(6);
      return;
    }
    if (state.step === 6 && !state.provisioning.ready && !state.skipped) {
      if (hostLocked()) { say(t("host.blocked-say")); return; }
      startProvisioning();
      return;
    }
    goTo(state.step + 1);
  }

  function showInlineError(reason) {
    if (el.stepError) {
      el.stepError.textContent = reason;
      el.stepError.hidden = false;
    }
    say(reason);
  }

  function skip() {
    state.skipped = true;
    goTo(state.totalSteps);
    say(t("nav.skipped"));
  }

  function finish() {
    if (!state.provisioning.ready && !state.context.first_run) { handoff(); return; }
    if (!state.provisioning.ready) { startProvisioning(); return; }
    handoff();
  }

  function handoff() {
    el.next.disabled = true;
    el.next.textContent = t("finish.opening");
    B.finish({
      display_name: draft.display_name,
      model: draft.model,
      workspace: draft.workspace,
      services: draft.services,
      admin: draft.admin
    }).then(function () {
      say(t("finish.opened"));
    }).catch(function (e) {
      el.next.disabled = false;
      el.next.textContent = t("wizard.open-assistant");
      var status = document.getElementById("finishStatus");
      if (status) { status.textContent = String(e && e.message ? e.message : e); status.className = "note err"; }
    });
  }

  App.goTo = goTo;
  App.next = next;
  App.showInlineError = showInlineError;
  App.skip = skip;
  App.finish = finish;
  App.handoff = handoff;
})();
