"use strict";
/* #2460 — Drive removable-media panel. Lists mounted devices reported by the
 * OS API, offers a one-action import using Drive's existing folder-upload
 * pipeline (with progress) and a safe eject that surfaces "in use". */
(function (window, document) {
  function t(key, fallback) {
    return window.gbT ? window.gbT(key, fallback) : fallback;
  }
  function tokenHeaders() {
    var h = { Accept: "application/json" };
    var tok = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
    if (tok) h.Authorization = "Bearer " + tok;
    return h;
  }
  function panel() {
    var p = document.getElementById("driveRemovablePanel");
    if (p) return p;
    p = document.createElement("section");
    p.id = "driveRemovablePanel";
    p.className = "drive-removable-panel";
    p.setAttribute("role", "region");
    p.setAttribute("aria-label", t("drive-removable-title", "Removable devices"));
    p.innerHTML = '<h2 data-i18n="drive-removable-title"></h2>' +
      '<div id="driveRemovableList" role="status" aria-live="polite"></div>';
    var host = document.getElementById("driveSidebar") || document.querySelector(".drive-sidebar") || document.body;
    host.appendChild(p);
    return p;
  }
  function deviceNode(dev) {
    var row = document.createElement("div");
    row.className = "drive-removable-device";
    var name = document.createElement("span");
    name.textContent = dev.label || dev.name || "device";
    var meta = document.createElement("small");
    meta.textContent = [dev.fstype, dev.size, dev.trans].filter(Boolean).join(" · ");
    row.appendChild(name);
    row.appendChild(meta);
    var importBtn = document.createElement("button");
    importBtn.type = "button";
    importBtn.textContent = t("drive-removable-import", "Import to Drive");
    importBtn.addEventListener("click", function () {
      // Reuse Drive's folder picker + upload progress pipeline.
      var input = document.getElementById("folder-input");
      if (input) { input.click(); return; }
      showStatus(t("drive-removable-picker-missing", "Use Upload folder to import this device."));
    });
    var ejectBtn = document.createElement("button");
    ejectBtn.type = "button";
    ejectBtn.textContent = t("drive-removable-eject", "Eject");
    ejectBtn.addEventListener("click", function () { eject(dev.name || dev.label); });
    row.appendChild(importBtn);
    row.appendChild(ejectBtn);
    return row;
  }
  function showStatus(msg) {
    var list = document.getElementById("driveRemovableList");
    if (list) list.textContent = msg;
  }
  function eject(device) {
    fetch("/api/os/removable/eject", {
      method: "POST", headers: Object.assign({ "Content-Type": "application/json" }, tokenHeaders()),
      credentials: "same-origin", body: JSON.stringify({ device: device })
    }).then(function (r) { return r.json(); }).then(function (body) {
      if (body.ok) showStatus(t("drive-removable-ejected", "Device safely ejected."));
      else showStatus(t("drive-removable-in-use", "Device is in use. Close its files and try again."));
    }).catch(function () { showStatus(t("drive-removable-eject-failed", "Could not eject the device.")); });
  }
  function load() {
    fetch("/api/os/removable", { headers: tokenHeaders(), credentials: "same-origin" })
      .then(function (r) { if (!r.ok) throw new Error("HTTP " + r.status); return r.json(); })
      .then(function (data) {
        var list = document.getElementById("driveRemovableList");
        if (!list) return;
        list.textContent = "";
        var devices = (data.devices && data.devices.blockdevices) || [];
        if (!devices.length) {
          list.textContent = t("drive-removable-none", "No removable devices detected.");
          return;
        }
        devices.forEach(function (d) { list.appendChild(deviceNode(d)); });
      })
      .catch(function (e) { showStatus(t("drive-removable-unavailable", "Removable devices unavailable: ") + e.message); });
  }
  function init() {
    if (!document.getElementById("driveRemovablePanel")) { panel(); load(); }
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
  window.DriveRemovable = { load: load, eject: eject };
})(window, document);
