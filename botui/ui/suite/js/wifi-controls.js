"use strict";
/* #2453 — Wi-Fi scan/join/saved-network UI and tray state. Gated OS API; DOM
 * events let the web shell simulate NetworkManager results and captive portals. */
(function (window, document) {
  function t(key, fallback) { return window.gbT ? window.gbT(key, fallback) : fallback; }
  function id(x) { return document.getElementById(x); }
  function headers() {
    var h = { "Content-Type": "application/json", Accept: "application/json" };
    var tok = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
    if (tok) h.Authorization = "Bearer " + tok;
    return h;
  }
  function action(payload) {
    return fetch("/api/os/network/wifi", {
      method: "POST", headers: headers(), credentials: "same-origin",
      body: JSON.stringify(payload)
    }).then(function (r) { return r.json(); });
  }
  function join() {
    var ssid = id("osWifiSsid"), pwd = id("osWifiPassword"), hidden = id("osWifiHidden");
    if (!ssid || !ssid.value.trim()) return;
    action({ action: "join", ssid: ssid.value.trim(),
      password: pwd && pwd.value ? pwd.value : null,
      hidden: !!(hidden && hidden.checked) }).then(function (body) {
      document.dispatchEvent(new CustomEvent("gb:wifi-change", { detail: { action: "join", ok: !!body.ok, ssid: ssid.value } }));
    }).catch(function () { /* web mode */ });
  }
  function scan() {
    fetch("/api/os/network/wifi", { headers: headers(), credentials: "same-origin" })
      .then(function (r) { return r.json(); }).then(render).catch(function () { /* web mode */ });
  }
  function render(data) {
    var list = id("osWifiNetworks"), details = id("osWifiDetails");
    if (list && Array.isArray(data.networks)) {
      list.textContent = data.networks.map(function (n) {
        return n.ssid || n.SSID || "network";
      }).join("\n") || t("os-wifi-none", "No networks found.");
    }
    if (details && data.ip_dns) details.textContent = String(data.ip_dns).replace(/\n/g, " · ");
  }
  function captive(active) {
    var el = id("osWifiCaptive");
    if (el) el.hidden = !active;
  }
  function openPortal() {
    document.dispatchEvent(new CustomEvent("gb:wifi-portal", { detail: { action: "open" } }));
  }
  function trayState(state) {
    var btn = id("trayNetworkStatus");
    if (btn) btn.setAttribute("data-network-state", state);
  }
  function init() {
    document.addEventListener("gb:wifi-networks", function (e) { render(e.detail || {}); });
    document.addEventListener("gb:wifi-captive", function (e) { captive(!!(e.detail && e.detail.active)); });
    document.addEventListener("gb:wifi-tray", function (e) { trayState(e.detail && e.detail.state); });
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
  window.GBWifiControls = { join: join, scan: scan, render: render, captive: captive, openPortal: openPortal, trayState: trayState };
})(window, document);
