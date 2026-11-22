"use strict";
/* #2477 — opt-in LAN exposure switch for the appliance. Off by default,
 * time-boxed, with a visible warning while active. The browser-side state is
 * driven by the gated /api/os/network/lan-exposure endpoint; a DOM event lets
 * the web shell reflect the same indicator for UAT. */
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
    var h = { "Content-Type": "application/json", Accept: "application/json" };
    var tok = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
    if (tok) h.Authorization = "Bearer " + tok;
    return h;
  }
  function reflect(enabled, remaining) {
    var warn = id("osLanExposureWarning");
    var check = id("osLanExposure");
    if (check) check.checked = !!enabled;
    document.body.classList.toggle("gb-lan-exposure-active", !!enabled);
    if (warn) {
      warn.hidden = !enabled;
      warn.textContent = enabled
        ? t("os-net-lan-active", "Other devices can reach this suite for {n} min", { n: Math.ceil((remaining || 900) / 60) })
        : "";
    }
    document.dispatchEvent(new CustomEvent("gb:lan-exposure", {
      detail: { enabled: !!enabled, remaining_seconds: remaining || 0 }
    }));
  }
  function refresh() {
    fetch("/api/os/network/lan-exposure", { headers: headers(), credentials: "same-origin" })
      .then(function (r) { return r.ok ? r.json() : {}; })
      .then(function (d) { reflect(!!d.enabled, d.remaining_seconds); })
      .catch(function () { /* keep the last known state */ });
  }
  function setLanExposure(enabled) {
    fetch("/api/os/network/lan-exposure", {
      method: "POST", headers: headers(), credentials: "same-origin",
      body: JSON.stringify({ enabled: !!enabled })
    }).then(function (r) { return r.ok ? r.json() : {}; })
      .then(function (d) { reflect(!!d.enabled, d.remaining_seconds); })
      .catch(function () { reflect(false, 0); });
  }
  function init() {
    refresh();
    document.addEventListener("gb:lan-exposure-set", function (e) {
      reflect(!!(e.detail && e.detail.enabled), e.detail && e.detail.remaining_seconds);
    });
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
  window.GBNetworkSecurity = { refresh: refresh, setLanExposure: setLanExposure, reflect: reflect };
})(window, document);
