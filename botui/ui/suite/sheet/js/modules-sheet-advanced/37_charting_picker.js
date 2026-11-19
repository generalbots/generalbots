"use strict";
/* Advanced charting — accessible chart-type picker with live preview (#2490). */

(function () {
  const t = window.SheetCharting ? window.SheetCharting.t : function (key, fallback) { return fallback; };
  const TYPES = [
    "waterfall", "histogram", "pareto", "box", "heatmap", "treemap",
    "sunburst", "combo", "stacked-bar", "stacked-column", "clustered-bar", "clustered-column",
  ];
  const HIERARCHY = { heatmap: true, treemap: true, sunburst: true };

  function label(key, fallback) { return t(key, fallback); }

  function field(labelKey, fallback, control) {
    const row = document.createElement("label");
    row.className = "ss-charting-field";
    const caption = document.createElement("span");
    caption.textContent = label(labelKey, fallback);
    caption.setAttribute("data-i18n", labelKey);
    control.style.width = "100%";
    row.appendChild(caption);
    row.appendChild(control);
    return row;
  }

  function input(type, id, value) {
    const element = document.createElement("input");
    element.type = type;
    element.id = id;
    if (value != null) element.value = value;
    return element;
  }

  function select(id, options, selected) {
    const element = document.createElement("select");
    element.id = id;
    options.forEach(function (entry) {
      const option = document.createElement("option");
      option.value = entry.value;
      option.textContent = label(entry.key, entry.fallback);
      if (entry.value === selected) option.selected = true;
      element.appendChild(option);
    });
    return element;
  }

  function button(textKey, fallback, className) {
    const element = document.createElement("button");
    element.type = "button";
    element.className = className || "";
    element.textContent = label(textKey, fallback);
    element.setAttribute("data-i18n", textKey);
    return element;
  }

  function closeDialog(dialog, restore, onKey, onDown) {
    if (!dialog.isConnected) return;
    document.removeEventListener("keydown", onKey, true);
    document.removeEventListener("mousedown", onDown, true);
    dialog.remove();
    if (restore && restore.isConnected && typeof restore.focus === "function") restore.focus();
  }

  function open(existing, anchor) {
    const previous = window.SheetChartingPicker && window.SheetChartingPicker.dialog;
    if (previous && previous.isConnected) previous.remove();
    const restore = document.activeElement;
    const source = existing || {};
    const dialog = document.createElement("div");
    dialog.className = "ss-charting-dialog";
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");
    dialog.setAttribute("aria-labelledby", "ss-charting-dialog-title");
    dialog.tabIndex = -1;

    const heading = document.createElement("h2");
    heading.id = "ss-charting-dialog-title";
    heading.textContent = label("sheet-charting-title", "Advanced charts");
    heading.setAttribute("data-i18n", "sheet-charting-title");
    const close = button("sheet-charting-close", "Close", "ss-charting-close");
    close.setAttribute("aria-label", label("sheet-charting-close", "Close"));
    const header = document.createElement("div");
    header.className = "ss-charting-header";
    header.appendChild(heading);
    header.appendChild(close);
    dialog.appendChild(header);

    const body = document.createElement("div");
    body.className = "ss-charting-body";
    const chooser = document.createElement("div");
    chooser.className = "ss-charting-types";
    chooser.setAttribute("role", "radiogroup");
    chooser.setAttribute("aria-label", label("sheet-charting-type", "Chart type"));
    let selectedType = source.chart_type && TYPES.indexOf(source.chart_type) >= 0 ? source.chart_type : TYPES[0];
    const typeButtons = [];
    TYPES.forEach(function (type) {
      const option = button("sheet-charting-type-" + type, type, "ss-charting-type-btn");
      option.setAttribute("role", "radio");
      option.setAttribute("aria-checked", type === selectedType ? "true" : "false");
      option.dataset.type = type;
      chooser.appendChild(option);
      typeButtons.push(option);
    });
    body.appendChild(chooser);

    const form = document.createElement("div");
    form.className = "ss-charting-form";
    const titleInput = input("text", "ss-charting-title-input", source.title || "");
    const rangeInput = input("text", "ss-charting-range-input", source.data_range || "");
    if (!rangeInput.value && window.SheetCharts && window.SheetCharts.currentRange) {
      rangeInput.value = window.SheetCharts.currentRange() || "";
    }
    const options = source.options || {};
    const minInput = input("number", "ss-charting-min-input", options.min != null ? options.min : "");
    const maxInput = input("number", "ss-charting-max-input", options.max != null ? options.max : "");
    const paletteSelect = select("ss-charting-palette-select", [
      { value: "cvd", key: "sheet-charting-palette-cvd", fallback: "Colour-blind safe" },
      { value: "contrast", key: "sheet-charting-palette-contrast", fallback: "High contrast" },
    ], options.palette || "cvd");
    const binsInput = input("number", "ss-charting-bins-input", options.bins != null ? options.bins : 6);
    binsInput.min = "2";
    binsInput.max = "30";
    const depthSelect = select("ss-charting-depth-select", [
      { value: "2", key: "sheet-charting-max-depth", fallback: "Maximum depth: 2" },
      { value: "1", key: "sheet-charting-total", fallback: "Total" },
    ], String(options.maxDepth || 2));
    form.appendChild(field("sheet-charting-chart-title", "Chart title", titleInput));
    form.appendChild(field("sheet-charting-data-range", "Data range", rangeInput));
    const bounds = document.createElement("div");
    bounds.className = "ss-charting-bounds";
    bounds.appendChild(field("sheet-charting-min", "Minimum bound", minInput));
    bounds.appendChild(field("sheet-charting-max", "Maximum bound", maxInput));
    form.appendChild(bounds);
    form.appendChild(field("sheet-charting-palette", "Palette", paletteSelect));
    const binsField = field("sheet-charting-bins", "Bins", binsInput);
    const depthField = field("sheet-charting-depth", "Depth", depthSelect);
    form.appendChild(binsField);
    form.appendChild(depthField);

    const analysis = source.analysis || {};
    const analysisFields = document.createElement("fieldset");
    analysisFields.className = "ss-charting-analysis";
    const legend = document.createElement("legend");
    legend.textContent = label("sheet-charting-analysis", "Analysis overlays");
    legend.setAttribute("data-i18n", "sheet-charting-analysis");
    analysisFields.appendChild(legend);
    const trendSelect = select("ss-charting-trend-select", [
      { value: "none", key: "sheet-charting-trendline-none", fallback: "None" },
      { value: "linear", key: "sheet-charting-trendline-linear", fallback: "Linear" },
      { value: "exponential", key: "sheet-charting-trendline-exponential", fallback: "Exponential" },
      { value: "moving", key: "sheet-charting-trendline-moving", fallback: "Moving average" },
    ], analysis.trendline || "none");
    analysisFields.appendChild(field("sheet-charting-trendline", "Trendline", trendSelect));
    analysisFields.appendChild(field("sheet-charting-moving-period", "Period", input("number", "ss-charting-period-input", analysis.movingPeriod || 3)));
    const errorBox = input("checkbox", "ss-charting-error-box");
    errorBox.checked = Boolean(analysis.errorBars);
    analysisFields.appendChild(field("sheet-charting-error-bars", "Error bars", errorBox));
    analysisFields.appendChild(field("sheet-charting-error-percent", "Error (%)", input("number", "ss-charting-error-input", analysis.errorPercent || 5)));
    const referenceValue = input("number", "ss-charting-reference-input", "");
    const referenceLabel = input("text", "ss-charting-reference-label-input", "");
    analysisFields.appendChild(field("sheet-charting-reference-value", "Reference line", referenceValue));
    analysisFields.appendChild(field("sheet-charting-reference-label", "Reference label", referenceLabel));
    analysisFields.appendChild(field("sheet-charting-target-low", "Target low", input("number", "ss-charting-target-low-input", analysis.targetBand ? analysis.targetBand.low : "")));
    analysisFields.appendChild(field("sheet-charting-target-high", "Target high", input("number", "ss-charting-target-high-input", analysis.targetBand ? analysis.targetBand.high : "")));
    const varianceBox = input("checkbox", "ss-charting-variance-box");
    varianceBox.checked = Boolean(analysis.variance);
    analysisFields.appendChild(field("sheet-charting-variance-callouts", "Variance callouts", varianceBox));
    form.appendChild(analysisFields);

    const preview = document.createElement("div");
    preview.className = "ss-charting-preview";
    preview.setAttribute("role", "img");
    const previewCaption = document.createElement("span");
    previewCaption.className = "ss-charting-preview-caption";
    previewCaption.textContent = label("sheet-charting-preview", "Live preview");
    previewCaption.setAttribute("data-i18n", "sheet-charting-preview");
    const previewFrame = document.createElement("div");
    previewFrame.appendChild(previewCaption);
    previewFrame.appendChild(preview);
    body.appendChild(form);
    body.appendChild(previewFrame);
    dialog.appendChild(body);

    const status = document.createElement("div");
    status.className = "ss-charting-status";
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    status.textContent = label("sheet-charting-status-ready", "Choose a chart type and a data range.");
    status.setAttribute("data-i18n", "sheet-charting-status-ready");
    const cancel = button("sheet-charting-cancel", "Cancel", "ss-charting-cancel");
    const apply = button("sheet-charting-apply", "Apply", "ss-charting-apply");
    const footer = document.createElement("div");
    footer.className = "ss-charting-footer";
    footer.appendChild(status);
    footer.appendChild(cancel);
    footer.appendChild(apply);
    dialog.appendChild(footer);
    document.body.appendChild(dialog);

    function definition() {
      const bound = window.SheetChartingData.bindRange(rangeInput.value, {
        min: minInput.value, max: maxInput.value,
      });
      return {
        id: source.id,
        chart_type: selectedType,
        title: titleInput.value,
        data_range: rangeInput.value,
        labels: bound.labels,
        datasets: bound.series.map(function (series) {
          return { label: series.label, data: series.data };
        }),
        options: {
          palette: paletteSelect.value,
          min: minInput.value,
          max: maxInput.value,
          bins: binsInput.value,
          maxDepth: depthSelect.value,
        },
        analysis: {
          trendline: trendSelect.value,
          movingPeriod: Number(document.getElementById("ss-charting-period-input").value) || 3,
          errorBars: errorBox.checked,
          errorPercent: Number(document.getElementById("ss-charting-error-input").value) || 0,
          referenceLines: [{ value: referenceValue.value, label: referenceLabel.value }],
          targetBand: {
            low: document.getElementById("ss-charting-target-low-input").value,
            high: document.getElementById("ss-charting-target-high-input").value,
          },
          variance: varianceBox.checked,
        },
        position: source.position || { row: 1, col: 1, width: 320, height: 200 },
        localOnly: true,
      };
    }

    function refresh() {
      const def = definition();
      const hidden = Boolean(HIERARCHY[selectedType]);
      binsField.hidden = selectedType !== "histogram";
      depthField.hidden = selectedType !== "treemap" && selectedType !== "sunburst";
      analysisFields.hidden = hidden;
      if (!def.datasets.length || !def.datasets.some(function (series) {
        return series.data.some(function (value) { return value != null; });
      })) {
        preview.textContent = label("sheet-charting-status-empty", "No numeric data in the selected range.");
        status.textContent = label("sheet-charting-status-empty", "No numeric data in the selected range.");
        return;
      }
      preview.innerHTML = window.SheetCharting.renderSvg(def, 300, 180);
      status.textContent = label("sheet-charting-status-preview", "Preview updated for {type}.", {
        type: label("sheet-charting-type-" + selectedType, selectedType),
      });
    }

    typeButtons.forEach(function (option, index) {
      option.addEventListener("click", function () {
        selectedType = option.dataset.type;
        typeButtons.forEach(function (item) { item.setAttribute("aria-checked", item === option ? "true" : "false"); });
        refresh();
      });
      option.addEventListener("keydown", function (event) {
        if (event.key !== "ArrowRight" && event.key !== "ArrowDown" && event.key !== "ArrowLeft" && event.key !== "ArrowUp") return;
        event.preventDefault();
        const step = event.key === "ArrowRight" || event.key === "ArrowDown" ? 1 : -1;
        const next = typeButtons[(index + step + typeButtons.length) % typeButtons.length];
        next.focus();
        next.click();
      });
    });
    body.addEventListener("input", refresh);
    body.addEventListener("change", refresh);

    function focusables() {
      return Array.from(dialog.querySelectorAll("button:not([hidden]), input:not([hidden]), select:not([hidden])"))
        .filter(function (element) { return element.offsetParent !== null || element === apply; });
    }

    function onKey(event) {
      if (event.key === "Escape") {
        event.preventDefault();
        closeDialog(dialog, restore, onKey, onDown);
        return;
      }
      if (event.key !== "Tab") return;
      const elements = focusables();
      if (!elements.length) return;
      const first = elements[0];
      const last = elements[elements.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    }

    function onDown(event) {
      if (!dialog.contains(event.target)) closeDialog(dialog, restore, onKey, onDown);
    }

    close.addEventListener("click", function () { closeDialog(dialog, restore, onKey, onDown); });
    cancel.addEventListener("click", function () { closeDialog(dialog, restore, onKey, onDown); });
    apply.addEventListener("click", function () {
      if (!window.SheetChartingData.parseRange(rangeInput.value)) {
        status.textContent = label("sheet-charting-status-invalid-range", "Enter a range such as A1:D10.");
        rangeInput.focus();
        return;
      }
      const def = definition();
      if (source.id) {
        const list = window.SheetCharting.localCharts();
        const index = list.findIndex(function (item) { return item.id === source.id; });
        if (index >= 0) list[index] = def;
        window.SheetCharting.renderAll();
      } else {
        window.SheetCharting.addLocal(def);
      }
      const message = label("sheet-charting-status-applied", "Local chart added: {type}.", {
        type: label("sheet-charting-type-" + selectedType, selectedType),
      });
      status.textContent = message;
      if (window.sheetNotify) window.sheetNotify(message, false);
      closeDialog(dialog, restore, onKey, onDown);
    });

    document.addEventListener("keydown", onKey, true);
    document.addEventListener("mousedown", onDown, true);
    window.SheetChartingPicker.dialog = dialog;
    refresh();
    titleInput.focus();
  }

  window.SheetChartingPicker = { open: open, dialog: null };
})();
