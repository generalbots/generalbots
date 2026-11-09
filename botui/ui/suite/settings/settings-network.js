/* #1865 — Settings → Network.
 *
 * Reads/writes the network mode (Offline or P2P) and mirrors it into
 * localStorage key "gb_peer_network_mode". The switch is a hard gate: it calls
 * POST /api/peer/{enable,disable}, which stops or starts every peer surface and
 * persists GB_NETWORK_MODE in the stack .env so the choice survives a restart.
 * The relay is managed automatically.
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

  function token() {
    try {
      return (window.getGBAccessToken && window.getGBAccessToken()) ||
        window.localStorage.getItem("gb-access-token") ||
        window.sessionStorage.getItem("gb-access-token") ||
        "";
    } catch (e) {
      return "";
    }
  }

  /* #1865 — flip the backend gate, not just a browser preference. Resolves
   * with the server's `enabled` value, or null when the call failed so the
   * caller can say the switch did not apply. */
  function postMode(mode) {
    var headers = { "Content-Type": "application/json" };
    var value = token();
    if (value) { headers.Authorization = "Bearer " + value; }
    return fetch("/api/peer/" + (mode === "p2p" ? "enable" : "disable"), {
      method: "POST",
      headers: headers,
      cache: "no-store"
    }).then(function (response) {
      return response.ok ? response.json() : null;
    }).then(function (data) {
      return data && typeof data.enabled === "boolean" ? data.enabled : null;
    }).catch(function () {
      return null;
    });
  }

  /* One entry point for both switches (Settings → Network and Settings →
   * Shared Links), so the UI state, the runtime gate and the nav visibility
   * never disagree. */
  function applyMode(chosen) {
    var mode = chosen === "p2p" ? "p2p" : "offline";
    saveMode(mode);
    updateBadge(mode);
    document.dispatchEvent(new CustomEvent("gb-network-mode-changed", { detail: { mode: mode } }));
    return postMode(mode);
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
      if (statusEl) {
        statusEl.textContent = "Applying " + chosen + "…";
        statusEl.className = "note";
      }
      applyMode(chosen).then(function (enabled) {
        if (statusEl) {
          if (enabled === null) {
            statusEl.textContent = "Could not reach the server: the mode is " +
              "remembered and applies on the next start.";
            statusEl.className = "note warn";
          } else {
            statusEl.textContent = enabled
              ? "P2P is on: this desktop is reachable and can share links."
              : "Offline: no listeners, no identity, no links — stopped now.";
            statusEl.className = "note ok";
          }
        }
        if (checkedEl) {
          checkedEl.textContent = "Updated at " + new Date().toLocaleTimeString();
        }
      });
    });
  }

  window.loadNetworkSettings = init;
  window.GBNetworkMode = { read: readMode, apply: applyMode };
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
