/* Split from 07_status.js for #2305 (450-line limit): help/boot/exports. */
(function (window) {
  "use strict";
  var _I = window.__GBStatusInternal || {};
  var WM = _I.WM, WindowManager = _I.WindowManager;
  var store = _I.store, helpTexts = _I.helpTexts;
  var translate = _I.translate, escapeHtml = _I.escapeHtml;
  var renderStatus = _I.renderStatus, listenApp = _I.listenApp;
  var setStatus = _I.setStatus, clearStatus = _I.clearStatus, getStatus = _I.getStatus, controller = _I.controller;

  function registerHelp(target, key, text) {
    var el = resolveTarget(target);
    if (!el) return null;
    var fallback = text != null ? text : helpTexts[key] || "";
    var resolved = translate(key, fallback || key);
    if (key) {
      el.setAttribute("data-help", key);
      helpTexts[key] = fallback || resolved;
    }
    if (resolved) {
      el.setAttribute("data-tooltip", resolved);
      el.setAttribute("title", resolved);
      if (!el.hasAttribute("aria-label") && (el.getAttribute("role") === "button" || el.tagName === "BUTTON")) {
        el.setAttribute("aria-label", resolved);
      }
    }
    return el;
  }

  function refreshHelp(root) {
    var scope = root && root.querySelectorAll ? root : document;
    var nodes = scope.querySelectorAll("[data-help]");
    nodes.forEach(function (el) {
      var key = el.getAttribute("data-help");
      var text = translate(key, helpTexts[key] || "");
      if (text) {
        el.setAttribute("data-tooltip", text);
        el.setAttribute("title", text);
      }
    });
  }

  function boot() {
    var registry = window.APPS_REGISTRY || WM.APPS_REGISTRY || [];
    registry.forEach(function (app) {
      if (app && app.id) listenApp(app.id);
    });
    if (window.WindowManager && window.WindowManager.openWindows) {
      window.WindowManager.openWindows.forEach(function (win) {
        if (win && win.id) {
          listenApp(win.id);
          renderStatus(win.id);
        }
      });
    }
    refreshHelp(document);
  }

  document.addEventListener("gb:status", function (event) {
    var detail = event && event.detail ? event.detail : {};
    if (detail.source === "window-manager") return;
    var id = detail.appId || detail.windowId || detail.id;
    if (id) setStatus(id, detail);
  });

  // Backwards-compatible bridge for the pre-#1849 Vibe event.
  document.addEventListener("gb:vibe-status", function (event) {
    var detail = event && event.detail ? event.detail : {};
    var patch = {
      state: detail.state || "ready",
      text: detail.status || detail.text || "",
      busy: detail.state === "running" || detail.state === "busy",
    };
    if (detail.progress !== undefined) patch.progress = detail.progress;
    setStatus("vibe", patch);
  });

  document.addEventListener("gb-apps-catalog-loaded", function () {
    var registry = window.APPS_REGISTRY || [];
    registry.forEach(function (app) {
      if (app && app.id) listenApp(app.id);
    });
    // The catalog carries the authoritative app descriptions; replay the
    // default payload so hover help picks them up on already-open windows.
    if (window.WindowManager && window.WindowManager.openWindows) {
      window.WindowManager.openWindows.forEach(function (win) {
        if (!win || !win.id) return;
        if (store[win.id] && !store[win.id].customDetail) {
          store[win.id].detail = appDescription(win.id);
        }
        renderStatus(win.id);
      });
    }
  });

  document.addEventListener("gb-window-changed", function (event) {
    var detail = event && event.detail ? event.detail : {};
    if (detail.action === "open" && detail.id) {
      listenApp(detail.id);
      renderStatus(detail.id);
    }
  });

  document.addEventListener("localeChanged", function () {
    refreshHelp(document);
    Object.keys(store).forEach(function (id) {
      if (!store[id].customText) {
        // Default payloads follow the active locale; app-authored text is
        // left untouched.
        var defaultText = translate("status-" + store[id].state, store[id].text);
        if (defaultText) store[id].text = defaultText;
      }
      renderStatus(id);
    });
  });

  // Delegated hover/focus help refresh keeps data-help labels in the active
  // locale even when a control is added after boot.
  document.addEventListener("mouseover", function (event) {
    var el = event.target && event.target.closest ? event.target.closest("[data-help]") : null;
    if (!el) return;
    var key = el.getAttribute("data-help");
    var text = translate(key, helpTexts[key] || "");
    if (text) {
      el.setAttribute("data-tooltip", text);
      el.setAttribute("title", text);
    }
  }, true);
  document.addEventListener("focusin", function (event) {
    var el = event.target && event.target.closest ? event.target.closest("[data-help]") : null;
    if (!el) return;
    var key = el.getAttribute("data-help");
    var text = translate(key, helpTexts[key] || "");
    if (text) {
      el.setAttribute("data-tooltip", text);
      el.setAttribute("title", text);
    }
  }, true);

  WM.defineMethods(WindowManager.prototype, {
    status(id) {
      return controller(id || this.activeWindowId);
    },
    setWindowStatus(id, patch) {
      return setStatus(id || this.activeWindowId, patch);
    },
    clearWindowStatus(id) {
      return clearStatus(id || this.activeWindowId);
    },
    registerStatusHelp(target, key, text) {
      return registerHelp(target, key, text);
    },
    _statusBarInit(id) {
      listenApp(id);
      renderStatus(id);
      return this;
    },
  });

  window.GBStatus = {
    set: setStatus,
    clear: clearStatus,
    get: getStatus,
    render: renderStatus,
    registerHelp: registerHelp,
    help: registerHelp,
    forApp: controller,
    status: controller,
    on: function (appId, callback) {
      if (typeof callback !== "function") return function () {};
      var name = "gb:status-changed:" + String(appId || "");
      document.addEventListener(name, callback);
      return function () { document.removeEventListener(name, callback); };
    },
    escape: escapeHtml,
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})(window);
