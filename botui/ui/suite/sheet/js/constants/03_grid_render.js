"use strict";
/* sheet grid rendering — headers, cell pool and row painting */

(function () {
  var D = window.SheetDefaults;
  var ROW_HEIGHT = D.ROW_HEIGHT;
  var HEADER_WIDTH = D.HEADER_WIDTH;
  var colName = D.colName;
  var rTop = D.rTop;
  var rH = D.rH;
  const VirtualGrid = D.virtualGrid;

  VirtualGrid.init = function (host) {
    this.root = document.createElement("div");
    this.root.className = "virtual-grid";
    this.root.style.cssText = "position:relative;display:flex;flex-direction:column;flex:1;background:var(--sheet-bg, var(--sheet-bg, #0f172a));overflow:hidden;";

    this.headerRow = document.createElement("div");
    this.headerRow.className = "vg-header-row";
    this.headerRow.style.cssText = "display:flex;height:24px;background:var(--sheet-bg, var(--sheet-bg, #0f172a));border-bottom:1px solid var(--sheet-border, var(--sheet-border, #334155));overflow:hidden;flex-shrink:0;";

    this.midRow = document.createElement("div");
    this.midRow.style.cssText = "display:flex;flex:1;overflow:hidden;";

    this.scrollArea = document.createElement("div");
    this.scrollArea.className = "vg-scroll";
    this.scrollArea.style.cssText = "flex:1;overflow:auto;background:var(--sheet-bg, var(--sheet-bg, #0f172a));position:relative;";

    this.body = document.createElement("div");
    this.body.className = "vg-body";
    this.body.style.cssText = "position:relative;";

    const inner = document.createElement("div");
    inner.style.cssText = "position:relative;height:" + (this.totalRows * ROW_HEIGHT) + "px;width:" + this.totalColsWidth() + "px;";
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
    // Select first cell by default — unless the window was opened by a deep
    // link that asked for a specific cell (#2271), which must not be undone by
    // this deferred default.
    var self = this;
    setTimeout(function () {
      if (typeof gbSheetConsumePendingCell === 'function' && gbSheetConsumePendingCell()) return;
      self.selectCell(0, 0);
    }, 100);
  };

  VirtualGrid.renderHeaders = function () {
    const cols = this.visibleColRange();
    this.headerRow.innerHTML = "";
    const corner = document.createElement("div");
    corner.style.cssText = "width:" + HEADER_WIDTH + "px;background:var(--sheet-bg, var(--sheet-bg, #0f172a));border-right:1px solid var(--sheet-border, var(--sheet-border, #334155));flex-shrink:0;";
    this.headerRow.appendChild(corner);
    // Virtualized headers: absolute-positioned inside the translated header
    // row, so only the visible columns exist in the DOM (#786).
    for (let c = cols.start; c < cols.end; c++) {
      const h = document.createElement("div");
      h.className = "ss-col-head";
      h.dataset.col = c;
      h.textContent = colName(c);
      h.style.cssText = "position:absolute;left:" + this.colXOf(c) + "px;width:" + this.colWidthOf(c) + "px;background:var(--sheet-bg, var(--sheet-bg, #0f172a));color:var(--sheet-muted, var(--sheet-muted, #94a3b8));text-align:center;line-height:24px;font-size:11px;border-right:1px solid var(--sheet-border, var(--sheet-border, #334155));flex-shrink:0;box-sizing:border-box;";
      this.headerRow.appendChild(h);
    }
    this.headerColPool = [];
  };

  VirtualGrid.getOrCreateNode = function () {
    let node;
    if (this.usedCount < this.pool.length) {
      node = this.pool[this.usedCount];
    } else {
      node = document.createElement("div");
      node.className = "vg-cell";
      node.contentEditable = "false";
      node.tabIndex = -1;
      node.style.cssText = "position:absolute;background:var(--sheet-bg, var(--sheet-bg, #0f172a));color:var(--sheet-fg, var(--sheet-fg, #f8fafc));border-right:1px solid var(--sheet-border, var(--sheet-border, #334155));border-bottom:1px solid var(--sheet-border, var(--sheet-border, #334155));padding:2px 4px;font-size:12px;overflow:hidden;outline:none;box-sizing:border-box;cursor:pointer;";
      node.addEventListener("mousedown", this.onCellMouseDown.bind(this));
      node.addEventListener("dblclick", this.onCellDblClick.bind(this));
      node.addEventListener("blur", this.onCellBlur.bind(this));
      node.addEventListener("keydown", this.onCellKeyDown.bind(this));
      this.bodyInner.appendChild(node);
      this.pool.push(node);
    }
    this.usedCount++;
    return node;
  };

  VirtualGrid.render = function () {
    this.usedCount = 0;
    const range = this.visibleRowRange();

    this.renderRowHeaders(range);

    // Render the virtualised cells
    for (let r = range.start; r < range.end; r++) {
      this.renderRow(r);
    }
    for (let i = this.usedCount; i < this.pool.length; i++) {
      this.pool[i].style.display = "none";
    }
  };

  // #2025 — a cell with an explicit fill (a value the bot wrote with a
  // background, or conditional formatting) must stay readable in either theme.
  // Parse the fill and pick near-black/near-white text from its luminance,
  // unless the cell states its own colour.
  function parseColor(value) {
    var text = String(value == null ? "" : value).trim();
    var m = /^#([0-9a-f]{3})$/i.exec(text);
    if (m) {
      return m[1].split("").map(function (c) { return parseInt(c + c, 16); });
    }
    m = /^#([0-9a-f]{6})$/i.exec(text);
    if (m) {
      return [0, 2, 4].map(function (i) { return parseInt(m[1].slice(i, i + 2), 16); });
    }
    m = /^rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)/i.exec(text);
    if (m) return [Number(m[1]), Number(m[2]), Number(m[3])];
    return null;
  }

  function readableTextOn(background) {
    var rgb = parseColor(background);
    if (!rgb) return null;
    var luminance = (0.299 * rgb[0] + 0.587 * rgb[1] + 0.114 * rgb[2]) / 255;
    return luminance > 0.6 ? "#1f2328" : "var(--sheet-fg, #f8fafc)";
  }

  VirtualGrid.applyCellStyle = function (node, cellData) {
    if (!cellData || !cellData.style) return;
    var s = cellData.style;
    if (s.font_weight) node.style.fontWeight = s.font_weight;
    if (s.font_style) node.style.fontStyle = s.font_style;
    if (s.text_decoration) node.style.textDecoration = s.text_decoration;
    if (s.font_family) node.style.fontFamily = s.font_family;
    if (s.font_size) node.style.fontSize = s.font_size + 'px';
    if (s.color) node.style.color = s.color;
    if (s.background) {
      node.style.backgroundColor = s.background;
      if (!s.color) {
        var contrast = readableTextOn(s.background);
        if (contrast) node.style.color = contrast;
      }
    }
    if (s.text_align) node.style.textAlign = s.text_align;
    if (s.vertical_align) node.style.verticalAlign = s.vertical_align;
    if (s.border) node.style.border = s.border;
  };

  // Cell text that does not fit used to be clipped at the column edge, so a
  // title in a one-column cell read "FINAN…". Excel spills text into the empty
  // cells on its right; replicate that when at least three neighbours are free
  // and the cell is not part of a merged region (#2152).
  function mergedRegion(row, col) {
    var sheet = window.__LOADED_SHEET;
    var ws = sheet && sheet.worksheets && sheet.worksheets[D.WORKSHEET_INDEX];
    var merges = (ws && ws.merged_cells) || [];
    for (var i = 0; i < merges.length; i++) {
      var m = merges[i];
      if (row >= m.start_row && row <= m.end_row && col >= m.start_col && col <= m.end_col) {
        return true;
      }
    }
    return false;
  }

  VirtualGrid.canSpill = function (row, col, cellData) {
    if (!cellData) return false;
    var typed = cellData.typed;
    var textual = typed ? typed.t === "text" : typeof cellData.value === "string";
    if (!textual) return false;
    if (mergedRegion(row, col)) return false;
    for (var i = 1; i <= 3; i++) {
      var next = this.cells.get(row + "," + (col + i));
      if (next && next.value != null && String(next.value).length) return false;
      if (mergedRegion(row, col + i)) return false;
    }
    return true;
  };

  // Conditional-format rules and number-format colour sections ([Red], [Green],
  // …) are carried by the cell but were never applied during paint, so a
  // "negative red, positive green" cash-flow stayed the default white (#2260).
  function colorNameToHex(name) {
    var map = { red: "#DC2626", green: "#16A34A", blue: "#2563EB",
      yellow: "#CA8A04", magenta: "#C026D3", cyan: "#0891B2",
      white: "#FFFFFF", black: "#000000" };
    return map[String(name || "").toLowerCase()] || null;
  }

  function matchesCondition(rule, num, text) {
    var condition = String(rule.condition || "").trim();
    if (!condition) return false;
    var m = condition.match(/^(>=|<=|<>|>|<|=)\s*(-?\d+(?:\.\d+)?)$/);
    if (m && num != null && !isNaN(num)) {
      var rhs = Number(m[2]);
      switch (m[1]) {
        case ">": return num > rhs;
        case "<": return num < rhs;
        case ">=": return num >= rhs;
        case "<=": return num <= rhs;
        case "=": return num === rhs;
        case "<>": return num !== rhs;
      }
    }
    // "between a and b"
    var bm = condition.match(/^between\s+(-?\d+(?:\.\d+)?)\s+and\s+(-?\d+(?:\.\d+)?)$/i);
    if (bm && num != null && !isNaN(num)) {
      return num >= Number(bm[1]) && num <= Number(bm[2]);
    }
    // equality on text
    return String(text).trim().toLowerCase() === condition.toLowerCase();
  }

  VirtualGrid.conditionalStyleFor = function (row, col, cellData) {
    var sheet = window.__LOADED_SHEET;
    var ws = sheet && sheet.worksheets && sheet.worksheets[D.WORKSHEET_INDEX];
    var rules = ws && ws.conditional_formats;
    if (!rules || !rules.length) return null;
    var value = cellData && cellData.value != null ? cellData.value : "";
    var num = (cellData && cellData.typed && cellData.typed.t === "number")
      ? Number(cellData.typed.v) : (cellData && cellData.typed && typeof cellData.typed.as_number === "function")
      ? cellData.typed.as_number() : (value.trim() === "" ? null : Number(String(value).replace(/,/g, "")));
    for (var i = 0; i < rules.length; i++) {
      var rule = rules[i];
      if (row < rule.start_row || row > rule.end_row || col < rule.start_col || col > rule.end_col) continue;
      if (matchesCondition(rule, num, value)) return rule.style || null;
    }
    return null;
  };

  VirtualGrid.formatColorFor = function (cellData, value) {
    if (!cellData || !cellData.format) return null;
    var sections = String(cellData.format).split(";");
    var num = (cellData.typed && cellData.typed.t === "number")
      ? Number(cellData.typed.v) : (cellData.typed && typeof cellData.typed.as_number === "function")
      ? cellData.typed.as_number() : Number(String(value).replace(/,/g, ""));
    var idx = 0;
    if (!isNaN(num)) {
      if (num < 0 && sections.length > 1) idx = 1;
      else if (num === 0 && sections.length > 2) idx = 2;
    }
    var section = sections[idx] || sections[0] || "";
    var m = /\[(Red|Green|Blue|Yellow|Magenta|Cyan|White|Black)\]/i.exec(section);
    if (!m) return null;
    return colorNameToHex(m[1]);
  };

  VirtualGrid.renderRow = function (row) {
    const cols = this.visibleColRange();
    for (let c = cols.start; c < cols.end; c++) {
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
      node.style.left = this.colXOf(c) + "px";
      node.style.top = rTop(row) + "px";
      node.style.width = this.colWidthOf(c) + "px";
      node.style.height = rH(row) + "px";
      // Reset style then apply cell-level style
      node.style.fontWeight = "";
      node.style.fontStyle = "";
      node.style.textDecoration = "";
      node.style.fontFamily = "";
      node.style.fontSize = "12px";
      node.style.color = "var(--sheet-fg, var(--sheet-fg, #f8fafc))";
      node.style.backgroundColor = "var(--sheet-bg, var(--sheet-bg, #0f172a))";
      this.applyCellStyle(node, cellData);
      var condStyle = this.conditionalStyleFor(row, c, cellData);
      if (condStyle) this.applyCellStyle(node, { style: condStyle });
      var fmtColor = this.formatColorFor(cellData, value);
      if (fmtColor) node.style.color = fmtColor;
      if (this.editingCell !== node) {
        node.contentEditable = "false";
        node.style.outline = "none";
        node.style.zIndex = this.canSpill(row, c, cellData) ? "2" : "1";
      }
      node.style.overflow = cellData && this.canSpill(row, c, cellData) ? "visible" : "hidden";
    }
  };
})();
