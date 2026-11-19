"use strict";
/* Mail merge dialog (#2427): paste/upload CSV, map columns to local fields,
 * browse records and run a combined local preview. */
(function (window) {
  var NS = window.DocsForms;
  if (!NS) return;
  var D = NS.dialog;
  var M = NS.merge;
  if (!D || !M) return;
  var UI = NS.dialogs;

  UI.openMerge = function () {
    var shell = D.shell({ titleKey: "docs-forms-merge-title", titleFallback: "Mail merge" });
    var source = D.input("textarea", "docs-forms-merge-source-placeholder",
      "Paste CSV here; the first row holds column names", NS.state.csvText);
    shell.body.appendChild(D.field("docs-forms-merge-source", "Data source (CSV text)", source));

    var upload = D.el("input", "gb-forms-input");
    upload.setAttribute("type", "file");
    upload.setAttribute("data-i18n-aria-label", "docs-forms-merge-upload");
    upload.setAttribute("aria-label", NS.tr("docs-forms-merge-upload", "Upload CSV file"));
    upload.addEventListener("change", function () {
      var file = upload.files && upload.files[0];
      if (!file || !window.FileReader) return;
      var reader = new window.FileReader();
      reader.onload = function () { source.value = String(reader.result || ""); load(); };
      reader.readAsText(file);
    });
    shell.body.appendChild(upload);

    var load = D.button("docs-forms-merge-load", "Load data source", true);
    shell.body.appendChild(load);

    var mapHost = D.el("div", "gb-forms-merge-map");
    var recordBar = D.el("div", "gb-forms-merge-record-bar");
    var recordText = D.el("pre", "gb-forms-merge-record");
    var result = D.el("pre", "gb-forms-merge-result gb-forms-preview-note");
    shell.body.appendChild(mapHost);
    shell.body.appendChild(recordBar);
    shell.body.appendChild(recordText);
    shell.body.appendChild(result);

    var records = [];
    var headers = [];

    function refreshRecord() {
      var view = M.previewRecord(records, NS.state.recordIndex);
      NS.state.recordIndex = view.index;
      recordBar.textContent = view.record
        ? NS.tr("docs-forms-merge-record", "Record { $current } of { $total }",
            { current: view.index + 1, total: records.length })
        : NS.tr("docs-forms-merge-no-data", "No data rows to merge");
      if (!view.record) { recordText.textContent = ""; return; }
      recordText.textContent = NS.state.controls.map(function (field) {
        var header = Object.keys(NS.state.mapping).filter(function (name) {
          return NS.state.mapping[name] === field.id;
        })[0];
        return String(field.label || field.id) + ": " + (header ? (view.record[header] || "") : "");
      }).join("\n");
    }

    function renderMap() {
      mapHost.textContent = "";
      if (!headers.length) return;
      var title = D.el("div", "gb-forms-merge-map-title");
      D.i18n(title, "docs-forms-merge-map", "Map columns to fields");
      mapHost.appendChild(title);
      headers.forEach(function (header) {
        var row = D.el("div", "gb-forms-merge-map-row");
        var name = D.el("span", "gb-forms-merge-column-name");
        name.textContent = header;
        row.appendChild(name);
        var select = D.el("select", "gb-forms-input");
        select.setAttribute("data-form-map-column", header);
        var none = D.el("option");
        none.setAttribute("value", "");
        none.textContent = "\u2014";
        select.appendChild(none);
        NS.state.controls.forEach(function (field) {
          var option = D.el("option");
          option.setAttribute("value", field.id);
          option.textContent = field.label || field.id;
          select.appendChild(option);
        });
        select.value = NS.state.mapping[header] || "";
        select.addEventListener("change", function () {
          NS.state.mapping[header] = select.value;
          NS.persist();
          refreshRecord();
        });
        row.appendChild(select);
        mapHost.appendChild(row);
      });
    }

    function load() {
      var parsed = M.load(source.value);
      records = parsed.records;
      headers = parsed.parsed.headers;
      renderMap();
      refreshRecord();
      shell.status.textContent = NS.tr("docs-forms-merge-count", "{ $count } records loaded",
        { count: records.length });
    }

    var prev = D.button("docs-forms-merge-prev", "Previous");
    prev.addEventListener("click", function () {
      NS.state.recordIndex = Math.max(0, NS.state.recordIndex - 1);
      refreshRecord();
    });
    var next = D.button("docs-forms-merge-next", "Next");
    next.addEventListener("click", function () {
      NS.state.recordIndex = Math.min(Math.max(0, records.length - 1), NS.state.recordIndex + 1);
      refreshRecord();
    });
    var run = D.button("docs-forms-merge-run", "Merge all records", true);
    run.addEventListener("click", function () {
      if (!records.length) {
        shell.status.textContent = NS.tr("docs-forms-merge-no-data", "No data rows to merge");
        return;
      }
      var outcome = M.run(records, NS.state.controls, NS.state.mapping);
      NS.state.merged = outcome.text;
      NS.persist();
      result.textContent = outcome.text;
      shell.status.textContent = outcome.missing.length
        ? NS.tr("docs-forms-control-missing-required", "Required field is empty: { $label }",
            { label: outcome.missing[0].label })
        : NS.tr("docs-forms-merge-result", "Combined document preview");
    });

    var nav = D.el("div", "gb-forms-merge-nav");
    nav.appendChild(prev);
    nav.appendChild(next);
    shell.body.appendChild(nav);
    shell.body.appendChild(run);
    load.addEventListener("click", load);
    refreshRecord();
    return shell.close;
  };
})(window);
