/* First-run wizard — provisioning, component progress and host prerequisites. */
(function () {
  "use strict";

  var App = window.GBWizardApp;
  var B = App.B, esc = App.esc, t = App.t;
  var state = App.state, draft = App.draft, logLines = App.logLines;

  /* Forwarders resolve the functions defined in sibling modules at call time. */

  function render() { return App.render(); }
  function renderFooter() { return App.renderFooter(); }
  function goTo(step) { return App.goTo(step); }
  function openModal(options) { return App.openModal(options); }
  function say(text) { return App.say(text); }
  function persistDraft() { return App.persistDraft(); }

  /* The install step owns the installer: it checks the host first, reports
     every component and keeps the action buttons together. */
  function bindInstall() {
    var start = document.getElementById("startSetupBtn");
    if (start) { start.addEventListener("click", startProvisioning); }
    var cancel = document.getElementById("cancelSetupBtn");
    if (cancel) { cancel.addEventListener("click", function () { B.cancel(); }); }
    var logs = document.getElementById("openLogsBtn");
    if (logs) { logs.addEventListener("click", function () { B.openFolder("logs"); }); }
    var reset = document.getElementById("resetStackBtn");
    if (reset) { reset.addEventListener("click", confirmReset); }
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
        // Explicit Chat/Drive-only choice: bypass WSL2/agent requirements and
        // persist so re-runs remember (#1937).
        draft.skip_all_agentic_work = true;
        persistDraft();
        renderHostCheck();
        if (!hostLocked() && !state.provisioning.running && !state.provisioning.ready) {
          startProvisioning();
        }
      });
    }
    // #1961 — desktop-local opt-in. Checked = agent mode requested (WSL2 must
    // be ready); unchecked = Chat/Drive only. Persisted like the skip button.
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

  /* ---------------- host prerequisites (#1742) ---------------- */

  function hostLocked() { return window.GBWizardHost.locked(state); }
  function syncStartButton() { window.GBWizardHost.syncStartButton(state); }
  function renderHostCheck() { window.GBWizardHost.render(state); }
  function checkHost(autoStart) {
    window.GBWizardHost.check(state, autoStart, startProvisioning, say);
  }

  function componentTemplate(component) {
    return '<div class="component" data-comp="' + esc(component.id) + '">' +
      '<span class="dot" aria-hidden="true"></span>' +
      '<span><span class="card-title" style="font-size:13px">' + esc(component.label) + "</span>" +
      '<span class="msg" style="display:block">' + esc(t("install.pending")) + '</span></span>' +
      '<span class="dur"></span></div>';
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
    return (state.context.components || []).filter(function (component) {
      return draft.model.mode === "local" || component.id !== "llm";
    });
  }

  function updateInstallProgress() {
    var planned = plannedComponents();
    var ready = 0;
    var failed = 0;
    planned.forEach(function (component) {
      var entry = null;
      for (var i = 0; i < state.provisioning.components.length; i += 1) {
        if (state.provisioning.components[i].id === component.id) { entry = state.provisioning.components[i]; }
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
    if (!node) { return; }
    var dot = node.querySelector(".dot");
    dot.className = "dot " + (event.state === "starting" ? "run" : event.state === "ready" ? "ok" : event.state === "failed" ? "err" : "");
    node.querySelector(".msg").textContent = event.message || event.state;
    node.querySelector(".dur").textContent = event.elapsed_ms ? (event.elapsed_ms / 1000).toFixed(1) + "s" : "";
    updateInstallProgress();
  }

  function renderLog() {
    var pre = document.getElementById("provisionLog");
    if (pre) { pre.textContent = logLines.length ? logLines.slice(-40).join("\n") : t("install.no-output"); }
  }

  function setProvisionUi(running) {
    var cancel = document.getElementById("cancelSetupBtn");
    if (cancel) { cancel.disabled = !running; }
    if (running) {
      state.provisioning.running = true;
      state.provisioning.failed = null;
      state.provisioning.startedAt = Date.now();
      state.provisioning.components = [];
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
    B.status().then(function (status) {
      // Keep polling while this page believes setup is running, even if the
      // first snapshot raced the child and still reported `running: false`.
      if (!status) {
        if (state.provisioning.running) { setTimeout(pollStatus, 700); }
        return;
      }
      if (status.failed && !status.ready) {
        showProvisionError(status.failed);
        return;
      }
      (status.components || []).forEach(function (event) {
        if (event.id !== "__done__") { updateComponent(event); }
      });
      renderLog();
      if (status.ready) { handleDone(); return; }
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
      } else {
        say(t("host.blocked-say"));
      }
      return;
    }
    setProvisionUi(true);
    renderComponents();
    var request = {
      model: draft.model,
      workspace: draft.workspace,
      services: draft.services,
      display_name: draft.display_name,
      skip_all_agentic_work: draft.skip_all_agentic_work === true
    };
    pollStatus();
    B.provision(request).then(function () {
      handleDone();
    }).catch(function (e) {
      state.provisioning.running = false;
      state.provisioning.failed = String(e && e.message ? e.message : e);
      showProvisionError(state.provisioning.failed);
    });
  }

  function showProvisionError(message) {
    var box = document.getElementById("provisionError");
    if (box) {
      box.innerHTML = esc(message) +
        ' <button type="button" class="btn btn-secondary btn-sm" id="retryComponentBtn">' + esc(t("install.retry")) + '</button>';
      var retry = document.getElementById("retryComponentBtn");
      if (retry) { retry.addEventListener("click", function () { startProvisioning(); }); }
    }
    setProvisionUi(false);
  }

  function markAllReady() {
    (state.context.components || []).forEach(function (component) {
      updateComponent({ id: component.id, state: "ready", message: t("install.ready"), elapsed_ms: 0 });
    });
    updateInstallProgress();
  }

  function confirmReset() {
    openModal({
      title: t("nav.reset-title"),
      text: t("nav.reset-text", { path: state.context.stack_path }),
      fieldLabel: t("nav.reset-field"),
      confirmLabel: t("nav.reset-confirm"),
      onConfirm: function (value) {
        B.resetStack(value).then(function () {
          state.provisioning.ready = false;
          state.provisioning.components = [];
          render();
          say(t("nav.reset-done"));
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
