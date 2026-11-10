/* WindowManager — shared live status bar contract (7/8). Issue #1849.
 *
 * Every app window owns one footer status bar (added in #725). This module
 * turns that footer into the single, event-driven status contract:
 *
 *   GBStatus.set("browser", { state:"running", text:"Loading…",
 *                             progress:{ value:42, max:100 },
 *                             metrics:[{ label:"Tabs", value:3 }] });
 *   GBStatus.clear("browser");
 *   GBStatus.registerHelp(control, "browser-url-help", "Enter a URL");
 *   GBStatus.forApp("browser").set({ ... });
 *
 * Apps may also publish `new CustomEvent("gb:status:browser", { detail })`
 * (or a generic `gb:status` event with `detail.appId`) and the window manager
 * re-renders the footer. Apps never write the footer DOM directly.
 *
 * Payload (all fields optional):
 *   state      ready | loading | busy | running | paused | saving | success |
 *              warn | error | offline | connected | idle
 *   text       primary text (auto-localised default per state)
 *   detail     extended hover-help text (defaults to the app description)
 *   progress   number 0..100 or { value, max, indeterminate, label }
 *   metrics    [{ label, value, title, key, state }] (or string values)
 *   selection  number | string | { count, text, title }
 *   connection string | boolean | { state, text, title }
 *   busy       convenience flag; mirrors state busy/ready when state is absent
 *   helpKey    optional i18n key used for the primary slot hover help
 *
 * Renderer contract: state LED + primary text, determinate/indeterminate
 * progress, metrics, selection and connection slots. All slots have hover
 * help (data-tooltip + title) and the primary slot is an aria-live region.
 */
