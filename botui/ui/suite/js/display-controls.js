"use strict";
/* #2456 — Display settings: brightness, resolution, scale, rotation, output
 * mode, night light and auto-rotate. Driven by the gated OS display API; a DOM
 * event lets the web shell simulate hot-plug and command results for UAT. */
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
    if (setting === "brightness") {
      var out = id("osBrightnessValue");
      if (out) out.textContent = value + "%";
    }
    fetch("/api/os/display/controls", {
      method: "POST", headers: headers(), credentials: "same-origin",
      body: JSON.stringify({ setting: setting, value: String(value) })
    }).then(function (r) { return r.json(); }).then(function (body) {
      document.dispatchEvent(new CustomEvent("gb:display-change", {
        detail: { setting: setting, value: value, ok: !!body.ok }
      }));
    }).catch(function () { /* the OS backend may be unavailable in web mode */ });
  }
  function hotplug(outputs) {
    var el = id("osDisplayHotplug");
    if (!el) return;
    var external = (outputs || []).filter(function (o) {
      return o && o.connected && !/eDP|LVDS/.test(o.name || "");
    });
    el.textContent = external.length
      ? t("os-display-hotplug-detected", "External display connected — extend mode available")
      : t("os-display-hotplug-none", "No external display detected");
  }
  function init() {
    document.addEventListener("gb:display-hotplug", function (e) { hotplug(e.detail && e.detail.outputs); });
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
  window.GBDisplayControls = { set: set, hotplug: hotplug };
})(window, document);
