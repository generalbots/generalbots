/* #1865 — Settings → Network.
 *
 * Reads/writes the runtime network mode preference (Offline or P2P) via
 * localStorage key "gb_peer_network_mode". The relay is managed automatically;
 * restart the app to apply a permanent change.
 */
(function (window) {
  "use strict";

  var STORAGE_KEY = "gb_peer_network_mode";

  /* #1969 — peer and home-site resolution always goes through one fixed
   * relay. The backend publishes it in `/api/product`; the constant below is
   * only the fallback for a build that predates the manifest field. */
  var DEFAULT_RELAY = "peer.generalbots.org";

  function renderRelay(host) {
    var resolved = host || DEFAULT_RELAY;
    window.GB_PEER_RELAY = resolved;
    var el = byId("network-relay-host");
    if (el) { el.textContent = resolved; }
  }

  function loadRelay() {
    renderRelay(DEFAULT_RELAY);
    fetch("/api/product", { cache: "no-store" })
      .then(function (response) {
        return response.ok ? response.json() : null;
      })
      .then(function (data) {
        if (data && data.peer && data.peer.relay) {
          renderRelay(String(data.peer.relay));
        }
      })
      .catch(function () {});
  }

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
  loadRelay();

  document.addEventListener("htmx:load", function () {
    if (byId("network-section")) { init(); }
  });

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})(window);
