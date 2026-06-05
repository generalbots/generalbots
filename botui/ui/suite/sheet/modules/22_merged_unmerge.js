"use strict";

/**
 * Module 22: Merged cell unmerge and overlap detection for Sheet.
 * Adds:
 *   - unmergeCells(mergeIndex) - removes a merge and restores hidden
 *     cells (with empty values).
 *   - validateMergeRange(startRow, startCol, endRow, endCol) - checks
 *     that the proposed range does not overlap with any existing merge.
 *     Returns { ok: true } or { ok: false, conflicts: [...] }.
 *   - renderMerges() - re-renders the merged cells with explicit pixel
 *     widths/heights, supporting virtual mode (where grid-column/row-span
 *     is ignored by absolute positioning).
 *
 * Public API: window.SheetMergeUtil = { unmergeCells, validateMergeRange,
 *   renderMerges, ensureMergeState }.
 */

(function () {
  function getState() { return window.state || null; }
  function getWorksheet() {
    const s = getState();
    if (!s || !s.worksheets) return null;
    return s.worksheets[s.activeWorksheet];
  }

  function ensureMergeState() {
    const ws = getWorksheet();
    if (!ws) return null;
    if (!ws.merges) ws.merges = [];
    return ws.merges;
  }

  function getDefaultColWidth() {
    if (typeof window.CONFIG !== "undefined" && window.CONFIG && window.CONFIG.COL_WIDTH) return window.CONFIG.COL_WIDTH;
    return 100;
  }
  function getDefaultRowHeight() {
    if (typeof window.CONFIG !== "undefined" && window.CONFIG && window.CONFIG.ROW_HEIGHT) return window.CONFIG.ROW_HEIGHT;
    return 24;
  }

  function columnWidth(ws, col) {
    if (ws && ws.colWidths && ws.colWidths[col] != null) return ws.colWidths[col];
    return getDefaultColWidth();
  }
  function rowHeight(ws, row) {
    if (ws && ws.rowHeights && ws.rowHeights[row] != null) return ws.rowHeights[row];
    return getDefaultRowHeight();
  }

  function columnOffset(ws, col) {
    let total = 0;
    for (let i = 0; i < col; i++) total += columnWidth(ws, i);
    return total;
  }
  function rowOffset(ws, row) {
    let total = 0;
    for (let i = 0; i < row; i++) total += rowHeight(ws, i);
    return total;
  }

  function overlaps(a, b) {
    if (a.startRow > b.endRow || a.endRow < b.startRow) return false;
    if (a.startCol > b.endCol || a.endCol < b.startCol) return false;
    return true;
  }

  function validateMergeRange(startRow, startCol, endRow, endCol) {
    const merges = ensureMergeState();
    if (!merges) return { ok: false, reason: "No worksheet" };
    if (startRow === endRow && startCol === endCol) {
      return { ok: false, reason: "Range is a single cell" };
    }
    const proposed = { startRow, startCol, endRow, endCol };
    const conflicts = [];
    for (let i = 0; i < merges.length; i++) {
      if (overlaps(proposed, merges[i])) conflicts.push({ index: i, merge: merges[i] });
    }
    if (conflicts.length > 0) {
      return { ok: false, conflicts, reason: "Range overlaps with existing merges" };
    }
    return { ok: true };
  }

  function unmergeCells(mergeIndex) {
    const merges = ensureMergeState();
    const ws = getWorksheet();
    if (!merges || !ws) return false;
    if (mergeIndex < 0 || mergeIndex >= merges.length) return false;
    const removed = merges[mergeIndex];
    const existing = merges.slice();
    merges.splice(mergeIndex, 1);
    if (window.SheetUndo && window.SheetUndo.execute) {
      window.SheetUndo.execute({
        id: "unmerge-" + Date.now(),
        type: "unmerge",
        name: "Unmerge cells",
        mergeIndex, removed, existing,
        do() { merges.splice(this.mergeIndex, 1); renderMerges(); },
        undo() { merges.length = 0; merges.push.apply(merges, this.existing); renderMerges(); },
      });
    } else {
      renderMerges();
    }
    return true;
  }

  function unmergeByRange(startRow, startCol) {
    const merges = ensureMergeState();
    if (!merges) return -1;
    for (let i = 0; i < merges.length; i++) {
      if (merges[i].startRow === startRow && merges[i].startCol === startCol) {
        unmergeCells(i);
        return i;
      }
    }
    return -1;
  }

  function renderMerges() {
    const ws = getWorksheet();
    if (!ws) return;
    const merges = ws.merges || [];
    document.querySelectorAll(".merged-cell-overlay, .merge-resize-handle").forEach((el) => el.remove());
    const grid = document.getElementById("sheetGrid") || document.querySelector(".sheet-grid");
    if (!grid) return;
    for (const m of merges) {
      const overlay = document.createElement("div");
      overlay.className = "merged-cell-overlay";
      const left = columnOffset(ws, m.startCol);
      const top = rowOffset(ws, m.startRow);
      let width = 0;
      for (let c = m.startCol; c <= m.endCol; c++) width += columnWidth(ws, c);
      let height = 0;
      for (let r = m.startRow; r <= m.endRow; r++) height += rowHeight(ws, r);
      overlay.style.cssText = "position:absolute;left:" + left + "px;top:" + top + "px;width:" + width + "px;height:" + height + "px;border:2px solid #1a73e8;background:rgba(26,115,232,0.04);z-index:3;pointer-events:none;";
      const unmergeBtn = document.createElement("div");
      unmergeBtn.className = "merge-unmerge-btn";
      unmergeBtn.textContent = "✕";
      unmergeBtn.title = "Unmerge";
      unmergeBtn.style.cssText = "position:absolute;right:2px;top:2px;width:18px;height:18px;line-height:18px;text-align:center;background:#fff;border:1px solid #888;border-radius:50%;cursor:pointer;font-size:11px;z-index:4;";
      unmergeBtn.addEventListener("click", (e) => { e.stopPropagation(); unmergeByRange(m.startRow, m.startCol); });
      overlay.appendChild(unmergeBtn);
      overlay.addEventListener("dblclick", (e) => { e.stopPropagation(); unmergeByRange(m.startRow, m.startCol); });
      grid.appendChild(overlay);
    }
  }

  function attach() {
    document.addEventListener("sheetRender", renderMerges);
    document.addEventListener("sheetWorksheetChange", renderMerges);
    if (typeof window.mergeCells === "function") {
      const original = window.mergeCells;
      window.mergeCells = function (sr, sc, er, ec) {
        const v = validateMergeRange(sr, sc, er, ec);
        if (!v.ok) {
          if (window.addChatMessage) window.addChatMessage("system", "Cannot merge: " + v.reason);
          return false;
        }
        return original.call(this, sr, sc, er, ec);
      };
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 100);
  }

  window.SheetMergeUtil = {
    validateMergeRange,
    unmergeCells,
    unmergeByRange,
    renderMerges,
    ensureMergeState,
  };
})();
