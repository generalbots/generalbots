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
    // Select first cell by default
    var self = this;
    setTimeout(function () {
      self.selectCell(0, 0);
    }, 100);
  };

  VirtualGrid.renderHeaders = function () {
    const cols = this.visibleColRange();
    this.headerRow.innerHTML = "";
    const corner = document.createElement("div");
    corner.style.cssText = "width:" + HEADER_WIDTH + "px;background:#0f172a;border-right:1px solid #334155;flex-shrink:0;";
    this.headerRow.appendChild(corner);
    // Virtualized headers: absolute-positioned inside the translated header
    // row, so only the visible columns exist in the DOM (#786).
    for (let c = cols.start; c < cols.end; c++) {
      const h = document.createElement("div");
      h.className = "ss-col-head";
      h.dataset.col = c;
      h.textContent = colName(c);
      h.style.cssText = "position:absolute;left:" + this.colXOf(c) + "px;width:" + this.colWidthOf(c) + "px;background:#0f172a;color:#94a3b8;text-align:center;line-height:24px;font-size:11px;border-right:1px solid #334155;flex-shrink:0;box-sizing:border-box;";
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

  VirtualGrid.applyCellStyle = function (node, cellData) {
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
      node.style.color = "#f8fafc";
      node.style.backgroundColor = "#0f172a";
      this.applyCellStyle(node, cellData);
      if (this.editingCell !== node) {
        node.contentEditable = "false";
        node.style.outline = "none";
        node.style.zIndex = "1";
      }
    }
  };
})();
