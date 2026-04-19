/* =============================================================================
   GB SHEET - Modern Spreadsheet with AI Chat
   ============================================================================= */

(function () {
  "use strict";

  const CONFIG = {
    COLS: 26,
    ROWS: 100,
    COL_WIDTH: 100,
    ROW_HEIGHT: 24,
    MAX_HISTORY: 50,
    AUTOSAVE_DELAY: 3000,
    WS_RECONNECT_DELAY: 3000,
    VIRTUAL_SCROLL_THRESHOLD: 500,
    BUFFER_SIZE: 10,
  };

  let virtualGrid = null;
  let useVirtualScroll = false;

  class VirtualGrid {
    constructor(container, options = {}) {
      this.options = {
        colCount: options.colCount || CONFIG.COLS,
        rowCount: options.rowCount || CONFIG.ROWS,
        colWidth: options.colWidth || CONFIG.COL_WIDTH,
        rowHeight: options.rowHeight || CONFIG.ROW_HEIGHT,
        bufferSize: options.bufferSize || CONFIG.BUFFER_SIZE,
        ...options
      };
      
      this.container = container;
      this.cellCache = new Map();
      this.renderedCells = new Map();
      this.visibleStartRow = 0;
      this.visibleEndRow = 0;
      this.visibleStartCol = 0;
      this.visibleEndCol = 0;
      this.scrollLeft = 0;
      this.scrollTop = 0;
      this.isRendering = false;
      
      this.initialize();
    }

    initialize() {
      this.viewport = document.createElement('div');
      this.viewport.className = 'virtual-viewport';
      this.viewport.style.cssText = 'position:relative; overflow:auto; width:100%; height:100%;';
      
      this.content = document.createElement('div');
      this.content.className = 'virtual-content';
      this.content.style.cssText = `position:absolute; top:0; left:0; width:${this.options.colCount * this.options.colWidth}px; height:${this.options.rowCount * this.options.rowHeight}px;`;
      
      this.viewport.appendChild(this.content);
      this.container.appendChild(this.viewport);
      
      this.viewport.addEventListener('scroll', () => this.onScroll(), { passive: true });
      
      this.rowHeaders = document.createElement('div');
      this.rowHeaders.className = 'virtual-row-headers';
      this.rowHeaders.style.cssText = 'position:sticky; left:0; z-index:10; display:flex; flex-direction:column;';
      
      this.updateDimensions();
      this.onScroll();
    }

    updateDimensions() {
      this.content.style.width = `${this.options.colCount * this.options.colWidth}px`;
      this.content.style.height = `${this.options.rowCount * this.options.rowHeight}px`;
    }

    onScroll() {
      if (this.isRendering) return;
      
      const lastScrollTop = this.scrollTop;
      const lastScrollLeft = this.scrollLeft;
      
      this.scrollTop = this.viewport.scrollTop;
      this.scrollLeft = this.viewport.scrollLeft;
      
      if (this.scrollTop === lastScrollTop && this.scrollLeft === lastScrollLeft) return;
      
      requestAnimationFrame(() => this.renderVisibleCells());
    }

    renderVisibleCells() {
      this.isRendering = true;
      
      const viewHeight = this.viewport.clientHeight;
      const viewWidth = this.viewport.clientWidth;
      const buffer = this.options.bufferSize;
      
      const newStartRow = Math.max(0, Math.floor(this.scrollTop / this.options.rowHeight) - buffer);
      const newEndRow = Math.min(this.options.rowCount - 1, Math.ceil((this.scrollTop + viewHeight) / this.options.rowHeight) + buffer);
      const newStartCol = Math.max(0, Math.floor(this.scrollLeft / this.options.colWidth) - buffer);
      const newEndCol = Math.min(this.options.colCount - 1, Math.ceil((this.scrollLeft + viewWidth) / this.options.colWidth) + buffer);
      
      if (newStartRow === this.visibleStartRow && newEndRow === this.visibleEndRow &&
          newStartCol === this.visibleStartCol && newEndCol === this.visibleEndCol) {
        this.isRendering = false;
        return;
      }
      
      this.visibleStartRow = newStartRow;
      this.visibleEndRow = newEndRow;
      this.visibleStartCol = newStartCol;
      this.visibleEndCol = newEndCol;
      
      for (const [key, el] of this.renderedCells) {
        const [r, c] = key.split(',').map(Number);
        if (r < this.visibleStartRow || r > this.visibleEndRow ||
            c < this.visibleStartCol || c > this.visibleEndCol) {
          el.remove();
          this.renderedCells.delete(key);
        }
      }
      
      for (let row = this.visibleStartRow; row <= this.visibleEndRow; row++) {
        for (let col = this.visibleStartCol; col <= this.visibleEndCol; col++) {
          const key = `${row},${col}`;
          const cellData = this.cellCache.get(key);
          
          if (!this.renderedCells.has(key)) {
            const cell = this.createCellElement(row, col, cellData);
            cell.style.position = 'absolute';
            cell.style.top = `${row * this.options.rowHeight}px`;
            cell.style.left = `${col * this.options.colWidth}px`;
            cell.style.width = `${this.options.colWidth}px`;
            cell.style.height = `${this.options.rowHeight}px`;
            cell.dataset.row = row;
            cell.dataset.col = col;
            this.content.appendChild(cell);
            this.renderedCells.set(key, cell);
          }
        }
      }
      
      this.isRendering = false;
    }

    createCellElement(row, col, cellData) {
      const cell = document.createElement('div');
      cell.className = 'cell';
      
      if (cellData) {
        if (cellData.formula) {
          cell.textContent = evaluateFormula(cellData.formula, row, col);
        } else if (cellData.value !== undefined) {
          cell.textContent = cellData.value;
        }
        if (cellData.style) {
          this.applyStyle(cell, cellData.style);
        }
        if (cellData.merged) {
          const { rowSpan, colSpan } = cellData.merged;
          if (rowSpan > 1) cell.style.gridRow = `span ${rowSpan}`;
          if (colSpan > 1) cell.style.gridColumn = `span ${colSpan}`;
        }
      }
      
      return cell;
    }

    applyStyle(cell, style) {
      if (!style) return;
      if (style.fontFamily) cell.style.fontFamily = style.fontFamily;
      if (style.fontSize) cell.style.fontSize = style.fontSize + 'px';
      if (style.fontWeight) cell.style.fontWeight = style.fontWeight;
      if (style.fontStyle) cell.style.fontStyle = style.fontStyle;
      if (style.textDecoration) cell.style.textDecoration = style.textDecoration;
      if (style.color) cell.style.color = style.color;
      if (style.background) cell.style.backgroundColor = style.background;
      if (style.textAlign) cell.style.textAlign = style.textAlign;
    }

    setCellValue(row, col, value) {
      const key = `${row},${col}`;
      
      if (!value || (typeof value === 'object' && !value.value && !value.formula)) {
        this.cellCache.delete(key);
      } else {
        if (typeof value === 'object') {
          this.cellCache.set(key, value);
        } else {
          this.cellCache.set(key, { value: String(value) });
        }
      }
      
      if (row >= this.visibleStartRow && row <= this.visibleEndRow &&
          col >= this.visibleStartCol && col <= this.visibleEndCol) {
        const existing = this.renderedCells.get(key);
        
        if (!value || (typeof value === 'object' && !value.value && !value.formula)) {
          if (existing) {
            existing.remove();
            this.renderedCells.delete(key);
          }
        } else if (existing) {
          const cell = this.createCellElement(row, col, typeof value === 'object' ? value : { value });
          existing.textContent = cell.textContent;
          existing.style.cssText = cell.style.cssText;
        } else {
          const cell = this.createCellElement(row, col, typeof value === 'object' ? value : { value });
          cell.style.position = 'absolute';
          cell.style.top = `${row * this.options.rowHeight}px`;
          cell.style.left = `${col * this.options.colWidth}px`;
          cell.style.width = `${this.options.colWidth}px`;
          cell.style.height = `${this.options.rowHeight}px`;
          cell.dataset.row = row;
          cell.dataset.col = col;
          this.content.appendChild(cell);
          this.renderedCells.set(key, cell);
        }
      }
    }

    getCellValue(row, col) {
      return this.cellCache.get(`${row},${col}`);
    }

    scrollToCell(row, col) {
      const targetTop = row * this.options.rowHeight;
      const targetLeft = col * this.options.colWidth;
      const viewHeight = this.viewport.clientHeight;
      const viewWidth = this.viewport.clientWidth;
      
      this.viewport.scrollTo({
        left: targetLeft - viewWidth / 2,
        top: targetTop - viewHeight / 2,
        behavior: 'smooth'
      });
    }

    getVisibleRange() {
      return {
        startRow: this.visibleStartRow,
        endRow: this.visibleEndRow,
        startCol: this.visibleStartCol,
        endCol: this.visibleEndCol
      };
    }

    refresh() {
      this.renderVisibleCells();
    }

    destroy() {
      this.viewport.remove();
      this.cellCache.clear();
      this.renderedCells.clear();
    }

    loadData(data) {
      this.cellCache.clear();
      for (const [key, value] of Object.entries(data)) {
        if (value && (value.value || value.formula || value.style)) {
          this.cellCache.set(key, value);
        }
      }
      this.refresh();
    }

    getViewportScroll() {
      return { top: this.scrollTop, left: this.scrollLeft };
    }
  }

  const state = {
    sheetId: null,
    sheetName: "Untitled Spreadsheet",
    worksheets: [{ name: "Sheet1", data: {} }],
    activeWorksheet: 0,
    selection: {
      start: { row: 0, col: 0 },
      end: { row: 0, col: 0 },
    },
    activeCell: { row: 0, col: 0 },
    clipboard: null,
    clipboardMode: null,
    history: [],
    historyIndex: -1,
    zoom: 100,
    collaborators: [],
    ws: null,
    isEditing: false,
    isSelecting: false,
    isDirty: false,
    autoSaveTimer: null,

    findMatches: [],
    findMatchIndex: -1,
    decimalPlaces: 2,
  };

  const elements = {};

  class AuditLog {
    constructor() {
      this.entries = [];
      this.maxEntries = 1000;
    }

    log(action, details = {}) {
      const entry = {
        timestamp: new Date().toISOString(),
        action,
        details,
        sheetId: state.sheetId
      };
      this.entries.push(entry);
      if (this.entries.length > this.maxEntries) {
        this.entries.shift();
      }
      this.persistEntry(entry);
    }

    async persistEntry(entry) {
      if (!state.sheetId) return;
      try {
        await fetch('/api/sheet/audit', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(entry)
        });
      } catch (e) {
        console.warn('Audit log persist failed:', e);
      }
    }

    getHistory(filter = {}) {
      let filtered = this.entries;
      if (filter.action) {
        filtered = filtered.filter(e => e.action === filter.action);
      }
      if (filter.startTime) {
        filtered = filtered.filter(e => new Date(e.timestamp) >= new Date(filter.startTime));
      }
      if (filter.endTime) {
        filtered = filtered.filter(e => new Date(e.timestamp) <= new Date(filter.endTime));
      }
      return filtered;
    }
  }

  class VersionManager {
    constructor() {
      this.versions = [];
      this.currentVersion = -1;
      this.maxVersions = 100;
      this.autoSaveInterval = null;
      this.lastSavedState = null;
    }

    createSnapshot(reason = 'manual') {
      const snapshot = {
        timestamp: new Date().toISOString(),
        reason,
        worksheets: JSON.parse(JSON.stringify(state.worksheets)),
        sheetName: state.sheetName
      };

      if (this.lastSavedState && JSON.stringify(this.lastSavedState) === JSON.stringify(snapshot.worksheets)) {
        return null;
      }

      this.versions.push(snapshot);
      this.currentVersion = this.versions.length - 1;
      this.lastSavedState = JSON.parse(JSON.stringify(snapshot.worksheets));

      if (this.versions.length > this.maxVersions) {
        this.versions.shift();
        this.currentVersion--;
      }

      this.persistVersion(snapshot);
      return this.versions.length - 1;
    }

    async persistVersion(snapshot) {
      if (!state.sheetId) return;
      try {
        await fetch('/api/sheet/version', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            sheetId: state.sheetId,
            snapshot
          })
        });
      } catch (e) {
        console.warn('Version persist failed:', e);
      }
    }

    restoreVersion(versionIndex) {
      if (versionIndex < 0 || versionIndex >= this.versions.length) return false;

      const version = this.versions[versionIndex];
      state.worksheets = JSON.parse(JSON.stringify(version.worksheets));
      state.sheetName = version.sheetName;

      if (useVirtualScroll && virtualGrid) {
        const ws = state.worksheets[state.activeWorksheet];
        virtualGrid.loadData(ws?.data || {});
      } else {
        renderAllCells();
      }
      renderWorksheetTabs();

      auditLog.log('version_restore', { versionIndex, timestamp: version.timestamp });
      return true;
    }

    getVersionList() {
      return this.versions.map((v, i) => ({
        index: i,
        timestamp: v.timestamp,
        reason: v.reason,
        sheetName: v.sheetName
      })).reverse();
    }

    startAutoSave() {
      if (this.autoSaveInterval) return;
      this.autoSaveInterval = setInterval(() => {
        if (state.isDirty) {
          this.createSnapshot('auto');
        }
      }, 60000);
    }

    stopAutoSave() {
      if (this.autoSaveInterval) {
        clearInterval(this.autoSaveInterval);
        this.autoSaveInterval = null;
      }
    }
  }

  class PermissionManager {
    constructor() {
      this.permissions = new Map();
      this.currentUserLevel = 'edit';
    }

    setPermission(userId, level) {
      this.permissions.set(userId, level);
    }

    setCurrentUserLevel(level) {
      this.currentUserLevel = level;
    }

    canEdit() {
      return this.currentUserLevel === 'edit' || this.currentUserLevel === 'admin';
    }

    canDelete() {
      return this.currentUserLevel === 'admin';
    }

    canShare() {
      return this.currentUserLevel === 'admin';
    }

    canExport() {
      return this.currentUserLevel === 'view' || this.currentUserLevel === 'edit' || this.currentUserLevel === 'admin';
    }
  }

  const auditLog = new AuditLog();
  const versionManager = new VersionManager();
  const permissions = new PermissionManager();

  function init() {
    cacheElements();
    renderGrid();
    bindEvents();
    loadFromUrlParams();
    connectWebSocket();

    selectCell(0, 0);
    updateCellAddress();
    renderCharts();
    renderImages();
  }

  function cacheElements() {
    elements.app = document.getElementById("sheet-app");
    elements.sheetName = document.getElementById("sheetName");
    elements.columnHeaders = document.getElementById("columnHeaders");
    elements.rowHeaders = document.getElementById("rowHeaders");
    elements.cells = document.getElementById("cells");
    elements.cellsContainer = document.getElementById("cellsContainer");
    elements.formulaInput = document.getElementById("formulaInput");
    elements.cellAddress = document.getElementById("cellAddress");
    elements.worksheetTabs = document.getElementById("worksheetTabs");
    elements.collaborators = document.getElementById("collaborators");
    elements.contextMenu = document.getElementById("contextMenu");
    elements.shareModal = document.getElementById("shareModal");
    elements.chartModal = document.getElementById("chartModal");
    elements.cursorIndicators = document.getElementById("cursorIndicators");
    elements.selectionBox = document.getElementById("selectionBox");
    elements.selectionInfo = document.getElementById("selectionInfo");
    elements.calculationResult = document.getElementById("calculationResult");
    elements.saveStatus = document.getElementById("saveStatus");
    elements.zoomLevel = document.getElementById("zoomLevel");

    elements.findReplaceModal = document.getElementById("findReplaceModal");
    elements.conditionalFormatModal = document.getElementById(
      "conditionalFormatModal",
    );
    elements.dataValidationModal = document.getElementById(
      "dataValidationModal",
    );
    elements.printPreviewModal = document.getElementById("printPreviewModal");
    elements.customNumberFormatModal = document.getElementById(
      "customNumberFormatModal",
    );
    elements.insertImageModal = document.getElementById("insertImageModal");
  }

  function initVirtualGrid() {
    const container = document.getElementById('cellsContainer');
    if (!container || virtualGrid) return;
    
    virtualGrid = new VirtualGrid(container, {
      colCount: CONFIG.COLS,
      rowCount: CONFIG.ROWS,
      colWidth: CONFIG.COL_WIDTH,
      rowHeight: CONFIG.ROW_HEIGHT
    });
    
    const ws = state.worksheets[state.activeWorksheet];
    if (ws && ws.data) {
      virtualGrid.loadData(ws.data);
    }
  }

  function destroyVirtualGrid() {
    if (virtualGrid) {
      virtualGrid.destroy();
      virtualGrid = null;
    }
  }

  function renderGrid() {
    renderColumnHeaders();
    renderRowHeaders();
    
    useVirtualScroll = CONFIG.ROWS > CONFIG.VIRTUAL_SCROLL_THRESHOLD;
    
    if (useVirtualScroll) {
      elements.cells.style.display = 'none';
      if (!virtualGrid) {
        initVirtualGrid();
      } else {
        virtualGrid.refresh();
      }
    } else {
      if (virtualGrid) {
        destroyVirtualGrid();
      }
      elements.cells.style.display = '';
      renderAllCellsLegacy();
    }
  }

  function renderColumnHeaders() {
    elements.columnHeaders.innerHTML = "";
    for (let col = 0; col < CONFIG.COLS; col++) {
      const header = document.createElement("div");
      header.className = "column-header";
      header.textContent = getColName(col);
      header.dataset.col = col;
      header.addEventListener('click', handleColumnHeaderClick);
      elements.columnHeaders.appendChild(header);
    }
  }

  function renderRowHeaders() {
    elements.rowHeaders.innerHTML = "";
    const maxRows = useVirtualScroll ? Math.min(100, CONFIG.ROWS) : CONFIG.ROWS;
    for (let row = 0; row < maxRows; row++) {
      const header = document.createElement("div");
      header.className = "row-header";
      header.textContent = row + 1;
      header.dataset.row = row;
      header.addEventListener('click', handleRowHeaderClick);
      elements.rowHeaders.appendChild(header);
    }
  }

  function renderAllCellsLegacy() {
    const ws = state.worksheets[state.activeWorksheet];
    if (!ws) return;

    elements.cells.innerHTML = "";
    elements.cells.style.gridTemplateColumns = `repeat(${CONFIG.COLS}, ${CONFIG.COL_WIDTH}px)`;
    elements.cells.style.gridTemplateRows = `repeat(${CONFIG.ROWS}, ${CONFIG.ROW_HEIGHT}px)`;
    
    for (let row = 0; row < CONFIG.ROWS; row++) {
      for (let col = 0; col < CONFIG.COLS; col++) {
        const cell = document.createElement("div");
        cell.className = "cell";
        cell.dataset.row = row;
        cell.dataset.col = col;
        elements.cells.appendChild(cell);
      }
    }
    
    const cells = elements.cells.querySelectorAll(".cell");
    cells.forEach((cell) => {
      const row = parseInt(cell.dataset.row);
      const col = parseInt(cell.dataset.col);
      renderCellLegacy(row, col);
    });
  }

  function renderAllCells() {
    if (useVirtualScroll && virtualGrid) {
      const ws = state.worksheets[state.activeWorksheet];
      if (ws && ws.data) {
        virtualGrid.loadData(ws.data);
      }
    } else {
      renderAllCellsLegacy();
    }
  }

  function renderCell(row, col) {
    if (useVirtualScroll && virtualGrid) {
      const ws = state.worksheets[state.activeWorksheet];
      const data = ws?.data?.[`${row},${col}`];
      virtualGrid.setCellValue(row, col, data);
    } else {
      renderCellLegacy(row, col);
    }
  }

  function renderCellLegacy(row, col) {
    const cell = elements.cells.querySelector(
      `[data-row="${row}"][data-col="${col}"]`,
    );
    if (!cell) return;

    const data = getCellData(row, col);
    let displayValue = "";

    if (data) {
      if (data.formula) {
        displayValue = evaluateFormula(data.formula, row, col);
      } else if (data.value !== undefined) {
        displayValue = data.value;
      }
      applyFormatToCell(cell, data.style);
    } else {
      cell.style.cssText = "";
    }

    cell.textContent = displayValue;
  }

  function applyFormatToCell(cell, style) {
    if (!style) return;
    if (style.fontFamily) cell.style.fontFamily = style.fontFamily;
    if (style.fontSize) cell.style.fontSize = style.fontSize + "px";
    if (style.fontWeight) cell.style.fontWeight = style.fontWeight;
    if (style.fontStyle) cell.style.fontStyle = style.fontStyle;
    if (style.textDecoration) cell.style.textDecoration = style.textDecoration;
    if (style.color) cell.style.color = style.color;
    if (style.background) cell.style.backgroundColor = style.background;
    if (style.textAlign) cell.style.textAlign = style.textAlign;
  }

  function getColName(col) {
    let name = "";
    col++;
    while (col > 0) {
      col--;
      name = String.fromCharCode(65 + (col % 26)) + name;
      col = Math.floor(col / 26);
    }
    return name;
  }

  function parseColName(name) {
    let col = 0;
    for (let i = 0; i < name.length; i++) {
      col = col * 26 + (name.charCodeAt(i) - 64);
    }
    return col - 1;
  }

  function getCellRef(row, col) {
    return getColName(col) + (row + 1);
  }

  function parseCellRef(ref) {
    const match = ref.match(/^([A-Z]+)(\d+)$/i);
    if (!match) return null;
    return {
      row: parseInt(match[2]) - 1,
      col: parseColName(match[1].toUpperCase()),
    };
  }

  function bindEvents() {
    elements.cells.addEventListener("mousedown", handleCellMouseDown);
    elements.cells.addEventListener("dblclick", handleCellDoubleClick);
    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
    document.addEventListener("keydown", handleKeyDown);
    document.addEventListener("click", handleDocumentClick);
    document.addEventListener("contextmenu", handleContextMenu);

    elements.columnHeaders.addEventListener("click", handleColumnHeaderClick);
    elements.rowHeaders.addEventListener("click", handleRowHeaderClick);

    elements.formulaInput.addEventListener("keydown", handleFormulaKey);
    elements.formulaInput.addEventListener("input", updateFormulaPreview);

    document.getElementById("undoBtn")?.addEventListener("click", undo);
    document.getElementById("redoBtn")?.addEventListener("click", redo);
    document
      .getElementById("boldBtn")
      ?.addEventListener("click", () => formatCells("bold"));
    document
      .getElementById("italicBtn")
      ?.addEventListener("click", () => formatCells("italic"));
    document
      .getElementById("underlineBtn")
      ?.addEventListener("click", () => formatCells("underline"));
    document
      .getElementById("strikeBtn")
      ?.addEventListener("click", () => formatCells("strikethrough"));
    document
      .getElementById("alignLeftBtn")
      ?.addEventListener("click", () => formatCells("alignLeft"));
    document
      .getElementById("alignCenterBtn")
      ?.addEventListener("click", () => formatCells("alignCenter"));
    document
      .getElementById("alignRightBtn")
      ?.addEventListener("click", () => formatCells("alignRight"));
    document
      .getElementById("mergeCellsBtn")
      ?.addEventListener("click", mergeCells);
    document
      .getElementById("numberFormat")
      ?.addEventListener("change", handleNumberFormatChange);
    document
      .getElementById("decreaseDecimalBtn")
      ?.addEventListener("click", decreaseDecimal);
    document
      .getElementById("increaseDecimalBtn")
      ?.addEventListener("click", increaseDecimal);

    document
      .getElementById("textColorInput")
      ?.addEventListener("input", (e) => {
        formatCells("color", e.target.value);
        document.getElementById("textColorIndicator").style.background =
          e.target.value;
      });
    document.getElementById("bgColorInput")?.addEventListener("input", (e) => {
      formatCells("backgroundColor", e.target.value);
      document.getElementById("bgColorIndicator").style.background =
        e.target.value;
    });

    document
      .getElementById("fontFamily")
      ?.addEventListener("change", (e) =>
        formatCells("fontFamily", e.target.value),
      );
    document
      .getElementById("fontSize")
      ?.addEventListener("change", (e) =>
        formatCells("fontSize", e.target.value),
      );

    document
      .getElementById("shareBtn")
      ?.addEventListener("click", showShareModal);
    document
      .getElementById("closeShareModal")
      ?.addEventListener("click", () => hideModal("shareModal"));
    document
      .getElementById("closeChartModal")
      ?.addEventListener("click", () => hideModal("chartModal"));
    document
      .getElementById("copyLinkBtn")
      ?.addEventListener("click", copyShareLink);

    document
      .getElementById("addSheetBtn")
      ?.addEventListener("click", addWorksheet);
    document.getElementById("zoomInBtn")?.addEventListener("click", zoomIn);
    document.getElementById("zoomOutBtn")?.addEventListener("click", zoomOut);

    document
      .getElementById("importXlsxBtn")
      ?.addEventListener("click", () => {
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = '.xlsx,.xls,.csv,.ods';
        input.onchange = async (e) => {
          if (e.target.files[0]) {
            await importXlsx(e.target.files[0]);
          }
        };
        input.click();
      });
    document.getElementById("exportXlsxBtn")?.addEventListener("click", exportXlsx);
    document.getElementById("exportCsvBtn")?.addEventListener("click", exportCsv);

    document
      .getElementById("findReplaceBtn")
      ?.addEventListener("click", showFindReplaceModal);
    document
      .getElementById("closeFindReplaceModal")
      ?.addEventListener("click", () => hideModal("findReplaceModal"));
    document.getElementById("findNextBtn")?.addEventListener("click", findNext);
    document.getElementById("findPrevBtn")?.addEventListener("click", findPrev);
    document
      .getElementById("replaceBtn")
      ?.addEventListener("click", replaceOne);
    document
      .getElementById("replaceAllBtn")
      ?.addEventListener("click", replaceAll);
    document
      .getElementById("findInput")
      ?.addEventListener("input", performFind);

    document
      .getElementById("conditionalFormatBtn")
      ?.addEventListener("click", showConditionalFormatModal);
    document
      .getElementById("closeConditionalFormatModal")
      ?.addEventListener("click", () => hideModal("conditionalFormatModal"));
    document
      .getElementById("applyCfBtn")
      ?.addEventListener("click", applyConditionalFormat);
    document
      .getElementById("cancelCfBtn")
      ?.addEventListener("click", () => hideModal("conditionalFormatModal"));
    document
      .getElementById("cfRuleType")
      ?.addEventListener("change", handleCfRuleTypeChange);
    document
      .getElementById("cfBgColor")
      ?.addEventListener("input", updateCfPreview);
    document
      .getElementById("cfTextColor")
      ?.addEventListener("input", updateCfPreview);
    document
      .getElementById("cfBold")
      ?.addEventListener("change", updateCfPreview);
    document
      .getElementById("cfItalic")
      ?.addEventListener("change", updateCfPreview);

    document
      .getElementById("dataValidationBtn")
      ?.addEventListener("click", showDataValidationModal);
    document
      .getElementById("closeDataValidationModal")
      ?.addEventListener("click", () => hideModal("dataValidationModal"));
    document
      .getElementById("applyDvBtn")
      ?.addEventListener("click", applyDataValidation);
    document
      .getElementById("cancelDvBtn")
      ?.addEventListener("click", () => hideModal("dataValidationModal"));
    document
      .getElementById("clearDvBtn")
      ?.addEventListener("click", clearDataValidation);
    document
      .getElementById("dvType")
      ?.addEventListener("change", handleDvTypeChange);
    document
      .getElementById("dvOperator")
      ?.addEventListener("change", handleDvOperatorChange);
    document.querySelectorAll(".dv-tab").forEach((tab) => {
      tab.addEventListener("click", () => switchDvTab(tab.dataset.tab));
    });

    document
      .getElementById("printPreviewBtn")
      ?.addEventListener("click", showPrintPreview);
    document
      .getElementById("closePrintPreviewModal")
      ?.addEventListener("click", () => hideModal("printPreviewModal"));
    document.getElementById("printBtn")?.addEventListener("click", printSheet);
    document
      .getElementById("cancelPrintBtn")
      ?.addEventListener("click", () => hideModal("printPreviewModal"));
    document
      .getElementById("printOrientation")
      ?.addEventListener("change", updatePrintPreview);
    document
      .getElementById("printPaperSize")
      ?.addEventListener("change", updatePrintPreview);
    document
      .getElementById("printScale")
      ?.addEventListener("change", updatePrintPreview);
    document
      .getElementById("printGridlines")
      ?.addEventListener("change", updatePrintPreview);
    document
      .getElementById("printHeaders")
      ?.addEventListener("change", updatePrintPreview);

    document
      .getElementById("insertChartBtn")
      ?.addEventListener("click", () => showModal("chartModal"));
    document
      .getElementById("insertChartBtnConfirm")
      ?.addEventListener("click", insertChart);
    document
      .getElementById("cancelChartBtn")
      ?.addEventListener("click", () => hideModal("chartModal"));

    document
      .getElementById("insertImageBtn")
      ?.addEventListener("click", showInsertImageModal);
    document
      .getElementById("closeInsertImageModal")
      ?.addEventListener("click", () => hideModal("insertImageModal"));
    document
      .getElementById("insertImgBtn")
      ?.addEventListener("click", insertImage);
    document
      .getElementById("cancelImgBtn")
      ?.addEventListener("click", () => hideModal("insertImageModal"));
    document.querySelectorAll(".img-tab").forEach((tab) => {
      tab.addEventListener("click", () => switchImgTab(tab.dataset.tab));
    });

    document
      .getElementById("filterBtn")
      ?.addEventListener("click", toggleFilter);
    document
      .getElementById("sortAscBtn")
      ?.addEventListener("click", sortAscending);
    document
      .getElementById("sortDescBtn")
      ?.addEventListener("click", sortDescending);

    document
      .getElementById("closeCustomFormatModal")
      ?.addEventListener("click", () => hideModal("customNumberFormatModal"));
    document
      .getElementById("applyCnfBtn")
      ?.addEventListener("click", applyCustomNumberFormat);
    document
      .getElementById("cancelCnfBtn")
      ?.addEventListener("click", () => hideModal("customNumberFormatModal"));
    document.querySelectorAll(".cnf-format-item").forEach((item) => {
      item.addEventListener("click", () =>
        selectCustomFormat(item.dataset.format),
      );
    });
    document
      .getElementById("cnfFormatCode")
      ?.addEventListener("input", updateCnfPreview);



    document.querySelectorAll(".context-item").forEach((item) => {
      item.addEventListener("click", () =>
        handleContextAction(item.dataset.action),
      );
    });

    elements.sheetName?.addEventListener("change", (e) => {
      state.sheetName = e.target.value;
      scheduleAutoSave();
    });

    window.addEventListener("beforeunload", handleBeforeUnload);
  }

  function handleCellMouseDown(e) {
    const cell = e.target.closest(".cell");
    if (!cell) return;

    const row = parseInt(cell.dataset.row);
    const col = parseInt(cell.dataset.col);

    if (state.isEditing) {
      finishEditing();
    }

    if (e.shiftKey) {
      extendSelection(row, col);
    } else {
      selectCell(row, col);
      state.isSelecting = true;
    }
  }

  function handleMouseMove(e) {
    if (!state.isSelecting) return;

    const cell = document
      .elementFromPoint(e.clientX, e.clientY)
      ?.closest(".cell");
    if (cell) {
      const row = parseInt(cell.dataset.row);
      const col = parseInt(cell.dataset.col);
      extendSelection(row, col);
    }
  }

  function handleMouseUp() {
    state.isSelecting = false;
  }

  function handleCellDoubleClick(e) {
    const cell = e.target.closest(".cell");
    if (!cell) return;

    const row = parseInt(cell.dataset.row);
    const col = parseInt(cell.dataset.col);
    startEditing(row, col);
  }

  function selectCell(row, col) {
    clearSelection();

    state.activeCell = { row, col };
    state.selection = {
      start: { row, col },
      end: { row, col },
    };

    if (useVirtualScroll && virtualGrid) {
      virtualGrid.scrollToCell(row, col);
      setTimeout(() => highlightVirtualCell(row, col), 50);
    } else {
      const cell = elements.cells.querySelector(
        `[data-row="${row}"][data-col="${col}"]`,
      );
      if (cell) {
        cell.classList.add("selected");
        cell.scrollIntoView({ block: "nearest", inline: "nearest" });
      }
    }

    updateCellAddress();
    updateFormulaBar();
    updateSelectionInfo();
  }

  function highlightVirtualCell(row, col) {
    const cell = elements.cells.querySelector(`[data-row="${row}"][data-col="${col}"]`);
    if (cell && !cell.classList.contains('selected')) {
      clearSelection();
      cell.classList.add('selected');
    }
  }

  function extendSelection(row, col) {
    clearSelection();

    const start = state.activeCell;
    state.selection = {
      start: {
        row: Math.min(start.row, row),
        col: Math.min(start.col, col),
      },
      end: {
        row: Math.max(start.row, row),
        col: Math.max(start.col, col),
      },
    };

    for (let r = state.selection.start.row; r <= state.selection.end.row; r++) {
      for (
        let c = state.selection.start.col;
        c <= state.selection.end.col;
        c++
      ) {
        const cell = elements.cells.querySelector(
          `[data-row="${r}"][data-col="${c}"]`,
        );
        if (cell) {
          if (r === state.activeCell.row && c === state.activeCell.col) {
            cell.classList.add("selected");
          } else {
            cell.classList.add("in-range");
          }
        }
      }
    }

    updateSelectionInfo();
    updateCalculationResult();
  }

  function clearSelection() {
    elements.cells
      .querySelectorAll(".cell.selected, .cell.in-range")
      .forEach((cell) => {
        cell.classList.remove("selected", "in-range");
      });
  }

  function handleColumnHeaderClick(e) {
    const header = e.target.closest(".column-header");
    if (!header) return;

    const col = parseInt(header.dataset.col);
    clearSelection();

    state.activeCell = { row: 0, col };
    state.selection = {
      start: { row: 0, col },
      end: { row: CONFIG.ROWS - 1, col },
    };

    for (let row = 0; row < CONFIG.ROWS; row++) {
      const cell = elements.cells.querySelector(
        `[data-row="${row}"][data-col="${col}"]`,
      );
      if (cell) cell.classList.add("in-range");
    }

    header.classList.add("selected");
    updateSelectionInfo();
  }

  function handleRowHeaderClick(e) {
    const header = e.target.closest(".row-header");
    if (!header) return;

    const row = parseInt(header.dataset.row);
    clearSelection();

    state.activeCell = { row, col: 0 };
    state.selection = {
      start: { row, col: 0 },
      end: { row, col: CONFIG.COLS - 1 },
    };

    for (let col = 0; col < CONFIG.COLS; col++) {
      const cell = elements.cells.querySelector(
        `[data-row="${row}"][data-col="${col}"]`,
      );
      if (cell) cell.classList.add("in-range");
    }

    header.classList.add("selected");
    updateSelectionInfo();
  }

  function startEditing(row, col) {
    const cell = elements.cells.querySelector(
      `[data-row="${row}"][data-col="${col}"]`,
    );
    if (!cell) return;

    state.isEditing = true;
    const data = getCellData(row, col);

    const input = document.createElement("input");
    input.type = "text";
    input.className = "cell-input";
    input.value = data?.formula || data?.value || "";
    cell.textContent = "";
    cell.classList.add("editing");
    cell.appendChild(input);
    input.focus();
    input.select();

    input.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        finishEditing(true);
        navigateCell(1, 0);
      } else if (e.key === "Tab") {
        e.preventDefault();
        finishEditing(true);
        navigateCell(0, e.shiftKey ? -1 : 1);
      } else if (e.key === "Escape") {
        cancelEditing();
      }
    });

    input.addEventListener("blur", () => {
      if (state.isEditing) finishEditing(true);
    });
  }

  function finishEditing(save = true) {
    if (!state.isEditing) return;

    const { row, col } = state.activeCell;
    const cell = elements.cells.querySelector(
      `[data-row="${row}"][data-col="${col}"]`,
    );
    const input = cell?.querySelector(".cell-input");

    if (input && save) {
      const value = input.value.trim();
      setCellValue(row, col, value);
    }

    state.isEditing = false;
    cell?.classList.remove("editing");
    renderCell(row, col);
    updateFormulaBar();
  }

  function cancelEditing() {
    state.isEditing = false;
    const { row, col } = state.activeCell;
    const cell = elements.cells.querySelector(
      `[data-row="${row}"][data-col="${col}"]`,
    );
    cell?.classList.remove("editing");
    renderCell(row, col);
  }

  function setCellValue(row, col, value) {
    if (!permissions.canEdit()) {
      addChatMessage("system", "You don't have permission to edit this sheet");
      return;
    }

    const ws = state.worksheets[state.activeWorksheet];
    const key = `${row},${col}`;
    const oldValue = ws.data[key]?.value || ws.data[key]?.formula || '';

    saveToHistory();

    if (!value) {
      delete ws.data[key];
    } else if (value.startsWith("=")) {
      ws.data[key] = { formula: value };
    } else {
      ws.data[key] = { value };
    }

    if (useVirtualScroll && virtualGrid) {
      virtualGrid.setCellValue(row, col, ws.data[key]);
    }

    auditLog.log('cell_change', { row, col, oldValue, newValue: value, cellRef: getCellRef(row, col) });
    
    state.isDirty = true;
    scheduleAutoSave();
    broadcastChange("cell", { row, col, value });
  }

  function getCellData(row, col) {
    const ws = state.worksheets[state.activeWorksheet];
    return ws?.data[`${row},${col}`];
  }

  function getCellValue(row, col) {
    const data = getCellData(row, col);
    if (!data) return "";
    if (data.formula) return evaluateFormula(data.formula, row, col);
    return data.value || "";
  }

  function getCellValue(row, col) {
    const data = getCellData(row, col);
    if (!data) return "";
    if (data.formula) return evaluateFormula(data.formula, row, col);
    return data.value || "";
  }

  function evaluateFormula(formula, sourceRow, sourceCol) {
    if (!formula.startsWith("=")) return formula;

    try {
      let expr = formula.substring(1).toUpperCase();

      expr = expr.replace(/([A-Z]+)(\d+)/g, (match, col, row) => {
        const r = parseInt(row) - 1;
        const c = parseColName(col);
        const val = getCellValue(r, c);
        const num = parseFloat(val);
        return isNaN(num) ? `"${val}"` : num;
      });

      if (expr.startsWith("SUM(")) {
        return evaluateSum(expr);
      } else if (expr.startsWith("AVERAGE(")) {
        return evaluateAverage(expr);
      } else if (expr.startsWith("COUNT(")) {
        return evaluateCount(expr);
      } else if (expr.startsWith("MAX(")) {
        return evaluateMax(expr);
      } else if (expr.startsWith("MIN(")) {
        return evaluateMin(expr);
      } else if (expr.startsWith("IF(")) {
        return evaluateIf(expr);
      }

      const result = new Function("return " + expr)();
      return typeof result === "number"
        ? Math.round(result * 1000000) / 1000000
        : result;
    } catch (e) {
      return "#ERROR";
    }
  }

  function evaluateSum(expr) {
    const match = expr.match(/SUM\(([^)]+)\)/i);
    if (!match) return "#ERROR";
    const values = parseRange(match[1]);
    return values.reduce((a, b) => a + b, 0);
  }

  function evaluateAverage(expr) {
    const match = expr.match(/AVERAGE\(([^)]+)\)/i);
    if (!match) return "#ERROR";
    const values = parseRange(match[1]);
    return values.length
      ? values.reduce((a, b) => a + b, 0) / values.length
      : 0;
  }

  function evaluateCount(expr) {
    const match = expr.match(/COUNT\(([^)]+)\)/i);
    if (!match) return "#ERROR";
    const values = parseRange(match[1]);
    return values.length;
  }

  function evaluateMax(expr) {
    const match = expr.match(/MAX\(([^)]+)\)/i);
    if (!match) return "#ERROR";
    const values = parseRange(match[1]);
    return values.length ? Math.max(...values) : 0;
  }

  function evaluateMin(expr) {
    const match = expr.match(/MIN\(([^)]+)\)/i);
    if (!match) return "#ERROR";
    const values = parseRange(match[1]);
    return values.length ? Math.min(...values) : 0;
  }

  function evaluateIf(expr) {
    const match = expr.match(/IF\(([^,]+),([^,]+),([^)]+)\)/i);
    if (!match) return "#ERROR";
    try {
      const condition = new Function("return " + match[1])();
      return condition
        ? new Function("return " + match[2])()
        : new Function("return " + match[3])();
    } catch {
      return "#ERROR";
    }
  }

  function parseRange(rangeStr) {
    const values = [];
    const parts = rangeStr.split(":");

    if (parts.length === 2) {
      const start = parseCellRef(parts[0].trim());
      const end = parseCellRef(parts[1].trim());
      if (start && end) {
        for (let r = start.row; r <= end.row; r++) {
          for (let c = start.col; c <= end.col; c++) {
            const val = parseFloat(getCellValue(r, c));
            if (!isNaN(val)) values.push(val);
          }
        }
      }
    } else {
      const ref = parseCellRef(parts[0].trim());
      if (ref) {
        const val = parseFloat(getCellValue(ref.row, ref.col));
        if (!isNaN(val)) values.push(val);
      }
    }

    return values;
  }

  function handleKeyDown(e) {
    if (e.target.closest(".chat-input, .modal input, .sheet-name-input"))
      return;

    const { row, col } = state.activeCell;

    if (e.ctrlKey || e.metaKey) {
      switch (e.key.toLowerCase()) {
        case "c":
          copySelection();
          return;
        case "x":
          cutSelection();
          return;
        case "v":
          pasteSelection();
          return;
        case "z":
          e.shiftKey ? redo() : undo();
          e.preventDefault();
          return;
        case "y":
          redo();
          e.preventDefault();
          return;
        case "b":
          formatCells("bold");
          e.preventDefault();
          return;
        case "i":
          formatCells("italic");
          e.preventDefault();
          return;
        case "u":
          formatCells("underline");
          e.preventDefault();
          return;
        case "a":
          selectAll();
          e.preventDefault();
          return;
      }
    }

    if (state.isEditing) return;

    switch (e.key) {
      case "ArrowUp":
        navigateCell(-1, 0);
        e.preventDefault();
        break;
      case "ArrowDown":
        navigateCell(1, 0);
        e.preventDefault();
        break;
      case "ArrowLeft":
        navigateCell(0, -1);
        e.preventDefault();
        break;
      case "ArrowRight":
        navigateCell(0, 1);
        e.preventDefault();
        break;
      case "Tab":
        navigateCell(0, e.shiftKey ? -1 : 1);
        e.preventDefault();
        break;
      case "Enter":
        if (e.shiftKey) {
          navigateCell(-1, 0);
        } else {
          startEditing(row, col);
        }
        e.preventDefault();
        break;
      case "Delete":
      case "Backspace":
        clearCells();
        e.preventDefault();
        break;
      case "F2":
        startEditing(row, col);
        e.preventDefault();
        break;
      default:
        if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {
          startEditing(row, col);
          const cell = elements.cells.querySelector(
            `[data-row="${row}"][data-col="${col}"]`,
          );
          const input = cell?.querySelector(".cell-input");
          if (input) input.value = e.key;
        }
    }
  }

  function navigateCell(dRow, dCol) {
    const newRow = Math.max(
      0,
      Math.min(CONFIG.ROWS - 1, state.activeCell.row + dRow),
    );
    const newCol = Math.max(
      0,
      Math.min(CONFIG.COLS - 1, state.activeCell.col + dCol),
    );
    selectCell(newRow, newCol);
  }

  function selectAll() {
    clearSelection();
    state.selection = {
      start: { row: 0, col: 0 },
      end: { row: CONFIG.ROWS - 1, col: CONFIG.COLS - 1 },
    };

    elements.cells.querySelectorAll(".cell").forEach((cell) => {
      cell.classList.add("in-range");
    });

    const activeCell = elements.cells.querySelector(
      `[data-row="${state.activeCell.row}"][data-col="${state.activeCell.col}"]`,
    );
    if (activeCell) {
      activeCell.classList.remove("in-range");
      activeCell.classList.add("selected");
    }

    updateSelectionInfo();
  }

  function handleFormulaKey(e) {
    if (e.key === "Enter") {
      e.preventDefault();
      const value = elements.formulaInput.value;
      const { row, col } = state.activeCell;
      setCellValue(row, col, value);
      renderCell(row, col);
      elements.formulaInput.blur();
    } else if (e.key === "Escape") {
      updateFormulaBar();
      elements.formulaInput.blur();
    }
  }

  function updateFormulaPreview() {
    const value = elements.formulaInput.value;
    if (value.startsWith("=")) {
      const result = evaluateFormula(
        value,
        state.activeCell.row,
        state.activeCell.col,
      );
      elements.calculationResult.textContent = `= ${result}`;
    } else {
      elements.calculationResult.textContent = "";
    }
  }

  function updateCellAddress() {
    const ref = getCellRef(state.activeCell.row, state.activeCell.col);
    elements.cellAddress.textContent = ref;
  }

  function updateFormulaBar() {
    const data = getCellData(state.activeCell.row, state.activeCell.col);
    elements.formulaInput.value = data?.formula || data?.value || "";
  }

  function updateSelectionInfo() {
    const { start, end } = state.selection;
    const rows = end.row - start.row + 1;
    const cols = end.col - start.col + 1;
    const count = rows * cols;

    if (count === 1) {
      elements.selectionInfo.textContent = "Ready";
    } else {
      elements.selectionInfo.textContent = `${rows}R × ${cols}C = ${count} cells`;
    }
  }

  function updateCalculationResult() {
    const { start, end } = state.selection;
    const values = [];

    for (let r = start.row; r <= end.row; r++) {
      for (let c = start.col; c <= end.col; c++) {
        const val = parseFloat(getCellValue(r, c));
        if (!isNaN(val)) values.push(val);
      }
    }

    if (values.length > 1) {
      const sum = values.reduce((a, b) => a + b, 0);
      const avg = sum / values.length;
      elements.calculationResult.textContent = `Sum: ${sum.toFixed(2)} | Avg: ${avg.toFixed(2)} | Count: ${values.length}`;
    } else {
      elements.calculationResult.textContent = "";
    }
  }

  function copySelection() {
    state.clipboard = getSelectionData();
    state.clipboardMode = "copy";
    showCopyBox();
  }

  function cutSelection() {
    state.clipboard = getSelectionData();
    state.clipboardMode = "cut";
    showCopyBox();
  }

  function pasteSelection() {
    if (!state.clipboard) return;

    saveToHistory();
    const { row, col } = state.activeCell;
    const ws = state.worksheets[state.activeWorksheet];

    state.clipboard.forEach((rowData, rOffset) => {
      rowData.forEach((cellData, cOffset) => {
        const targetRow = row + rOffset;
        const targetCol = col + cOffset;
        const key = `${targetRow},${targetCol}`;

        if (cellData) {
          ws.data[key] = { ...cellData };
        }

        renderCell(targetRow, targetCol);
      });
    });

    if (state.clipboardMode === "cut") {
      clearSourceCells();
      state.clipboardMode = null;
    }

    hideCopyBox();
    state.isDirty = true;
    scheduleAutoSave();
  }

  function getSelectionData() {
    const { start, end } = state.selection;
    const data = [];

    for (let r = start.row; r <= end.row; r++) {
      const rowData = [];
      for (let c = start.col; c <= end.col; c++) {
        rowData.push(getCellData(r, c) || null);
      }
      data.push(rowData);
    }

    return data;
  }

  function clearSourceCells() {
    const { start, end } = state.selection;
    const ws = state.worksheets[state.activeWorksheet];

    for (let r = start.row; r <= end.row; r++) {
      for (let c = start.col; c <= end.col; c++) {
        delete ws.data[`${r},${c}`];
        renderCell(r, c);
      }
    }
  }

  function clearCells() {
    saveToHistory();
    const { start, end } = state.selection;
    const ws = state.worksheets[state.activeWorksheet];

    for (let r = start.row; r <= end.row; r++) {
      for (let c = start.col; c <= end.col; c++) {
        delete ws.data[`${r},${c}`];
        renderCell(r, c);
      }
    }

    state.isDirty = true;
    scheduleAutoSave();
  }

  function showCopyBox() {
    const copyBox = document.getElementById("copyBox");
    if (copyBox) copyBox.classList.remove("hidden");
  }

  function hideCopyBox() {
    const copyBox = document.getElementById("copyBox");
    if (copyBox) copyBox.classList.add("hidden");
  }

  function formatCells(format, value) {
    saveToHistory();
    const { start, end } = state.selection;
    const ws = state.worksheets[state.activeWorksheet];

    for (let r = start.row; r <= end.row; r++) {
      for (let c = start.col; c <= end.col; c++) {
        const key = `${r},${c}`;
        if (!ws.data[key]) ws.data[key] = { value: "" };
        if (!ws.data[key].style) ws.data[key].style = {};

        const style = ws.data[key].style;

        switch (format) {
          case "bold":
            style.fontWeight = style.fontWeight === "bold" ? "normal" : "bold";
            break;
          case "italic":
            style.fontStyle =
              style.fontStyle === "italic" ? "normal" : "italic";
            break;
          case "underline":
            style.textDecoration =
              style.textDecoration === "underline" ? "none" : "underline";
            break;
          case "strikethrough":
            style.textDecoration =
              style.textDecoration === "line-through" ? "none" : "line-through";
            break;
          case "alignLeft":
            style.textAlign = "left";
            break;
          case "alignCenter":
            style.textAlign = "center";
            break;
          case "alignRight":
            style.textAlign = "right";
            break;
          case "fontFamily":
            style.fontFamily = value;
            break;
          case "fontSize":
            style.fontSize = value;
            break;
          case "color":
            style.color = value;
            break;
          case "backgroundColor":
            style.background = value;
            break;
          case "currency":
            if (ws.data[key].value) {
              const num = parseFloat(ws.data[key].value);
              if (!isNaN(num)) ws.data[key].value = "$" + num.toFixed(2);
            }
            break;
          case "percent":
            if (ws.data[key].value) {
              const num = parseFloat(ws.data[key].value);
              if (!isNaN(num))
                ws.data[key].value = (num * 100).toFixed(0) + "%";
            }
            break;
        }

        renderCell(r, c);
      }
    }

    state.isDirty = true;
    scheduleAutoSave();
  }

  function mergeCells() {
    const { start, end } = state.selection;
    if (start.row === end.row && start.col === end.col) {
      addChatMessage("assistant", "Select multiple cells to merge.");
      return;
    }

    saveToHistory();
    const ws = state.worksheets[state.activeWorksheet];

    const firstKey = `${start.row},${start.col}`;
    let mergedValue = "";
    for (let r = start.row; r <= end.row; r++) {
      for (let c = start.col; c <= end.col; c++) {
        const key = `${r},${c}`;
        const cellData = ws.data[key];
        if (cellData?.value && !mergedValue) {
          mergedValue = cellData.value;
        }
        if (r !== start.row || c !== start.col) {
          delete ws.data[key];
        }
      }
    }

    if (!ws.data[firstKey]) ws.data[firstKey] = {};
    ws.data[firstKey].value = mergedValue;
    ws.data[firstKey].merged = {
      rowSpan: end.row - start.row + 1,
      colSpan: end.col - start.col + 1,
    };

    renderAllCells();
    state.isDirty = true;
    scheduleAutoSave();
    addChatMessage("assistant", "Cells merged successfully!");
  }

  function saveToHistory() {
    const snapshot = JSON.stringify(state.worksheets);
    state.history = state.history.slice(0, state.historyIndex + 1);
    state.history.push(snapshot);
    if (state.history.length > CONFIG.MAX_HISTORY) state.history.shift();
    state.historyIndex = state.history.length - 1;
  }

  function undo() {
    if (state.historyIndex > 0) {
      state.historyIndex--;
      state.worksheets = JSON.parse(state.history[state.historyIndex]);
      renderAllCells();
      state.isDirty = true;
    }
  }

  function redo() {
    if (state.historyIndex < state.history.length - 1) {
      state.historyIndex++;
      state.worksheets = JSON.parse(state.history[state.historyIndex]);
      renderAllCells();
      state.isDirty = true;
    }
  }

  function handleContextMenu(e) {
    const cell = e.target.closest(".cell");
    if (!cell) return;

    e.preventDefault();
    elements.contextMenu.style.left = e.clientX + "px";
    elements.contextMenu.style.top = e.clientY + "px";
    elements.contextMenu.classList.remove("hidden");
  }

  function handleDocumentClick(e) {
    if (!e.target.closest(".context-menu")) {
      elements.contextMenu?.classList.add("hidden");
    }
  }

  function handleContextAction(action) {
    elements.contextMenu.classList.add("hidden");

    switch (action) {
      case "cut":
        cutSelection();
        break;
      case "copy":
        copySelection();
        break;
      case "paste":
        pasteSelection();
        break;
      case "insertRowAbove":
        insertRow(state.activeCell.row);
        break;
      case "insertRowBelow":
        insertRow(state.activeCell.row + 1);
        break;
      case "insertColLeft":
        insertColumn(state.activeCell.col);
        break;
      case "insertColRight":
        insertColumn(state.activeCell.col + 1);
        break;
      case "deleteRow":
        deleteRow(state.activeCell.row);
        break;
      case "deleteCol":
        deleteColumn(state.activeCell.col);
        break;
      case "clearContents":
        clearCells();
        break;
      case "clearFormatting":
        clearFormatting();
        break;
    }
  }

  function insertRow(atRow) {
    saveToHistory();
    const ws = state.worksheets[state.activeWorksheet];
    const newData = {};

    for (const key in ws.data) {
      const [r, c] = key.split(",").map(Number);
      if (r >= atRow) {
        newData[`${r + 1},${c}`] = ws.data[key];
      } else {
        newData[key] = ws.data[key];
      }
    }

    ws.data = newData;
    renderAllCells();
    state.isDirty = true;
    scheduleAutoSave();
  }

  function insertColumn(atCol) {
    saveToHistory();
    const ws = state.worksheets[state.activeWorksheet];
    const newData = {};

    for (const key in ws.data) {
      const [r, c] = key.split(",").map(Number);
      if (c >= atCol) {
        newData[`${r},${c + 1}`] = ws.data[key];
      } else {
        newData[key] = ws.data[key];
      }
    }

    ws.data = newData;
    renderAllCells();
    state.isDirty = true;
    scheduleAutoSave();
  }

  function deleteRow(row) {
    saveToHistory();
    const ws = state.worksheets[state.activeWorksheet];
    const newData = {};

    for (const key in ws.data) {
      const [r, c] = key.split(",").map(Number);
      if (r < row) {
        newData[key] = ws.data[key];
      } else if (r > row) {
        newData[`${r - 1},${c}`] = ws.data[key];
      }
    }

    ws.data = newData;
    renderAllCells();
    state.isDirty = true;
    scheduleAutoSave();
  }

  function deleteColumn(col) {
    saveToHistory();
    const ws = state.worksheets[state.activeWorksheet];
    const newData = {};

    for (const key in ws.data) {
      const [r, c] = key.split(",").map(Number);
      if (c < col) {
        newData[key] = ws.data[key];
      } else if (c > col) {
        newData[`${r},${c - 1}`] = ws.data[key];
      }
    }

    ws.data = newData;
    renderAllCells();
    state.isDirty = true;
    scheduleAutoSave();
  }

  function clearFormatting() {
    const { start, end } = state.selection;
    const ws = state.worksheets[state.activeWorksheet];

    for (let r = start.row; r <= end.row; r++) {
      for (let c = start.col; c <= end.col; c++) {
        const key = `${r},${c}`;
        if (ws.data[key]) {
          delete ws.data[key].style;
          renderCell(r, c);
        }
      }
    }

    state.isDirty = true;
    scheduleAutoSave();
  }

  function addWorksheet() {
    const num = state.worksheets.length + 1;
    state.worksheets.push({ name: `Sheet${num}`, data: {} });
    state.activeWorksheet = state.worksheets.length - 1;
    renderWorksheetTabs();
    renderAllCells();
    selectCell(0, 0);
    state.isDirty = true;
    scheduleAutoSave();
  }

  function switchWorksheet(index) {
    if (index < 0 || index >= state.worksheets.length) return;
    state.activeWorksheet = index;
    renderWorksheetTabs();
    renderAllCells();
    selectCell(0, 0);
  }

  function renderWorksheetTabs() {
    elements.worksheetTabs.innerHTML = state.worksheets
      .map(
        (ws, i) => `
                <div class="sheet-tab ${i === state.activeWorksheet ? "active" : ""}" data-index="${i}">
                    <span>${escapeHtml(ws.name)}</span>
                    <button class="tab-menu-btn">▼</button>
                </div>
            `,
      )
      .join("");

    elements.worksheetTabs.querySelectorAll(".sheet-tab").forEach((tab) => {
      tab.addEventListener("click", () =>
        switchWorksheet(parseInt(tab.dataset.index)),
      );
    });
  }

  function zoomIn() {
    state.zoom = Math.min(200, state.zoom + 10);
    applyZoom();
  }

  function zoomOut() {
    state.zoom = Math.max(50, state.zoom - 10);
    applyZoom();
  }

  function applyZoom() {
    const scale = state.zoom / 100;
    elements.cells.style.transform = `scale(${scale})`;
    elements.cells.style.transformOrigin = "top left";
    elements.zoomLevel.textContent = state.zoom + "%";
  }

  function showModal(id) {
    document.getElementById(id)?.classList.remove("hidden");
  }

  function hideModal(id) {
    document.getElementById(id)?.classList.add("hidden");
  }

  function showShareModal() {
    const link = document.getElementById("shareLink");
    if (link) link.value = window.location.href;
    showModal("shareModal");
  }

  function copyShareLink() {
    const input = document.getElementById("shareLink");
    if (input) {
      navigator.clipboard.writeText(input.value);
    }
  }

  function scheduleAutoSave() {
    if (state.autoSaveTimer) clearTimeout(state.autoSaveTimer);
    state.autoSaveTimer = setTimeout(() => {
      if (state.isDirty) saveSheet();
    }, CONFIG.AUTOSAVE_DELAY);
  }

  async function saveSheet() {
    elements.saveStatus.textContent = "Saving...";

    try {
      const response = await fetch("/api/sheet/save", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          id: state.sheetId,
          name: state.sheetName,
          worksheets: state.worksheets,
        }),
      });

      if (response.ok) {
        const result = await response.json();
        if (result.id) {
          state.sheetId = result.id;
          window.history.replaceState({}, "", `#id=${state.sheetId}`);
        }
        state.isDirty = false;
        elements.saveStatus.textContent = "Saved";
      } else {
        elements.saveStatus.textContent = "Save failed";
      }
    } catch (e) {
      elements.saveStatus.textContent = "Save failed";
    }
  }

  async function importXlsx(file) {
    elements.saveStatus.textContent = "Importing...";
    
    const formData = new FormData();
    formData.append('file', file);
    
    try {
      const response = await fetch('/api/sheet/import', {
        method: 'POST',
        body: formData
      });
      
      if (response.ok) {
        const data = await response.json();
        state.sheetId = data.id;
        state.sheetName = data.name || file.name.replace(/\.[^/.]+$/, '');
        state.worksheets = data.worksheets || [{ name: "Sheet1", data: {} }];
        
        if (elements.sheetName) elements.sheetName.value = state.sheetName;
        
        CONFIG.ROWS = Math.max(CONFIG.ROWS, state.worksheets.reduce((max, ws) => {
          const maxRow = Object.keys(ws.data || {}).reduce((m, key) => {
            const [r] = key.split(',').map(Number);
            return Math.max(m, r);
          }, 0);
          return Math.max(max, maxRow + 1);
        }, CONFIG.ROWS));
        
        window.history.replaceState({}, "", `#id=${state.sheetId}`);
        
        renderWorksheetTabs();
        renderGrid();
        
        elements.saveStatus.textContent = "Imported";
        addChatMessage("system", `Successfully imported ${file.name}`);
      } else {
        const err = await response.json();
        elements.saveStatus.textContent = "Import failed";
        addChatMessage("error", `Import failed: ${err.error || 'Unknown error'}`);
      }
    } catch (e) {
      elements.saveStatus.textContent = "Import failed";
      addChatMessage("error", `Import failed: ${e.message}`);
    }
  }

  async function exportXlsx() {
    elements.saveStatus.textContent = "Exporting...";
    
    try {
      if (!state.sheetId) {
        const response = await saveSheet();
        if (!response.ok) throw new Error('Failed to save first');
      }
      
      const response = await fetch('/api/sheet/export', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: state.sheetId,
          format: 'xlsx'
        })
      });
      
      if (response.ok) {
        const blob = await response.blob();
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `${state.sheetName || 'spreadsheet'}.xlsx`;
        a.click();
        URL.revokeObjectURL(url);
        
        elements.saveStatus.textContent = "Exported";
        addChatMessage("system", "Spreadsheet exported successfully");
      } else {
        const err = await response.json();
        elements.saveStatus.textContent = "Export failed";
        addChatMessage("error", `Export failed: ${err.error || 'Unknown error'}`);
      }
    } catch (e) {
      elements.saveStatus.textContent = "Export failed";
      addChatMessage("error", `Export failed: ${e.message}`);
    }
  }

  async function exportCsv() {
    elements.saveStatus.textContent = "Exporting...";
    
    try {
      if (!state.sheetId) {
        await saveSheet();
      }
      
      const response = await fetch('/api/sheet/export', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: state.sheetId,
          format: 'csv'
        })
      });
      
      if (response.ok) {
        const blob = await response.blob();
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `${state.sheetName || 'spreadsheet'}.csv`;
        a.click();
        URL.revokeObjectURL(url);
        
        elements.saveStatus.textContent = "Exported";
      } else {
        elements.saveStatus.textContent = "Export failed";
      }
    } catch (e) {
      elements.saveStatus.textContent = "Export failed";
    }
  }

  async function loadFromUrlParams() {
    const hash = window.location.hash;
    if (!hash) return;

    const params = new URLSearchParams(hash.substring(1));
    const sheetId = params.get("id");

    if (sheetId) {
      try {
        const response = await fetch(`/api/sheet/${sheetId}`);
        if (response.ok) {
          const data = await response.json();
          state.sheetId = sheetId;
          state.sheetName = data.name || "Untitled Spreadsheet";
          state.worksheets = data.worksheets || [{ name: "Sheet1", data: {} }];

          if (elements.sheetName) elements.sheetName.value = state.sheetName;

          renderWorksheetTabs();
          renderAllCells();
        }
      } catch (e) {
        console.error("Load failed:", e);
      }
    }
  }

  function handleBeforeUnload(e) {
    if (state.isDirty) {
      e.preventDefault();
      e.returnValue = "";
    }
  }

  function connectWebSocket() {
    if (!state.sheetId) return;

    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const wsUrl = `${protocol}//${window.location.host}/api/sheet/ws/${state.sheetId}`;

    try {
      state.ws = new WebSocket(wsUrl);

      state.ws.onopen = () => {
        state.ws.send(
          JSON.stringify({
            type: "join",
            sheetId: state.sheetId,
            userId: getUserId(),
            userName: getUserName(),
          }),
        );
      };

      state.ws.onmessage = (event) => {
        const msg = JSON.parse(event.data);
        handleWebSocketMessage(msg);
      };

      state.ws.onclose = () => {
        setTimeout(connectWebSocket, CONFIG.WS_RECONNECT_DELAY);
      };
    } catch (e) {
      console.error("WebSocket failed:", e);
    }
  }

  function handleWebSocketMessage(msg) {
    switch (msg.type) {
      case "cellChange":
        if (msg.userId !== getUserId()) {
          const ws = state.worksheets[state.activeWorksheet];
          const key = `${msg.row},${msg.col}`;
          if (msg.value) {
            ws.data[key] = { value: msg.value };
          } else {
            delete ws.data[key];
          }
          renderCell(msg.row, msg.col);
        }
        break;
      case "cursor":
        updateRemoteCursor(msg);
        break;
      case "userJoined":
        addCollaborator(msg.user);
        break;
      case "userLeft":
        removeCollaborator(msg.userId);
        break;
    }
  }

  function broadcastChange(type, data) {
    if (state.ws?.readyState === WebSocket.OPEN) {
      state.ws.send(
        JSON.stringify({
          type,
          sheetId: state.sheetId,
          userId: getUserId(),
          ...data,
        }),
      );
    }
  }

  function updateRemoteCursor(msg) {
    let cursor = document.getElementById(`cursor-${msg.userId}`);
    if (!cursor) {
      cursor = document.createElement("div");
      cursor.id = `cursor-${msg.userId}`;
      cursor.className = "cursor-indicator";
      cursor.style.borderColor = msg.color || "#4285f4";
      cursor.innerHTML = `<div class="cursor-label" style="background:${msg.color || "#4285f4"}">${escapeHtml(msg.userName)}</div>`;
      elements.cursorIndicators?.appendChild(cursor);
    }

    const cell = elements.cells.querySelector(
      `[data-row="${msg.row}"][data-col="${msg.col}"]`,
    );
    if (cell) {
      const rect = cell.getBoundingClientRect();
      const container = elements.cellsContainer.getBoundingClientRect();
      cursor.style.left = rect.left - container.left + "px";
      cursor.style.top = rect.top - container.top + "px";
      cursor.style.width = rect.width + "px";
      cursor.style.height = rect.height + "px";
    }
  }

  function addCollaborator(user) {
    if (!state.collaborators.find((u) => u.id === user.id)) {
      state.collaborators.push(user);
      renderCollaborators();
    }
  }

  function removeCollaborator(userId) {
    state.collaborators = state.collaborators.filter((u) => u.id !== userId);
    document.getElementById(`cursor-${userId}`)?.remove();
    renderCollaborators();
  }

  function renderCollaborators() {
    elements.collaborators.innerHTML = state.collaborators
      .slice(0, 4)
      .map(
        (u) => `
                <div class="collaborator-avatar" style="background:${u.color || "#4285f4"}" title="${escapeHtml(u.name)}">
                    ${u.name.charAt(0).toUpperCase()}
                </div>
            `,
      )
      .join("");
  }

  function getUserId() {
    let id = localStorage.getItem("gb-user-id");
    if (!id) {
      id = "user-" + Math.random().toString(36).substr(2, 9);
      localStorage.setItem("gb-user-id", id);
    }
    return id;
  }

  function getUserName() {
    return localStorage.getItem("gb-user-name") || "Anonymous";
  }

  function escapeHtml(str) {
    if (!str) return "";
    return String(str)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
  }

  function toggleChatPanel() {
    state.chatPanelOpen = !state.chatPanelOpen;
    elements.chatPanel.classList.toggle("collapsed", !state.chatPanelOpen);
  }

  function handleChatSubmit(e) {
    e.preventDefault();
    const message = elements.chatInput.value.trim();
    if (!message) return;

    addChatMessage("user", message);
    elements.chatInput.value = "";

    processAICommand(message);
  }

  function handleSuggestionClick(action) {
    const commands = {
      sum: "Sum column B",
      format: "Format selected cells as currency",
      chart: "Create a bar chart from selected data",
      sort: "Sort selected column A to Z",
    };

    const message = commands[action] || action;
    addChatMessage("user", message);
    processAICommand(message);
  }

  function addChatMessage(role, content) {
    const div = document.createElement("div");
    div.className = `chat-message ${role}`;
    div.innerHTML = `<div class="message-bubble">${escapeHtml(content)}</div>`;
    elements.chatMessages.appendChild(div);
    elements.chatMessages.scrollTop = elements.chatMessages.scrollHeight;
  }

  async function processAICommand(command) {
    const lower = command.toLowerCase();
    let response = "";

    if (lower.includes("sum")) {
      const { start, end } = state.selection;
      const colLetter = getColName(start.col);
      const formula = `=SUM(${colLetter}${start.row + 1}:${colLetter}${end.row + 1})`;

      const resultRow = end.row + 1;
      if (resultRow < CONFIG.ROWS) {
        setCellValue(resultRow, start.col, formula);
        renderCell(resultRow, start.col);
        selectCell(resultRow, start.col);
        response = `Done! Added SUM formula in cell ${getColName(start.col)}${resultRow + 1}`;
      } else {
        response = "Cannot add sum - no row available below selection";
      }
    } else if (lower.includes("currency") || lower.includes("$")) {
      formatCells("currency");
      response = "Formatted selected cells as currency";
    } else if (lower.includes("percent") || lower.includes("%")) {
      formatCells("percent");
      response = "Formatted selected cells as percentage";
    } else if (lower.includes("bold")) {
      formatCells("bold");
      response = "Applied bold formatting to selected cells";
    } else if (lower.includes("italic")) {
      formatCells("italic");
      response = "Applied italic formatting to selected cells";
    } else if (lower.includes("sort") && lower.includes("z")) {
      sortDescending();
      response = "Sorted selection Z to A";
    } else if (lower.includes("sort")) {
      sortAscending();
      response = "Sorted selection A to Z";
    } else if (lower.includes("chart")) {
      showModal("chartModal");
      response =
        "Opening chart dialog. Select chart type and configure options.";
    } else if (lower.includes("clear")) {
      clearCells();
      response = "Cleared selected cells";
    } else if (lower.includes("average") || lower.includes("avg")) {
      const { start, end } = state.selection;
      const colLetter = getColName(start.col);
      const formula = `=AVERAGE(${colLetter}${start.row + 1}:${colLetter}${end.row + 1})`;
      const resultRow = end.row + 1;
      if (resultRow < CONFIG.ROWS) {
        setCellValue(resultRow, start.col, formula);
        renderCell(resultRow, start.col);
        selectCell(resultRow, start.col);
        response = `Done! Added AVERAGE formula in cell ${getColName(start.col)}${resultRow + 1}`;
      }
    } else {
      try {
        const res = await fetch("/api/sheet/ai", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            command,
            selection: state.selection,
            activeCell: state.activeCell,
            sheetId: state.sheetId,
          }),
        });
        const data = await res.json();
        response = data.response || "I processed your request";
      } catch {
        response =
          "I can help you with:\n• Sum/Average a column\n• Format as currency or percent\n• Bold/Italic formatting\n• Sort data\n• Create charts\n• Clear cells";
      }
    }

    addChatMessage("assistant", response);
  }

  function sortAscending() {
    sortSelection(true);
  }

  function sortDescending() {
    sortSelection(false);
  }

  function sortSelection(ascending) {
    saveToHistory();
    const { start, end } = state.selection;
    const ws = state.worksheets[state.activeWorksheet];

    const rows = [];
    for (let r = start.row; r <= end.row; r++) {
      const rowData = [];
      for (let c = start.col; c <= end.col; c++) {
        rowData.push(getCellData(r, c));
      }
      rows.push({ row: r, data: rowData });
    }

    rows.sort((a, b) => {
      const valA = a.data[0]?.value || a.data[0]?.formula || "";
      const valB = b.data[0]?.value || b.data[0]?.formula || "";
      const numA = parseFloat(valA);
      const numB = parseFloat(valB);

      if (!isNaN(numA) && !isNaN(numB)) {
        return ascending ? numA - numB : numB - numA;
      }
      return ascending
        ? String(valA).localeCompare(String(valB))
        : String(valB).localeCompare(String(valA));
    });

    rows.forEach((rowObj, i) => {
      const targetRow = start.row + i;
      rowObj.data.forEach((cellData, j) => {
        const targetCol = start.col + j;
        const key = `${targetRow},${targetCol}`;
        if (cellData) {
          ws.data[key] = cellData;
        } else {
          delete ws.data[key];
        }
      });
    });

    renderAllCells();
    state.isDirty = true;
    scheduleAutoSave();
  }

  function connectChatWebSocket() {
    // Chat uses main WebSocket connection
  }

  function handleNumberFormatChange(e) {
    const format = e.target.value;
    if (format === "custom") {
      showModal("customNumberFormatModal");
      return;
    }
    applyNumberFormat(format);
  }

  function applyNumberFormat(format) {
    saveToHistory();
    const { start, end } = state.selection;
    const ws = state.worksheets[state.activeWorksheet];

    for (let r = start.row; r <= end.row; r++) {
      for (let c = start.col; c <= end.col; c++) {
        const key = `${r},${c}`;
        if (!ws.data[key]) ws.data[key] = { value: "" };
        ws.data[key].format = format;

        const rawValue = ws.data[key].rawValue || ws.data[key].value;
        if (rawValue) {
          ws.data[key].rawValue = rawValue;
          ws.data[key].value = formatValue(rawValue, format);
        }
        renderCell(r, c);
      }
    }

    state.isDirty = true;
    scheduleAutoSave();
  }

  function formatValue(value, format) {
    const num = parseFloat(value);
    if (isNaN(num) && format !== "text") return value;

    switch (format) {
      case "number":
        return num.toLocaleString("en-US", {
          minimumFractionDigits: state.decimalPlaces,
          maximumFractionDigits: state.decimalPlaces,
        });
      case "currency":
        return num.toLocaleString("en-US", {
          style: "currency",
          currency: "USD",
          minimumFractionDigits: state.decimalPlaces,
        });
      case "accounting":
        const formatted = Math.abs(num).toLocaleString("en-US", {
          style: "currency",
          currency: "USD",
        });
        return num < 0 ? `(${formatted})` : formatted;
      case "percent":
        return (num * 100).toFixed(state.decimalPlaces) + "%";
      case "scientific":
        return num.toExponential(state.decimalPlaces);
      case "date_short":
        const d1 = new Date(num);
        return isNaN(d1.getTime()) ? value : d1.toLocaleDateString("en-US");
      case "date_long":
        const d2 = new Date(num);
        return isNaN(d2.getTime())
          ? value
          : d2.toLocaleDateString("en-US", {
            year: "numeric",
            month: "long",
            day: "numeric",
          });
      case "time":
        const d3 = new Date(num);
        return isNaN(d3.getTime())
          ? value
          : d3.toLocaleTimeString("en-US", {
            hour: "numeric",
            minute: "2-digit",
          });
      case "datetime":
        const d4 = new Date(num);
        return isNaN(d4.getTime()) ? value : d4.toLocaleString("en-US");
      case "fraction":
        return toFraction(num);
      case "text":
        return String(value);
      default:
        return value;
    }
  }

  function toFraction(decimal) {
    const tolerance = 1e-6;
    let h1 = 1,
      h2 = 0,
      k1 = 0,
      k2 = 1;
    let b = decimal;
    do {
      const a = Math.floor(b);
      let aux = h1;
      h1 = a * h1 + h2;
      h2 = aux;
      aux = k1;
      k1 = a * k1 + k2;
      k2 = aux;
      b = 1 / (b - a);
    } while (Math.abs(decimal - h1 / k1) > decimal * tolerance);

    if (k1 === 1) return String(h1);
    const whole = Math.floor(h1 / k1);
    const remainder = h1 % k1;
    if (whole === 0) return `${remainder}/${k1}`;
    return `${whole} ${remainder}/${k1}`;
  }

  function decreaseDecimal() {
    if (state.decimalPlaces > 0) {
      state.decimalPlaces--;
      reapplyFormats();
    }
  }

  function increaseDecimal() {
    if (state.decimalPlaces < 10) {
      state.decimalPlaces++;
      reapplyFormats();
    }
  }

  function reapplyFormats() {
    const { start, end } = state.selection;
    const ws = state.worksheets[state.activeWorksheet];

    for (let r = start.row; r <= end.row; r++) {
      for (let c = start.col; c <= end.col; c++) {
        const key = `${r},${c}`;
        const cellData = ws.data[key];
        if (cellData?.format && cellData?.rawValue) {
          cellData.value = formatValue(cellData.rawValue, cellData.format);
          renderCell(r, c);
        }
      }
    }
  }

  function showFindReplaceModal() {
    showModal("findReplaceModal");
    document.getElementById("findInput")?.focus();
    state.findMatches = [];
    state.findMatchIndex = -1;
  }

  function performFind() {
    const searchText = document.getElementById("findInput")?.value || "";
    const matchCase = document.getElementById("findMatchCase")?.checked;
    const wholeCell = document.getElementById("findWholeCell")?.checked;
    const useRegex = document.getElementById("findRegex")?.checked;

    state.findMatches = [];
    state.findMatchIndex = -1;

    if (!searchText) {
      updateFindResults();
      return;
    }

    const ws = state.worksheets[state.activeWorksheet];
    let pattern;

    if (useRegex) {
      try {
        pattern = new RegExp(searchText, matchCase ? "" : "i");
      } catch (e) {
        updateFindResults();
        return;
      }
    }

    for (let r = 0; r < CONFIG.ROWS; r++) {
      for (let c = 0; c < CONFIG.COLS; c++) {
        const key = `${r},${c}`;
        const cellData = ws.data[key];
        const cellValue = cellData?.value || "";

        if (!cellValue) continue;

        let matches = false;
        const compareValue = matchCase ? cellValue : cellValue.toLowerCase();
        const compareSearch = matchCase ? searchText : searchText.toLowerCase();

        if (useRegex) {
          matches = pattern.test(cellValue);
        } else if (wholeCell) {
          matches = compareValue === compareSearch;
        } else {
          matches = compareValue.includes(compareSearch);
        }

        if (matches) {
          state.findMatches.push({ row: r, col: c });
        }
      }
    }

    updateFindResults();
    if (state.findMatches.length > 0) {
      state.findMatchIndex = 0;
      highlightFindMatch();
    }
  }

  function updateFindResults() {
    const resultsEl = document.getElementById("findResults");
    if (resultsEl) {
      const count = state.findMatches.length;
      resultsEl.querySelector("span").textContent =
        count === 0
          ? "0 matches found"
          : `${state.findMatchIndex + 1} of ${count} matches`;
    }
  }

  function highlightFindMatch() {
    if (state.findMatches.length === 0) return;
    const match = state.findMatches[state.findMatchIndex];
    selectCell(match.row, match.col);
    updateFindResults();
  }

  function findNext() {
    if (state.findMatches.length === 0) return;
    state.findMatchIndex =
      (state.findMatchIndex + 1) % state.findMatches.length;
    highlightFindMatch();
  }

  function findPrev() {
    if (state.findMatches.length === 0) return;
    state.findMatchIndex =
      (state.findMatchIndex - 1 + state.findMatches.length) %
      state.findMatches.length;
    highlightFindMatch();
  }

  function replaceOne() {
    if (state.findMatches.length === 0 || state.findMatchIndex < 0) return;

    const replaceText = document.getElementById("replaceInput")?.value || "";
    const match = state.findMatches[state.findMatchIndex];
    const ws = state.worksheets[state.activeWorksheet];
    const key = `${match.row},${match.col}`;

    saveToHistory();

    const searchText = document.getElementById("findInput")?.value || "";
    const matchCase = document.getElementById("findMatchCase")?.checked;
    const useRegex = document.getElementById("findRegex")?.checked;
    const cellValue = ws.data[key]?.value || "";

    let newValue;
    if (useRegex) {
      const pattern = new RegExp(searchText, matchCase ? "g" : "gi");
      newValue = cellValue.replace(pattern, replaceText);
    } else {
      const flags = matchCase ? "g" : "gi";
      const escapedSearch = searchText.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      newValue = cellValue.replace(
        new RegExp(escapedSearch, flags),
        replaceText,
      );
    }

    if (!ws.data[key]) ws.data[key] = {};
    ws.data[key].value = newValue;
    renderCell(match.row, match.col);

    state.findMatches.splice(state.findMatchIndex, 1);
    if (state.findMatches.length > 0) {
      state.findMatchIndex = state.findMatchIndex % state.findMatches.length;
      highlightFindMatch();
    } else {
      state.findMatchIndex = -1;
      updateFindResults();
    }

    state.isDirty = true;
    scheduleAutoSave();
  }

  function replaceAll() {
    if (state.findMatches.length === 0) return;

    const replaceText = document.getElementById("replaceInput")?.value || "";
    const searchText = document.getElementById("findInput")?.value || "";
    const matchCase = document.getElementById("findMatchCase")?.checked;
    const useRegex = document.getElementById("findRegex")?.checked;
    const ws = state.worksheets[state.activeWorksheet];

    saveToHistory();

    let count = 0;
    for (const match of state.findMatches) {
      const key = `${match.row},${match.col}`;
      const cellValue = ws.data[key]?.value || "";

      let newValue;
      if (useRegex) {
        const pattern = new RegExp(searchText, matchCase ? "g" : "gi");
        newValue = cellValue.replace(pattern, replaceText);
      } else {
        const flags = matchCase ? "g" : "gi";
        const escapedSearch = searchText.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
        newValue = cellValue.replace(
          new RegExp(escapedSearch, flags),
          replaceText,
        );
      }

      if (!ws.data[key]) ws.data[key] = {};
      ws.data[key].value = newValue;
      renderCell(match.row, match.col);
      count++;
    }

    state.findMatches = [];
    state.findMatchIndex = -1;
    updateFindResults();

    state.isDirty = true;
    scheduleAutoSave();
    addChatMessage("assistant", `Replaced ${count} occurrences.`);
  }

  function showConditionalFormatModal() {
    const { start, end } = state.selection;
    const range = `${getColName(start.col)}${start.row + 1}:${getColName(end.col)}${end.row + 1}`;
    const rangeInput = document.getElementById("cfRange");
    if (rangeInput) rangeInput.value = range;
    showModal("conditionalFormatModal");
    handleCfRuleTypeChange();
    updateCfPreview();
  }

  function handleCfRuleTypeChange() {
    const ruleType = document.getElementById("cfRuleType")?.value;
    const value2 = document.getElementById("cfValue2");
    const valuesSection = document.getElementById("cfValuesSection");

    if (value2) {
      if (ruleType === "between") {
        value2.classList.remove("hidden");
        value2.placeholder = "and";
      } else {
        value2.classList.add("hidden");
      }
    }

    const noValueTypes = [
      "duplicate",
      "unique",
      "blank",
      "not_blank",
      "above_average",
      "below_average",
      "color_scale",
      "data_bar",
      "icon_set",
    ];
    if (valuesSection) {
      if (noValueTypes.includes(ruleType)) {
        valuesSection.style.display = "none";
      } else {
        valuesSection.style.display = "flex";
      }
    }
  }

  function updateCfPreview() {
    const bgColor = document.getElementById("cfBgColor")?.value || "#ffeb3b";
    const textColor =
      document.getElementById("cfTextColor")?.value || "#000000";
    const bold = document.getElementById("cfBold")?.checked;
    const italic = document.getElementById("cfItalic")?.checked;

    const previewCell = document.getElementById("cfPreviewCell");
    if (previewCell) {
      previewCell.style.background = bgColor;
      previewCell.style.color = textColor;
      previewCell.style.fontWeight = bold ? "bold" : "normal";
      previewCell.style.fontStyle = italic ? "italic" : "normal";
    }
  }

  function applyConditionalFormat() {
    const rangeStr = document.getElementById("cfRange")?.value;
    if (!rangeStr) {
      alert("Please specify a range.");
      return;
    }

    const ruleType = document.getElementById("cfRuleType")?.value;
    const value1 = document.getElementById("cfValue1")?.value;
    const value2 = document.getElementById("cfValue2")?.value;
    const bgColor = document.getElementById("cfBgColor")?.value;
    const textColor = document.getElementById("cfTextColor")?.value;
    const bold = document.getElementById("cfBold")?.checked;
    const italic = document.getElementById("cfItalic")?.checked;

    const ws = state.worksheets[state.activeWorksheet];
    if (!ws.conditionalFormats) ws.conditionalFormats = [];

    const rule = {
      id: `cf_${Date.now()}`,
      range: rangeStr,
      ruleType,
      value1,
      value2,
      style: {
        background: bgColor,
        color: textColor,
        fontWeight: bold ? "bold" : "normal",
        fontStyle: italic ? "italic" : "normal",
      },
    };

    ws.conditionalFormats.push(rule);
    applyConditionalFormatsToRange(rule);

    hideModal("conditionalFormatModal");
    state.isDirty = true;
    scheduleAutoSave();
    addChatMessage("assistant", "Conditional formatting applied!");
  }

  function applyConditionalFormatsToRange(rule) {
    const ws = state.worksheets[state.activeWorksheet];
    const rangeParts = rule.range.split(":");
    if (rangeParts.length !== 2) return;

    const startRef = parseCellRef(rangeParts[0]);
    const endRef = parseCellRef(rangeParts[1]);
    if (!startRef || !endRef) return;

    for (let r = startRef.row; r <= endRef.row; r++) {
      for (let c = startRef.col; c <= endRef.col; c++) {
        const key = `${r},${c}`;
        const cellData = ws.data[key];
        const cellValue = parseFloat(cellData?.value) || 0;

        let conditionMet = false;
        switch (rule.ruleType) {
          case "greater_than":
            conditionMet = cellValue > parseFloat(rule.value1);
            break;
          case "less_than":
            conditionMet = cellValue < parseFloat(rule.value1);
            break;
          case "equal_to":
            conditionMet = cellValue === parseFloat(rule.value1);
            break;
          case "between":
            conditionMet =
              cellValue >= parseFloat(rule.value1) &&
              cellValue <= parseFloat(rule.value2);
            break;
          case "text_contains":
            conditionMet = (cellData?.value || "")
              .toLowerCase()
              .includes(rule.value1.toLowerCase());
            break;
          case "blank":
            conditionMet = !cellData?.value;
            break;
          case "not_blank":
            conditionMet = !!cellData?.value;
            break;
          default:
            conditionMet = false;
        }

        if (conditionMet && cellData) {
          if (!cellData.style) cellData.style = {};
          Object.assign(cellData.style, rule.style);
          renderCell(r, c);
        }
      }
    }
  }

  function showDataValidationModal() {
    const { start, end } = state.selection;
    const range = `${getColName(start.col)}${start.row + 1}:${getColName(end.col)}${end.row + 1}`;
    const rangeInput = document.getElementById("dvRange");
    if (rangeInput) rangeInput.value = range;
    showModal("dataValidationModal");
    handleDvTypeChange();
  }

  function switchDvTab(tabName) {
    document.querySelectorAll(".dv-tab").forEach((tab) => {
      tab.classList.toggle("active", tab.dataset.tab === tabName);
    });
    document.querySelectorAll(".dv-tab-content").forEach((content) => {
      const contentId = content.id
        .replace("dv", "")
        .replace("Tab", "")
        .toLowerCase();
      content.classList.toggle("active", contentId === tabName);
    });
  }

  function handleDvTypeChange() {
    const dvType = document.getElementById("dvType")?.value;
    const criteriaSection = document.getElementById("dvCriteriaSection");
    const valuesSection = document.getElementById("dvValuesSection");
    const listSection = document.getElementById("dvListSection");
    const value2Row = document.getElementById("dvValue2Row");
    const value1Label = document.getElementById("dvValue1Label");

    if (criteriaSection) {
      criteriaSection.style.display =
        dvType === "any" || dvType === "list" || dvType === "custom"
          ? "none"
          : "block";
    }

    if (valuesSection) {
      valuesSection.style.display =
        dvType === "any" || dvType === "list" ? "none" : "block";
    }

    if (listSection) {
      listSection.classList.toggle("hidden", dvType !== "list");
    }

    if (value1Label) {
      value1Label.textContent = dvType === "custom" ? "Formula:" : "Minimum:";
    }
  }

  function handleDvOperatorChange() {
    const operator = document.getElementById("dvOperator")?.value;
    const value2Row = document.getElementById("dvValue2Row");
    const value1Label = document.getElementById("dvValue1Label");

    if (value2Row) {
      value2Row.style.display =
        operator === "between" || operator === "not_between" ? "block" : "none";
    }

    if (value1Label) {
      if (operator === "between" || operator === "not_between") {
        value1Label.textContent = "Minimum:";
      } else {
        value1Label.textContent = "Value:";
      }
    }
  }

  function applyDataValidation() {
    const rangeStr = document.getElementById("dvRange")?.value;
    if (!rangeStr) {
      alert("Please specify a range.");
      return;
    }

    const dvType = document.getElementById("dvType")?.value;
    const operator = document.getElementById("dvOperator")?.value;
    const value1 = document.getElementById("dvValue1")?.value;
    const value2 = document.getElementById("dvValue2")?.value;
    const listSource = document.getElementById("dvListSource")?.value;
    const showInput = document.getElementById("dvShowInput")?.checked;
    const inputTitle = document.getElementById("dvInputTitle")?.value;
    const inputMessage = document.getElementById("dvInputMessage")?.value;
    const showError = document.getElementById("dvShowError")?.checked;
    const errorStyle = document.getElementById("dvErrorStyle")?.value;
    const errorTitle = document.getElementById("dvErrorTitle")?.value;
    const errorMessage = document.getElementById("dvErrorMessage")?.value;

    const ws = state.worksheets[state.activeWorksheet];
    if (!ws.validations) ws.validations = {};

    const validation = {
      type: dvType,
      operator,
      value1,
      value2,
      listValues: listSource ? listSource.split(",").map((s) => s.trim()) : [],
      showInput,
      inputTitle,
      inputMessage,
      showError,
      errorStyle,
      errorTitle,
      errorMessage,
    };

    const rangeParts = rangeStr.split(":");
    const startRef = parseCellRef(rangeParts[0]);
    const endRef =
      rangeParts.length > 1 ? parseCellRef(rangeParts[1]) : startRef;

    if (startRef && endRef) {
      for (let r = startRef.row; r <= endRef.row; r++) {
        for (let c = startRef.col; c <= endRef.col; c++) {
          ws.validations[`${r},${c}`] = validation;
        }
      }
    }

    hideModal("dataValidationModal");
    state.isDirty = true;
    scheduleAutoSave();
    addChatMessage("assistant", "Data validation applied!");
  }

  function clearDataValidation() {
    const rangeStr = document.getElementById("dvRange")?.value;
    if (!rangeStr) return;

    const ws = state.worksheets[state.activeWorksheet];
    if (!ws.validations) return;

    const rangeParts = rangeStr.split(":");
    const startRef = parseCellRef(rangeParts[0]);
    const endRef =
      rangeParts.length > 1 ? parseCellRef(rangeParts[1]) : startRef;

    if (startRef && endRef) {
      for (let r = startRef.row; r <= endRef.row; r++) {
        for (let c = startRef.col; c <= endRef.col; c++) {
          delete ws.validations[`${r},${c}`];
        }
      }
    }

    hideModal("dataValidationModal");
    state.isDirty = true;
    scheduleAutoSave();
  }

  function showPrintPreview() {
    showModal("printPreviewModal");
    updatePrintPreview();
  }

  function updatePrintPreview() {
    const orientation =
      document.getElementById("printOrientation")?.value || "portrait";
    const showGridlines = document.getElementById("printGridlines")?.checked;
    const showHeaders = document.getElementById("printHeaders")?.checked;
    const printPage = document.getElementById("printPage");
    const printContent = document.getElementById("printContent");

    if (printPage) {
      printPage.className = `print-page ${orientation}`;
    }

    if (!printContent) return;

    const ws = state.worksheets[state.activeWorksheet];
    let html = "<table>";

    if (showHeaders) {
      html += "<thead><tr><th></th>";
      for (let c = 0; c < CONFIG.COLS; c++) {
        html += `<th>${getColName(c)}</th>`;
      }
      html += "</tr></thead>";
    }

    html += "<tbody>";
    let hasData = false;
    let maxRow = 0;
    let maxCol = 0;

    for (const key in ws.data) {
      if (ws.data[key]?.value) {
        hasData = true;
        const [r, c] = key.split(",").map(Number);
        maxRow = Math.max(maxRow, r);
        maxCol = Math.max(maxCol, c);
      }
    }

    if (!hasData) {
      maxRow = 10;
      maxCol = 5;
    }

    for (let r = 0; r <= maxRow; r++) {
      html += "<tr>";
      if (showHeaders) {
        html += `<th>${r + 1}</th>`;
      }
      for (let c = 0; c <= maxCol; c++) {
        const key = `${r},${c}`;
        const cellData = ws.data[key];
        const value = cellData?.value || "";
        const style = cellData?.style || {};
        let styleStr = "";

        if (style.fontWeight) styleStr += `font-weight:${style.fontWeight};`;
        if (style.fontStyle) styleStr += `font-style:${style.fontStyle};`;
        if (style.textAlign) styleStr += `text-align:${style.textAlign};`;
        if (style.color) styleStr += `color:${style.color};`;
        if (style.background) styleStr += `background:${style.background};`;

        const borderStyle = showGridlines ? "" : "border:none;";
        html += `<td style="${styleStr}${borderStyle}">${escapeHtml(value)}</td>`;
      }
      html += "</tr>";
    }

    html += "</tbody></table>";
    printContent.innerHTML = html;
  }

  function printSheet() {
    const printContent = document.getElementById("printContent")?.innerHTML;
    if (!printContent) return;

    const orientation =
      document.getElementById("printOrientation")?.value || "portrait";
    const printWindow = window.open("", "_blank");

    printWindow.document.write(`
      <!DOCTYPE html>
      <html>
      <head>
        <title>${state.sheetName}</title>
        <style>
          @page { size: ${orientation}; margin: 0.5in; }
          body { font-family: Arial, sans-serif; font-size: 10pt; }
          table { width: 100%; border-collapse: collapse; }
          td, th { border: 1px solid #ccc; padding: 4px 8px; text-align: left; }
          th { background: #f5f5f5; font-weight: 600; }
        </style>
      </head>
      <body>
        ${printContent}
      </body>
      </html>
    `);

    printWindow.document.close();
    printWindow.focus();
    setTimeout(() => {
      printWindow.print();
      printWindow.close();
    }, 250);

    hideModal("printPreviewModal");
  }

  function insertChart() {
    const chartType =
      document.querySelector(".chart-type-btn.active")?.dataset.type || "bar";
    const dataRange = document.getElementById("chartDataRange")?.value;
    const chartTitle = document.getElementById("chartTitle")?.value || "Chart";

    if (!dataRange) {
      alert("Please specify a data range.");
      return;
    }

    const ws = state.worksheets[state.activeWorksheet];
    if (!ws.charts) ws.charts = [];

    const chart = {
      id: `chart_${Date.now()}`,
      type: chartType,
      title: chartTitle,
      dataRange,
      position: {
        row: state.activeCell.row,
        col: state.activeCell.col,
        width: 400,
        height: 300,
      },
    };

    ws.charts.push(chart);
    hideModal("chartModal");
    state.isDirty = true;
    scheduleAutoSave();
    addChatMessage(
      "assistant",
      `${chartType.charAt(0).toUpperCase() + chartType.slice(1)} chart created!`,
    );
  }

  function showInsertImageModal() {
    showModal("insertImageModal");
  }

  function switchImgTab(tabName) {
    document.querySelectorAll(".img-tab").forEach((tab) => {
      tab.classList.toggle("active", tab.dataset.tab === tabName);
    });
    document.querySelectorAll(".img-tab-content").forEach((content) => {
      const contentId = content.id
        .replace("img", "")
        .replace("Tab", "")
        .toLowerCase();
      content.classList.toggle("active", contentId === tabName);
    });
  }

  function insertImage() {
    const urlTab = document.getElementById("imgUrlTab");
    const isUrlTab = urlTab?.classList.contains("active");
    let imageUrl;

    if (isUrlTab) {
      imageUrl = document.getElementById("imgUrl")?.value;
    } else {
      const fileInput = document.getElementById("imgFile");
      if (fileInput?.files?.[0]) {
        addChatMessage(
          "assistant",
          "Image upload coming soon! Please use a URL for now.",
        );
        return;
      }
    }

    if (!imageUrl) {
      alert("Please enter an image URL.");
      return;
    }

    const ws = state.worksheets[state.activeWorksheet];
    if (!ws.images) ws.images = [];

    const image = {
      id: `img_${Date.now()}`,
      url: imageUrl,
      position: {
        row: state.activeCell.row,
        col: state.activeCell.col,
        width: 200,
        height: 150,
      },
    };

    ws.images.push(image);
    hideModal("insertImageModal");
    state.isDirty = true;
    scheduleAutoSave();
    addChatMessage("assistant", "Image inserted!");
  }

  function toggleFilter() {
    const ws = state.worksheets[state.activeWorksheet];
    ws.filterEnabled = !ws.filterEnabled;
    addChatMessage(
      "assistant",
      ws.filterEnabled
        ? "Filter enabled. Click column headers to filter."
        : "Filter disabled.",
    );
  }

  function selectCustomFormat(formatCode) {
    document.querySelectorAll(".cnf-format-item").forEach((item) => {
      item.classList.toggle("selected", item.dataset.format === formatCode);
    });
    const formatInput = document.getElementById("cnfFormatCode");
    if (formatInput) {
      formatInput.value = formatCode;
    }
    updateCnfPreview();
  }

  function updateCnfPreview() {
    const formatCode =
      document.getElementById("cnfFormatCode")?.value || "#,##0.00";
    const previewEl = document.getElementById("cnfPreview");
    if (!previewEl) return;

    const sampleValue = 1234.5678;
    let formatted;

    if (formatCode.includes("$")) {
      formatted = sampleValue.toLocaleString("en-US", {
        style: "currency",
        currency: "USD",
      });
    } else if (formatCode.includes("%")) {
      formatted = (sampleValue * 100).toFixed(2) + "%";
    } else if (formatCode.includes("E")) {
      formatted = sampleValue.toExponential(2);
    } else if (formatCode.includes("MM") || formatCode.includes("DD")) {
      formatted = new Date().toLocaleDateString();
    } else if (formatCode.includes("HH")) {
      formatted = new Date().toLocaleTimeString();
    } else {
      const decimals = (formatCode.match(/0+$/)?.[0] || "").length;
      formatted = sampleValue.toLocaleString("en-US", {
        minimumFractionDigits: decimals,
        maximumFractionDigits: decimals,
      });
    }

    previewEl.textContent = formatted;
  }

  function applyCustomNumberFormat() {
    const formatCode = document.getElementById("cnfFormatCode")?.value;
    if (!formatCode) return;

    saveToHistory();
    const { start, end } = state.selection;
    const ws = state.worksheets[state.activeWorksheet];

    for (let r = start.row; r <= end.row; r++) {
      for (let c = start.col; c <= end.col; c++) {
        const key = `${r},${c}`;
        if (!ws.data[key]) ws.data[key] = { value: "" };
        ws.data[key].customFormat = formatCode;
        renderCell(r, c);
      }
    }

    hideModal("customNumberFormatModal");
    state.isDirty = true;
    scheduleAutoSave();
  }

  function renderCharts() {
    const chartsContainer = document.getElementById("chartsContainer");
    if (!chartsContainer) return;

    const ws = state.worksheets[state.activeWorksheet];
    if (!ws.charts || ws.charts.length === 0) {
      chartsContainer.innerHTML = "";
      return;
    }

    chartsContainer.innerHTML = ws.charts
      .map((chart) => renderChartHTML(chart))
      .join("");

    chartsContainer.querySelectorAll(".chart-wrapper").forEach((wrapper) => {
      const chartId = wrapper.dataset.chartId;
      wrapper.addEventListener("click", () => selectChart(chartId));
      wrapper.querySelector(".chart-delete")?.addEventListener("click", (e) => {
        e.stopPropagation();
        deleteChart(chartId);
      });
      wrapper
        .querySelector(".chart-header")
        ?.addEventListener("mousedown", (e) => {
          startDragChart(e, chartId);
        });
    });
  }

  function renderChartHTML(chart) {
    const { id, type, title, position, dataRange } = chart;
    const left = position?.col ? position.col * CONFIG.COL_WIDTH : 100;
    const top = position?.row ? position.row * CONFIG.ROW_HEIGHT : 100;
    const width = position?.width || 400;
    const height = position?.height || 300;

    const data = getChartData(dataRange);
    let chartContent = "";

    switch (type) {
      case "bar":
        chartContent = renderBarChart(data, height - 80);
        break;
      case "line":
        chartContent = renderLineChart(data, width - 32, height - 80);
        break;
      case "pie":
        chartContent = renderPieChart(data, Math.min(width, height) - 100);
        break;
      default:
        chartContent = renderBarChart(data, height - 80);
    }

    return `
      <div class="chart-wrapper" data-chart-id="${id}" style="left:${left}px;top:${top}px;width:${width}px;height:${height}px;">
        <div class="chart-header">
          <h4 class="chart-title">${escapeHtml(title || "Chart")}</h4>
          <div class="chart-actions">
            <button class="chart-delete" title="Delete">×</button>
          </div>
        </div>
        <div class="chart-content">
          ${chartContent}
        </div>
        ${renderChartLegend(data)}
      </div>
    `;
  }

  function getChartData(dataRange) {
    if (!dataRange) return { labels: [], values: [] };

    const ws = state.worksheets[state.activeWorksheet];
    const rangeParts = dataRange.split(":");
    if (rangeParts.length !== 2) return { labels: [], values: [] };

    const startRef = parseCellRef(rangeParts[0]);
    const endRef = parseCellRef(rangeParts[1]);
    if (!startRef || !endRef) return { labels: [], values: [] };

    const labels = [];
    const values = [];

    if (startRef.col === endRef.col) {
      for (let r = startRef.row; r <= endRef.row; r++) {
        const key = `${r},${startRef.col}`;
        const cellData = ws.data[key];
        const val = parseFloat(cellData?.value) || 0;
        values.push(val);
        labels.push(`Row ${r + 1}`);
      }
    } else {
      for (let c = startRef.col; c <= endRef.col; c++) {
        const key = `${startRef.row},${c}`;
        const cellData = ws.data[key];
        const val = parseFloat(cellData?.value) || 0;
        values.push(val);
        labels.push(getColName(c));
      }
    }

    return { labels, values };
  }

  function renderBarChart(data, maxHeight) {
    if (!data.values.length) return '<div class="chart-empty">No data</div>';

    const maxVal = Math.max(...data.values, 1);
    const bars = data.values
      .map((val, i) => {
        const height = (val / maxVal) * maxHeight;
        return `<div class="chart-bar" style="height:${height}px;" title="${data.labels[i]}: ${val}"></div>`;
      })
      .join("");

    return `<div class="chart-bar-container" style="height:${maxHeight}px;">${bars}</div>`;
  }

  function renderLineChart(data, width, height) {
    if (!data.values.length) return '<div class="chart-empty">No data</div>';

    const maxVal = Math.max(...data.values, 1);
    const padding = 20;
    const chartWidth = width - padding * 2;
    const chartHeight = height - padding * 2;

    const points = data.values.map((val, i) => {
      const x = padding + (i / (data.values.length - 1 || 1)) * chartWidth;
      const y = padding + chartHeight - (val / maxVal) * chartHeight;
      return `${x},${y}`;
    });

    const circles = data.values
      .map((val, i) => {
        const x = padding + (i / (data.values.length - 1 || 1)) * chartWidth;
        const y = padding + chartHeight - (val / maxVal) * chartHeight;
        return `<circle class="chart-line-point" cx="${x}" cy="${y}" r="4"/>`;
      })
      .join("");

    return `
      <svg class="chart-canvas" viewBox="0 0 ${width} ${height}">
        <polyline class="chart-line" points="${points.join(" ")}"/>
        ${circles}
      </svg>
    `;
  }

  function renderPieChart(data, size) {
    if (!data.values.length) return '<div class="chart-empty">No data</div>';

    const total = data.values.reduce((a, b) => a + b, 0) || 1;
    const colors = [
      "#4285f4",
      "#34a853",
      "#fbbc04",
      "#ea4335",
      "#9c27b0",
      "#00bcd4",
      "#ff5722",
    ];
    const cx = size / 2;
    const cy = size / 2;
    const r = size / 2 - 10;

    let startAngle = 0;
    const slices = data.values
      .map((val, i) => {
        const angle = (val / total) * 360;
        const endAngle = startAngle + angle;
        const largeArc = angle > 180 ? 1 : 0;

        const x1 = cx + r * Math.cos((startAngle * Math.PI) / 180);
        const y1 = cy + r * Math.sin((startAngle * Math.PI) / 180);
        const x2 = cx + r * Math.cos((endAngle * Math.PI) / 180);
        const y2 = cy + r * Math.sin((endAngle * Math.PI) / 180);

        const path = `M ${cx} ${cy} L ${x1} ${y1} A ${r} ${r} 0 ${largeArc} 1 ${x2} ${y2} Z`;
        startAngle = endAngle;

        return `<path d="${path}" fill="${colors[i % colors.length]}" stroke="white" stroke-width="2"/>`;
      })
      .join("");

    return `
      <div class="chart-pie-container">
        <svg class="chart-canvas" viewBox="0 0 ${size} ${size}" width="${size}" height="${size}">
          ${slices}
        </svg>
      </div>
    `;
  }

  function renderChartLegend(data) {
    const colors = [
      "#4285f4",
      "#34a853",
      "#fbbc04",
      "#ea4335",
      "#9c27b0",
      "#00bcd4",
      "#ff5722",
    ];
    const items = data.labels
      .map(
        (label, i) =>
          `<div class="legend-item"><span class="legend-color" style="background:${colors[i % colors.length]}"></span>${escapeHtml(label)}</div>`,
      )
      .join("");

    return `<div class="chart-legend">${items}</div>`;
  }

  function selectChart(chartId) {
    document.querySelectorAll(".chart-wrapper").forEach((el) => {
      el.classList.toggle("selected", el.dataset.chartId === chartId);
    });
  }

  function deleteChart(chartId) {
    const ws = state.worksheets[state.activeWorksheet];
    if (!ws.charts) return;

    ws.charts = ws.charts.filter((c) => c.id !== chartId);
    renderCharts();
    state.isDirty = true;
    scheduleAutoSave();
  }

  function startDragChart(e, chartId) {
    const wrapper = document.querySelector(`[data-chart-id="${chartId}"]`);
    if (!wrapper) return;

    const startX = e.clientX;
    const startY = e.clientY;
    const startLeft = parseInt(wrapper.style.left) || 0;
    const startTop = parseInt(wrapper.style.top) || 0;

    const onMouseMove = (moveEvent) => {
      const dx = moveEvent.clientX - startX;
      const dy = moveEvent.clientY - startY;
      wrapper.style.left = `${startLeft + dx}px`;
      wrapper.style.top = `${startTop + dy}px`;
    };

    const onMouseUp = () => {
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);

      const ws = state.worksheets[state.activeWorksheet];
      const chart = ws.charts?.find((c) => c.id === chartId);
      if (chart) {
        chart.position.col = Math.round(
          parseInt(wrapper.style.left) / CONFIG.COL_WIDTH,
        );
        chart.position.row = Math.round(
          parseInt(wrapper.style.top) / CONFIG.ROW_HEIGHT,
        );
        state.isDirty = true;
        scheduleAutoSave();
      }
    };

    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
  }

  function renderImages() {
    const imagesContainer = document.getElementById("imagesContainer");
    if (!imagesContainer) return;

    const ws = state.worksheets[state.activeWorksheet];
    if (!ws.images || ws.images.length === 0) {
      imagesContainer.innerHTML = "";
      return;
    }

    imagesContainer.innerHTML = ws.images
      .map((img) => {
        const left = img.position?.col
          ? img.position.col * CONFIG.COL_WIDTH
          : 100;
        const top = img.position?.row
          ? img.position.row * CONFIG.ROW_HEIGHT
          : 100;
        const width = img.position?.width || 200;
        const height = img.position?.height || 150;

        return `
          <div class="image-wrapper" data-image-id="${img.id}" style="left:${left}px;top:${top}px;width:${width}px;height:${height}px;">
            <img src="${escapeHtml(img.url)}" alt="Embedded image" />
            <div class="image-resize-handle"></div>
          </div>
        `;
      })
      .join("");

    imagesContainer.querySelectorAll(".image-wrapper").forEach((wrapper) => {
      const imageId = wrapper.dataset.imageId;
      wrapper.addEventListener("click", () => selectImage(imageId));
      wrapper.addEventListener("mousedown", (e) => {
        if (e.target.classList.contains("image-resize-handle")) {
          startResizeImage(e, imageId);
        } else {
          startDragImage(e, imageId);
        }
      });
    });
  }

  function selectImage(imageId) {
    document.querySelectorAll(".image-wrapper").forEach((el) => {
      el.classList.toggle("selected", el.dataset.imageId === imageId);
    });
  }

  function startDragImage(e, imageId) {
    const wrapper = document.querySelector(`[data-image-id="${imageId}"]`);
    if (!wrapper) return;

    const startX = e.clientX;
    const startY = e.clientY;
    const startLeft = parseInt(wrapper.style.left) || 0;
    const startTop = parseInt(wrapper.style.top) || 0;

    const onMouseMove = (moveEvent) => {
      const dx = moveEvent.clientX - startX;
      const dy = moveEvent.clientY - startY;
      wrapper.style.left = `${startLeft + dx}px`;
      wrapper.style.top = `${startTop + dy}px`;
    };

    const onMouseUp = () => {
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);

      const ws = state.worksheets[state.activeWorksheet];
      const img = ws.images?.find((i) => i.id === imageId);
      if (img) {
        img.position.col = Math.round(
          parseInt(wrapper.style.left) / CONFIG.COL_WIDTH,
        );
        img.position.row = Math.round(
          parseInt(wrapper.style.top) / CONFIG.ROW_HEIGHT,
        );
        state.isDirty = true;
        scheduleAutoSave();
      }
    };

    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
    e.preventDefault();
  }

  function startResizeImage(e, imageId) {
    const wrapper = document.querySelector(`[data-image-id="${imageId}"]`);
    if (!wrapper) return;

    const startX = e.clientX;
    const startY = e.clientY;
    const startWidth = parseInt(wrapper.style.width) || 200;
    const startHeight = parseInt(wrapper.style.height) || 150;
    const aspectRatio = startWidth / startHeight;

    const onMouseMove = (moveEvent) => {
      const dx = moveEvent.clientX - startX;
      const newWidth = Math.max(50, startWidth + dx);
      const newHeight = newWidth / aspectRatio;
      wrapper.style.width = `${newWidth}px`;
      wrapper.style.height = `${newHeight}px`;
    };

    const onMouseUp = () => {
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);

      const ws = state.worksheets[state.activeWorksheet];
      const img = ws.images?.find((i) => i.id === imageId);
      if (img) {
        img.position.width = parseInt(wrapper.style.width);
        img.position.height = parseInt(wrapper.style.height);
        state.isDirty = true;
        scheduleAutoSave();
      }
    };

    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
    e.preventDefault();
    e.stopPropagation();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
