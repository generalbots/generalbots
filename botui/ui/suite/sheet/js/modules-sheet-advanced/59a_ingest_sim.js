"use strict";
/* Sheets ingest (#2494) — deterministic, in-browser SIMULATED parse results.
 * No document bytes are read. The fixture includes three pages, a repeated
 * header, a subtotal, a merged marker, low-confidence cells and one unreadable
 * cell so the review workflow is exercised without a parser or OCR engine. */

(function () {
  function area(x, y, w, h) {
    return { x: x, y: y, w: w, h: h };
  }

  function cell(value, confidence, page, region, unreadable) {
    return {
      value: value == null ? "" : String(value),
      confidence: confidence,
      page: page,
      region: region,
      unreadable: Boolean(unreadable),
    };
  }

  function headerRow(page, names) {
    return names.map(function (name, index) {
      return cell(name, 0.99, page, area(40 + index * 110, 60, 100, 18));
    });
  }

  function statementParse(name) {
    var headers = ["Date", "Description", "Debit", "Credit", "Balance"];
    var rows = [headerRow(1, headers)];
    rows.push([
      cell("01/03/2026", 0.98, 1, area(40, 90, 100, 18)),
      cell("Opening balance", 0.97, 1, area(150, 90, 260, 18)),
      cell("", 0.96, 1, area(410, 90, 90, 18)),
      cell("", 0.96, 1, area(500, 90, 90, 18)),
      cell("1000.00", 0.98, 1, area(590, 90, 100, 18)),
    ]);
    rows.push([
      cell("03/03/2026", 0.96, 1, area(40, 110, 100, 18)),
      cell("Office supplies", 0.93, 1, area(150, 110, 260, 18)),
      cell("120.50", 0.62, 1, area(410, 110, 90, 18)),
      cell("", 0.95, 1, area(500, 110, 90, 18)),
      cell("", 0.95, 1, area(590, 110, 100, 18)),
    ]);
    rows.push([
      cell("05/03/2026", 0.97, 1, area(40, 130, 100, 18)),
      cell("Client payment", 0.94, 1, area(150, 130, 260, 18)),
      cell("", 0.96, 1, area(410, 130, 90, 18)),
      cell("500.00", 0.97, 1, area(500, 130, 90, 18)),
      cell("", 0.96, 1, area(590, 130, 100, 18)),
    ]);
    rows.push([
      cell("07/03/2026", 0.91, 2, area(40, 80, 100, 18)),
      cell("Unreadable charge", 0.88, 2, area(150, 80, 260, 18)),
      cell("", 0.30, 2, area(410, 80, 90, 18), true),
      cell("", 0.95, 2, area(500, 80, 90, 18)),
      cell("", 0.95, 2, area(590, 80, 100, 18)),
    ]);
    rows.push([
      cell("09/03/2026", 0.95, 2, area(40, 100, 100, 18)),
      cell("Subscription", 0.92, 2, area(150, 100, 260, 18)),
      cell("45.00", 0.94, 2, area(410, 100, 90, 18)),
      cell("", 0.96, 2, area(500, 100, 90, 18)),
      cell("", 0.96, 2, area(590, 100, 100, 18)),
    ]);
    rows.push([
      cell("SUBTOTAL", 0.92, 2, area(40, 130, 100, 18)),
      cell("", 0.90, 2, area(150, 130, 260, 18)),
      cell("165.50", 0.91, 2, area(410, 130, 90, 18)),
      cell("500.00", 0.91, 2, area(500, 130, 90, 18)),
      cell("", 0.90, 2, area(590, 130, 100, 18)),
    ]);
    rows.push(headerRow(3, headers));
    rows.push([
      cell("11/03/2026", 0.94, 3, area(40, 90, 100, 18)),
      cell("Final fee", 0.90, 3, area(150, 90, 260, 18)),
      cell("12.00", 0.93, 3, area(410, 90, 90, 18)),
      cell("", 0.96, 3, area(500, 90, 90, 18)),
      cell("1300.00", 0.78, 3, area(590, 90, 100, 18)),
    ]);
    rows.push([
      cell("12/03/2026", 0.96, 3, area(40, 110, 100, 18)),
      cell("Closing balance", 0.95, 3, area(150, 110, 260, 18)),
      cell("", 0.96, 3, area(410, 110, 90, 18)),
      cell("", 0.96, 3, area(500, 110, 90, 18)),
      cell("1322.50", 0.97, 3, area(590, 110, 100, 18)),
    ]);
    return {
      name: name,
      kind: "bank",
      pages: 3,
      columns: headers.map(function (header, index) {
        return {
          index: index,
          header: header,
          sample: rows[1][index].value,
          target: ["date", "text", "currency", "currency", "currency"][index],
        };
      }),
      rows: rows,
      structures: [
        { kind: "subtotal", row: 6, page: 2, col: 0, colSpan: 1 },
        { kind: "merged", row: 6, page: 2, col: 1, colSpan: 2 },
        { kind: "repeated-header", row: 7, page: 3, col: 0, colSpan: 5 },
      ],
    };
  }

  function invoiceParse(name) {
    var headers = ["Item", "Qty", "Unit price", "Amount"];
    var rows = [headerRow(1, headers)];
    rows.push([
      cell("Paper A4", 0.96, 1, area(40, 90, 200, 18)),
      cell("2", 0.94, 1, area(240, 90, 60, 18)),
      cell("12.00", 0.95, 1, area(300, 90, 90, 18)),
      cell("24.00", 0.55, 1, area(390, 90, 90, 18)),
    ]);
    rows.push([
      cell("Toner", 0.93, 1, area(40, 110, 200, 18)),
      cell("1", 0.95, 1, area(240, 110, 60, 18)),
      cell("60.00", 0.94, 1, area(300, 110, 90, 18)),
      cell("", 0.20, 1, area(390, 110, 90, 18), true),
    ]);
    return {
      name: name,
      kind: "invoice",
      pages: 1,
      columns: headers.map(function (header, index) {
        return {
          index: index,
          header: header,
          sample: rows[1][index].value,
          target: ["text", "number", "currency", "currency"][index],
        };
      }),
      rows: rows,
      structures: [],
    };
  }

  function simulateParse(name) {
    return /invoice/i.test(String(name || "")) ? invoiceParse(name) : statementParse(name);
  }

  function ensureParse(target, fileId) {
    if (target.parses[fileId]) return target.parses[fileId];
    var file = target.files.filter(function (item) { return item.id === fileId; })[0];
    target.parses[fileId] = simulateParse(file ? file.name : fileId);
    return target.parses[fileId];
  }

  window.SheetIngestSim = {
    simulateParse: simulateParse,
    ensureParse: ensureParse,
  };
})();
