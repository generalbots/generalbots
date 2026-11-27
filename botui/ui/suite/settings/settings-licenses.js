"use strict";
/* Settings > Licenses — port of the assistant shell's 09g_settings_licenses.js.
 *
 * Reads the byte-identical copies the suite serves from /suite/about/licenses/
 * (kept in step by scripts/check_third_party_licenses.py). No external fetch,
 * no CDN: the texts ship with the install.
 */
(function () {
  var BASE = "/suite/about/licenses/";
  var texts = null;
  var failed = false;
  var open = false;
  var loading = null;

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }
  function esc(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;")
      .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }
  function readText(name) {
    return fetch(BASE + name, { cache: "no-store", credentials: "same-origin" })
      .then(function (response) {
        var type = response.headers.get("content-type") || "";
        if (!response.ok || type.indexOf("text/html") === 0) {
          throw new Error("licenses: " + name + " is not in this build");
        }
        return response.text();
      });
  }
  function load() {
    if (!loading) {
      loading = Promise.all([readText("NOTICE"), readText("LICENSES-desktop.md")])
        .then(function (parts) {
          texts = { notice: parts[0], record: parts[1] };
          failed = false;
        })
        .catch(function () {
          failed = true;
          loading = null;
        });
    }
    return loading;
  }
  function title() { return t("settings-licenses-title", "Third-party licenses"); }
  function toggleLabel() {
    return open ? t("settings-licenses-hide", "Hide licenses") : t("settings-licenses-show", "Show licenses");
  }
  function body() {
    if (failed) {
      return '<p class="setting-desc" role="status" aria-live="polite">' +
        esc(t("settings-licenses-failed",
          "The licenses could not be opened in this build. The texts are in the licenses folder of the install.")) +
        "</p>";
    }
    if (!texts) {
      return '<p class="setting-desc" role="status" aria-live="polite">' +
        esc(t("settings-licenses-loading", "Loading…")) + "</p>";
    }
    return "<h3>" + esc(t("settings-licenses-notices", "Notices")) + "</h3>" +
      '<pre class="gb-lic-text" tabindex="0">' + esc(texts.notice) + "</pre>" +
      "<h3>" + esc(t("settings-licenses-components", "Component record")) + "</h3>" +
      '<pre class="gb-lic-text" tabindex="0">' + esc(texts.record) + "</pre>";
  }

  function renderLicenses() {
    var panel = document.getElementById("licenses-card");
    if (!panel) return;
    panel.innerHTML = '<div class="setting-card"><div class="section-header"><h2>' +
      esc(title()) + "</h2></div><p class=\"setting-desc\">" +
      esc(t("settings-licenses-intro",
        "The app uses programs by other authors, each under its own license. Full texts are also in the licenses folder of the install.")) +
      '</p><button type="button" class="btn-secondary btn-sm" id="gbLicToggle" ' +
      'aria-controls="gbLicBody" aria-expanded="' + (open ? "true" : "false") + '">' +
      esc(toggleLabel()) + '</button><div id="gbLicBody"' + (open ? "" : " hidden") + ">" +
      (open ? body() : "") + "</div></div>";
    var button = document.getElementById("gbLicToggle");
    var target = document.getElementById("gbLicBody");
    if (!button || !target) return;
    button.addEventListener("click", function () {
      open = !open;
      target.hidden = !open;
      target.innerHTML = open ? body() : "";
      button.setAttribute("aria-expanded", open ? "true" : "false");
      button.textContent = toggleLabel();
      if (open && !texts && !failed) {
        load().then(function () {
          if (target.isConnected && open) {
            target.innerHTML = body();
          }
        });
      }
    });
  }

  window.loadLicensesPanel = renderLicenses;
  var previousHook = window.GBSettingsOnSection;
  window.GBSettingsOnSection = function (section) {
    if (typeof previousHook === "function") {
      try { previousHook(section); } catch (error) { /* optional hook */ }
    }
    if (section === "licenses") renderLicenses();
  };
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", renderLicenses);
  } else {
    renderLicenses();
  }
})();
