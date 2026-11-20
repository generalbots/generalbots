"use strict";
/* Sheets ingest (#2494) — correction overrides, provenance log and the local
 * sheet preview with number/date formats and computed running-balance formulas.
 * Nothing here writes to the workbook or to a server. */

(function () {
  var C = window.SheetIngestCore;

  function t(key, fallback, params) { return C.t(key, fallback, params); }

  function reviewableCells(parse, fileId) {
    var out = [];
    if (!parse) return out;
    parse.rows.forEach(function (row, rowIndex) {
      row.forEach(function (cellValue, colIndex) {
        if (C.isLow(cellValue) || C.isUnreadable(cellValue)) {
          out.push({ row: rowIndex, col: colIndex, cell: cellValue });
        }
      });
    });
    return out;
  }

  function renderCorrectionForm(host, ctx) {
    var current = ctx.state();
    var fileId = current.activeFileId;
    var parse = current.parses[fileId];
    var form = C.element("div", "ss-ingest-correction");
    form.appendChild(C.element("h4", "ss-ingest-subtitle",
      t("sheet-ingest-edit-title", "Correct a flagged value")));
    if (!parse) {
      form.appendChild(C.element("p", "ss-ingest-muted",
        t("sheet-ingest-batch-empty", "No files in the batch.")));
      host.appendChild(form);
      return;
    }
    var cells = reviewableCells(parse, fileId);
    if (!cells.length) {
      form.appendChild(C.element("p", "ss-ingest-muted",
        t("sheet-ingest-review-empty-cells", "No flagged cells in this parse.")));
      host.appendChild(form);
      return;
    }
    var picker = C.element("select", "ss-ingest-select");
    picker.id = "ss-ingest-correction-cell";
    picker.setAttribute("aria-label", t("sheet-ingest-cell-pick", "Cell to correct"));
    cells.forEach(function (item) {
      var option = C.element("option", "",
        C.cellLabel(parse, item.row, item.col) + " - " +
        (C.isUnreadable(item.cell)
          ? t("sheet-ingest-source-empty", "No value read (left empty)") : item.cell.value));
      option.value = item.row + ":" + item.col;
      picker.appendChild(option);
    });
    var label = C.element("label", "ss-ingest-field-label",
      t("sheet-ingest-cell-pick", "Cell to correct"));
    label.htmlFor = picker.id;
    form.appendChild(label);
    form.appendChild(picker);
    var valueLabel = C.element("label", "ss-ingest-field-label",
      t("sheet-ingest-correction-value", "Corrected value"));
    var input = C.element("input", "ss-ingest-input");
    input.type = "text";
    input.id = "ss-ingest-correction-value";
    valueLabel.htmlFor = input.id;
    form.appendChild(valueLabel);
    form.appendChild(input);
    var button = C.element("button", "ss-ingest-btn",
      t("sheet-ingest-save-correction", "Record correction"));
    button.type = "button";
    button.addEventListener("click", function () {
      var parts = picker.value.split(":");
      if (parts.length !== 2) return;
      var row = Number(parts[0]);
      var col = Number(parts[1]);
      var corrected = input.value.trim();
      if (!corrected) {
        ctx.setStatus(t("sheet-ingest-correction-required", "Enter a corrected value."), true);
        return;
      }
      var original = C.cellAt(parse, row, col);
      var key = C.cellKey(fileId, row, col);
      C.mutate(function (target) {
        target.overrides.push({
          key: key,
          fileId: fileId,
          row: row,
          col: col,
          cell: C.cellLabel(parse, row, col),
          original: C.isUnreadable(original) ? "" : original.value,
          corrected: corrected.slice(0, 200),
          author: t("sheet-ingest-author-local", "Local reviewer"),
          timestamp: new Date().toISOString(),
        });
      });
      input.value = "";
      ctx.setStatus(t("sheet-ingest-override-added", "Correction recorded for { $cell }.",
        { cell: C.cellLabel(parse, row, col) }), false);
      ctx.rerender();
    });
    form.appendChild(button);
    host.appendChild(form);
  }

  function renderLog(host, ctx) {
    var current = ctx.state();
    var list = C.element("div", "ss-ingest-log");
    list.appendChild(C.element("h4", "ss-ingest-subtitle",
      t("sheet-ingest-log-title", "Provenance and override log")));
    if (!current.overrides.length) {
      list.appendChild(C.element("p", "ss-ingest-muted",
        t("sheet-ingest-override-empty", "No corrections recorded.")));
    } else {
      current.overrides.forEach(function (item) {
        list.appendChild(C.element("p", "ss-ingest-detail",
          t("sheet-ingest-override-item",
            "{ $cell }: { $original } \u2192 { $corrected } by { $author } at { $time }", {
              cell: item.cell,
              original: item.original || t("sheet-ingest-source-empty", "No value read (left empty)"),
              corrected: item.corrected,
              author: item.author,
              time: item.timestamp,
            })));
      });
    }
    host.appendChild(list);
  }

  function balanceInputs(profile) {
    var cols = profile.columns;
    return [cols.balance, cols.debit, cols.credit].filter(function (col) { return col >= 0; });
  }

  function buildPreview(current, fileId) {
    var parse = current.parses[fileId];
    var profile = C.detectProfile(parse);
    var result = C.runningBalance(current, fileId);
    var mismatchRows = {};
    result.rows.forEach(function (item) { mismatchRows[item.row] = item; });
    var types = C.columnTypes(current, fileId);
    var inputs = balanceInputs(profile);
    var rows = [];
    for (var row = 1; row < parse.rows.length; row++) {
      if (!C.isDataRow(parse, row)) continue;
      var cells = parse.columns.map(function (column, col) {
        var raw = C.cellAt(parse, row, col);
        var unconfirmed = C.isLow(raw) && !C.isResolved(current, fileId, row, col);
        var blocked = unconfirmed || (inputs.indexOf(col) >= 0 && C.formulaBlocked(current, fileId, row, col));
        var flag = mismatchRows[row];
        var isBalance = profile.kind === "bank" && col === profile.columns.balance;
        var formula = !blocked && isBalance
          ? (row === 1
            ? "=" + C.colName(col) + (row + 1)
            : "=" + C.colName(col) + row + "+" +
              (profile.columns.credit >= 0 ? C.colName(profile.columns.credit) + (row + 1) : "0") +
              "-" + (profile.columns.debit >= 0 ? C.colName(profile.columns.debit) + (row + 1) : "0"))
          : "";
        var status = blocked ? "blocked" : (isBalance && flag && !flag.blocked ? "mismatch" : "ok");
        return {
          label: C.cellLabel(parse, row, col),
          value: blocked || C.isUnreadable(raw) ? "" : C.effectiveValue(current, fileId, row, col),
          format: types[col],
          formula: formula,
          status: status,
        };
      });
      rows.push({ row: row, cells: cells });
    }
    return { fileId: fileId, rows: rows, createdAt: new Date().toISOString() };
  }

  function renderPreview(host, ctx) {
    var current = ctx.state();
    var preview = current.preview;
    var section = C.element("section", "ss-ingest-section");
    section.appendChild(C.element("h3", "ss-ingest-section-title",
      t("sheet-ingest-preview-grid-title", "Local sheet preview (not saved)")));
    if (!preview) {
      section.appendChild(C.element("p", "ss-ingest-muted",
        t("sheet-ingest-preview-empty", "Create the preview to see formatted cells and formulas.")));
      host.appendChild(section);
      return;
    }
    var table = C.element("table", "ss-ingest-preview-grid");
    var head = C.element("tr");
    [["sheet-ingest-cell-header", "Cell"],
      ["sheet-ingest-value-header", "Effective value"],
      ["sheet-ingest-format-header", "Format"],
      ["sheet-ingest-formula", "Formula (running balance)"],
      ["sheet-ingest-status-header", "Status"]].forEach(function (pair) {
      head.appendChild(C.element("th", "", t(pair[0], pair[1])));
    });
    var thead = C.element("thead");
    thead.appendChild(head);
    table.appendChild(thead);
    var tbody = C.element("tbody");
    preview.rows.forEach(function (row) {
      row.cells.forEach(function (item) {
        var tr = C.element("tr");
        tr.appendChild(C.element("td", "", item.label));
        tr.appendChild(C.element("td", "", item.value || "\u2014"));
        tr.appendChild(C.element("td", "", item.format));
        tr.appendChild(C.element("td", "ss-ingest-formula-cell", item.formula || ""));
        var statusText = item.status === "blocked"
          ? t("sheet-ingest-formula-blocked", "Blocked - confirm low-confidence cells first")
          : item.status === "mismatch"
            ? t("sheet-ingest-status-mismatch", "Mismatch flagged")
            : t("sheet-ingest-status-ok", "Ready");
        var td = C.element("td", "ss-ingest-status-" + item.status, statusText);
        tr.appendChild(td);
        tbody.appendChild(tr);
      });
    });
    table.appendChild(tbody);
    section.appendChild(table);
    host.appendChild(section);
  }

  function render(body, ctx) {
    var section = C.element("section", "ss-ingest-section");
    section.appendChild(C.element("h3", "ss-ingest-section-title",
      t("sheet-ingest-review-title", "Corrections and override log")));
    section.appendChild(C.element("p", "ss-ingest-note",
      t("sheet-ingest-review-note", "Corrections are recorded, never silently applied.")));
    renderCorrectionForm(section, ctx);
    renderLog(section, ctx);
    var create = C.element("button", "ss-ingest-btn ss-ingest-btn-primary",
      t("sheet-ingest-create", "Create local sheet preview"));
    create.type = "button";
    create.addEventListener("click", function () {
      var current = ctx.state();
      var fileId = current.activeFileId;
      if (!current.parses[fileId]) {
        ctx.setStatus(t("sheet-ingest-batch-empty", "No files in the batch."), true);
        return;
      }
      var built = buildPreview(current, fileId);
      C.mutate(function (target) { target.preview = built; });
      ctx.setStatus(t("sheet-ingest-created", "Local preview sheet created with { $rows } row(s).",
        { rows: built.rows.length }), false);
      ctx.rerender();
    });
    section.appendChild(create);
    body.appendChild(section);
    renderPreview(body, ctx);
  }

  window.SheetIngestReview = {
    render: render,
    buildPreview: buildPreview,
    reviewableCells: reviewableCells,
  };
})();
