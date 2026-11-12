"use strict";

// GB Sidebar Apps (#1188): unified launcher inside the left bar — every
// APPS_REGISTRY app in one grid, with an ON/OFF switch and a filter box.
// Loaded BEFORE sidebar.js, which delegates renderApps() here.

window.GBSidebarApps = window.GBSidebarApps || {};

(function (mod) {
  var SWITCH_KEY = "gb-sidebar-apps-on";

  function tr(key, fallback, params) {
    return window.gbT ? window.gbT(key, fallback, params) : fallback;
  }

  // The collapsed rail (icon-only state of the same bar) lists a fixed handful
  // of shortcuts. Honour the same per-app preference there so a hidden app is
  // not resurrected just because the sidebar is collapsed.
  mod.applyCollapsed = function () {
    var p = window.GBLauncherAppsPrefs;
    document
      .querySelectorAll(".chat-sidebar-collapsed-icons [data-app]")
      .forEach(function (el) {
        el.hidden = !!(p && !p.isEnabled(el.getAttribute("data-app")));
      });
  };

  function appsOn() {
    try {
      // Hidden by default — the sidebar is for pinned essentials and
      // conversations. The user opts into the full app grid via the toggle.
      return localStorage.getItem(SWITCH_KEY) === "1";
    } catch (e) {
      return true;
    }
  }

  function appIcon(app) {
    return (
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">' +
      (app.icon || "") +
      "</svg>"
    );
  }

  // Every app the rail is ALLOWED to offer, before the user's own ON/OFF
  // choices are applied. `js/launcher-apps-manage.js` lists exactly this set,
  // so an app hidden by the user is always reachable to be turned back on.
  // APPS_REGISTRY itself is never modified — palette and deep links still work.
  mod.railApps = function () {
    var previewOn = !!(window.GBPreviewMode && window.GBPreviewMode.isOn());
    return (window.APPS_REGISTRY || [])
      // #1386c — the rail lists apps the user can open by hand. Every other
      // native surface is a Vibe workbench pane (opened from the Vibe
      // toolbar), so it never shows up as a standalone app here.
      // #2097 — `launcher_visibility: "always"` pins an app regardless;
      // preview apps stay out of the rail unless Preview mode is on.
      .filter(function (a) {
        if (a.launcher_visibility === "always") return true;
        if ((a.preview || a.launcher_visibility === "preview") && !previewOn) return false;
        // Preview apps are openable (not auto-pinned) when preview mode is on.
        return true;
      })
      .slice()
      .sort(function (a, b) {
        return String(a.title || a.id).localeCompare(String(b.title || b.id));
      });
  };

  // The rail's visible subset: the per-app ON/OFF preference is applied here,
  // at render time, so a disabled app never flashes while the bar is drawn.
  function visibleApps() {
    var p = window.GBLauncherAppsPrefs;
    return mod.railApps().filter(function (a) {
      return !p || p.isEnabled(a.id);
    });
  }

  mod.render = function () {
    var nav = document.getElementById("sidebarAppsNav");
    if (!nav || nav.dataset.gbAppsBound === "1") return;

    nav.innerHTML = "";
    var wrap = document.createElement("div");
    wrap.className = "gb-side-apps";

    // Header: title + count + ON/OFF switch.
    var head = document.createElement("div");
    head.className = "gb-side-apps-head";
    var title = document.createElement("span");
    title.className = "gb-side-apps-title";
    var count = document.createElement("span");
    count.className = "gb-side-apps-count";
    var swLabel = document.createElement("label");
    swLabel.className = "gb-side-switch";
    swLabel.title = tr("sidebar-apps-toggle", "Show/hide apps");
    var sw = document.createElement("input");
    sw.type = "checkbox";
    sw.checked = appsOn();
    sw.setAttribute("aria-label", tr("sidebar-apps-toggle-aria", "Toggle apps panel"));
    var slider = document.createElement("span");
    slider.className = "gb-side-switch-slider";
    swLabel.appendChild(sw);
    swLabel.appendChild(slider);
    head.appendChild(title);
    head.appendChild(count);
    head.appendChild(swLabel);

    // Body: filter + grid.
    var body = document.createElement("div");
    body.className = "gb-side-apps-body";
    if (!sw.checked) body.hidden = true;

    var search = document.createElement("input");
    search.type = "search";
    search.className = "gb-side-apps-search";
    search.placeholder = tr("sidebar-apps-filter", "Filter apps…");
    search.setAttribute("aria-label", tr("sidebar-apps-filter-aria", "Filter apps"));

    var grid = document.createElement("div");
    grid.className = "gb-side-apps-grid";

    // Preview mode (#1348): reveals the applications the registry marks
    // preview-only (#2097). They are absent from APPS_REGISTRY until this
    // switch is on, which is what keeps unreleased apps out of the way.
    var previewRow = document.createElement("div");
    previewRow.className = "gb-side-preview";
    var previewLabel = document.createElement("span");
    previewLabel.className = "gb-side-preview-label";
    previewLabel.textContent = tr("sidebar-preview", "Preview mode");
    var previewSwitchLabel = document.createElement("label");
    previewSwitchLabel.className = "gb-side-switch";
    previewSwitchLabel.title = tr("sidebar-preview-hint", "Show the applications that are still in preview");
    var previewSwitch = document.createElement("input");
    previewSwitch.type = "checkbox";
    previewSwitch.checked = !!(window.GBPreviewMode && window.GBPreviewMode.isOn());
    previewSwitch.setAttribute("aria-label", tr("sidebar-preview-toggle", "Toggle preview mode"));
    var previewSlider = document.createElement("span");
    previewSlider.className = "gb-side-switch-slider";
    previewSwitchLabel.appendChild(previewSwitch);
    previewSwitchLabel.appendChild(previewSlider);
    previewRow.appendChild(previewLabel);
    previewRow.appendChild(previewSwitchLabel);

    body.appendChild(search);
    body.appendChild(grid);

    // Bottom of the rail: the only way back for an app the user turned off.
    // It stays outside `body` so it is reachable even when the app grid is
    // collapsed by the panel switch above.
    var manageBar = document.createElement("div");
    manageBar.className = "gb-side-apps-manage";
    var manageBtn = document.createElement("button");
    manageBtn.type = "button";
    manageBtn.className = "gb-side-manage-btn";
    manageBtn.title = tr("sidebar-apps-manage", "Manage apps");
    manageBtn.setAttribute("aria-label", manageBtn.title);
    manageBtn.setAttribute("aria-expanded", "false");
    manageBtn.innerHTML =
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">' +
      '<circle cx="12" cy="12" r="3"/>' +
      '<path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h.09a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.09a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>' +
      "</svg>";
    var manageLabel = document.createElement("span");
    manageLabel.className = "gb-side-manage-btn-label";
    manageLabel.textContent = tr("sidebar-apps-manage", "Manage apps");
    manageBtn.appendChild(manageLabel);
    manageBtn.addEventListener("click", function () {
      if (window.GBLauncherAppsManage) window.GBLauncherAppsManage.toggle(nav);
    });
    manageBar.appendChild(manageBtn);

    wrap.appendChild(head);
    wrap.appendChild(previewRow);
    wrap.appendChild(body);
    wrap.appendChild(manageBar);
    nav.appendChild(wrap);

    function fill() {
      // Per-app ON/OFF preference is applied by visibleApps() before anything
      // is drawn, so a hidden app is never painted even for a frame.
      var reg = visibleApps();
      var allowed = mod.railApps().length;
      title.textContent = tr("sidebar-apps", "Apps");
      grid.innerHTML = "";
      var previewCount = 0;
      reg.forEach(function (app) {
        if (app.preview) previewCount += 1;
        var tile = document.createElement("div");
        tile.className = app.preview
          ? "gb-side-app-tile gb-side-app-tile-preview"
          : "gb-side-app-tile";
        tile.setAttribute("data-app-id", app.id);
        tile.setAttribute("title", app.preview ? tr("sidebar-app-preview-title", "{app} (preview)", { app: app.title }) : app.title);
        if (app.preview) tile.setAttribute("data-preview", "1");
        tile.innerHTML =
          '<span class="gb-side-app-icon" style="color:' +
          (app.color || "#88ccff") +
          '">' +
          appIcon(app) +
          "</span>" +
          '<span class="gb-side-app-label"></span>' +
          '<button type="button" class="gb-side-app-hide">×</button>';
        tile.querySelector(".gb-side-app-label").textContent = app.title;

        // Per-app OFF: the tile disappears from the rail only. The app stays in
        // APPS_REGISTRY, so the palette and deep links still open it.
        var hide = tile.querySelector(".gb-side-app-hide");
        var hideTitle = tr("sidebar-app-hide", "Hide {app} from this bar", { app: app.title });
        hide.title = hideTitle;
        hide.setAttribute("aria-label", hideTitle);
        hide.addEventListener("click", function (ev) {
          ev.stopPropagation();
          if (window.GBLauncherAppsPrefs) window.GBLauncherAppsPrefs.setEnabled(app.id, false);
        });

        tile.addEventListener("click", function () {
          if (window.openDeepLink) window.openDeepLink(app.id, {});
        });
        // Right-click is the second, keyboard-free route to the same action.
        tile.addEventListener("contextmenu", function (ev) {
          ev.preventDefault();
          if (window.GBLauncherAppsPrefs) window.GBLauncherAppsPrefs.setEnabled(app.id, false);
        });
        grid.appendChild(tile);
      });

      if (!reg.length) {
        var empty = document.createElement("div");
        empty.className = "gb-side-apps-empty";
        empty.textContent = allowed
          ? tr("sidebar-apps-empty", "All apps are turned off — use Manage apps below.")
          : tr("sidebar-apps-none", "No apps available");
        grid.appendChild(empty);
      }

      var hidden = allowed - reg.length;
      // The count names the preview share so it is obvious why the grid grew
      // when the switch was turned on; the off count explains a short rail.
      count.textContent = previewCount
        ? tr("sidebar-apps-count-preview", "{count} (+ {preview} preview)", {
            count: reg.length,
            preview: previewCount,
          })
        : tr("sidebar-apps-count", "{count} apps", { count: reg.length });
      if (hidden > 0) {
        count.textContent +=
          " · " + tr("sidebar-apps-count-off", "{count} off", { count: hidden });
      }
      filter();
    }

    function filter() {
      var q = (search.value || "").toLowerCase().trim();
      grid.querySelectorAll(".gb-side-app-tile").forEach(function (tile) {
        var label = tile.querySelector(".gb-side-app-label");
        var hit =
          !q ||
          (label && label.textContent.toLowerCase().includes(q)) ||
          (tile.getAttribute("data-app-id") || "").includes(q);
        tile.style.display = hit ? "" : "none";
      });
      // The "everything is off" hint only describes the unfiltered rail.
      var hint = grid.querySelector(".gb-side-apps-empty");
      if (hint) hint.style.display = q ? "none" : "";
    }

    sw.addEventListener("change", function () {
      body.hidden = !sw.checked;
      try {
        localStorage.setItem(SWITCH_KEY, sw.checked ? "1" : "0");
      } catch (e) {}
    });
    previewSwitch.addEventListener("change", function () {
      // Persisting calls refreshAppsCatalog(), which re-dispatches
      // `gb-apps-catalog-loaded`; fill() runs again from that event, so the
      // grid is not rebuilt twice here.
      if (window.GBPreviewMode) window.GBPreviewMode.set(previewSwitch.checked);
      else fill();
    });
    // Keep the left-bar switch in sync when preview mode changes elsewhere
    // (e.g. the Settings toggle). GBPreviewMode is the canonical source.
    window.addEventListener("gb-preview-changed", function (e) {
      var on = !!(e && e.detail && e.detail.preview);
      if (window.GBPreviewMode) on = window.GBPreviewMode.isOn();
      previewSwitch.checked = on;
    });
    search.addEventListener("input", filter);

    nav.dataset.gbAppsBound = "1";
    fill();
    mod.fillRail = fill;
    mod.applyCollapsed();

    // The backend catalog merges into APPS_REGISTRY asynchronously —
    // refresh the grid when it lands so nothing is missing.
    window.addEventListener("gb-apps-catalog-loaded", fill);
  };

  // A tile's hide action (or the Manage apps panel) changed the per-app
  // preference: redraw the rail. Bound once, so locale re-renders do not stack
  // duplicate handlers.
  window.addEventListener("gb-launcher-apps-changed", function () {
    if (typeof mod.fillRail === "function") mod.fillRail();
    mod.applyCollapsed();
  });

  // Language switch: labels here are generated imperatively, so clear the
  // bind guard and render again to pick up the new catalogue.
  window.addEventListener("localeChanged", function () {
    var nav = document.getElementById("sidebarAppsNav");
    if (nav) nav.dataset.gbAppsBound = "";
    mod.render();
  });
})(window.GBSidebarApps);
