/* First-run wizard controller: navigation, validation, provisioning, handoff. */
(function () {
  "use strict";

  var W = window.GBWizard;
  var S = window.GBWizardSteps;
  var B = window.GBWizardBridge;
  var esc = S.esc;

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
        '<span class="name">' + esc(step.name) + "</span></li>";
    }).join("");
  }

  function renderFooter() {
    var last = state.step === state.totalSteps;
    el.back.disabled = state.step === 1 || state.provisioning.running;
    el.next.textContent = last ? "Open the assistant" : "Next";
    el.next.disabled = !canAdvance();
    el.progress.style.width = Math.round((state.step / state.totalSteps) * 100) + "%";
    el.footLabel.textContent = "Step " + state.step + " of " + state.totalSteps;
    el.counter.textContent = "STEP " + state.step + " OF " + state.totalSteps;
  }

  function canAdvance() {
    if (state.provisioning.running) { return false; }
    if (state.step === 5 && !state.provisioning.ready && !state.skipped) { return false; }
    if (state.step === 6) { return state.provisioning.ready || state.skipped || !ctx.first_run; }
    return S.validate(state.step, draft).ok;
  }

  function render() {
    renderRail();
    renderFooter();
    el.heading.textContent = S.heading(state.step);
    el.body.innerHTML = S.render(state.step, draft, ctx);
    if (el.stepError) { el.stepError.hidden = true; el.stepError.textContent = ""; }
    if (state.step === 5) { renderComponents(); }
    bindStep();
    el.heading.focus();
    say("Step " + state.step + " of " + state.totalSteps + ": " + S.heading(state.step));
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
    if (state.step === 6) { bindFinish(); }
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
        document.getElementById("modelTestStatus").textContent = "Not tested yet.";
        renderFooter();
      });
    }
    if (endpoint) { endpoint.addEventListener("input", function () { draft.model.endpoint = endpoint.value.trim(); draft.model.tested = false; renderFooter(); }); }
    if (modelName) { modelName.addEventListener("input", function () { draft.model.model = modelName.value.trim(); draft.model.tested = false; renderFooter(); }); }
    if (key) { key.addEventListener("input", function () { draft.model.api_key = key.value; draft.model.tested = false; renderFooter(); }); }
    if (test) {
      test.addEventListener("click", function () {
        var status = document.getElementById("modelTestStatus");
        status.textContent = "Testing…";
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
    if (!path) { status.textContent = "Choose a folder."; return; }
    status.textContent = "Checking the folder…";
    B.checkWorkspace(path).then(function (result) {
      draft.workspace.blocked = !result.writable;
      draft.workspace.block_reason = result.reason || "";
      status.textContent = result.writable ? "Folder is writable." : (result.reason || "Folder is not writable.");
      status.className = "hint " + (result.writable ? "ok" : "err");
      if (free) { free.textContent = "Free space at this location: " + W.formatBytes(result.free_bytes) + "."; }
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
          navigator.clipboard.writeText(value).then(function () { say("Redirect URI copied."); });
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
    var start = document.getElementById("startSetupBtn");
    if (start) { start.addEventListener("click", startProvisioning); }
    var cancel = document.getElementById("cancelSetupBtn");
    if (cancel) { cancel.addEventListener("click", function () { B.cancel(); }); }
    var logs = document.getElementById("openLogsBtn");
    if (logs) { logs.addEventListener("click", function () { B.openFolder("logs"); }); }
    var reset = document.getElementById("resetStackBtn");
    if (reset) { reset.addEventListener("click", confirmReset); }
    if (state.provisioning.running) { setProvisionUi(true); }
    if (state.provisioning.ready) { markAllReady(); }
  }

  function bindFinish() {
    var email = document.getElementById("adminEmail");
    var password = document.getElementById("adminPassword");
    if (email) {
      email.addEventListener("input", function () { draft.admin.email = email.value.trim(); });
    }
    var copy = document.getElementById("copyAdminBtn");
    if (copy) {
      copy.addEventListener("click", function () {
        var value = "email: " + draft.admin.email + "\npassword: " + draft.admin.password;
        if (navigator.clipboard) {
          navigator.clipboard.writeText(value).then(function () { say("Administrator credentials copied."); });
        }
      });
    }
    var regenerate = document.getElementById("regenerateAdminBtn");
    if (regenerate) {
      regenerate.addEventListener("click", function () {
        draft.admin.password = generatePassword();
        if (password) { password.value = draft.admin.password; }
        say("A new administrator password was generated.");
      });
    }
    var status = document.getElementById("finishStatus");
    if (status && !state.provisioning.ready && !state.skipped) {
      status.textContent = "Start setup on the previous step to enable this action.";
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
      '<span class="msg" style="display:block">Pending</span></span>' +
      '<span class="dur"></span></div>';
  }

  function renderComponents() {
    var list = document.getElementById("componentList");
    if (!list) { return; }
    list.innerHTML = (ctx.components || []).map(componentTemplate).join("");
    (state.provisioning.components || []).forEach(updateComponent);
    renderLog();
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
    entry.state = event.state;
    entry.message = event.message || "";
    entry.elapsed_ms = event.elapsed_ms || 0;
    if (!node) { return; }
    var dot = node.querySelector(".dot");
    dot.className = "dot " + (event.state === "starting" ? "run" : event.state === "ready" ? "ok" : event.state === "failed" ? "err" : "");
    node.querySelector(".msg").textContent = event.message || event.state;
    node.querySelector(".dur").textContent = event.elapsed_ms ? (event.elapsed_ms / 1000).toFixed(1) + "s" : "";
  }

  function renderLog() {
    var pre = document.getElementById("provisionLog");
    if (pre) { pre.textContent = logLines.length ? logLines.slice(-40).join("\n") : "No output yet."; }
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
    logLines.push("Stack is ready");
    renderLog();
    markAllReady();
    setProvisionUi(false);
    var start = document.getElementById("startSetupBtn");
    if (start) { start.textContent = "Setup complete"; start.disabled = true; }
    var intro = document.getElementById("provisionIntro");
    if (intro) { intro.textContent = "All components are ready."; }
    renderFooter();
    if (state.step === 6) { handoff(); }
  }

  function pollStatus() {
    B.status().then(function (status) {
      if (!status) { return; }
      if (status.failed && !status.ready) {
        showProvisionError(status.failed);
        return;
      }
      (status.components || []).forEach(function (event) {
        if (event.id !== "__done__") { updateComponent(event); }
      });
      if (status.ready) { handleDone(); return; }
      if (status.running) { setTimeout(pollStatus, 500); }
    }).catch(function () {
      setTimeout(pollStatus, 1000);
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
        ' <button type="button" class="btn btn-secondary btn-sm" id="retryComponentBtn">Retry</button>';
      var retry = document.getElementById("retryComponentBtn");
      if (retry) { retry.addEventListener("click", function () { startProvisioning(); }); }
    }
    setProvisionUi(false);
  }

  function markAllReady() {
    (ctx.components || []).forEach(function (component) {
      updateComponent({ id: component.id, state: "ready", message: "Ready", elapsed_ms: 0 });
    });
  }

  function confirmReset() {
    openModal({
      title: "Reset the stack?",
      text: "This empties exactly this folder and nothing else:\n" + ctx.stack_path,
      fieldLabel: "Type the folder name to confirm",
      confirmLabel: "Reset stack",
      onConfirm: function (value) {
        B.resetStack(value).then(function () {
          state.provisioning.ready = false;
          state.provisioning.components = [];
          render();
          say("Stack reset.");
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
    el.modalConfirm.textContent = options.confirmLabel || "Confirm";
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
    if (state.step === 5 && !state.provisioning.ready && !state.skipped) {
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
    say("Setup skipped. You can finish it later in Settings.");
  }

  function finish() {
    if (!state.provisioning.ready && !ctx.first_run) { handoff(); return; }
    if (!state.provisioning.ready) { startProvisioning(); return; }
    handoff();
  }

  function handoff() {
    el.next.disabled = true;
    el.next.textContent = "Opening…";
    B.finish({
      display_name: draft.display_name,
      telemetry: draft.telemetry,
      model: draft.model,
      workspace: draft.workspace,
      services: draft.services,
      admin: draft.admin
    }).then(function () {
      say("Opening the assistant.");
    }).catch(function (e) {
      el.next.disabled = false;
      el.next.textContent = "Open the assistant";
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
    banner.innerHTML = '<p class="note warn" style="margin:0">Setup was interrupted at step ' +
      esc(ctx.resume_step) + ' — continue.</p>';
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
    if (event.state === "ready" && event.id === "botserver") { say("Assistant service is ready."); }
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
      el.note.textContent = ctx.desktop_local
        ? "Desktop edition. Nothing is installed until you choose Start setup."
        : "Setup runs locally. Nothing is installed until you choose Start setup.";
      render();
      if (state.resumed) { resumeBanner(); }
      if (ctx.first_run === false) {
        el.skip.hidden = true;
        state.provisioning.ready = true;
      }
    });
  }).catch(function (e) {
    el.body.innerHTML = '<p class="note err">Could not read the setup state: ' + esc(e && e.message ? e.message : e) + "</p>";
    el.root.setAttribute("data-state", "error");
  });
})();
