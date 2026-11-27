"use strict";
/* Settings > Agent runtime (#gbgov parity, port of 10k_settings_agent_runtime.js).
 *
 * Reads the existing /api/product status (no new endpoint) and, in the desktop
 * application only, drives the Tauri bridge for install/cancel/remove/update.
 * Polls every 1.5 s ONLY while installing, and stops when the card is removed.
 */
(function () {
  var T = window.GBSettingsAgentRuntimeText || {};
  var t = T.t || function (key, fallback) { return fallback; };
  var esc = T.esc || function (value) {
    return String(value == null ? "" : value).replace(/&/g, "&amp;").replace(/</g, "&lt;");
  };
  var R = (window.GBSettingsAgentRuntime = window.GBSettingsAgentRuntime || {});

  var SERVER_STATE = {
    ready: "ready", unsupported: "unsupported", wsl_missing: "needs_wsl",
    distro_missing: "not_installed", runtime_not_installed: "not_installed",
    installing: "installing", paused: "paused", broken: "broken",
    needs_reboot: "needs_reboot", policy_blocked: "policy_blocked",
    needs_virtualization: "needs_virtualization"
  };
  var localPaused = false;
  var pollTimer = null;
  var cached = null;

  function bridge() {
    return window.GBTAURI && window.GBTAURI.available && typeof window.GBTAURI.invoke === "function"
      ? window.GBTAURI : null;
  }
  function hasTauri() { return !!bridge(); }
  function invoke(cmd, args) { return bridge().invoke(cmd, args || {}); }

  function normalize(status) {
    if (!status) return null;
    return {
      state: SERVER_STATE[status.state] || status.state || "not_installed",
      managed: status.managed !== false,
      runtime_version: status.runtime_version || "",
      disk_bytes: status.disk_bytes || 0,
      progress_percent: status.progress_percent != null ? status.progress_percent : status.percent,
      progress_done_bytes: status.progress_done_bytes || status.done_bytes || 0,
      progress_total_bytes: status.progress_total_bytes || status.total_bytes || 0,
      last_error: status.last_error || null,
      last_error_code: status.last_error_code || "",
      paused: !!status.paused
    };
  }

  function serverStatus() {
    return fetch("/api/product", { cache: "no-store", credentials: "same-origin" })
      .then(function (response) { return response.ok ? response.json() : null; })
      .then(function (data) {
        if (!data) return null;
        return normalize({
          state: data.agent_runtime_state || "not_installed",
          managed: true,
          runtime_version: data.agent_runtime_version || "",
          disk_bytes: data.agent_runtime_disk_bytes || 0
        });
      })
      .catch(function () { return null; });
  }

  function fetchStatus() {
    if (hasTauri()) {
      return invoke("agent_runtime_status")
        .then(function (status) { return normalize(status) || serverStatus(); })
        .catch(serverStatus);
    }
    return serverStatus();
  }

  function stopPoll() {
    if (pollTimer) { clearInterval(pollTimer); pollTimer = null; }
  }
  function startPoll(panel) {
    stopPoll();
    pollTimer = setInterval(function () {
      if (!document.body.contains(panel)) { stopPoll(); return; }
      fetchStatus().then(function (status) {
        if (!status) return;
        cached = status;
        if (status.state !== "installing") stopPoll();
        render(panel, status);
      });
    }, 1500);
  }

  function stateOf(status) {
    var state = status.state;
    return localPaused && state !== "installing" && state !== "ready" ? "paused" : state;
  }

  function card(panel, status) {
    var state = stateOf(status);
    var html = '<div class="setting-card gb-art-card">';
    html += '<div class="section-header"><h2>' + esc(T.text.title()) + "</h2></div>";
    if (status.managed === false) {
      return html + '<p class="setting-desc">' +
        esc(t("settings-agent-runtime-manual",
          "Manually configured. The agent environment is managed outside this application.")) +
        "</p></div>";
    }
    html += '<p class="setting-desc">' + esc(T.text.intro()) + "</p>";
    html += '<div class="setting-row"><div class="setting-info">' +
      '<span class="setting-title">' + esc(T.text.status()) + "</span>" +
      '<span class="setting-desc" role="status" aria-live="polite" aria-atomic="true">' +
      esc(T.label(state)) + "</span></div></div>";

    var body = T.desc(state);
    if (body) html += '<p class="setting-desc">' + esc(body) + "</p>";

    if (state === "not_installed" || state === "needs_wsl") {
      html += "<details class=\"gb-art-tech\"><summary>" + esc(T.text.details()) +
        "</summary><p class=\"setting-desc\">" + esc(T.text.tech()) + "</p></details>";
    }
    if (state === "ready") {
      if (status.runtime_version) {
        html += '<div class="setting-row"><span class="setting-title">' + esc(T.text.version()) +
          '</span><span class="setting-desc">' + esc(status.runtime_version) + "</span></div>";
      }
      if (status.disk_bytes) {
        html += '<div class="setting-row"><span class="setting-title">' + esc(T.text.space()) +
          '</span><span class="setting-desc">' + esc(T.fmtBytes(status.disk_bytes)) + "</span></div>";
      }
    }
    if (state === "installing") {
      html += '<div role="status" aria-live="polite" aria-atomic="true">' +
        T.progress(status) + "</div>";
    }
    var err = status.last_error || (status.last_error_code
      ? { code: status.last_error_code } : null);
    if (state === "broken" && err) {
      html += '<p class="setting-desc">' + esc(T.error(err.code)) + "</p>";
      if (err.detail) {
        html += "<details class=\"gb-art-tech\"><summary>" + esc(T.text.details()) +
          "</summary><p class=\"setting-desc\">" + esc(err.detail) + "</p></details>";
      }
    }
    if (state === "policy_blocked") {
      html += '<div class="gb-art-it"><p class="setting-desc">' +
        esc(t("settings-agent-runtime-it-label", "Text to send to your IT team:")) + "</p>" +
        '<p class="setting-desc" id="gbArtItText">' + esc(T.text.itText()) + "</p>" +
        '<button type="button" class="btn-secondary btn-sm" data-gb-art="copy" id="gbArtCopyIt">' +
        esc(T.text.copy()) + "</button></div>";
    }
    if (state === "not_installed" || state === "needs_wsl" || state === "paused" || state === "broken") {
      html += '<p class="setting-desc">' + esc(T.text.experimental()) + "</p>";
    }
    html += '<div class="gb-art-actions" role="group">';
    html += actions(state);
    html += "</div>";
    if (!hasTauri() && state !== "ready" && state !== "unsupported" && state !== "policy_blocked") {
      html += '<p class="setting-desc">' + esc(T.text.desktopOnly()) + "</p>";
    }
    return html + "</div>";
  }

  function actions(state) {
    var recheck = '<button type="button" class="btn-secondary btn-sm" data-gb-art="recheck">' +
      esc(T.text.recheck()) + "</button>";
    switch (state) {
      case "not_installed":
      case "needs_wsl":
        return '<button type="button" class="btn-primary btn-sm" data-gb-art="install">' +
          esc(T.text.install()) + "</button>";
      case "needs_reboot":
        return '<button type="button" class="btn-primary btn-sm" data-gb-art="resume">' +
          esc(t("settings-agent-runtime-continue", "Continue")) + "</button>";
      case "needs_virtualization":
      case "unsupported":
      case "policy_blocked":
        return recheck;
      case "installing":
        return '<button type="button" class="btn-secondary btn-sm" data-gb-art="cancel">' +
          esc(T.text.cancel()) + "</button>";
      case "paused":
        return '<button type="button" class="btn-primary btn-sm" data-gb-art="continue">' +
          esc(t("settings-agent-runtime-continue", "Continue")) + "</button>";
      case "broken":
        return '<button type="button" class="btn-primary btn-sm" data-gb-art="retry">' +
          esc(T.text.retry()) + "</button>";
      case "ready":
        return '<button type="button" class="btn-secondary btn-sm" data-gb-art="update">' +
          esc(T.text.update()) + "</button>" +
          '<button type="button" class="btn-secondary btn-sm" data-gb-art="remove">' +
          esc(T.text.remove()) + "</button>";
      default:
        return "";
    }
  }

  function say(panel, text) {
    var node = panel.querySelector(".gb-art-status");
    if (!node) {
      node = document.createElement("p");
      node.className = "gb-art-status setting-desc";
      node.setAttribute("role", "status");
      node.setAttribute("aria-live", "polite");
      panel.appendChild(node);
    }
    node.textContent = String(text || "");
  }

  function render(panel, status) {
    panel.innerHTML = card(panel, status);
    bind(panel, status);
    if (status.state === "installing") startPoll(panel);
    else stopPoll();
  }

  function refresh(panel) {
    return fetchStatus().then(function (status) {
      if (!status) { say(panel, T.text.unavailable()); return; }
      cached = status;
      render(panel, status);
    });
  }

  function run(panel, command, args) {
    if (!hasTauri()) return Promise.reject(new Error("Tauri IPC unavailable"));
    return invoke(command, args).then(function () { return refresh(panel); });
  }

  function confirmRemove(panel) {
    var cardNode = panel.querySelector(".gb-art-card");
    if (!cardNode) return;
    var box = document.createElement("div");
    box.className = "gb-art-confirm";
    box.innerHTML = '<p class="setting-desc">' + esc(T.text.removeConfirm()) + "</p>" +
      '<label class="setting-row"><input type="checkbox" id="gbArtKeepExport" /> ' +
      esc(T.text.keepExport()) + "</label>" +
      '<button type="button" class="btn-secondary btn-sm" data-gb-art="confirm-remove">' +
      esc(T.text.remove()) + "</button> " +
      '<button type="button" class="btn-secondary btn-sm" data-gb-art="cancel-remove">' +
      esc(T.text.cancel()) + "</button>";
    cardNode.appendChild(box);
  }

  function bind(panel, status) {
    if (panel.dataset.gbArtBound !== "1") {
      panel.dataset.gbArtBound = "1";
      panel.addEventListener("click", onClick);
    }
    function onClick(event) {
      var button = event.target && event.target.closest
        ? event.target.closest("[data-gb-art]") : null;
      if (!button) return;
      var action = button.getAttribute("data-gb-art");
      if (action === "copy") {
        if (navigator.clipboard && navigator.clipboard.writeText) {
          navigator.clipboard.writeText(T.text.itText()).then(function () {
            button.textContent = T.text.copied();
          }).catch(function () { /* read-only clipboard */ });
        }
        return;
      }
      if (action === "remove") { confirmRemove(panel); return; }
      if (action === "cancel-remove") {
        var box = panel.querySelector(".gb-art-confirm");
        if (box) box.remove();
        return;
      }
      if (action === "confirm-remove") {
        var keep = !!(panel.querySelector("#gbArtKeepExport") || {}).checked;
        button.disabled = true;
        run(panel, "agent_runtime_remove", { request: { keep_export: keep } })
          .catch(function (error) { say(panel, error && error.message ? error.message : T.error("")); });
        return;
      }
      if (action === "update") {
        button.disabled = true;
        invoke("agent_runtime_check_update").then(function (next) {
          if (next && next.available_version && next.available_version !== next.runtime_version) {
            cached = next;
            render(panel, next);
          } else {
            button.textContent = T.text.noUpdate();
          }
        }).catch(function (error) {
          button.disabled = false;
          say(panel, error && error.message ? error.message : T.error(""));
        });
        return;
      }
      var commands = {
        install: "agent_runtime_install", resume: "agent_runtime_install",
        continue: "agent_runtime_install", retry: "agent_runtime_install",
        cancel: "agent_runtime_cancel", recheck: ""
      };
      var command = commands[action];
      if (!command) {
        button.disabled = true;
        localPaused = false;
        refresh(panel);
        return;
      }
      button.disabled = true;
      button.textContent = action === "recheck" ? T.text.checking() : T.text.starting();
      localPaused = action === "cancel";
      run(panel, command).catch(function (error) {
        button.disabled = false;
        say(panel, error && error.message ? error.message : T.error(""));
      });
    }
  }

  R.ready = function () { return !!cached && cached.state === "ready"; };
  R.state = function () { return cached ? cached.state : null; };
  R.fetch = function () {
    return fetchStatus().then(function (status) {
      if (status) cached = status;
      return status;
    });
  };
  R.openSection = function () {
    if (typeof window.openDeepLink === "function") {
      window.openDeepLink("settings", { section: "agent-runtime" });
      window.setTimeout(function () {
        if (typeof window.GBSettingsActivateSection === "function") {
          window.GBSettingsActivateSection("agent-runtime");
        }
      }, 250);
      return;
    }
    window.location.href = "/settings#agent-runtime";
  };

  function load() {
    var panel = document.getElementById("agent-runtime-card");
    if (!panel) return;
    fetchStatus().then(function (status) {
      if (!status) {
        panel.innerHTML = '<p class="setting-desc">' + esc(T.text.unavailable()) + "</p>";
        return;
      }
      cached = status;
      render(panel, status);
    });
  }
  window.loadAgentRuntimePanel = load;

  var previousHook = window.GBSettingsOnSection;
  window.GBSettingsOnSection = function (section) {
    if (typeof previousHook === "function") {
      try { previousHook(section); } catch (error) { /* optional hook */ }
    }
    if (section === "agent-runtime") load();
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", load);
  } else {
    load();
  }
})();
