"use strict";
/* Slides table & media authoring (#2487) — table grid model. Spanned cells are
 * represented as null placeholders so every mutation can emit a valid table. */
(function () {
  var STM = window.SlidesTableMedia = window.SlidesTableMedia || {};
  var DEFAULT_OPTIONS = {
    header: true, banded: false, borders: true, padding: 4,
    align: "left", style: "plain", widths: []
  };

  function clone(value) { return JSON.parse(JSON.stringify(value)); }
  STM.cloneTable = clone;

  function emptyCell() { return { text: "", colspan: 1, rowspan: 1 }; }

  STM.tableModelFromDom = function (table) {
    if (!table) return null;
    var grid = [];
    for (var r = 0; r < table.rows.length; r++) {
      var source = table.rows[r];
      if (!grid[r]) grid[r] = [];
      for (var c = 0; c < source.cells.length; c++) {
        var sourceCell = source.cells[c];
        var col = c;
        while (grid[r][col]) col += 1;
        var span = Math.max(1, parseInt(sourceCell.getAttribute("colspan"), 10) || 1);
        var vspan = Math.max(1, parseInt(sourceCell.getAttribute("rowspan"), 10) || 1);
        var cell = {
          text: sourceCell.textContent || "",
          colspan: span,
          rowspan: vspan
        };
        var cellStyle = sourceCell.getAttribute("data-stm-cell-style");
        if (cellStyle) cell.style = cellStyle;
        grid[r][col] = cell;
        for (var dc = 1; dc < span; dc++) grid[r][col + dc] = null;
        for (var dr = 1; dr < vspan; dr++) {
          if (!grid[r + dr]) grid[r + dr] = [];
          for (var sc = 0; sc < span; sc++) grid[r + dr][col + sc] = null;
        }
      }
    }
    var cols = 0;
    grid.forEach(function (row) { cols = Math.max(cols, row.length); });
    grid.forEach(function (row) { while (row.length < cols) row.push(emptyCell()); });
    var firstRow = table.rows[0];
    var attr = table.getAttribute("data-stm-options") || "";
    var stored = {};
    try { stored = attr ? JSON.parse(attr) : {}; } catch (e) { stored = {}; }
    var options = Object.assign({}, DEFAULT_OPTIONS, stored, {
      header: stored.header !== undefined ? stored.header : !!(firstRow && firstRow.cells[0] && firstRow.cells[0].tagName === "TH"),
      widths: stored.widths || Array.from(table.querySelectorAll("col")).map(function (col) {
        return parseFloat(col.getAttribute("width")) || parseFloat(col.style.width) || 0;
      }).filter(function (value) { return value > 0; })
    });
    return { cols: cols, grid: grid, options: options };
  };

  function ensureGrid(model) {
    if (!model || !Array.isArray(model.grid)) return model;
    var cols = model.cols || 0;
    model.grid.forEach(function (row) { while (row.length < cols) row.push(emptyCell()); });
    while (model.grid.length && model.grid[model.grid.length - 1].every(function (cell) { return !cell; })) model.grid.pop();
    return model;
  }

  /* Replace every span with explicit cells. Structural edits expand first so a
   * merge can never leave a dangling placeholder behind. */
  STM.tableExpandAll = function (model) {
    var next = clone(model);
    var cols = next.cols;
    var full = [];
    for (var r = 0; r < next.grid.length; r++) {
      full[r] = [];
      for (var c = 0; c < cols; c++) {
        var cell = next.grid[r][c];
        full[r][c] = cell ? { text: cell.text, colspan: 1, rowspan: 1, style: cell.style } : emptyCell();
      }
    }
    next.grid = full;
    return ensureGrid(next);
  };

  function at(model, r, c) {
    if (!model.grid[r]) return null;
    return model.grid[r][c] || null;
  }

  STM.tableSetCell = function (model, r, c, text) {
    var next = clone(model);
    var cell = at(next, r, c);
    if (!cell) return next;
    cell.text = String(text === null || text === undefined ? "" : text);
    return next;
  };

  STM.tableInsertRow = function (model, index, after) {
    var next = STM.tableExpandAll(model);
    var atIndex = Math.max(0, Math.min(next.grid.length, index + (after ? 1 : 0)));
    var row = [];
    for (var i = 0; i < next.cols; i++) row.push(emptyCell());
    next.grid.splice(atIndex, 0, row);
    return ensureGrid(next);
  };

  STM.tableDeleteRow = function (model, index) {
    if (model.grid.length <= 1) return clone(model);
    var next = STM.tableExpandAll(model);
    next.grid.splice(Math.max(0, Math.min(index, next.grid.length - 1)), 1);
    return ensureGrid(next);
  };

  STM.tableInsertCol = function (model, index, after) {
    var next = STM.tableExpandAll(model);
    var atIndex = Math.max(0, Math.min(next.cols, index + (after ? 1 : 0)));
    next.grid.forEach(function (row) { row.splice(atIndex, 0, emptyCell()); });
    next.cols += 1;
    if (next.options && next.options.widths && next.options.widths.length) {
      next.options.widths.splice(atIndex, 0, 0);
    }
    return ensureGrid(next);
  };

  STM.tableDeleteCol = function (model, index) {
    if (model.cols <= 1) return clone(model);
    var next = STM.tableExpandAll(model);
    if (index < 0 || index >= next.cols) return next;
    next.grid.forEach(function (row) { row.splice(index, 1); });
    next.cols -= 1;
    if (next.options && next.options.widths && next.options.widths.length) {
      next.options.widths.splice(index, 1);
    }
    return ensureGrid(next);
  };

  STM.tableMerge = function (model, r1, c1, r2, c2) {
    var next = STM.tableExpandAll(model);
    var top = Math.min(r1, r2), bottom = Math.max(r1, r2);
    var left = Math.min(c1, c2), right = Math.max(c1, c2);
    var anchor = at(next, top, left);
    if (!anchor) return next;
    anchor.colspan = right - left + 1;
    anchor.rowspan = bottom - top + 1;
    for (var r = top; r <= bottom; r++) {
      for (var c = left; c <= right; c++) {
        if (r === top && c === left) continue;
        next.grid[r][c] = null;
      }
    }
    return ensureGrid(next);
  };

  STM.tableSplit = function (model, r, c) {
    var next = STM.tableExpandAll(model);
    var cell = at(next, r, c);
    if (cell) { cell.colspan = 1; cell.rowspan = 1; }
    return next;
  };

  function addCell(row, cell, r, c, options) {
    var isHeader = !!(options && options.header && r === 0);
    var el = document.createElement(isHeader ? "th" : "td");
    el.setAttribute("scope", isHeader ? "col" : "row");
    el.setAttribute("data-stm-r", String(r));
    el.setAttribute("data-stm-c", String(c));
    if (cell.colspan > 1) el.colSpan = cell.colspan;
    if (cell.rowspan > 1) el.rowSpan = cell.rowspan;
    if (cell.style) el.setAttribute("data-stm-cell-style", cell.style);
    el.textContent = cell.text || "";
    row.appendChild(el);
  }

  STM.tableRender = function (table, model, options) {
    if (!table) return;
    var opts = Object.assign({}, DEFAULT_OPTIONS, options || {});
    var cols = Math.max(1, model.cols || 1);
    table.innerHTML = "";
    table.className = "stm-table";
    table.setAttribute("data-stm-options", JSON.stringify(opts));
    table.setAttribute("data-stm-edited", "1");
    table.classList.add("stm-tbl-align-" + (opts.align || "left"));
    table.classList.add("stm-tbl-style-" + (opts.style || "plain"));
    if (opts.header) table.classList.add("stm-tbl-header");
    if (opts.banded) table.classList.add("stm-tbl-banded");
    if (!opts.borders) table.classList.add("stm-tbl-noborders");
    table.style.setProperty("--stm-pad", (Number(opts.padding) || 0) + "px");

    var colgroup = document.createElement("colgroup");
    for (var ci = 0; ci < cols; ci++) {
      var col = document.createElement("col");
      var width = opts.widths && opts.widths[ci];
      if (width) col.style.width = width + "px";
      colgroup.appendChild(col);
    }
    table.appendChild(colgroup);

    var body = document.createElement("tbody");
    for (var r = 0; r < model.grid.length; r++) {
      var tr = document.createElement("tr");
      for (var c = 0; c < cols; c++) {
        var cell = model.grid[r][c];
        if (cell) addCell(tr, cell, r, c, opts);
      }
      body.appendChild(tr);
    }
    table.appendChild(body);
  };

  STM.tableDefaults = function () { return clone(DEFAULT_OPTIONS); };

  STM.persistTable = function (table, model, options) {
    var id = STM.elementIdOf(table);
    if (!id) return;
    STM.patch(function (data) {
      data.tables[id] = {
        model: clone(model),
        options: clone(options || model.options || DEFAULT_OPTIONS),
        local: true
      };
    });
  };

  STM.restoreTable = function (table) {
    var id = STM.elementIdOf(table);
    if (!id) return false;
    var saved = STM.load().tables[id];
    if (!saved || !saved.model) return false;
    STM.tableRender(table, saved.model, saved.options);
    return true;
  };

  STM.tableWidths = function (model, options, values) {
    var next = clone(options || DEFAULT_OPTIONS);
    next.widths = [];
    for (var i = 0; i < (model.cols || 0); i++) {
      var value = parseFloat(values[i]);
      next.widths.push(isFinite(value) && value > 0 ? value : 0);
    }
    return next;
  };
})();
