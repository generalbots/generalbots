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

  function currentSheetId() { return window.__SHEET_INITIAL_ID || "current"; }

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
      const which = tab.dataset.sidebarTab;
      $$(".sidebar-tab").forEach(function (b) {
        b.classList.toggle("active", b === tab);
        b.style.background = b === tab ? "#1e293b" : "#0f172a";
        b.style.color = b === tab ? "#f8fafc" : "#94a3b8";
      });
      $$(".sidebar-content").forEach(function (c) {
        c.style.display = c.dataset.sidebarContent === which ? "flex" : "none";
      });
      try { sessionStorage.setItem(SIDEBAR_TAB_KEY, which); } catch (_) {}
    }
  });

  function initSidebar() {
    let saved = null;
    try { saved = sessionStorage.getItem(SIDEBAR_TAB_KEY); } catch (_) {}
    if (saved) {
      const btn = document.querySelector('[data-sidebar-tab="' + saved + '"]');
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
})();
