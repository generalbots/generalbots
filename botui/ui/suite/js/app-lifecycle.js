"use strict";

// GB App Lifecycle — Mindfulness contract helper (issue #938).
// Every suite app registers under a stable app id; reopening the window
// clears the previous timers/listeners/sockets before re-init, so no
// orphaned handles accumulate across mount/reopen/close cycles.

(function () {
  var registry = {};

  function bucket(appId) {
    if (!registry[appId]) {
      registry[appId] = { intervals: [], timeouts: [], sockets: [], cleanups: [] };
    }
    return registry[appId];
  }

  function cleanup(appId) {
    var b = bucket(appId);
    b.intervals.forEach(clearInterval);
    b.timeouts.forEach(clearTimeout);
    b.sockets.forEach(function (ws) {
      try { ws.close(); } catch (e) {}
    });
    b.cleanups.forEach(function (fn) {
      try { fn(); } catch (e) {}
    });
    registry[appId] = { intervals: [], timeouts: [], sockets: [], cleanups: [] };
    var live = document.querySelector(".gb-app[data-gb-live-status]");
    if (live && live.dataset.gbApp === appId) live.removeAttribute("data-gb-live-status");
  }

  window.GBAppLifecycle = {
    // Call at the top of every app init: wipes handles from the previous mount.
    begin: function (appId) {
      cleanup(appId);
      var root = document.querySelector(".gb-app");
      if (root && !root.dataset.gbApp) root.dataset.gbApp = appId;
      return bucket(appId);
    },
    interval: function (appId, fn, ms) {
      var t = setInterval(fn, ms);
      bucket(appId).intervals.push(t);
      return t;
    },
    timeout: function (appId, fn, ms) {
      var t = setTimeout(fn, ms);
      bucket(appId).timeouts.push(t);
      return t;
    },
    socket: function (appId, ws) {
      bucket(appId).sockets.push(ws);
      return ws;
    },
    onCleanup: function (appId, fn) {
      bucket(appId).cleanups.push(fn);
    },
    end: function (appId) {
      cleanup(appId);
    },

    // Screen-reader + visual status announcement.
    announce: function (message) {
      var el = document.getElementById("gb-sr-live");
      if (!el) {
        el = document.createElement("div");
        el.id = "gb-sr-live";
        el.className = "gb-sr-status";
        el.setAttribute("aria-live", "polite");
        document.body.appendChild(el);
      }
      el.textContent = message;
    },

    // Sets data-gb-state on the nearest .gb-app root; CSS shows the
    // matching intentional-state block. Pass null to clear back to content.
    setState: function (stateName, message) {
      var roots = document.querySelectorAll(".gb-app");
      for (var i = 0; i < roots.length; i++) {
        if (stateName) {
          roots[i].setAttribute("data-gb-state", stateName);
        } else {
          roots[i].removeAttribute("data-gb-state");
        }
        var msgEl = roots[i].querySelector(".gb-state.is-active .gb-state-msg");
        if (msgEl && message !== undefined) msgEl.textContent = message;
      }
      if (message) GBAppLifecycle.announce(message);
    },
  };
})();
