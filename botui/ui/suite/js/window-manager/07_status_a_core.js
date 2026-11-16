/* WindowManager — shared live status bar contract (7/9). Issue #1849.
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


  // Internals for split module (#2305).
  var _I = window.__GBStatusInternal = window.__GBStatusInternal || {};
  _I.WM = WM; _I.WindowManager = WindowManager; _I.I18N_FALLBACK = I18N_FALLBACK; _I.STATE_ALIASES = STATE_ALIASES;
  _I.store = store; _I.listeners = listeners; _I.helpTexts = helpTexts;
  _I.translate = translate; _I.escapeHtml = escapeHtml; _I.appMeta = appMeta; _I.appTitle = appTitle; _I.appDescription = appDescription;
  _I.normalizeState = normalizeState; _I.defaultPayload = defaultPayload; _I.normalizeProgress = normalizeProgress;
  _I.normalizeMetrics = normalizeMetrics; _I.normalizeSelection = normalizeSelection; _I.normalizeConnection = normalizeConnection;
  _I.buildPayload = buildPayload; _I.statusBarFor = statusBarFor; _I.applyHelp = applyHelp; _I.renderSlot = renderSlot;
})(window);
