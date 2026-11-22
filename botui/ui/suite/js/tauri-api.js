"use strict";
/* #2497 — local, vendored Tauri IPC API.
 *
 * The appliance build sets `withGlobalTauri: false`, so the webview no longer
 * exposes `window.GBTAURI`. Tauri v2 still exposes the low-level
 * `window.__TAURI_INTERNALS__` bridge. This module is the single imported
 * surface the suite uses; it wraps that bridge and is loaded locally (never
 * from a CDN) before every other suite script. In a plain browser `available`
 * is false and every call rejects/no-ops, so the web shell keeps working.
 */
(function (window) {
  var internals = window.__TAURI_INTERNALS__ || null;

  function invoke(command, args) {
    if (!internals || typeof internals.invoke !== "function") {
      return Promise.reject(new Error("Tauri IPC unavailable"));
    }
    return internals.invoke(command, args || {});
  }

  function label() {
    return invoke("plugin:window|current_window");
  }

  function windowApi() {
    function call(command) {
      return label().then(function (current) {
        return invoke("plugin:window|" + command, { label: current });
      });
    }
    return {
      minimize: function () { return call("minimize"); },
      toggleMaximize: function () { return call("toggle_maximize"); },
      close: function () { return call("close"); },
      isMaximized: function () { return call("is_maximized"); }
    };
  }

  var eventApi = {
    emit: function (name, payload) {
      return invoke("plugin:event|emit", { event: name, payload: payload || {} });
    },
    listen: function (name, handler) {
      if (!internals || typeof internals.transformCallback !== "function") {
        return Promise.resolve(function () {});
      }
      return invoke("plugin:event|listen", {
        event: name,
        handler: internals.transformCallback(handler)
      });
    }
  };

  window.GBTAURI = Object.freeze({
    available: !!internals,
    invoke: invoke,
    event: eventApi,
    window: { getCurrentWindow: windowApi }
  });
})(window);