(function (window) {
  "use strict";
  var WM = window.__GBWindowManager;
  if (!WM) return;
  var WindowManager = WM.WindowManager;

  var I18N_FALLBACK = {
    "status-ready": "Ready",
    "status-loading": "Loading...",
    "status-busy": "Busy",
    "status-in-progress": "In progress",
    "status-connected": "Connected",
    "status-disconnected": "Disconnected",
    "status-online": "Online",
    "status-offline": "Offline",
    "status-complete": "Complete",
    "status-success": "Success",
    "status-warning": "Warning",
    "status-error": "Error",
    "status-pending": "Pending",
    "label-selected": "selected",
  };

  var STATE_ALIASES = {
    done: "success",
    complete: "success",
    completed: "success",
    failed: "error",
    failure: "error",
    warning: "warn",
    online: "connected",
  };

  // appId -> normalized payload. Survives window close/reopen so an app can
  // publish status before its window exists (or while it is reloading).
  var store = Object.create(null);
  var listeners = Object.create(null);
  var helpTexts = Object.create(null);

  function translate(key, fallback) {
    if (!key) return fallback || "";
    try {
      if (window.i18n && typeof window.i18n.t === "function") {
        var value = window.i18n.t(key);
        if (value && value !== key) return value;
      }
    } catch (e) {
      /* i18n not loaded yet — use the authored fallback */
    }
    if (fallback != null && fallback !== "") return fallback;
    return I18N_FALLBACK[key] || key;
  }

  function escapeHtml(value) {
    return String(value == null ? "" : value).replace(/[&<>"']/g, function (ch) {
      return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[ch];
    });
  }

  function appMeta(id) {
    var registry = window.APPS_REGISTRY || WM.APPS_REGISTRY || [];
    for (var i = 0; i < registry.length; i += 1) {
      if (registry[i] && String(registry[i].id) === String(id)) return registry[i];
    }
    return null;
  }

  function appTitle(id) {
    var app = appMeta(id);
    if (app && window.gbAppTitle) return window.gbAppTitle(app);
    return (app && app.title) || String(id || "");
  }

  function appDescription(id) {
    var app = appMeta(id);
    if (app && window.gbAppDesc) return window.gbAppDesc(app);
    return (app && app.description) || "";
  }

  function normalizeState(state) {
    var key = String(state == null ? "" : state).trim().toLowerCase();
    if (!key) return "ready";
    return STATE_ALIASES[key] || key;
  }

  function defaultPayload(id) {
    return {
      state: "ready",
      text: translate("status-ready", "Ready"),
      detail: appDescription(id),
      progress: null,
      metrics: [],
      selection: null,
      connection: null,
      busy: false,
      helpKey: "status-ready",
    };
  }

  function normalizeProgress(progress) {
    if (progress == null || progress === false || progress === "") return null;
    if (typeof progress === "number" || (typeof progress === "string" && progress.trim() !== "" && !isNaN(Number(progress)))) {
      return { value: Math.max(0, Math.min(100, Number(progress))), max: 100, indeterminate: false, label: "" };
    }
    if (progress === true) return { value: null, max: 100, indeterminate: true, label: "" };
    if (typeof progress !== "object") return null;

    var max = progress.max != null && isFinite(Number(progress.max)) ? Number(progress.max) : 100;
    if (max <= 0) max = 100;
    var raw = progress.value != null ? progress.value
      : progress.percent != null ? progress.percent
      : progress.progress != null ? progress.progress
      : null;
    var indeterminate = progress.indeterminate === true || raw == null || raw === "" || isNaN(Number(raw));
    var value = indeterminate ? null : Math.max(0, Math.min(max, Number(raw)));
    return {
      value: value,
      max: max,
      indeterminate: indeterminate,
      label: progress.label || progress.text || "",
    };
  }

  function normalizeMetrics(metrics) {
    if (metrics == null) return [];
    var list = Array.isArray(metrics) ? metrics : [metrics];
    var out = [];
    list.forEach(function (metric) {
      if (metric == null || metric === "") return;
      if (typeof metric === "string" || typeof metric === "number") {
        out.push({ label: "", value: String(metric), title: "", key: "", state: "" });
        return;
      }
      if (typeof metric !== "object") return;
      var value = metric.value != null ? metric.value : metric.text != null ? metric.text : "";
      var label = metric.label != null ? metric.label : metric.name != null ? metric.name : "";
      out.push({
        label: String(label || ""),
        value: String(value == null ? "" : value),
        title: metric.title || metric.help || metric.detail || "",
        key: metric.key || "",
        state: metric.state ? normalizeState(metric.state) : "",
      });
    });
    return out;
  }

  function normalizeSelection(selection) {
    if (selection == null || selection === "") return null;
    if (typeof selection === "number" || typeof selection === "string") {
      var count = Number(selection);
      var text = selection;
      if (!isNaN(count) && String(selection).trim() !== "") {
        text = count + " " + translate("label-selected", "selected");
      }
      return { count: isNaN(count) ? null : count, text: String(text), title: "" };
    }
    if (typeof selection !== "object") return null;
    var c = selection.count != null ? Number(selection.count) : null;
    var value = selection.text || selection.label ||
      (c != null && isFinite(c) ? c + " " + translate("label-selected", "selected") : "");
    if (!value) return null;
    return {
      count: c != null && isFinite(c) ? c : null,
      text: String(value),
      title: selection.title || selection.help || "",
    };
  }

  function normalizeConnection(connection) {
    if (connection == null || connection === "") return null;
    if (connection === true) return { state: "connected", text: translate("status-connected", "Connected"), title: "" };
    if (connection === false) return { state: "offline", text: translate("status-offline", "Offline"), title: "" };
    if (typeof connection === "string") {
      var state = normalizeState(connection);
      return { state: state, text: translate("status-" + state, connection), title: "" };
    }
    if (typeof connection !== "object") return null;
    var raw = connection.state || connection.status || connection.connection || "";
    var normalized = normalizeState(raw);
    var text = connection.text || connection.label || translate("status-" + normalized, raw || normalized);
    if (!text) return null;
    return { state: normalized, text: String(text), title: connection.title || connection.help || "" };
  }

  function buildPayload(id, patch) {
    patch = patch || {};
    var base = store[id] ? Object.assign({}, store[id]) : defaultPayload(id);
    var next = Object.assign({}, base, patch);
    next.state = normalizeState(next.state);
    next.text = next.text == null ? "" : String(next.text);
    var hasOwnDetail = Object.prototype.hasOwnProperty.call(patch, "detail");
    next.customDetail = hasOwnDetail
      ? patch.detail != null && String(patch.detail) !== ""
      : !!(store[id] && store[id].customDetail);
    next.detail = next.customDetail
      ? String(next.detail == null ? "" : next.detail)
      : appDescription(id);
    next.progress = normalizeProgress(next.progress);
    next.metrics = normalizeMetrics(next.metrics);
    next.selection = normalizeSelection(next.selection);
    next.connection = normalizeConnection(next.connection);
    next.busy = next.busy === true;

    var hasOwnText = Object.prototype.hasOwnProperty.call(patch, "text");
    next.customText = hasOwnText
      ? patch.text != null && String(patch.text) !== ""
      : !!(store[id] && store[id].customText);
    if (hasOwnText) {
      // A custom primary text must not be re-resolved from a generic key.
      next.helpKey = patch.helpKey || "";
    } else if (patch.helpKey) {
      next.helpKey = patch.helpKey;
    }
    // Changing state without supplying text falls back to that state's
    // localised default instead of leaving the previous label.
    if (patch.state !== undefined && !hasOwnText) {
      next.text = "";
      next.customText = false;
    }

    if (patch.state === undefined) {
      if (patch.busy === true && next.state === "ready") next.state = "busy";
      if (patch.busy === false && next.state === "busy") next.state = "ready";
    }
    if (!next.text) {
      var stateKey = "status-ready";
      if (next.state === "loading") { next.text = translate("status-loading", "Loading..."); stateKey = "status-loading"; }
      else if (next.state === "busy") { next.text = translate("status-busy", "Busy"); stateKey = "status-busy"; }
      else if (next.state === "error") { next.text = translate("status-error", "Error"); stateKey = "status-error"; }
      else if (next.state === "offline") { next.text = translate("status-offline", "Offline"); stateKey = "status-offline"; }
      else if (next.state === "connected") { next.text = translate("status-connected", "Connected"); stateKey = "status-connected"; }
      else if (next.state === "success") { next.text = translate("status-complete", "Complete"); stateKey = "status-complete"; }
      else next.text = translate("status-ready", "Ready");
      next.helpKey = stateKey;
    }
    return next;
  }

  function statusBarFor(id) {
    var win = document.getElementById("window-" + id);
    if (!win) return null;
    return win.querySelector(".window-statusbar");
  }

  function applyHelp(element, text, key) {
    if (!element) return;
    if (key) {
      element.setAttribute("data-help", key);
      if (text) helpTexts[key] = text;
    }
    if (text) {
      element.setAttribute("data-tooltip", text);
      element.setAttribute("title", text);
    } else {
      element.removeAttribute("data-tooltip");
      element.removeAttribute("title");
    }
  }

  function renderSlot(bar, selector, value, state, help, helpKey) {
    var el = bar.querySelector(selector);
    if (!el) return null;
    if (value == null || value === "") {
      el.hidden = true;
      el.textContent = "";
      el.removeAttribute("data-state");
      applyHelp(el, "", "");
      return el;
    }
    el.hidden = false;
    el.textContent = value;
    if (state) el.setAttribute("data-state", state);
    else el.removeAttribute("data-state");
    applyHelp(el, help || value, helpKey);
    return el;
  }

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
