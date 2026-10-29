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

  function importBackup() {
    var input = document.createElement("input");
    input.type = "file";
    input.accept = ".json";
    input.style.display = "none";
    input.addEventListener("change", function () {
      var file = input.files && input.files[0];
      if (!file) return;
      var reader = new FileReader();
      reader.onload = function () {
        try {
          var bundle = JSON.parse(reader.result);
          if (!bundle.export_version) throw new Error("Invalid backup file");
          applyBackup(bundle);
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

  function applyBackup(bundle) {
    if (!confirm("This will overwrite your current data with the backup from " + (bundle.exported_at || "unknown date") + ". Continue?")) return;

    var count = 0;

    if (bundle.local_storage) {
      var ALLOWED_PREFIX = "gb_";
      Object.keys(bundle.local_storage).forEach(function (key) {
        if (key.indexOf(ALLOWED_PREFIX) !== 0) return;
        try { localStorage.setItem(key, bundle.local_storage[key]); count++; } catch (e) {}
      });
    }

    if (bundle.settings) {
      Object.keys(bundle.settings).forEach(function (key) {
        if (key.indexOf("gb_") !== 0) return;
        try { localStorage.setItem(key, bundle.settings[key]); } catch (e) {}
      });
    }

    if (typeof showNotification === "function") {
      showNotification("Backup restored (" + count + " items). Reloading...", "success");
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
