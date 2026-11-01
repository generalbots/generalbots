/* Data export and backup (GB6-24).
 * Collects all user data from localStorage, chat history, drive
 * metadata, and settings into a JSON bundle the user can download.
 * Also supports importing a backup to restore state. */
"use strict";

(function () {
  function collectLocalStorageData() {
    var data = {};
    for (var i = 0; i < localStorage.length; i++) {
      var key = localStorage.key(i);
      if (key && key.indexOf("gb_") === 0) {
        try { data[key] = localStorage.getItem(key); } catch (e) {}
      }
    }
    return data;
  }

  function collectChatHistory() {
    var sessions = [];
    for (var i = 0; i < localStorage.length; i++) {
      var key = localStorage.key(i);
      if (key && key.indexOf("gb_chat_history_") === 0) {
        try {
          var messages = JSON.parse(localStorage.getItem(key));
          sessions.push({ session_key: key, messages: messages });
        } catch (e) {}
      }
    }
    return sessions;
  }

  function collectMemory() {
    if (window.ChatMemory && typeof ChatMemory.loadMemory === "function") {
      return ChatMemory.loadMemory();
    }
    return {};
  }

  function collectSettings() {
    var settings = {};
    var keys = ["gb_theme", "gb_sync_folder", "gb_setup_complete", "gb_setup_user_name", "gb_telemetry_opted_in", "gb_privacy_settings"];
    keys.forEach(function (k) {
      var val = localStorage.getItem(k);
      if (val !== null) settings[k] = val;
    });
    return settings;
  }

  function exportAllData() {
    var bundle = {
      export_version: 1,
      exported_at: new Date().toISOString(),
      app_version: (window.GBUpdate && GBUpdate.version) || "unknown",
      settings: collectSettings(),
      chat_history: collectChatHistory(),
      chat_memory: collectMemory(),
      local_storage: collectLocalStorageData(),
    };

    var json = JSON.stringify(bundle, null, 2);
    var blob = new Blob([json], { type: "application/json" });
    var url = URL.createObjectURL(blob);
    var a = document.createElement("a");
    a.href = url;
    a.download = "gb-backup-" + new Date().toISOString().split("T")[0] + ".json";
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    if (typeof showNotification === "function") {
      showNotification("Backup exported successfully", "success");
    }
  }

  // GB6-24 — importing arbitrary JSON into localStorage is the riskiest part
  // of the backup feature: a crafted file could overwrite the session token or
  // exhaust storage. The bundle is validated against an explicit allowlist
  // before anything is written, and credentials are never restored.
  var BUNDLE_VERSION = 1;
  var TOP_LEVEL_KEYS = ["export_version", "exported_at", "app_version", "settings", "chat_history", "chat_memory", "local_storage"];
  var STORAGE_KEY_PATTERN = /^gb[_.-][a-z0-9_.:-]+$/i;
  var DENIED_KEYS = ["gb-access-token", "gb-desktop-local", "gb-auth-token", "management_token"];
  var MAX_VALUE_BYTES = 256 * 1024;
  var MAX_TOTAL_BYTES = 4 * 1024 * 1024;
  var MAX_KEY_LENGTH = 128;

  function isDeniedKey(key) {
    var lower = String(key).toLowerCase();
    if (lower === "__proto__" || lower === "constructor" || lower === "prototype") return true;
    return DENIED_KEYS.indexOf(lower) !== -1;
  }

  function sanitizeBackup(bundle) {
    if (!bundle || typeof bundle !== "object" || Array.isArray(bundle)) {
      throw new Error("the file is not a backup object");
    }
    var version = Number(bundle.export_version);
    if (!Number.isInteger(version) || version < 1 || version > BUNDLE_VERSION) {
      throw new Error("unsupported backup version");
    }
    var exportedAt = Date.parse(bundle.exported_at);
    if (!Number.isFinite(exportedAt)) {
      throw new Error("the backup has no valid export date");
    }
    Object.keys(bundle).forEach(function (key) {
      if (TOP_LEVEL_KEYS.indexOf(key) === -1) {
        throw new Error("unexpected section: " + key);
      }
    });

    var accepted = {};
    var rejected = 0;
    var totalBytes = 0;

    function accept(source) {
      if (!source || typeof source !== "object" || Array.isArray(source)) return;
      Object.keys(source).forEach(function (key) {
        var value = source[key];
        if (typeof key !== "string" || key.length > MAX_KEY_LENGTH) { rejected++; return; }
        if (isDeniedKey(key) || !STORAGE_KEY_PATTERN.test(key)) { rejected++; return; }
        if (typeof value !== "string") { rejected++; return; }
        var bytes = key.length + value.length;
        if (bytes > MAX_VALUE_BYTES) { rejected++; return; }
        if (totalBytes + bytes > MAX_TOTAL_BYTES) { rejected++; return; }
        totalBytes += bytes;
        accepted[key] = value;
      });
    }

    accept(bundle.settings);
    accept(bundle.local_storage);
    return { exported_at: bundle.exported_at, values: accepted, rejected: rejected };
  }

  function importBackup() {
    var input = document.createElement("input");
    input.type = "file";
    input.accept = ".json";
    input.style.display = "none";
    input.addEventListener("change", function () {
      var file = input.files && input.files[0];
      if (!file) return;
      if (file.size > MAX_TOTAL_BYTES) {
        if (typeof showNotification === "function") {
          showNotification("Backup rejected: the file is larger than 4 MB", "error");
        }
        return;
      }
      var reader = new FileReader();
      reader.onload = function () {
        try {
          var bundle = JSON.parse(reader.result);
          var clean = sanitizeBackup(bundle);
          applyBackup(clean);
        } catch (e) {
          if (typeof showNotification === "function") {
            showNotification("Invalid backup file: " + e.message, "error");
          }
        }
      };
      reader.readAsText(file);
      document.body.removeChild(input);
    });
    document.body.appendChild(input);
    input.click();
  }

  function applyBackup(clean) {
    var keys = Object.keys(clean.values);
    if (!keys.length) {
      if (typeof showNotification === "function") {
        showNotification("This backup contains no restorable settings", "error");
      }
      return;
    }
    if (!confirm(
      "This will restore " + keys.length + " setting(s) from the backup of " +
      (clean.exported_at || "unknown date") + ". Credentials and session tokens are never imported. Continue?",
    )) return;

    var count = 0;
    keys.forEach(function (key) {
      try {
        localStorage.setItem(key, clean.values[key]);
        count++;
      } catch (e) {
        // Storage full or disabled: keep restoring the remaining keys.
      }
    });

    var message = "Backup restored (" + count + " items).";
    if (clean.rejected > 0) message += " " + clean.rejected + " entry(ies) rejected by the allowlist.";
    if (typeof showNotification === "function") {
      showNotification(message + " Reloading...", "success");
    }
    setTimeout(function () { window.location.reload(); }, 2000);
  }

  function showExportDialog() {
    var modal = document.getElementById("export-modal");
    if (modal) modal.remove();
    modal = document.createElement("div");
    modal.id = "export-modal";
    modal.style.cssText = "position:fixed;top:0;left:0;width:100%;height:100%;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:99999;backdrop-filter:blur(8px);";
    modal.innerHTML = '<div style="background:#1e293b;border:1px solid #334155;border-radius:12px;width:90%;max-width:440px;padding:24px;">'
      + '<h3 style="margin:0 0 16px;color:#f8fafc;">Data Export &amp; Backup</h3>'
      + '<p style="color:#94a3b8;font-size:14px;margin-bottom:20px;">Export all your data including chat history, memory, drive metadata, and settings as a JSON backup file.</p>'
      + '<div style="display:flex;flex-direction:column;gap:10px;">'
      + '<button onclick="GBDataExport.exportAll();document.getElementById(\'export-modal\').remove()" style="background:#3b82f6;border:none;border-radius:8px;color:#fff;padding:12px;cursor:pointer;font-weight:500;font-size:14px;">Export All Data</button>'
      + '<button onclick="GBDataExport.importBackup();document.getElementById(\'export-modal\').remove()" style="background:#334155;border:none;border-radius:8px;color:#f8fafc;padding:12px;cursor:pointer;font-size:14px;">Import Backup</button>'
      + '<button onclick="document.getElementById(\'export-modal\').remove()" style="background:transparent;border:1px solid #334155;border-radius:8px;color:#94a3b8;padding:12px;cursor:pointer;font-size:14px;">Cancel</button>'
      + '</div></div>';
    modal.addEventListener("click", function (e) { if (e.target === modal) modal.remove(); });
    document.body.appendChild(modal);
  }

  window.GBDataExport = {
    exportAll: exportAllData,
    importBackup: importBackup,
    showDialog: showExportDialog,
  };
})();
