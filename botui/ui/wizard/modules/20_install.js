/* First-run wizard — provisioning, component progress and host prerequisites. */
(function () {
  "use strict";

  var App = window.GBWizardApp;
  var B = App.B, esc = App.esc, t = App.t;
  var state = App.state, draft = App.draft, logLines = App.logLines;

  function render() { return App.render(); }
  function renderFooter() { return App.renderFooter(); }
  function goTo(step) { return App.goTo(step); }
  function openModal(options) { return App.openModal(options); }
  function say(text) { return App.say(text); }
  function persistDraft() { return App.persistDraft(); }

  /* -- per-component timing ------------------------------------------- */
  var compStartTimes = {};

  function formatDuration(ms) {
    if (ms == null || ms <= 0) { return ""; }
    if (ms < 1000) { return ms + "ms"; }
    if (ms < 60000) { return (ms / 1000).toFixed(1) + "s"; }
    var m = Math.floor(ms / 60000);
    var s = Math.round((ms % 60000) / 1000);
    return m + "m " + s + "s";
  }

  /* -- install step binding ------------------------------------------- */

  function bindInstall() {
    var start = document.getElementById("startSetupBtn");
    if (start) { start.addEventListener("click", startProvisioning); }
    var cancel = document.getElementById("cancelSetupBtn");
    if (cancel) { cancel.addEventListener("click", function () { B.cancel(); }); }
    var logs = document.getElementById("openLogsBtn");
    if (logs) { logs.addEventListener("click", function () { B.openFolder("logs"); }); }
    var reset = document.getElementById("resetStackBtn");
    if (reset) { reset.addEventListener("click", confirmReset); }
    bindToggleDetails();
    var recheck = document.getElementById("hostRecheckBtn");
    if (recheck) { recheck.addEventListener("click", function () { checkHost(true); }); }
    var proceed = document.getElementById("hostContinueBtn");
    if (proceed) {
      proceed.addEventListener("click", function () {
        state.host.acknowledged = true;
        renderHostCheck();
        startProvisioning();
      });
    }
    var skipAgent = document.getElementById("hostSkipAgentBtn");
    if (skipAgent) {
      skipAgent.addEventListener("click", function () {
        draft.skip_all_agentic_work = true;
        persistDraft();
        renderHostCheck();
        if (!hostLocked() && !state.provisioning.running && !state.provisioning.ready) {
          startProvisioning();
        }
      });
    }
    var agentToggle = document.getElementById("hostAgentToggle");
    if (agentToggle) {
      agentToggle.addEventListener("change", function () {
        draft.skip_all_agentic_work = !agentToggle.checked;
        persistDraft();
        renderHostCheck();
      });
    }
    if (state.provisioning.running) {
      setProvisionUi(true);
      renderComponents();
      renderHostCheck();
    } else if (state.provisioning.ready) {
      markAllReady();
      updateInstallProgress();
      if (state.host.checked && !state.host.checking) { renderHostCheck(); }
      else { checkHost(false); }
    } else if (!state.skipped) {
      if (state.host.checked || state.host.checking) { renderHostCheck(); }
      else { checkHost(true); }
    }
  }

  /* -- toggle details (show/hide live output column) ------------------ */

  function bindToggleDetails() {
    var toggle = document.getElementById("toggleInstallDetails");
    if (!toggle) { return; }
    toggle.addEventListener("click", function () {
      var output = document.getElementById("installOutputCol");
      var cols = document.getElementById("installColumns");
      var icon = document.getElementById("toggleDetailsIcon");
      if (!output || !cols) { return; }
      var hidden = output.style.display === "none";
      output.style.display = hidden ? "" : "none";
      cols.style.gridTemplateColumns = hidden ? "45% 1fr" : "1fr";
      if (icon) { icon.textContent = hidden ? "▼" : "▶"; }
      toggle.lastChild.textContent = " " + t(hidden ? "install.hide-details" : "install.show-details");
    });
  }

  /* -- host prerequisites --------------------------------------------- */

  function hostLocked() { return window.GBWizardHost.locked(state); }
  function syncStartButton() { window.GBWizardHost.syncStartButton(state); }

  function renderHostCheck() {
    window.GBWizardHost.render(state);
    collapseHostIfReady();
  }

  function collapseHostIfReady() {
    var details = document.getElementById("hostReadiness");
    var summary = document.getElementById("hostSummaryText");
    if (!details || !summary) { return; }
    var result = state.host.result;
    var allReady = result && result.status === "ready";
    var skippedOk = result && result.desktop_ready && result.media_ready &&
      draft.skip_all_agentic_work;
    if (state.host.checked && (allReady || skippedOk)) {
      summary.textContent = result.message || t("host.status-ready");
      details.removeAttribute("open");
    } else if (state.host.checked && result) {
      details.setAttribute("open", "");
    }
  }

  function checkHost(autoStart) {
    window.GBWizardHost.check(state, autoStart, startProvisioning, say);
  }

  /* -- component template and rendering ------------------------------- */

  function componentTemplate(component) {
    return '<div class="install-step-item" data-comp="' + esc(component.id) + '">' +
      '<span class="install-step-icon" aria-hidden="true"></span>' +
      '<span class="install-step-name">' + esc(component.label) + "</span>" +
      '<span class="install-step-dur"></span></div>';
  }

  function renderComponents() {
    var list = document.getElementById("componentList");
    if (!list) { return; }
    list.innerHTML = (state.context.components || []).map(componentTemplate).join("");
    (state.provisioning.components || []).forEach(updateComponent);
    updateInstallProgress();
    renderLog();
  }

  function plannedComponents() {
    return (state.context.components || []).filter(function (c) {
      return draft.model.mode === "local" || c.id !== "llm";
    });
  }

  function updateInstallProgress() {
    var planned = plannedComponents();
    var ready = 0;
    var failed = 0;
    planned.forEach(function (c) {
      var entry = null;
      for (var i = 0; i < state.provisioning.components.length; i += 1) {
        if (state.provisioning.components[i].id === c.id) { entry = state.provisioning.components[i]; }
      }
      if (entry && entry.state === "ready") { ready += 1; }
      if (entry && entry.state === "failed") { failed += 1; }
    });
    var percent = planned.length ? Math.round((ready / planned.length) * 100) : 0;
    if (state.provisioning.ready) { percent = 100; }
    var label = document.getElementById("installPercent");
    var fill = document.getElementById("installFill");
    var status = document.getElementById("installStatus");
    if (label) { label.textContent = percent + "%"; }
    if (fill) { fill.style.width = percent + "%"; }
    var stepCount = document.getElementById("installStepCount");
    if (stepCount) { stepCount.textContent = t("install.steps-of", { ready: ready, total: planned.length }); }
    if (status) {
      if (state.provisioning.failed || failed) {
        status.textContent = t("install.stopped", { reason: state.provisioning.failed || t("nav.unknown") });
      } else if (state.provisioning.ready) {
        status.textContent = t("install.all-ready", { count: planned.length });
      } else if (state.provisioning.running) {
        status.textContent = t("install.installing", { ready: ready, total: planned.length });
      } else {
        status.textContent = t("install.waiting");
      }
    }
  }

  function updateComponent(event) {
    var node = document.querySelector('[data-comp="' + event.id + '"]');
    var entry = null;
    for (var i = 0; i < state.provisioning.components.length; i += 1) {
      if (state.provisioning.components[i].id === event.id) { entry = state.provisioning.components[i]; }
    }
    if (!entry) {
      entry = { id: event.id, state: "pending", message: "", elapsed_ms: 0 };
      state.provisioning.components.push(entry);
    }
    var changed = entry.state !== event.state;
    entry.state = event.state;
    entry.message = event.message || "";
    entry.elapsed_ms = event.elapsed_ms || 0;
    if (changed) {
      logLines.push("[" + event.id + "] " + event.state + " — " + (event.message || event.state));
      if (logLines.length > 200) { logLines.splice(0, logLines.length - 200); }
    }
    /* Track per-component start time for accurate durations. */
    if (event.state === "starting" && !compStartTimes[event.id]) {
      compStartTimes[event.id] = Date.now();
    }
    if (!node) { return; }
    var icon = node.querySelector(".install-step-icon");
    var dur = node.querySelector(".install-step-dur");
    if (event.state === "starting") {
      icon.innerHTML = '<span class="install-spinner"></span>';
      node.className = "install-step-item current";
      if (dur) { dur.textContent = "0s"; }
    } else if (event.state === "ready") {
      var elapsed = compStartTimes[event.id] ? Date.now() - compStartTimes[event.id] : event.elapsed_ms;
      node.className = "install-step-item done";
      if (dur) { dur.innerHTML = formatDuration(elapsed) + ' <span class="install-step-check">✓</span>'; }
      icon.textContent = "";
    } else if (event.state === "failed") {
      node.className = "install-step-item done";
      icon.textContent = "✗";
      icon.style.color = "#b3352a";
      if (dur) { dur.textContent = ""; }
    }
    updateInstallProgress();
  }

  function renderLog() {
    var pre = document.getElementById("provisionLog");
    if (pre) { pre.textContent = logLines.length ? logLines.slice(-40).join("\n") : t("install.no-output"); }
    var count = document.getElementById("logLineCount");
    if (count) { count.textContent = logLines.length + " " + t("install.lines"); }
  }

  /* -- provisioning lifecycle ----------------------------------------- */

  function setProvisionUi(running) {
    var cancel = document.getElementById("cancelSetupBtn");
    if (cancel) { cancel.disabled = !running; }
    if (running) {
      state.provisioning.running = true;
      state.provisioning.failed = null;
      state.provisioning.startedAt = Date.now();
      state.provisioning.components = [];
      compStartTimes = {};
    }
    syncStartButton();
    renderFooter();
  }

  function handleDone() {
    if (state.provisioning.ready) { return; }
    state.provisioning.running = false;
    state.provisioning.ready = true;
    state.provisioning.failed = null;
    logLines.push(t("install.stack-ready"));
    renderLog();
    markAllReady();
    setProvisionUi(false);
    var start = document.getElementById("startSetupBtn");
    if (start) { start.textContent = t("install.complete"); start.disabled = true; }
    updateInstallProgress();
    renderFooter();
    if (state.step === 7) { goTo(8); }
  }

  function pollStatus() {
    B.status().then(function (s) {
      if (!s) {
        if (state.provisioning.running) { setTimeout(pollStatus, 700); }
        return;
      }
      if (s.failed && !s.ready) { showProvisionError(s.failed); return; }
      (s.components || []).forEach(function (ev) {
        if (ev.id !== "__done__") { updateComponent(ev); }
      });
      renderLog();
      if (s.ready) { handleDone(); return; }
      if (state.provisioning.running) { setTimeout(pollStatus, 500); }
    }).catch(function () {
      if (state.provisioning.running) { setTimeout(pollStatus, 900); }
    });
  }

  function startProvisioning() {
    if (state.provisioning.running || state.provisioning.ready) { return; }
    if (hostLocked()) {
      if (window.GBWizardHost && window.GBWizardHost.windowsAgentBlocked(state.host.result)) {
        say(t("host.agent-wsl-say"));
      } else { say(t("host.blocked-say")); }
      return;
    }
    setProvisionUi(true);
    renderComponents();
    pollStatus();
    B.provision({
      model: draft.model, workspace: draft.workspace,
      services: draft.services, display_name: draft.display_name,
      skip_all_agentic_work: draft.skip_all_agentic_work === true
    }).then(function () { handleDone(); }).catch(function (e) {
      state.provisioning.running = false;
      state.provisioning.failed = String(e && e.message ? e.message : e);
      showProvisionError(state.provisioning.failed);
    });
  }

  function showProvisionError(message) {
    var box = document.getElementById("provisionError");
    if (box) {
      box.innerHTML = esc(message) +
        ' <button type="button" class="btn btn-secondary btn-sm" id="retryComponentBtn">' +
        esc(t("install.retry")) + "</button>";
      var retry = document.getElementById("retryComponentBtn");
      if (retry) { retry.addEventListener("click", startProvisioning); }
    }
    setProvisionUi(false);
  }

  function markAllReady() {
    (state.context.components || []).forEach(function (c) {
      updateComponent({ id: c.id, state: "ready", message: t("install.ready"), elapsed_ms: 0 });
    });
    updateInstallProgress();
  }

  function confirmReset() {
    openModal({
      title: t("nav.reset-title"),
      text: t("nav.reset-text", { path: state.context.stack_path }),
      confirmLabel: t("nav.reset-confirm"),
      onConfirm: function () {
        // #2689 — the typed folder name used to be the whole "confirmation".
        // The native dialog now mints a single-use receipt bound to this window;
        // without it the backend refuses to touch the stack.
        B.requestResetStackConfirmation().then(function (token) {
          return B.resetStack(token);
        }).then(function () {
          state.provisioning.ready = false;
          state.provisioning.components = [];
          render(); say(t("nav.reset-done"));
        }).catch(function (err) {
          say(String(err));
        });
      }
    });
  }

  App.bindInstall = bindInstall;
  App.hostLocked = hostLocked;
  App.syncStartButton = syncStartButton;
  App.renderHostCheck = renderHostCheck;
  App.checkHost = checkHost;
  App.componentTemplate = componentTemplate;
  App.renderComponents = renderComponents;
  App.plannedComponents = plannedComponents;
  App.updateInstallProgress = updateInstallProgress;
  App.updateComponent = updateComponent;
  App.renderLog = renderLog;
  App.setProvisionUi = setProvisionUi;
  App.handleDone = handleDone;
  App.pollStatus = pollStatus;
  App.startProvisioning = startProvisioning;
  App.showProvisionError = showProvisionError;
  App.markAllReady = markAllReady;
  App.confirmReset = confirmReset;
})();
