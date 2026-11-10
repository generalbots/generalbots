/* Desktop update check and release notes (#1862).
 *
 * In the Tauri desktop shell the Rust command `check_release_update` asks
 * GitHub for the latest release, compares it with the running application
 * version and returns the release page plus platform asset. This module runs
 * an opt-in startup check (once per day), shows a non-blocking prompt and
 * links to the official release page. Nothing is downloaded or installed
 * silently; "Update now" is the opt-in step.
 *
 * Browser and server deployments keep the historical release-notes surface
 * (`/api/product`) and never perform a GitHub check. The Settings surface
 * lives in `auto-update-settings.js` so both files stay under the size limit.
 */
"use strict";

(function () {
  if (window.GBUpdate) return;

  // #2095 — user-visible strings come from the catalogue; the authored English
  // text is the fallback when the active locale has no entry yet.
  function t(key, fallback) {
    return window.gbT ? window.gbT(key, fallback) : fallback;
  }

  var DAY_MS = 24 * 60 * 60 * 1000;
  var CHECK_INTERVAL = 4 * 60 * 60 * 1000;
  var STARTUP_DELAY = 30 * 1000;
  var RELEASES_PAGE = "https://github.com/generalbots/generalbots/releases";

  var KEYS = {
    enabled: "gb-auto-update",
    lastCheck: "gb-auto-update-last-check",
    skipped: "gb-auto-update-skip-version",
    lastSeen: "gb-last-seen-version",
    install: "gb-auto-update-install",
  };

  var currentVersion =
    (document.querySelector('meta[name="gb-version"]') || {}).content || "0.0.0";
  var lastInfo = null;

  function isDesktop() {
    return !!(window.__TAURI__ || window.__TAURI_INTERNALS__);
  }

  function invoke(command, args) {
    var tauri = window.__TAURI__;
    if (tauri && tauri.core && typeof tauri.core.invoke === "function") {
      return tauri.core.invoke(command, args);
    }
    if (
      window.__TAURI_INTERNALS__ &&
      typeof window.__TAURI_INTERNALS__.invoke === "function"
    ) {
      return window.__TAURI_INTERNALS__.invoke(command, args);
    }
    return Promise.reject(new Error("Tauri invoke is not available"));
  }

  function storageGet(key, fallback) {
    try {
      var value = window.localStorage.getItem(key);
      return value === null ? fallback : value;
    } catch (error) {
      return fallback;
    }
  }

  function storageSet(key, value) {
    try {
      window.localStorage.setItem(key, String(value));
    } catch (error) {
      /* private mode / disabled storage: keep the in-memory default */
    }
  }

  // Automatic checks are opt-out in the desktop shell (turn them off with the
  // Settings toggle). Installing an update is always opt-in.
  function startupCheckEnabled() {
    return storageGet(KEYS.enabled, "true") !== "false";
  }

  function autoInstallEnabled() {
    return storageGet(KEYS.install, "false") === "true";
  }

  function lastCheckAt() {
    var raw = Number(storageGet(KEYS.lastCheck, "0"));
    return Number.isFinite(raw) ? raw : 0;
  }

  function checkIsDue() {
    return Date.now() - lastCheckAt() > DAY_MS;
  }

  function isSettingsSurface() {
    return !!document.querySelector('[data-gb-app="settings"]');
  }

  function settingsModule() {
    return window.GBUpdateSettings || null;
  }

  function notifySettings(info, error) {
    var settings = settingsModule();
    if (settings && typeof settings.render === "function") {
      settings.render(info || lastInfo, error);
    }
  }

  function releaseUrl(info) {
    var url = (info && info.release_url) || "";
    if (/^https:\/\/github\.com\/generalbots\/generalbots\/releases\//.test(url)) {
      return url;
    }
    return RELEASES_PAGE;
  }

  function formatSize(bytes) {
    var size = Number(bytes);
    if (!Number.isFinite(size) || size <= 0) return "";
    if (size < 1024 * 1024) return Math.round(size / 1024) + " KB";
    return (size / (1024 * 1024)).toFixed(1) + " MB";
  }

  function notesSnippet(info, limit) {
    var notes = String((info && (info.release_notes || info.body)) || "").trim();
    if (!notes) return "";
    if (notes.length <= limit) return notes;
    return notes.slice(0, limit).trimEnd() + "…";
  }

  function hideBanner() {
    var banner = document.getElementById("gb-update-banner");
    if (banner) banner.remove();
  }

  function openRelease(info) {
    var url = releaseUrl(info);
    if (isDesktop()) {
      invoke("open_release_url", { url: url }).catch(function () {
        window.open(url, "_blank", "noopener,noreferrer");
      });
    } else {
      window.open(url, "_blank", "noopener,noreferrer");
    }
  }

  function button(label, styles, onClick) {
    var element = document.createElement("button");
    element.type = "button";
    element.textContent = label;
    element.style.cssText = styles;
    element.addEventListener("click", onClick);
    return element;
  }

  function showUpdateNotification(info) {
    if (!info || !info.update_available) return;
    if (storageGet(KEYS.skipped, "") === info.latest_version) return;
    hideBanner();

    var banner = document.createElement("div");
    banner.id = "gb-update-banner";
    banner.setAttribute("role", "status");
    banner.style.cssText =
      "position:fixed;bottom:24px;left:50%;transform:translateX(-50%);" +
      "background:#1e293b;border:1px solid #3b82f6;border-radius:12px;" +
      "padding:16px 24px;z-index:99999;box-shadow:0 8px 32px rgba(0,0,0,0.4);" +
      "max-width:520px;width:90%;color:#f8fafc;font-family:system-ui,sans-serif;";

    var header = document.createElement("div");
    header.style.cssText =
      "display:flex;justify-content:space-between;align-items:center;margin-bottom:8px;gap:12px;";

    var title = document.createElement("div");
    title.style.cssText = "font-size:14px;font-weight:600;";
    var latestVersion = String(info.latest_version || "");
    title.textContent = window.gbT
      ? window.gbT("update-available-title", "Update available — v" + latestVersion, { version: latestVersion })
      : "Update available — v" + latestVersion;
    header.appendChild(title);

    var close = button(
      "\u00d7",
      "background:none;border:none;color:#64748b;font-size:18px;cursor:pointer;line-height:1;",
      function () {
        dismiss(info.latest_version);
      }
    );
    close.setAttribute("aria-label", t("update-later", "Later"));
    header.appendChild(close);
    banner.appendChild(header);

    var notes = notesSnippet(info, 500);
    if (notes) {
      var body = document.createElement("div");
      body.style.cssText =
        "color:#94a3b8;font-size:12px;margin:8px 0;max-height:120px;" +
        "overflow-y:auto;line-height:1.5;white-space:pre-wrap;";
      body.textContent = notes;
      banner.appendChild(body);
    }

    var meta = document.createElement("div");
    meta.style.cssText = "color:#64748b;font-size:11px;margin-top:6px;";
    var parts = [];
    if (info.platform) parts.push(window.gbT ? window.gbT("update-platform", "Platform: " + info.platform, { name: info.platform }) : "Platform: " + info.platform);
    if (info.download_name) parts.push(window.gbT ? window.gbT("update-asset", "Asset: " + info.download_name, { name: info.download_name }) : "Asset: " + info.download_name);
    var size = formatSize(info.download_size);
    if (size) parts.push(size);
    meta.textContent = parts.join(" · ");
    if (meta.textContent) banner.appendChild(meta);

    var actions = document.createElement("div");
    actions.style.cssText = "display:flex;gap:10px;margin-top:12px;flex-wrap:wrap;";

    actions.appendChild(
      button(
        t("update-now", "Update now"),
        "background:#3b82f6;border:none;border-radius:6px;color:#fff;" +
          "padding:8px 20px;cursor:pointer;font-weight:500;font-size:13px;",
        function () {
          openRelease(info);
          dismiss(info.latest_version);
        }
      )
    );
    actions.appendChild(
      button(
        t("update-later", "Later"),
        "background:#334155;border:none;border-radius:6px;color:#f8fafc;" +
          "padding:8px 20px;cursor:pointer;font-size:13px;",
        function () {
          dismiss(info.latest_version);
        }
      )
    );
    actions.appendChild(
      button(
        t("update-skip-version", "Skip this version"),
        "background:transparent;border:1px solid #475569;border-radius:6px;" +
          "color:#cbd5e1;padding:8px 16px;cursor:pointer;font-size:12px;",
        function () {
          skip(info.latest_version);
        }
      )
    );
    banner.appendChild(actions);
    document.body.appendChild(banner);
  }

  function remember(info) {
    lastInfo = info;
    window.GBUpdate.lastInfo = info;
    if (info && info.current_version) {
      currentVersion = String(info.current_version);
      window.GBUpdate.version = currentVersion;
    }
    storageSet(KEYS.lastCheck, Date.now());
    notifySettings(info);
    return info;
  }

  function check(force) {
    if (!isDesktop()) return Promise.resolve(null);
    if (!force && !startupCheckEnabled()) return Promise.resolve(null);
    if (!force && !checkIsDue()) return Promise.resolve(null);

    // Record the attempt even when the network fails so an offline desktop
    // retries at most once a day instead of on every reload.
    storageSet(KEYS.lastCheck, Date.now());

    return invoke("check_release_update")
      .then(function (info) {
        remember(info);
        if (!info || !info.update_available) {
          hideBanner();
          return info;
        }
        if (storageGet(KEYS.skipped, "") === info.latest_version) return info;
        if (force || startupCheckEnabled()) showUpdateNotification(info);
        return info;
      })
      .catch(function (error) {
        console.warn("[update] GitHub release check failed:", error);
        notifySettings(null, error && error.message ? error.message : error);
        return null;
      });
  }

  function checkNow() {
    var settings = settingsModule();
    if (settings && typeof settings.setBusy === "function") settings.setBusy();
    return check(true);
  }

  function setStartupCheck(enabled) {
    storageSet(KEYS.enabled, enabled ? "true" : "false");
    if (!enabled) {
      hideBanner();
      return;
    }
    check(true);
  }

  function setAutoInstall(enabled) {
    // The signed updater artifacts are a follow-up; keep the preference so
    // the UI state is honest and the future installer can read it.
    storageSet(KEYS.install, enabled ? "true" : "false");
  }

  function dismiss(version) {
    if (version) storageSet(KEYS.lastSeen, version);
    hideBanner();
  }

  function skip(version) {
    if (version) storageSet(KEYS.skipped, version);
    hideBanner();
  }

  function showReleaseNotes(info) {
    var settings = settingsModule();
    if (settings && typeof settings.showReleaseNotes === "function") {
      return settings.showReleaseNotes(info);
    }
    return Promise.resolve();
  }

  function renderSettings() {
    var settings = settingsModule();
    if (settings && typeof settings.renderSettings === "function") {
      settings.renderSettings();
    }
  }

  function readAppVersion() {
    var tauri = window.__TAURI__;
    if (!tauri || !tauri.app || typeof tauri.app.getVersion !== "function") return;
    tauri.app
      .getVersion()
      .then(function (version) {
        if (version) currentVersion = String(version);
        window.GBUpdate.version = currentVersion;
        renderSettings();
      })
      .catch(function () {
        /* the Rust check still reports the authoritative version */
      });
  }

  function scheduleStartupCheck() {
    if (!isDesktop() || isSettingsSurface() || !startupCheckEnabled()) return;
    window.setTimeout(function () {
      check(false);
    }, STARTUP_DELAY);
    window.setInterval(function () {
      check(false);
    }, CHECK_INTERVAL);
  }

  window.GBUpdate = {
    version: currentVersion,
    lastInfo: lastInfo,
    check: check,
    checkNow: checkNow,
    dismiss: dismiss,
    skip: skip,
    showReleaseNotes: showReleaseNotes,
    renderSettings: renderSettings,
    setStartupCheck: setStartupCheck,
    setAutoInstall: setAutoInstall,
    isDesktop: isDesktop,
    startupCheckEnabled: startupCheckEnabled,
    autoInstallEnabled: autoInstallEnabled,
    storageGet: storageGet,
    storageSet: storageSet,
    notesSnippet: notesSnippet,
    releaseUrl: releaseUrl,
    keys: KEYS,
  };

  readAppVersion();
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", scheduleStartupCheck);
  } else {
    scheduleStartupCheck();
  }
})();
