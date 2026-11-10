/*
 * #2089 — localized application metadata.
 *
 * The catalog ships English `title`/`description` literals from
 * `botserver/src/apps/registry`; every key in `botlib/locales/<locale>/apps.ftl`
 * (`app-<id>-name` / `app-<id>-desc`) overrides them. Renderers that build
 * their own DOM (launcher, spotlight, pinned chips, taskbar previews, alt-tab,
 * window chrome) call these helpers instead of reading `app.title` directly,
 * so a locale change is picked up by the next render.
 *
 * Load after `/suite/js/i18n.js` (it provides `window.gbT`).
 */
(function () {
  "use strict";

  function fallbackTitle(app) {
    return (app && (app.title || app.name || app.id)) || "";
  }

  function title(app) {
    if (!app) return "";
    var id = app.id || app.appId;
    if (!id) return fallbackTitle(app);
    var key = "app-" + id + "-name";
    return window.gbT ? window.gbT(key, fallbackTitle(app)) : fallbackTitle(app);
  }

  function description(app) {
    if (!app) return "";
    var id = app.id || app.appId;
    var literal = app.description || "";
    if (!id) return literal;
    var key = "app-" + id + "-desc";
    return window.gbT ? window.gbT(key, literal) : literal;
  }

  // Keeps window titles, dock labels and the document title in sync when the
  // user switches language without a reload.
  function appForWindow(id, byId) {
    if (byId[id]) return byId[id];
    // Force-new windows carry a unique suffix (`chat-3f9k2`); match the
    // longest registry id the window id starts with.
    var match = null;
    Object.keys(byId).forEach(function (key) {
      if (id.indexOf(key) === 0 && (!match || key.length > match.length)) match = key;
    });
    return match ? byId[match] : null;
  }

  function refreshWindowTitles() {
    var wm = window.WindowManager;
    if (!wm || typeof wm.listWindows !== "function") return;
    var registry = window.APPS_REGISTRY || [];
    var byId = {};
    registry.forEach(function (app) {
      byId[app.id] = app;
    });
    (wm.listWindows() || []).forEach(function (win) {
      if (!win || !win.id) return;
      var app = appForWindow(String(win.id), byId);
      if (!app) return;
      var next = title(app);
      win.title = next;
      var element = document.getElementById("window-" + win.id);
      var label = element && element.querySelector(".window-title");
      if (label) label.textContent = next;
      var dock = document.getElementById("dock-item-" + win.id);
      if (dock) dock.title = next;
    });
  }

  window.gbAppTitle = title;
  window.gbAppDesc = description;

  window.addEventListener("localeChanged", function () {
    refreshWindowTitles();
    if (window.GBPinnedLauncher && window.GBPinnedLauncher.render) {
      try {
        window.GBPinnedLauncher.render();
      } catch (e) {
        console.warn("app-i18n: pinned launcher re-render failed", e);
      }
    }
  });
})();
