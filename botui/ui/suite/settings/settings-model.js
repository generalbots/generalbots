"use strict";

// Settings → Model (#2568). The classic Settings shell exposes the same model
// surface the assistant shell has: where the bot thinks (this machine, your own
// OpenAI-compatible endpoint, or the free hosted model) plus test/save. Reads
// and writes the /api/llm/config* routes from the botsettings crate; every
// route is admin-gated server-side, so a 401/403 renders the read-only notice.
(function () {
  var state = { cfg: null, modes: [], loaded: false, busy: false };

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }

  function token() {
    if (typeof window.getGBAccessToken === "function") {
      var value = window.getGBAccessToken();
      if (value) return value;
    }
    return (
      localStorage.getItem("gb-access-token") ||
      sessionStorage.getItem("gb-access-token") ||
      localStorage.getItem("management_token") ||
      ""
    );
  }

  function json(path, options) {
    var headers = { "Content-Type": "application/json" };
    var value = token();
    if (value) headers.Authorization = "Bearer " + value;
    return fetch(path, Object.assign({ headers: headers }, options || {})).then(function (response) {
      return response.json().catch(function () {
        return {};
      }).then(function (body) {
        if (!response.ok) {
          var err = new Error(body.error || "HTTP " + response.status);
          err.status = response.status;
          throw err;
        }
        return body;
      });
    });
  }

  function esc(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function say(message, isError) {
    var node = document.getElementById("mdl-status");
    if (!node) return;
    node.setAttribute("role", isError ? "alert" : "status");
    node.setAttribute("aria-live", isError ? "assertive" : "polite");
    node.textContent = message || "";
  }

  function shortName(model) {
    if (!model) return "—";
    var name = String(model);
    var slash = name.lastIndexOf("/");
    if (slash >= 0) name = name.substring(slash + 1);
    var colon = name.indexOf(":");
    if (colon > 0) name = name.substring(0, colon);
    return name || String(model);
  }

  function scopeText(scope, host) {
    if (scope === "machine") return t("mdl-scope-machine", "On this computer");
    if (scope === "network") {
      return t("mdl-scope-network", "Your organization's server (%h)").replace("%h", host || "");
    }
    if (scope === "external") {
      return t("mdl-scope-external", "An external host (%h)").replace("%h", host || "");
    }
    return "";
  }

  function renderCurrent() {
    var cfg = state.cfg;
    var name = document.getElementById("mdl-model-name");
    var scope = document.getElementById("mdl-scope");
    var host = document.getElementById("mdl-host");
    var provider = document.getElementById("mdl-provider");
    var warn = document.getElementById("mdl-inconsistent");
    if (!cfg) {
      if (name) name.textContent = "—";
      if (scope) scope.textContent = "";
      if (host) host.textContent = "";
      return;
    }
    if (name) name.textContent = shortName(cfg.model);
    if (scope) scope.textContent = scopeText(cfg.endpoint_scope, cfg.endpoint_host);
    if (host) host.textContent = cfg.provider ? cfg.provider : "";
    if (provider) provider.textContent = cfg.has_key ? t("mdl-key-stored", "key stored") : "";
    if (warn) warn.hidden = !cfg.inconsistent;
    var restore = document.getElementById("mdl-restore-free");
    if (restore && !restore.dataset.bound) {
      restore.dataset.bound = "1";
      restore.addEventListener("click", function () {
        putConfig({ mode: "free" }, restore);
      });
    }
  }

  function modeLabel(id) {
    if (id === "local") return t("mdl-local", "On this computer");
    if (id === "own_key") return t("mdl-own", "Your server or provider key");
    return t("mdl-free", "Free hosted model");
  }

  function renderModes() {
    var wrap = document.getElementById("mdl-modes");
    if (!wrap) return;
    var current = state.cfg ? state.cfg.mode : "";
    var ids = ["local", "own_key", "free"];
    var html = "";
    for (var i = 0; i < ids.length; i++) {
      var id = ids[i];
      var active = current === id;
      var meta = null;
      for (var j = 0; j < state.modes.length; j++) {
        if (state.modes[j].id === id) meta = state.modes[j];
      }
      var available = !meta || meta.available !== false;
      html += '<button type="button" role="tab" class="mdl-mode' +
        (active ? " mdl-mode-active" : "") + '" data-mode="' + id + '"' +
        ' aria-selected="' + (active ? "true" : "false") + '"' +
        (available ? "" : " disabled") + ">" +
        '<span class="mdl-mode-label">' + esc(modeLabel(id)) + "</span>" +
        (active ? '<span class="mdl-badge">' + esc(t("mdl-active", "In use")) + "</span>" : "") +
        "</button>";
    }
    wrap.innerHTML = html;
    var buttons = wrap.querySelectorAll(".mdl-mode");
    for (var k = 0; k < buttons.length; k++) {
      buttons[k].addEventListener("click", function (event) {
        var mode = event.currentTarget.getAttribute("data-mode");
        renderDetail(mode);
      });
    }
    renderDetail(current || "free");
  }

  function localMeta() {
    for (var i = 0; i < state.modes.length; i++) {
      if (state.modes[i].id === "local") return state.modes[i];
    }
    return null;
  }

  function renderDetail(mode) {
    var detail = document.getElementById("mdl-detail");
    if (!detail) return;
    if (mode === "own_key") return renderOwnKey(detail);
    if (mode === "local") return renderLocal(detail);
    return renderFree(detail);
  }

  function renderLocal(detail) {
    var meta = localMeta();
    var runtime = (state.cfg && state.cfg.local_runtime) || {};
    var note = meta && meta.note ? meta.note : "";
    var active = runtime.active_model
      ? t("mdl-local-active", "Active local model: %m").replace("%m", runtime.active_model)
      : t("mdl-local-hint", "Pick a model on the On-device AI page, then it can be used here.");
    detail.innerHTML = '<p class="mdl-hint">' + esc(active) + "</p>" +
      (note ? '<p class="mdl-muted">' + esc(note) + "</p>" : "") +
      '<button type="button" class="btn-secondary btn-sm" id="mdl-open-local">' +
      esc(t("mdl-local-open", "Open On-device AI")) + "</button>";
    var open = document.getElementById("mdl-open-local");
    if (open) {
      open.addEventListener("click", function () {
        if (typeof window.GBSettingsActivateSection === "function") {
          window.GBSettingsActivateSection("on-device-ai", null);
        }
        if (typeof window.loadOnDeviceAi === "function") window.loadOnDeviceAi();
      });
    }
  }

  function renderOwnKey(detail) {
    var cfg = state.cfg || {};
    var endpoint = cfg.mode === "own_key" && cfg.endpoint_host ? cfg.endpoint_host : "";
    var model = cfg.mode === "own_key" ? cfg.model || "" : "";
    detail.innerHTML = '<div class="mdl-form">' +
      '<label class="mdl-label" for="mdl-endpoint">' + esc(t("mdl-endpoint", "Endpoint URL")) + "</label>" +
      '<input class="mdl-input" id="mdl-endpoint" value="' + esc(endpoint) +
      '" placeholder="https://api.example.com/v1/chat/completions"/>' +
      '<label class="mdl-label" for="mdl-model">' + esc(t("mdl-model", "Model")) + "</label>" +
      '<input class="mdl-input" id="mdl-model" value="' + esc(model) + '" placeholder="gpt-4o-mini"/>' +
      '<label class="mdl-label" for="mdl-key">' + esc(t("mdl-key", "API key")) +
      (cfg.has_key ? ' <small>(' + esc(t("mdl-key-stored", "key stored")) + ")</small>" : "") + "</label>" +
      '<input class="mdl-input" id="mdl-key" type="password" autocomplete="off" placeholder="sk-..."/>' +
      '<div class="mdl-actions">' +
      '<button type="button" class="btn-secondary btn-sm" id="mdl-test">' + esc(t("mdl-test", "Test")) + "</button>" +
      '<button type="button" class="btn-primary btn-sm" id="mdl-save">' + esc(t("mdl-save", "Save")) + "</button>" +
      "</div></div>";
    document.getElementById("mdl-test").addEventListener("click", testEndpoint);
    document.getElementById("mdl-save").addEventListener("click", saveOwnKey);
  }

  function renderFree(detail) {
    var cfg = state.cfg || {};
    var host = cfg.mode === "free" && cfg.endpoint_host ? cfg.endpoint_host : "cloud provider";
    detail.innerHTML = '<p class="mdl-hint">' +
      esc(t("mdl-free-note", "Everything you write and attach is sent to %h.").replace("%h", host)) +
      "</p>" +
      (cfg.mode !== "free"
        ? '<button type="button" class="btn-primary btn-sm" id="mdl-free-save">' +
          esc(t("mdl-save", "Save")) + "</button>"
        : "");
    var save = document.getElementById("mdl-free-save");
    if (save) {
      save.addEventListener("click", function () {
        if (!window.confirm(t("mdl-confirm-external", "This sends your conversations to an external host. Continue?"))) return;
        putConfig({ mode: "free" }, save);
      });
    }
  }

  function testEndpoint() {
    var endpoint = document.getElementById("mdl-endpoint").value.trim();
    var model = document.getElementById("mdl-model").value.trim();
    var key = document.getElementById("mdl-key").value;
    var button = document.getElementById("mdl-test");
    if (button) button.disabled = true;
    say(t("mdl-testing", "Testing…"));
    var body = { endpoint: endpoint, model: model };
    if (key) body.api_key = key;
    json("/api/llm/config/test", { method: "POST", body: JSON.stringify(body) })
      .then(function (data) {
        if (data.reachable && data.model_responded) {
          say(t("mdl-test-ok", "Reachable, latency %ms ms").replace("%ms", String(data.latency_ms || "?")));
        } else {
          say(t("mdl-test-fail", "Failed: %e").replace("%e", data.error || "unreachable"), true);
        }
      })
      .catch(function (error) {
        say(t("mdl-test-fail", "Failed: %e").replace("%e", error.message || String(error)), true);
      })
      .then(function () {
        if (button) button.disabled = false;
      });
  }

  function saveOwnKey() {
    var endpoint = document.getElementById("mdl-endpoint").value.trim();
    var model = document.getElementById("mdl-model").value.trim();
    var key = document.getElementById("mdl-key").value;
    var button = document.getElementById("mdl-save");
    var host = endpoint;
    try {
      host = new URL(endpoint).hostname;
    } catch (e) {
      host = endpoint;
    }
    if (!window.confirm(t("mdl-confirm-external", "What you write and attach will be sent to %h, outside this computer. Continue?").replace("%h", host))) {
      return;
    }
    var body = { mode: "own_key", endpoint: endpoint, model: model };
    if (key) body.api_key = key;
    putConfig(body, button);
    var keyEl = document.getElementById("mdl-key");
    if (keyEl) keyEl.value = "";
  }

  function putConfig(body, button) {
    if (state.busy) return;
    state.busy = true;
    if (button) button.disabled = true;
    say(t("mdl-saving", "Saving…"));
    json("/api/llm/config", { method: "PUT", body: JSON.stringify(body) })
      .then(function (data) {
        state.cfg = null;
        return load().then(function () {
          if (data && data.restart_required) {
            say(t("mdl-restart", "Saved. Restart the app to use the new model."));
          } else if (data && data.provider_swapped) {
            say(t("mdl-swapped", "Ready. The new model is now active."));
          } else {
            say(t("mdl-saved", "Saved. Takes effect from the next message."));
          }
        });
      })
      .catch(function (error) {
        if (error && (error.status === 401 || error.status === 403)) {
          say(t("mdl-admin-only", "Only the administrator can change the model."), true);
        } else {
          say(t("mdl-test-fail", "Failed: %e").replace("%e", error.message || String(error)), true);
        }
      })
      .then(function () {
        state.busy = false;
        if (button) button.disabled = false;
      });
  }

  function load() {
    say(t("mdl-loading", "Loading…"));
    return Promise.all([
      json("/api/llm/config").catch(function (error) {
        if (error && (error.status === 401 || error.status === 403)) {
          throw error;
        }
        // A server without the model routes (or a transient failure) must not
        // leave the panel silently blank.
        if (error && error.status === 404) {
          say(t("mdl-unavailable", "This server does not expose the model API."), true);
        } else {
          say(t("mdl-test-fail", "Failed: %e").replace("%e", error.message || String(error)), true);
        }
        return null;
      }),
      json("/api/llm/config/modes").catch(function () {
        return { modes: [] };
      })
    ]).then(function (results) {
      state.cfg = results[0];
      state.modes = (results[1] && results[1].modes) || [];
      state.loaded = true;
      renderCurrent();
      renderModes();
      if (state.cfg) say("");
    }).catch(function (error) {
      if (error && (error.status === 401 || error.status === 403)) {
        say(t("mdl-admin-only", "Only the administrator can change the model."), true);
      } else {
        say(t("mdl-test-fail", "Failed: %e").replace("%e", error.message || String(error)), true);
      }
    });
  }

  window.loadModelPanel = function () {
    if (!document.getElementById("model-section")) return Promise.resolve();
    return load();
  };

  // settings-nav.js intercepts section clicks (capture phase) and calls this
  // hook on every activation, so the panel loads whenever the section opens —
  // from the sidebar, a deep link or a UI plan.
  var previousHook = window.GBSettingsOnSection;
  window.GBSettingsOnSection = function (section) {
    if (typeof previousHook === "function") {
      try {
        previousHook(section);
      } catch (e) {
        /* an earlier hook is optional */
      }
    }
    if (section === "model") window.loadModelPanel();
  };

  if (window.location.hash === "#model") {
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", window.loadModelPanel);
    } else {
      window.loadModelPanel();
    }
  }
})();
