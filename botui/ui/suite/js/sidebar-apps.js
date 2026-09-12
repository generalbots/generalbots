"use strict";

// GB Sidebar Apps (#1188): unified launcher inside the left bar — every
// APPS_REGISTRY app in one grid, with an ON/OFF switch and a filter box.
// Loaded BEFORE sidebar.js, which delegates renderApps() here.

window.GBSidebarApps = window.GBSidebarApps || {};

(function (mod) {
  var SWITCH_KEY = "gb-sidebar-apps-on";

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
    swLabel.title = "Show/hide apps";
    var sw = document.createElement("input");
    sw.type = "checkbox";
    sw.checked = appsOn();
    sw.setAttribute("aria-label", "Toggle apps panel");
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
    search.placeholder = "Filter apps…";
    search.setAttribute("aria-label", "Filter apps");

    var grid = document.createElement("div");
    grid.className = "gb-side-apps-grid";

    // Preview mode (#1348): reveals the applications the product file lists
    // under `preview_apps`. They are absent from the launcher until this
    // switch is on, which is what keeps unreleased apps out of the way.
    var previewRow = document.createElement("div");
    previewRow.className = "gb-side-preview";
    var previewLabel = document.createElement("span");
    previewLabel.className = "gb-side-preview-label";
    previewLabel.textContent = "Preview mode";
    var previewSwitchLabel = document.createElement("label");
    previewSwitchLabel.className = "gb-side-switch";
    previewSwitchLabel.title = "Show the applications that are still in preview";
    var previewSwitch = document.createElement("input");
    previewSwitch.type = "checkbox";
    previewSwitch.checked = !!(window.GBPreviewMode && window.GBPreviewMode.isOn());
    previewSwitch.setAttribute("aria-label", "Toggle preview mode");
    var previewSlider = document.createElement("span");
    previewSlider.className = "gb-side-switch-slider";
    previewSwitchLabel.appendChild(previewSwitch);
    previewSwitchLabel.appendChild(previewSlider);
    previewRow.appendChild(previewLabel);
    previewRow.appendChild(previewSwitchLabel);

    body.appendChild(search);
    body.appendChild(grid);
    wrap.appendChild(head);
    wrap.appendChild(previewRow);
    wrap.appendChild(body);
    nav.appendChild(wrap);

    function fill() {
      var reg = (window.APPS_REGISTRY || []).slice().sort(function (a, b) {
        return String(a.title || a.id).localeCompare(String(b.title || b.id));
      });
      title.textContent = "Apps";
      grid.innerHTML = "";
      var previewCount = 0;
      reg.forEach(function (app) {
        if (app.preview) previewCount += 1;
        var tile = document.createElement("div");
        tile.className = app.preview
          ? "gb-side-app-tile gb-side-app-tile-preview"
          : "gb-side-app-tile";
        tile.setAttribute("data-app-id", app.id);
        tile.setAttribute("title", app.preview ? app.title + " (preview)" : app.title);
        if (app.preview) tile.setAttribute("data-preview", "1");
        tile.innerHTML =
          '<span class="gb-side-app-icon" style="color:' +
          (app.color || "#88ccff") +
          '">' +
          appIcon(app) +
          "</span>" +
          '<span class="gb-side-app-label"></span>';
        tile.querySelector(".gb-side-app-label").textContent = app.title;
        tile.addEventListener("click", function () {
          if (window.openDeepLink) window.openDeepLink(app.id, {});
        });
        grid.appendChild(tile);
      });
      // The count names the preview share so it is obvious why the grid grew
      // when the switch was turned on.
      count.textContent = previewCount
        ? reg.length + " (+ " + previewCount + " preview)"
        : String(reg.length);
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
    search.addEventListener("input", filter);

    nav.dataset.gbAppsBound = "1";
    fill();

    // The backend catalog merges into APPS_REGISTRY asynchronously —
    // refresh the grid when it lands so nothing is missing.
    window.addEventListener("gb-apps-catalog-loaded", fill);
  };
})(window.GBSidebarApps);
