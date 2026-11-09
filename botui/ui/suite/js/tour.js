/* Guided tour engine for suite apps.
 *
 * A tour is declared per app and registered with GBTour.define(appId, def):
 *   { version: 1, root: "#app-root", steps: [{ target, titleKey, bodyKey,
 *     title, body, placement }] }
 *
 * The first open of an app shows a start/skip prompt. The result is stored in
 * localStorage under "gb-tour:<appId>:v<version>" as completed, skipped or
 * seen. Tours never call the network and degrade to the authored English copy
 * when a translation key is missing.
 */
(function (window, document) {
  "use strict";

  if (window.GBTour) return;

  var STORAGE_PREFIX = "gb-tour:";
  var DEFAULT_VERSION = 1;
  var T = (window.__GBTour = window.__GBTour || {});
  var queue = [];
  var ready = false;

  T.definitions = Object.create(null);
  T.waiters = Object.create(null);
  T.runtime = {
    state: null,
    promptState: null,
    layer: null,
    lastFocused: null,
    positionScheduled: false,
    listenersBound: false,
  };

  function storageGet(key) {
    try {
      return window.localStorage.getItem(key);
    } catch (e) {
      return null;
    }
  }

  function storageSet(key, value) {
    try {
      window.localStorage.setItem(key, value);
    } catch (e) {
      /* localStorage may be disabled; the tour still works for this visit. */
    }
  }

  function storageRemove(key) {
    try {
      window.localStorage.removeItem(key);
    } catch (e) {
      /* ignore */
    }
  }

  function versionOf(def) {
    return String((def && def.version) || DEFAULT_VERSION);
  }

  function statusKey(appId, def) {
    return STORAGE_PREFIX + appId + ":v" + versionOf(def);
  }

  function status(appId) {
    var def = T.definitions[appId];
    return def ? storageGet(statusKey(appId, def)) : null;
  }

  function isComplete(appId) {
    var value = status(appId);
    return value === "completed" || value === "skipped" || value === "seen";
  }

  function translate(key, fallback, params) {
    var value = null;
    if (window.i18n && typeof window.i18n.t === "function") {
      value = window.i18n.t(key, params);
    }
    if (!value || value === key) {
      value = fallback || key;
      if (params && typeof params === "object") {
        Object.keys(params).forEach(function (name) {
          var pattern = new RegExp("\\{\\s*\\$?" + name + "\\s*\\}", "g");
          value = value.replace(pattern, params[name]);
        });
      }
    }
    return value;
  }

  function setupWizardOpen() {
    var wizard = document.getElementById("setupWizardOverlay");
    if (!wizard) return false;
    var style = window.getComputedStyle(wizard);
    if (style.display === "none" || style.visibility === "hidden") return false;
    return wizard.getBoundingClientRect().height > 0;
  }

  function resolveTarget(step, root) {
    if (!step) return null;
    var target = null;
    if (typeof step.target === "function") {
      try {
        target = step.target(root);
      } catch (e) {
        return null;
      }
    } else if (step.target) {
      target = root ? root.querySelector(step.target) : document.querySelector(step.target);
    }
    if (!target || !document.documentElement.contains(target)) return null;
    return target;
  }

  function mark(appId, value) {
    var def = T.definitions[appId];
    if (def) storageSet(statusKey(appId, def), value);
  }

  function define(appId, def) {
    if (!appId || !def || !Array.isArray(def.steps) || !def.steps.length) return false;
    T.definitions[appId] = def;
    return true;
  }

  function reset(appId) {
    var def = T.definitions[appId];
    if (!def) return;
    storageRemove(statusKey(appId, def));
  }

  function removeWaiter(waiter) {
    waiter.done = true;
    if (waiter.timer) window.clearTimeout(waiter.timer);
    if (T.waiters[waiter.appId] === waiter) delete T.waiters[waiter.appId];
  }

  function scheduleWaiter(waiter) {
    if (waiter.elapsed >= waiter.timeout) {
      removeWaiter(waiter);
      return;
    }
    waiter.elapsed += waiter.delay;
    waiter.delay = Math.min(Math.round(waiter.delay * 1.5), 900);
    waiter.timer = window.setTimeout(function () {
      attemptWaiter(waiter);
    }, waiter.delay);
  }

  function attemptWaiter(waiter) {
    if (waiter.done) return;
    if (!waiter.force && isComplete(waiter.appId)) {
      removeWaiter(waiter);
      return;
    }
    if (T.runtime.state || T.runtime.promptState) return;

    var root = document.querySelector(waiter.def.root || "[data-gb-app]");
    var first = waiter.def.steps && waiter.def.steps[0];
    if (root && resolveTarget(first, root) && !setupWizardOpen()) {
      removeWaiter(waiter);
      if (waiter.force) T.start(waiter.appId, { force: true });
      else T.showPrompt(waiter.appId, waiter.def);
      return;
    }
    scheduleWaiter(waiter);
  }

  function autoStart(appId, options) {
    options = options || {};
    var def = T.definitions[appId];
    if (!def || T.waiters[appId]) return;
    if (!options.force && isComplete(appId)) return;
    var waiter = {
      appId: appId,
      def: def,
      force: !!options.force,
      delay: 150,
      elapsed: 0,
      timeout: options.timeout || 8000,
      timer: null,
      done: false,
    };
    T.waiters[appId] = waiter;
    attemptWaiter(waiter);
  }

  function drainWaiters() {
    Object.keys(T.waiters).forEach(function (appId) {
      var waiter = T.waiters[appId];
      if (!waiter) return;
      waiter.elapsed = 0;
      waiter.delay = 150;
      attemptWaiter(waiter);
    });
  }

  T.storageGet = storageGet;
  T.storageSet = storageSet;
  T.storageRemove = storageRemove;
  T.versionOf = versionOf;
  T.statusKey = statusKey;
  T.status = status;
  T.isComplete = isComplete;
  T.translate = translate;
  T.setupWizardOpen = setupWizardOpen;
  T.resolveTarget = resolveTarget;
  T.mark = mark;
  T.define = define;
  T.reset = reset;
  T.removeWaiter = removeWaiter;
  T.scheduleWaiter = scheduleWaiter;
  T.attemptWaiter = attemptWaiter;
  T.autoStart = autoStart;
  T.drainWaiters = drainWaiters;
  // #1861 — apps that actually ship a tour, for the replay surface (command
  // palette "Show tour: <App>").
  T.definitionIds = function () {
    return Object.keys(T.definitions);
  };

  function overlayReady() {
    return typeof T.start === "function" && typeof T.showPrompt === "function";
  }

  function defer(method) {
    return function () {
      var args = Array.prototype.slice.call(arguments);
      if (ready || overlayReady()) return T[method].apply(null, args);
      queue.push([method, args]);
      return undefined;
    };
  }

  window.GBTour = {
    define: define,
    autoStart: function (appId, options) {
      if (ready || overlayReady()) return autoStart(appId, options);
      queue.push(["autoStart", [appId, options]]);
      return undefined;
    },
    definitionIds: function () {
      return Object.keys(T.definitions);
    },
    start: defer("start"),
    next: defer("next"),
    prev: defer("prev"),
    skip: defer("skip"),
    close: defer("close"),
    status: status,
    isComplete: isComplete,
    reset: reset,
    current: function () {
      return ready ? window.GBTour.current() : null;
    },
  };

  window.__GBTourQueue = queue;
  window.__GBTourBoot = function (api) {
    ready = true;
    window.GBTour = api;
    queue.forEach(function (call) {
      var method = api[call[0]];
      if (typeof method === "function") method.apply(null, call[1]);
    });
    queue.length = 0;
  };

  var modules = ["/suite/js/tour/02_overlay.js", "/suite/js/tour/99_init.js"];

  function loadNext() {
    var src = modules.shift();
    if (!src) return;
    var script = document.createElement("script");
    script.src = src;
    script.async = false;
    script.onload = loadNext;
    script.onerror = loadNext;
    document.head.appendChild(script);
  }

  loadNext();
})(window, document);
