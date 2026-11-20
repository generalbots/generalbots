"use strict";
/* Sheets structured data & what-if (#2436) — local-only shared core.
 * Namespace: window.SheetStructured. This slice never writes server schema:
 * authoring state is workbook-scoped in localStorage and clearly labelled.
 * It reuses existing read-only cell helpers (31_charting_data) when present. */

(function () {
  let dialogSeq = 0;

  function t(key, fallback, params) {
    if (window.gbT) return window.gbT(key, fallback, params);
    if (window.i18n && window.i18n.tOr) return window.i18n.tOr(key, fallback, params);
    if (window.SheetI18n && window.SheetI18n.t) return window.SheetI18n.t(key, fallback, params);
    return fallback;
  }

  function activeGrid() {
    return window.SheetVirtualGrid || null;
  }

  function sheetId() {
    try {
      if (window.SheetCore && typeof window.SheetCore.currentSheetId === "function") {
        const id = window.SheetCore.currentSheetId();
        if (id) return String(id);
      }
    } catch (_) { /* fall through */ }
    if (window.__LOADED_SHEET && window.__LOADED_SHEET.id) return String(window.__LOADED_SHEET.id);
    return "current";
  }

  function storageKey() {
    return "gb-sheet-structured:" + sheetId();
  }

  function emptyState() {
    return { sparklines: [], slicers: [], scenarios: [], localOnly: true };
  }

  function state() {
    try {
      const raw = localStorage.getItem(storageKey());
      if (!raw) return emptyState();
      const parsed = JSON.parse(raw);
      const base = emptyState();
      ["sparklines", "slicers", "scenarios"].forEach(function (name) {
        if (Array.isArray(parsed[name])) base[name] = parsed[name];
      });
      return base;
    } catch (_) {
      return emptyState();
    }
  }

  function save(next) {
    try {
      localStorage.setItem(storageKey(), JSON.stringify(next));
      return true;
    } catch (_) {
      return false;
    }
  }

  function mutate(fn) {
    const current = state();
    const result = fn(current);
    save(current);
    return result;
  }

  function parseRef(ref) {
    if (window.SheetChartingData && window.SheetChartingData.parseRef) {
      return window.SheetChartingData.parseRef(ref);
    }
    if (typeof ref !== "string") return null;
    const match = /^\$?([A-Za-z]{1,3})\$?([0-9]+)$/.exec(ref.trim());
    if (!match) return null;
    let col = 0;
    const letters = match[1].toUpperCase();
    for (let i = 0; i < letters.length; i++) col = col * 26 + (letters.charCodeAt(i) - 64);
    return (parseInt(match[2], 10) < 1 || col > 16384) ? null : { col: col - 1, row: parseInt(match[2], 10) - 1 };
  }

  function parseRange(text) {
    if (window.SheetChartingData && window.SheetChartingData.parseRange) {
      return window.SheetChartingData.parseRange(text);
    }
    if (typeof text !== "string") return null;
    const parts = text.trim().split("!");
    const refs = parts[parts.length - 1].split(":");
    if (refs.length !== 2) return null;
    const a = parseRef(refs[0]);
    const b = parseRef(refs[1]);
    if (!a || !b) return null;
    return {
      start: { row: Math.min(a.row, b.row), col: Math.min(a.col, b.col) },
      end: { row: Math.max(a.row, b.row), col: Math.max(a.col, b.col) },
    };
  }

  function colName(index) {
    let n = index + 1;
    let out = "";
    while (n > 0) {
      const rem = (n - 1) % 26;
      out = String.fromCharCode(65 + rem) + out;
      n = Math.floor((n - 1) / 26);
    }
    return out;
  }

  function refOf(row, col) {
    return colName(col) + (row + 1);
  }

  function cellData(row, col) {
    const grid = activeGrid();
    const key = row + "," + col;
    if (grid && grid.cells) {
      const found = grid.cells instanceof Map ? grid.cells.get(key) : grid.cells[key];
      if (found) return found;
    }
    const sheet = window.__LOADED_SHEET;
    let wsIndex = 0;
    try {
      const parsed = parseInt(sessionStorage.getItem("sheet_active_index") || "0", 10);
      if (!isNaN(parsed) && parsed >= 0) wsIndex = parsed;
    } catch (_) { /* default */ }
    const ws = sheet && sheet.worksheets && sheet.worksheets[wsIndex];
    if (ws && ws.cells) {
      return ws.cells instanceof Map ? ws.cells.get(key) : ws.cells[key];
    }
    return null;
  }

  function cellValue(row, col) {
    const data = cellData(row, col);
    if (!data) return "";
    if (data.value != null && String(data.value) !== "") return data.value;
    if (data.formula) return data.formula;
    return "";
  }

  function numeric(value) {
    if (value === "" || value === null || value === undefined || typeof value === "boolean") return null;
    const n = Number(String(value).replace(/,/g, ""));
    return Number.isFinite(n) ? n : null;
  }

  function writeValues(entries, labelKey, labelFallback) {
    const grid = activeGrid();
    const keys = [];
    const refs = [];
    entries.forEach(function (entry) {
      const pos = parseRef(entry.ref);
      if (!pos) return;
      keys.push(pos.row + "," + pos.col);
      refs.push(entry.ref);
    });
    if (!keys.length || !grid || !grid.cells) return false;
    const undo = window.SheetUndo;
    const previous = window.__gbSuspendUndo;
    window.__gbSuspendUndo = true;
    try {
      if (undo && undo.beforeBulk) undo.beforeBulk(t(labelKey, labelFallback), keys);
      entries.forEach(function (entry, index) {
        if (index >= refs.length) return;
        const pos = parseRef(entry.ref);
        const value = String(entry.value);
        grid.cells.set(pos.row + "," + pos.col, { value: value });
        if (window.SheetAPI && window.SheetAPI.updateCell) {
          const done = window.SheetAPI.updateCell(entry.ref, value);
          if (done && typeof done.catch === "function") done.catch(function () { /* local slice keeps its value */ });
        }
      });
      if (undo && undo.recordBulk) undo.recordBulk();
    } finally {
      window.__gbSuspendUndo = previous;
    }
    grid.lastRenderedRange = null;
    if (grid.requestRange) grid.requestRange();
    return true;
  }

  function writeValue(ref, value, labelKey, labelFallback) {
    return writeValues([{ ref: ref, value: value }], labelKey, labelFallback);
  }

  function escapeHtml(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  }

  function focusables(root) {
    return Array.prototype.slice.call(
      root.querySelectorAll("button:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex='-1'])"),
    ).filter(function (node) { return node.offsetParent !== null || node === root; });
  }

  function setStatus(dialog, message, isError) {
    const region = dialog && dialog.querySelector(".ss-structured-status");
    if (!region) return;
    region.textContent = message;
    region.classList.toggle("ss-structured-status-error", Boolean(isError));
  }

  function createDialog(titleKey, titleFallback) {
    const restore = document.activeElement;
    const dialog = document.createElement("div");
    dialog.className = "ss-structured-dialog";
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");
    const titleId = "ss-structured-title-" + (++dialogSeq);
    dialog.setAttribute("aria-labelledby", titleId);
    dialog.tabIndex = -1;

    const header = document.createElement("div");
    header.className = "ss-structured-header";
    const heading = document.createElement("h2");
    heading.id = titleId;
    heading.textContent = t(titleKey, titleFallback);
    heading.setAttribute("data-i18n", titleKey);
    const close = document.createElement("button");
    close.type = "button";
    close.className = "ss-structured-close";
    close.textContent = "×";
    close.setAttribute("aria-label", t("sheet-structured-close", "Close"));
    close.setAttribute("data-i18n-aria-label", "sheet-structured-close");
    header.appendChild(heading);
    header.appendChild(close);

    const body = document.createElement("div");
    body.className = "ss-structured-body";
    const footer = document.createElement("div");
    footer.className = "ss-structured-footer";
    const status = document.createElement("div");
    status.className = "ss-structured-status";
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    status.textContent = t("sheet-structured-status-ready", "Ready.");
    status.setAttribute("data-i18n", "sheet-structured-status-ready");
    footer.appendChild(status);

    dialog.appendChild(header);
    dialog.appendChild(body);
    dialog.appendChild(footer);

    function onKey(event) {
      if (event.key === "Escape") {
        event.preventDefault();
        api.close();
        return;
      }
      if (event.key !== "Tab") return;
      const items = focusables(dialog);
      if (!items.length) return;
      const first = items[0];
      const last = items[items.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    }

    function onDown(event) {
      if (!dialog.contains(event.target)) api.close();
    }

    const api = {
      dialog: dialog,
      body: body,
      footer: footer,
      status: status,
      close: function () {
        if (!dialog.isConnected) return;
        document.removeEventListener("keydown", onKey, true);
        document.removeEventListener("mousedown", onDown, true);
        dialog.remove();
        if (restore && restore.isConnected && typeof restore.focus === "function") restore.focus();
      },
      setStatus: function (message, isError) { setStatus(dialog, message, isError); },
    };
    close.addEventListener("click", api.close);
    document.addEventListener("keydown", onKey, true);
    document.addEventListener("mousedown", onDown, true);
    const host = document.getElementById("sheet-modal-container") || document.body;
    host.appendChild(dialog);
    return api;
  }

  function localNote() {
    const note = document.createElement("p");
    note.className = "ss-structured-local-note";
    note.textContent = t("sheet-structured-local-note", "Local only — kept in this browser, not saved to the workbook.");
    note.setAttribute("data-i18n", "sheet-structured-local-note");
    return note;
  }

  function notify(message, isError) {
    if (window.sheetNotify) window.sheetNotify(message, Boolean(isError));
  }

  const api = {
    t: t,
    state: state,
    save: save,
    mutate: mutate,
    parseRef: parseRef,
    parseRange: parseRange,
    colName: colName,
    refOf: refOf,
    cellData: cellData,
    cellValue: cellValue,
    numeric: numeric,
    activeGrid: activeGrid,
    writeValue: writeValue,
    writeValues: writeValues,
    escapeHtml: escapeHtml,
    createDialog: createDialog,
    setStatus: setStatus,
    localNote: localNote,
    notify: notify,
  };
  window.SheetStructured = api;
})();
