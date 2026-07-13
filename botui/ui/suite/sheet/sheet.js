"use strict";
/* sheet shell — virtual scrolling for 100k+ rows, all data via Rust endpoints */

(function () {
  const SIDEBAR_TAB_KEY = "sheet_sidebar_tab";
  const DEFAULT_TOTAL_ROWS = 1200000;
  const DEFAULT_TOTAL_COLS = 26;
  const CELL_COLS = DEFAULT_TOTAL_COLS;
  const COL_WIDTH = 96;
  const ROW_HEIGHT = 24;
  const HEADER_WIDTH = 48;
  const OVERSCAN = 5;
  const MAX_VISIBLE_ROWS = 60;
  let WORKSHEET_INDEX = 0;
  function currentSheetId() { return window.__SHEET_INITIAL_ID || "current"; }

  function $(s, r) { return (r || document).querySelector(s); }
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

  const VirtualGrid = {
    totalRows: DEFAULT_TOTAL_ROWS,
    totalCols: DEFAULT_TOTAL_COLS,
    scrollTop: 0,
    scrollLeft: 0,
    viewportHeight: 0,
    viewportWidth: 0,
    cells: new Map(),
    pool: [],
    usedCount: 0,
    root: null,
    headerRow: null,
    headerCol: null,
    body: null,
    requestSeq: 0,
    lastRenderedRange: null,

    init: function (host) {
      this.root = document.createElement("div");
      this.root.className = "virtual-grid";
      this.root.style.cssText = "position:relative;display:flex;flex-direction:column;flex:1;background:#0f172a;overflow:hidden;";

      this.headerRow = document.createElement("div");
      this.headerRow.className = "vg-header-row";
      this.headerRow.style.cssText = "display:flex;height:24px;background:#0f172a;border-bottom:1px solid #334155;overflow:hidden;flex-shrink:0;";

      this.scrollArea = document.createElement("div");
      this.scrollArea.className = "vg-scroll";
      this.scrollArea.style.cssText = "flex:1;overflow:auto;background:#0f172a;position:relative;";

      this.headerCol = document.createElement("div");
      this.headerCol.className = "vg-header-col";
      this.headerCol.style.cssText = "position:absolute;top:0;left:0;width:" + HEADER_WIDTH + "px;background:#0f172a;border-right:1px solid #334155;z-index:3;";

      this.body = document.createElement("div");
      this.body.className = "vg-body";
      this.body.style.cssText = "position:relative;";

      const inner = document.createElement("div");
      inner.style.cssText = "position:relative;height:" + (this.totalRows * ROW_HEIGHT) + "px;width:" + (HEADER_WIDTH + this.totalCols * COL_WIDTH) + "px;";
      this.body.appendChild(inner);
      this.bodyInner = inner;

      this.scrollArea.appendChild(this.headerCol);
      this.scrollArea.appendChild(this.body);
      this.root.appendChild(this.headerRow);
      this.root.appendChild(this.scrollArea);

      this.scrollArea.addEventListener("scroll", this.onScroll.bind(this), { passive: true });
      host.appendChild(this.root);
      this.updateViewport();
      this.renderHeaders();
      this.requestRange();
    },

    updateViewport: function () {
      this.viewportHeight = this.scrollArea.clientHeight;
      this.viewportWidth = this.scrollArea.clientWidth;
    },

    onScroll: function () {
      this.scrollTop = this.scrollArea.scrollTop;
      this.scrollLeft = this.scrollArea.scrollLeft;
      this.headerCol.style.transform = "translateY(" + this.scrollTop + "px)";
      this.headerRow.style.transform = "translateX(" + this.scrollLeft + "px)";
      this.requestRange();
    },

    renderHeaders: function () {
      this.headerRow.innerHTML = "";
      const corner = document.createElement("div");
      corner.style.cssText = "width:" + HEADER_WIDTH + "px;background:#0f172a;border-right:1px solid #334155;flex-shrink:0;";
      this.headerRow.appendChild(corner);
      for (let c = 0; c < this.totalCols; c++) {
        const h = document.createElement("div");
        h.textContent = colName(c);
        h.style.cssText = "width:" + COL_WIDTH + "px;background:#0f172a;color:#94a3b8;text-align:center;line-height:24px;font-size:11px;border-right:1px solid #334155;flex-shrink:0;";
        this.headerRow.appendChild(h);
      }
      // Otimização: A barra lateral de cabeçalhos de linhas é agora virtualizada
      this.headerCol.innerHTML = "";
      this.headerColPool = [];
    },

    getOrCreateHeaderColNode: function (idx) {
      let node;
      if (idx < this.headerColPool.length) {
        node = this.headerColPool[idx];
      } else {
        node = document.createElement("div");
        node.style.cssText = "position:absolute;width:100%;height:" + ROW_HEIGHT + "px;background:#0f172a;color:#94a3b8;text-align:center;line-height:" + ROW_HEIGHT + "px;font-size:11px;border-bottom:1px solid #334155;box-sizing:border-box;";
        this.headerCol.appendChild(node);
        this.headerColPool.push(node);
      }
      return node;
    },

    visibleRowRange: function () {
      const start = Math.max(0, Math.floor(this.scrollTop / ROW_HEIGHT) - OVERSCAN);
      const visible = Math.ceil(this.viewportHeight / ROW_HEIGHT) + OVERSCAN * 2;
      const end = Math.min(this.totalRows, start + visible);
      return { start: start, end: end };
    },

    requestRange: function () {
      if (!this.viewportHeight) return;
      const range = this.visibleRowRange();
      const rangeKey = range.start + ":" + range.end;
      if (this.lastRenderedRange === rangeKey) return;
      this.lastRenderedRange = rangeKey;

      const seq = ++this.requestSeq;
      SheetAPI.getRange(range.start, 0, range.end - 1, this.totalCols - 1)
        .then(function (data) {
          if (seq !== VirtualGrid.requestSeq) return;
          VirtualGrid.cells = new Map(Object.entries(data.cells || {}));
          if (data.total_rows) VirtualGrid.totalRows = Math.max(VirtualGrid.totalRows, data.total_rows);
          VirtualGrid.render();
        })
        .catch(function () {});
    },

    render: function () {
      this.usedCount = 0;
      const range = this.visibleRowRange();

      // Renderização virtualizada dos cabeçalhos de linhas
      if (!this.headerColPool) this.headerColPool = [];
      let headerUsed = 0;
      for (let r = range.start; r < range.end; r++) {
        const node = this.getOrCreateHeaderColNode(headerUsed++);
        node.textContent = r + 1;
        node.style.display = "block";
        node.style.top = (r * ROW_HEIGHT) + "px";
        node.dataset.row = r;
      }
      for (let i = headerUsed; i < this.headerColPool.length; i++) {
        this.headerColPool[i].style.display = "none";
      }

      // Renderização virtualizada das células
      for (let r = range.start; r < range.end; r++) {
        this.renderRow(r);
      }
      for (let i = this.usedCount; i < this.pool.length; i++) {
        this.pool[i].style.display = "none";
      }
    },

    recycleAll: function () {
      // no-op: o reaproveitamento é feito in-place usando this.pool e this.usedCount
    },

    getOrCreateNode: function () {
      let node;
      if (this.usedCount < this.pool.length) {
        node = this.pool[this.usedCount];
      } else {
        node = document.createElement("div");
        node.className = "vg-cell";
        node.contentEditable = "true";
        node.style.cssText = "position:absolute;background:#0f172a;color:#f8fafc;border-right:1px solid #334155;border-bottom:1px solid #334155;padding:2px 4px;font-size:12px;overflow:hidden;outline:none;box-sizing:border-box;";
        node.addEventListener("focus", this.onCellFocus);
        node.addEventListener("blur", this.onCellBlur);
        this.bodyInner.appendChild(node);
        this.pool.push(node);
      }
      this.usedCount++;
      return node;
    },

    onCellFocus: function (e) {
      e.target.style.outline = "2px solid #3b82f6";
      e.target.style.zIndex = "5";
      const ref = e.target.dataset.ref || "";
      
      e.target.dataset.origVal = e.target.textContent || "";
      if (e.target.dataset.formula) {
        e.target.textContent = e.target.dataset.formula;
      }
      
      if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
        const p = parseCellRef(ref);
        if (p) window.GBCollab.sendCursor(p.row * VirtualGrid.totalCols + p.col);
      }
    },

    onCellBlur: function (e) {
      e.target.style.outline = "none";
      e.target.style.zIndex = "1";
      const ref = e.target.dataset.ref;
      const val = e.target.textContent || "";
      const origVal = e.target.dataset.origVal || "";
      
      if (val === origVal) {
        if (e.target.dataset.formula) {
          const key = e.target.dataset.row + "," + e.target.dataset.col;
          const cellData = VirtualGrid.cells.get(key);
          e.target.textContent = cellData ? (cellData.value || "") : "";
        }
        return;
      }

      SheetAPI.updateCell(ref, val).then(function (res) {
        if (res && res.success) {
          VirtualGrid.lastRenderedRange = null;
          VirtualGrid.requestRange();
        }
      });

      if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
        const p = parseCellRef(ref);
        if (p) {
          window.GBCollab.sendEdit({ position: p.row * VirtualGrid.totalCols + p.col, content: val, length: val.length });
          window.GBCollab.sendTypingStop();
        }
      }
    },

    markCellComputed: function (ref, result) {
      const el = this.bodyInner.querySelector('[data-ref="' + ref + '"]');
      if (el) el.dataset.formulaResult = result;
    },

    renderRow: function (row) {
      for (let c = 0; c < this.totalCols; c++) {
        const ref = colName(c) + (row + 1);
        const key = row + "," + c;
        const cellData = this.cells.get(key);
        const value = cellData ? (cellData.value || "") : "";
        const formula = cellData ? (cellData.formula || "") : "";
        const node = this.getOrCreateNode();
        
        node.style.display = "block";
        node.dataset.ref = ref;
        node.dataset.row = row;
        node.dataset.col = c;
        node.dataset.formula = formula;
        node.textContent = value;
        node.style.left = (HEADER_WIDTH + c * COL_WIDTH) + "px";
        node.style.top = (row * ROW_HEIGHT) + "px";
        node.style.width = COL_WIDTH + "px";
        node.style.height = ROW_HEIGHT + "px";
      }
    }
  };

  const SheetAPI = {
    updateCell: function (ref, value) {
      const p = parseCellRef(ref);
      if (!p) return Promise.resolve(null);
      return fetch("/api/sheet/cell", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          sheet_id: currentSheetId(),
          worksheet_index: WORKSHEET_INDEX,
          row: p.row,
          col: p.col,
          value: value
        })
      }).then(function (r) { return r.json(); }).catch(function () { return null; });
    },

    evaluateFormula: function (formula) {
      return fetch("/api/sheet/formula", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          sheet_id: currentSheetId(),
          worksheet_index: WORKSHEET_INDEX,
          formula: formula
        })
      })
        .then(function (r) { return r.json(); })
        .then(function (j) { return j && j.value !== undefined ? j.value : (j && j.error ? "#ERR: " + j.error : ""); })
        .catch(function () { return "#ERR: network"; });
    },

    getRange: function (startRow, startCol, endRow, endCol) {
      return fetch("/api/sheet/range", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          sheet_id: currentSheetId(),
          worksheet_index: WORKSHEET_INDEX,
          start_row: startRow,
          start_col: startCol,
          end_row: endRow,
          end_col: endCol
        })
      }).then(function (r) { return r.json(); }).catch(function () { return { cells: {} }; });
    },

    list: function () {
      return fetch("/api/sheet/list", { method: "GET" })
        .then(function (r) { return r.json(); }).catch(function () { return []; });
    },

    load: function (id) {
      const url = "/api/sheet/load" + (id ? "?id=" + encodeURIComponent(id) : "");
      return fetch(url, { method: "GET" })
        .then(function (r) { return r.json(); }).catch(function () { return null; });
    },

    addWorksheet: function () {
      return fetch("/api/sheet/worksheets/add", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: currentSheetId() })
      }).then(function (r) { return r.text(); });
    },

    switchWorksheet: function (index) {
      WORKSHEET_INDEX = index;
      return fetch("/api/sheet/worksheets/switch", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: currentSheetId(), index: index })
      }).then(function (r) { return r.text(); });
    },

    deleteWorksheet: function (index) {
      return fetch("/api/sheet/worksheets/delete", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: currentSheetId(), index: index })
      }).then(function (r) { return r.text(); });
    },

    renameWorksheet: function (index, name) {
      return fetch("/api/sheet/worksheets/rename", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: currentSheetId(), index: index, name: name })
      }).then(function (r) { return r.json(); });
    }
  };

  window.addEventListener("gb-sheet-tab", function (e) {
    const idx = e && e.detail && typeof e.detail.index === "number" ? e.detail.index : 0;
    WORKSHEET_INDEX = idx;
    try { sessionStorage.setItem("sheet_active_index", String(idx)); } catch (_) {}
  });

  try {
    const saved = parseInt(sessionStorage.getItem("sheet_active_index") || "0", 10);
    if (!isNaN(saved) && saved >= 0) WORKSHEET_INDEX = saved;
  } catch (_) {}

  document.addEventListener("keydown", function (e) {
    if (!(e.target && e.target.classList && e.target.classList.contains("vg-cell"))) return;
    const row = parseInt(e.target.dataset.row, 10);
    const col = parseInt(e.target.dataset.col, 10);
    if (isNaN(row) || isNaN(col)) return;

    if (e.key === "Enter") {
      e.preventDefault();
      const next = VirtualGrid.bodyInner.querySelector('[data-row="' + (row + 1) + '"][data-col="' + col + '"]');
      if (next) {
        next.focus();
      } else {
        // Se a célula de baixo não está visível no DOM, realiza a rolagem vertical inteligente
        VirtualGrid.scrollArea.scrollTop += ROW_HEIGHT;
        setTimeout(function () {
          const retry = VirtualGrid.bodyInner.querySelector('[data-row="' + (row + 1) + '"][data-col="' + col + '"]');
          if (retry) retry.focus();
        }, 50);
      }
    } else if (e.key === "Tab") {
      e.preventDefault();
      const next = VirtualGrid.bodyInner.querySelector('[data-row="' + row + '"][data-col="' + (col + 1) + '"]');
      if (next) {
        next.focus();
      } else {
        // Se a célula da direita não está visível no DOM, realiza a rolagem horizontal inteligente
        VirtualGrid.scrollArea.scrollLeft += COL_WIDTH;
        setTimeout(function () {
          const retry = VirtualGrid.bodyInner.querySelector('[data-row="' + row + '"][data-col="' + (col + 1) + '"]');
          if (retry) retry.focus();
        }, 50);
      }
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      const next = VirtualGrid.bodyInner.querySelector('[data-row="' + (row + 1) + '"][data-col="' + col + '"]');
      if (next) {
        next.focus();
      } else {
        VirtualGrid.scrollArea.scrollTop += ROW_HEIGHT;
        setTimeout(function () {
          const retry = VirtualGrid.bodyInner.querySelector('[data-row="' + (row + 1) + '"][data-col="' + col + '"]');
          if (retry) retry.focus();
        }, 50);
      }
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      const next = VirtualGrid.bodyInner.querySelector('[data-row="' + (row - 1) + '"][data-col="' + col + '"]');
      if (next) {
        next.focus();
      } else {
        if (row > 0) {
          VirtualGrid.scrollArea.scrollTop -= ROW_HEIGHT;
          setTimeout(function () {
            const retry = VirtualGrid.bodyInner.querySelector('[data-row="' + (row - 1) + '"][data-col="' + col + '"]');
            if (retry) retry.focus();
          }, 50);
        }
      }
    } else if (e.key === "ArrowRight") {
      const sel = window.getSelection();
      const isAtEnd = !e.target.textContent || (sel.rangeCount > 0 && sel.getRangeAt(0).startOffset === e.target.textContent.length);
      if (isAtEnd) {
        e.preventDefault();
        const next = VirtualGrid.bodyInner.querySelector('[data-row="' + row + '"][data-col="' + (col + 1) + '"]');
        if (next) {
          next.focus();
        } else {
          VirtualGrid.scrollArea.scrollLeft += COL_WIDTH;
          setTimeout(function () {
            const retry = VirtualGrid.bodyInner.querySelector('[data-row="' + row + '"][data-col="' + (col + 1) + '"]');
            if (retry) retry.focus();
          }, 50);
        }
      }
    } else if (e.key === "ArrowLeft") {
      const sel = window.getSelection();
      const isAtStart = !e.target.textContent || (sel.rangeCount > 0 && sel.getRangeAt(0).startOffset === 0);
      if (isAtStart) {
        e.preventDefault();
        const next = VirtualGrid.bodyInner.querySelector('[data-row="' + row + '"][data-col="' + (col - 1) + '"]');
        if (next) {
          next.focus();
        } else {
          if (col > 0) {
            VirtualGrid.scrollArea.scrollLeft -= COL_WIDTH;
            setTimeout(function () {
              const retry = VirtualGrid.bodyInner.querySelector('[data-row="' + row + '"][data-col="' + (col - 1) + '"]');
              if (retry) retry.focus();
            }, 50);
          }
        }
      }
    } else if (e.target.textContent && e.target.textContent.startsWith("=")) {
      if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
        const ref = e.target.dataset.ref || "";
        const p = parseCellRef(ref);
        if (p) window.GBCollab.debouncedTypingStart(p.row * VirtualGrid.totalCols + p.col);
      }
    }
  });

  window.addEventListener("resize", function () {
    if (VirtualGrid.root) {
      VirtualGrid.updateViewport();
      VirtualGrid.lastRenderedRange = null;
      VirtualGrid.requestRange();
    }
  });

  function initCollab() {
    if (!window.GBCollab) return;
    const connStatus = document.getElementById("gb-conn-status");
    window.GBCollab.connect({
      app: "sheet",
      docId: currentSheetId(),
      collaboratorsEl: document.getElementById("collaborators"),
      onConnect: function () {
        if (connStatus) { connStatus.className = "gb-connection-status online"; connStatus.style.display = "inline-flex"; connStatus.querySelector(".label").textContent = "online"; }
      },
      onDisconnect: function () {
        if (connStatus) { connStatus.className = "gb-connection-status offline"; connStatus.style.display = "inline-flex"; connStatus.querySelector(".label").textContent = "offline"; }
      },
      onEdit: function (msg) {
        if (!msg || !msg.position) return;
        const row = Math.floor(msg.position / VirtualGrid.totalCols);
        const col = msg.position % VirtualGrid.totalCols;
        const key = row + "," + col;
        const existing = VirtualGrid.cells.get(key) || {};
        VirtualGrid.cells.set(key, Object.assign({}, existing, { value: msg.content }));
        if (VirtualGrid.lastRenderedRange) {
          const parts = VirtualGrid.lastRenderedRange.split(":");
          const start = parseInt(parts[0], 10), end = parseInt(parts[1], 10);
          if (row >= start && row < end) {
            const el = VirtualGrid.bodyInner.querySelector('[data-row="' + row + '"][data-col="' + col + '"]');
            if (el) el.textContent = msg.content || "";
          }
        }
      }
    });
  }

  function initAuth() {
    if (window.GBAuthGuard) GBAuthGuard.injectLoginButton(document.getElementById("gb-auth-button"));
  }

  window.addEventListener("DOMContentLoaded", function () {
    initSidebar();
    initAuth();
    initCollab();
    // Wait for boot script to load xlsx from Drive before init'ing
    var bootPromise = window.__SHEET_BOOT || Promise.resolve();
    bootPromise.then(function () {
      var host = document.getElementById("sheet-content");
      if (host) {
        VirtualGrid.init(host);
        if (window.SheetAdvanced && window.SheetAdvanced.init) {
          var adv = window.SheetAdvanced.init(host, { sheetId: (host.dataset.sheetId || "current") });
          if (adv) window.SheetAdvancedInstance = adv;
        }
      }
      window.SheetAPI = SheetAPI;
      window.SheetVirtualGrid = VirtualGrid;
    });
  });
})();
