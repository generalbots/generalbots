(function (window) {
  "use strict";

  var PAGES = [
    { id: "os-network", endpoint: "/api/os/network", body: "os-net-status-body" },
    { id: "os-power", endpoint: "/api/os/power", body: "os-power-status-body" },
    { id: "os-display", endpoint: "/api/os/display", body: "os-display-status-body" },
    { id: "os-sound", endpoint: "/api/os/sound", body: "os-sound-status-body" },
    { id: "os-keyboard", endpoint: "/api/os/input", body: "os-keyboard-status-body" },
    { id: "os-storage-reset", endpoint: "/api/os/storage", body: "os-storage-reset-status-body" },
    { id: "os-users", endpoint: "/api/os/users", body: "os-users-status-body" },
    { id: "os-diagnostics", endpoint: "/api/os/diagnostics", body: "os-diagnostics-status-body" }
  ];

  function t(key, fallback) {
    if (typeof window._t === "function") {
      var v = window._t(key, fallback);
      if (v && v !== key) return v;
    }
    return fallback;
  }

  function byId(id) { return document.getElementById(id); }

  function setBody(id, text) {
    var el = byId(id);
    if (!el) return;
    el.textContent = text;
  }

  function loadPage(page) {
    var msgPending = t("os-backend-pending", "Waiting for OS backend…");
    var msgUnavailable = t("os-backend-unavailable", "OS backend endpoint unavailable in this build. This page is read-only.");
    setBody(page.body, msgPending);
    fetch(page.endpoint, { headers: { Accept: "application/json" }, credentials: "same-origin" })
      .then(function (r) {
        if (!r.ok) throw new Error(String(r.status));
        return r.json().catch(function () { return {}; });
      })
      .then(function (data) {
        if (data && (data.message || data.status)) {
          setBody(page.body, String(data.message || data.status));
        } else {
          setBody(page.body, t("os-backend-connected", "Connected to OS backend."));
        }
      })
      .catch(function () {
        setBody(page.body, msgUnavailable);
      });
  }

  function loadIfActive(id) {
    var page = null;
    for (var i = 0; i < PAGES.length; i++) {
      if (PAGES[i].id === id) { page = PAGES[i]; break; }
    }
    if (!page) return;
    loadPage(page);
  }

  function onHash() {
    var hash = (window.location.hash || "").replace(/^#/, "");
    loadIfActive(hash);
  }

  window.loadOsAppliancePage = loadIfActive;
  document.addEventListener("DOMContentLoaded", onHash);
  window.addEventListener("hashchange", onHash);
  document.addEventListener("htmx:load", onHash);
})(window);