"use strict";
/* Docs table keyboard navigation + init (#2482). Tab/Shift+Tab and arrow keys
 * move between cells of the innermost table only, so a nested table never
 * hijacks the outer table's caret. Ctrl+Alt+T opens the contextual toolbar. */
(function (window) {
  var NS = (window.DocsTables = window.DocsTables || {});
  var UI = (window.DocsTablesUI = window.DocsTablesUI || {});
  var NAV = (window.DocsTablesNav = window.DocsTablesNav || {});

  function orderedCells(table) {
    var model = NS.buildModel(table);
    if (!model) return [];
    var cells = NS.ownCells(table);
    return model.placements.slice().sort(function (a, b) {
      return a.row - b.row || a.col - b.col;
    }).map(function (p) { return cells[p.ref._idx]; }).filter(Boolean);
  }

  function moveFocus(table, index) {
    var cells = orderedCells(table);
    if (!cells.length) return false;
    var target = cells[(index + cells.length) % cells.length];
    if (!target || !window.getSelection || !document.createRange) return false;
    try {
      var range = document.createRange();
      range.selectNodeContents(target);
      range.collapse(true);
      var sel = window.getSelection();
      sel.removeAllRanges();
      sel.addRange(range);
      return true;
    } catch (error) { return false; }
  }

  function currentIndex(table, cell) {
    return orderedCells(table).indexOf(cell);
  }

  function caretAtEdge(cell, atStart) {
    var sel = window.getSelection();
    if (!sel || !sel.isCollapsed || !sel.rangeCount) return false;
    var range = sel.getRangeAt(0);
    var start = cell.textContent ? cell.textContent.length : 0;
    if (atStart) return range.startOffset === 0;
    return range.endOffset >= start;
  }

  NAV.handleTab = function (event) {
    var cur = NS.cellFromSelection();
    if (!cur) return false;
    var index = currentIndex(cur.table, cur.cell);
    var cells = orderedCells(cur.table);
    if (!event.shiftKey && index === cells.length - 1) {
      event.preventDefault();
      if (NS.ops && NS.ops.insertRow) NS.ops.insertRow("below");
      return true;
    }
    event.preventDefault();
    return moveFocus(cur.table, index + (event.shiftKey ? -1 : 1));
  };

  NAV.handleArrow = function (event) {
    var cur = NS.cellFromSelection();
    if (!cur) return false;
    var model = cur.model;
    var occ = NS.layout(model);
    var dr = event.key === "ArrowDown" ? 1 : (event.key === "ArrowUp" ? -1 : 0);
    var dc = event.key === "ArrowRight" ? 1 : (event.key === "ArrowLeft" ? -1 : 0);
    if (dr && !caretAtEdge(cur.cell, dr < 0)) return false;
    if (dc && !caretAtEdge(cur.cell, dc < 0)) return false;
    var row = cur.row, col = cur.col, target = null;
    for (var step = 1; step <= model.rows + model.cols; step++) {
      var rr = row + dr * step;
      var cc = col + dc * step;
      if (rr < 0 || cc < 0 || rr >= model.rows || cc >= model.cols) break;
      var entry = occ[rr][cc];
      if (entry && entry.start && entry.ref._idx !== cur.ref._idx) {
        target = NS.ownCells(cur.table)[entry.ref._idx];
        break;
      }
    }
    if (!target) return false;
    event.preventDefault();
    return moveFocus(cur.table, currentIndex(cur.table, target));
  };

  function dialogOpen() {
    return !!document.querySelector(".gb-dtbl-backdrop");
  }

  function onKeyDown(event) {
    if (event.ctrlKey && event.altKey && (event.key === "t" || event.key === "T")) {
      event.preventDefault();
      if (dialogOpen()) return;
      UI.toggle();
      var first = document.querySelector("#gb-docs-tbl-toolbar button");
      if (first && first.focus) first.focus();
      return;
    }
    if (dialogOpen() || event.ctrlKey || event.metaKey || event.altKey) return;
    if (event.key === "Tab" && NS.activeTable()) { NAV.handleTab(event); return; }
    if (["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].indexOf(event.key) >= 0) {
      NAV.handleArrow(event);
    }
  }

  function ready(fn) {
    if (document.readyState === "loading" && document.addEventListener) {
      document.addEventListener("DOMContentLoaded", fn, { once: true });
    } else {
      fn();
    }
  }

  NAV.init = function () {
    if (NAV._initialized) return;
    NAV._initialized = true;
    if (UI.init) UI.init();
    if (document.addEventListener) document.addEventListener("keydown", onKeyDown, true);
  };

  ready(NAV.init);
})(window);
