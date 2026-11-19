"use strict";
/* Docs rich table presentation + conversion operations (#2482). Styles are
 * theme-bound classes only: no colour is ever written into a cell inline. */
(function (window) {
  var NS = (window.DocsTables = window.DocsTables || {});
  var OPS = (NS.ops = NS.ops || {});
  var PAD_CLASSES = ["docs-tbl-pad-compact", "", "docs-tbl-pad-relaxed"];
  var VALIGN_CLASSES = ["docs-cell-valign-top", "docs-cell-valign-middle", "docs-cell-valign-bottom"];

  function say(key, fallback) { NS.status(NS.tr(key, fallback)); }
  function current() { return NS.cellFromSelection(); }
  function commit(model) {
    return !!NS.replaceTable(model.table, NS.modelToTable(model));
  }
  function cloneCommit(table, mutate) {
    if (!table) return false;
    var clone = table.cloneNode(true);
    mutate(clone);
    return !!NS.replaceTable(table, clone);
  }
  function classAttr(ref) {
    for (var i = 0; i < ref.attrs.length; i++) {
      if (ref.attrs[i][0] === "class") return ref.attrs[i];
    }
    var pair = ["class", ""];
    ref.attrs.push(pair);
    return pair;
  }
  function refClass(ref, name, force) {
    var pair = classAttr(ref);
    var list = String(pair[1] || "").split(/\s+/).filter(function (c) { return c && c !== name; });
    var has = String(pair[1] || "").split(/\s+/).indexOf(name) >= 0;
    var want = force === undefined ? !has : !!force;
    if (want) list.push(name);
    pair[1] = list.join(" ");
    return want;
  }
  function styleAttr(ref) {
    for (var i = 0; i < ref.attrs.length; i++) {
      if (ref.attrs[i][0] === "style") return ref.attrs[i];
    }
    var pair = ["style", ""];
    ref.attrs.push(pair);
    return pair;
  }
  function setWidth(ref, px) {
    var pair = styleAttr(ref);
    var decls = String(pair[1] || "").split(";").map(function (d) { return d.trim(); })
      .filter(function (d) { return d && !/^width\s*:/i.test(d); });
    if (px > 0) decls.push("width:" + Math.round(px) + "px");
    pair[1] = decls.join(";");
  }
  function refsForSelection() {
    var cur = current();
    if (!cur) { say("docs-tables-no-table", "Place the cursor in a table to edit it"); return null; }
    var indexes = NS.selectedCells(cur.table).map(function (cell) {
      return NS.ownCells(cur.table).indexOf(cell);
    });
    if (indexes.indexOf(cur.ref._idx) < 0) indexes.push(cur.ref._idx);
    return { model: cur.model, cur: cur, indexes: indexes };
  }
  function textOf(html) {
    return String(html || "").replace(/<[^>]*>/g, "")
      .replace(/&nbsp;/gi, " ").replace(/&amp;/gi, "&")
      .replace(/&lt;/gi, "<").replace(/&gt;/gi, ">").trim().toLocaleLowerCase();
  }

  OPS.toggleBanded = function () {
    var table = NS.activeTable();
    if (!table) { say("docs-tables-no-table", "Place the cursor in a table to edit it"); return false; }
    return cloneCommit(table, function (clone) { NS.toggleClass(clone, "docs-tbl-banded"); });
  };

  OPS.toggleBorders = function () {
    var table = NS.activeTable();
    if (!table) { say("docs-tables-no-table", "Place the cursor in a table to edit it"); return false; }
    return cloneCommit(table, function (clone) { NS.toggleClass(clone, "docs-tbl-noborders"); });
  };

  OPS.toggleShading = function () {
    var sel = refsForSelection();
    if (!sel) return false;
    sel.model.placements.forEach(function (p) {
      if (sel.indexes.indexOf(p.ref._idx) >= 0) refClass(p.ref, "docs-cell-shade");
    });
    return commit(sel.model);
  };

  OPS.cellPadding = function (delta) {
    var table = NS.activeTable();
    if (!table) { say("docs-tables-no-table", "Place the cursor in a table to edit it"); return false; }
    var currentName = PAD_CLASSES.filter(function (name) {
      return name && (table.getAttribute("class") || "").split(/\s+/).indexOf(name) >= 0;
    })[0] || "";
    var index = PAD_CLASSES.indexOf(currentName);
    var next = PAD_CLASSES[Math.max(0, Math.min(PAD_CLASSES.length - 1, index + delta))];
    return cloneCommit(table, function (clone) {
      PAD_CLASSES.forEach(function (name) { if (name) NS.toggleClass(clone, name, false); });
      if (next) NS.toggleClass(clone, next, true);
    });
  };

  OPS.vAlign = function (value) {
    var sel = refsForSelection();
    if (!sel) return false;
    var wanted = "docs-cell-valign-" + value;
    if (VALIGN_CLASSES.indexOf(wanted) < 0) return false;
    sel.model.placements.forEach(function (p) {
      if (sel.indexes.indexOf(p.ref._idx) < 0) return;
      VALIGN_CLASSES.forEach(function (name) { refClass(p.ref, name, name === wanted); });
    });
    return commit(sel.model);
  };

  OPS.setColumnWidth = function (px) {
    var sel = refsForSelection();
    if (!sel) return false;
    var col = sel.cur.col;
    sel.model.placements.forEach(function (p) {
      if (p.col === col) setWidth(p.ref, px);
    });
    say("docs-tables-width-applied", "Column width applied");
    return commit(sel.model);
  };

  OPS.autofit = function () {
    var sel = refsForSelection();
    if (!sel) return false;
    sel.model.placements.forEach(function (p) { setWidth(p.ref, 0); });
    var newTable = NS.modelToTable(sel.model);
    newTable.removeAttribute("style");
    return !!NS.replaceTable(sel.model.table, newTable);
  };

  OPS.applyStyle = function (name) {
    var table = NS.activeTable();
    if (!table) return false;
    if (NS.STYLE_CLASSES.indexOf(name) < 0) return false;
    var applied = cloneCommit(table, function (clone) {
      NS.STYLE_CLASSES.forEach(function (style) { NS.toggleClass(clone, style, false); });
      NS.toggleClass(clone, name, true);
    });
    if (applied) say("docs-tables-style-applied", "Table style applied");
    return applied;
  };

  OPS.sort = function (desc) {
    var cur = current();
    if (!cur) return false;
    var model = cur.model;
    var hasSpans = model.placements.some(function (p) {
      return p.ref.rowSpan > 1 || p.ref.colSpan > 1;
    });
    if (hasSpans) { say("docs-tables-sort-spans", "Unmerge cells before sorting the table"); return false; }
    var order = [];
    for (var r = 0; r < model.rows; r++) order.push(r);
    var bodyRows = order.filter(function (row) { return model.rowSections[row] === "tbody"; });
    var occ = NS.layout(model);
    var col = cur.col;
    bodyRows.sort(function (a, b) {
      var ea = occ[a][col], eb = occ[b][col];
      var ta = ea ? textOf(ea.ref.html) : "", tb = eb ? textOf(eb.ref.html) : "";
      var cmp = ta < tb ? -1 : (ta > tb ? 1 : 0);
      return desc ? -cmp : cmp;
    });
    var nextBody = 0;
    var newOrder = order.map(function (original) {
      if (model.rowSections[original] !== "tbody") return original;
      return bodyRows[nextBody++];
    });
    var map = {};
    newOrder.forEach(function (original, index) { map[original] = index; });
    var sections = newOrder.map(function (original) { return model.rowSections[original]; });
    var attrs = newOrder.map(function (original) { return model.rowAttrs[original] || []; });
    model.placements.forEach(function (p) { p.row = map[p.row]; });
    model.rowSections = sections;
    model.rowAttrs = attrs;
    commit(model);
    say("docs-tables-sorted", "Table sorted");
    return true;
  };

  OPS.textToTable = function (text, separator) {
    var lines = String(text || "").replace(/\r/g, "").split("\n").filter(function (line) {
      return line.trim().length > 0;
    });
    if (!lines.length) return false;
    var delimiter = lines.some(function (line) { return line.indexOf("\t") >= 0; })
      ? "\t" : (separator || ",");
    var matrix = lines.map(function (line) { return line.split(delimiter); });
    var cols = matrix.reduce(function (max, row) { return Math.max(max, row.length); }, 1);
    var model = {
      table: document.createElement("table"), placements: [], rowSections: [], rowAttrs: [],
      sectionAttrs: { thead: [], tbody: [], tfoot: [] }, cols: cols, rows: matrix.length,
    };
    matrix.forEach(function (cells, r) {
      var section = r === 0 ? "thead" : "tbody";
      model.rowSections.push(section);
      model.rowAttrs.push([]);
      cells.forEach(function (value, c) {
        var ref = NS.blankRef(section);
        ref.html = NS.esc(value);
        model.placements.push({ ref: ref, row: r, col: c });
      });
    });
    var table = NS.modelToTable(model);
    if (window.DocsAuthoringCore && typeof window.DocsAuthoringCore.insertNodeAtCaret === "function") {
      window.DocsAuthoringCore.insertNodeAtCaret(table);
    } else {
      var article = NS.getArticle();
      if (article) article.appendChild(table);
    }
    if (window.DocsTablesUI && typeof window.DocsTablesUI.refresh === "function") {
      window.DocsTablesUI.refresh(table);
    }
    say("docs-tables-converted-table", "Text converted to a table");
    return true;
  };

  OPS.tableToText = function () {
    var table = NS.activeTable();
    if (!table) { say("docs-tables-no-table", "Place the cursor in a table to edit it"); return false; }
    var model = NS.buildModel(table);
    var occ = NS.layout(model);
    var lines = [];
    for (var r = 0; r < model.rows; r++) {
      var values = [];
      for (var c = 0; c < model.cols; c++) {
        var entry = occ[r][c];
        if (!entry || !entry.start) continue;
        var cell = NS.ownCells(table)[entry.ref._idx];
        values.push(cell ? NS.cellText(cell) : "");
      }
      lines.push(values.join("\t"));
    }
    var html = "<p>" + lines.map(function (line) { return NS.esc(line); }).join("<br>") + "</p>";
    if (!NS.replaceNode(table, html)) return false;
    say("docs-tables-converted-text", "Table converted to text");
    return true;
  };
})(window);
