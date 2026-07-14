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

  function updateFormulaBar() {
    const addrEl = document.getElementById("cellAddress");
    const formulaEl = document.getElementById("formulaInput");
    if (!addrEl) return;
    if (VirtualGrid.selectedRow == null || VirtualGrid.selectedCol == null) {
      addrEl.value = "";
      if (formulaEl) formulaEl.value = "";
      return;
    }
    const ref = colName(VirtualGrid.selectedCol) + (VirtualGrid.selectedRow + 1);
    addrEl.value = ref;
    if (formulaEl) {
      const key = VirtualGrid.selectedRow + "," + VirtualGrid.selectedCol;
      const cellData = VirtualGrid.cells.get(key);
      const formula = cellData && cellData.formula ? cellData.formula : "";
      const val = cellData && cellData.value ? cellData.value : "";
      formulaEl.value = formula ? formula : val;
    }
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

  function clearSelectionOverlay() {
    if (VirtualGrid.selectionOverlay) {
      VirtualGrid.selectionOverlay.style.display = "none";
    }
  }

  function positionSelectionOverlay(row, col) {
    if (!VirtualGrid.selectionOverlay) return;
    VirtualGrid.selectionOverlay.style.display = "block";
    VirtualGrid.selectionOverlay.style.left = (HEADER_WIDTH + col * COL_WIDTH - 1) + "px";
    VirtualGrid.selectionOverlay.style.top = (row * ROW_HEIGHT - 1) + "px";
    VirtualGrid.selectionOverlay.style.width = (COL_WIDTH + 2) + "px";
    VirtualGrid.selectionOverlay.style.height = (ROW_HEIGHT + 2) + "px";
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
    selectedRow: 0,
    selectedCol: 0,
    editingCell: null,
    selectionOverlay: null,

    init: function (host) {
      this.root = document.createElement("div");
      this.root.className = "virtual-grid";
      this.root.style.cssText = "position:relative;display:flex;flex-direction:column;flex:1;background:#0f172a;overflow:hidden;";

      this.headerRow = document.createElement("div");
      this.headerRow.className = "vg-header-row";
      this.headerRow.style.cssText = "display:flex;height:24px;background:#0f172a;border-bottom:1px solid #334155;overflow:hidden;flex-shrink:0;";

      this.midRow = document.createElement("div");
      this.midRow.style.cssText = "display:flex;flex:1;overflow:hidden;";

      this.scrollArea = document.createElement("div");
      this.scrollArea.className = "vg-scroll";
      this.scrollArea.style.cssText = "flex:1;overflow:auto;background:#0f172a;position:relative;";

      this.body = document.createElement("div");
      this.body.className = "vg-body";
      this.body.style.cssText = "position:relative;";

      const inner = document.createElement("div");
      inner.style.cssText = "position:relative;height:" + (this.totalRows * ROW_HEIGHT) + "px;width:" + (HEADER_WIDTH + this.totalCols * COL_WIDTH) + "px;";
      this.body.appendChild(inner);
      this.bodyInner = inner;

      // Row number pool (appended to bodyInner, left:0, scrolls naturally with cells)
      this.headerColPool = [];

      this.midRow.appendChild(this.scrollArea);
      this.scrollArea.appendChild(this.body);
      this.root.appendChild(this.headerRow);
      this.root.appendChild(this.midRow);

      // Selection overlay inside bodyInner so it scrolls naturally
      this.selectionOverlay = document.createElement("div");
      this.selectionOverlay.className = "vg-selection";
      this.selectionOverlay.style.cssText = "position:absolute;border:2px solid #3b82f6;background:rgba(59,130,246,0.08);pointer-events:none;z-index:10;display:none;";
      this.bodyInner.appendChild(this.selectionOverlay);

      this.scrollArea.addEventListener("scroll", this.onScroll.bind(this), { passive: true });
      host.appendChild(this.root);
      this.updateViewport();
      this.renderHeaders();
      this.requestRange();
      // Select first cell by default
      var self = this;
      setTimeout(function () {
        self.selectCell(0, 0);
      }, 100);
    },

    updateViewport: function () {
      this.viewportHeight = this.scrollArea.clientHeight;
      this.viewportWidth = this.scrollArea.clientWidth;
    },

    onScroll: function () {
      this.scrollTop = this.scrollArea.scrollTop;
      this.scrollLeft = this.scrollArea.scrollLeft;
      this.headerRow.style.transform = "translateX(" + (-this.scrollLeft) + "px)";
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
      this.headerColPool = [];
    },

    getOrCreateHeaderColNode: function (idx) {
      let node;
      if (idx < this.headerColPool.length) {
        node = this.headerColPool[idx];
      } else {
        node = document.createElement("div");
        node.style.cssText = "position:absolute;width:" + HEADER_WIDTH + "px;height:" + ROW_HEIGHT + "px;background:#0f172a;color:#94a3b8;text-align:center;line-height:" + ROW_HEIGHT + "px;font-size:11px;border-bottom:1px solid #334155;border-right:1px solid #334155;box-sizing:border-box;z-index:5;";
        this.bodyInner.appendChild(node);
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

      // Render row numbers IMMEDIATELY — don't wait for API
      this.renderRowHeaders(range);

      const seq = ++this.requestSeq;
      SheetAPI.getRange(range.start, 0, range.end - 1, this.totalCols - 1)
        .then(function (data) {
          if (seq !== VirtualGrid.requestSeq) return;
          // Merge API cells without overwriting pre-loaded data (e.g. from Drive)
          if (data.cells) {
            for (var k in data.cells) {
              if (!VirtualGrid.cells.has(k)) {
                VirtualGrid.cells.set(k, data.cells[k]);
              }
            }
          }
          if (data.total_rows) VirtualGrid.totalRows = Math.max(VirtualGrid.totalRows, data.total_rows);
          VirtualGrid.render();
        })
        .catch(function () {});
    },

    renderRowHeaders: function (range) {
      if (!this.headerColPool) this.headerColPool = [];
      let headerUsed = 0;
      for (let r = range.start; r < range.end; r++) {
        const node = this.getOrCreateHeaderColNode(headerUsed++);
        node.textContent = r + 1;
        node.style.display = "block";
        node.style.left = "0px";
        node.style.top = (r * ROW_HEIGHT) + "px";
        node.dataset.row = r;
      }
      for (let i = headerUsed; i < this.headerColPool.length; i++) {
        this.headerColPool[i].style.display = "none";
      }
    },

    render: function () {
      this.usedCount = 0;
      const range = this.visibleRowRange();

      this.renderRowHeaders(range);

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
        node.contentEditable = "false";
        node.tabIndex = -1;
        node.style.cssText = "position:absolute;background:#0f172a;color:#f8fafc;border-right:1px solid #334155;border-bottom:1px solid #334155;padding:2px 4px;font-size:12px;overflow:hidden;outline:none;box-sizing:border-box;cursor:pointer;";
        node.addEventListener("mousedown", this.onCellMouseDown.bind(this));
        node.addEventListener("dblclick", this.onCellDblClick.bind(this));
        node.addEventListener("blur", this.onCellBlur.bind(this));
        node.addEventListener("keydown", this.onCellKeyDown.bind(this));
        this.bodyInner.appendChild(node);
        this.pool.push(node);
      }
      this.usedCount++;
      return node;
    },

    onCellMouseDown: function (e) {
      if (e.button !== 0) return;
      var row = parseInt(e.target.dataset.row, 10);
      var col = parseInt(e.target.dataset.col, 10);
      if (!isNaN(row) && !isNaN(col)) {
        this.selectCell(row, col);
      }
    },

    selectCell: function (row, col) {
      this.selectedRow = row;
      this.selectedCol = col;
      positionSelectionOverlay(row, col);
      updateFormulaBar();
      this.ensureVisible(row, col);
      // Focus the cell element so keyboard events reach it
      var cellEl = this.bodyInner.querySelector('[data-row="' + row + '"][data-col="' + col + '"]');
      if (cellEl) cellEl.focus({preventScroll: true});
      if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
        window.GBCollab.sendCursor(row * this.totalCols + col);
      }
    },

    ensureVisible: function (row, col) {
      var viewTop = this.scrollTop;
      var viewBottom = viewTop + this.viewportHeight;
      var viewLeft = this.scrollLeft;
      var viewRight = viewLeft + this.viewportWidth;
      var cellTop = row * ROW_HEIGHT;
      var cellBottom = cellTop + ROW_HEIGHT;
      var cellLeft = HEADER_WIDTH + col * COL_WIDTH;
      var cellRight = cellLeft + COL_WIDTH;
      if (cellTop < viewTop) this.scrollArea.scrollTop = cellTop - 10;
      else if (cellBottom > viewBottom) this.scrollArea.scrollTop = cellBottom - this.viewportHeight + 10;
      if (cellLeft < viewLeft + HEADER_WIDTH) this.scrollArea.scrollLeft = cellLeft - HEADER_WIDTH - 10;
      else if (cellRight > viewRight) this.scrollArea.scrollLeft = cellRight - this.viewportWidth + 10;
    },

    onCellDblClick: function (e) {
      var cell = e.target;
      cell.contentEditable = "true";
      this.editingCell = cell;
      cell.dataset.origVal = cell.textContent || "";
      if (cell.dataset.formula) {
        cell.textContent = cell.dataset.formula;
      }
      // Focus and place cursor at end
      var range = document.createRange();
      var sel = window.getSelection();
      range.selectNodeContents(cell);
      range.collapse(false);
      sel.removeAllRanges();
      sel.addRange(range);
      cell.focus();
    },

    onCellBlur: function (e) {
      var cell = e.target;
      // Only process if this cell was being edited
      if (this.editingCell !== cell) return;
      this.editingCell = null;
      cell.contentEditable = "false";
      cell.style.outline = "none";
      cell.style.zIndex = "1";

      const ref = cell.dataset.ref;
      const val = cell.textContent || "";
      const origVal = cell.dataset.origVal || "";

      if (val === origVal) {
        if (cell.dataset.formula) {
          const key = cell.dataset.row + "," + cell.dataset.col;
          const cellData = VirtualGrid.cells.get(key);
          cell.textContent = cellData ? (cellData.value || "") : "";
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

    onCellKeyDown: function (e) {
      var cell = e.target;
      var row = parseInt(cell.dataset.row, 10);
      var col = parseInt(cell.dataset.col, 10);
      if (isNaN(row) || isNaN(col)) return;

      var isEditing = (this.editingCell === cell);

      if (e.key === "Enter") {
        e.preventDefault();
        if (isEditing) {
          // Finish editing, move down
          cell.blur();
          this.selectCell(row + 1, col);
          var next = this.bodyInner.querySelector('[data-row="' + (row + 1) + '"][data-col="' + col + '"]');
          if (!next && row + 1 < this.totalRows) {
            this.scrollArea.scrollTop += ROW_HEIGHT;
          }
        } else {
          // Enter edit mode on selected cell
          this.startEditingCell(cell);
        }
      } else if (e.key === "Tab") {
        e.preventDefault();
        if (isEditing) cell.blur();
        this.selectCell(row, col + 1);
      } else if (e.key === "ArrowDown" && !isEditing) {
        e.preventDefault();
        this.selectCell(row + 1, col);
      } else if (e.key === "ArrowUp" && !isEditing) {
        e.preventDefault();
        if (row > 0) this.selectCell(row - 1, col);
      } else if (e.key === "ArrowRight" && !isEditing) {
        e.preventDefault();
        this.selectCell(row, col + 1);
      } else if (e.key === "ArrowLeft" && !isEditing) {
        e.preventDefault();
        if (col > 0) this.selectCell(row, col - 1);
      } else if ((e.key === "F2") && !isEditing) {
        e.preventDefault();
        this.startEditingCell(cell);
      } else if (isEditing) {
        if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
          var ref = cell.dataset.ref || "";
          var p = parseCellRef(ref);
          if (p) window.GBCollab.debouncedTypingStart(p.row * VirtualGrid.totalCols + p.col);
        }
      }
    },

    startEditingCell: function (cell) {
      if (!cell) return;
      cell.contentEditable = "true";
      this.editingCell = cell;
      cell.dataset.origVal = cell.textContent || "";
      if (cell.dataset.formula) {
        cell.textContent = cell.dataset.formula;
      }
      cell.focus();
      var range = document.createRange();
      var sel = window.getSelection();
      range.selectNodeContents(cell);
      range.collapse(false);
      sel.removeAllRanges();
      sel.addRange(range);
    },

    exitEditMode: function () {
      if (this.editingCell) {
        this.editingCell.blur();
      }
    },

    markCellComputed: function (ref, result) {
      const el = this.bodyInner.querySelector('[data-ref="' + ref + '"]');
      if (el) el.dataset.formulaResult = result;
    },

    applyCellStyle: function (node, cellData) {
      if (!cellData || !cellData.style) return;
      var s = cellData.style;
      if (s.font_weight) node.style.fontWeight = s.font_weight;
      if (s.font_style) node.style.fontStyle = s.font_style;
      if (s.text_decoration) node.style.textDecoration = s.text_decoration;
      if (s.font_family) node.style.fontFamily = s.font_family;
      if (s.font_size) node.style.fontSize = s.font_size + 'px';
      if (s.color) node.style.color = s.color;
      if (s.background) node.style.backgroundColor = s.background;
      if (s.text_align) node.style.textAlign = s.text_align;
      if (s.vertical_align) node.style.verticalAlign = s.vertical_align;
      if (s.border) node.style.border = s.border;
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
        // Reset style then apply cell-level style
        node.style.fontWeight = "";
        node.style.fontStyle = "";
        node.style.textDecoration = "";
        node.style.fontFamily = "";
        node.style.fontSize = "12px";
        node.style.color = "#f8fafc";
        node.style.backgroundColor = "#0f172a";
        this.applyCellStyle(node, cellData);
        if (this.editingCell !== node) {
          node.contentEditable = "false";
          node.style.outline = "none";
          node.style.zIndex = "1";
        }
      }
    },

    // Format the currently selected cell with given style properties
    formatSelectedCell: function (styleProps) {
      var row = this.selectedRow;
      var col = this.selectedCol;
      if (row == null || col == null) return Promise.resolve(null);
      var self = this;
      return fetch("/api/sheet/format", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          sheet_id: currentSheetId(),
          worksheet_index: WORKSHEET_INDEX,
          start_row: row,
          start_col: col,
          end_row: row,
          end_col: col,
          style: styleProps
        })
      }).then(function (r) { return r.json(); }).then(function () {
        // Update local cell data and re-render the cell
        var key = row + "," + col;
        var existing = self.cells.get(key) || {};
        var mergedStyle = Object.assign({}, existing.style || {}, styleProps);
        existing.style = mergedStyle;
        self.cells.set(key, existing);
        // Re-render just this cell
        var node = self.bodyInner.querySelector('[data-row="' + row + '"][data-col="' + col + '"]');
        if (node) self.applyCellStyle(node, existing);
      }).catch(function () { return null; });
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

  // Toolbar format button handlers
  function initToolbar() {
    var boldBtn = document.getElementById("boldBtn");
    if (boldBtn) {
      boldBtn.addEventListener("click", function (e) {
        e.preventDefault();
        VirtualGrid.formatSelectedCell({ font_weight: "bold" });
      });
    }
    var italicBtn = document.getElementById("italicBtn");
    if (italicBtn) {
      italicBtn.addEventListener("click", function (e) {
        e.preventDefault();
        VirtualGrid.formatSelectedCell({ font_style: "italic" });
      });
    }
    var underlineBtn = document.getElementById("underlineBtn");
    if (underlineBtn) {
      underlineBtn.addEventListener("click", function (e) {
        e.preventDefault();
        VirtualGrid.formatSelectedCell({ text_decoration: "underline" });
      });
    }
    var strikeBtn = document.getElementById("strikeBtn");
    if (strikeBtn) {
      strikeBtn.addEventListener("click", function (e) {
        e.preventDefault();
        VirtualGrid.formatSelectedCell({ text_decoration: "line-through" });
      });
    }
    var mergeBtn = document.getElementById("mergeCellsBtn");
    if (mergeBtn) {
      mergeBtn.addEventListener("click", function (e) {
        e.preventDefault();
        var row = VirtualGrid.selectedRow;
        var col = VirtualGrid.selectedCol;
        if (row == null || col == null) return;
        fetch("/api/sheet/merge", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            sheet_id: currentSheetId(),
            worksheet_index: WORKSHEET_INDEX,
            start_row: row,
            start_col: col,
            end_row: row,
            end_col: col
          })
        });
      });
    }
    var formulaInput = document.getElementById("formulaInput");
    if (formulaInput) {
      formulaInput.addEventListener("keydown", function (e) {
        if (e.key === "Enter") {
          e.preventDefault();
          var val = formulaInput.value;
          if (VirtualGrid.selectedRow != null && VirtualGrid.selectedCol != null) {
            var ref = colName(VirtualGrid.selectedCol) + (VirtualGrid.selectedRow + 1);
            SheetAPI.updateCell(ref, val).then(function (res) {
              if (res && res.success) {
                VirtualGrid.lastRenderedRange = null;
                VirtualGrid.requestRange();
              }
            });
          }
        }
      });
    }
  }
  window.initToolbar = initToolbar;

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

  function bootSheet() {
    // Discover per-window sheet data from __SHEET_DATA_MAP
    var myData = null;
    var myBody = document.currentScript ? document.currentScript.closest('[id^="window-body-"]') : null;
    if (!myBody) {
      // Fallback: find the visible sheet-content and climb
      var els = document.querySelectorAll('#sheet-content');
      for (var i = 0; i < els.length; i++) {
        var p = els[i].closest('[id^="window-body-"]');
        if (p && p.style.display !== 'none') { myBody = p; break; }
      }
    }
    if (myBody && myBody.dataset.windowId) {
      myData = (window.__SHEET_DATA_MAP || {})[myBody.dataset.windowId];
    }

    initSidebar();
    initAuth();
    initCollab();
    initToolbar();
    var bootPromise = window.__SHEET_BOOT || Promise.resolve();
    bootPromise.then(function () {
      var host = myBody ? myBody.querySelector('#sheet-content') : document.getElementById("sheet-content");
      if (host) {
        VirtualGrid.init(host);
        var loaded = myData && myData.loadedSheet ? myData.loadedSheet : window.__LOADED_SHEET;
        if (loaded && loaded.worksheets && loaded.worksheets.length > 0) {
          var ws = loaded.worksheets[0];
          if (ws.data) {
            for (var cellRef in ws.data) {
              VirtualGrid.cells.set(cellRef, ws.data[cellRef]);
            }
          }
          VirtualGrid.requestSeq++;
          VirtualGrid.totalRows = Math.max(VirtualGrid.totalRows, 50);
          VirtualGrid.render();
        }
        if (window.SheetAdvanced && window.SheetAdvanced.init) {
          var adv = window.SheetAdvanced.init(host, { sheetId: (host.dataset.sheetId || "current") });
          if (adv) window.SheetAdvancedInstance = adv;
        }
      }
      window.SheetAPI = SheetAPI;
      window.SheetVirtualGrid = VirtualGrid;
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bootSheet);
  } else {
    bootSheet();
  }
})();
