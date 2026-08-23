"use strict";

// GB Widget Renderer (#1160): DOM construction for desktop widgets.
// Kept separate from widget-registry.js so neither file exceeds the
// project line budget. The renderer is pure: it receives an instance and a
// small API object and builds/starts the element tree.

(function () {
  if (window.GBWidgetRenderer) return;

  var MIN_W = 220;
  var MIN_H = 140;

  function findApp(appId) {
    var reg = window.APPS_REGISTRY || [];
    for (var i = 0; i < reg.length; i++) {
      if (reg[i].id === appId) return reg[i];
    }
    return null;
  }

  function iconMarkup(inst) {
    var inner =
      '<rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18"/>';
    if (inst.kind === "web") {
      inner =
        '<circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>';
    } else if (inst.kind === "app") {
      var app = findApp(inst.appId);
      if (app && app.icon) inner = app.icon;
    }
    return (
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">' +
      inner +
      "</svg>"
    );
  }

  function appendScripts(container, scripts) {
    scripts.forEach(function (s) {
      var clone = document.createElement("script");
      Array.from(s.attributes).forEach(function (a) {
        clone.setAttribute(a.name, a.value);
      });
      clone.textContent = s.textContent;
      // Dynamically inserted scripts are async by default; force
      // insertion-order execution so partial dependencies hold.
      if (clone.hasAttribute("src")) clone.async = false;
      container.appendChild(clone);
    });
  }

  function loadPartial(body, partial) {
    fetch(partial + "?_=" + Date.now())
      .then(function (r) {
        if (!r.ok) throw new Error("partial " + partial);
        return r.text();
      })
      .then(function (html) {
        var tmp = document.createElement("div");
        tmp.innerHTML = html;
        var scripts = Array.from(tmp.querySelectorAll("script")).map(
          function (s) {
            s.remove();
            return s;
          }
        );
        body.innerHTML = tmp.innerHTML;
        appendScripts(body, scripts);
        if (window.htmx) htmx.process(body);
      })
      .catch(function () {
        body.innerHTML = '<div class="gb-widget-error">Widget failed to load</div>';
      });
  }

  function isTrustedOrigin(url) {
    try {
      var parsed = new URL(url, window.location.href);
      return parsed.origin === window.location.origin;
    } catch (e) {
      return true;
    }
  }

  function loadWeb(body, inst) {
    var frame = document.createElement("iframe");
    frame.className = "gb-widget-frame";
    frame.setAttribute("loading", "lazy");
    if (isTrustedOrigin(inst.url)) {
      frame.setAttribute("sandbox", "allow-scripts allow-same-origin");
    } else {
      frame.setAttribute("sandbox", "allow-scripts");
      body.classList.add("gb-widget-external");
    }
    frame.src = inst.url;
    body.appendChild(frame);
  }

  function renderBody(el, inst) {
    var body = el.querySelector(".gb-widget-body");
    body.classList.remove("gb-widget-external");
    body.innerHTML = "";
    if (inst.kind === "web" && inst.url) {
      loadWeb(body, inst);
      return;
    }
    var partial = null;
    if (inst.kind === "app") {
      var app = findApp(inst.appId);
      partial = app ? app.hxGet : null;
    } else if (inst.partial) {
      partial = inst.partial;
    }
    if (partial) {
      loadPartial(body, partial);
    } else {
      body.innerHTML = '<div class="gb-widget-error">No widget source configured</div>';
    }
  }

  function buildHeader(inst, api) {
    var head = document.createElement("div");
    head.className = "gb-widget-header";
    head.innerHTML =
      '<span class="gb-widget-icon">' +
      iconMarkup(inst) +
      "</span>" +
      '<span class="gb-widget-title"></span>' +
      '<span class="gb-widget-actions">' +
      '<button class="gb-widget-btn" data-action="refresh" title="Refresh">⟳</button>' +
      '<button class="gb-widget-btn" data-action="window" title="Open windowed">▣</button>' +
      '<button class="gb-widget-btn" data-action="pin" title="Pin to launcher">📌</button>' +
      '<button class="gb-widget-btn" data-action="close" title="Remove">✕</button>' +
      "</span>";
    head.querySelector(".gb-widget-title").textContent = inst.title;
    head
      .querySelector('[data-action="refresh"]')
      .addEventListener("click", function () {
        if (api.onRefresh) api.onRefresh(inst.id);
      });
    head
      .querySelector('[data-action="window"]')
      .addEventListener("click", function () {
        if (api.onWindow) api.onWindow(inst.id);
      });
    head
      .querySelector('[data-action="pin"]')
      .addEventListener("click", function () {
        if (api.onPin) api.onPin(inst.id);
      });
    head
      .querySelector('[data-action="close"]')
      .addEventListener("click", function () {
        if (api.onRemove) api.onRemove(inst.id);
      });
    return head;
  }

  function bindDrag(el, inst, api) {
    var head = el.querySelector(".gb-widget-header");
    head.addEventListener("mousedown", function (e) {
      if (e.target.closest(".gb-widget-btn")) return;
      if (api.onFocus) api.onFocus(inst.id);
      var startX = e.clientX;
      var startY = e.clientY;
      var originX = inst.x || 0;
      var originY = inst.y || 0;
      var host = el.parentElement;

      function onMove(ev) {
        var x = Math.max(0, originX + ev.clientX - startX);
        var y = Math.max(0, originY + ev.clientY - startY);
        // Keep the header reachable: clamp to the workspace so widgets can
        // never be dragged out of view.
        var maxX = host ? Math.max(0, host.clientWidth - 60) : 4000;
        var maxY = host ? Math.max(0, host.clientHeight - 40) : 4000;
        el.style.left = Math.min(x, maxX) + "px";
        el.style.top = Math.min(y, maxY) + "px";
      }
      function onUp() {
        document.removeEventListener("mousemove", onMove);
        document.removeEventListener("mouseup", onUp);
        if (api.persistPos) {
          api.persistPos(
            inst.id,
            parseInt(el.style.left, 10),
            parseInt(el.style.top, 10)
          );
        }
      }
      document.addEventListener("mousemove", onMove);
      document.addEventListener("mouseup", onUp);
    });
  }

  function bindResize(el, inst, api) {
    var grip = el.querySelector(".gb-widget-grip");
    grip.addEventListener("mousedown", function (e) {
      e.preventDefault();
      e.stopPropagation();
      var startX = e.clientX;
      var startY = e.clientY;
      var startW = el.offsetWidth;
      var startH = el.offsetHeight;

      function onMove(ev) {
        el.style.width = Math.max(MIN_W, startW + ev.clientX - startX) + "px";
        el.style.height = Math.max(MIN_H, startH + ev.clientY - startY) + "px";
      }
      function onUp() {
        document.removeEventListener("mousemove", onMove);
        document.removeEventListener("mouseup", onUp);
        if (api.persistSize) {
          api.persistSize(inst.id, el.offsetWidth, el.offsetHeight);
        }
      }
      document.addEventListener("mousemove", onMove);
      document.addEventListener("mouseup", onUp);
    });
  }

  var API = {
    // Builds the widget element for an instance. api may provide:
    // onRefresh(id), onWindow(id), onPin(id), onRemove(id), onFocus(id),
    // persistPos(id,x,y), persistSize(id,w,h).
    mount: function (inst, api) {
      var api = api || {};
      var el = document.createElement("div");
      el.id = "gb-widget-" + inst.id;
      el.className = "gb-widget";
      el.setAttribute("data-widget-id", inst.id);
      el.style.left = (inst.x || 24) + "px";
      el.style.top = (inst.y || 24) + "px";
      el.style.width = (inst.w || 260) + "px";
      el.style.height = (inst.h || 140) + "px";
      el.appendChild(buildHeader(inst, api));
      var body = document.createElement("div");
      body.className = "gb-widget-body";
      el.appendChild(body);
      var grip = document.createElement("div");
      grip.className = "gb-widget-grip";
      el.appendChild(grip);
      bindDrag(el, inst, api);
      bindResize(el, inst, api);
      renderBody(el, inst);
      return el;
    },

    refreshBody: function (el, inst, api) {
      renderBody(el, inst);
    },

    urlSafe: function (url) {
      return String(url || "").replace(/"/g, "&quot;");
    },

    isTrustedOrigin: isTrustedOrigin,
  };

  window.GBWidgetRenderer = API;
})();