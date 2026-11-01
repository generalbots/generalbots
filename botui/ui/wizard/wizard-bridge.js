/*
 * Bridge between the static wizard and the desktop shell (Tauri commands in
 * botapp/src/desktop/wizard.rs).
 *
 * The page must render with no botserver, no network and no Tauri runtime, so
 * every command has a deterministic in-memory fallback. The fallback is also
 * what the headless acceptance check (selftest.html) exercises.
 */
(function () {
  "use strict";

  var tauri = window.__TAURI__ || null;
  var internals = window.__TAURI_INTERNALS__ || null;
  var isDesktop = !!(tauri && tauri.core && tauri.core.invoke) || !!internals;

  function invoke(cmd, args) {
    if (tauri && tauri.core && tauri.core.invoke) {
      return tauri.core.invoke(cmd, args || {});
    }
    if (internals && internals.invoke) {
      return internals.invoke(cmd, args || {});
    }
    return Promise.reject(new Error("Tauri runtime is not available"));
  }

  function listen(event, handler) {
    if (tauri && tauri.event && tauri.event.listen) {
      return tauri.event.listen(event, function (e) { handler(e.payload); });
    }
    return Promise.resolve(function () {});
  }

  function emitLocal(event, payload) {
    (listeners[event] || []).forEach(function (fn) { fn(payload); });
  }

  var listeners = {};

  /* ---------------- in-browser harness (no Tauri) ---------------- */

  var now = function () { return Date.now(); };
  var mock = {
    startedAt: 0,
    timer: null,
    cancelled: false,
    completed: [],
    draftKey: "gb_wizard_draft_v1",
    context: function () {
      var stack = "/home/user/.local/share/generalbots/stack";
      return {
        desktop_mode: true,
        first_run: true,
        desktop_local: true,
        stack_path: stack,
        workspace_path: stack + "/workspace",
        log_path: stack + "/logs",
        platform: navigator.platform || "unknown",
        api_base: "http://127.0.0.1:8080",
        redirect_uri: "http://127.0.0.1:8080/api/integrations/oauth/{provider}/callback",
        resume_step: null,
        completed_components: [],
        components: [
          { id: "vault", label: "Local vault", optional: false },
          { id: "tables", label: "Database", optional: false },
          { id: "cache", label: "Cache", optional: false },
          { id: "drive", label: "Object store", optional: false },
          { id: "vector_db", label: "Vector store", optional: false },
          { id: "llm", label: "Local model runtime", optional: true },
          { id: "botserver", label: "Assistant service", optional: false }
        ]
      };
    },
    saveDraft: function (draft) {
      try { localStorage.setItem(mock.draftKey, JSON.stringify(draft)); } catch (e) {}
      return Promise.resolve();
    },
    loadDraft: function () {
      try {
        var raw = localStorage.getItem(mock.draftKey);
        return Promise.resolve(raw ? JSON.parse(raw) : null);
      } catch (e) { return Promise.resolve(null); }
    },
    checkWorkspace: function (path) {
      var writable = !!path && path.indexOf("readonly") === -1;
      return Promise.resolve({
        writable: writable,
        free_bytes: 64 * 1024 * 1024 * 1024,
        reason: writable ? "" : "The folder is not writable. Choose another location."
      });
    },
    pickFolder: function () { return Promise.resolve("/home/user/Documents/General Bots"); },
    testModel: function (req) {
      if (req.mode === "local") {
        return Promise.resolve({ ok: true, message: "Bundled model is ready (nothing leaves this machine).", latency_ms: 12 });
      }
      if (!req.api_key) {
        return Promise.resolve({ ok: false, message: "Enter the key for this provider.", latency_ms: 0 });
      }
      if (req.api_key === "invalid") {
        return Promise.resolve({ ok: false, message: "The provider rejected this key (401).", latency_ms: 180 });
      }
      return Promise.resolve({ ok: true, message: "Provider answered and accepted the key.", latency_ms: 240 });
    },
    provision: function () {
      mock.cancelled = false;
      mock.done = false;
      mock.startedAt = now();
      mock.completed = [];
      var comps = mock.context().components;
      var i = 0;
      function step() {
        if (mock.cancelled) { return; }
        if (i > 0) {
          mock.completed.push(comps[i - 1].id);
          emitLocal("wizard://progress", {
            id: comps[i - 1].id,
            state: "ready",
            message: "Ready",
            elapsed_ms: now() - mock.startedAt
          });
        }
        if (i >= comps.length) {
          mock.done = true;
          emitLocal("wizard://progress", {
            id: "__done__",
            state: "ready",
            message: "Stack is ready",
            elapsed_ms: now() - mock.startedAt
          });
          return;
        }
        var comp = comps[i];
        emitLocal("wizard://progress", {
          id: comp.id,
          state: "starting",
          message: "Starting " + comp.label.toLowerCase() + "…",
          elapsed_ms: now() - mock.startedAt
        });
        i += 1;
        mock.timer = setTimeout(step, 320);
      }
      step();
      return Promise.resolve();
    },
    cancel: function () {
      mock.cancelled = true;
      if (mock.timer) { clearTimeout(mock.timer); }
      return Promise.resolve();
    },
    status: function () {
      var components = (mock.completed || []).map(function (id) {
        return { id: id, state: "ready", message: "Ready", elapsed_ms: 0 };
      });
      return {
        running: !mock.done && !mock.cancelled,
        ready: mock.done === true,
        failed: mock.cancelled ? "cancelled" : null,
        components: components
      };
    },
    resetStack: function () { mock.completed = []; return Promise.resolve(); },
    openFolder: function () { return Promise.resolve(); },
    openUrl: function (url) {
      if (isDesktop) { return invoke("wizard_open_url", { url: url }); }
      window.open(url, "_blank", "noopener");
      return Promise.resolve();
    },
    finish: function () { return Promise.resolve({ ok: true }); }
  };

  /* ---------------- public API ---------------- */

  var api = {
    isDesktop: isDesktop,
    mock: mock,

    context: function () {
      return isDesktop ? invoke("wizard_context") : Promise.resolve(mock.context());
    },

    saveDraft: function (draft) {
      return isDesktop ? invoke("wizard_save_draft", { draft: draft }) : mock.saveDraft(draft);
    },

    loadDraft: function () {
      return isDesktop ? invoke("wizard_load_draft") : mock.loadDraft();
    },

    checkWorkspace: function (path) {
      return isDesktop ? invoke("wizard_check_workspace", { path: path }) : mock.checkWorkspace(path);
    },

    pickFolder: function () {
      return isDesktop ? invoke("wizard_pick_folder") : mock.pickFolder();
    },

    testModel: function (req) {
      return isDesktop ? invoke("wizard_test_model", { request: req }) : mock.testModel(req);
    },

    provision: function (req) {
      return isDesktop ? invoke("wizard_provision", { request: req }) : mock.provision(req);
    },

    cancel: function () {
      return isDesktop ? invoke("wizard_cancel_provision") : mock.cancel();
    },

    status: function () {
      return isDesktop ? invoke("wizard_provision_status") : Promise.resolve(mock.status());
    },

    resetStack: function (confirmation) {
      return isDesktop ? invoke("wizard_reset_stack", { confirmation: confirmation }) : mock.resetStack();
    },

    openFolder: function (kind) {
      return isDesktop ? invoke("wizard_open_folder", { kind: kind }) : mock.openFolder(kind);
    },

    openUrl: function (url) { return mock.openUrl(url); },

    finish: function (payload) {
      return isDesktop ? invoke("wizard_finish", { request: payload }) : mock.finish(payload);
    },

    onProgress: function (handler) {
      if (isDesktop) {
        return listen("wizard://progress", handler);
      }
      listeners["wizard://progress"] = listeners["wizard://progress"] || [];
      listeners["wizard://progress"].push(handler);
      return Promise.resolve(function () {
        listeners["wizard://progress"] = (listeners["wizard://progress"] || []).filter(function (fn) {
          return fn !== handler;
        });
      });
    }
  };

  window.GBWizardBridge = api;
})();
