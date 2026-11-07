/* Privacy controls (local-only).
 * This build has NO outbound telemetry, analytics, or crash reporting.
 * Chat history, memory and settings stay on the device. The legacy
 * telemetry fields are kept hard-wired to false so old settings, tests and
 * callers cannot re-enable a sender that no longer exists. */
"use strict";

(function () {
  var PRIVACY_KEY = "gb_privacy_settings";
  var LEGACY_TELEMETRY_KEY = "gb_telemetry_opted_in";
  var LEGACY_TELEMETRY_LOG = "gb_telemetry_log";

  var defaults = {
    telemetry_enabled: false,
    save_chat_history: true,
    save_memory: true,
    share_usage_stats: false,
    crash_reports: false,
  };

  function loadSettings() {
    var settings;
    try {
      var saved = JSON.parse(localStorage.getItem(PRIVACY_KEY) || "{}");
      settings = Object.assign({}, defaults, saved);
    } catch (e) {
      settings = Object.assign({}, defaults);
    }
    // Hard invariant: no telemetry, even if local storage was edited or a
    // previous build saved an opt-in.
    settings.telemetry_enabled = false;
    settings.share_usage_stats = false;
    settings.crash_reports = false;
    return settings;
  }

  function saveSettings(settings) {
    var safe = Object.assign({}, settings, {
      telemetry_enabled: false,
      share_usage_stats: false,
      crash_reports: false,
    });
    try { localStorage.setItem(PRIVACY_KEY, JSON.stringify(safe)); } catch (e) {}
    try {
      localStorage.removeItem(LEGACY_TELEMETRY_KEY);
      localStorage.removeItem(LEGACY_TELEMETRY_LOG);
    } catch (e) {}
  }

  function isTelemetryEnabled() {
    return false;
  }

  function clearAllUserData() {
    if (!confirm("This will permanently delete ALL your local data including chat history, memory, settings, and cached files. This cannot be undone. Continue?")) return;
    var keys = [];
    for (var i = 0; i < localStorage.length; i++) {
      var key = localStorage.key(i);
      if (key && key.indexOf("gb_") === 0) keys.push(key);
    }
    keys.forEach(function (k) { localStorage.removeItem(k); });

    try {
      indexedDB.deleteDatabase("gb_drive_offline");
    } catch (e) {}

    if (typeof showNotification === "function") {
      showNotification("All data cleared. Reloading...", "success");
    }
    setTimeout(function () { window.location.reload(); }, 1500);
  }

  function showPrivacyPanel() {
    var settings = loadSettings();
    var modal = document.getElementById("privacy-modal");
    if (modal) modal.remove();
    modal = document.createElement("div");
    modal.id = "privacy-modal";
    modal.style.cssText = "position:fixed;top:0;left:0;width:100%;height:100%;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:99999;backdrop-filter:blur(8px);";

    function toggle(id, label, description, checked) {
      return '<div style="display:flex;justify-content:space-between;align-items:center;padding:12px 0;border-bottom:1px solid #1e293b;">'
        + '<div><div style="color:#f8fafc;font-size:14px;">' + label + '</div><div style="color:#64748b;font-size:12px;">' + description + '</div></div>'
        + '<label style="position:relative;width:42px;height:24px;cursor:pointer;"><input type="checkbox" id="' + id + '" ' + (checked ? 'checked' : '') + ' style="opacity:0;width:0;height:0;"><span style="position:absolute;top:0;left:0;right:0;bottom:0;background:' + (checked ? '#3b82f6' : '#334155') + ';border-radius:12px;transition:background 0.3s;"></span><span style="position:absolute;top:3px;left:' + (checked ? '21px' : '3px') + ';width:18px;height:18px;background:#fff;border-radius:50%;transition:left 0.3s;"></span></label>'
        + '</div>';
    }

    modal.innerHTML = '<div style="background:#1e293b;border:1px solid #334155;border-radius:12px;width:90%;max-width:500px;max-height:85vh;overflow-y:auto;padding:24px;">'
      + '<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:20px;"><h3 style="margin:0;color:#f8fafc;">Privacy &amp; Data</h3><button onclick="document.getElementById(\'privacy-modal\').remove()" style="background:none;border:none;color:#94a3b8;font-size:24px;cursor:pointer;">×</button></div>'
      + '<div style="background:#0f172a;border-radius:8px;padding:16px;margin-bottom:20px;">'
      + '<div style="color:#f8fafc;font-size:13px;font-weight:600;margin-bottom:8px;">Your data stays local</div>'
      + '<div style="color:#94a3b8;font-size:12px;line-height:1.5;">Chat history, memory, and settings are stored on your device. This build has no outbound telemetry, analytics, or crash reporting.</div>'
      + '</div>'
      + toggle("priv-history", "Save Chat History", "Keep conversation history locally", settings.save_chat_history)
      + toggle("priv-memory", "Chat Memory", "Remember preferences across conversations", settings.save_memory)
      + '<div style="color:#64748b;font-size:12px;line-height:1.5;padding:12px 0;">Telemetry, usage statistics, and crash reports are permanently disabled and never leave this device.</div>'
      + '<div style="display:flex;gap:10px;margin-top:20px;">'
      + '<button onclick="GBPrivacy.save()" style="flex:1;background:#3b82f6;border:none;border-radius:6px;color:#fff;padding:10px;cursor:pointer;font-weight:500;">Save</button>'
      + '<button onclick="GBPrivacy.clearAll()" style="background:#dc2626;border:none;border-radius:6px;color:#fff;padding:10px;cursor:pointer;font-size:12px;">Delete All Data</button>'
      + '</div></div>';
    modal.addEventListener("click", function (e) { if (e.target === modal) modal.remove(); });
    document.body.appendChild(modal);

    modal.querySelectorAll("input[type=checkbox]").forEach(function (cb) {
      cb.addEventListener("change", function () {
        var span = this.parentElement.querySelectorAll("span");
        if (span[0]) span[0].style.background = this.checked ? "#3b82f6" : "#334155";
        if (span[1]) span[1].style.left = this.checked ? "21px" : "3px";
      });
    });
  }

  function saveFromPanel() {
    var settings = {
      telemetry_enabled: false,
      save_chat_history: !!document.getElementById("priv-history") && document.getElementById("priv-history").checked,
      save_memory: !!document.getElementById("priv-memory") && document.getElementById("priv-memory").checked,
      share_usage_stats: false,
      crash_reports: false,
    };
    saveSettings(settings);
    var modal = document.getElementById("privacy-modal");
    if (modal) modal.remove();
    if (typeof showNotification === "function") showNotification("Privacy settings saved", "success");
  }

  window.GBPrivacy = {
    getSettings: loadSettings,
    isTelemetryEnabled: isTelemetryEnabled,
    showPanel: showPrivacyPanel,
    save: saveFromPanel,
    clearAll: clearAllUserData,
  };
})();
