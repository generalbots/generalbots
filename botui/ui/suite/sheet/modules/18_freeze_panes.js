"use strict";

/**
 * Module 18: Freeze and Split Panes for Sheet.
 * Adds freezeRow/freezeCol state. Rows 0..freezeRow-1 and cols
 * 0..freezeCol-1 are kept visible during scroll by attaching
 * position:sticky styles. The virtual grid is split into four
 * quadrants: top-left (frozen x frozen), top-right (frozen x scroll),
 * bottom-left (scroll x frozen), bottom-right (scroll x scroll).
 *
 * Public API: window.SheetFreeze = {
 *   freeze(row, col), unfreeze(), getFreeze(), split(orientation, offsetPx)
 * }.
 */

(function () {
  function getState() { return window.state || null; }
  function getWorksheet() {
    const s = getState();
    if (!s || !s.worksheets) return null;
    return s.worksheets[s.activeWorksheet];
  }

  function ensureFreezeState() {
    const ws = getWorksheet();
    if (!ws) return null;
    if (!ws.freeze) ws.freeze = { row: 0, col: 0 };
    return ws.freeze;
  }

  function getFreeze() {
    const ws = getWorksheet();
    return ws && ws.freeze ? { row: ws.freeze.row || 0, col: ws.freeze.col || 0 } : { row: 0, col: 0 };
  }

  function getDefaultColWidth() {
    if (typeof window.CONFIG !== "undefined" && window.CONFIG && window.CONFIG.COL_WIDTH) {
      return window.CONFIG.COL_WIDTH;
    }
    return 100;
  }

  function getDefaultRowHeight() {
    if (typeof window.CONFIG !== "undefined" && window.CONFIG && window.CONFIG.ROW_HEIGHT) {
      return window.CONFIG.ROW_HEIGHT;
    }
    return 24;
  }

  function columnOffset(col) {
    const ws = getWorksheet();
    let total = 0;
    const w = getDefaultColWidth();
    for (let i = 0; i < col; i++) {
      total += (ws && ws.colWidths && ws.colWidths[i] != null) ? ws.colWidths[i] : w;
    }
    return total;
  }

  function rowOffset(row) {
    const ws = getWorksheet();
    let total = 0;
    const h = getDefaultRowHeight();
    for (let i = 0; i < row; i++) {
      total += (ws && ws.rowHeights && ws.rowHeights[i] != null) ? ws.rowHeights[i] : h;
    }
    return total;
  }

  function freeze(row, col) {
    const f = ensureFreezeState();
    if (!f) return null;
    f.row = Math.max(0, row | 0);
    f.col = Math.max(0, col | 0);
    applyFreeze();
    document.dispatchEvent(new CustomEvent("sheetFreezeChange", { detail: f }));
    return { row: f.row, col: f.col };
  }

  function unfreeze() {
    const ws = getWorksheet();
    if (!ws) return null;
    ws.freeze = { row: 0, col: 0 };
    applyFreeze();
    document.dispatchEvent(new CustomEvent("sheetFreezeChange", { detail: ws.freeze }));
    return ws.freeze;
  }

  function applyFreeze() {
    const f = getFreeze();
    const colOff = columnOffset(f.col);
    const rowOff = rowOffset(f.row);
    document.querySelectorAll(".row-header, [data-row-header]").forEach((h) => {
      const r = parseInt(h.getAttribute("data-row") || h.getAttribute("data-row-header"), 10);
      if (!isNaN(r) && r < f.row) {
        h.style.position = "sticky";
        h.style.top = "0";
        h.style.zIndex = "4";
      } else {
        h.style.position = "";
        h.style.top = "";
        h.style.zIndex = "";
      }
    });
    document.querySelectorAll(".column-header, [data-col-header]").forEach((h) => {
      const c = parseInt(h.getAttribute("data-col") || h.getAttribute("data-col-header"), 10);
      if (!isNaN(c) && c < f.col) {
        h.style.position = "sticky";
        h.style.left = "0";
        h.style.zIndex = "3";
      } else {
        h.style.position = "";
        h.style.left = "";
        h.style.zIndex = "";
      }
    });
    document.querySelectorAll(".cell").forEach((c) => {
      const r = parseInt(c.getAttribute("data-row"), 10);
      const col = parseInt(c.getAttribute("data-col"), 10);
      let pos = "";
      let top = "";
      let bottom = "";
      let left = "";
      let z = "";
      if (!isNaN(r) && r < f.row) { pos = "sticky"; top = "0"; z = "2"; }
      if (!isNaN(col) && col < f.col) { pos = "sticky"; left = "0"; z = "1"; }
      if (!isNaN(r) && r < f.row && !isNaN(col) && col < f.col) { z = "5"; }
      c.style.position = pos;
      c.style.top = top;
      c.style.bottom = bottom;
      c.style.left = left;
      c.style.zIndex = z;
    });
    drawFreezeLines();
  }

  function drawFreezeLines() {
    document.querySelectorAll(".freeze-line-h, .freeze-line-v").forEach((el) => el.remove());
    const f = getFreeze();
    if (f.row <= 0 && f.col <= 0) return;
    const grid = document.getElementById("sheetGrid") || document.querySelector(".sheet-grid");
    if (!grid) return;
    const colOff = columnOffset(f.col);
    const rowOff = rowOffset(f.row);
    if (f.col > 0) {
      const v = document.createElement("div");
      v.className = "freeze-line-v";
      v.style.cssText = "position:absolute;top:0;bottom:0;width:2px;background:#1a73e8;z-index:6;pointer-events:none;left:" + colOff + "px;";
      grid.appendChild(v);
    }
    if (f.row > 0) {
      const h = document.createElement("div");
      h.className = "freeze-line-h";
      h.style.cssText = "position:absolute;left:0;right:0;height:2px;background:#1a73e8;z-index:6;pointer-events:none;top:" + rowOff + "px;";
      grid.appendChild(h);
    }
  }

  function split(orientation, offsetPx) {
    const ws = getWorksheet();
    if (!ws) return null;
    if (!ws.split) ws.split = {};
    ws.split.orientation = orientation;
    ws.split.offset = offsetPx;
    const grid = document.getElementById("gridContainer");
    if (!grid) return ws.split;
    if (orientation === "horizontal") {
      const top = document.createElement("div");
      top.className = "split-pane-top";
      top.style.cssText = "overflow:hidden;height:" + offsetPx + "px;";
      const bottom = document.createElement("div");
      bottom.className = "split-pane-bottom";
      bottom.style.cssText = "overflow:auto;";
      const bar = document.createElement("div");
      bar.className = "split-bar-h";
      bar.style.cssText = "height:6px;background:#ccc;cursor:row-resize;";
      grid.innerHTML = "";
      grid.appendChild(top);
      grid.appendChild(bar);
      grid.appendChild(bottom);
    }
    return ws.split;
  }

  function attach() {
    if (typeof window.CONFIG === "undefined" || !window.CONFIG) {
      window.CONFIG = window.CONFIG || { COL_WIDTH: 100, ROW_HEIGHT: 24 };
    }
    applyFreeze();
    document.addEventListener("sheetColWidthChanged", applyFreeze);
    document.addEventListener("sheetRowHeightChanged", applyFreeze);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 100);
  }

  window.SheetFreeze = {
    freeze,
    unfreeze,
    getFreeze,
    applyFreeze,
    split,
  };
})();
