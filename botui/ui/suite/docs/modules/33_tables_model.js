"use strict";
/* Docs rich table model (#2482).
 *
 * Tables are edited as a rectangular grid of *placements*. A placement is one
 * real <th>/<td> sitting at (row, col) with a rowSpan/colSpan; the cells it
 * covers are derived, never stored twice. Structural operations edit the
 * placement list and re-lay-out the grid, so the emitted HTML always has
 * balanced thead/tbody/tr/th/td, valid scope and consistent spans.
 *
 * Everything lives in window.DocsTables. All DOM changes go through
 * replaceTable(), which replaces the table with one gbFormat("insertHTML") call
 * (a single native undo step) plus a fallback for stubs/older engines. */
(function (window) {
  var NS = (window.DocsTables = window.DocsTables || {});
  var STYLE_CLASSES = [
    "docs-tbl-style-plain", "docs-tbl-style-grid", "docs-tbl-style-banded",
    "docs-tbl-style-accent", "docs-tbl-style-minimal",
  ];

  function nav() { return window.DocsNav || {}; }
  function tr(key, fallback, params) {
    var n = nav();
    if (typeof n.tr === "function") return n.tr(key, fallback, params);
    if (typeof docsT === "function") return docsT(key, fallback, params);
    if (window.gbT) return window.gbT(key, fallback, params);
    return fallback || key;
  }
  NS.tr = tr;
  function esc(value) {
    var n = nav();
    return typeof n.escapeHtml === "function" ? n.escapeHtml(value) : String(value == null ? "" : value);
  }
  NS.esc = esc;
  function getArticle() {
    var n = nav();
    if (typeof n.getArticle === "function") return n.getArticle();
    return document.querySelector("article[contenteditable]");
  }
  NS.getArticle = getArticle;

  function status(message) {
    var ui = window.DocsTablesUI;
    if (ui && typeof ui.setStatus === "function") ui.setStatus(message);
    else if (typeof nav().status === "function") nav().status(message);
  }
  NS.status = status;

  function save() {
    if (typeof scheduleSave === "function") {
      var article = getArticle();
      if (article) scheduleSave(article.dataset.docId, article.innerHTML);
    }
    if (typeof updatePageCount === "function") updatePageCount();
  }
  NS.save = save;

  var seq = 0;
  function uid() { seq += 1; return "dt" + Date.now().toString(36) + seq; }
  NS.uid = uid;

  /* ---- small DOM helpers that also work on the offline harness stubs ---- */
  function children(el) {
    var out = [];
    if (!el || !el.childNodes) return out;
    for (var i = 0; i < el.childNodes.length; i++) {
      if (el.childNodes[i].nodeType === 1) out.push(el.childNodes[i]);
    }
    return out;
  }
  function isTag(el, tag) {
    return !!el && !!el.tagName && String(el.tagName).toLowerCase() === tag;
  }
  function direct(el, tag) {
    var all = children(el);
    for (var i = 0; i < all.length; i++) if (isTag(all[i], tag)) return all[i];
    return null;
  }
  function directAll(el, tags) {
    return children(el).filter(function (c) { return tags.indexOf(String(c.tagName || "").toLowerCase()) >= 0; });
  }
  function closestEl(node, tag) {
    while (node && (node.nodeType === 3 ? (node = node.parentNode) : node.nodeType === 1)) {
      if (isTag(node, tag)) return node;
      node = node.parentNode;
    }
    return null;
  }
  function ownCell(node) {
    while (node && (node.nodeType === 3 ? (node = node.parentNode) : node.nodeType === 1)) {
      if (isTag(node, "td") || isTag(node, "th")) return node;
      node = node.parentNode;
    }
    return null;
  }
  function ownCells(table) {
    if (!table) return [];
    var all = table.querySelectorAll ? table.querySelectorAll("td,th") : [];
    return Array.prototype.filter.call(all, function (cell) {
      return closestEl(cell, "table") === table;
    });
  }
  NS.ownCells = ownCells;
  NS.ownCell = ownCell;
  NS.closestEl = closestEl;

  function attrList(el, skip) {
    var out = [];
    var attrs = el && el.attributes;
    if (!attrs) return out;
    if (typeof attrs.length === "number") {
      for (var i = 0; i < attrs.length; i++) {
        if (skip.indexOf(attrs[i].name) < 0) out.push([attrs[i].name, attrs[i].value]);
      }
    } else {
      Object.keys(attrs).forEach(function (name) {
        if (skip.indexOf(name) < 0) out.push([name, attrs[name]]);
      });
    }
    return out;
  }
  function span(cell, name) {
    var value = cell && cell.getAttribute ? cell.getAttribute(name) : null;
    var n = parseInt(value, 10);
    return n > 0 ? n : 1;
  }
  function applyAttrs(el, attrs) {
    attrs.forEach(function (pair) { el.setAttribute(pair[0], pair[1]); });
  }
  function addClass(el, name) {
    var current = el.getAttribute("class") || "";
    if (current.split(/\s+/).indexOf(name) < 0) el.setAttribute("class", (current + " " + name).trim());
  }
  function removeClass(el, name) {
    var current = (el.getAttribute("class") || "").split(/\s+/).filter(function (c) { return c && c !== name; });
    el.setAttribute("class", current.join(" "));
  }
  function toggleClass(el, name, force) {
    var has = (el.getAttribute("class") || "").split(/\s+/).indexOf(name) >= 0;
    var want = force === undefined ? !has : !!force;
    if (want) addClass(el, name); else removeClass(el, name);
    return want;
  }
  NS.toggleClass = toggleClass;
  NS.STYLE_CLASSES = STYLE_CLASSES;

  /* ---- grid model ---- */
  function blankRef(sectionTag) {
    return {
      tag: sectionTag === "thead" ? "th" : "td",
      html: "&nbsp;",
      scope: sectionTag === "thead" ? "col" : "",
      rowSpan: 1,
      colSpan: 1,
      sectionTag: sectionTag,
      attrs: [],
      _idx: -1,
    };
  }
  NS.blankRef = blankRef;

  function refFromCell(cell, sectionTag) {
    return {
      tag: isTag(cell, "th") ? "th" : "td",
      html: cell.innerHTML || "&nbsp;",
      scope: cell.getAttribute("scope") || "",
      rowSpan: span(cell, "rowspan"),
      colSpan: span(cell, "colspan"),
      sectionTag: sectionTag,
      attrs: attrList(cell, ["rowspan", "colspan", "scope"]),
      _idx: -1,
    };
  }

  function buildModel(table) {
    if (!table) return null;
    var sections = [];
    var head = direct(table, "thead");
    var bodies = directAll(table, ["tbody"]);
    var foot = direct(table, "tfoot");
    if (head) sections.push({ tag: "thead", rows: directAll(head, ["tr"]), el: head });
    bodies.forEach(function (b) { sections.push({ tag: "tbody", rows: directAll(b, ["tr"]), el: b }); });
    if (foot) sections.push({ tag: "tfoot", rows: directAll(foot, ["tr"]), el: foot });
    if (!sections.length) {
      var loose = directAll(table, ["tr"]);
      if (loose.length) sections.push({ tag: "tbody", rows: loose, el: null });
    }
    var model = {
      table: table,
      placements: [],
      rowSections: [],
      rowAttrs: [],
      sectionAttrs: { thead: [], tbody: [], tfoot: [] },
      cols: 0,
      rows: 0,
    };
    Object.keys(model.sectionAttrs).forEach(function (tag) {
      sections.forEach(function (s) {
        if (s.tag === tag && s.el) model.sectionAttrs[tag] = attrList(s.el, []);
      });
    });
    var occ = [];
    var r = 0;
    var index = 0;
    sections.forEach(function (sec) {
      sec.rows.forEach(function (tr) {
        while (occ.length <= r) occ.push([]);
        model.rowSections.push(sec.tag);
        model.rowAttrs.push(attrList(tr, []));
        var c = 0;
        directAll(tr, ["td", "th"]).forEach(function (cell) {
          while (occ[r] && occ[r][c]) c++;
          var ref = refFromCell(cell, sec.tag);
          ref._idx = index++;
          model.placements.push({ ref: ref, row: r, col: c });
          for (var dr = 0; dr < ref.rowSpan; dr++) {
            for (var dc = 0; dc < ref.colSpan; dc++) {
              while (occ.length <= r + dr) occ.push([]);
              occ[r + dr][c + dc] = { ref: ref, start: dr === 0 && dc === 0 };
            }
          }
          c += ref.colSpan;
        });
        r++;
      });
    });
    model.rows = r;
    model.cols = 0;
    occ.forEach(function (row) { model.cols = Math.max(model.cols, row.length); });
    return model;
  }
  NS.buildModel = buildModel;

  function layout(model) {
    var occ = [];
    var r;
    for (r = 0; r < model.rows; r++) {
      occ.push(new Array(model.cols));
    }
    model.placements.forEach(function (p) {
      var ref = p.ref;
      for (var dr = 0; dr < ref.rowSpan; dr++) {
        for (var dc = 0; dc < ref.colSpan; dc++) {
          var rr = p.row + dr;
          var cc = p.col + dc;
          if (rr >= 0 && rr < model.rows && cc >= 0 && cc < model.cols) {
            occ[rr][cc] = { ref: ref, start: dr === 0 && dc === 0, originRow: p.row, originCol: p.col };
          }
        }
      }
    });
    return occ;
  }
  NS.layout = layout;

  function placementOf(model, ref) {
    for (var i = 0; i < model.placements.length; i++) {
      if (model.placements[i].ref === ref) return model.placements[i];
    }
    return null;
  }

  /* Serialize the model into a fresh <table> element. The caller commits it
   * with replaceTable(); no partial DOM mutations ever reach the article. */
  function modelToTable(model) {
    var table = document.createElement("table");
    applyAttrs(table, attrList(model.table, []));
    addClass(table, "docs-table");
    if (!table.getAttribute("data-docs-tables-id")) {
      table.setAttribute("data-docs-tables-id", uid());
    }
    var occ = layout(model);
    var serialIndex = 0;
    ["thead", "tbody", "tfoot"].forEach(function (tag) {
      var rows = [];
      for (var r = 0; r < model.rows; r++) if (model.rowSections[r] === tag) rows.push(r);
      if (!rows.length) return;
      var section = document.createElement(tag);
      applyAttrs(section, model.sectionAttrs[tag]);
      rows.forEach(function (r) {
        var tr = document.createElement("tr");
        applyAttrs(tr, model.rowAttrs[r] || []);
        for (var c = 0; c < model.cols; c++) {
          var entry = occ[r][c];
          if (!entry || !entry.start) continue;
          var ref = entry.ref;
          ref._idx = serialIndex++;
          var cell = document.createElement(ref.tag);
          applyAttrs(cell, ref.attrs);
          if (ref.rowSpan > 1) cell.setAttribute("rowspan", String(ref.rowSpan));
          if (ref.colSpan > 1) cell.setAttribute("colspan", String(ref.colSpan));
          if (ref.scope) cell.setAttribute("scope", ref.scope);
          cell.innerHTML = ref.html || "&nbsp;";
          tr.appendChild(cell);
        }
        section.appendChild(tr);
      });
      table.appendChild(section);
    });
    return table;
  }
  NS.modelToTable = modelToTable;

  /* Selection, caret and commit helpers live in 33b_tables_commit.js. */
})(window);
