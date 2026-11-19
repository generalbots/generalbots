"use strict";
/* Sheets ingest (#2494) — column mapping, bank-statement profile detection and
 * the running-balance check. All outputs are local preview observations. */

(function () {
  var C = window.SheetIngestCore;

  function t(key, fallback, params) { return C.t(key, fallback, params); }

  function typeLabel(type) {
    var map = {
      text: ["sheet-ingest-type-text", "Text"],
      number: ["sheet-ingest-type-number", "Number"],
      date: ["sheet-ingest-type-date", "Date"],
      currency: ["sheet-ingest-type-currency", "Currency"],
    };
    var entry = map[type] || map.text;
    return t(entry[0], entry[1]);
  }

  function renderMapping(host, ctx) {
    var current = ctx.state();
    var fileId = current.activeFileId;
    var parse = current.parses[fileId];
    if (!parse) {
      host.appendChild(C.element("p", "ss-ingest-muted",
        t("sheet-ingest-batch-empty", "No files in the batch.")));
      return;
    }
    var types = C.columnTypes(current, fileId);
    var section = C.element("section", "ss-ingest-section");
    section.appendChild(C.element("h3", "ss-ingest-section-title",
      t("sheet-ingest-map-title", "Map detected columns")));
    section.appendChild(C.element("p", "ss-ingest-note",
      t("sheet-ingest-map-note", "Choose a target type for each detected column.")));
    var head = C.element("div", "ss-ingest-map-row ss-ingest-map-head");
    [["sheet-ingest-map-column", "Column"],
      ["sheet-ingest-map-detected", "Detected header"],
      ["sheet-ingest-map-sample", "Sample"],
      ["sheet-ingest-map-target", "Target type"]].forEach(function (pair) {
      head.appendChild(C.element("span", "", t(pair[0], pair[1])));
    });
    section.appendChild(head);
    parse.columns.forEach(function (column, index) {
      var row = C.element("div", "ss-ingest-map-row");
      row.appendChild(C.element("span", "", C.colName(index)));
      row.appendChild(C.element("span", "", column.header));
      row.appendChild(C.element("span", "ss-ingest-muted", column.sample));
      var select = C.element("select", "ss-ingest-select");
      select.dataset.index = String(index);
      select.setAttribute("aria-label", t("sheet-ingest-map-target", "Target type") +
        " " + C.colName(index));
      C.TYPES.forEach(function (type) {
        var option = C.element("option", "", typeLabel(type));
        option.value = type;
        if (types[index] === type) option.selected = true;
        select.appendChild(option);
      });
      row.appendChild(select);
      section.appendChild(row);
    });
    var save = C.element("button", "ss-ingest-btn", t("sheet-ingest-map-save", "Save mapping"));
    save.type = "button";
    save.addEventListener("click", function () {
      var chosen = Array.prototype.slice.call(section.querySelectorAll("select")).map(function (node) {
        return C.oneOf(node.value, C.TYPES, "text");
      });
      C.mutate(function (target) { target.mappings[fileId] = chosen; });
      ctx.setStatus(t("sheet-ingest-map-saved", "Column mapping saved to the local preview."), false);
      ctx.rerender();
    });
    section.appendChild(save);
    host.appendChild(section);
  }

  function renderProfile(host, ctx) {
    var current = ctx.state();
    var parse = current.parses[current.activeFileId];
    if (!parse) return;
    var profile = C.detectProfile(parse);
    var section = C.element("section", "ss-ingest-section");
    section.appendChild(C.element("h3", "ss-ingest-section-title",
      t("sheet-ingest-profile-title", "Bank statement profile")));
    var cols = profile.columns;
    var labels = {
      date: C.colName(cols.date),
      description: C.colName(cols.description),
      debit: C.colName(cols.debit),
      credit: C.colName(cols.credit),
      balance: C.colName(cols.balance),
    };
    if (profile.kind === "bank") {
      section.appendChild(C.element("p", "ss-ingest-badge-ok",
        t("sheet-ingest-profile-bank", "Bank statement detected: date, description, debit, credit and balance columns.")));
      section.appendChild(C.element("p", "ss-ingest-muted",
        t("sheet-ingest-profile-columns",
          "Date { $date }, description { $description }, debit { $debit }, credit { $credit }, balance { $balance }",
          labels)));
    } else {
      section.appendChild(C.element("p", "ss-ingest-muted",
        t("sheet-ingest-profile-generic", "No bank-statement column pattern detected.")));
    }
    host.appendChild(section);
  }

  function renderBalance(host, ctx) {
    var current = ctx.state();
    var fileId = current.activeFileId;
    if (!current.parses[fileId]) return;
    var result = C.runningBalance(current, fileId);
    var section = C.element("section", "ss-ingest-section");
    section.appendChild(C.element("h3", "ss-ingest-section-title",
      t("sheet-ingest-balance-title", "Running-balance check")));
    if (result.kind !== "bank") {
      section.appendChild(C.element("p", "ss-ingest-muted",
        t("sheet-ingest-balance-empty", "No rows available for the check.")));
      host.appendChild(section);
      return;
    }
    if (result.blocked) {
      section.appendChild(C.element("p", "ss-ingest-warning",
        t("sheet-ingest-balance-blocked", "{ $count } row(s) blocked by unconfirmed low-confidence cells.",
          { count: result.blocked })));
    }
    if (result.mismatch) {
      section.appendChild(C.element("p", "ss-ingest-mismatch",
        t("sheet-ingest-balance-mismatch", "{ $count } row(s) with a balance mismatch.",
          { count: result.mismatch })));
    } else if (!result.blocked) {
      section.appendChild(C.element("p", "ss-ingest-badge-ok",
        t("sheet-ingest-balance-ok", "Running balance matches on all readable rows.")));
    }
    if (!result.available) {
      section.appendChild(C.element("p", "ss-ingest-muted",
        t("sheet-ingest-balance-empty", "No rows available for the check.")));
    }
    result.rows.forEach(function (item) {
      var text = item.blocked
        ? t("sheet-ingest-balance-blocked-row", "Row { $row }: blocked by an unconfirmed low-confidence cell.",
          { row: item.row + 1 })
        : t("sheet-ingest-balance-row", "Row { $row }: expected { $expected }, statement { $stated }", {
          row: item.row + 1,
          expected: item.expected.toFixed(2),
          stated: item.stated == null ? "" : item.stated.toFixed(2),
        });
      section.appendChild(C.element("p", "ss-ingest-detail", text));
    });
    host.appendChild(section);
  }

  function render(body, ctx) {
    var list = C.element("div", "ss-ingest-stack");
    renderMapping(list, ctx);
    renderProfile(list, ctx);
    renderBalance(list, ctx);
    body.appendChild(list);
  }

  window.SheetIngestMap = { render: render, typeLabel: typeLabel };
})();
