/* First-run wizard controller: navigation, validation, provisioning, handoff. */
(function () {
  "use strict";

  var W = window.GBWizard;
  var S = window.GBWizardSteps;
  var B = window.GBWizardBridge;
  var esc = S.esc;
  var I = window.WizardI18n;
  var t = function (key, params) { return I.t(key, params); };

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

  var state = W.state;
  var draft = state.draft;
  var ctx = null;
  var logLines = [];
  var modalAction = null;

  function say(text) { el.status.textContent = text; }

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
    if (state.step === 7) { return state.provisioning.ready || state.skipped || !ctx.first_run; }
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
    el.body.innerHTML = S.render(state.step, draft, ctx);
    if (el.stepError) { el.stepError.hidden = true; el.stepError.textContent = ""; }
    if (state.step === 5) { renderComponents(); }
    bindStep();
    el.heading.focus();
    say(t("wizard.step-say", { current: state.step, total: state.totalSteps, title: S.heading(state.step) }));
  }

  function persistDraft() {
    var safe = W.clone(draft);
    safe.model.api_key = "";
    Object.keys(safe.services || {}).forEach(function (id) { safe.services[id].client_secret = ""; });
    B.saveDraft(safe).catch(function () {});
  }

  /* ---------------- step bindings ---------------- */

  function bindStep() {
    if (state.step === 1) {
      var name = document.getElementById("displayName");
      if (name) { name.addEventListener("input", function () { draft.display_name = name.value.trim(); }); }
    }

    if (state.step === 2) {
      Array.prototype.forEach.call(document.querySelectorAll('input[name="modelMode"]'), function (input) {
        input.addEventListener("change", function () {
          draft.model.mode = input.value;
          if (input.value === "free") {
            draft.model.provider = "kilo";
            draft.model.endpoint = W.defaultModelEndpoint("kilo");
            draft.model.model = W.defaultModelName("kilo");
            draft.model.api_key = "";
          } else if (input.value === "own-key" || input.value === "hosted") {
            if (!W.defaultModelEndpoint(draft.model.provider)) {
              draft.model.provider = "openai";
            }
            draft.model.endpoint = W.defaultModelEndpoint(draft.model.provider);
            draft.model.model = W.defaultModelName(draft.model.provider);
          }
          draft.model.tested = input.value === "local";
          document.getElementById("modelDetail").innerHTML = S.modelDetail(draft);
          bindModelDetail();
          renderFooter();
          persistDraft();
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
    var toggle = document.getElementById("telemetryToggle");
    if (toggle) { toggle.addEventListener("change", function () { draft.telemetry = toggle.checked; persistDraft(); }); }
    var reveal = document.getElementById("revealVaultBtn");
    if (reveal) { reveal.addEventListener("click", function () { B.openFolder("vault"); }); }
  }

  /* The install step owns the installer: it starts on arrival (Borland
     style), reports every component and keeps the action buttons together. */
  function bindInstall() {
    var start = document.getElementById("startSetupBtn");
    if (start) { start.addEventListener("click", startProvisioning); }
    var cancel = document.getElementById("cancelSetupBtn");
    if (cancel) { cancel.addEventListener("click", function () { B.cancel(); }); }
    var logs = document.getElementById("openLogsBtn");
    if (logs) { logs.addEventListener("click", function () { B.openFolder("logs"); }); }
    var reset = document.getElementById("resetStackBtn");
    if (reset) { reset.addEventListener("click", confirmReset); }
    if (state.provisioning.running) {
      setProvisionUi(true);
      renderComponents();
    } else if (state.provisioning.ready) {
      markAllReady();
      updateInstallProgress();
    } else if (!state.skipped) {
      startProvisioning();
    }
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
  }

  function generatePassword() {
    var alphabet = "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789!@#$%&*";
    var bytes = new Uint8Array(18);
    if (window.crypto && window.crypto.getRandomValues) {
      window.crypto.getRandomValues(bytes);
    } else {
      for (var i = 0; i < bytes.length; i += 1) { bytes[i] = Math.floor(Math.random() * 256); }
    }
    var out = "";
    for (var j = 0; j < bytes.length; j += 1) {
      out += alphabet.charAt(bytes[j] % alphabet.length);
    }
    return out;
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

  /* ---------------- provisioning ---------------- */

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
    list.innerHTML = (ctx.components || []).map(componentTemplate).join("");
    (state.provisioning.components || []).forEach(updateComponent);
    updateInstallProgress();
    renderLog();
  }

  function plannedComponents() {
    return (ctx.components || []).filter(function (component) {
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
    var start = document.getElementById("startSetupBtn");
    var cancel = document.getElementById("cancelSetupBtn");
    if (start) { start.disabled = running || state.provisioning.ready; }
    if (cancel) { cancel.disabled = !running; }
    if (running) {
      state.provisioning.running = true;
      state.provisioning.failed = null;
      state.provisioning.startedAt = Date.now();
      state.provisioning.components = [];
    }
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
    if (state.step === 6) { goTo(7); }
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
    setProvisionUi(true);
    renderComponents();
    var request = {
      model: draft.model,
      workspace: draft.workspace,
      telemetry: draft.telemetry,
      services: draft.services,
      display_name: draft.display_name
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
    (ctx.components || []).forEach(function (component) {
      updateComponent({ id: component.id, state: "ready", message: t("install.ready"), elapsed_ms: 0 });
    });
    updateInstallProgress();
  }

  function confirmReset() {
    openModal({
      title: t("nav.reset-title"),
      text: t("nav.reset-text", { path: ctx.stack_path }),
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

  /* ---------------- modal ---------------- */

  function openModal(options) {
    el.modalTitle.textContent = options.title;
    el.modalText.textContent = options.text;
    el.modalField.hidden = !options.fieldLabel;
    el.modalFieldLabel.textContent = options.fieldLabel || "";
    el.modalInput.value = "";
    el.modalConfirm.textContent = options.confirmLabel || t("wizard.confirm");
    el.modal.hidden = false;
    modalAction = options.onConfirm;
    if (options.fieldLabel) { el.modalInput.focus(); } else { el.modalConfirm.focus(); }
  }

  function closeModal() {
    el.modal.hidden = true;
    modalAction = null;
    el.heading.focus();
  }

  el.modalCancel.addEventListener("click", closeModal);
  el.modalConfirm.addEventListener("click", function () {
    var action = modalAction;
    var value = el.modalInput.value;
    closeModal();
    if (action) { action(value); }
  });
  el.modal.addEventListener("keydown", function (event) {
    if (event.key === "Escape") { event.stopPropagation(); closeModal(); }
  });

  /* ---------------- navigation ---------------- */

  function goTo(step) {
    if (step < 1 || step > state.totalSteps) { return; }
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
      startProvisioning();
      return;
    }
    if (state.step === 6 && !state.provisioning.ready && !state.skipped) {
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
    if (!state.provisioning.ready && !ctx.first_run) { handoff(); return; }
    if (!state.provisioning.ready) { startProvisioning(); return; }
    handoff();
  }

  function handoff() {
    el.next.disabled = true;
    el.next.textContent = t("finish.opening");
    B.finish({
      display_name: draft.display_name,
      telemetry: draft.telemetry,
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

  el.next.addEventListener("click", next);
  el.back.addEventListener("click", function () { goTo(state.step - 1); });
  el.skip.addEventListener("click", skip);

  document.addEventListener("keydown", function (event) {
    if (!el.modal.hidden) { return; }
    if (event.altKey && event.key === "ArrowRight") { event.preventDefault(); next(); }
    if (event.altKey && event.key === "ArrowLeft") { event.preventDefault(); goTo(state.step - 1); }
    var tag = (event.target && event.target.tagName) || "";
    var typing = tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || tag === "BUTTON";
    if (event.key === "Enter" && !typing) { event.preventDefault(); next(); }
  });

  /* ---------------- init ---------------- */

  function resumeBanner() {
    if (!ctx.resume_step || ctx.resume_step <= 1) { return; }
    var banner = document.createElement("div");
    banner.className = "card";
    banner.style.marginBottom = "14px";
    banner.innerHTML = '<p class="note warn" style="margin:0">' + esc(t("nav.resume", { step: ctx.resume_step })) + '</p>';
    el.body.insertBefore(banner, el.body.firstChild);
  }

  B.onProgress(function (event) {
    if (event.id === "__done__") {
      handleDone();
      return;
    }
    if (event.state === "failed") {
      state.provisioning.running = false;
      state.provisioning.failed = event.message;
      logLines.push("[" + event.id + "] failed: " + event.message);
      showProvisionError(event.message);
      return;
    }
    logLines.push("[" + event.id + "] " + event.state + " — " + event.message);
    updateComponent(event);
    renderLog();
    if (event.state === "ready" && event.id === "botserver") { say(t("nav.assistant-ready")); }
  });

  I.populate(el.lang);
  window.addEventListener("wizard:localeChanged", function () {
    if (!ctx) { return; }
    render();
  });

  B.context().then(function (context) {
    ctx = context;
    state.context = ctx;
    if (!draft.workspace.path) { draft.workspace.path = ctx.workspace_path; }
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
      state.resumed = !!ctx.resume_step;
      state.provisioning.components = (ctx.completed_components || []).map(function (id) {
        return { id: id, state: "ready", message: "Ready", elapsed_ms: 0 };
      });
      ensureAdmin();
      state.step = ctx.resume_step || 1;
      el.root.setAttribute("data-state", "ready");
      if (el.note) {
        el.note.textContent = ctx.desktop_local
          ? t("nav.rail-desktop")
          : t("nav.rail-local");
      }
      render();
      if (state.resumed) { resumeBanner(); }
      if (ctx.first_run === false) {
        el.skip.hidden = true;
        state.provisioning.ready = true;
      }
    });
  }).catch(function (e) {
    el.body.innerHTML = '<p class="note err">' + esc(t("nav.state-error", { error: e && e.message ? e.message : e })) + "</p>";
    el.root.setAttribute("data-state", "error");
  });
})();
