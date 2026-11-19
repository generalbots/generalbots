"use strict";
/* Docs layout preview (#2424), page setup + layout inspector.
 *
 * Section geometry, computed page count, per-page line counts and fired keep
 * rules are read from the last layout pass. Section breaks restart numbering.
 * Real print/PDF fidelity, PDF/UA, benchmarks and CI image comparison are
 * explicitly deferred. */
(function (window) {
  var NS = window.DocsLayout;
  if (!NS) return;

  function line(labelKey, fallback, value) {
    var row = NS.el("div", "gb-layout-row");
    row.appendChild(NS.i18n(NS.el("span", "gb-layout-label"), labelKey, fallback));
    var val = NS.el("span", "gb-layout-value");
    val.textContent = String(value == null ? "" : value);
    row.appendChild(val);
    return row;
  }
  NS.layoutRow = line;

  function sectionNode(titleKey, titleFallback) {
    var wrap = NS.el("section", "gb-layout-section");
    wrap.appendChild(NS.i18n(NS.el("h4", "gb-layout-section-title"), titleKey, titleFallback));
    return wrap;
  }

  function selectField(parent, labelKey, labelFallback, options, value, onChange) {
    var row = NS.el("div", "gb-layout-row");
    row.appendChild(NS.i18n(NS.el("span", "gb-layout-label"), labelKey, labelFallback));
    var select = NS.el("select", "gb-layout-input");
    options.forEach(function (option) {
      var node = NS.i18n(NS.el("option"), option.key, option.fallback);
      node.value = option.value;
      select.appendChild(node);
    });
    select.value = value;
    select.addEventListener("change", function () { onChange(select.value); });
    row.appendChild(select);
    parent.appendChild(row);
  }

  function numberField(parent, labelKey, labelFallback, value, onChange) {
    var row = NS.el("div", "gb-layout-row");
    row.appendChild(NS.i18n(NS.el("span", "gb-layout-label"), labelKey, labelFallback));
    var input = NS.el("input", "gb-layout-input");
    input.type = "number";
    input.value = String(value);
    input.setAttribute("data-i18n-aria-label", labelKey);
    input.setAttribute("aria-label", NS.tr(labelKey, labelFallback));
    input.addEventListener("change", function () { onChange(Number(input.value)); });
    row.appendChild(input);
    parent.appendChild(row);
  }

  function checkboxField(parent, labelKey, labelFallback, checked, onChange) {
    var row = NS.el("div", "gb-layout-row");
    var input = NS.el("input", "gb-layout-checkbox");
    input.type = "checkbox";
    input.checked = !!checked;
    labelledCheckbox(input, labelKey, labelFallback);
    input.addEventListener("change", function () { onChange(input.checked); });
    row.appendChild(input);
    row.appendChild(NS.i18n(NS.el("span", "gb-layout-label"), labelKey, labelFallback));
    parent.appendChild(row);
  }

  function labelledCheckbox(node, key, fallback) {
    node.setAttribute("data-i18n-aria-label", key);
    node.setAttribute("aria-label", NS.tr(key, fallback));
  }

  function updateSetup(key, value, refresh) {
    NS.state.setup[key] = value;
    NS.persist();
    refresh();
    NS.statusKey("docs-layout-status-setup", "Page setup updated");
  }

  function renderSetup(host, refresh) {
    var wrap = sectionNode("docs-layout-setup", "Page setup");
    var s = NS.state.setup;
    selectField(wrap, "docs-layout-paper", "Paper size", [
      { value: "a4", key: "docs-layout-paper-a4", fallback: "A4" },
      { value: "letter", key: "docs-layout-paper-letter", fallback: "Letter" }
    ], s.paper, function (value) { updateSetup("paper", value, refresh); });
    selectField(wrap, "docs-layout-orientation", "Orientation", [
      { value: "portrait", key: "docs-layout-orientation-portrait", fallback: "Portrait" },
      { value: "landscape", key: "docs-layout-orientation-landscape", fallback: "Landscape" }
    ], s.orientation, function (value) { updateSetup("orientation", value, refresh); });
    selectField(wrap, "docs-layout-columns", "Columns", [
      { value: "1", key: "docs-layout-columns-one", fallback: "1" },
      { value: "2", key: "docs-layout-columns-two", fallback: "2" },
      { value: "3", key: "docs-layout-columns-three", fallback: "3" }
    ], String(s.columns), function (value) { updateSetup("columns", Number(value), refresh); });
    numberField(wrap, "docs-layout-margin-top", "Top margin", s.marginTop, function (v) { updateSetup("marginTop", v, refresh); });
    numberField(wrap, "docs-layout-margin-bottom", "Bottom margin", s.marginBottom, function (v) { updateSetup("marginBottom", v, refresh); });
    numberField(wrap, "docs-layout-margin-left", "Left margin", s.marginLeft, function (v) { updateSetup("marginLeft", v, refresh); });
    numberField(wrap, "docs-layout-margin-right", "Right margin", s.marginRight, function (v) { updateSetup("marginRight", v, refresh); });
    numberField(wrap, "docs-layout-gutter", "Gutter", s.gutter, function (v) { updateSetup("gutter", v, refresh); });
    checkboxField(wrap, "docs-layout-mirror", "Mirror margins", s.mirrorMargins, function (v) { updateSetup("mirrorMargins", v, refresh); });
    host.appendChild(wrap);
  }

  function renderSections(host, refresh) {
    var wrap = sectionNode("docs-layout-sections", "Sections");
    NS.state.sections.forEach(function (section, index) {
      var row = NS.el("div", "gb-layout-section-row");
      row.appendChild(NS.i18n(NS.el("span", "gb-layout-section-name"),
        "docs-layout-section-label", "Section { $n }", { n: index + 1 }));
      checkboxField(row, "docs-layout-section-restart", "Restart numbering",
        section.restartNumbering, function (value) {
          section.restartNumbering = value;
          NS.persist(); refresh();
          NS.statusKey("docs-layout-status-section", "Section updated");
        });
      numberField(row, "docs-layout-section-start", "Start at", section.startNumber, function (value) {
        section.startNumber = Math.max(1, value);
        NS.persist(); refresh();
        NS.statusKey("docs-layout-status-section", "Section updated");
      });
      wrap.appendChild(row);
    });
    var add = NS.layoutButton("docs-layout-add-section", "Add section break", function () {
      NS.addSectionBreakAtCurrentPage(refresh);
    });
    wrap.appendChild(add);
    host.appendChild(wrap);
  }

  function renderStats(host, result) {
    var wrap = sectionNode("docs-layout-inspector", "Layout inspector");
    wrap.appendChild(line("docs-layout-page-count", "Computed pages", result.totalPages));
    result.pages.forEach(function (page) {
      var node = NS.i18n(NS.el("div", "gb-layout-page-stat"),
        "docs-layout-page-lines", "Page { $n }: { $c } lines · { $v }",
        { n: page.number, c: page.lineCount, v: page.variant });
      wrap.appendChild(node);
      var geometry = page.geometry;
      wrap.appendChild(line("docs-layout-column-width", "Column width",
        Math.round(geometry.columnWidth) + " px"));
      wrap.appendChild(line("docs-layout-content-height", "Text height",
        Math.round(geometry.bodyHeight) + " px"));
    });
    wrap.appendChild(NS.i18n(NS.el("h4", "gb-layout-section-title"),
      "docs-layout-keep-rules", "Keep rules fired"));
    if (!result.keepFirings.length) {
      wrap.appendChild(NS.i18n(NS.el("p", "gb-layout-empty"),
        "docs-layout-keep-none", "No keep rules fired"));
    } else {
      result.keepFirings.forEach(function (firing) {
        var node = NS.el("div", "gb-layout-firing");
        node.textContent = firing.rule + " · " + firing.id + " · " + (firing.page + 1);
        wrap.appendChild(node);
      });
    }
    wrap.appendChild(NS.i18n(NS.el("p", "gb-layout-deferred"), "docs-layout-deferred",
      "Deferred: real PDF renderer, tagged PDF/PDF-UA, pixel parity, 100-page benchmark and CI page-image comparison."));
    host.appendChild(wrap);
  }

  NS.addSectionBreakAtCurrentPage = function (refresh) {
    var result = NS.result || NS.compute();
    var page = result.pages[NS.state.currentPage];
    var boxes = page && page.boxes ? page.boxes : [];
    var startLine = boxes.length ? boxes[0].lineIndex : 0;
    var exists = NS.state.sections.some(function (section) { return section.startLine === startLine; });
    if (!exists && startLine > 0) {
      NS.state.sections.push({
        id: "section-" + (NS.state.sections.length + 1),
        startLine: startLine, restartNumbering: true, startNumber: 1,
        firstPageDifferent: false, geometry: {},
        header: NS.defaultSections()[0].header,
        footer: NS.defaultSections()[0].footer,
        watermark: NS.defaultSections()[0].watermark
      });
      NS.state.sections.sort(function (a, b) { return a.startLine - b.startLine; });
      NS.persist();
      NS.statusKey("docs-layout-status-added", "Section break added at line { $n }", { n: startLine });
    } else {
      NS.statusKey("docs-layout-status-noadd", "Section break already present here");
    }
    if (refresh) refresh();
  };

  NS.openInspector = function () {
    var shell = NS.dialog.shell({
      titleKey: "docs-layout-inspector-title", titleFallback: "Layout inspector and page setup",
      statusId: "docs-layout-inspect-status"
    });
    function refresh() {
      var result = NS.compute();
      shell.body.textContent = "";
      renderStats(shell.body, result);
      renderSetup(shell.body, refresh);
      renderSections(shell.body, refresh);
    }
    refresh();
    return shell;
  };
})(typeof window !== "undefined" ? window : globalThis);
