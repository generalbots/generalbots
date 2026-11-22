"use strict";

/* #2504 — Settings → Privacy device switches. In the desktop shell these call
   the Tauri command that flips the OS-level permission flags; in the web shell
   the switches persist in localStorage and the status explains the scope. */
(function (window) {
  function t(key, fallback) {
    return typeof window._t === "function" ? window._t(key, fallback) : fallback;
  }

  function invoke(kind, enabled) {
    var tauri = window.GBTAURI || window.__TAURI_INTERNALS__;
    var invokeFn = tauri && (tauri.invoke || (tauri.core && tauri.core.invoke));
    if (typeof invokeFn === "function") {
      return invokeFn("set_media_permission", { kind: kind, enabled: enabled });
    }
    try {
      window.localStorage.setItem("gb-media-permission-" + kind, enabled ? "1" : "0");
    } catch (ignore) { /* storage disabled: keep the in-memory switch */ }
    return Promise.resolve();
  }

  function announce(message) {
    var el = document.getElementById("mediaPermissionStatus");
    if (el) el.textContent = message;
  }

  // Delegated: the Privacy section is injected by HTMX after this script
  // runs, so binding directly at DOMContentLoaded misses the switches.
  document.addEventListener("change", function (event) {
    var input = event.target;
    if (!input || !input.matches || !input.matches("[data-media-device]")) return;
    var kind = input.getAttribute("data-media-device");
    invoke(kind, input.checked).then(function () {
      announce((kind === "audio" ? t("settings-media-microphone", "Microphone") : t("settings-media-camera", "Camera")) +
        ": " + (input.checked ? t("label-enabled", "enabled") : t("label-disabled", "disabled")));
    }).catch(function (error) {
      announce(String(error && error.message ? error.message : error));
    });
  });
})(window);
