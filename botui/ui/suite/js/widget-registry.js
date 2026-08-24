"use strict";

// GB Widget Registry (#1160, #1150): desktop widget pane, instance storage
// and launch modes. DOM construction lives in widget-renderer.js; launcher
// pins live in pinned-launcher.js. Persistence: localStorage["gb-widgets"].

(function () {
  if (window.GBWidgets) return;

  var STORAGE_KEY = "gb-widgets";

  var SYSTEM_WIDGETS = [
    { id: "clock", title: "Clock", partial: "/suite/widgets/clock.html", w: 260, h: 120 },
    { id: "calendar", title: "Calendar", partial: "/suite/widgets/calendar.html", w: 340, h: 320 },
    { id: "resources", title: "System Resources", partial: "/suite/widgets/resources.html", w: 260, h: 130 },
    { id: "calculator", title: "Calculator", partial: "/suite/widgets/calculator.html", w: 250, h: 330 },
    { id: "notes", title: "Sticky Note", partial: "/suite/widgets/notes.html", w: 260, h: 220 },
    { id: "todo", title: "Quick Tasks", partial: "/suite/widgets/todo.html", w: 280, h: 300 },
    { id: "battery", title: "Battery", partial: "/suite/widgets/battery.html", w: 250, h: 140 },
    { id: "timer", title: "Pomodoro Timer", partial: "/suite/widgets/timer.html", w: 250, h: 220 },
    { id: "weather", title: "Weather", partial: "/suite/widgets/weather.html", w: 300, h: 220 },
    { id: "photos", title: "Photos", partial: "/suite/widgets/photos.html", w: 300, h: 240 },
  ];

  var pane = null;
  var instances = [];
  var zCounter = 20;
  var SPAWN_STEP = 36;

  // Cascade new widgets across the workspace so overlapping stacks never
  // hide earlier ones.
  function nextPosition() {
    var host = pane || document.getElementById("desktop-content");
    var maxX = host ? Math.max(0, host.clientWidth - 300) : 900;
    var maxY = host ? Math.max(0, host.clientHeight - 220) : 600;
    var n = instances.length;
    return {
      x: 24 + ((n * SPAWN_STEP) % maxX),
      y: 24 + ((n * SPAWN_STEP) % maxY),
    };
  }

  function readStore() {
    try {
      return JSON.parse(localStorage.getItem(STORAGE_KEY) || "[]");
    } catch (e) {
      return [];
    }
  }

  function writeStore() {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(instances));
    } catch (e) {}
  }

  function changed() {
    writeStore();
    window.dispatchEvent(
      new CustomEvent("gb-widgets-changed", { detail: { widgets: instances } })
    );
  }

  function byId(id) {
    for (var i = 0; i < instances.length; i++) {
      if (instances[i].id === id) return instances[i];
    }
    return null;
  }

  function elById(id) {
    return document.getElementById("gb-widget-" + id);
  }

  function focusInst(inst) {
    var el = elById(inst.id);
    if (el) el.style.zIndex = String(++zCounter);
  }

  function persistPos(id, x, y) {
    var inst = byId(id);
    if (!inst) return;
    inst.x = Math.round(x);
    inst.y = Math.round(y);
    changed();
  }

  function persistSize(id, w, h) {
    var inst = byId(id);
    if (!inst) return;
    inst.w = Math.round(w);
    inst.h = Math.round(h);
    changed();
  }

  function findApp(appId) {
    var reg = window.APPS_REGISTRY || [];
    for (var i = 0; i < reg.length; i++) {
      if (reg[i].id === appId) return reg[i];
    }
    return null;
  }

  function makeApi() {
    return {
      onRefresh: function (id) {
        var inst = byId(id);
        var el = elById(id);
        if (inst && el && window.GBWidgetRenderer) {
          window.GBWidgetRenderer.refreshBody(el, inst, makeApi());
        }
      },
      onWindow: function (id) {
        openWindowed(id);
      },
      onPin: function (id) {
        var inst = byId(id);
        if (!inst) return;
        window.dispatchEvent(
          new CustomEvent("gb-launcher-pin-request", {
            detail: {
              kind: "widget",
              id: inst.id,
              title: inst.title,
              appId: inst.appId || null,
              url: inst.url || null,
            },
          })
        );
      },
      onRemove: function (id) {
        remove(id);
      },
      onFocus: function (id) {
        var inst = byId(id);
        if (inst) focusInst(inst);
      },
      persistPos: persistPos,
      persistSize: persistSize,
    };
  }

  function mount(inst) {
    if (!window.GBWidgetRenderer || !pane) return;
    var el = window.GBWidgetRenderer.mount(inst, makeApi());
    el.style.zIndex = String(++zCounter);
    pane.appendChild(el);
  }

  function addInstance(inst) {
    instances.push(inst);
    mount(inst);
    changed();
    focusInst(inst);
  }

  // ── Public API ───────────────────────────────────────────────

  var API = {
    init: function (workspaceEl) {
      if (pane) return;
      pane = document.createElement("div");
      pane.id = "gb-widget-pane";
      pane.className = "gb-widget-pane";
      pane.setAttribute("aria-label", "Desktop widgets");
      var host = workspaceEl || document.getElementById("desktop-content");
      if (host) host.appendChild(pane);
      instances = readStore();
      instances.forEach(function (inst) {
        mount(inst);
      });
    },

    list: function () {
      return instances.slice();
    },

    byId: byId,

    // Definitions for the desktop "Add widget" picker (context menu).
    systemWidgets: function () {
      return SYSTEM_WIDGETS.slice();
    },

    // Context-menu picker listing every system widget; picks position from
    // the triggering mouse event.
    openPicker: function (e) {
      var x = e && e.clientX ? e.clientX : 60;
      var y = e && e.clientY ? e.clientY : 60;
      document
        .querySelectorAll(".desktop-context-menu.gb-widget-picker")
        .forEach(function (m) { m.remove(); });
      var menu = document.createElement("div");
      menu.className = "desktop-context-menu gb-widget-picker";
      menu.style.left = Math.min(x, window.innerWidth - 210) + "px";
      menu.style.top = Math.min(y, window.innerHeight - 260) + "px";
      SYSTEM_WIDGETS.forEach(function (def) {
        var item = document.createElement("div");
        item.className = "desktop-context-item";
        item.textContent = def.title;
        item.addEventListener("click", function () {
          API.addSystem(def.id);
          menu.remove();
        });
        menu.appendChild(item);
      });
      document.body.appendChild(menu);
      setTimeout(function () {
        document.addEventListener(
          "click",
          function () { menu.remove(); },
          { once: true }
        );
      }, 0);
    },

    addSystem: function (id) {
      var def = null;
      SYSTEM_WIDGETS.forEach(function (w) {
        if (w.id === id) def = w;
      });
      if (!def) return;
      var existingId = "sys-" + id;
      if (byId(existingId)) {
        // One instance per system widget: focus the existing one instead of
        // creating a duplicate DOM id.
        this.ensureVisible(existingId);
        return;
      }
      var pos = nextPosition();
      addInstance({
        id: existingId,
        kind: "system",
        title: def.title,
        partial: def.partial,
        w: def.w,
        h: def.h,
        x: pos.x,
        y: pos.y,
      });
    },

    addApp: function (appId) {
      var app = findApp(appId);
      if (!app) return;
      var pos = nextPosition();
      addInstance({
        id: "app-" + appId,
        kind: "app",
        appId: appId,
        title: app.title,
        w: 420,
        h: 320,
        x: pos.x,
        y: pos.y,
      });
    },

    addWeb: function (url, title) {
      var hostname = "Web";
      try {
        hostname = new URL(url).hostname || "Web";
      } catch (e) {
        hostname = "Web";
      }
      var pos = nextPosition();
      addInstance({
        id: "web-" + Date.now(),
        kind: "web",
        title: title || hostname,
        url: url,
        w: 420,
        h: 320,
        x: pos.x,
        y: pos.y,
      });
    },

    addWebPrompt: function () {
      var url = window.prompt("Web address for the widget:", "https://");
      if (!url) return;
      var title = window.prompt("Widget title:", "") || "Web";
      this.addWeb(url, title);
    },

    ensureVisible: function (id) {
      var inst = byId(id);
      if (!inst) return;
      var el = elById(id);
      if (el) {
        el.style.display = "";
        focusInst(inst);
      }
    },

    refresh: function (id) {
      var inst = byId(id);
      var el = elById(id);
      if (inst && el && window.GBWidgetRenderer) {
        window.GBWidgetRenderer.refreshBody(el, inst, makeApi());
      }
    },

    remove: function (id) {
      var el = elById(id);
      if (el) {
        el.dispatchEvent(new CustomEvent("gb-widget-remove"));
        el.remove();
      }
      instances = instances.filter(function (i) {
        return i.id !== id;
      });
      changed();
    },

    openWindowed: function (id) {
      var inst = byId(id);
      if (!inst || !window.WindowManager) return;
      var title = inst.title || "Widget";
      if (inst.kind === "web" && inst.url) {
        var sandbox = window.GBWidgetRenderer.isTrustedOrigin(inst.url)
          ? "allow-scripts allow-same-origin"
          : "allow-scripts";
        var frameTag =
          '<iframe class="gb-widget-frame gb-widget-window-frame" src="' +
          window.GBWidgetRenderer.urlSafe(inst.url) +
          '" sandbox="' +
          sandbox +
          '"></iframe>';
        window.WindowManager.open("widgetwin-" + id, title, frameTag);
        return;
      }
      var partial = inst.partial;
      if (inst.kind === "app") {
        var app = findApp(inst.appId);
        partial = app ? app.hxGet : null;
      }
      if (!partial) return;
      window.WindowManager.open("widgetwin-" + id, title, "");
      fetch(partial + "?_=" + Date.now())
        .then(function (r) {
          return r.text();
        })
        .then(function (html) {
          var body = document.getElementById("window-body-widgetwin-" + id);
          if (body && window.WindowManager._injectBodyContent) {
            window.WindowManager._injectBodyContent("widgetwin-" + id, html);
          }
        })
        .catch(function () {});
    },

    openIsolated: function (id) {
      var inst = byId(id);
      if (!inst) return;
      if (inst.kind === "web" && inst.url) {
        window.open(inst.url, "_blank", "noopener");
        return;
      }
      var partial = inst.partial;
      if (inst.kind === "app") {
        var app = findApp(inst.appId);
        partial = app ? app.hxGet : null;
      }
      if (!partial) return;
      var sep = partial.indexOf("?") === -1 ? "?" : "&";
      window.open(partial + sep + "isolated=1", "_blank", "noopener");
    },
  };

  window.GBWidgets = API;
})();