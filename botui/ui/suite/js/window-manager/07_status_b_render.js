/* Split from 07_status.js for #2305 (450-line limit): rendering and status operations. */
(function (window) {
  "use strict";
  var _I = window.__GBStatusInternal || {};
  var WM = _I.WM, WindowManager = _I.WindowManager;
  var store = _I.store, listeners = _I.listeners, helpTexts = _I.helpTexts;
  var translate = _I.translate, escapeHtml = _I.escapeHtml, appTitle = _I.appTitle, appDescription = _I.appDescription;
  var defaultPayload = _I.defaultPayload, buildPayload = _I.buildPayload, statusBarFor = _I.statusBarFor, applyHelp = _I.applyHelp, renderSlot = _I.renderSlot;
  var normalizeState = _I.normalizeState;

  function renderStatus(id) {
    var key = String(id || "");
    if (!key) return false;
    var bar = statusBarFor(key);
    if (!bar) return false;

    var data = store[key] || defaultPayload(key);
    var title = appTitle(key);
    bar.dataset.state = data.state;
    bar.dataset.appId = key;
    bar.setAttribute("aria-label", (title ? title + " " : "") + "status");

    var primary = bar.querySelector(".window-statusbar-status");
    if (primary) {
      primary.textContent = data.text;
      primary.setAttribute("data-state", data.state);
      primary.setAttribute("aria-live", data.state === "error" ? "assertive" : "polite");
      var primaryHelp = data.detail ? data.text + " — " + data.detail : data.text;
      // When the app supplied a description, that is the useful hover text;
      // don't let the generic state key overwrite it with just "Ready".
      applyHelp(primary, primaryHelp, data.detail ? "" : (data.helpKey || ""));
    }

    var progress = bar.querySelector(".window-statusbar-progress");
    if (progress) {
      var p = data.progress;
      if (!p) {
        progress.hidden = true;
        progress.classList.remove("is-indeterminate");
        progress.removeAttribute("aria-valuenow");
        progress.removeAttribute("aria-valuetext");
        var emptyFill = progress.querySelector(".window-statusbar-progress-fill");
        if (emptyFill) emptyFill.style.width = "0%";
        renderSlot(bar, ".window-statusbar-progress-label", "", "", "", "");
      } else {
        progress.hidden = false;
        progress.classList.toggle("is-indeterminate", !!p.indeterminate);
        progress.setAttribute("aria-valuemin", "0");
        progress.setAttribute("aria-valuemax", String(p.max || 100));
        var fill = progress.querySelector(".window-statusbar-progress-fill");
        var label = progress.querySelector(".window-statusbar-progress-label");
        var percent = p.indeterminate ? null : Math.round((p.value / (p.max || 100)) * 100);
        if (fill) fill.style.width = p.indeterminate ? "100%" : percent + "%";
        if (p.indeterminate) {
          progress.removeAttribute("aria-valuenow");
          progress.setAttribute("aria-valuetext", p.label || translate("status-in-progress", "In progress"));
        } else {
          progress.setAttribute("aria-valuenow", String(percent));
          progress.setAttribute("aria-valuetext", (p.label ? p.label + " " : "") + percent + "%");
        }
        if (label) {
          label.textContent = p.label || (p.indeterminate ? translate("status-in-progress", "In progress") : percent + "%");
          label.hidden = !label.textContent;
        }
        applyHelp(progress, p.label ? p.label + (percent == null ? "" : " — " + percent + "%") :
          translate("status-in-progress", "In progress"), "");
      }
    }

    var metrics = bar.querySelector(".window-statusbar-metrics");
    if (metrics) {
      metrics.textContent = "";
      if (!data.metrics.length) {
        metrics.hidden = true;
      } else {
        metrics.hidden = false;
        data.metrics.forEach(function (metric) {
          var item = document.createElement("span");
          item.className = "window-statusbar-metric status-item";
          item.textContent = (metric.label ? metric.label + ": " : "") + metric.value;
          if (metric.state) item.setAttribute("data-state", metric.state);
          applyHelp(item, metric.title || item.textContent, metric.key);
          metrics.appendChild(item);
        });
      }
    }

    var selection = data.selection;
    renderSlot(
      bar,
      ".window-statusbar-selection",
      selection ? selection.text : "",
      "",
      selection ? selection.title || selection.text : "",
      ""
    );

    var connection = data.connection;
    var connectionEl = renderSlot(
      bar,
      ".window-statusbar-connection",
      connection ? connection.text : "",
      connection ? connection.state : "",
      connection ? connection.title || connection.text : "",
      ""
    );
    if (connectionEl && connection && connection.state) connectionEl.setAttribute("data-state", connection.state);

    var titleEl = bar.querySelector(".window-statusbar-title");
    if (titleEl) {
      titleEl.textContent = title;
      applyHelp(titleEl, data.detail ? title + " — " + data.detail : title, "");
    }
    return true;
  }

  function emitStatus(id, payload, action) {
    var detail = Object.assign({}, payload, {
      appId: id,
      windowId: id,
      action: action || "set",
      // Marks events produced by the manager itself; listeners must ignore
      // these to avoid a set → event → set feedback loop.
      source: "window-manager",
    });
    ["gb:status:" + id, "gb:status-changed:" + id, "gb:status"].forEach(function (name) {
      document.dispatchEvent(new CustomEvent(name, { detail: detail }));
    });
  }

  function listenApp(id) {
    var key = String(id || "");
    if (!key || listeners[key]) return;
    var handler = function (event) {
      var detail = event && event.detail ? event.detail : {};
      if (detail.source === "window-manager") return;
      setStatus(key, detail);
    };
    listeners[key] = handler;
    document.addEventListener("gb:status:" + key, handler);
  }

  function activeWindowId() {
    return window.WindowManager && window.WindowManager.activeWindowId || "";
  }

  function setStatus(appId, patch) {
    if (appId && typeof appId === "object") {
      patch = appId;
      appId = patch.appId || patch.windowId || patch.id || activeWindowId();
    }
    var id = String(appId || activeWindowId() || "");
    if (!id) return null;
    listenApp(id);
    var next = buildPayload(id, patch || {});
    store[id] = next;
    renderStatus(id);
    emitStatus(id, next, "set");
    return controller(id);
  }

  function clearStatus(appId) {
    if (appId && typeof appId === "object") {
      appId = appId.appId || appId.windowId || appId.id || activeWindowId();
    }
    var id = String(appId || activeWindowId() || "");
    if (!id) return null;
    listenApp(id);
    var next = defaultPayload(id);
    store[id] = next;
    renderStatus(id);
    emitStatus(id, next, "clear");
    return controller(id);
  }

  function getStatus(appId) {
    if (appId && typeof appId === "object") {
      appId = appId.appId || appId.windowId || appId.id || activeWindowId();
    }
    var id = String(appId || activeWindowId() || "");
    if (!id) return null;
    return store[id] ? Object.assign({}, store[id]) : defaultPayload(id);
  }

  function controller(appId) {
    var id = String(appId || activeWindowId() || "");
    return {
      id: id,
      set: function (patch) { return setStatus(id, patch); },
      update: function (patch) { return setStatus(id, patch); },
      clear: function () { return clearStatus(id); },
      get: function () { return getStatus(id); },
      render: function () { return renderStatus(id); },
      progress: function (value, label) {
        return setStatus(id, {
          progress: value == null ? null : { value: value, label: label || "" },
          busy: value != null,
        });
      },
      metrics: function (metrics) { return setStatus(id, { metrics: metrics }); },
      help: function (target, key, text) { return registerHelp(target, key, text); },
      registerHelp: function (target, key, text) { return registerHelp(target, key, text); },
    };
  }

  function resolveTarget(target) {
    if (!target) return null;
    if (typeof target === "string") {
      try {
        return document.querySelector(target);
      } catch (e) {
        return null;
      }
    }
    if (target.nodeType === 1) return target;
    if (target[0] && target[0].nodeType === 1) return target[0];
    return null;
  }


  _I.renderStatus = renderStatus; _I.emitStatus = emitStatus; _I.listenApp = listenApp; _I.activeWindowId = activeWindowId;
  _I.setStatus = setStatus; _I.clearStatus = clearStatus; _I.getStatus = getStatus; _I.controller = controller;
})(window);
