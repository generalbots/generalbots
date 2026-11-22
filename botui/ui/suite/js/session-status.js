"use strict";
/* #2449 — kiosk user-manager session status. Gated OS API; DOM event lets the
 * web shell simulate the session report without a systemd host. */
(function (window, document) {
  function t(key, fallback, params) {
    var value = window.gbT ? window.gbT(key, fallback, params) : fallback;
    if (!value || value.charAt(0) === "[") value = String(fallback);
    return String(value).replace(/\{(\w+)\}/g, function (_, k) {
      return params && params[k] != null ? String(params[k]) : "{" + k + "}";
    });
  }
  function id(x) { return document.getElementById(x); }
  function headers() {
    var h = { Accept: "application/json" };
    var tok = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
    if (tok) h.Authorization = "Bearer " + tok;
    return h;
  }
  function render(data) {
    var el = id("osSessionStatus");
    if (!el) return;
    var manager = data.user_manager || "unknown";
    var runtime = data.xdg_runtime_dir || "unset";
    var services = (data.user_services || []).map(function (s) {
      return s.unit + ": " + (s.active || "unknown");
    }).join("\n");
    el.textContent = t("os-session-report", "User manager: {manager}\nXDG_RUNTIME_DIR: {runtime}\n{services}", {
      manager: manager, runtime: runtime, services: services
    });
    el.setAttribute("data-manager-active", /running|degraded/.test(manager) ? "true" : "false");
  }
  function refresh() {
    fetch("/api/os/session", { headers: headers(), credentials: "same-origin" })
      .then(function (r) { return r.ok ? r.json() : {}; }).then(render)
      .catch(function () { /* web mode */ });
  }
  function init() {
    document.addEventListener("gb:session-status", function (e) { render(e.detail || {}); });
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
  window.GBSessionStatus = { refresh: refresh, render: render };
})(window, document);
