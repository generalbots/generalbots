"use strict";
/* #2464 — power profile selector and fwupd firmware page. Driven by gated OS
 * APIs; DOM events let the web shell simulate device lists and results. */
(function (window, document) {
  function t(key, fallback) { return window.gbT ? window.gbT(key, fallback) : fallback; }
  function id(x) { return document.getElementById(x); }
  function headers() {
    var h = { "Content-Type": "application/json", Accept: "application/json" };
    var tok = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
    if (tok) h.Authorization = "Bearer " + tok;
    return h;
  }
  function setProfile(profile) {
    fetch("/api/os/power-profile", {
      method: "POST", headers: headers(), credentials: "same-origin",
      body: JSON.stringify({ profile: profile })
    }).then(function (r) { return r.json(); }).then(function (body) {
      document.dispatchEvent(new CustomEvent("gb:power-profile", { detail: { profile: profile, ok: !!body.ok } }));
    }).catch(function () { /* web mode */ });
  }
  function firmware(action) {
    fetch("/api/os/firmware", {
      method: "POST", headers: headers(), credentials: "same-origin",
      body: JSON.stringify({ action: action })
    }).then(function (r) { return r.json(); }).then(function (body) {
      var restart = id("osFirmwareRestart");
      if (restart && action === "install" && body.restart_required) restart.hidden = false;
      document.dispatchEvent(new CustomEvent("gb:firmware-action", {
        detail: { action: action, ok: !!body.ok, restart_required: !!body.restart_required }
      }));
    }).catch(function () { /* web mode */ });
  }
  function showDevices(devices) {
    var el = id("osFirmwareDevices");
    if (!el) return;
    el.textContent = (devices && devices.length)
      ? devices.map(function (d) { return (d.name || d.id || "device") + " — " + (d.version || "unknown"); }).join("\n")
      : t("os-firmware-none", "No updateable devices detected.");
  }
  function init() {
    document.addEventListener("gb:firmware-devices", function (e) { showDevices(e.detail && e.detail.devices); });
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
  window.GBPowerFirmware = { setProfile: setProfile, firmware: firmware, showDevices: showDevices };
})(window, document);
