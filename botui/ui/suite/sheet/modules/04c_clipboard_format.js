"use strict";

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
        if (!cellData) return;
        if (typeof window.setCellValue === "function") {
          const v = cellData.formula ? cellData.formula : (cellData.value != null ? String(cellData.value) : "");
          window.setCellValue(targetRow, targetCol, v, { skipHistory: true });
        } else {
          const key = `${targetRow},${targetCol}`;
          ws.data[key] = { ...cellData };
          renderCell(targetRow, targetCol);
        }
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

  window.pasteSelection = pasteSelection;
  window.cutSelection = cutSelection;
  window.getSelectionData = getSelectionData;
  window.clearCells = clearCells;
