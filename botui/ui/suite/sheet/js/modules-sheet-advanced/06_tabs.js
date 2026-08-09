"use strict";
/* Sheet advanced module: 06_tabs — worksheet tab bar (add/switch/delete/rename) */

(function () {
  function grid() {
    if (window.SheetCore && window.SheetCore.getGrid) return window.SheetCore.getGrid();
    return window.SheetVirtualGrid || null;
  }

  function api() {
    if (window.SheetCore && window.SheetCore.api) return window.SheetCore.api();
    return window.SheetAPI || null;
  }

  function currentSheetId() {
    if (window.SheetCore && window.SheetCore.currentSheetId) return window.SheetCore.currentSheetId();
    return window.__SHEET_INITIAL_ID || "current";
  }

  function wsIndex() {
    if (window.SheetCore && window.SheetCore.wsIndex) return window.SheetCore.wsIndex();
    try {
      const v = parseInt(sessionStorage.getItem("sheet_active_index") || "0", 10);
      return isNaN(v) || v < 0 ? 0 : v;
    } catch (_) {
      return 0;
    }
  }

  function tabBar() {
    if (window.SheetCore && window.SheetCore.getTabBar) return window.SheetCore.getTabBar();
    return null;
  }

  function renderTabBar() {
    const tb = tabBar();
    if (!tb) return;
    const sheet = window.__LOADED_SHEET;
    const idx = wsIndex();
    tb.innerHTML = "";
    const addBtn = document.createElement("button");
    addBtn.type = "button";
    addBtn.className = "ss-tab-add";
    addBtn.textContent = "+";
    addBtn.title = "Nova planilha";
    addBtn.addEventListener("click", addWorksheetClient);
    tb.appendChild(addBtn);
    if (!sheet || !sheet.worksheets || !sheet.worksheets.length) return;
    sheet.worksheets.forEach(function (ws, i) {
      const tab = document.createElement("div");
      tab.className = "ss-tab" + (i === idx ? " ss-tab-active" : "");
      tab.dataset.index = i;
      const label = document.createElement("span");
      label.textContent = ws.name;
      label.addEventListener("dblclick", function () { renameWorksheetClient(i); });
      tab.appendChild(label);
      const del = document.createElement("button");
      del.type = "button";
      del.className = "ss-tab-del";
      del.textContent = "×";
      del.title = "Excluir planilha";
      del.addEventListener("click", function (e) { e.stopPropagation(); deleteWorksheetClient(i); });
      tab.appendChild(del);
      tab.addEventListener("click", function () { switchWorksheetClient(i); });
      tb.appendChild(tab);
    });
  }

  function rehydrateGrid() {
    const g = grid();
    if (!g) return;
    const sheet = window.__LOADED_SHEET;
    const idx = wsIndex();
    if (!sheet || !sheet.worksheets || !sheet.worksheets[idx]) return;
    const ws = sheet.worksheets[idx];
    g.cells = new Map();
    if (ws.data) {
      for (const cellRef in ws.data) {
        g.cells.set(cellRef, ws.data[cellRef]);
      }
    }
    g.requestSeq++;
    g.lastRenderedRange = null;
    g.requestRange();
    if (window.SheetAdvanced && window.SheetAdvanced.setRange) {
      window.SheetAdvanced.setRange(0, 0, 0, 0);
    }
  }

  function reloadSheetAfterMutation() {
    return api().load(currentSheetId()).then(function (sheet) {
      if (sheet) {
        window.__LOADED_SHEET = sheet;
        window.__SHEET_INITIAL_ID = sheet.id;
      }
      renderTabBar();
      rehydrateGrid();
      return sheet;
    });
  }

  function switchWorksheetClient(i) {
    window.dispatchEvent(new CustomEvent("gb-sheet-tab", { detail: { index: i } }));
    renderTabBar();
    rehydrateGrid();
  }

  function addWorksheetClient() {
    api().addWorksheet().then(function () {
      reloadSheetAfterMutation().then(function () {
        const sheet = window.__LOADED_SHEET;
        if (sheet) switchWorksheetClient(sheet.worksheets.length - 1);
      });
    });
  }

  function deleteWorksheetClient(i) {
    const sheet = window.__LOADED_SHEET;
    if (!sheet || !sheet.worksheets || sheet.worksheets.length <= 1) return;
    if (!window.confirm("Excluir planilha " + sheet.worksheets[i].name + "?")) return;
    api().deleteWorksheet(i).then(function () {
      reloadSheetAfterMutation().then(function () {
        const idx = wsIndex();
        if (idx >= sheet.worksheets.length) switchWorksheetClient(sheet.worksheets.length - 1);
      });
    });
  }

  function renameWorksheetClient(i) {
    const sheet = window.__LOADED_SHEET;
    if (!sheet || !sheet.worksheets || !sheet.worksheets[i]) return;
    const name = window.prompt("Novo nome da planilha:", sheet.worksheets[i].name);
    if (!name || !name.trim()) return;
    api().renameWorksheet(i, name.trim()).then(function () {
      sheet.worksheets[i].name = name.trim();
      renderTabBar();
    });
  }

  window.SheetTabs = {
    render: renderTabBar,
    switchWorksheet: switchWorksheetClient,
    addWorksheet: addWorksheetClient,
    deleteWorksheet: deleteWorksheetClient,
    renameWorksheet: renameWorksheetClient,
  };

  if (window.SheetCore) {
    window.SheetCore.renderTabBar = renderTabBar;
    window.SheetCore.rehydrateGrid = rehydrateGrid;
  }
})();