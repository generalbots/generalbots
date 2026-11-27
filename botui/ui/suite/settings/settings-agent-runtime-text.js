"use strict";
/* Settings > Agent runtime — all user-facing copy.
 *
 * Ported from the gbgov assistant shell (10j_settings_agent_text.js). Every
 * string is a literal hyphenated catalogue key through gbT with an English
 * fallback; no locale literal is authored here and no key is machine-made.
 */
(function () {
  var T = (window.GBSettingsAgentRuntimeText = window.GBSettingsAgentRuntimeText || {});

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }
  function esc(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;")
      .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }
  function fmtBytes(bytes) {
    var n = Number(bytes);
    if (!isFinite(n) || n <= 0) return "";
    return n >= 1073741824
      ? (n / 1073741824).toFixed(1) + " GB"
      : Math.round(n / 1048576) + " MB";
  }

  /* One-line chip labels. Default covers unknown/legacy server states. */
  function label(state) {
    switch (state) {
      case "ready": return t("settings-agent-runtime-state-ready", "Installed");
      case "installing": return t("settings-agent-runtime-state-installing", "Installing");
      case "not_installed":
      case "needs_wsl": return t("settings-agent-runtime-state-not-installed", "Not installed");
      case "paused": return t("settings-agent-runtime-state-paused", "Installation paused");
      case "broken": return t("settings-agent-runtime-state-broken", "Install problem");
      case "needs_reboot": return t("settings-agent-runtime-state-reboot", "Restart needed");
      case "unsupported":
      case "policy_blocked":
      case "needs_virtualization":
        return t("settings-agent-runtime-state-blocked", "Blocked on this computer");
      default: return String(state || "");
    }
  }

  /* Plain-language body text, one paragraph per state group. */
  function desc(state) {
    switch (state) {
      case "ready": return t("settings-agent-runtime-desc-ready",
        "Agent Mode is installed and running. Subscription assistants are available in the model menu.");
      case "installing": return ""; /* progress region replaces the description */
      case "paused": return t("settings-agent-runtime-desc-paused",
        "Installation was paused. Downloaded files are kept.");
      case "broken": return t("settings-agent-runtime-desc-broken",
        "The installation needs attention. Try again; if the problem repeats, remove and reinstall.");
      case "needs_reboot": return t("settings-agent-runtime-desc-reboot",
        "Restart the computer, then open General Bots and return here. Installation resumes where it stopped.");
      case "not_installed":
      case "needs_wsl": return t("settings-agent-runtime-desc-install",
        "Agent Mode uses a Windows feature to run an isolated Linux environment. It is not set up on this computer yet.");
      case "unsupported":
      case "policy_blocked":
      case "needs_virtualization":
        return t("settings-agent-runtime-desc-blocked",
          "Your organization blocks this feature. Ask your IT team to enable Linux and virtual machines.");
      default: return "";
    }
  }

  function error(code) {
    switch (code) {
      case "offline": return t("settings-agent-runtime-error-offline",
        "No internet connection. Check the network and try again.");
      case "disk_full": return t("settings-agent-runtime-error-disk",
        "Not enough disk space. Free some space and try again.");
      case "cancelled": return t("settings-agent-runtime-cancel", "Cancel");
      default: return t("settings-agent-runtime-error-generic", "Something went wrong. Try again.");
    }
  }

  /* role=status + aria-live=polite is set by the caller; text is always a key. */
  function progress(status) {
    var state = status || {};
    var percent = Number(state.progress_percent != null ? state.progress_percent : state.percent);
    var html = '<p class="setting-desc">' + esc(t("settings-agent-runtime-progress",
      "Installing. This may take a few minutes.")) + "</p>";
    html += '<div class="gb-art-progress" role="progressbar" aria-valuemin="0" aria-valuemax="100"';
    html += ' aria-valuenow="' + (isFinite(percent) ? Math.min(100, Math.max(0, Math.round(percent))) : 0) + '">';
    html += '<div class="gb-art-progress-bar"><i style="width:' +
      (isFinite(percent) && percent > 0 ? Math.min(100, Math.round(percent)) : 0) + '%"></i></div>';
    var done = fmtBytes(state.progress_done_bytes || state.done_bytes);
    var total = fmtBytes(state.progress_total_bytes || state.total_bytes);
    if (done && total) {
      html += '<p class="setting-desc">' + esc(done) + " / " + esc(total) + "</p>";
    }
    return html + "</div>";
  }

  T.t = t;
  T.esc = esc;
  T.fmtBytes = fmtBytes;
  T.label = label;
  T.desc = desc;
  T.error = error;
  T.progress = progress;
  T.text = {
    title: function () { return t("settings-agent-runtime-title", "Agent Mode"); },
    intro: function () { return t("settings-agent-runtime-intro",
      "Use subscription assistants that run commands in an isolated environment on this computer."); },
    status: function () { return t("settings-agent-runtime-status", "Status"); },
    details: function () { return t("settings-agent-runtime-details", "Technical details"); },
    tech: function () { return t("settings-agent-runtime-tech",
      "Installs WSL2 and a lightweight Linux environment. Data is stored in the agent runtime folder."); },
    itText: function () { return t("settings-agent-runtime-it-text",
      "Please enable Windows Subsystem for Linux (WSL2), Virtual Machine Platform and wsl.exe in group policy so Agent Mode can be installed."); },
    copy: function () { return t("settings-agent-runtime-copy", "Copy text"); },
    copied: function () { return t("settings-agent-runtime-copied", "Copied"); },
    experimental: function () { return t("settings-agent-runtime-experimental",
      "Experimental install in this preview: not validated on every computer. Your files and conversations are not affected."); },
    install: function () { return t("settings-agent-runtime-install", "Install"); },
    starting: function () { return t("settings-agent-runtime-starting", "Starting…"); },
    checking: function () { return t("settings-agent-runtime-checking", "Checking…"); },
    cancel: function () { return t("settings-agent-runtime-cancel", "Cancel"); },
    recheck: function () { return t("settings-agent-runtime-recheck", "Check again"); },
    retry: function () { return t("settings-agent-runtime-retry", "Try again"); },
    update: function () { return t("settings-agent-runtime-update", "Check for update"); },
    noUpdate: function () { return t("settings-agent-runtime-no-update", "No update available"); },
    remove: function () { return t("settings-agent-runtime-remove", "Remove"); },
    removeConfirm: function () { return t("settings-agent-runtime-remove-confirm",
      "Removing Agent Mode deletes its Linux environment and everything agents created inside it. This cannot be undone."); },
    keepExport: function () { return t("settings-agent-runtime-keep-export",
      "Keep a copy of the data before removing"); },
    desktopOnly: function () { return t("settings-agent-runtime-desktop-only",
      "Actions are available only in the desktop application."); },
    unavailable: function () { return t("settings-agent-runtime-unavailable",
      "Could not retrieve Agent Mode status."); },
    version: function () { return t("settings-agent-runtime-version", "Version"); },
    space: function () { return t("settings-agent-runtime-space", "Space used"); }
  };
})();
