"use strict";
/* Sheet grid accessibility (#2493): ARIA grid semantics, a polite live region
   and a virtualised row-proxy layer. Loaded after 03_grid_render.js; every
   method here is installed on the shared VirtualGrid prototype object. */

(function () {
  var D = window.SheetDefaults;
  if (!D || !D.virtualGrid) return;
  var VirtualGrid = D.virtualGrid;
  var A11Y_DEBOUNCE_MS = 260;

  function t(key, fallback, params) {
    return D.a11yT ? D.a11yT(key, fallback, params) : fallback;
  }

  function numberText(n) {
    if (n == null || isNaN(Number(n))) return String(n == null ? "" : n);
    try {
      // #2329 — announce numbers in the suite locale, not the browser's.
      if (window.gbFormatNumber) {
        return window.gbFormatNumber(Number(n), { maximumFractionDigits: 2 });
      }
      return String(n);
    } catch (e) {
      return String(n);
    }
  }

  function loadedWorksheet(grid) {
    try {
      var inst = grid && grid.instance;
      var sheet = (inst && inst.loadedSheet) || window.__LOADED_SHEET;
      if (!sheet || !sheet.worksheets || !sheet.worksheets.length) return null;
      var index = (inst && typeof inst.worksheetIndex === "number")
        ? inst.worksheetIndex : D.WORKSHEET_INDEX;
      if (index < 0 || index >= sheet.worksheets.length) return null;
      return sheet.worksheets[index];
    } catch (e) {
      console.warn("[sheet-a11y] could not read worksheet metadata", e);
      return null;
    }
  }

  VirtualGrid.isReadOnly = function () {
    var ws = loadedWorksheet(this);
    if (!ws) return false;
    return !!(ws.protected || ws.sheet_protection || (ws.protection && ws.protection.enabled));
  };

  VirtualGrid.initA11y = function () {
    if (!this.root || this.a11yGridId) return;
    var id = "gb-sheet-grid-" + (this.windowId || "main");
    this.a11yGridId = id;
    this.root.id = id;
    this.root.setAttribute("role", "grid");
    this.root.setAttribute("aria-multiselectable", "true");
    this.root.setAttribute("aria-label", t("sheet-a11y-grid-label", "Spreadsheet grid"));
    this.root.setAttribute("aria-rowcount", String(this.totalRows || 1));
    this.root.setAttribute("aria-colcount", String(this.totalCols || 1));
    this.root.setAttribute("aria-readonly", this.isReadOnly() ? "true" : "false");
    // Focusing the container forwards to the active cell, keeping a single
    // useful tab stop while preserving the roving-cell focus model.
    this.root.tabIndex = 0;
    var self = this;
    this.root.addEventListener("focus", function () {
      if (self.selectedRow == null || self.selectedCol == null) return;
      var active = self.bodyInner && self.bodyInner.querySelector(
        '[data-row="' + self.selectedRow + '"][data-col="' + self.selectedCol + '"]');
      if (active && active.isConnected) active.focus({ preventScroll: true });
    });

    if (this.headerRow) this.headerRow.setAttribute("role", "row");
    if (this.bodyInner) this.bodyInner.setAttribute("role", "presentation");

    this.a11yLive = document.createElement("div");
    this.a11yLive.className = "sheet-a11y-live";
    this.a11yLive.setAttribute("role", "status");
    this.a11yLive.setAttribute("aria-live", "polite");
    this.a11yLive.setAttribute("aria-atomic", "true");
    this.root.appendChild(this.a11yLive);

    this.a11yRowGroup = document.createElement("div");
    this.a11yRowGroup.className = "sheet-a11y-rowgroup";
    this.a11yRowGroup.setAttribute("role", "rowgroup");
    this.a11yRowGroup.setAttribute("aria-label", t("sheet-a11y-rowgroup-label", "Grid rows"));
    this.root.appendChild(this.a11yRowGroup);

    this.associateFormulaBar();
    document.addEventListener("gb-sheet-selection", function () {
      self.updateSelectionA11y();
    });
  };

  VirtualGrid.associateFormulaBar = function () {
    var self = this;
    var scope = this.windowBody || document;
    var nameBox = scope.querySelector("#cellAddress");
    var formula = scope.querySelector("#formulaInput");
    var gridId = this.a11yGridId;
    if (nameBox) {
      nameBox.setAttribute("aria-controls", gridId);
      nameBox.setAttribute("aria-label", t("sheet-a11y-name-box-label", "Name box, enter a cell reference"));
      nameBox.setAttribute("aria-keyshortcuts", "Enter");
      var jump = function () {
        var ref = String(nameBox.value || "").trim();
        var pos = (typeof gbSheetParseA1 === "function") ? gbSheetParseA1(ref) : D.parseCellRef(ref);
        if (!pos || pos.row < 0 || pos.col < 0) {
          self.announce(t("sheet-a11y-invalid-reference", "Invalid cell reference: {value}", { value: ref }));
          return;
        }
        self.selectCell(pos.row, pos.col);
      };
      nameBox.addEventListener("keydown", function (e) {
        if (e.key === "Enter") { e.preventDefault(); jump(); }
      });
      nameBox.addEventListener("change", jump);
    }
    if (formula) {
      formula.setAttribute("aria-controls", gridId);
      formula.setAttribute("aria-label", t("sheet-a11y-formula-bar-label", "Formula bar"));
      formula.setAttribute("aria-keyshortcuts", "Enter Escape");
      formula.addEventListener("keydown", function (e) {
        if (e.key !== "Escape") return;
        var grid = window.SheetVirtualGrid || self;
        if (grid.selectedRow == null) return;
        var data = grid.cells.get(grid.selectedRow + "," + grid.selectedCol);
        formula.value = data && (data.formula || data.value) ? (data.formula || data.value) : "";
        e.preventDefault();
      });
    }
  };

  VirtualGrid.announce = function (text, immediate) {
    if (!text || !this.a11yLive) return;
    var live = this.a11yLive;
    if (this.a11yAnnounceTimer) clearTimeout(this.a11yAnnounceTimer);
    var write = function () {
      // Identical consecutive messages are not re-announced by most screen
      // readers; a trailing zero-width space alternates the string safely.
      var suffix = live.textContent === text ? "\u200b" : "";
      live.textContent = text + suffix;
    };
    if (immediate) write();
    else this.a11yAnnounceTimer = setTimeout(write, A11Y_DEBOUNCE_MS);
  };

  VirtualGrid.cellLabel = function (row, col, cellData) {
    var address = D.colName(col) + (row + 1);
    var value = cellData && cellData.value != null ? String(cellData.value) : "";
    var label = t("sheet-a11y-cell-label", "Cell {address}: {value}", { address: address, value: value });
    if (cellData && cellData.formula) {
      label += " " + t("sheet-a11y-cell-formula-label", "Formula {formula}",
        { formula: String(cellData.formula) });
    }
    return label;
  };

  VirtualGrid.updateCellA11y = function (node, row, col, cellData) {
    if (!node) return;
    node.setAttribute("aria-rowindex", String(row + 1));
    node.setAttribute("aria-colindex", String(col + 1));
    node.setAttribute("aria-readonly", this.isReadOnly() ? "true" : "false");
    node.setAttribute("aria-label", this.cellLabel(row, col, cellData));
    var sel = window.SheetAdvanced && window.SheetAdvanced.getSelection
      ? window.SheetAdvanced.getSelection() : null;
    var inside = sel && row >= sel.startRow && row <= sel.endRow &&
      col >= sel.startCol && col <= sel.endCol;
    node.setAttribute("aria-selected", inside ? "true" : "false");
    node.tabIndex = (row === this.selectedRow && col === this.selectedCol) ? 0 : -1;
  };

  VirtualGrid.selectionInfo = function () {
    var sel = window.SheetAdvanced && window.SheetAdvanced.getSelection
      ? window.SheetAdvanced.getSelection() : null;
    if (!sel || typeof sel.startRow !== "number") {
      if (this.selectedRow == null || this.selectedCol == null) return null;
      sel = {
        startRow: this.selectedRow, endRow: this.selectedRow,
        startCol: this.selectedCol, endCol: this.selectedCol,
      };
    }
    var rows = Math.max(1, sel.endRow - sel.startRow + 1);
    var cols = Math.max(1, sel.endCol - sel.startCol + 1);
    var ref = D.colName(sel.startCol) + (sel.startRow + 1);
    if (rows > 1 || cols > 1) {
      ref += ":" + D.colName(sel.endCol) + (sel.endRow + 1);
    }
    var sum = 0;
    var hasNumber = false;
    for (var r = sel.startRow; r <= sel.endRow; r++) {
      for (var c = sel.startCol; c <= sel.endCol; c++) {
        var data = this.cells.get(r + "," + c);
        if (!data || data.value == null || data.value === "") continue;
        var n = Number(data.value);
        if (!isNaN(n)) { sum += n; hasNumber = true; }
      }
    }
    return { sel: sel, rows: rows, cols: cols, ref: ref, sum: hasNumber ? sum : null };
  };

  VirtualGrid.announceSelection = function () {
    var info = this.selectionInfo();
    if (!info) return;
    this.announce(t("sheet-a11y-live-selection",
      "{range}, {rows} rows by {cols} columns, sum {sum}", {
        range: info.ref,
        rows: String(info.rows),
        cols: String(info.cols),
        sum: info.sum == null ? "\u2014" : numberText(info.sum),
      }));
  };

  VirtualGrid.updateSelectionA11y = function () {
    if (!this.bodyInner) return;
    var self = this;
    var sel = window.SheetAdvanced && window.SheetAdvanced.getSelection
      ? window.SheetAdvanced.getSelection() : null;
    Array.prototype.forEach.call(this.bodyInner.querySelectorAll('[role="gridcell"]'), function (node) {
      var r = parseInt(node.dataset.row, 10);
      var c = parseInt(node.dataset.col, 10);
      if (isNaN(r) || isNaN(c)) return;
      var inside = sel && r >= sel.startRow && r <= sel.endRow &&
        c >= sel.startCol && c <= sel.endCol;
      node.setAttribute("aria-selected", inside ? "true" : "false");
      node.tabIndex = (r === self.selectedRow && c === self.selectedCol) ? 0 : -1;
    });
    this.updateActiveCellA11y();
    this.announceSelection();
  };

  VirtualGrid.updateActiveCellA11y = function () {
    if (!this.root || !this.bodyInner) return;
    var active = null;
    if (this.selectedRow != null && this.selectedCol != null) {
      active = this.bodyInner.querySelector(
        '[data-row="' + this.selectedRow + '"][data-col="' + this.selectedCol + '"]');
      if (active && active.style.display === "none") active = null;
    }
    if (active) {
      this.root.setAttribute("aria-activedescendant", active.id || "");
      this.root.tabIndex = -1;
      active.tabIndex = 0;
    } else {
      this.root.setAttribute("aria-activedescendant", "");
      this.root.tabIndex = 0;
    }
  };

  VirtualGrid.syncA11yRows = function (range) {
    if (!this.root) return;
    this.root.setAttribute("aria-rowcount", String(this.totalRows || 1));
    this.root.setAttribute("aria-colcount", String(this.totalCols || 1));
    this.root.setAttribute("aria-readonly", this.isReadOnly() ? "true" : "false");
    if (!this.a11yRowGroup || !this.bodyInner) return;

    var byRow = {};
    Array.prototype.forEach.call(this.bodyInner.querySelectorAll('[role="gridcell"]'), function (node) {
      if (node.style.display === "none") return;
      var r = parseInt(node.dataset.row, 10);
      if (isNaN(r)) return;
      (byRow[r] = byRow[r] || []).push(node.id);
    });

    var used = 0;
    var self = this;
    for (var r = range.start; r < range.end; r++) {
      var proxy = self.a11yRowPool[used];
      if (!proxy) {
        proxy = document.createElement("div");
        self.a11yRowGroup.appendChild(proxy);
        self.a11yRowPool.push(proxy);
      }
      proxy.setAttribute("role", "row");
      proxy.setAttribute("aria-rowindex", String(r + 1));
      proxy.setAttribute("aria-owns", (byRow[r] || []).join(" "));
      proxy.style.display = "block";
      used++;
    }
    for (var i = used; i < this.a11yRowPool.length; i++) {
      this.a11yRowPool[i].style.display = "none";
      this.a11yRowPool[i].setAttribute("aria-owns", "");
    }
    this.updateActiveCellA11y();
  };

  VirtualGrid.onCellFocusA11y = function (e) {
    var cell = e.target;
    var row = parseInt(cell.dataset.row, 10);
    var col = parseInt(cell.dataset.col, 10);
    if (isNaN(row) || isNaN(col) || this.editingCell === cell) return;
    this.updateActiveCellA11y();
    var data = this.cells.get(row + "," + col);
    var message = this.cellLabel(row, col, data);
    this.announce(message);
  };

  VirtualGrid.onCellValueChanged = function (row, col, value) {
    var address = D.colName(col) + (row + 1);
    this.announce(t("sheet-a11y-live-edited", "Cell {address} updated to {value}", {
      address: address,
      value: value == null ? "" : String(value),
    }));
  };

  // Keep the prototype copy used by per-window grids (#2270) in sync.
  for (var k in VirtualGrid) {
    if (typeof VirtualGrid[k] === "function") D.GridProto[k] = VirtualGrid[k];
  }
})();
