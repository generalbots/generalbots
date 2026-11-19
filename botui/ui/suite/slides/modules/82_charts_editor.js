"use strict";
/* Slides chart editor (#2486) — layer 6a: dialog composition, type picker,
 * data grid and styling panel. Every control mutates a working copy; only Save
 * commits to deck-scoped localStorage. All labels come from catalogues. */
(function (window) {
  var C = (window.GBSlidesCharts = window.GBSlidesCharts || {});
  var E = (C.editor = C.editor || {});
  var U = C.ui;
  var S = C.style;

  E.TYPE_ENTRIES = [
    ["column", "slides-charts-type-column", "Column (clustered)"],
    ["bar", "slides-charts-type-bar", "Bar"],
    ["column-stacked", "slides-charts-type-stacked", "Stacked column"],
    ["line", "slides-charts-type-line", "Line"],
    ["area", "slides-charts-type-area", "Area"],
    ["pie", "slides-charts-type-pie", "Pie"],
    ["donut", "slides-charts-type-donut", "Donut"],
    ["scatter", "slides-charts-type-scatter", "Scatter"],
    ["bubble", "slides-charts-type-bubble", "Bubble"],
    ["combo", "slides-charts-type-combo", "Combo (secondary axis)"],
    ["waterfall", "slides-charts-type-waterfall", "Waterfall"]
  ];
  E.FORMAT_ENTRIES = [
    ["0", "slides-charts-format-integer", "Integer"],
    ["0.0", "slides-charts-format-decimal", "One decimal"],
    ["#,##0", "slides-charts-format-grouped", "Grouped"],
    ["0%", "slides-charts-format-percent", "Percent"],
    ["currency", "slides-charts-format-currency", "Currency"]
  ];

  E.theme = function () {
    var M = window.GBSlidesMasters;
    if (!M || !M.state || !M.state.themes) return null;
    return M.state.themes[M.state.master.defaultThemeId] || null;
  };

  function onChange(node, handler) {
    node.addEventListener("change", handler);
    node.addEventListener("input", handler);
    return node;
  }

  function dataGrid(working, redraw) {
    var section = U.section("slides-charts-data", "Data");
    var grid = document.createElement("div");
    grid.className = "gbch-grid";
    var head = document.createElement("div");
    head.className = "gbch-grid-row gbch-grid-head";
    head.appendChild(U.el("span", "gbch-grid-cell", "slides-charts-category", "Category"));
    working.series.forEach(function (series, index) {
      var cell = document.createElement("span");
      cell.className = "gbch-grid-cell";
      var input = document.createElement("input");
      input.type = "text";
      input.value = series.name;
      U.labelled(input, "data-i18n-aria-label", "slides-charts-series-name", "Series name");
      onChange(input, function () { working.series[index].name = input.value; redraw(false); });
      cell.appendChild(input);
      var remove = U.button("slides-charts-remove", "Remove", function () {
        working.series.splice(index, 1); redraw(true);
      }, "danger");
      cell.appendChild(remove);
      head.appendChild(cell);
    });
    grid.appendChild(head);
    working.categories.forEach(function (category, rowIndex) {
      var row = document.createElement("div");
      row.className = "gbch-grid-row";
      var label = document.createElement("input");
      label.type = "text";
      label.value = category;
      U.labelled(label, "data-i18n-aria-label", "slides-charts-category", "Category");
      onChange(label, function () { working.categories[rowIndex] = label.value; redraw(false); });
      row.appendChild(label);
      working.series.forEach(function (series, seriesIndex) {
        var cell = document.createElement("input");
        cell.type = "number";
        cell.value = C.num(series.values[rowIndex], 0);
        U.labelled(cell, "data-i18n-aria-label", "slides-charts-value", "Value");
        onChange(cell, function () { working.series[seriesIndex].values[rowIndex] = C.num(cell.value, 0); redraw(false); });
        row.appendChild(cell);
      });
      grid.appendChild(row);
    });
    var actions = document.createElement("div");
    actions.className = "gbch-grid-actions";
    actions.appendChild(U.button("slides-charts-add-category", "Add category", function () {
      working.categories.push(C.tr("slides-charts-category", "Category") + " " + (working.categories.length + 1));
      working.series.forEach(function (series) { series.values.push(0); });
      redraw(true);
    }));
    actions.appendChild(U.button("slides-charts-add-series", "Add series", function () {
      working.series.push({
        id: C.uid("s"), name: C.tr("slides-charts-series", "Series") + " " + (working.series.length + 1),
        values: working.categories.map(function () { return 0; }), color: "", axis: "primary"
      });
      redraw(true);
    }));
    section.appendChild(grid);
    section.appendChild(actions);
    return section;
  }

  function stylingPanel(working, redraw) {
    var section = U.section("slides-charts-style", "Styling");
    var titleField = U.field("slides-charts-chart-title", "Chart title", working.title,
      "slides-charts-chart-title-placeholder", "Chart title");
    onChange(titleField.control, function () { working.title = titleField.control.value; redraw(false); });
    section.appendChild(titleField);

    var type = U.select("slides-charts-type", "Chart type", E.TYPE_ENTRIES, working.type);
    onChange(type.control, function () { working.type = type.control.value; redraw(true); });
    section.appendChild(type);

    var format = U.select("slides-charts-number-format", "Number format", E.FORMAT_ENTRIES, working.numberFormat);
    onChange(format.control, function () { working.numberFormat = format.control.value; redraw(false); });
    section.appendChild(format);

    var toggles = document.createElement("div");
    toggles.className = "gbch-toggles";
    [
      ["slides-charts-legend", "Show legend", "legend"],
      ["slides-charts-data-labels", "Show data labels", "dataLabels"],
      ["slides-charts-gridlines", "Show gridlines", "gridlines"]
    ].forEach(function (entry) {
      var box = U.checkbox(entry[0], entry[1], working[entry[2]]);
      onChange(box.control, function () { working[entry[2]] = box.control.checked; redraw(false); });
      toggles.appendChild(box);
    });
    [
      ["slides-charts-stacked", "Stacked", "stacked"],
      ["slides-charts-secondary", "Secondary axis (combo)", "comboSecondary"],
      ["slides-charts-trendline", "Trendline", "trendline"],
      ["slides-charts-error-bars", "Error bars", "errorBars"]
    ].forEach(function (entry) {
      var box = U.checkbox(entry[0], entry[1], working.options[entry[2]]);
      onChange(box.control, function () { working.options[entry[2]] = box.control.checked; redraw(false); });
      toggles.appendChild(box);
    });
    section.appendChild(toggles);

    [["axisTitles", "x", "slides-charts-axis-x", "X axis title"],
      ["axisTitles", "y", "slides-charts-axis-y", "Y axis title"],
      ["axisTitles", "y2", "slides-charts-axis-y2", "Secondary axis title"]].forEach(function (entry) {
      var field = U.field(entry[2], entry[3], working[entry[0]][entry[1]]);
      onChange(field.control, function () { working[entry[0]][entry[1]] = field.control.value; redraw(false); });
      section.appendChild(field);
    });

    E.seriesColors(section, working, redraw);
    return section;
  }

  E.seriesColors = function (section, working, redraw) {
    if (!working.series.length) return;
    var wrap = document.createElement("div");
    wrap.className = "gbch-series-colors";
    working.series.forEach(function (series, index) {
      var label = U.el("label", "gbch-color-field", "slides-charts-series-color",
        "Series { $n } colour");
      label.textContent = C.tr("slides-charts-series-color", "Series { $n } colour", { n: index + 1 });
      var input = document.createElement("input");
      input.type = "color";
      input.value = S.seriesColor(working, index, E.theme());
      onChange(input, function () { working.series[index].color = input.value; redraw(false); });
      label.appendChild(input);
      wrap.appendChild(label);
    });
    section.appendChild(wrap);
  };

  E.compose = function (api, working, existing, holder) {
    api.body.appendChild(U.localNote());
    api.body.appendChild(U.deferredNote());
    api.body.appendChild(dataGrid(working, holder.redraw));
    api.body.appendChild(stylingPanel(working, holder.redraw));
    if (typeof E.extraPanels === "function") E.extraPanels(api, working, existing, holder);
  };

  E.open = function (chartId, slideIndex) {
    var existing = chartId ? C.get(chartId) : null;
    var working = existing ? C.clone(existing) : C.normalize({
      slideIndex: C.num(slideIndex, 0),
      categories: ["Q1", "Q2", "Q3"],
      series: [{ id: "s1", name: "Series 1", values: [12, 19, 15], axis: "primary" }]
    });
    if (!existing) working.id = "";
    var holder = {};
    var dialog = U.openDialog({
      titleKey: "slides-charts-title",
      titleFallback: "Chart editor (local)",
      onBuild: function (api) {
        holder.api = api;
        holder.redraw = function (structure) {
          api.body.textContent = "";
          E.compose(api, working, existing, holder);
          if (structure === false) api.setStatus("slides-charts-status-preview", "Preview updated");
        };
        api.addButton("slides-charts-save", "Save chart", function () {
          var saved = existing ? C.update(existing.id, working) : C.create(working);
          if (!saved) { api.setStatus("slides-charts-status-invalid", "Could not save this chart."); return; }
          api.setStatus(existing ? "slides-charts-status-saved" : "slides-charts-status-created",
            existing ? "Chart saved locally" : "Chart created locally");
          holder.saved = saved;
          if (C.canvas) C.canvas.refresh();
        }, "primary");
        if (existing) {
          api.addButton("slides-charts-duplicate", "Duplicate", function () {
            var copy = C.duplicate(existing.id);
            api.setStatus("slides-charts-status-duplicated", copy ? "Chart duplicated locally" : "Could not duplicate this chart.");
            if (C.canvas) C.canvas.refresh();
          });
          api.addButton("slides-charts-delete", "Delete", function () {
            C.remove(existing.id);
            api.setStatus("slides-charts-status-deleted", "Chart deleted locally");
            if (C.canvas) C.canvas.refresh();
            dialog.close();
          }, "danger");
        }
        api.addButton("slides-charts-close", "Close", function () { dialog.close(); });
        holder.redraw(true);
      }
    });
    holder.dialog = dialog;
    return dialog;
  };
})(typeof window !== "undefined" ? window : globalThis);
