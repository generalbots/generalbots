/* Settings surface for the desktop update check (#1862).
 *
 * Kept separate from `auto-update.js` so both modules stay under the file-size
 * limit. Loaded by the suite shell and by the standalone Settings page. */
"use strict";

(function () {
  if (!window.GBUpdate || window.GBUpdateSettings) return;
  // #2095 — user-visible strings come from the catalogue; the authored English
  // text is the fallback when the active locale has no entry yet.
  function t(key, fallback) {
    return window.gbT ? window.gbT(key, fallback) : fallback;
  }


  var GB = window.GBUpdate;

  function formatDate(value) {
    if (!value) return "";
    try {
      return new Date(value).toLocaleString();
    } catch (error) {
      return String(value);
    }
  }

  function escapeHtml(value) {
    return String(value == null ? "" : value).replace(/[&<>"']/g, function (character) {
      return {
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        '"': "&quot;",
        "'": "&#39;",
      }[character];
    });
  }

  function lastCheckAt() {
    var raw = Number(GB.storageGet(GB.keys.lastCheck, "0"));
    return Number.isFinite(raw) ? raw : 0;
  }

  function setStatus(text, error) {
    var status = document.getElementById("updates-status");
    if (!status) return;
    status.style.color = error ? "#f87171" : "";
    if (error) {
      var message = String(error);
      status.textContent = window.gbT
        ? window.gbT("update-check-failed", "Update check failed: " + message, { error: message })
        : "Update check failed: " + message;
      return;
    }
    status.textContent = text || "";
  }

  // #2103 — the version comes from /api/product (window.GBUpdate.version).
  // A build without a resolvable version shows "Unknown", never a literal.
  function formatVersion(value) {
    var text = value ? String(value).trim().replace(/^v/i, "") : "";
    return text ? "v" + text : "Unknown";
  }

  function render(info, error) {
    if (error) {
      setStatus(error, true);
      return;
    }
    if (!info) return;

    if (info.update_available) {
      setStatus("");
      var status = document.getElementById("updates-status");
      if (status) {
        status.innerHTML =
          'Update available: <strong>v' +
          escapeHtml(info.latest_version) +
          '</strong> <a href="#" id="updates-release-link">' + t("update-view-release-notes", "View release notes") + '</a>';
        var link = document.getElementById("updates-release-link");
        if (link) {
          link.addEventListener("click", function (event) {
            event.preventDefault();
            showReleaseNotes(info);
          });
        }
      }
    } else {
      setStatus(
        "You are running the latest version (" +
          formatVersion(info.current_version || GB.version) +
          ")."
      );
    }

    var current = document.getElementById("updates-current-version");
    if (current) current.textContent = formatVersion(info.current_version || GB.version);
    var checked = document.getElementById("updates-last-check");
    if (checked) {
      checked.textContent = formatDate(info.checked_at || lastCheckAt());
    }
    var notesButton = document.getElementById("updates-notes-button");
    if (notesButton) notesButton.disabled = false;
  }

  function setBusy() {
    setStatus("Checking GitHub releases…");
  }

  function renderSettings() {
    var checkbox = document.getElementById("update-check-startup");
    if (checkbox) checkbox.checked = GB.startupCheckEnabled();
    var install = document.getElementById("update-install-automatic");
    if (install) install.checked = GB.autoInstallEnabled();

    var current = document.getElementById("updates-current-version");
    if (current) current.textContent = formatVersion(GB.version);
    var checked = document.getElementById("updates-last-check");
    if (checked) {
      var timestamp = lastCheckAt();
      checked.textContent = timestamp ? formatDate(timestamp) : "Never";
    }
    if (GB.lastInfo) render(GB.lastInfo);
  }

  function showReleaseNotes(info) {
    var target = info || GB.lastInfo;

    function renderNotes(notes, version) {
      var modal = document.getElementById("release-notes-modal");
      if (modal) modal.remove();
      modal = document.createElement("div");
      modal.id = "release-notes-modal";
      modal.style.cssText =
        "position:fixed;top:0;left:0;width:100%;height:100%;" +
        "background:rgba(15,23,42,0.85);display:flex;align-items:center;" +
        "justify-content:center;z-index:99999;backdrop-filter:blur(8px);";
      var text = notes || t("update-no-release-notes", "No release notes available.");
      modal.innerHTML =
        '<div style="background:#1e293b;border:1px solid #334155;border-radius:12px;' +
        'width:90%;max-width:560px;max-height:80vh;display:flex;flex-direction:column;overflow:hidden;">' +
        '<div style="display:flex;justify-content:space-between;align-items:center;padding:16px 24px;' +
        'border-bottom:1px solid #334155;"><h3 style="margin:0;color:#f8fafc;">' +
        (window.gbT
          ? window.gbT("update-release-notes-title", "Release Notes — v" + escapeHtml(version), { version: escapeHtml(version) })
          : "Release Notes — v" + escapeHtml(version)) +
        '</h3><button type="button" id="release-notes-close" style="background:none;border:none;' +
        'color:#94a3b8;font-size:24px;cursor:pointer;">×</button></div>' +
        '<div style="padding:24px;overflow-y:auto;flex:1;color:#cbd5e1;font-size:14px;' +
        'line-height:1.6;white-space:pre-wrap;">' +
        escapeHtml(text) +
        "</div></div>";
      modal.addEventListener("click", function (event) {
        if (event.target === modal) modal.remove();
      });
      document.body.appendChild(modal);
      var close = document.getElementById("release-notes-close");
      if (close) {
        close.addEventListener("click", function () {
          modal.remove();
        });
      }
    }

    if (target && target.release_notes) {
      renderNotes(target.release_notes, target.latest_version || GB.version);
      return Promise.resolve();
    }

    if (GB.isDesktop()) {
      return GB.check(true).then(function (latest) {
        renderNotes(
          latest && latest.release_notes,
          (latest && latest.latest_version) || GB.version
        );
      });
    }

    // Browser/server fallback: the historical release-notes surface.
    return fetch("/api/product", { cache: "no-store" })
      .then(function (response) {
        return response.ok ? response.json() : null;
      })
      .then(function (data) {
        var product = data || {};
        renderNotes(
          product.changelog || product.release_notes || "",
          product.version || GB.version
        );
      })
      .catch(function () {
        renderNotes("", GB.version);
      });
  }

  window.GBUpdateSettings = {
    render: render,
    renderSettings: renderSettings,
    setBusy: setBusy,
    showReleaseNotes: showReleaseNotes,
  };

  function start() {
    renderSettings();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start);
  } else {
    start();
  }
  // Settings partials arrive through HTMX after this script runs.
  document.addEventListener("htmx:afterSwap", renderSettings);
  document.addEventListener("htmx:load", renderSettings);
})();
