/* Auto-update and release notes (GB6-23).
 * Checks for updates via /api/product and shows a notification
 * with changelog when a new version is available. On Tauri, triggers
 * the native updater. On web, shows a "refresh" prompt. */
"use strict";

(function () {
  var CHECK_INTERVAL = 4 * 60 * 60 * 1000;
  var LAST_SEEN_KEY = "gb_last_seen_version";
  var currentVersion = document.querySelector('meta[name="gb-version"]');
  var appVersion = currentVersion ? currentVersion.content : "0.0.0";

  function checkForUpdate() {
    fetch("/api/product", { cache: "no-store" })
      .then(function (r) { return r.ok ? r.json() : null; })
      .then(function (data) {
        if (!data || !data.version) return;
        if (data.version === appVersion) return;
        var lastSeen = localStorage.getItem(LAST_SEEN_KEY);
        if (lastSeen === data.version) return;
        showUpdateNotification(data);
      })
      .catch(function () {});
  }

  function showUpdateNotification(data) {
    var existing = document.getElementById("gb-update-banner");
    if (existing) existing.remove();

    var banner = document.createElement("div");
    banner.id = "gb-update-banner";
    banner.style.cssText = "position:fixed;bottom:24px;left:50%;transform:translateX(-50%);background:#1e293b;border:1px solid #3b82f6;border-radius:12px;padding:16px 24px;z-index:99999;box-shadow:0 8px 32px rgba(0,0,0,0.4);max-width:480px;width:90%;";

    var changelog = (data.changelog || data.release_notes || "").substring(0, 500);
    var changelogHtml = changelog ? '<div style="color:#94a3b8;font-size:12px;margin:8px 0;max-height:120px;overflow-y:auto;line-height:1.5;">' + escapeHtmlSafe(changelog) + '</div>' : '';

    banner.innerHTML = '<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:8px;">'
      + '<div style="color:#f8fafc;font-size:14px;font-weight:600;">Update Available — v' + escapeHtmlSafe(data.version) + '</div>'
      + '<button onclick="document.getElementById(\'gb-update-banner\').remove()" style="background:none;border:none;color:#64748b;font-size:18px;cursor:pointer;">×</button>'
      + '</div>'
      + changelogHtml
      + '<div style="display:flex;gap:10px;margin-top:12px;">'
      + '<button id="gb-update-btn" style="background:#3b82f6;border:none;border-radius:6px;color:#fff;padding:8px 20px;cursor:pointer;font-weight:500;font-size:13px;">Update Now</button>'
      + '<button onclick="GBUpdate.dismiss(\'' + escapeHtmlSafe(data.version) + '\')" style="background:#334155;border:none;border-radius:6px;color:#f8fafc;padding:8px 20px;cursor:pointer;font-size:13px;">Later</button>'
      + '</div>';

    document.body.appendChild(banner);

    document.getElementById("gb-update-btn").addEventListener("click", function () {
      applyUpdate(data);
    });
  }

  function applyUpdate(data) {
    var btn = document.getElementById("gb-update-btn");
    if (btn) { btn.textContent = "Updating..."; btn.disabled = true; }

    if (window.__TAURI__ && window.__TAURI__.updater) {
      window.__TAURI__.updater.checkUpdate()
        .then(function (update) {
          if (update.shouldUpdate) {
            return update.manifest ? window.__TAURI__.updater.installUpdate() : Promise.resolve();
          }
        })
        .then(function () {
          localStorage.setItem(LAST_SEEN_KEY, data.version);
        })
        .catch(function (err) {
          console.error("[update] Tauri update failed:", err);
          if (btn) { btn.textContent = "Retry"; btn.disabled = false; }
        });
    } else {
      localStorage.setItem(LAST_SEEN_KEY, data.version);
      window.location.reload();
    }
  }

  function dismiss(version) {
    localStorage.setItem(LAST_SEEN_KEY, version);
    var banner = document.getElementById("gb-update-banner");
    if (banner) banner.remove();
  }

  function showReleaseNotes() {
    fetch("/api/product", { cache: "no-store" })
      .then(function (r) { return r.ok ? r.json() : null; })
      .then(function (data) {
        if (!data) return;
        var modal = document.getElementById("release-notes-modal");
        if (modal) modal.remove();
        modal = document.createElement("div");
        modal.id = "release-notes-modal";
        modal.style.cssText = "position:fixed;top:0;left:0;width:100%;height:100%;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:99999;backdrop-filter:blur(8px);";
        var notes = data.changelog || data.release_notes || "No release notes available.";
        modal.innerHTML = '<div style="background:#1e293b;border:1px solid #334155;border-radius:12px;width:90%;max-width:560px;max-height:80vh;display:flex;flex-direction:column;overflow:hidden;">'
          + '<div style="display:flex;justify-content:space-between;align-items:center;padding:16px 24px;border-bottom:1px solid #334155;"><h3 style="margin:0;color:#f8fafc;">Release Notes — v' + escapeHtmlSafe(data.version) + '</h3><button onclick="document.getElementById(\'release-notes-modal\').remove()" style="background:none;border:none;color:#94a3b8;font-size:24px;cursor:pointer;">×</button></div>'
          + '<div style="padding:24px;overflow-y:auto;flex:1;color:#cbd5e1;font-size:14px;line-height:1.6;">' + escapeHtmlSafe(notes) + '</div>'
          + '</div>';
        modal.addEventListener("click", function (e) { if (e.target === modal) modal.remove(); });
        document.body.appendChild(modal);
      });
  }

  function escapeHtmlSafe(str) {
    return String(str || "").replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }

  setTimeout(checkForUpdate, 10000);
  setInterval(checkForUpdate, CHECK_INTERVAL);

  window.GBUpdate = {
    check: checkForUpdate,
    dismiss: dismiss,
    showReleaseNotes: showReleaseNotes,
    version: appVersion,
  };
})();
