"use strict";
/* Docs rich table structural operations (#2482). Row/column insert/delete,
 * merge/split and header-row conversion. Every operation edits the placement
 * model, re-lays-out the grid and commits through one undoable replacement. */
(function (window) {
  var NS = (window.DocsTables = window.DocsTables || {});
  var OPS = (NS.ops = NS.ops || {});

  function say(key, fallback) { NS.status(NS.tr(key, fallback)); }
  function commit(model, focus) {
    if (!model) return false;
    return !!NS.replaceTable(model.table, NS.modelToTable(model), focus);
  }
  function requireCell() {
    var cur = NS.cellFromSelection();
    if (!cur) say("docs-tables-no-table", "Place the cursor in a table to edit it");
    return cur;
  }

  OPS.insertRow = function (where) {
    var cur = requireCell();
    if (!cur) return false;
    var model = cur.model;
    if (!model.rows) return false;
    var at = where === "below" ? cur.row + 1 : cur.row;
    if (at < 0) at = 0;
    if (at > model.rows) at = model.rows;
    var section = model.rowSections[Math.min(cur.row, model.rows - 1)] || "tbody";
    var occ = NS.layout(model);
    for (var c = 0; c < model.cols; c++) {
      var entry = occ[at] && occ[at][c];
      if (entry && entry.originRow < at) entry.ref.rowSpan += 1;
    }
    model.placements.forEach(function (p) { if (p.row >= at) p.row += 1; });
    model.rows += 1;
    model.rowSections.splice(at, 0, section);
    model.rowAttrs.splice(at, 0, []);
    var nextOcc = NS.layout(model);
    for (c = 0; c < model.cols; c++) {
      if (!nextOcc[at][c]) model.placements.push({ ref: NS.blankRef(section), row: at, col: c });
    }
    commit(model, { row: at, col: cur.col });
    say("docs-tables-row-inserted", "Row inserted");
    return true;
  };

  OPS.deleteRow = function () {
    var cur = requireCell();
    if (!cur) return false;
    var model = cur.model;
    if (model.rows <= 1) { say("docs-tables-cannot-delete", "A table must keep at least one row"); return false; }
    var at = cur.row;
    var occ = NS.layout(model);
    var remove = [];
    for (var c = 0; c < model.cols; c++) {
      var entry = occ[at] && occ[at][c];
      if (!entry) continue;
      if (entry.originRow < at) {
        entry.ref.rowSpan = Math.max(1, entry.ref.rowSpan - 1);
      } else if (entry.originRow === at) {
        if (entry.ref.rowSpan > 1) entry.ref.rowSpan -= 1;
        else remove.push(entry.ref);
      }
    }
    model.placements = model.placements.filter(function (p) { return remove.indexOf(p.ref) < 0; });
    model.placements.forEach(function (p) { if (p.row > at) p.row -= 1; });
    model.rowSections.splice(at, 1);
    model.rowAttrs.splice(at, 1);
    model.rows -= 1;
    commit(model, { row: Math.min(at, model.rows - 1), col: cur.col });
    say("docs-tables-row-deleted", "Row deleted");
    return true;
  };

  OPS.insertCol = function (where) {
    var cur = requireCell();
    if (!cur) return false;
    var model = cur.model;
    var at = where === "right" ? cur.col + 1 : cur.col;
    if (at < 0) at = 0;
    if (at > model.cols) at = model.cols;
    var occ = NS.layout(model);
    for (var r = 0; r < model.rows; r++) {
      var entry = occ[r] && occ[r][at];
      if (entry && entry.originCol < at) entry.ref.colSpan += 1;
    }
    model.placements.forEach(function (p) { if (p.col >= at) p.col += 1; });
    model.cols += 1;
    var nextOcc = NS.layout(model);
    for (r = 0; r < model.rows; r++) {
      if (!nextOcc[r][at]) {
        model.placements.push({ ref: NS.blankRef(model.rowSections[r] || "tbody"), row: r, col: at });
      }
    }
    commit(model, { row: cur.row, col: at });
    say("docs-tables-col-inserted", "Column inserted");
    return true;
  };

  OPS.deleteCol = function () {
    var cur = requireCell();
    if (!cur) return false;
    var model = cur.model;
    if (model.cols <= 1) { say("docs-tables-cannot-delete-col", "A table must keep at least one column"); return false; }
    var at = cur.col;
    var occ = NS.layout(model);
    var remove = [];
    for (var r = 0; r < model.rows; r++) {
      var entry = occ[r] && occ[r][at];
      if (!entry) continue;
      if (entry.originCol < at) {
        entry.ref.colSpan = Math.max(1, entry.ref.colSpan - 1);
      } else if (entry.originCol === at) {
        if (entry.ref.colSpan > 1) entry.ref.colSpan -= 1;
        else remove.push(entry.ref);
      }
    }
    model.placements = model.placements.filter(function (p) { return remove.indexOf(p.ref) < 0; });
    model.placements.forEach(function (p) { if (p.col > at) p.col -= 1; });
    model.cols -= 1;
    commit(model, { row: cur.row, col: Math.min(at, model.cols - 1) });
    say("docs-tables-col-deleted", "Column deleted");
    return true;
  };

  OPS.merge = function () {
    var cur = requireCell();
    if (!cur) return false;
    var rect = NS.selectionRect(cur.table);
    if (!rect || (rect.minR === rect.maxR && rect.minC === rect.maxC)) {
      say("docs-tables-no-merge", "Select at least two cells to merge");
      return false;
    }
    var model = rect.model;
    var occ = NS.layout(model);
    var refs = [];
    for (var rr = rect.minR; rr <= rect.maxR; rr++) {
      for (var cc = rect.minC; cc <= rect.maxC; cc++) {
        var entry = occ[rr] && occ[rr][cc];
        if (entry && refs.indexOf(entry.ref) < 0) refs.push(entry.ref);
      }
    }
    if (refs.length < 2) { say("docs-tables-no-merge", "Select at least two cells to merge"); return false; }
    var minR = rect.minR, minC = rect.minC, maxR = rect.maxR, maxC = rect.maxC;
    model.placements.forEach(function (p) {
      if (refs.indexOf(p.ref) < 0) return;
      minR = Math.min(minR, p.row);
      minC = Math.min(minC, p.col);
      maxR = Math.max(maxR, p.row + p.ref.rowSpan - 1);
      maxC = Math.max(maxC, p.col + p.ref.colSpan - 1);
    });
    var trapped = model.placements.some(function (p) {
      return refs.indexOf(p.ref) < 0 && p.row >= minR && p.row <= maxR && p.col >= minC && p.col <= maxC;
    });
    if (trapped) { say("docs-tables-no-merge", "Select at least two cells to merge"); return false; }
    var ordered = model.placements.filter(function (p) { return refs.indexOf(p.ref) >= 0; })
      .sort(function (a, b) { return a.row - b.row || a.col - b.col; });
    var first = ordered[0].ref;
    var html = refs.map(function (ref) { return ref.html || ""; })
      .filter(function (h) { return h.replace(/&nbsp;/g, "").trim(); }).join(" ");
    var tag = refs.every(function (ref) { return ref.tag === "th"; }) ? "th" : "td";
    var scope = tag === "th"
      ? (first.sectionTag === "thead" ? (maxC > minC ? "colgroup" : "col")
         : (maxR > minR ? "row" : ""))
      : "";
    var merged = {
      tag: tag, html: html || "&nbsp;", scope: scope,
      rowSpan: maxR - minR + 1, colSpan: maxC - minC + 1,
      sectionTag: first.sectionTag, attrs: first.attrs.slice(), _idx: first._idx,
    };
    model.placements = model.placements.filter(function (p) { return refs.indexOf(p.ref) < 0; });
    model.placements.push({ ref: merged, row: minR, col: minC });
    commit(model, { row: minR, col: minC });
    say("docs-tables-cells-merged", "Cells merged");
    return true;
  };

  OPS.split = function () {
    var cur = requireCell();
    if (!cur) return false;
    var model = cur.model;
    var indexes = NS.selectedCells(cur.table).map(function (cell) {
      return NS.ownCells(cur.table).indexOf(cell);
    });
    var didSplit = false;
    model.placements.slice().forEach(function (p) {
      if (indexes.indexOf(p.ref._idx) < 0) return;
      var rs = p.ref.rowSpan, cs = p.ref.colSpan;
      if (rs === 1 && cs === 1) return;
      p.ref.rowSpan = 1;
      p.ref.colSpan = 1;
      for (var dr = 0; dr < rs; dr++) {
        for (var dc = 0; dc < cs; dc++) {
          if (dr === 0 && dc === 0) continue;
          model.placements.push({
            ref: NS.blankRef(model.rowSections[p.row + dr] || p.ref.sectionTag),
            row: p.row + dr, col: p.col + dc,
          });
        }
      }
      didSplit = true;
    });
    if (!didSplit) { say("docs-tables-no-split", "The selected cell is not merged"); return false; }
    commit(model, { row: cur.row, col: cur.col });
    say("docs-tables-cells-split", "Cells split");
    return true;
  };

  OPS.toggleHeader = function () {
    var cur = requireCell();
    if (!cur) return false;
    var model = cur.model;
    if (!model.rows) return false;
    if (model.rowSections[0] === "thead") {
      model.rowSections[0] = "tbody";
      model.placements.forEach(function (p) {
        if (p.row !== 0) return;
        p.ref.tag = "td";
        p.ref.scope = "";
        p.ref.sectionTag = "tbody";
      });
      commit(model, { row: 0, col: cur.col });
      say("docs-tables-header-cleared", "Header row removed");
      return true;
    }
    var occ = NS.layout(model);
    for (var c = 0; c < model.cols; c++) {
      var entry = occ[0][c];
      if (entry && entry.originRow !== 0) {
        say("docs-tables-no-header-row", "The row cannot be used as a header");
        return false;
      }
    }
    model.rowSections[0] = "thead";
    model.placements.forEach(function (p) {
      if (p.row !== 0) return;
      p.ref.tag = "th";
      p.ref.sectionTag = "thead";
      p.ref.scope = p.ref.colSpan > 1 ? "colgroup" : "col";
    });
    commit(model, { row: 0, col: cur.col });
    say("docs-tables-header-set", "Header row applied");
    return true;
  };
})(window);
