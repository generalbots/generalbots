"use strict";
/* Docs layout preview (#2424), page navigation driven by computed layout.
 *
 * Page count comes from `paginate()`, never from a stored counter. Thumbnails,
 * go-to-page, the current-page indicator and the print-preview overlay all read
 * the same computed page boxes. */
(function (window) {
  var NS = window.DocsLayout;
  if (!NS) return;

  NS.compute = function () {
    var input = NS.projectInput(NS.state);
    NS.result = NS.repaginate(NS.result, input);
    var last = Math.max(0, NS.result.totalPages - 1);
    NS.state.currentPage = Math.min(Math.max(0, Number(NS.state.currentPage) || 0), last);
    return NS.result;
  };

  function labelledInput(node, key, fallback) {
    node.setAttribute("data-i18n-aria-label", key);
    node.setAttribute("aria-label", NS.tr(key, fallback));
    return node;
  }
  NS.labelledInput = labelledInput;

  function button(key, fallback, onClick, cls) {
    var node = NS.i18n(NS.el("button", cls || "gb-layout-btn"), key, fallback);
    node.setAttribute("type", "button");
    if (onClick) node.addEventListener("click", onClick);
    return node;
  }
  NS.layoutButton = button;

  function currentLabel(result, page) {
    return NS.tr("docs-layout-current", "Page { $n } of { $t }", {
      n: page ? page.number : 0, t: result ? result.totalPages : 0
    });
  }
  NS.currentLabel = currentLabel;

  NS.renderIndicator = function (node, result) {
    if (!node) return;
    var page = result && result.pages[NS.state.currentPage];
    node.textContent = currentLabel(result, page);
    node.setAttribute("data-i18n", "docs-layout-current");
    node.setAttribute("data-i18n-params", JSON.stringify({
      n: page ? page.number : 0, t: result ? result.totalPages : 0
    }));
  };

  NS.renderThumbnails = function (host, result, onSelect) {
    if (!host) return;
    host.textContent = "";
    var pages = result && result.pages ? result.pages : [];
    if (!pages.length) {
      host.appendChild(NS.i18n(NS.el("p", "gb-layout-empty"), "docs-layout-no-pages", "No pages to show"));
      return;
    }
    pages.forEach(function (page, index) {
      var thumb = NS.el("button", "gb-layout-thumb");
      thumb.setAttribute("type", "button");
      thumb.setAttribute("data-page-index", String(index));
      if (index === NS.state.currentPage) thumb.setAttribute("aria-current", "page");
      var head = NS.i18n(NS.el("span", "gb-layout-thumb-title"), "docs-layout-page-short", "Page { $n }", { n: page.number });
      var meta = NS.i18n(NS.el("span", "gb-layout-thumb-meta"), "docs-layout-thumb-line-count",
        "{ $n } lines · { $s }", { n: page.lineCount, s: page.sectionName });
      var variant = NS.el("span", "gb-layout-thumb-variant");
      variant.textContent = page.variant;
      thumb.appendChild(head);
      thumb.appendChild(meta);
      thumb.appendChild(variant);
      thumb.addEventListener("click", function () {
        if (onSelect) onSelect(index);
      });
      host.appendChild(thumb);
    });
  };

  function renderSheet(host, page) {
    var sheet = NS.el("div", "gb-layout-sheet");
    sheet.appendChild(NS.i18n(NS.el("div", "gb-layout-sheet-title"), "docs-layout-page-short",
      "Page { $n }", { n: page.number }));
    var header = NS.el("div", "gb-layout-sheet-header");
    header.textContent = (page.headerLines || []).join(" ");
    sheet.appendChild(header);
    var content = NS.el("div", "gb-layout-sheet-body");
    var boxes = page.boxes || [];
    var shown = Math.min(boxes.length, 8);
    for (var i = 0; i < shown; i += 1) {
      var row = NS.el("div", "gb-layout-sheet-line");
      row.textContent = boxes[i].kind + " · " + boxes[i].id;
      content.appendChild(row);
    }
    if (boxes.length > shown) {
      var more = NS.i18n(NS.el("div", "gb-layout-sheet-more"), "docs-layout-more-lines",
        "{ $n } more lines", { n: boxes.length - shown });
      content.appendChild(more);
    }
    sheet.appendChild(content);
    if (page.watermark) {
      var watermark = NS.el("div", "gb-layout-sheet-watermark");
      watermark.textContent = page.watermark;
      sheet.appendChild(watermark);
    }
    var notes = NS.el("div", "gb-layout-sheet-notes");
    (page.footnotes || []).forEach(function (note) {
      var foot = NS.el("div", "gb-layout-sheet-footnote");
      foot.textContent = note.id + " · " + note.text;
      notes.appendChild(foot);
    });
    sheet.appendChild(notes);
    var footer = NS.el("div", "gb-layout-sheet-footer");
    footer.textContent = (page.footerLines || []).join(" ");
    sheet.appendChild(footer);
    host.appendChild(sheet);
  }

  NS.renderPrintPreview = function (host, result) {
    if (!host) return;
    host.textContent = "";
    var pages = result && result.pages ? result.pages : [];
    pages.forEach(function (page) { renderSheet(host, page); });
  };

  NS.openPrintPreview = function () {
    var result = NS.compute();
    var shell = NS.dialog.shell({
      titleKey: "docs-layout-print-title", titleFallback: "Print preview (computed layout)",
      statusId: "docs-layout-print-status"
    });
    NS.renderPrintPreview(shell.body, result);
    shell.status.textContent = currentLabel(result, result.pages[NS.state.currentPage]);
    return shell;
  };

  NS.selectPage = function (index, refresh) {
    if (!NS.result || !NS.result.pages.length) return;
    var target = Math.min(Math.max(0, Number(index) || 0), NS.result.pages.length - 1);
    NS.state.currentPage = target;
    NS.persist();
    if (refresh) refresh();
    NS.statusKey("docs-layout-status-goto", "Page { $n } selected", { n: NS.result.pages[target].number });
  };

  NS.openPages = function () {
    var shell = NS.dialog.shell({
      titleKey: "docs-layout-title", titleFallback: "Pages and layout",
      statusId: "docs-layout-status"
    });
    var indicator = NS.el("div", "gb-layout-indicator");
    indicator.setAttribute("role", "status");
    indicator.setAttribute("aria-live", "polite");
    shell.body.appendChild(indicator);

    function refresh() {
      var result = NS.compute();
      NS.renderIndicator(indicator, result);
      NS.renderThumbnails(thumbs, result, function (index) { NS.selectPage(index, refresh); });
      return result;
    }

    var controls = NS.el("div", "gb-layout-controls");
    var input = labelledInput(NS.el("input", "gb-layout-goto"), "docs-layout-goto-label", "Go to page");
    input.type = "number";
    input.min = "1";
    input.setAttribute("data-i18n-placeholder", "docs-layout-goto-label");
    var go = button("docs-layout-goto", "Go", function () {
      NS.selectPage((Number(input.value) || 1) - 1, refresh);
    });
    var preview = button("docs-layout-print-preview", "Print preview", function () {
      NS.openPrintPreview();
    });
    var recompute = button("docs-layout-refresh", "Recompute layout", function () {
      refresh();
      NS.statusKey("docs-layout-status-recomputed", "Layout recomputed: { $n } pages",
        { n: NS.result.totalPages });
    });
    controls.appendChild(input);
    controls.appendChild(go);
    controls.appendChild(preview);
    controls.appendChild(recompute);

    var thumbs = NS.el("div", "gb-layout-thumbs");
    shell.body.appendChild(controls);
    shell.body.appendChild(thumbs);
    refresh();
    return shell;
  };
})(typeof window !== "undefined" ? window : globalThis);
