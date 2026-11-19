"use strict";
/* Sheets print, reporting and scheduled delivery (#2438) — page-by-page
 * browser preview. The exported helpers are pure so pagination, repeated
 * titles and header/footer fields can be verified without a browser. */

(function () {
  var C = window.SheetReportingCore;
  var P = window.SheetReportingPrint;
  var t = C.t;
  var MAX_CELLS_X = 6;
  var MAX_CELLS_Y = 6;

  function variantFor(index, variant) {
    if (variant === "first") return index === 0;
    if (variant === "even") return index % 2 === 1;
    if (variant === "odd") return index % 2 === 0;
    return true;
  }

  function repeatRows(value) {
    var raw = String(value == null ? "" : value).trim();
    if (!raw) return [];
    var parts = /^([1-9][0-9]*)(?::([1-9][0-9]*))?$/.exec(raw);
    if (!parts) return [];
    var first = parseInt(parts[1], 10) - 1;
    var last = parseInt(parts[2] || parts[1], 10) - 1;
    if (last < first) return [];
    var rows = [];
    for (var row = first; row <= Math.min(last, first + 20); row++) rows.push(row);
    return rows;
  }

  function fieldText(field, ctx, index, plan) {
    var code = String(field == null ? "" : field);
    if (!code) return "";
    if (code === "page") {
      return t("sheet-rpt-field-page", "Page {page} of {total}", { page: index + 1, total: plan.pages });
    }
    if (code === "sheet") {
      return t("sheet-rpt-field-sheet", "Sheet: {sheet}", { sheet: ctx.sheetName || "" });
    }
    if (code === "file") {
      return t("sheet-rpt-field-file", "File: {file}", { file: ctx.fileName || "" });
    }
    if (code === "date") {
      return t("sheet-rpt-field-date", "Date: {date}", { date: ctx.date || "" });
    }
    return code;
  }

  function lineFor(section, index, ctx, plan) {
    var setup = C.normalizeSetup(ctx.setup);
    var config = setup[section];
    var variant = section === "header" ? setup.headerVariant : setup.footerVariant;
    var active = variantFor(index, variant);
    return {
      left: active ? fieldText(config.left, ctx, index, plan) : "",
      center: active ? fieldText(config.center, ctx, index, plan) : "",
      right: active ? fieldText(config.right, ctx, index, plan) : "",
    };
  }

  function pageCells(index, plan, repeatValue) {
    var colBand = plan.colBreaks[index % Math.max(1, plan.colPages)];
    var rowBand = plan.rowBreaks[Math.floor(index / Math.max(1, plan.colPages)) % Math.max(1, plan.rowPages)];
    if (!colBand || !rowBand) return [];
    var titles = repeatRows(repeatValue);
    var cells = [];
    titles.forEach(function (row) {
      for (var col = colBand.start; col <= Math.min(colBand.end, colBand.start + MAX_CELLS_X - 1); col++) {
        cells.push({ ref: C.numberToCol(col) + (row + 1), repeat: true });
      }
    });
    for (var row = rowBand.start; row <= Math.min(rowBand.end, rowBand.start + MAX_CELLS_Y - 1); row++) {
      if (titles.indexOf(row) >= 0) continue;
      for (var col = colBand.start; col <= Math.min(colBand.end, colBand.start + MAX_CELLS_X - 1); col++) {
        cells.push({ ref: C.numberToCol(col) + (row + 1), repeat: false });
      }
    }
    return cells.slice(0, 80);
  }

  function contextFromState() {
    var state = C.state();
    var areas = state.printAreas;
    var sheetName = "";
    if (window.SheetCore && typeof window.SheetCore.currentSheetName === "function") {
      sheetName = String(window.SheetCore.currentSheetName() || "");
    }
    if (!sheetName && window.__LOADED_SHEET && window.__LOADED_SHEET.name) {
      sheetName = String(window.__LOADED_SHEET.name);
    }
    return {
      setup: state.pageSetup,
      area: areas.length ? areas[0] : null,
      sheetName: sheetName,
      fileName: document.title || "",
      date: new Date().toISOString().slice(0, 10),
    };
  }

  function lineNode(className, values) {
    var row = C.element("div", className);
    ["left", "center", "right"].forEach(function (slot) {
      row.appendChild(C.element("span", "ss-rpt-page-slot", values[slot] || ""));
    });
    return row;
  }

  function render(container, ctx) {
    container.textContent = "";
    var source = ctx || contextFromState();
    var note = C.element("p", "ss-rpt-preview-note", t("sheet-rpt-preview-note",
      "Browser preview only - not a real PDF export; page breaks and layout are approximate."));
    note.setAttribute("data-i18n", "sheet-rpt-preview-note");
    container.appendChild(note);
    var plan = P.paginate(source.area, source.setup);
    if (!plan.pages) {
      container.appendChild(C.element("p", "ss-rpt-empty", t("sheet-rpt-preview-empty",
        "Define a print area to preview pages.")));
      return plan;
    }
    var summary = C.element("p", "ss-rpt-summary", t("sheet-rpt-preview-count",
      "{count} page(s) - {cols} column group(s) x {rows} row group(s).",
      { count: plan.pages, cols: plan.colPages, rows: plan.rowPages }));
    container.appendChild(summary);
    for (var index = 0; index < plan.pages; index++) {
      var page = C.element("article", "ss-rpt-page");
      page.setAttribute("aria-label", t("sheet-rpt-page-label", "Page {page} of {total}",
        { page: index + 1, total: plan.pages }));
      page.appendChild(lineNode("ss-rpt-page-header", lineFor("header", index, source, plan)));
      if (plan.repeatRows) {
        page.appendChild(C.element("div", "ss-rpt-page-titles", t("sheet-rpt-repeat-label",
          "Repeated title rows: {rows}", { rows: plan.repeatRows })));
      }
      var grid = C.element("div", "ss-rpt-page-grid");
      pageCells(index, plan, plan.repeatRows).forEach(function (cell) {
        grid.appendChild(C.element("span", cell.repeat ? "ss-rpt-page-cell ss-rpt-page-cell-title" : "ss-rpt-page-cell", cell.ref));
      });
      page.appendChild(grid);
      page.appendChild(lineNode("ss-rpt-page-footer", lineFor("footer", index, source, plan)));
      container.appendChild(page);
      if (index < plan.pages - 1) {
        container.appendChild(C.element("div", "ss-rpt-page-break", t("sheet-rpt-break-label", "Page break")));
      }
    }
    return plan;
  }

  window.SheetReportingPreview = {
    render: render,
    variantFor: variantFor,
    fieldText: fieldText,
    lineFor: lineFor,
    pageCells: pageCells,
    repeatRows: repeatRows,
  };
})();
