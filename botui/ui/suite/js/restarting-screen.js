"use strict";
/* #2450 — branded "Restarting…" screen.
 *
 * A kiosk has no terminal to fall back to: when the webview is closing for a
 * restart/shutdown, or the shell reports a crashed service, the user must see
 * a branded recovery surface instead of a black TTY. The screen is driven by
 * a DOM event (`gb:restarting`) so it is testable in a plain browser, and by
 * the Tauri `gb-restarting` event in the appliance.
 */
(function (window, document) {
  function t(key, fallback) {
    if (typeof window.gbT !== "function") return fallback;
    var value = window.gbT(key, fallback);
    // A registered key whose catalogue value has not reached this build is
    // rendered as "[key]" — never show that placeholder on a kiosk screen.
    return value && value.charAt(0) === "[" ? fallback : value;
  }
  var ID = "gb-restarting-screen";
  var node = null;

  function show(message) {
    if (!node) {
      node = document.createElement("div");
      node.id = ID;
      node.setAttribute("role", "status");
      node.setAttribute("aria-live", "assertive");
      node.innerHTML =
        '<div class="gb-restarting-card">' +
        '<div class="gb-restarting-logo" aria-hidden="true">GB</div>' +
        '<div class="gb-restarting-title"></div>' +
        '<div class="gb-restarting-detail"></div>' +
        "</div>";
      document.body.appendChild(node);
    }
    var title = node.querySelector(".gb-restarting-title");
    var detail = node.querySelector(".gb-restarting-detail");
    if (title) title.textContent = t("shell-restarting-title", "Restarting…");
    if (detail) detail.textContent = message || t("shell-restarting-hint", "The appliance is restarting. Please wait.");
    node.classList.add("is-visible");
    node.hidden = false;
  }

  function hide() {
    if (!node) return;
    node.classList.remove("is-visible");
    node.hidden = true;
  }

  document.addEventListener("gb:restarting", function (e) {
    show(e && e.detail ? e.detail.message : null);
  });

  function init() {
    var tauri = window.__TAURI__;
    if (tauri && tauri.event && typeof tauri.event.listen === "function") {
      tauri.event.listen("gb-restarting", function (payload) {
        show(payload && payload.message);
      });
    }
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }

  window.GBRestartingScreen = { show: show, hide: hide };
})(window, document);
