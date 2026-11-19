"use strict";
/* Docs large .docx load path (#2533), baseline measurement display.
 *
 * A tabular LOCAL ESTIMATE derived from the deterministic fixture. Values are
 * formulas over block count, not measured browser/server numbers. The real
 * measured baseline (peak RSS + JSHeapUsedSize, TTFP, keystroke latency,
 * autosave duration, wire bytes) requires a compiler/backend and is DEFERRED. */
(function (window) {
  var NS = window.DocsLarge;
  if (!NS) return;
  var B = (NS.Baseline = NS.Baseline || {});

  B.COLUMNS = [
    { key: "docs-large-col-size", fallback: "Size" },
    { key: "docs-large-col-heap", fallback: "Peak heap est." },
    { key: "docs-large-col-ttfp", fallback: "Time to first paint" },
    { key: "docs-large-col-keystroke", fallback: "Keystroke latency" },
    { key: "docs-large-col-autosave", fallback: "Autosave duration" },
    { key: "docs-large-col-wire", fallback: "Wire bytes" }
  ];

  function estimate(sizeMb) {
    var size = Number(sizeMb) || 0;
    var blocks = NS.Fixture.blockCount(size);
    return {
      size: size + " MB",
      blocks: blocks,
      heap: Math.round(size * 6) + " MB",
      ttfp: (0.4 + size * 0.12).toFixed(1) + " s",
      keystroke: Math.round(6 + size * 0.4) + " ms",
      autosave: Math.round(20 + size * 8) + " ms",
      wire: NS.Fixture.bytesForMb(size) + " B"
    };
  }
  B.estimate = estimate;

  function rows() { return NS.SIZES_MB.map(estimate); }
  B.rows = rows;

  function localLabel() {
    return NS.tr("docs-large-baseline-local", "LOCAL ESTIMATE — not measured browser/server numbers");
  }
  B.localLabel = localLabel;

  function render(host) {
    if (!host) return;
    host.textContent = "";
    var title = NS.i18n(NS.el("h4", "gb-large-baseline-title"),
      "docs-large-baseline-title", "Baseline (local estimate)");
    host.appendChild(title);
    var note = NS.el("p", "gb-large-baseline-note");
    note.setAttribute("data-i18n", "docs-large-baseline-local");
    note.textContent = localLabel();
    host.appendChild(note);
    var table = NS.el("table", "gb-large-baseline-table");
    var thead = NS.el("thead");
    var headRow = NS.el("tr");
    B.COLUMNS.forEach(function (column) {
      var th = NS.i18n(NS.el("th"), column.key, column.fallback);
      th.setAttribute("scope", "col");
      headRow.appendChild(th);
    });
    thead.appendChild(headRow);
    table.appendChild(thead);
    var tbody = NS.el("tbody");
    rows().forEach(function (row) {
      var tr = NS.el("tr");
      [row.size, row.heap, row.ttfp, row.keystroke, row.autosave, row.wire].forEach(function (value) {
        var cell = NS.el("td");
        cell.textContent = value;
        tr.appendChild(cell);
      });
      tbody.appendChild(tr);
    });
    table.appendChild(tbody);
    host.appendChild(table);
    var deferred = NS.el("p", "gb-large-deferred");
    deferred.textContent = NS.deferredNote();
    host.appendChild(deferred);
  }
  B.render = render;
})(typeof window !== "undefined" ? window : globalThis);
