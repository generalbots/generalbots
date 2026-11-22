(function () {
  "use strict";
  /* #2504/#2305 — WebKit media-grant tray indicator, split from the main
   * tray module to keep the file under the 450-line module limit. */
  // #2504 — the WebKit permission handler publishes one structured event per
  // media grant. Show a red tray dot while a grant is live, with the origin
  // in the accessible label; the audit log keeps the same origin server-side.
  function showMediaGrant(payload) {
    var el = document.getElementById("trayMedia");
    if (!el) return;
    var detail = (payload && (payload.detail || payload)) || {};
    var origin = String(detail.origin || "an app").slice(0, 120);
    var kinds = [];
    if (detail.audio) kinds.push("microphone");
    if (detail.video) kinds.push("camera");
    var what = kinds.length ? kinds.join(" and ") : "microphone or camera";
    var label = what + " in use by " + origin;
    el.hidden = false;
    el.classList.add("is-recording");
    el.title = label;
    el.setAttribute("aria-label", label);
    var live = document.getElementById("osTrayLiveStatus");
    if (live) live.textContent = label;
  }

  function installMediaIndicator() {
    // DOM custom event also lets the web shell render the same indicator in
    // tests and in non-Tauri builds where the permission plugin is absent.
    document.addEventListener("gb-media-grant", function (e) { showMediaGrant(e); });
    try {
      var tauri = window.__TAURI__ || window.__TAURI_INTERNALS__;
      var listen = tauri && tauri.event && tauri.event.listen;
      if (typeof listen === "function") {
        listen("gb-media-grant", function (e) { showMediaGrant(e && e.payload); });
      }
    } catch (ignore) { /* web shell: DOM event path already installed */ }
  }

  // #2504 — the indicator is not edition-gated: a Tauri WebView can grant
  // the microphone or camera too.
  installMediaIndicator();
})();
