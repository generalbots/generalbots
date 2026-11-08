/* #1865 — Settings → Network.
 *
 * Reads/writes the runtime network mode preference (Offline or P2P) via
 * localStorage key "gb_peer_network_mode". The relay is managed automatically;
 * restart the app to apply a permanent change.
 */
(function (window) {
  "use strict";

  var STORAGE_KEY = "gb_peer_network_mode";

  function byId(id) {
    return document.getElementById(id);
  }

  function readMode() {
    try {
      return (window.localStorage && window.localStorage.getItem(STORAGE_KEY)) || "offline";
    } catch (e) {
      return "offline";
    }
  }

  function saveMode(mode) {
    try {
      if (window.localStorage) {
        window.localStorage.setItem(STORAGE_KEY, mode);
      }
    } catch (e) {}
  }

  function updateBadge(mode) {
    var badge = byId("network-mode-badge");
    if (!badge) { return; }
    if (mode === "p2p") {
      badge.className = "shared-status shared-status-ready";
      badge.innerHTML = '<span class="shared-status-dot"></span>P2P';
    } else {
      badge.className = "shared-status shared-status-off";
      badge.innerHTML = '<span class="shared-status-dot"></span>Offline';
    }
  }

  function init() {
    var section = byId("network-section");
    if (!section) { return; }

    var select = byId("network-mode-select");
    var applyBtn = byId("network-mode-apply");
    var statusEl = byId("network-mode-status");
    var checkedEl = byId("network-checked");

    if (!select || !applyBtn) { return; }
    if (applyBtn.getAttribute("data-wired") === "1") { return; }
    applyBtn.setAttribute("data-wired", "1");

    var current = readMode();
    select.value = current === "p2p" ? "p2p" : "offline";
    updateBadge(current);

    if (checkedEl) {
      checkedEl.textContent = "Loaded at " + new Date().toLocaleTimeString();
    }

    applyBtn.addEventListener("click", function () {
      var chosen = select.value === "p2p" ? "p2p" : "offline";
      saveMode(chosen);
      updateBadge(chosen);
      if (statusEl) {
        statusEl.textContent = "Network mode set to " + chosen +
          ". Restart the app for the change to take effect.";
        statusEl.className = "note ok";
      }
      if (checkedEl) {
        checkedEl.textContent = "Updated at " + new Date().toLocaleTimeString();
      }
    });
  }

  window.loadNetworkSettings = init;

  document.addEventListener("htmx:load", function () {
    if (byId("network-section")) { init(); }
  });

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})(window);
