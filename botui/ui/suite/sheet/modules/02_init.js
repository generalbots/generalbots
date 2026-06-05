// sheet/modules/02_init.js
"use strict";

// Functions: init, cacheElements, initVirtualGrid, destroyVirtualGrid, renderGrid, renderColumnHeaders, renderRowHeaders, renderAllCellsLegacy, renderAllCells, renderCell, renderCellLegacy, applyFormatToCell, getColName, parseColName, getCellRef, parseCellRef, bindEvents

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
