"use strict";
/* Sheets structured data (#2436) — local slicer bound to a column of a range.
 * Filters only the rows shown in this panel; the grid itself is not mutated. */

(function () {
  const S = window.SheetStructured;
  const t = S.t;
  let activeDialog = null;

  function closeActive() {
    if (activeDialog) {
      activeDialog.close();
      activeDialog = null;
    }
  }

  function readTable(rangeText) {
    const range = S.parseRange(rangeText);
    if (!range) return { range: null, headers: [], rows: [] };
    const headers = [];
    for (let c = range.start.col; c <= range.end.col; c++) {
      headers.push(String(S.cellValue(range.start.row, c) || ""));
    }
    const rows = [];
    for (let r = range.start.row + 1; r <= range.end.row; r++) {
      const row = [];
      for (let c = range.start.col; c <= range.end.col; c++) {
        row.push(String(S.cellValue(r, c) || ""));
      }
      rows.push(row);
    }
    return { range: range, headers: headers, rows: rows };
  }

  function columnIndex(table, column) {
    const raw = String(column || "").trim();
    if (/^\d+$/.test(raw)) {
      const pos = parseInt(raw, 10) - 1;
      return pos >= 0 && pos < table.headers.length ? pos : -1;
    }
    const lowered = raw.toLowerCase();
    return table.headers.findIndex(function (header) { return header.toLowerCase() === lowered; });
  }

  function distinctValues(table, index) {
    if (index < 0) return [];
    const seen = Object.create(null);
    const out = [];
    table.rows.forEach(function (row) {
      const value = row[index] || "";
      if (!seen[value]) {
        seen[value] = true;
        out.push(value);
      }
    });
    return out;
  }

  function fieldRow(labelKey, fallback, control) {
    const row = document.createElement("label");
    row.className = "ss-structured-field";
    const caption = document.createElement("span");
    caption.textContent = t(labelKey, fallback);
    caption.setAttribute("data-i18n", labelKey);
    control.style.width = "100%";
    row.appendChild(caption);
    row.appendChild(control);
    return row;
  }

  function button(labelKey, fallback, className) {
    const node = document.createElement("button");
    node.type = "button";
    node.className = className || "ss-structured-btn";
    node.textContent = t(labelKey, fallback);
    node.setAttribute("data-i18n", labelKey);
    return node;
  }

  function openList() {
    closeActive();
    const slicers = S.state().slicers;
    const dialog = S.createDialog("sheet-structured-slicer", "Slicer");
    activeDialog = dialog;
    dialog.body.appendChild(S.localNote());

    const form = document.createElement("div");
    form.className = "ss-structured-form";
    const rangeInput = document.createElement("input");
    rangeInput.type = "text";
    rangeInput.className = "ss-structured-input";
    const columnInput = document.createElement("input");
    columnInput.type = "text";
    columnInput.className = "ss-structured-input";
    form.appendChild(fieldRow("sheet-structured-slicer-range", "Source range", rangeInput));
    form.appendChild(fieldRow("sheet-structured-slicer-column", "Column header", columnInput));
    dialog.body.appendChild(form);

    const add = button("sheet-structured-slicer-apply", "Add slicer", "ss-structured-btn ss-structured-primary");
    add.addEventListener("click", function () {
      const rangeText = rangeInput.value.trim().toUpperCase();
      const column = columnInput.value.trim();
      const table = readTable(rangeText);
      if (!table.range || columnIndex(table, column) < 0) {
        dialog.setStatus(t("sheet-structured-spark-invalid", "Enter a target cell and a valid source range."), true);
        rangeInput.focus();
        return;
      }
      const entry = { id: "slicer-" + Date.now(), range: rangeText, column: column, selected: [] };
      S.mutate(function (state) { state.slicers.push(entry); });
      openSlicer(entry);
    });
    dialog.footer.appendChild(add);

    const list = document.createElement("div");
    list.className = "ss-structured-list";
    if (!slicers.length) {
      const empty = document.createElement("p");
      empty.className = "ss-structured-empty";
      empty.textContent = t("sheet-structured-slicer-none", "No slicers yet.");
      empty.setAttribute("data-i18n", "sheet-structured-slicer-none");
      list.appendChild(empty);
    } else {
      slicers.forEach(function (entry) {
        const row = document.createElement("div");
        row.className = "ss-structured-list-row";
        const label = document.createElement("span");
        label.textContent = entry.column + " · " + entry.range;
        row.appendChild(label);
        const open = button("sheet-structured-slicer", "Slicer", "ss-structured-btn");
        open.addEventListener("click", function () { openSlicer(entry); });
        row.appendChild(open);
        const remove = button("sheet-structured-spark-remove", "Remove", "ss-structured-btn ss-structured-danger");
        remove.addEventListener("click", function () {
          S.mutate(function (state) {
            state.slicers = state.slicers.filter(function (item) { return item.id !== entry.id; });
          });
          openList();
        });
        row.appendChild(remove);
        list.appendChild(row);
      });
    }
    dialog.body.appendChild(list);
    rangeInput.focus();
  }

  function openSlicer(entry) {
    closeActive();
    const dialog = S.createDialog("sheet-structured-slicer", "Slicer");
    activeDialog = dialog;
    dialog.body.appendChild(S.localNote());
    const table = readTable(entry.range);
    const index = columnIndex(table, entry.column);
    const values = distinctValues(table, index);
    const selected = Object.create(null);
    (entry.selected || []).forEach(function (value) { selected[value] = true; });

    const chooser = document.createElement("div");
    chooser.className = "ss-structured-slicer-values";
    chooser.setAttribute("role", "group");
    const all = button("sheet-structured-slicer-all", "All", "ss-structured-btn");
    all.setAttribute("aria-pressed", Object.keys(selected).length ? "false" : "true");
    chooser.appendChild(all);
    const valueButtons = [];
    values.forEach(function (value) {
      const chip = document.createElement("button");
      chip.type = "button";
      chip.className = "ss-structured-btn ss-structured-chip";
      chip.textContent = value || "—";
      chip.setAttribute("aria-pressed", selected[value] ? "true" : "false");
      chip.addEventListener("click", function () {
        if (selected[value]) delete selected[value];
        else selected[value] = true;
        chip.setAttribute("aria-pressed", selected[value] ? "true" : "false");
        all.setAttribute("aria-pressed", Object.keys(selected).length ? "false" : "true");
        renderRows();
      });
      chooser.appendChild(chip);
      valueButtons.push(chip);
    });
    all.addEventListener("click", function () {
      Object.keys(selected).forEach(function (key) { delete selected[key]; });
      valueButtons.forEach(function (chip) { chip.setAttribute("aria-pressed", "false"); });
      all.setAttribute("aria-pressed", "true");
      renderRows();
    });
    dialog.body.appendChild(chooser);

    const counter = document.createElement("p");
    counter.className = "ss-structured-slicer-count";
    counter.setAttribute("role", "status");
    counter.setAttribute("aria-live", "polite");
    dialog.body.appendChild(counter);

    const tableHost = document.createElement("div");
    tableHost.className = "ss-structured-table-host";
    dialog.body.appendChild(tableHost);

    function renderRows() {
      const active = Object.keys(selected);
      const rows = active.length
        ? table.rows.filter(function (row) { return selected[row[index] || ""]; })
        : table.rows;
      counter.textContent = t("sheet-structured-slicer-rows", "Showing {shown} of {total} rows", {
        shown: String(rows.length), total: String(table.rows.length),
      });
      tableHost.innerHTML = "";
      if (!rows.length) {
        const empty = document.createElement("p");
        empty.className = "ss-structured-empty";
        empty.textContent = t("sheet-structured-slicer-empty", "No rows match the current selection.");
        empty.setAttribute("data-i18n", "sheet-structured-slicer-empty");
        tableHost.appendChild(empty);
        return;
      }
      const html = ["<table class=\"ss-structured-grid\"><thead><tr>"];
      table.headers.forEach(function (header) {
        html.push("<th scope=\"col\">" + S.escapeHtml(header) + "</th>");
      });
      html.push("</tr></thead><tbody>");
      rows.slice(0, 300).forEach(function (row) {
        html.push("<tr>");
        row.forEach(function (value) { html.push("<td>" + S.escapeHtml(value) + "</td>"); });
        html.push("</tr>");
      });
      html.push("</tbody></table>");
      tableHost.innerHTML = html.join("");
    }

    const clear = button("sheet-structured-slicer-clear", "Clear selection", "ss-structured-btn");
    clear.addEventListener("click", function () {
      Object.keys(selected).forEach(function (key) { delete selected[key]; });
      valueButtons.forEach(function (chip) { chip.setAttribute("aria-pressed", "false"); });
      all.setAttribute("aria-pressed", "true");
      renderRows();
    });
    dialog.footer.appendChild(clear);
    renderRows();
  }

  window.SheetStructuredSlicer = { open: openList, table: readTable, openSlicer: openSlicer };
})();
