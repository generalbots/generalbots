"use strict";
/* #2502 — an appliance page must not start a screencast without an explicit
 * user gesture. Wrap getDisplayMedia so it is refused unless the user has just
 * activated the in-app "Share screen" action, and keep the tray indicator
 * visible for the whole capture. */
(function (window, document) {
  var armedAt = 0;
  var ARM_WINDOW_MS = 5000;
  function t(key, fallback) { return window.gbT ? window.gbT(key, fallback) : fallback; }
  function arm() {
    armedAt = Date.now();
    document.dispatchEvent(new CustomEvent("gb:screen-capture-armed", { detail: { at: armedAt } }));
  }
  function indicator(active) {
    var dot = document.getElementById("trayMedia");
    if (dot) {
      dot.hidden = !active;
      dot.classList.toggle("is-recording", active);
      dot.setAttribute("aria-label", active
        ? t("os-capture-active", "Screen capture in progress") : "");
    }
    document.dispatchEvent(new CustomEvent("gb:screen-capture", { detail: { active: active } }));
  }
  function init() {
    var md = navigator.mediaDevices;
    if (!md || !md.getDisplayMedia || md.__gbGuarded) return;
    var original = md.getDisplayMedia.bind(md);
    md.__gbGuarded = true;
    md.getDisplayMedia = function (constraints) {
      if (Date.now() - armedAt > ARM_WINDOW_MS || !armedAt) {
        document.dispatchEvent(new CustomEvent("gb:screen-capture-denied", { detail: {} }));
        return Promise.reject(new DOMException("User gesture required", "NotAllowedError"));
      }
      armedAt = 0;
      return original(constraints).then(function (stream) {
        indicator(true);
        if (stream && stream.getTracks) {
          stream.getTracks().forEach(function (track) {
            track.addEventListener("ended", function () { indicator(false); });
          });
        }
        return stream;
      });
    };
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
  window.GBScreenCaptureGuard = { arm: arm };
})(window, document);
