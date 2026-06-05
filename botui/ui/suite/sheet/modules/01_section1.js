
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
