"use strict";

/**
 * Module 17: Column and Row resize for Sheet.
 * Adds mousedown/mousemove/mouseup drag handlers on the borders between
 * column/row headers. Persists custom sizes in state.worksheets[i].colWidths
 * and rowHeights as Record<index, pixels>. Custom sizes are honored by
 * VirtualGrid (per-column widths override COL_WIDTH) and by the column
 * header rendering (the inline width is set to the stored px value).
 *
 * Public API: window.SheetResize = { getColWidth, getRowHeight,
 *   setColWidth, setRowHeight, resetSizes, attach() }.
 */

(function () {
  const MIN_SIZE = 20;
  const MAX_SIZE = 800;
  const DRAG_THRESHOLD = 3;

  function getState() { return window.state || null; }
  function getWorksheet() {
    const s = getState();
    if (!s || !s.worksheets) return null;
    return s.worksheets[s.activeWorksheet];
  }

  function getColWidth(col) {
    const ws = getWorksheet();
    if (!ws) return null;
    if (ws.colWidths && ws.colWidths[col] != null) return ws.colWidths[col];
    return null;
  }

  function getRowHeight(row) {
    const ws = getWorksheet();
    if (!ws) return null;
    if (ws.rowHeights && ws.rowHeights[row] != null) return ws.rowHeights[row];
    return null;
  }

  function setColWidth(col, pixels) {
    const ws = getWorksheet();
    if (!ws) return;
    if (!ws.colWidths) ws.colWidths = {};
    ws.colWidths[col] = Math.max(MIN_SIZE, Math.min(MAX_SIZE, pixels));
    applyColWidthToHeader(col, ws.colWidths[col]);
    applyColWidthToCells(col, ws.colWidths[col]);
  }

  function setRowHeight(row, pixels) {
    const ws = getWorksheet();
    if (!ws) return;
    if (!ws.rowHeights) ws.rowHeights = {};
    ws.rowHeights[row] = Math.max(MIN_SIZE, Math.min(MAX_SIZE, pixels));
    applyRowHeightToCells(row, ws.rowHeights[row]);
  }

  function resetSizes() {
    const ws = getWorksheet();
    if (!ws) return;
    ws.colWidths = {};
    ws.rowHeights = {};
    document.querySelectorAll(".column-header, [data-col-header]").forEach((h) => {
      h.style.width = "";
    });
    document.querySelectorAll(".cell, [data-row]").forEach((c) => {
      c.style.minWidth = "";
      c.style.minHeight = "";
    });
    document.querySelectorAll(".row-header, [data-row-header]").forEach((h) => {
      h.style.height = "";
    });
  }

  function applyColWidthToHeader(col, px) {
    document.querySelectorAll('.column-header[data-col="' + col + '"], [data-col-header="' + col + '"]').forEach((h) => {
      h.style.width = px + "px";
    });
  }

  function applyColWidthToCells(col, px) {
    document.querySelectorAll('.cell[data-col="' + col + '"]').forEach((c) => {
      c.style.minWidth = px + "px";
      c.style.width = px + "px";
    });
  }

  function applyRowHeightToCells(row, px) {
    document.querySelectorAll('.cell[data-row="' + row + '"], [data-row="' + row + '"]').forEach((c) => {
      c.style.minHeight = px + "px";
      c.style.height = px + "px";
    });
    document.querySelectorAll('.row-header[data-row="' + row + '"]').forEach((h) => {
      h.style.height = px + "px";
    });
  }

  function startColDrag(e, header) {
    e.preventDefault();
    e.stopPropagation();
    const col = parseInt(header.getAttribute("data-col") || header.getAttribute("data-col-header"), 10);
    if (isNaN(col)) return;
    const startX = e.clientX;
    const startWidth = header.getBoundingClientRect().width;
    const ghost = document.createElement("div");
    ghost.style.cssText = "position:fixed;top:0;bottom:0;width:2px;background:#1a73e8;z-index:9999;pointer-events:none;left:" + (startX) + "px;";
    document.body.appendChild(ghost);
    let lastDelta = 0;
    function onMove(ev) {
      const delta = ev.clientX - startX;
      if (Math.abs(delta) < DRAG_THRESHOLD) return;
      lastDelta = delta;
      ghost.style.left = (startX + delta) + "px";
    }
    function onUp() {
      document.removeEventListener("mousemove", onMove);
      document.removeEventListener("mouseup", onUp);
      ghost.remove();
      if (lastDelta !== 0) setColWidth(col, startWidth + lastDelta);
    }
    document.addEventListener("mousemove", onMove);
    document.addEventListener("mouseup", onUp);
  }

  function startRowDrag(e, header) {
    e.preventDefault();
    e.stopPropagation();
    const row = parseInt(header.getAttribute("data-row") || header.getAttribute("data-row-header"), 10);
    if (isNaN(row)) return;
    const startY = e.clientY;
    const startHeight = header.getBoundingClientRect().height;
    const ghost = document.createElement("div");
    ghost.style.cssText = "position:fixed;left:0;right:0;height:2px;background:#1a73e8;z-index:9999;pointer-events:none;top:" + (startY) + "px;";
    document.body.appendChild(ghost);
    let lastDelta = 0;
    function onMove(ev) {
      const delta = ev.clientY - startY;
      if (Math.abs(delta) < DRAG_THRESHOLD) return;
      lastDelta = delta;
      ghost.style.top = (startY + delta) + "px";
    }
    function onUp() {
      document.removeEventListener("mousemove", onMove);
      document.removeEventListener("mouseup", onUp);
      ghost.remove();
      if (lastDelta !== 0) setRowHeight(row, startHeight + lastDelta);
    }
    document.addEventListener("mousemove", onMove);
    document.addEventListener("mouseup", onUp);
  }

  function onMouseDown(e) {
    if (!e.target) return;
    const target = e.target.closest ? e.target.closest(".column-resize, .row-resize, .column-header, .row-header") : null;
    if (!target) return;
    const rect = target.getBoundingClientRect();
    if (target.classList.contains("column-header") || target.classList.contains("column-resize") || target.hasAttribute("data-col")) {
      const xWithin = e.clientX - rect.left;
      const colW = rect.width;
      if (xWithin > colW - 6) {
        if (e.shiftKey) {
          e.preventDefault();
          e.stopPropagation();
        }
        startColDrag(e, target);
      }
    } else if (target.classList.contains("row-header") || target.classList.contains("row-resize") || target.hasAttribute("data-row")) {
      const yWithin = e.clientY - rect.top;
      const rowH = rect.height;
      if (yWithin > rowH - 6) {
        if (e.shiftKey) {
          e.preventDefault();
          e.stopPropagation();
        }
        startRowDrag(e, target);
      }
    }
  }

  function addResizeCursor(element, type) {
    if (!element) return;
    if (type === "col") {
      element.style.cursor = "col-resize";
      element.addEventListener("mousedown", function (e) {
        const rect = element.getBoundingClientRect();
        if (e.clientX - rect.left > rect.width - 6) startColDrag(e, element);
      });
    } else if (type === "row") {
      element.style.cursor = "row-resize";
      element.addEventListener("mousedown", function (e) {
        const rect = element.getBoundingClientRect();
        if (e.clientY - rect.top > rect.height - 6) startRowDrag(e, element);
      });
    }
  }

  function attach() {
    document.addEventListener("mousedown", onMouseDown);
    document.querySelectorAll(".column-header, [data-col-header]").forEach((h) => addResizeCursor(h, "col"));
    document.querySelectorAll(".row-header, [data-row-header]").forEach((h) => addResizeCursor(h, "row"));
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 100);
  }

  window.SheetResize = {
    getColWidth,
    getRowHeight,
    setColWidth,
    setRowHeight,
    resetSizes,
    attach,
  };
})();
