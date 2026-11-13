"use strict";

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
  }

  window.GBAppLifecycle = {
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
    announce: function (message) {
      var el = document.getElementById("gb-sr-live");
      if (!el) {
        el = document.createElement("div");
        el.id = "gb-sr-live";
        el.className = "gb-sr-status";
        el.setAttribute("aria-live", "polite");
        document.body.appendChild(el);
      }
      // #2155 UI-REG-01c: clear before setting so polite live regions pick up
      // repeated/transition text and stale "Connecting…" cannot linger in DOM.
      if (el.textContent !== message) {
        el.textContent = "";
      }
      el.textContent = message;
    },
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
        // Clear/refresh static loading placeholders when leaving loading state
        // so stale "Connecting…" text does not persist in app state regions.
        if (!stateName && message !== undefined) {
          var loadingMsgs = roots[i].querySelectorAll(
            ".gb-state-loading .gb-state-msg, .gb-state.is-loading .gb-state-msg"
          );
          for (var j = 0; j < loadingMsgs.length; j++) {
            loadingMsgs[j].textContent = message;
          }
        }
      }
      if (message) window.GBAppLifecycle.announce(message);
    },
  };
})();
