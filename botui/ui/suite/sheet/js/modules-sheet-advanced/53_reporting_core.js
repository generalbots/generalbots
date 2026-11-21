"use strict";
/* Sheets print, reporting and scheduled delivery (#2438) — local-only shared
 * core. Namespace: window.SheetReportingCore. All state is workbook-scoped in
 * localStorage, explicitly labelled local only. This layer never generates a
 * real PDF, sends email/chat, schedules a job or writes to the workbook. */

(function () {
  var dialogSeq = 0;
  var MAX_ITEMS = 200;

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
    return "gb-sheet-reporting:" + sheetId();
  }

  function defaultSetup() {
    return {
      orientation: "portrait",
      paper: "A4",
      margins: { top: 0.5, bottom: 0.5, left: 0.5, right: 0.5 },
      scaleMode: "fit",
      fitToWidth: 1,
      fitToHeight: 0,
      percent: 100,
      header: { left: "", center: "", right: "" },
      footer: { left: "", center: "page", right: "sheet" },
      headerVariant: "all",
      footerVariant: "all",
      repeatRows: "",
      repeatCols: "",
      manualBreaks: [],
      pageOrder: "down",
      gridlines: false,
      headings: false,
      blackWhite: false,
    };
  }

  function emptyState() {
    return {
      version: 1,
      localOnly: true,
      printAreas: [],
      pageSetup: defaultSetup(),
      dashboards: [],
      schedules: [],
    };
  }

  function oneOf(value, allowed, fallback) {
    return allowed.indexOf(value) >= 0 ? value : fallback;
  }

  function numberIn(value, min, max, fallback) {
    var parsed = Number(value);
    if (!Number.isFinite(parsed)) return fallback;
    return Math.min(Math.max(parsed, min), max);
  }

  function normalizeSetup(parsed) {
    var base = defaultSetup();
    if (!parsed || typeof parsed !== "object") return base;
    base.orientation = oneOf(parsed.orientation, ["portrait", "landscape"], base.orientation);
    base.paper = oneOf(parsed.paper, ["A4", "A3", "Letter", "Legal"], base.paper);
    base.scaleMode = oneOf(parsed.scaleMode, ["fit", "percent"], base.scaleMode);
    base.fitToWidth = numberIn(parsed.fitToWidth, 0, 20, base.fitToWidth);
    base.fitToHeight = numberIn(parsed.fitToHeight, 0, 20, base.fitToHeight);
    base.percent = numberIn(parsed.percent, 10, 400, base.percent);
    base.headerVariant = oneOf(parsed.headerVariant, ["all", "first", "even", "odd"], base.headerVariant);
    base.footerVariant = oneOf(parsed.footerVariant, ["all", "first", "even", "odd"], base.footerVariant);
    base.pageOrder = oneOf(parsed.pageOrder, ["down", "over"], base.pageOrder);
    ["top", "bottom", "left", "right"].forEach(function (side) {
      if (parsed.margins && typeof parsed.margins === "object") {
        base.margins[side] = numberIn(parsed.margins[side], 0, 3, base.margins[side]);
      }
    });
    ["header", "footer"].forEach(function (section) {
      if (parsed[section] && typeof parsed[section] === "object") {
        ["left", "center", "right"].forEach(function (slot) {
          base[section][slot] = String(parsed[section][slot] == null ? "" : parsed[section][slot]).slice(0, 120);
        });
      }
    });
    base.repeatRows = String(parsed.repeatRows == null ? "" : parsed.repeatRows).slice(0, 40);
    base.repeatCols = String(parsed.repeatCols == null ? "" : parsed.repeatCols).slice(0, 40);
    if (Array.isArray(parsed.manualBreaks)) {
      base.manualBreaks = parsed.manualBreaks.filter(function (brk) {
        return brk && (brk.axis === "col" || brk.axis === "row") && Number.isInteger(brk.before) && brk.before >= 0;
      }).slice(0, 50).map(function (brk) {
        return { axis: brk.axis, before: brk.before };
      });
    }
    ["gridlines", "headings", "blackWhite"].forEach(function (flag) {
      base[flag] = Boolean(parsed[flag]);
    });
    return base;
  }

  function normalize(parsed) {
    var base = emptyState();
    if (!parsed || typeof parsed !== "object") return base;
    if (Array.isArray(parsed.printAreas)) base.printAreas = parsed.printAreas.slice(0, MAX_ITEMS);
    if (Array.isArray(parsed.dashboards)) base.dashboards = parsed.dashboards.slice(0, MAX_ITEMS);
    if (Array.isArray(parsed.schedules)) base.schedules = parsed.schedules.slice(0, MAX_ITEMS);
    base.pageSetup = normalizeSetup(parsed.pageSetup);
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

  function colToNumber(letters) {
    var upper = String(letters || "").toUpperCase();
    var value = 0;
    for (var i = 0; i < upper.length; i++) {
      var code = upper.charCodeAt(i) - 64;
      if (code < 1 || code > 26) return -1;
      value = value * 26 + code;
    }
    return value > 0 ? value - 1 : -1;
  }

  function numberToCol(index) {
    var value = Math.max(0, Number(index) || 0) + 1;
    var out = "";
    while (value > 0) {
      var remainder = (value - 1) % 26;
      out = String.fromCharCode(65 + remainder) + out;
      value = Math.floor((value - 1) / 26);
    }
    return out;
  }

  function parseRange(value) {
    var raw = String(value == null ? "" : value).trim().toUpperCase();
    var match = /^(?:([A-Z0-9_]+)!)?([A-Z]+[1-9][0-9]*):([A-Z]+[1-9][0-9]*)$/.exec(raw);
    if (!match) return null;
    var first = /^([A-Z]+)([1-9][0-9]*)$/.exec(match[2]);
    var last = /^([A-Z]+)([1-9][0-9]*)$/.exec(match[3]);
    if (!first || !last) return null;
    var c1 = colToNumber(first[1]);
    var c2 = colToNumber(last[1]);
    if (c1 < 0 || c2 < 0) return null;
    var r1 = parseInt(first[2], 10) - 1;
    var r2 = parseInt(last[2], 10) - 1;
    return {
      sheet: match[1] || "",
      startCol: Math.min(c1, c2),
      endCol: Math.max(c1, c2),
      startRow: Math.min(r1, r2),
      endRow: Math.max(r1, r2),
      cols: Math.abs(c2 - c1) + 1,
      rows: Math.abs(r2 - r1) + 1,
    };
  }

  function focusables(root) {
    if (!root || typeof root.querySelectorAll !== "function") return [];
    return Array.prototype.slice.call(root.querySelectorAll(
      'button:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ));
  }

  function setStatus(dialog, message, isError) {
    var region = dialog && dialog.querySelector(".ss-rpt-status");
    if (!region) return;
    region.textContent = message;
    if (region.classList) region.classList.toggle("ss-rpt-status-error", Boolean(isError));
  }

  /* Focus-trapped dialog: Esc closes, Tab cycles, focus returns to opener. */
  function makeDialog(titleKey, titleFallback) {
    var restore = document.activeElement;
    var root = element("div", "ss-rpt-dialog");
    root.setAttribute("role", "dialog");
    root.setAttribute("aria-modal", "true");
    var titleId = "ss-rpt-title-" + (++dialogSeq);
    root.setAttribute("aria-labelledby", titleId);
    root.tabIndex = -1;

    var note = element("p", "ss-rpt-local-note", t("sheet-rpt-local-note",
      "Local only - preview in this browser; not saved to the workbook, exported as PDF, scheduled or delivered."));
    note.setAttribute("data-i18n", "sheet-rpt-local-note");
    var header = element("div", "ss-rpt-header");
    var heading = element("h2", "", t(titleKey, titleFallback));
    heading.id = titleId;
    heading.setAttribute("data-i18n", titleKey);
    var close = element("button", "ss-rpt-close", "\u00d7");
    close.type = "button";
    close.setAttribute("aria-label", t("sheet-rpt-close", "Close"));
    close.setAttribute("data-i18n-aria-label", "sheet-rpt-close");
    header.appendChild(heading);
    header.appendChild(close);

    var tabs = element("div", "ss-rpt-tabs");
    tabs.setAttribute("role", "tablist");
    var status = element("p", "ss-rpt-status", t("sheet-rpt-status-ready", "Ready."));
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    status.setAttribute("data-i18n", "sheet-rpt-status-ready");
    var body = element("div", "ss-rpt-body");
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
        if (restore && restore.isConnected !== false && typeof restore.focus === "function") {
          restore.focus();
        }
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

  function flash(message) {
    if (window.sheetNotify) window.sheetNotify(message, false);
  }

  window.SheetReportingCore = {
    t: t,
    sheetId: sheetId,
    storageKey: storageKey,
    defaultSetup: defaultSetup,
    normalizeSetup: normalizeSetup,
    emptyState: emptyState,
    state: state,
    save: save,
    mutate: mutate,
    element: element,
    escapeHtml: escapeHtml,
    colToNumber: colToNumber,
    numberToCol: numberToCol,
    parseRange: parseRange,
    focusables: focusables,
    setStatus: setStatus,
    makeDialog: makeDialog,
    flash: flash,
    MAX_ITEMS: MAX_ITEMS,
  };
})();
