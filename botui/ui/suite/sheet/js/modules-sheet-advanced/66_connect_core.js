"use strict";
/* Sheets data connectivity and refresh (#2492) - local-only core.
 * Namespace: window.SheetConnectCore. All state is workbook-scoped localStorage
 * under gb-sheet-connect:<sheetId>. The feed, refresh and Vault resolution are
 * SIMULATED in this browser: no network fetch, no cron, no Vault call and no
 * server persistence. This module only owns state, validation and the dialog. */

(function () {
  var CONNECTOR_TYPES = ["csv", "rest", "postgres", "mysql", "drive", "gsheet"];
  var STEP_TYPES = ["select", "filter", "rename", "type", "unpivot", "group"];
  var FREQUENCIES = ["daily", "weekly", "monthly"];

  function t(key, fallback, params) {
    if (window.gbT) return window.gbT(key, fallback, params);
    if (window.i18n && window.i18n.tOr) return window.i18n.tOr(key, fallback, params);
    if (window.SheetI18n && window.SheetI18n.t) return window.SheetI18n.t(key, fallback, params);
    return fallback;
  }

  function sheetId() {
    try {
      if (window.SheetCore && typeof window.SheetCore.currentSheetId === "function") {
        var id = window.SheetCore.currentSheetId();
        if (id) return String(id);
      }
    } catch (error) {
      return "current";
    }
    if (window.__LOADED_SHEET && window.__LOADED_SHEET.id) {
      return String(window.__LOADED_SHEET.id);
    }
    return "current";
  }

  function storageKey() {
    return "gb-sheet-connect:" + sheetId();
  }

  function defaultParameters() {
    return [
      {
        id: "p-date-from", key: "dateFrom", kind: "date",
        labelKey: "sheet-connect-param-date-from", labelFallback: "Date from",
        promptKey: "sheet-connect-param-prompt-date", promptFallback: "Start of the reporting period",
        defaultValue: "2026-03-01", value: "2026-03-01",
      },
      {
        id: "p-date-to", key: "dateTo", kind: "date",
        labelKey: "sheet-connect-param-date-to", labelFallback: "Date to",
        promptKey: "sheet-connect-param-prompt-date", promptFallback: "Start of the reporting period",
        defaultValue: "2026-03-31", value: "2026-03-31",
      },
      {
        id: "p-region", key: "region", kind: "text",
        labelKey: "sheet-connect-param-region", labelFallback: "Region",
        promptKey: "sheet-connect-param-prompt-region", promptFallback: "Region to filter, or leave blank for all",
        defaultValue: "", value: "",
      },
    ];
  }

  function emptyState() {
    return {
      version: 1,
      localOnly: true,
      connectors: [],
      allowlist: ["data.example.com"],
      steps: [],
      parameters: defaultParameters(),
      schedule: {
        enabled: true, frequency: "daily", hour: 6, timezone: "UTC",
        notifyOnFailure: true, lastRefreshAt: null, nextRefreshAt: null,
        lastStatus: "never", lastError: "",
      },
      snapshot: null,
      audit: [],
      offline: false,
      simulateFailure: false,
      activeConnectorId: null,
      selectedCell: null,
    };
  }

  function oneOf(value, allowed, fallback) {
    return allowed.indexOf(value) >= 0 ? value : fallback;
  }

  function text(value, max) {
    return String(value == null ? "" : value).slice(0, max || 200);
  }

  function normalizeConnector(raw) {
    if (!raw || typeof raw !== "object" || !raw.id) return null;
    return {
      id: text(raw.id, 60),
      name: text(raw.name, 80),
      type: oneOf(raw.type, CONNECTOR_TYPES, "csv"),
      config: raw.config && typeof raw.config === "object" ? raw.config : {},
      createdAt: text(raw.createdAt, 40),
    };
  }

  function normalizeStep(raw) {
    if (!raw || typeof raw !== "object" || !raw.id) return null;
    return {
      id: text(raw.id, 60),
      type: oneOf(raw.type, STEP_TYPES, "select"),
      enabled: raw.enabled !== false,
      config: raw.config && typeof raw.config === "object" ? raw.config : {},
    };
  }

  function normalizeParameter(raw, index) {
    var base = defaultParameters()[index];
    if (!raw || typeof raw !== "object") return base;
    return {
      id: text(raw.id, 60),
      key: text(raw.key, 40),
      kind: oneOf(raw.kind, ["date", "text"], "text"),
      labelKey: text(raw.labelKey, 80),
      labelFallback: text(raw.labelFallback, 80),
      promptKey: text(raw.promptKey, 80),
      promptFallback: text(raw.promptFallback, 120),
      defaultValue: text(raw.defaultValue, 120),
      value: text(raw.value, 120),
    };
  }

  function normalize(raw) {
    var base = emptyState();
    if (!raw || typeof raw !== "object") return base;
    if (Array.isArray(raw.connectors)) {
      base.connectors = raw.connectors.map(normalizeConnector).filter(Boolean).slice(0, 40);
    }
    if (Array.isArray(raw.allowlist)) {
      base.allowlist = raw.allowlist.map(function (host) { return text(host, 200).toLowerCase(); })
        .filter(function (host) { return host.length > 0; }).slice(0, 100);
    }
    if (Array.isArray(raw.steps)) {
      base.steps = raw.steps.map(normalizeStep).filter(Boolean).slice(0, 60);
    }
    if (Array.isArray(raw.parameters)) {
      base.parameters = raw.parameters.slice(0, 20).map(normalizeParameter)
        .filter(function (parameter) { return parameter.key.length > 0; });
      if (!base.parameters.length) base.parameters = defaultParameters();
    }
    if (raw.schedule && typeof raw.schedule === "object") {
      var s = raw.schedule;
      base.schedule.frequency = oneOf(s.frequency, FREQUENCIES, "daily");
      base.schedule.hour = Number.isFinite(Number(s.hour)) ? Math.min(23, Math.max(0, Math.floor(Number(s.hour)))) : 6;
      base.schedule.timezone = text(s.timezone, 60) || "UTC";
      base.schedule.enabled = Boolean(s.enabled);
      base.schedule.notifyOnFailure = Boolean(s.notifyOnFailure);
      base.schedule.lastRefreshAt = s.lastRefreshAt ? text(s.lastRefreshAt, 40) : null;
      base.schedule.nextRefreshAt = s.nextRefreshAt ? text(s.nextRefreshAt, 40) : null;
      base.schedule.lastStatus = oneOf(s.lastStatus, ["never", "success", "failed"], "never");
      base.schedule.lastError = text(s.lastError, 200);
    }
    if (raw.snapshot && typeof raw.snapshot === "object") base.snapshot = raw.snapshot;
    if (Array.isArray(raw.audit)) base.audit = raw.audit.slice(0, 200);
    base.offline = Boolean(raw.offline);
    base.simulateFailure = Boolean(raw.simulateFailure);
    base.activeConnectorId = raw.activeConnectorId ? text(raw.activeConnectorId, 60) : null;
    base.selectedCell = raw.selectedCell && typeof raw.selectedCell === "object" ? raw.selectedCell : null;
    return base;
  }

  function state() {
    try {
      var raw = localStorage.getItem(storageKey());
      if (!raw) return emptyState();
      return normalize(JSON.parse(raw));
    } catch (error) {
      return emptyState();
    }
  }

  function save(next) {
    try {
      localStorage.setItem(storageKey(), JSON.stringify(next));
      return true;
    } catch (error) {
      return false;
    }
  }

  function mutate(fn) {
    var current = state();
    var result = fn(current);
    save(current);
    return result;
  }

  function element(tag, className, value) {
    var node = document.createElement(tag);
    if (className) node.className = className;
    if (value != null) node.textContent = String(value);
    return node;
  }

  function focused(root) {
    if (!root || typeof root.querySelectorAll !== "function") return [];
    return Array.prototype.slice.call(root.querySelectorAll(
      'button:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ));
  }

  function setStatus(dialog, message, isError) {
    var region = dialog && dialog.querySelector(".ss-connect-status");
    if (!region) return;
    region.textContent = message;
    if (region.classList) region.classList.toggle("ss-connect-status-error", Boolean(isError));
  }

  function makeDialog() {
    var restore = document.activeElement;
    var root = element("div", "ss-connect-dialog");
    root.setAttribute("role", "dialog");
    root.setAttribute("aria-modal", "true");
    root.tabIndex = -1;
    var heading = element("h2", "", t("sheet-connect-title", "Data connectivity and refresh - local preview"));
    heading.setAttribute("data-i18n", "sheet-connect-title");
    heading.id = "ss-connect-title";
    root.setAttribute("aria-labelledby", heading.id);
    var note = element("p", "ss-connect-local-note", t("sheet-connect-local-note",
      "Local only - simulated feed and refresh in this browser; not saved to the workbook. No network fetch, scheduling, Vault call or server persistence."));
    note.setAttribute("data-i18n", "sheet-connect-local-note");
    var header = element("div", "ss-connect-header");
    var close = element("button", "ss-connect-close", "\u00d7");
    close.type = "button";
    close.setAttribute("aria-label", t("sheet-connect-close", "Close"));
    close.setAttribute("data-i18n-aria-label", "sheet-connect-close");
    header.appendChild(heading);
    header.appendChild(close);
    var tabs = element("div", "ss-connect-tabs");
    tabs.setAttribute("role", "tablist");
    var status = element("p", "ss-connect-status", t("sheet-connect-status-ready", "Ready."));
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    status.setAttribute("data-i18n", "sheet-connect-status-ready");
    var body = element("div", "ss-connect-body");
    root.appendChild(note);
    root.appendChild(header);
    root.appendChild(tabs);
    root.appendChild(status);
    root.appendChild(body);

    function onKey(event) {
      if (event.key === "Escape") {
        event.preventDefault();
        api.close();
        return;
      }
      if (event.key !== "Tab") return;
      var items = focused(root);
      if (!items.length) return;
      var first = items[0];
      var last = items[items.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    }

    function onDown(event) {
      if (root.contains && !root.contains(event.target)) api.close();
    }

    var api = {
      root: root,
      tabs: tabs,
      body: body,
      close: function () {
        if (root.isConnected === false) return;
        document.removeEventListener("keydown", onKey, true);
        document.removeEventListener("pointerdown", onDown, true);
        if (root.remove) root.remove();
        if (restore && restore.isConnected !== false && typeof restore.focus === "function") restore.focus();
      },
      setStatus: function (message, isError) { setStatus(root, message, isError); },
    };
    close.addEventListener("click", api.close);
    document.addEventListener("keydown", onKey, true);
    document.addEventListener("pointerdown", onDown, true);
    var host = document.getElementById("sheet-modal-container") || document.body;
    if (host && host.appendChild) host.appendChild(root);
    return api;
  }

  window.SheetConnectCore = {
    CONNECTOR_TYPES: CONNECTOR_TYPES,
    STEP_TYPES: STEP_TYPES,
    FREQUENCIES: FREQUENCIES,
    t: t,
    sheetId: sheetId,
    storageKey: storageKey,
    defaultParameters: defaultParameters,
    emptyState: emptyState,
    normalize: normalize,
    state: state,
    save: save,
    mutate: mutate,
    oneOf: oneOf,
    text: text,
    element: element,
    focused: focused,
    setStatus: setStatus,
    makeDialog: makeDialog,
  };
})();
