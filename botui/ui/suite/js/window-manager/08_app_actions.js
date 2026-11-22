"use strict";
/* #2341 — universal per-app sanity affordances.
 *
 * Every app window carries the same "App actions" button in its title bar
 * (built by 03_window_chrome.js). This module fills that button with:
 *   Refresh   — asks the app to reload its data (gb:app-refresh event)
 *   Copy ID   — copies the app identifier to the clipboard
 *   Copy link — copies a deep link that opens/apply focuses the app
 *   Diagnose  — a non-secret status panel with a "Copy diagnostics" action
 *
 * The surface is keyboard reachable, labelled and i18n-backed. It exposes no
 * secrets: only the app id, title, GBStatus payload, online flag and version.
 */
(function (window, document) {
  function t(key, fallback, params) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
  }
  function esc(value) {
    return String(value == null ? "" : value).replace(/[&<>"']/g, function (ch) {
      return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[ch];
    });
  }
  function toastText(message, kind) {
    if (window.GBToasts && typeof window.GBToasts.show === "function") {
      window.GBToasts.show(message, "", kind || "info");
    }
  }
  function copy(value) {
    if (navigator.clipboard && navigator.clipboard.writeText) {
      return navigator.clipboard.writeText(value);
    }
    return Promise.resolve();
  }

  var menu = null;
  var panel = null;
  var currentId = null;
  var collabLoading = null;

  // #2336 — load the shared GBCollab scripts on first use so every app can
  // expose the same comments/activity panels without per-app forks. Apps that
  // already ship them are untouched (the globals are identical singletons).
  function ensureCollab() {
    if (window.GBCollabComments && window.GBCollabActivity) return Promise.resolve();
    if (collabLoading) return collabLoading;
    var files = [
      "/suite/js/collab-a11y.js",
      "/suite/js/collab-comments/01_helpers.js",
      "/suite/js/collab-comments/02_data.js",
      "/suite/js/collab-comments/99_panel.js",
      "/suite/js/collab-activity.js"
    ];
    collabLoading = new Promise(function (resolve, reject) {
      var i = 0;
      function next() {
        if (i >= files.length) { resolve(); return; }
        var s = document.createElement("script");
        s.src = files[i++];
        s.onload = next;
        s.onerror = next;
        document.head.appendChild(s);
      }
      next();
    });
    return collabLoading;
  }

  function openCollab(kind, id) {
    ensureCollab().then(function () {
      var opts = { resourceType: "app", resourceId: id, title: appTitle(id) };
      if (kind === "comments" && window.GBCollabComments) window.GBCollabComments.open(opts);
      else if (kind === "activity" && window.GBCollabActivity) window.GBCollabActivity.open(opts);
      else toastText(t("app-actions-unavailable", "Not available"), "error");
    });
  }

  function closeAll() {
    if (menu) { menu.hidden = true; menu = null; }
    if (panel) { panel.hidden = true; }
    document.querySelectorAll(".window-sanity-btn[aria-expanded=\"true\"]")
      .forEach(function (b) { b.setAttribute("aria-expanded", "false"); });
    currentId = null;
  }

  function ensureMenu() {
    if (menu && menu.parentNode) return menu;
    menu = document.createElement("div");
    menu.id = "gb-app-actions-menu";
    menu.className = "gb-app-actions-menu";
    menu.setAttribute("role", "menu");
    menu.hidden = true;
    document.body.appendChild(menu);
    return menu;
  }

  function menuItem(labelKey, fallback, onClick) {
    var b = document.createElement("button");
    b.type = "button";
    b.className = "gb-app-actions-item";
    b.setAttribute("role", "menuitem");
    b.textContent = t(labelKey, fallback);
    b.addEventListener("click", function (e) { e.stopPropagation(); closeAll(); onClick(); });
    return b;
  }

  function appTitle(id) {
    var rec = window.WindowManager && window.WindowManager.openWindows
      ? window.WindowManager.openWindows.filter(function (w) { return w.id === id; })[0] : null;
    return (rec && rec.title) || id;
  }

  function refresh(id) {
    window.dispatchEvent(new CustomEvent("gb:app-refresh", { detail: { appId: id } }));
    var body = document.getElementById("window-body-" + id);
    var frame = body && body.querySelector("iframe");
    if (frame && frame.contentWindow) {
      try { frame.contentWindow.location.reload(); } catch (e) { /* cross origin — event only */ }
    }
    toastText(t("app-actions-refreshed", "Refreshed"), "success");
  }

  function diagnostics(id) {
    var payload = (window.GBStatus && typeof window.GBStatus.get === "function")
      ? window.GBStatus.get(id) : null;
    var lines = [
      t("app-diagnostics-app", "App") + ": " + appTitle(id) + " (" + id + ")",
      t("app-diagnostics-status", "Status") + ": " + ((payload && (payload.text || payload.state)) || t("status-ready", "Ready")),
      t("app-diagnostics-online", "Network") + ": " + (navigator.onLine ? "online" : "offline"),
      t("app-diagnostics-updated", "Check") + ": " + new Date().toISOString()
    ];
    return {
      text: lines.join("\n"),
      title: t("os-diagnostics-title", "Diagnostics") + " — " + appTitle(id)
    };
  }

  function ensurePanel() {
    if (panel && panel.parentNode) return panel;
    panel = document.createElement("div");
    panel.id = "gb-app-diagnostics";
    panel.className = "gb-app-diagnostics";
    panel.setAttribute("role", "dialog");
    panel.setAttribute("aria-modal", "false");
    panel.hidden = true;
    document.body.appendChild(panel);
    return panel;
  }

  function openDiagnostics(id) {
    var box = ensurePanel();
    var data = diagnostics(id);
    box.textContent = "";
    var title = document.createElement("div");
    title.className = "gb-app-diagnostics-title";
    title.textContent = data.title;
    box.appendChild(title);
    var pre = document.createElement("pre");
    pre.className = "gb-app-diagnostics-body";
    pre.setAttribute("role", "status");
    pre.setAttribute("aria-live", "polite");
    pre.textContent = data.text;
    box.appendChild(pre);
    var actions = document.createElement("div");
    actions.className = "gb-app-diagnostics-actions";
    var copyBtn = document.createElement("button");
    copyBtn.type = "button";
    copyBtn.textContent = t("integrations-copy-diagnostics", "Copy diagnostics");
    copyBtn.addEventListener("click", function () {
      copy(data.text).then(function () { toastText(t("app-actions-copied", "Copied"), "success"); });
    });
    var closeBtn = document.createElement("button");
    closeBtn.type = "button";
    closeBtn.textContent = t("action-close", "Close");
    closeBtn.addEventListener("click", closeAll);
    actions.appendChild(copyBtn);
    actions.appendChild(closeBtn);
    box.appendChild(actions);
    box.hidden = false;
    box.dataset.appId = id;
  }

  function open(id, anchor) {
    if (!id) return;
    var wasOpen = menu && !menu.hidden && currentId === id;
    closeAll();
    if (wasOpen) return;
    currentId = id;
    var m = ensureMenu();
    m.textContent = "";
    m.appendChild(menuItem("app-action-refresh", "Refresh", function () { refresh(id); }));
    m.appendChild(menuItem("app-copy-id", "Copy ID", function () {
      copy(id).then(function () { toastText(t("app-actions-copied", "Copied"), "success"); });
    }));
    m.appendChild(menuItem("app-copy-link", "Copy link", function () {
      var url = location.origin + "/?app=" + encodeURIComponent(id);
      copy(url).then(function () { toastText(t("app-actions-copied", "Copied"), "success"); });
    }));
    m.appendChild(menuItem("app-diagnose", "Diagnose", function () { openDiagnostics(id); }));
    m.appendChild(menuItem("app-comments", "Comments", function () { openCollab("comments", id); }));
    m.appendChild(menuItem("app-activity", "Activity", function () { openCollab("activity", id); }));
    m.hidden = false;
    if (anchor) {
      var r = anchor.getBoundingClientRect();
      m.style.top = Math.min(r.bottom + 4, window.innerHeight - 200) + "px";
      m.style.left = Math.max(8, Math.min(r.left, window.innerWidth - 220)) + "px";
      anchor.setAttribute("aria-expanded", "true");
    }
  }

  document.addEventListener("click", function (e) {
    if (!menu || menu.hidden) return;
    if (e.target.closest && (e.target.closest("#gb-app-actions-menu") ||
      e.target.closest(".window-sanity-btn"))) return;
    closeAll();
  });

  document.addEventListener("keydown", function (e) {
    if (e.key === "Escape") closeAll();
  });

  window.GBAppActions = { open: open, close: closeAll, diagnostics: diagnostics };
})(window, document);
