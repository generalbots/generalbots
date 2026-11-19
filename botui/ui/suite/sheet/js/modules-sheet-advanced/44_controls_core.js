"use strict";
/* Sheets enterprise controls (#2437) — local-only shared core.
 * Namespace: window.SheetControls. This slice never writes server schema:
 * policy preview state is workbook-scoped in localStorage and clearly labelled
 * as local only, with no server enforcement, signing or persistence. */

(function () {
  var dialogSeq = 0;

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
    return "gb-sheet-controls:" + sheetId();
  }

  function defaultAllowances() {
    return {
      selectLocked: true,
      selectUnlocked: true,
      format: false,
      insertDeleteRows: false,
      sort: false,
      filter: false,
      usePivot: false,
    };
  }

  function emptyState() {
    return {
      version: 1,
      localOnly: true,
      protection: {
        scope: "sheet",
        target: "",
        locked: false,
        hash: "",
        allowances: defaultAllowances(),
      },
      inputs: [],
      acl: [],
      classification: { level: "internal", exportPolicy: "allow" },
      links: { allowlist: [], current: [] },
      audit: { retentionDays: 365, entries: [] },
      snapshot: null,
    };
  }

  function normalize(parsed) {
    var base = emptyState();
    if (!parsed || typeof parsed !== "object") return base;
    if (parsed.protection && typeof parsed.protection === "object") {
      base.protection.scope = parsed.protection.scope || base.protection.scope;
      base.protection.target = parsed.protection.target || "";
      base.protection.locked = Boolean(parsed.protection.locked);
      base.protection.hash = parsed.protection.hash || "";
      var allowed = parsed.protection.allowances;
      if (allowed && typeof allowed === "object") {
        Object.keys(base.protection.allowances).forEach(function (key) {
          if (typeof allowed[key] === "boolean") base.protection.allowances[key] = allowed[key];
        });
      }
    }
    if (Array.isArray(parsed.inputs)) base.inputs = parsed.inputs;
    if (Array.isArray(parsed.acl)) base.acl = parsed.acl;
    if (parsed.classification && typeof parsed.classification === "object") {
      base.classification.level = parsed.classification.level || base.classification.level;
      base.classification.exportPolicy =
        parsed.classification.exportPolicy || base.classification.exportPolicy;
    }
    if (parsed.links && typeof parsed.links === "object") {
      if (Array.isArray(parsed.links.allowlist)) base.links.allowlist = parsed.links.allowlist;
      if (Array.isArray(parsed.links.current)) base.links.current = parsed.links.current;
    }
    if (parsed.audit && typeof parsed.audit === "object") {
      var days = Number(parsed.audit.retentionDays);
      if (Number.isFinite(days) && days > 0) base.audit.retentionDays = days;
      if (Array.isArray(parsed.audit.entries)) base.audit.entries = parsed.audit.entries;
    }
    if (parsed.snapshot && typeof parsed.snapshot === "object") base.snapshot = parsed.snapshot;
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

  /* Non-cryptographic preview hash. Never presented as real security. */
  function hash(value) {
    var text = String(value == null ? "" : value);
    var h = 2166136261;
    for (var i = 0; i < text.length; i++) {
      h ^= text.charCodeAt(i);
      h = Math.imul(h, 16777619);
    }
    return (h >>> 0).toString(16).padStart(8, "0");
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
    var region = dialog && dialog.querySelector(".ss-ctrl-status");
    if (!region) return;
    region.textContent = message;
    if (region.classList) region.classList.toggle("ss-ctrl-status-error", Boolean(isError));
  }

  /* Focus-trapped dialog: Esc closes, Tab cycles, focus returns to opener. */
  function makeDialog(titleKey, titleFallback) {
    var restore = document.activeElement;
    var dialog = element("div", "ss-ctrl-dialog");
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");
    var titleId = "ss-ctrl-title-" + (++dialogSeq);
    dialog.setAttribute("aria-labelledby", titleId);
    dialog.tabIndex = -1;

    var header = element("div", "ss-ctrl-header");
    var heading = element("h2", "", t(titleKey, titleFallback));
    heading.id = titleId;
    heading.setAttribute("data-i18n", titleKey);
    var close = element("button", "ss-ctrl-close", "\u00d7");
    close.type = "button";
    close.setAttribute("aria-label", t("sheet-ctrl-close", "Close"));
    close.setAttribute("data-i18n-aria-label", "sheet-ctrl-close");
    header.appendChild(heading);
    header.appendChild(close);

    var note = element("p", "ss-ctrl-local-note", t("sheet-ctrl-local-note",
      "Local only - policy preview in this browser; no server enforcement, signing or persistence."));
    note.setAttribute("data-i18n", "sheet-ctrl-local-note");

    var tabs = element("div", "ss-ctrl-tabs");
    tabs.setAttribute("role", "tablist");
    var status = element("p", "ss-ctrl-status", t("sheet-ctrl-status-ready", "Ready."));
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    status.setAttribute("data-i18n", "sheet-ctrl-status-ready");
    var body = element("div", "ss-ctrl-body");
    dialog.appendChild(note);
    dialog.appendChild(header);
    dialog.appendChild(tabs);
    dialog.appendChild(status);
    dialog.appendChild(body);

    function onKey(event) {
      if (event.key === "Escape") {
        event.preventDefault();
        api.close();
        return;
      }
      if (event.key !== "Tab") return;
      var items = focusables(dialog);
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
      if (dialog.contains && !dialog.contains(event.target)) api.close();
    }

    var api = {
      root: dialog,
      tabs: tabs,
      body: body,
      status: status,
      close: function () {
        if (dialog.isConnected === false) return;
        document.removeEventListener("keydown", onKey, true);
        document.removeEventListener("mousedown", onDown, true);
        if (dialog.remove) dialog.remove();
        if (restore && restore.isConnected !== false && typeof restore.focus === "function") {
          restore.focus();
        }
      },
      setStatus: function (message, isError) { setStatus(dialog, message, isError); },
    };
    close.addEventListener("click", api.close);
    document.addEventListener("keydown", onKey, true);
    document.addEventListener("mousedown", onDown, true);
    var host = document.getElementById("sheet-modal-container") || document.body;
    if (host && host.appendChild) host.appendChild(dialog);
    return api;
  }

  function flash(message) {
    if (window.sheetNotify) window.sheetNotify(message, false);
  }

  var api = {
    t: t,
    sheetId: sheetId,
    storageKey: storageKey,
    emptyState: emptyState,
    state: state,
    save: save,
    mutate: mutate,
    hash: hash,
    element: element,
    escapeHtml: escapeHtml,
    focusables: focusables,
    setStatus: setStatus,
    makeDialog: makeDialog,
    flash: flash,
  };
  window.SheetControls = api;
})();
