"use strict";
/* sheet constants, reference helpers and sidebar */

(function () {
  var D = window.SheetDefaults = window.SheetDefaults || {};

  const SIDEBAR_TAB_KEY = "sheet_sidebar_tab";
  const DEFAULT_TOTAL_ROWS = 1048576;
  const DEFAULT_TOTAL_COLS = 16384;
  const CELL_COLS = DEFAULT_TOTAL_COLS;
  const COL_WIDTH = 96;
  const ROW_HEIGHT = 24;
  const HEADER_WIDTH = 48;
  const OVERSCAN = 5;
  const MAX_VISIBLE_ROWS = 60;
  D.WORKSHEET_INDEX = 0;

  // --- Per-window sheet instances (#2270) ---------------------------------
  // Two Sheets windows must keep independent grid/workbook state. Each window
  // body (id window-body-*) owns one instance registered here. The "active"
  // instance follows focus/click so global handlers and existing modules that
  // still read window.SheetVirtualGrid operate on the window the user touched.
  window.__SHEET_INSTANCES__ = window.__SHEET_INSTANCES__ || {};

  function getSheetWindowId(el) {
    if (!el) return null;
    if (typeof el === "string") return el;
    var node = el.nodeType ? el : (el.target || el.currentTarget || null);
    if (!node || !node.closest) return null;
    var body = node.closest('[id^="window-body-"]');
    if (!body) return null;
    return body.dataset.windowId || body.id.replace("window-body-", "") || null;
  }

  function getSheetInstance(winIdOrEl) {
    var reg = window.__SHEET_INSTANCES__ || {};
    if (winIdOrEl && typeof winIdOrEl === "object" && !winIdOrEl.nodeType && !winIdOrEl.target) {
      // already an instance-like object
      if (winIdOrEl.windowId && reg[winIdOrEl.windowId]) return reg[winIdOrEl.windowId];
    }
    var winId = getSheetWindowId(winIdOrEl);
    if (!winId && typeof winIdOrEl === "string") winId = winIdOrEl;
    if (!winId) return null;
    return reg[winId] || null;
  }

  function getActiveSheetInstance() {
    var reg = window.__SHEET_INSTANCES__ || {};
    var active = window.__SHEET_ACTIVE_WINDOW_ID__;
    if (active && reg[active]) return reg[active];
    // fallback: any registered instance whose host is connected
    var keys = Object.keys(reg);
    for (var i = 0; i < keys.length; i++) {
      var inst = reg[keys[i]];
      if (inst && inst.body && inst.body.isConnected) {
        window.__SHEET_ACTIVE_WINDOW_ID__ = inst.windowId;
        return inst;
      }
    }
    return null;
  }

  function setActiveSheetInstance(winIdOrEl) {
    var inst = getSheetInstance(winIdOrEl);
    if (!inst) {
      var winId = getSheetWindowId(winIdOrEl);
      if (!winId && typeof winIdOrEl === "string") winId = winIdOrEl;
      if (winId && window.__SHEET_INSTANCES__[winId]) inst = window.__SHEET_INSTANCES__[winId];
    }
    if (inst) {
      window.__SHEET_ACTIVE_WINDOW_ID__ = inst.windowId;
      window.__LOADED_SHEET = inst.loadedSheet || null;
      window.__SHEET_INITIAL_ID = inst.sheetId || null;
      if (inst.grid) window.SheetVirtualGrid = inst.grid;
      D.WORKSHEET_INDEX = typeof inst.worksheetIndex === "number" ? inst.worksheetIndex : 0;
    }
    return inst || null;
  }

  function ensureSheetInstance(winIdOrEl) {
    var winId = getSheetWindowId(winIdOrEl);
    if (!winId && typeof winIdOrEl === "string") winId = winIdOrEl;
    if (!winId) return null;
    window.__SHEET_INSTANCES__ = window.__SHEET_INSTANCES__ || {};
    if (!window.__SHEET_INSTANCES__[winId]) {
      var body = document.getElementById("window-body-" + winId);
      window.__SHEET_INSTANCES__[winId] = {
        windowId: winId,
        body: body,
        grid: null,
        loadedSheet: null,
        sheetId: null,
        worksheetIndex: 0,
      };
    }
    var inst = window.__SHEET_INSTANCES__[winId];
    if (!inst.body || !inst.body.isConnected) {
      inst.body = document.getElementById("window-body-" + winId);
    }
    return inst;
  }

  function qInSheet(winIdOrEl, selector) {
    var inst = getSheetInstance(winIdOrEl);
    var body = inst ? inst.body : null;
    if (!body || !body.isConnected) {
      var winId = getSheetWindowId(winIdOrEl);
      if (winId) body = document.getElementById("window-body-" + winId);
    }
    if (body) return body.querySelector(selector);
    return null;
  }

  function currentSheetId(context) {
    var inst = context ? getSheetInstance(context) : getActiveSheetInstance();
    if (inst) {
      if (inst.loadedSheet && inst.loadedSheet.id) return inst.loadedSheet.id;
      if (inst.sheetId) return inst.sheetId;
    }
    // legacy globals fallback
    if (window.__LOADED_SHEET && window.__LOADED_SHEET.id) return window.__LOADED_SHEET.id;
    if (window.__SHEET_INITIAL_ID) return window.__SHEET_INITIAL_ID;
    var host = qInSheet(inst || getActiveSheetInstance(), '#sheet-content') ||
      document.querySelector('[id^="window-body-sheet"] #sheet-content');
    if (host && host.dataset.sheetId) return host.dataset.sheetId;
    return "current";
  }

  D.getSheetWindowId = getSheetWindowId;
  D.getSheetInstance = getSheetInstance;
  D.getActiveSheetInstance = getActiveSheetInstance;
  D.setActiveSheetInstance = setActiveSheetInstance;
  D.ensureSheetInstance = ensureSheetInstance;
  D.qInSheet = qInSheet;


  function $$(s, r) { return Array.from((r || document).querySelectorAll(s)); }

  function colName(idx) {
    let n = idx + 1, s = "";
    while (n > 0) {
      const r = (n - 1) % 26;
      s = String.fromCharCode(65 + r) + s;
      n = Math.floor((n - 1) / 26);
    }
    return s;
  }

  function colIdx(name) {
    let n = 0;
    for (let i = 0; i < name.length; i++) n = n * 26 + (name.charCodeAt(i) - 64);
    return n - 1;
  }

  // Cumulative row geometry, delegated to the widths module when loaded so
  // resized rows shift every following row (never `row * ROW_HEIGHT`).
  function rTop(r) {
    if (window.SheetCore && window.SheetCore.rowTop) return window.SheetCore.rowTop(r);
    return r * ROW_HEIGHT;
  }
  function rH(r) {
    if (window.SheetCore && window.SheetCore.rowHeight) return window.SheetCore.rowHeight(r);
    return ROW_HEIGHT;
  }
  function rowIdx(y) {
    if (window.SheetCore && window.SheetCore.rowAt) return window.SheetCore.rowAt(y);
    return Math.max(0, Math.floor(y / ROW_HEIGHT));
  }

  function parseCellRef(ref) {
    const m = String(ref).match(/^([A-Z]+)(\d+)$/);
    if (!m) return null;
    return { col: colIdx(m[1]), row: parseInt(m[2], 10) - 1 };
  }

  document.addEventListener("click", function (e) {
    const tab = e.target.closest("[data-sidebar-tab]");
    if (tab) {
      var scope = tab.closest('[id^="window-body-"]') || document;
      const which = tab.dataset.sidebarTab;
      Array.from(scope.querySelectorAll(".sidebar-tab")).forEach(function (b) {
        b.classList.toggle("active", b === tab);
        b.style.background = b === tab ? "var(--sheet-bg-elevated, #1e293b)" : "var(--sheet-bg, #0f172a)";
        b.style.color = b === tab ? "var(--sheet-fg, #f8fafc)" : "var(--sheet-muted, #94a3b8)";
      });
      Array.from(scope.querySelectorAll(".sidebar-content")).forEach(function (c) {
        c.style.display = c.dataset.sidebarContent === which ? "flex" : "none";
      });
      try { sessionStorage.setItem(SIDEBAR_TAB_KEY, which); } catch (_) {}
    }
  });

  function initSidebar(context) {
    let saved = null;
    try { saved = sessionStorage.getItem(SIDEBAR_TAB_KEY); } catch (_) {}
    var scope = context ? (context.closest ? context.closest('[id^="window-body-"]') : context) : document;
    if (saved && scope) {
      const btn = scope.querySelector('[data-sidebar-tab="' + saved + '"]');
      if (btn) btn.click();
    }
  }

  D.DEFAULT_TOTAL_ROWS = DEFAULT_TOTAL_ROWS;
  D.DEFAULT_TOTAL_COLS = DEFAULT_TOTAL_COLS;
  D.CELL_COLS = CELL_COLS;
  D.COL_WIDTH = COL_WIDTH;
  D.ROW_HEIGHT = ROW_HEIGHT;
  D.HEADER_WIDTH = HEADER_WIDTH;
  D.OVERSCAN = OVERSCAN;
  D.MAX_VISIBLE_ROWS = MAX_VISIBLE_ROWS;
  D.currentSheetId = currentSheetId;
  D.colName = colName;
  D.colIdx = colIdx;
  D.rTop = rTop;
  D.rH = rH;
  D.rowIdx = rowIdx;
  D.parseCellRef = parseCellRef;
  D.initSidebar = initSidebar;

  // Active-grid compatibility shim (#2270): modules read window.SheetVirtualGrid.
  try {
    Object.defineProperty(window, "SheetVirtualGrid", {
      configurable: true,
      get: function () {
        var a = getActiveSheetInstance();
        return (a && a.grid) || D.virtualGrid || null;
      },
      set: function (v) {
        var a = getActiveSheetInstance();
        if (a && v) a.grid = v;
        D.virtualGrid = v;
      },
    });
  } catch (_) {
    window.SheetVirtualGrid = D.virtualGrid;
  }
})();
