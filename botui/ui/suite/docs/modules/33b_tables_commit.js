"use strict";
/* Docs rich table selection + commit (#2482). Continuation of 33_tables_model.
 * Selection is scoped to the innermost table so nested tables never leak into
 * an outer table operation. Every DOM change is committed as ONE undo step. */
(function (window) {
  var NS = (window.DocsTables = window.DocsTables || {});

  function activeTable() {
    var article = NS.getArticle();
    if (!article || !window.getSelection) return null;
    var sel = window.getSelection();
    var node = sel && sel.anchorNode;
    var cell = NS.ownCell(node);
    if (!cell || !article.contains(cell)) return null;
    var table = NS.closestEl(cell, "table");
    return table && article.contains(table) ? table : null;
  }
  NS.activeTable = activeTable;

  function cellAt(table, row, col) {
    var model = NS.buildModel(table);
    if (!model) return null;
    var occ = NS.layout(model);
    var entry = occ[row] && occ[row][col];
    if (!entry) return null;
    return NS.ownCells(table)[entry.ref._idx] || null;
  }
  NS.cellAt = cellAt;

  function cellFromSelection() {
    var table = activeTable();
    if (!table || !window.getSelection()) return null;
    var cell = NS.ownCell(window.getSelection().anchorNode);
    var idx = NS.ownCells(table).indexOf(cell);
    var model = NS.buildModel(table);
    if (!model || idx < 0) return null;
    var occ = NS.layout(model);
    for (var r = 0; r < model.rows; r++) {
      for (var c = 0; c < model.cols; c++) {
        if (occ[r][c] && occ[r][c].ref._idx === idx) {
          return { table: table, cell: cell, row: r, col: c, ref: occ[r][c].ref, model: model };
        }
      }
    }
    return null;
  }
  NS.cellFromSelection = cellFromSelection;

  function selectedCells(table) {
    var found = [];
    if (!table || !window.getSelection || !window.getSelection()) return found;
    var sel = window.getSelection();
    if (!sel.getRangeAt || !sel.rangeCount) return found;
    var range;
    try { range = sel.getRangeAt(0); } catch (error) { return found; }
    NS.ownCells(table).forEach(function (cell) {
      var inside = false;
      try {
        inside = typeof range.intersectsNode === "function"
          ? range.intersectsNode(cell)
          : !!(range.commonAncestorContainer && cell.contains(range.commonAncestorContainer));
      } catch (error) { inside = false; }
      if (inside || cell === NS.ownCell(sel.anchorNode) || cell === NS.ownCell(sel.focusNode)) {
        found.push(cell);
      }
    });
    return found;
  }
  NS.selectedCells = selectedCells;

  /* Map the cells selected by the user to their placement coordinates. */
  function selectionRect(table) {
    var cells = selectedCells(table);
    if (!cells.length) return null;
    var model = NS.buildModel(table);
    var occ = NS.layout(model);
    var indexes = cells.map(function (cell) { return NS.ownCells(table).indexOf(cell); });
    var minR = model.rows, minC = model.cols, maxR = -1, maxC = -1;
    for (var r = 0; r < model.rows; r++) {
      for (var c = 0; c < model.cols; c++) {
        var entry = occ[r][c];
        if (entry && indexes.indexOf(entry.ref._idx) >= 0) {
          minR = Math.min(minR, r); minC = Math.min(minC, c);
          maxR = Math.max(maxR, r); maxC = Math.max(maxC, c);
        }
      }
    }
    return maxR < 0 ? null : { minR: minR, minC: minC, maxR: maxR, maxC: maxC, model: model };
  }
  NS.selectionRect = selectionRect;

  function focusCell(table, row, col) {
    var cell = cellAt(table, row, col);
    if (!cell || !window.getSelection || !document.createRange) return;
    try {
      var range = document.createRange();
      range.selectNodeContents(cell);
      range.collapse(true);
      var sel = window.getSelection();
      sel.removeAllRanges();
      sel.addRange(range);
    } catch (error) { /* detached node / stub: caret is best effort */ }
  }
  NS.focusCell = focusCell;

  /* The single write boundary. selectNode + insertHTML is one native undo
   * entry; the outerHTML fallback exists for engines without execCommand. */
  function replaceTable(table, newTable, focus) {
    if (!table || !table.parentNode) return null;
    var id = table.getAttribute("data-docs-tables-id") || NS.uid();
    table.setAttribute("data-docs-tables-id", id);
    newTable.setAttribute("data-docs-tables-id", id);
    var html = newTable.outerHTML;
    if (!html) return null;
    var committed = false;
    try {
      if (typeof document.execCommand === "function" && document.createRange && window.getSelection) {
        var range = document.createRange();
        range.selectNode(table);
        var sel = window.getSelection();
        sel.removeAllRanges();
        sel.addRange(range);
        committed = !!document.execCommand("insertHTML", false, html);
      }
    } catch (error) { committed = false; }
    if (!committed) {
      try { table.outerHTML = html; } catch (replaceError) { return null; }
    }
    NS.save();
    var article = NS.getArticle();
    var query = '[data-docs-tables-id="' + id + '"]';
    var next = article && article.querySelector ? article.querySelector(query) : null;
    if (focus && next) focusCell(next, focus.row, focus.col);
    if (next && window.DocsTablesUI && typeof window.DocsTablesUI.refresh === "function") {
      window.DocsTablesUI.refresh(next);
    }
    return next;
  }
  NS.replaceTable = replaceTable;

  /* Same single-step boundary for non-table replacements (text conversion). */
  NS.replaceNode = function (node, html, callback) {
    if (!node || !node.parentNode || !html) return false;
    var committed = false;
    try {
      if (typeof document.execCommand === "function" && document.createRange && window.getSelection) {
        var range = document.createRange();
        range.selectNode(node);
        var sel = window.getSelection();
        sel.removeAllRanges();
        sel.addRange(range);
        committed = !!document.execCommand("insertHTML", false, html);
      }
    } catch (error) { committed = false; }
    if (!committed) {
      try { node.outerHTML = html; } catch (replaceError) { return false; }
    }
    NS.save();
    if (typeof callback === "function") callback();
    return true;
  };

  NS.cellText = function (cell) {
    return (cell.textContent || "").replace(/\u00a0/g, " ").trim();
  };
})(window);
