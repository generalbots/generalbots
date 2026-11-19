"use strict";
/* Sheets ingest (#2494) — parsed-grid preview, table-structure tags and the
 * per-cell provenance panel. Local-only rendering of a simulated parse. */

(function () {
  var C = window.SheetIngestCore;
  var selected = null;

  function t(key, fallback, params) { return C.t(key, fallback, params); }

  function badge(kind) {
    var map = {
      "repeated-header": ["sheet-ingest-struct-header", "Repeated header"],
      subtotal: ["sheet-ingest-struct-subtotal", "Subtotal row"],
      merged: ["sheet-ingest-struct-merged", "Merged cells"],
      low: ["sheet-ingest-low", "Low confidence"],
    };
    var entry = map[kind] || [kind, kind];
    return C.element("span", "ss-ingest-tag", t(entry[0], entry[1]));
  }

  function structureFor(parse, row, col) {
    for (var i = 0; i < parse.structures.length; i++) {
      var item = parse.structures[i];
      if (item.row !== row) continue;
      if (item.kind === "merged" && col >= item.col && col < item.col + item.colSpan) return item;
      if (item.kind !== "merged" && col === item.col) return item;
    }
    return null;
  }

  function cellNode(parse, fileId, row, col, ctx) {
    var cellValue = C.cellAt(parse, row, col);
    var button = C.element("button", "ss-ingest-cell");
    button.type = "button";
    button.dataset.row = String(row);
    button.dataset.col = String(col);
    var label = C.cellLabel(parse, row, col);
    var low = C.isLow(cellValue);
    var unreadable = C.isUnreadable(cellValue);
    var resolved = C.isResolved(window.SheetIngestCore.state(), fileId, row, col);
    if (low) button.classList.add("ss-ingest-cell-low");
    if (unreadable) button.classList.add("ss-ingest-cell-empty");
    if (resolved) button.classList.add("ss-ingest-cell-resolved");
    var tag = structureFor(parse, row, col);
    if (tag) button.classList.add("ss-ingest-cell-structured");
    button.textContent = unreadable ? "" : C.effectiveValue(window.SheetIngestCore.state(), fileId, row, col);
    var parts = [label, t("sheet-ingest-page", "Page { $page }", { page: cellValue ? cellValue.page : "?" })];
    if (low) parts.push(t("sheet-ingest-low", "Low confidence"));
    if (unreadable) parts.push(t("sheet-ingest-unreadable", "Unreadable - left empty"));
    button.setAttribute("aria-label", parts.join(" - "));
    if (selected && selected.fileId === fileId && selected.row === row && selected.col === col) {
      button.classList.add("ss-ingest-cell-selected");
    }
    button.addEventListener("click", function () { ctx.selectCell(fileId, row, col); });
    return button;
  }

  function renderGrid(host, parse, fileId, ctx) {
    var wrap = C.element("div", "ss-ingest-grid-wrap");
    var table = C.element("table", "ss-ingest-grid");
    var head = C.element("thead");
    var headRow = C.element("tr");
    headRow.appendChild(C.element("th", "ss-ingest-rowhead", ""));
    parse.columns.forEach(function (column) {
      var th = C.element("th", "", column.header);
      var small = C.element("small", "ss-ingest-col-hint", t("sheet-ingest-confidence-short",
        "conf. { $value }", { value: Number(column.confidence || 0.99).toFixed(2) }));
      th.appendChild(small);
      headRow.appendChild(th);
    });
    head.appendChild(headRow);
    var body = C.element("tbody");
    parse.rows.forEach(function (row, rowIndex) {
      var tr = C.element("tr");
      if (C.isDataRow(parse, rowIndex)) tr.classList.add("ss-ingest-row-data");
      var rowHead = C.element("th", "ss-ingest-rowhead", String(rowIndex + 1));
      var page = row[0] ? row[0].page : "?";
      rowHead.setAttribute("title", t("sheet-ingest-page", "Page { $page }", { page: page }));
      tr.appendChild(rowHead);
      row.forEach(function (unused, colIndex) {
        var td = C.element("td");
        td.appendChild(cellNode(parse, fileId, rowIndex, colIndex, ctx));
        tr.appendChild(td);
      });
      var tag = structureFor(parse, rowIndex, 0);
      if (tag) tr.classList.add("ss-ingest-row-" + tag.kind);
      body.appendChild(tr);
    });
    table.appendChild(head);
    table.appendChild(body);
    wrap.appendChild(table);
    host.appendChild(wrap);
  }

  function renderStructures(host, parse) {
    var box = C.element("div", "ss-ingest-structures");
    box.appendChild(C.element("h3", "ss-ingest-subtitle", t("sheet-ingest-structure-title", "Table structures")));
    if (!parse.structures.length) {
      box.appendChild(C.element("p", "ss-ingest-muted",
        t("sheet-ingest-structure-empty", "No special structures detected.")));
    } else {
      parse.structures.forEach(function (item) {
        var line = C.element("p", "ss-ingest-structure-item");
        line.appendChild(badge(item.kind));
        line.appendChild(document.createTextNode(" " + t("sheet-ingest-struct-item",
          "at row { $row }, page { $page }", { row: item.row + 1, page: item.page })));
        box.appendChild(line);
      });
    }
    host.appendChild(box);
  }

  function detailLine(host, key, fallback, value) {
    var line = C.element("p", "ss-ingest-detail");
    line.appendChild(C.element("strong", "", t(key, fallback) + ": "));
    line.appendChild(document.createTextNode(String(value == null ? "" : value)));
    host.appendChild(line);
  }

  function renderProvenance(host, ctx) {
    host.textContent = "";
    var panel = C.element("aside", "ss-ingest-provenance");
    panel.setAttribute("aria-label", t("sheet-ingest-provenance-title", "Cell provenance"));
    panel.appendChild(C.element("h3", "ss-ingest-subtitle", t("sheet-ingest-provenance-title", "Cell provenance")));
    var current = ctx.state();
    var parse = current.parses[current.activeFileId];
    if (!selected || !parse || selected.fileId !== current.activeFileId) {
      panel.appendChild(C.element("p", "ss-ingest-muted",
        t("sheet-ingest-provenance-hint", "Select a cell to see its source details.")));
      host.appendChild(panel);
      return;
    }
    var cellValue = C.cellAt(parse, selected.row, selected.col);
    if (!cellValue) {
      host.appendChild(panel);
      return;
    }
    var label = C.cellLabel(parse, selected.row, selected.col);
    detailLine(panel, "sheet-ingest-cell-header", "Cell", label);
    var region = cellValue.region || {};
    detailLine(panel, "sheet-ingest-source-page", "Source page", cellValue.page);
    detailLine(panel, "sheet-ingest-source-region", "Region",
      t("sheet-ingest-region", "x { $x }, y { $y }, { $w } x { $h }", {
        x: region.x, y: region.y, w: region.w, h: region.h,
      }));
    detailLine(panel, "sheet-ingest-source-confidence", "Confidence", Number(cellValue.confidence).toFixed(2));
    detailLine(panel, "sheet-ingest-source-original", "Original value",
      cellValue.unreadable || !cellValue.value
        ? t("sheet-ingest-source-empty", "No value read (left empty)") : cellValue.value);
    var corrected = C.overrideFor(current, selected.fileId, selected.row, selected.col);
    if (corrected) {
      detailLine(panel, "sheet-ingest-source-corrected", "Corrected value", corrected.corrected);
    }
    if (C.isLow(cellValue)) {
      panel.appendChild(badge("low"));
      if (C.isResolved(current, selected.fileId, selected.row, selected.col)) {
        panel.appendChild(C.element("span", "ss-ingest-badge-ok",
          t("sheet-ingest-confirmed", "Confirmed")));
      } else {
        panel.appendChild(C.element("p", "ss-ingest-warning",
          t("sheet-ingest-confirm-required", "Confirm this low-confidence value before a formula may use it.")));
        var confirm = C.element("button", "ss-ingest-btn",
          t("sheet-ingest-confirm", "Confirm value"));
        confirm.type = "button";
        confirm.addEventListener("click", function () {
          C.mutate(function (target) {
            target.confirmations[C.cellKey(selected.fileId, selected.row, selected.col)] = true;
          });
          ctx.setStatus(t("sheet-ingest-confirmed-status", "Cell confirmed for formula use."), false);
          ctx.rerender();
        });
        panel.appendChild(confirm);
      }
    }
    host.appendChild(panel);
  }

  function render(body, ctx) {
    var current = ctx.state();
    var fileId = current.activeFileId;
    var parse = current.parses[fileId];
    var section = C.element("section", "ss-ingest-section");
    section.appendChild(C.element("h3", "ss-ingest-section-title",
      t("sheet-ingest-preview-title", "Parsed grid preview")));
    section.appendChild(C.element("p", "ss-ingest-note",
      t("sheet-ingest-preview-note", "Simulated parse result. No PDF, image or CSV is actually read.")));
    if (!parse) {
      section.appendChild(C.element("p", "ss-ingest-muted",
        t("sheet-ingest-batch-empty", "No files in the batch.")));
      body.appendChild(section);
      return;
    }
    renderGrid(section, parse, fileId, ctx);
    renderStructures(section, parse);
    body.appendChild(section);
    var layout = C.element("div", "ss-ingest-with-panel");
    var holder = C.element("div", "ss-ingest-panel-holder");
    renderProvenance(holder, ctx);
    layout.appendChild(holder);
    body.appendChild(layout);
  }

  function select(fileId, row, col) {
    selected = { fileId: fileId, row: row, col: col };
  }

  function clear() { selected = null; }

  window.SheetIngestPreview = {
    render: render,
    select: select,
    clear: clear,
    renderProvenance: renderProvenance,
    selected: function () { return selected; },
  };
})();
