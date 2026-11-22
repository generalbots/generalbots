"use strict";
/* Issue #2503 — register the Events app with the suite window manager.
 * The authoritative registry lives in js/window-manager/01_registry.js, which
 * is owned by another workstream. This launcher adds one entry at runtime from
 * desktop.html so no shared registry file needs to be edited. All rebuilds of
 * window.APPS_REGISTRY are followed by a re-add. */
(function (window) {
  "use strict";
  var APP = {
    id: "events",
    title: "Events",
    category: "system",
    color: "#0ea5e9",
    hxGet: "/suite/events/events.html",
    icon: '<path d="M22 12h-4l-3 9L9 3l-3 9H2"/>',
    enabled: true,
    preview: false,
    launcher_visibility: "always"
  };

  function ensure() {
    var registry = window.APPS_REGISTRY;
    if (!Array.isArray(registry)) return;
    for (var i = 0; i < registry.length; i += 1) {
      if (registry[i] && registry[i].id === APP.id) return;
    }
    registry.push(APP);
  }

  ensure();
  window.addEventListener("gb-apps-catalog-loaded", ensure);
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", ensure, { once: true });
  } else {
    ensure();
  }
  // One deferred pass catches a registry rebuild that lands after boot.
  window.setTimeout(ensure, 50);
})(window);
