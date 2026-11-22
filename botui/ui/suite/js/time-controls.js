"use strict";
/* #2458 — time, timezone, keyboard layout and locale controls for the OS
 * appliance. Gated OS API calls; a DOM event surfaces NTP status for UAT. */
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
    fetch("/api/os/time", {
      method: "POST", headers: headers(), credentials: "same-origin",
      body: JSON.stringify({ setting: setting, value: String(value) })
    }).then(function (r) { return r.json(); }).then(function (body) {
      document.dispatchEvent(new CustomEvent("gb:time-change", {
        detail: { setting: setting, value: value, ok: !!body.ok }
      }));
    }).catch(function () { /* web mode has no OS backend */ });
  }
  function ntpStatus(synchronized) {
    var el = id("osClockWarning");
    if (!el) return;
    el.hidden = !!synchronized;
    el.textContent = synchronized ? "" : t("os-time-unsynced", "Clock not synchronised — set it automatically or manually.");
  }
  function init() {
    document.addEventListener("gb:ntp-status", function (e) { ntpStatus(!!(e.detail && e.detail.synchronized)); });
    // #2458/#2467 — first-run step 0 persists its locale choices before
    // provisioning starts.
    document.addEventListener("gb:first-run-complete", function (e) {
      var s = e.detail || {};
      if (s.timezone) set("timezone", s.timezone);
      if (s.keyboard) set("keyboard_layout", s.keyboard);
      if (s.language) set("locale", s.language);
    });
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
  window.GBTimeControls = { set: set, ntpStatus: ntpStatus };
})(window, document);
