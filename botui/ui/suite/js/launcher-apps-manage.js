"use strict";

// "Manage apps" panel for the desktop left rail. The rail only draws the apps
// the user left ON, so a hidden app needs a discoverable way back: this compact
// checklist lists EVERY app the rail may show (preview rules included), with an
// ON/OFF switch per row. State lives in js/launcher-apps-prefs.js
// (localStorage["gb_launcher_disabled_apps"]); APPS_REGISTRY is never modified.

window.GBLauncherAppsManage = window.GBLauncherAppsManage || {};

(function (mod) {
  var PANEL_CLASS = "gb-side-apps-manage-panel";
  var ROW_CLASS = "gb-side-manage-row";

  function tr(key, fallback, params) {
    return window.gbT ? window.gbT(key, fallback, params) : fallback;
  }

  function prefs() {
    return window.GBLauncherAppsPrefs;
  }

  function hostOf() {
    return document.getElementById("sidebarAppsNav");
  }

  function panelOf(host) {
    return host ? host.querySelector("." + PANEL_CLASS) : null;
  }

  // Same set the rail draws from (before the user's ON/OFF filter), so an app
  // that is currently hidden is still listed here and can be turned back on.
  function railApps() {
    if (window.GBSidebarApps && typeof window.GBSidebarApps.railApps === "function") {
      return window.GBSidebarApps.railApps();
    }
    return (window.APPS_REGISTRY || []).slice();
  }

  function appName(app) {
    return app.title || app.id;
  }

  function applyExpanded(host, open) {
    var btn = host && host.querySelector(".gb-side-manage-btn");
    if (btn) btn.setAttribute("aria-expanded", open ? "true" : "false");
  }

  function sync(panel) {
    if (!panel) return;
    var p = prefs();
    var apps = railApps();
    var on = 0;
    panel.querySelectorAll("." + ROW_CLASS).forEach(function (row) {
      var id = row.getAttribute("data-app-id");
      var box = row.querySelector("input[type=checkbox]");
      var enabled = !p || p.isEnabled(id);
      if (box) box.checked = enabled;
      if (enabled) on += 1;
    });
    var summary = panel.querySelector(".gb-side-manage-summary");
    if (summary) {
      summary.textContent = tr("sidebar-apps-manage-summary", "{on} of {total} shown", {
        on: on,
        total: apps.length,
      });
    }
  }

  mod.render = function (host) {
    host = host || hostOf();
    if (!host) return null;
    var p = prefs();
    var panel = panelOf(host);
    if (!panel) {
      panel = document.createElement("div");
      panel.className = PANEL_CLASS;
      panel.setAttribute("role", "group");
      panel.setAttribute("aria-label", tr("sidebar-apps-manage", "Manage apps"));
      host.appendChild(panel);
    }
    panel.innerHTML = "";

    var head = document.createElement("div");
    head.className = "gb-side-manage-head";
    var title = document.createElement("span");
    title.className = "gb-side-manage-title";
    title.textContent = tr("sidebar-apps-manage", "Manage apps");
    var close = document.createElement("button");
    close.type = "button";
    close.className = "gb-side-manage-close";
    close.title = tr("sidebar-apps-manage-close", "Close");
    close.setAttribute("aria-label", close.title);
    close.textContent = "×";
    close.addEventListener("click", function () {
      mod.close(host);
    });
    head.appendChild(title);
    head.appendChild(close);
    panel.appendChild(head);

    var all = document.createElement("button");
    all.type = "button";
    all.className = "gb-side-manage-all";
    all.textContent = tr("sidebar-apps-manage-all", "Turn all on");
    all.addEventListener("click", function () {
      if (p) p.enableAll();
      sync(panel);
    });
    panel.appendChild(all);

    var list = document.createElement("div");
    list.className = "gb-side-manage-list";
    railApps().forEach(function (app) {
      var row = document.createElement("label");
      row.className = ROW_CLASS;
      row.setAttribute("data-app-id", app.id);
      var box = document.createElement("input");
      box.type = "checkbox";
      box.checked = !p || p.isEnabled(app.id);
      box.setAttribute("aria-label", appName(app));
      box.addEventListener("change", function () {
        if (p) p.setEnabled(app.id, box.checked);
        sync(panel);
      });
      var name = document.createElement("span");
      name.className = "gb-side-manage-name";
      name.textContent = appName(app);
      row.appendChild(box);
      row.appendChild(name);
      list.appendChild(row);
    });
    if (!list.children.length) {
      var empty = document.createElement("div");
      empty.className = "gb-side-manage-empty";
      empty.textContent = tr("sidebar-apps-manage-empty", "No apps available");
      list.appendChild(empty);
    }
    panel.appendChild(list);

    var summary = document.createElement("div");
    summary.className = "gb-side-manage-summary";
    panel.appendChild(summary);

    sync(panel);
    applyExpanded(host, true);
    mod.open = true;
    return panel;
  };

  mod.isOpen = function (host) {
    return !!panelOf(host || hostOf());
  };

  mod.close = function (host) {
    host = host || hostOf();
    var panel = panelOf(host);
    if (panel) panel.remove();
    applyExpanded(host, false);
    mod.open = false;
  };

  mod.toggle = function (host) {
    host = host || hostOf();
    if (!host) return;
    if (panelOf(host)) mod.close(host);
    else mod.render(host);
  };

  // Turning an app on/off elsewhere (a rail tile's hide action) must keep an
  // open panel truthful, and vice versa.
  window.addEventListener("gb-launcher-apps-changed", function () {
    sync(panelOf(hostOf()));
  });

  // Escape closes the panel; it is a transient popover, not a dialog.
  document.addEventListener("keydown", function (e) {
    if (e.key !== "Escape") return;
    var host = hostOf();
    if (panelOf(host)) mod.close(host);
  });

  // The rail is re-rendered on locale change; re-open the panel so the user
  // does not lose the list mid-edit.
  window.addEventListener("localeChanged", function () {
    if (mod.open && hostOf()) mod.render(hostOf());
  });
})(window.GBLauncherAppsManage);
