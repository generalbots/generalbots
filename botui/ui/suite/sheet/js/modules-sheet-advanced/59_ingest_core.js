"use strict";
/* Sheets unstructured-to-structured ingest review (#2494) — local-only core.
 * Namespace: window.SheetIngestCore. All state is workbook-scoped localStorage.
 * The parse result is SIMULATED in-browser: no PDF/image/CSV is read, no OCR is
 * performed, no original is uploaded or deleted, nothing reaches the server. */

(function () {
  var THRESHOLD = 0.85;
  var MAX_FILES = 12;
  var MAX_OVERRIDES = 200;

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
    return "gb-sheet-ingest:" + sheetId();
  }

  function defaultPolicy() {
    return {
      retainDays: 30,
      ocrStored: true,
      ocrText: "01/03/2026 OPENING BALANCE 1000.00 | 03/03/2026 OFFICE SUPPLIES 120.50",
      lastRemoved: null,
    };
  }

  function emptyState() {
    return {
      version: 1,
      localOnly: true,
      activeFileId: "stmt",
      files: [
        { id: "stmt", name: "march-statement.pdf", status: "queued", error: "" },
        { id: "inv", name: "invoice-batch.pdf", status: "queued", error: "" },
        { id: "photo", name: "receipt-photo.png", status: "queued", error: "" },
        { id: "bad", name: "corrupt-scan.pdf", status: "queued", error: "" },
      ],
      parses: {},
      mappings: {},
      confirmations: {},
      overrides: [],
      preview: null,
      policy: defaultPolicy(),
    };
  }

  function oneOf(value, allowed, fallback) {
    return allowed.indexOf(value) >= 0 ? value : fallback;
  }

  function clampDays(value, fallback) {
    var parsed = Number(value);
    if (!Number.isFinite(parsed)) return fallback;
    return Math.min(3650, Math.max(1, Math.floor(parsed)));
  }

  function normalize(parsed) {
    var base = emptyState();
    if (!parsed || typeof parsed !== "object") return base;
    if (Array.isArray(parsed.files)) {
      base.files = parsed.files.filter(function (file) {
        return file && typeof file.id === "string";
      }).slice(0, MAX_FILES).map(function (file) {
        return {
          id: file.id.slice(0, 60),
          name: String(file.name || file.id).slice(0, 160),
          status: oneOf(file.status, ["queued", "parsing", "review", "ready", "failed"], "queued"),
          error: String(file.error || "").slice(0, 200),
        };
      });
    }
    if (parsed.parses && typeof parsed.parses === "object") base.parses = parsed.parses;
    if (parsed.mappings && typeof parsed.mappings === "object") base.mappings = parsed.mappings;
    if (parsed.confirmations && typeof parsed.confirmations === "object") base.confirmations = parsed.confirmations;
    if (Array.isArray(parsed.overrides)) base.overrides = parsed.overrides.slice(0, MAX_OVERRIDES);
    if (parsed.preview && typeof parsed.preview === "object") base.preview = parsed.preview;
    if (parsed.policy && typeof parsed.policy === "object") {
      base.policy.retainDays = clampDays(parsed.policy.retainDays, base.policy.retainDays);
      base.policy.ocrStored = Boolean(parsed.policy.ocrStored);
      base.policy.ocrText = String(parsed.policy.ocrText || "").slice(0, 4000);
      base.policy.lastRemoved = parsed.policy.lastRemoved ? String(parsed.policy.lastRemoved) : null;
    }
    if (typeof parsed.activeFileId === "string") base.activeFileId = parsed.activeFileId;
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

  function element(tag, className, text) {
    var node = document.createElement(tag);
    if (className) node.className = className;
    if (text != null) node.textContent = String(text);
    return node;
  }

  function escapeHtml(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  }

  function focusables(root) {
    if (!root || typeof root.querySelectorAll !== "function") return [];
    return Array.prototype.slice.call(root.querySelectorAll(
      'button:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ));
  }

  function setStatus(dialog, message, isError) {
    var region = dialog && dialog.querySelector(".ss-ingest-status");
    if (!region) return;
    region.textContent = message;
    if (region.classList) region.classList.toggle("ss-ingest-status-error", Boolean(isError));
  }

  function makeDialog() {
    var restore = document.activeElement;
    var root = element("div", "ss-ingest-dialog");
    root.setAttribute("role", "dialog");
    root.setAttribute("aria-modal", "true");
    var heading = element("h2", "", t("sheet-ingest-title", "Unstructured ingest - local review"));
    heading.setAttribute("data-i18n", "sheet-ingest-title");
    heading.id = "ss-ingest-title";
    root.setAttribute("aria-labelledby", heading.id);
    root.tabIndex = -1;
    var note = element("p", "ss-ingest-local-note", t("sheet-ingest-local-note",
      "Local only - simulated parse preview in this browser; not saved to the workbook, no OCR, no file upload or deletion."));
    note.setAttribute("data-i18n", "sheet-ingest-local-note");
    var header = element("div", "ss-ingest-header");
    var close = element("button", "ss-ingest-close", "\u00d7");
    close.type = "button";
    close.setAttribute("aria-label", t("sheet-ingest-close", "Close"));
    close.setAttribute("data-i18n-aria-label", "sheet-ingest-close");
    header.appendChild(heading);
    header.appendChild(close);
    var tabs = element("div", "ss-ingest-tabs");
    tabs.setAttribute("role", "tablist");
    var status = element("p", "ss-ingest-status", t("sheet-ingest-status-ready", "Ready."));
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    status.setAttribute("data-i18n", "sheet-ingest-status-ready");
    var body = element("div", "ss-ingest-body");
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
      var items = focusables(root);
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
      status: status,
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

  window.SheetIngestCore = {
    THRESHOLD: THRESHOLD,
    MAX_FILES: MAX_FILES,
    MAX_OVERRIDES: MAX_OVERRIDES,
    t: t,
    sheetId: sheetId,
    storageKey: storageKey,
    state: state,
    save: save,
    mutate: mutate,
    emptyState: emptyState,
    normalize: normalize,
    oneOf: oneOf,
    clampDays: clampDays,
    element: element,
    escapeHtml: escapeHtml,
    focusables: focusables,
    setStatus: setStatus,
    makeDialog: makeDialog,
  };
})();
