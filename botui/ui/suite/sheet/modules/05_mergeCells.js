
"use strict";

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
        await saveSheet();
        if (!state.sheetId) throw new Error('Failed to save sheet before export');
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
