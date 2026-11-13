"use strict";
/* Sheet advanced module: 13_merge — merge/unmerge the real selection and render merged regions */

(function () {
  let wrapped = false;

  function grid() {
    if (window.SheetCore && window.SheetCore.getGrid) return window.SheetCore.getGrid();
    return window.SheetVirtualGrid || null;
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

  function selection() {
    const adv = window.SheetAdvanced;
    return adv && adv.getSelection ? adv.getSelection() : null;
  }

  function mergedRanges() {
    const sheet = window.__LOADED_SHEET;
    if (!sheet || !sheet.worksheets || !sheet.worksheets[wsIndex()]) return [];
    return sheet.worksheets[wsIndex()].merged_cells || [];
  }

  function rangePayload() {
    const sel = selection();
    const g = grid();
    if (!sel || !g) return null;
    return {
      sheet_id: currentSheetId(),
      worksheet_index: wsIndex(),
      start_row: sel.startRow,
      start_col: sel.startCol,
      end_row: sel.endRow,
      end_col: sel.endCol,
    };
  }

  function mergeSelection() {
    const payload = rangePayload();
    if (!payload || (payload.start_row === payload.end_row && payload.start_col === payload.end_col)) {
      showToast("Select a range of more than one cell to merge");
      return;
    }
    return fetch("/api/sheet/merge", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    })
      .then(function (r) { return r.json(); })
      .then(function () { reloadFromServer(); })
      .catch(function (err) {
        console.error("[gb-sheet] Merge failed:", err);
        showToast("Could not merge cells");
      });
  }

  function unmergeSelection() {
    const payload = rangePayload();
    if (!payload) return;
    return fetch("/api/sheet/unmerge", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    })
      .then(function (r) { return r.json(); })
      .then(function () { reloadFromServer(); })
      .catch(function (err) {
        console.error("[gb-sheet] Unmerge failed:", err);
        showToast("Could not unmerge cells");
      });
  }

  function reloadFromServer() {
    const a = window.SheetAPI;
    if (!a) return;
    a.load(currentSheetId()).then(function (sheet) {
      if (sheet) {
        window.__LOADED_SHEET = sheet;
        window.__SHEET_INITIAL_ID = sheet.id;
      }
      if (window.SheetCore && window.SheetCore.rehydrateGrid) window.SheetCore.rehydrateGrid();
    });
  }

  function showToast(msg) {
    const id = "ss-merge-toast";
    let toast = document.getElementById(id);
    if (!toast) {
      toast = document.createElement("div");
      toast.id = id;
      toast.style.cssText = "position:fixed;bottom:24px;left:50%;transform:translateX(-50%);background:#dc2626;color:#fff;padding:10px 18px;border-radius:6px;font-size:13px;z-index:10000;box-shadow:0 4px 12px rgba(0,0,0,0.3);";
      document.body.appendChild(toast);
    }
    toast.textContent = msg;
    toast.style.display = "block";
    clearTimeout(toast.__timer);
    toast.__timer = setTimeout(function () { toast.style.display = "none"; }, 2600);
  }

  // Span of a merged region in pixels, using the real (imported) geometry of
  // every covered column/row — a fixed 96 px per column shrank merged headers
  // and indicator boxes on any workbook with custom widths (#2152).
  function spanWidth(g, first, last) {
    let width = 0;
    for (let c = first; c <= last; c++) {
      width += g.colWidthOf ? g.colWidthOf(c) : COL_WIDTH;
    }
    return width;
  }

  function spanHeight(g, first, last) {
    let height = 0;
    for (let r = first; r <= last; r++) {
      if (g.rowHeightOf) {
        height += g.rowHeightOf(r);
      } else if (window.SheetCore && window.SheetCore.rowHeight) {
        height += window.SheetCore.rowHeight(r);
      } else {
        height += ROW_HEIGHT;
      }
    }
    return height;
  }

  function renderMerges() {
    const g = grid();
    const merges = mergedRanges();
    if (!g || !merges.length) return;
    const visible = g.visibleRowRange();
    const COL_WIDTH = 96;
    const ROW_HEIGHT = 24;
    for (let i = 0; i < merges.length; i++) {
      const m = merges[i];
      if (m.end_row < visible.start || m.start_row >= visible.end) continue;
      for (let r = m.start_row; r <= m.end_row; r++) {
        for (let c = m.start_col; c <= m.end_col; c++) {
          const node = g.bodyInner.querySelector('[data-row="' + r + '"][data-col="' + c + '"]');
          if (!node) continue;
          if (r === m.start_row && c === m.start_col) {
            node.style.display = "block";
            node.style.width = spanWidth(g, m.start_col, m.end_col) + "px";
            node.style.height = spanHeight(g, m.start_row, m.end_row) + "px";
            node.style.zIndex = "4";
            node.style.overflow = "hidden";
            node.textContent = g.cells.get(r + "," + c) ? (g.cells.get(r + "," + c).value || "") : "";
          } else {
            node.style.display = "none";
          }
        }
      }
    }
  }

  function wrapRender() {
    const g = grid();
    if (!g || wrapped) return;
    wrapped = true;
    const orig = g.render.bind(g);
    g.render = function () {
      orig();
      renderMerges();
    };
  }

  function wire() {
    const g = grid();
    if (!g || !g.render) {
      setTimeout(wire, 100);
      return;
    }
    wrapRender();
    const host = document.getElementById("sheet-app");
    if (host && !host.__mergeBound) {
      host.__mergeBound = true;
      const btn = document.getElementById("mergeCellsBtn");
      if (btn) {
        btn.addEventListener("click", function (e) {
          e.preventDefault();
          e.stopPropagation();
          mergeSelection();
        });
      }
    }
  }

  window.SheetMerge = {
    merge: mergeSelection,
    unmerge: unmergeSelection,
    render: renderMerges,
  };

  if (window.SheetCore) {
    window.SheetCore.merge = mergeSelection;
  }

  setTimeout(wire, 0);
})();