"use strict";

// Per-app launcher visibility: the desktop left rail lets the user turn an
// individual app OFF without touching APPS_REGISTRY. Disabled ids live in
// localStorage["gb_launcher_disabled_apps"] as a plain JSON array of app ids;
// the rail reads them synchronously while rendering, so a hidden app never
// flashes on load. Nothing is removed from the registry — only rail drawing
// is filtered, so the command palette and deep links keep working.

window.GBLauncherAppsPrefs = window.GBLauncherAppsPrefs || {};

(function (mod) {
  var STORAGE_KEY = "gb_launcher_disabled_apps";
  var EVENT = "gb-launcher-apps-changed";
  var ids = null;

  function normalize(list) {
    var out = [];
    if (!Array.isArray(list)) return out;
    list.forEach(function (id) {
      if (typeof id === "string" && id && out.indexOf(id) === -1) out.push(id);
    });
    return out;
  }

  function load() {
    if (ids) return ids;
    var raw = null;
    try {
      raw = localStorage.getItem(STORAGE_KEY);
    } catch (e) {
      raw = null;
    }
    ids = [];
    if (raw) {
      try {
        ids = normalize(JSON.parse(raw));
      } catch (e) {
        // Corrupt value: start clean rather than break the rail.
        ids = [];
      }
    }
    return ids;
  }

  function persist() {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(load()));
    } catch (e) {
      // Private mode / quota: the rail still honours the in-memory choice.
    }
    try {
      window.dispatchEvent(
        new CustomEvent(EVENT, { detail: { disabled: load().slice() } })
      );
    } catch (e) {
      // No CustomEvent (very old shell): callers re-render on their own.
    }
  }

  mod.storageKey = STORAGE_KEY;
  mod.eventName = EVENT;

  mod.disabled = function () {
    return load().slice();
  };

  mod.isEnabled = function (id) {
    return load().indexOf(id) === -1;
  };

  mod.setEnabled = function (id, on) {
    if (!id) return load().slice();
    var list = load();
    var at = list.indexOf(id);
    if (on && at !== -1) list.splice(at, 1);
    else if (!on && at === -1) list.push(id);
    else return list.slice();
    persist();
    return list.slice();
  };

  mod.toggle = function (id) {
    return mod.setEnabled(id, !mod.isEnabled(id));
  };

  mod.enableAll = function () {
    ids = [];
    persist();
    return [];
  };

  mod.onChange = function (cb) {
    if (typeof cb === "function") window.addEventListener(EVENT, cb);
  };
})(window.GBLauncherAppsPrefs);
