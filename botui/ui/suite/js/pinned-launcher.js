"use strict";

// GB Pinned Launcher (#1160): taskbar chips for pinned apps, widgets and web
// pages, plus context menus (Pin / Unpin / Open windowed / Open isolated)
// on start-menu tiles, dock items and launcher chips.
//
// Persistence: localStorage["gb-pinned-launcher"] as
//   [{ kind: "app"|"widget"|"web", appId?, id?, title?, url? }, …]

(function () {
  if (window.GBPinnedLauncher) return;

  var STORAGE_KEY = "gb-pinned-launcher";
  var strip = null;
  var items = readStore();

  function readStore() {
    try {
      return JSON.parse(localStorage.getItem(STORAGE_KEY) || "[]");
    } catch (e) {
      return [];
    }
  }

  function save() {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(items));
    } catch (e) {}
  }

  function announce(text) {
    if (window.GBAppLifecycle) {
      window.GBAppLifecycle.announce(text);
    }
  }

  function findApp(appId) {
    var reg = window.APPS_REGISTRY || [];
    for (var i = 0; i < reg.length; i++) {
      if (reg[i].id === appId) return reg[i];
    }
    return null;
  }

  function iconFor(item) {
    var svg = "";
    if (item.kind === "app") {
      var app = findApp(item.appId);
      if (app && app.icon) svg = app.icon;
    } else if (item.kind !== "web") {
      svg =
        '<rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18"/>';
    } else {
      svg =
        '<circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>';
    }
    return (
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">' +
      svg +
      "</svg>"
    );
  }

  function titleFor(item) {
    if (item.title) return item.title;
    if (item.kind === "app") {
      var app = findApp(item.appId);
      if (app) return app.title;
    }
    return item.appId || item.id || "Pinned";
  }

  // ── Render ───────────────────────────────────────────────────

  function ensureStrip() {
    if (strip) return strip;
    var center = document.getElementById("taskbar-center");
    if (!center) return null;
    strip = document.createElement("div");
    strip.id = "taskbarPinnedStrip";
    strip.className = "taskbar-pinned-strip";
    center.insertBefore(strip, center.firstChild);
    return strip;
  }

  function render() {
    var host = ensureStrip();
    if (!host) return;
    host.innerHTML = "";
    items.forEach(function (item) {
      var chip = document.createElement("div");
      chip.className = "taskbar-pinned-item";
      chip.setAttribute("data-pin-kind", item.kind);
      chip.setAttribute(
        "data-pin-key",
        item.kind === "app" ? item.appId : item.id
      );
      chip.title = titleFor(item);
      chip.innerHTML = '<span class="taskbar-pinned-icon">' + iconFor(item) + "</span>";
      chip.addEventListener("click", function () {
        launch(item);
      });
      chip.addEventListener("contextmenu", function (e) {
        e.preventDefault();
        e.stopPropagation();
        openChipMenu(item, e.clientX, e.clientY);
      });
      host.appendChild(chip);
    });
  }

  function launch(item) {
    if (item.kind === "app") {
      if (window.openDeepLink) {
        window.openDeepLink(item.appId, {});
      }
      return;
    }
    if (item.kind === "web") {
      if (item.url) window.open(item.url, "_blank", "noopener");
      return;
    }
    if (item.kind === "widget") {
      if (window.GBWidgets) {
        window.GBWidgets.ensureVisible(item.id);
      }
    }
  }

  // ── Menu helpers ─────────────────────────────────────────────

  function closeMenus() {
    var menus = document.querySelectorAll(".gb-launcher-menu");
    for (var i = 0; i < menus.length; i++) menus[i].remove();
  }

  function buildMenu(entries, x, y) {
    closeMenus();
    var menu = document.createElement("div");
    menu.className = "desktop-context-menu gb-launcher-menu";
    menu.style.left = Math.min(x, window.innerWidth - 210) + "px";
    menu.style.top = Math.min(y, window.innerHeight - 160) + "px";
    entries.forEach(function (entry) {
      var item = document.createElement("div");
      item.className = "desktop-context-item";
      item.textContent = entry.label;
      item.addEventListener("click", function () {
        closeMenus();
        entry.action();
      });
      menu.appendChild(item);
    });
    document.body.appendChild(menu);
    setTimeout(function () {
      document.addEventListener("click", closeMenus, { once: true });
    }, 0);
  }

  function openChipMenu(e, x, y) {
    var chip = e.target.closest(".taskbar-pinned-item");
    if (!chip) return;
    var item = items.find(function (i) {
      return (i.kind === "app" ? i.appId : i.id) === chip.dataset.pinKey;
    });
    if (!item) return;
    var entries = [
      {
        label: item.kind === "web" ? "Open" : "Open windowed",
        action: function () {
          launch(item);
        },
      },
    ];
    if (item.kind === "app" || item.kind === "widget") {
      entries.push({
        label: "Open isolated",
        action: function () {
          if (item.kind === "app" && window.WindowManager) {
            window.WindowManager.openIsolated(item.appId, {});
          } else if (window.GBWidgets) {
            window.GBWidgets.openIsolated(item.id);
          }
        },
      });
    }
    entries.push({
      label: "Unpin",
      action: function () {
        remove(item);
      },
    });
    buildMenu(entries, x, y);
  }

  // ── Persistence + pin API ────────────────────────────────────

  function findByItem(item) {
    return items.find(function (i) {
      if (item.kind === "app") return i.kind === "app" && i.appId === item.appId;
      if (item.kind === "web") return i.kind === "web" && i.url === item.url;
      return i.kind === "widget" && i.id === item.id;
    });
  }

  function add(item) {
    if (findByItem(item)) return false;
    items.push(item);
    save();
    render();
    announce("Pinned to launcher: " + titleFor(item));
    return true;
  }

  function remove(item) {
    items = items.filter(function (existing) {
      if (item.kind === "app") {
        return !(existing.kind === "app" && existing.appId === item.appId);
      }
      if (item.kind === "web") {
        return !(existing.kind === "web" && existing.url === item.url);
      }
      return !(existing.kind === "widget" && existing.id === item.id);
    });
    save();
    render();
    announce("Unpinned: " + titleFor(item));
  }

  function pinApp(appId) {
    var app = findApp(appId);
    if (!app) return false;
    return add({ kind: "app", appId: appId, title: app.title });
  }

  function unpinApp(appId) {
    remove({ kind: "app", appId: appId });
  }

  function pinWidget(id) {
    if (!window.GBWidgets) return false;
    var inst = window.GBWidgets.byId(id);
    if (!inst) return false;
    return add({ kind: "widget", id: id, title: inst.title });
  }

  function pinWeb(url, title) {
    return add({ kind: "web", url: url, title: title });
  }

  function isPinned(apiItem) {
    return !!findByItem(apiItem);
  }

  // ── Delegated context menus (start menu + dock) ──────────────

  function bindGlobalMenus() {
    document.addEventListener("contextmenu", function (e) {
      var tile = e.target.closest(".start-menu-app");
      if (tile) {
        e.preventDefault();
        var appId = tile.getAttribute("data-app-id");
        openStartTileMenu(appId, e.clientX, e.clientY);
        return;
      }
      var dock = e.target.closest(".taskbar-dock-item");
      if (dock) {
        e.preventDefault();
        openDockMenu(dock.id.replace("dock-item-", ""), e.clientX, e.clientY);
        return;
      }
    });
  }

  function openStartTileMenu(appId, x, y) {
    var app = findApp(appId);
    var entries = [];
    if (!isPinned({ kind: "app", appId: appId })) {
      entries.push({
        label: "Pin to launcher",
        action: function () {
          pinApp(appId);
        },
      });
    } else {
      entries.push({
        label: "Unpin from launcher",
        action: function () {
          unpinApp(appId);
        },
      });
    }
    entries.push({
      label: "Open isolated",
      action: function () {
        if (window.WindowManager) window.WindowManager.openIsolated(appId, {});
      },
    });
    if (app && window.GBWidgets) {
      entries.push({
        label: "Add as widget",
        action: function () {
          window.GBWidgets.addApp(appId);
        },
      });
    }
    buildMenu(entries, x, y);
  }

  function openDockMenu(appId, x, y) {
    var entries = [];
    if (!isPinned({ kind: "app", appId: appId })) {
      entries.push({
        label: "Pin to launcher",
        action: function () {
          pinApp(appId);
        },
      });
    } else {
      entries.push({
        label: "Unpin from launcher",
        action: function () {
          unpinApp(appId);
        },
      });
    }
    entries.push({
      label: "Close window",
      action: function () {
        if (window.WindowManager) window.WindowManager.close(appId);
      },
    });
    buildMenu(entries, x, y);
  }

  // ── Integration with widget registry ─────────────────────────

  function listenForPins() {
    window.addEventListener("gb-launcher-pin-request", function (e) {
      var detail = e.detail || {};
      if (detail.kind === "app") {
        pinApp(detail.appId);
      } else if (detail.kind === "widget") {
        pinWidget(detail.id);
      } else if (detail.kind === "web") {
        pinWeb(detail.url, detail.title);
      }
    });
  }

  var API = {
    init: function () {
      bindGlobalMenus();
      listenForPins();
      // Wait for the taskbar to exist (WindowManager creates it lazily).
      var attempt = 0;
      var timer = setInterval(function () {
        attempt++;
        if (document.getElementById("taskbar-center")) {
          clearInterval(timer);
          render();
        } else if (attempt > 30) {
          clearInterval(timer);
        }
      }, 200);
    },
    render: render,
    add: add,
    remove: remove,
    pinApp: pinApp,
    unpinApp: unpinApp,
    pinWidget: pinWidget,
    pinWeb: pinWeb,
    isPinned: isPinned,
    list: function () {
      return items.slice();
    },
  };

  window.GBPinnedLauncher = API;
})();