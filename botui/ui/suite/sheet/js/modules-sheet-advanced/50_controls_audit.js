"use strict";
/* Sheets enterprise controls (#2437) — cell-level local audit trail viewer.
 * Entries are browser-only preview records; no server audit log is written. */

(function () {
  var C = window.SheetControls;

  var HEADERS = [
    { key: "sheet-ctrl-audit-who", fallback: "Who" },
    { key: "sheet-ctrl-audit-when", fallback: "When" },
    { key: "sheet-ctrl-audit-cell", fallback: "Cell" },
    { key: "sheet-ctrl-audit-before", fallback: "Before" },
    { key: "sheet-ctrl-audit-after", fallback: "After" },
  ];

  function recordEdit(ref, before, after, formulaChanged, who) {
    C.mutate(function (state) {
      state.audit.entries.push({
        id: "e" + Date.now() + "-" + Math.floor(Math.random() * 100000),
        kind: "edit",
        who: who || "local",
        when: new Date().toISOString(),
        ref: String(ref || ""),
        before: String(before == null ? "" : before),
        after: String(after == null ? "" : after),
        formulaChanged: Boolean(formulaChanged),
      });
      pruneIn(state);
    });
    return true;
  }

  function recordEvent(kind, detailKey) {
    C.mutate(function (state) {
      state.audit.entries.push({
        id: "e" + Date.now() + "-" + Math.floor(Math.random() * 100000),
        kind: String(kind || "event"),
        who: "local",
        when: new Date().toISOString(),
        ref: "",
        before: "",
        after: "",
        detailKey: String(detailKey || ""),
      });
      pruneIn(state);
    });
    return true;
  }

  function pruneIn(state) {
    var cutoff = Date.now() - state.audit.retentionDays * 86400000;
    var before = state.audit.entries.length;
    state.audit.entries = state.audit.entries.filter(function (entry) {
      var time = Date.parse(entry.when);
      return Number.isFinite(time) ? time >= cutoff : true;
    });
    return before - state.audit.entries.length;
  }

  function prune() {
    return C.mutate(function (state) { return pruneIn(state); });
  }

  function setRetention(days) {
    var value = Number(days);
    if (!Number.isFinite(value) || value <= 0) return false;
    C.mutate(function (state) { state.audit.retentionDays = value; });
    prune();
    return true;
  }

  function clear() {
    C.mutate(function (state) { state.audit.entries = []; });
    return true;
  }

  function visible(filter) {
    var text = String(filter || "").trim().toUpperCase();
    var cutoff = Date.now() - C.state().audit.retentionDays * 86400000;
    return C.state().audit.entries.filter(function (entry) {
      var time = Date.parse(entry.when);
      if (Number.isFinite(time) && time < cutoff) return false;
      return !text || String(entry.ref || "").toUpperCase().indexOf(text) !== -1;
    });
  }

  function render(container, ctx) {
    var doc = container.ownerDocument || document;
    function make(tag, className, text) {
      var node = doc.createElement(tag);
      if (className) node.className = className;
      if (text != null) node.textContent = text;
      return node;
    }
    var grid = make("div", "ss-ctrl-form");
    var controls = make("div", "ss-ctrl-row");

    var retention = make("input", "ss-ctrl-input");
    retention.type = "number";
    retention.min = "1";
    retention.id = "ss-ctrl-audit-retention";
    retention.value = String(C.state().audit.retentionDays);
    var retentionLabel = make("label", "ss-ctrl-label", C.t("sheet-ctrl-audit-retention", "Retention window (days)"));
    retentionLabel.setAttribute("data-i18n", "sheet-ctrl-audit-retention");
    controls.appendChild(retentionLabel);
    controls.appendChild(retention);

    var filter = make("input", "ss-ctrl-input");
    filter.type = "text";
    filter.id = "ss-ctrl-audit-filter";
    filter.placeholder = C.t("sheet-ctrl-audit-filter", "Filter by cell or range");
    filter.setAttribute("data-i18n-placeholder", "sheet-ctrl-audit-filter");
    controls.appendChild(filter);

    var clearButton = make("button", "ss-ctrl-button", C.t("sheet-ctrl-audit-clear", "Clear local trail"));
    clearButton.type = "button";
    clearButton.id = "ss-ctrl-audit-clear";
    clearButton.setAttribute("data-i18n", "sheet-ctrl-audit-clear");
    controls.appendChild(clearButton);
    grid.appendChild(controls);

    var note = make("p", "ss-ctrl-scope-note", C.t("sheet-ctrl-audit-pruned", "Entries outside the retention window are hidden."));
    note.setAttribute("data-i18n", "sheet-ctrl-audit-pruned");
    grid.appendChild(note);

    var table = make("table", "ss-ctrl-table");
    var head = make("thead");
    var headRow = make("tr");
    HEADERS.forEach(function (header) {
      var th = make("th", "", C.t(header.key, header.fallback));
      th.setAttribute("data-i18n", header.key);
      headRow.appendChild(th);
    });
    head.appendChild(headRow);
    table.appendChild(head);
    var body = make("tbody");
    body.id = "ss-ctrl-audit-rows";
    table.appendChild(body);
    grid.appendChild(table);
    container.appendChild(grid);

    function drawRows() {
      body.textContent = "";
      var entries = visible(filter.value);
      if (!entries.length) {
        var row = make("tr");
        var cell = make("td", "ss-ctrl-empty", C.t("sheet-ctrl-audit-empty", "No local edits recorded."));
        cell.colSpan = 5;
        cell.setAttribute("data-i18n", "sheet-ctrl-audit-empty");
        row.appendChild(cell);
        body.appendChild(row);
        return;
      }
      entries.forEach(function (entry) {
        var row = make("tr");
        row.setAttribute("data-ref", entry.ref);
        [entry.who, entry.when, entry.ref, entry.before, entry.after].forEach(function (value) {
          row.appendChild(make("td", "", value));
        });
        body.appendChild(row);
      });
    }
    drawRows();

    retention.addEventListener("change", function () {
      var ok = setRetention(retention.value);
      ctx.setStatus(ok ? C.t("sheet-ctrl-status-ready", "Ready.") : C.t("sheet-ctrl-audit-invalid", "Enter a positive number of days."), !ok);
      drawRows();
    });
    filter.addEventListener("input", drawRows);
    clearButton.addEventListener("click", function () {
      clear();
      drawRows();
      ctx.setStatus(C.t("sheet-ctrl-status-ready", "Ready."), false);
    });
  }

  window.SheetControlsAudit = {
    recordEdit: recordEdit,
    recordEvent: recordEvent,
    prune: prune,
    setRetention: setRetention,
    clear: clear,
    visible: visible,
    render: render,
  };
})();
