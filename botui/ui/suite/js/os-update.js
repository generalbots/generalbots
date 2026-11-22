"use strict";
/* #2443 — OS A/B slot update UI: channel, size, release notes, rollback
 * indicator and offline USB install. Gated API + DOM events for simulation. */
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
    var el = id("osUpdateDetails");
    if (!el) return;
    var slot = data.active_slot || "unversioned";
    el.textContent = t("os-update-slot", "Active slot: {slot} · channel: {channel} · rollback: {rollback}", {
      slot: slot, channel: data.channel || "stable", rollback: data.rollback_supported ? "yes" : "no"
    });
    var usb = id("osUpdateUsb");
    if (usb) usb.disabled = !data.offline_update_available;
    var size = id("osUpdateSize");
    if (size) size.textContent = data.size || "—";
  }
  function refresh() {
    fetch("/api/os/update/status", { headers: headers(), credentials: "same-origin" })
      .then(function (r) { return r.ok ? r.json() : {}; }).then(render)
      .catch(function () { /* web mode */ });
  }
  function setChannel(channel) {
    document.dispatchEvent(new CustomEvent("gb:update-channel", { detail: { channel: channel } }));
  }
  function installOffline() {
    document.dispatchEvent(new CustomEvent("gb:update-offline", { detail: { source: "usb" } }));
  }
  function init() {
    document.addEventListener("gb:update-status", function (e) { render(e.detail || {}); });
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
  window.GBOSUpdate = { refresh: refresh, render: render, setChannel: setChannel, installOffline: installOffline };
})(window, document);
