"use strict";
/* #2462 — touchpad, Bluetooth and printer controls for the kiosk. Gated OS
 * API calls; DOM events let the web shell simulate devices and tray state. */
(function (window, document) {
  function t(key, fallback) { return window.gbT ? window.gbT(key, fallback) : fallback; }
  function id(x) { return document.getElementById(x); }
  function headers() {
    var h = { "Content-Type": "application/json", Accept: "application/json" };
    var tok = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
    if (tok) h.Authorization = "Bearer " + tok;
    return h;
  }
  function set(setting, value) {
    var preview = id("osTouchpadPreview");
    if (preview) preview.setAttribute("data-" + setting, String(value));
    fetch("/api/os/peripherals", {
      method: "POST", headers: headers(), credentials: "same-origin",
      body: JSON.stringify({ setting: setting, value: String(value) })
    }).then(function (r) { return r.json(); }).then(function (body) {
      document.dispatchEvent(new CustomEvent("gb:peripheral-change", {
        detail: { setting: setting, value: value, ok: !!body.ok }
      }));
    }).catch(function () { /* web mode */ });
  }
  function scan() {
    fetch("/api/os/peripherals", { headers: headers(), credentials: "same-origin" })
      .then(function (r) { return r.json(); }).then(render).catch(function () { /* web mode */ });
  }
  function render(data) {
    var bt = id("osBtDevices"), printers = id("osPrinters");
    if (bt && data.devices) bt.textContent = String(data.devices).split("\n").filter(Boolean).join("\n") || t("os-periph-no-bt", "No Bluetooth devices.");
    if (printers && data.printers) printers.textContent = String(data.printers).split("\n").filter(Boolean).join("\n") || t("os-periph-no-printers", "No printers configured.");
    var toggle = id("osBluetoothOn");
    if (toggle && data.bluetooth) toggle.checked = String(data.bluetooth).indexOf("yes") >= 0 || String(data.bluetooth).indexOf("on") >= 0;
  }
  function init() {
    document.addEventListener("gb:peripherals-status", function (e) { render(e.detail || {}); });
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
  window.GBPeripherals = { set: set, scan: scan, render: render };
})(window, document);
