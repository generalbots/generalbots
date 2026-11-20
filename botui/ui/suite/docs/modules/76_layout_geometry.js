"use strict";
/* Docs layout preview (#2424), fixed page geometry + line metrics.
 *
 * All measurements are deterministic integers at 96 dpi. No DOM measurement,
 * fonts or renderer are involved: this is a preview model, not the PDF
 * renderer. */
(function (window) {
  var NS = window.DocsLayout;
  if (!NS) return;

  NS.PAPER = { a4: { width: 794, height: 1123 }, letter: { width: 816, height: 1056 } };

  NS.defaultSetup = function () {
    return {
      paper: "a4", orientation: "portrait",
      marginTop: 96, marginBottom: 96, marginLeft: 96, marginRight: 96,
      gutter: 0, mirrorMargins: false, columns: 1, columnGap: 32,
      lineHeight: 27, headerHeight: 32, footerHeight: 40,
      widows: 2, orphans: 2, tableSplit: false
    };
  };

  NS.defaultSections = function () {
    return [{
      id: "section-1", name: NS.tr("docs-layout-section-label", "Section { $n }", { n: 1 }),
      startLine: 0, restartNumbering: false, startNumber: 1, firstPageDifferent: false,
      geometry: {},
      header: {
        first: NS.tr("docs-layout-hdr-first", "{ $title } — { $date }"),
        even: NS.tr("docs-layout-hdr-even", "Internal — { $title }"),
        odd: NS.tr("docs-layout-hdr-odd", "{ $title }")
      },
      footer: {
        first: NS.tr("docs-layout-ftr-first", "First edition"),
        even: NS.tr("docs-layout-ftr-even", "Page { $page } of { $total }"),
        odd: NS.tr("docs-layout-ftr-odd", "Page { $page } of { $total }")
      },
      watermark: NS.tr("docs-layout-watermark-default", "DRAFT")
    }];
  };

  function num(value, fallback) {
    var n = Number(value);
    return isFinite(n) ? n : fallback;
  }

  /* Resolve the geometry for one page. `pageNumber` drives mirrored margins. */
  NS.resolveGeometry = function (setup, overrides, pageNumber) {
    var base = setup || NS.defaultSetup();
    var g = {};
    Object.keys(base).forEach(function (key) { g[key] = base[key]; });
    if (overrides && typeof overrides === "object") {
      Object.keys(overrides).forEach(function (key) { g[key] = overrides[key]; });
    }
    var paper = NS.PAPER[g.paper] || NS.PAPER.a4;
    var width = paper.width, height = paper.height;
    if (g.orientation === "landscape") { width = paper.height; height = paper.width; }

    var gutter = Math.max(0, num(g.gutter, 0));
    var left = Math.max(0, num(g.marginLeft, 96));
    var right = Math.max(0, num(g.marginRight, 96));
    if (g.mirrorMargins && (num(pageNumber, 1) % 2 === 0)) {
      var swap = left; left = right; right = swap;
      right += gutter;
    } else {
      left += gutter;
    }
    var columns = Math.min(3, Math.max(1, Math.round(num(g.columns, 1))));
    var gap = Math.max(0, num(g.columnGap, 0));
    var contentWidth = Math.max(0, width - left - right);
    var columnWidth = Math.max(0, (contentWidth - (columns - 1) * gap) / columns);
    var headerHeight = Math.max(0, num(g.headerHeight, 0));
    var footerHeight = Math.max(0, num(g.footerHeight, 0));
    var bodyTop = Math.max(0, num(g.marginTop, 96)) + headerHeight;
    var bodyBottom = Math.max(bodyTop, height - Math.max(0, num(g.marginBottom, 96)) - footerHeight);
    var cols = [];
    for (var i = 0; i < columns; i += 1) {
      cols.push({
        index: i, top: bodyTop, bottom: bodyBottom,
        left: left + i * (columnWidth + gap), width: columnWidth,
        height: bodyBottom - bodyTop
      });
    }
    return {
      paper: NS.PAPER[g.paper] ? g.paper : "a4",
      orientation: g.orientation === "landscape" ? "landscape" : "portrait",
      width: width, height: height,
      marginTop: num(g.marginTop, 96), marginBottom: num(g.marginBottom, 96),
      marginLeft: left, marginRight: right,
      gutter: gutter, mirrorMargins: !!g.mirrorMargins,
      columns: columns, columnGap: gap, columnWidth: columnWidth,
      contentWidth: contentWidth, headerHeight: headerHeight, footerHeight: footerHeight,
      bodyTop: bodyTop, bodyBottom: bodyBottom, bodyHeight: bodyBottom - bodyTop,
      cols: cols
    };
  };

  var FACTOR = {
    "heading1": 1.6, "heading2": 1.3, "heading3": 1.15,
    "code": 1, "paragraph": 1, "list": 1, "footnote": 0.8,
    "image": 1, "table-row": 1, "page-break": 0
  };
  NS.lineMetrics = function (line, geometry) {
    var item = line || {};
    var kind = item.kind || "paragraph";
    var base = Math.max(1, num(geometry && geometry.lineHeight, 27));
    var factor = FACTOR[kind] == null ? 1 : FACTOR[kind];
    var anchor = item.imageAnchor === "float" ? "float" : "inline";
    var height = typeof item.height === "number" && item.height >= 0
      ? item.height : base * factor;
    if (anchor === "float") height = 0; // float reserves side space, not vertical flow
    return {
      id: item.id, kind: kind, height: height, anchor: anchor,
      width: geometry ? geometry.columnWidth : 0
    };
  };

  NS.buildSections = function (sections, lines) {
    var src = Array.isArray(sections) && sections.length ? sections : NS.defaultSections();
    var fallback = NS.defaultSections()[0];
    var built = src.map(function (raw, index) {
      var s = raw && typeof raw === "object" ? raw : {};
      return {
        id: s.id || ("section-" + (index + 1)),
        name: s.name || NS.tr("docs-layout-section-label", "Section { $n }", { n: index + 1 }),
        startLine: Math.max(0, Math.round(num(s.startLine, 0))),
        restartNumbering: !!s.restartNumbering,
        startNumber: Math.max(1, Math.round(num(s.startNumber, 1))),
        firstPageDifferent: !!s.firstPageDifferent,
        geometry: s.geometry && typeof s.geometry === "object" ? s.geometry : {},
        header: s.header && typeof s.header === "object" ? s.header : fallback.header,
        footer: s.footer && typeof s.footer === "object" ? s.footer : fallback.footer,
        watermark: s.watermark == null ? fallback.watermark : s.watermark
      };
    });
    built.sort(function (a, b) { return a.startLine - b.startLine; });
    if (!built.length || built[0].startLine !== 0) {
      built.unshift(NS.buildSections([{ startLine: 0 }], lines || [])[0]);
    }
    return built;
  };

  /* Project typed model blocks (when present) into the local line-list. */
  NS.linesFromBlocks = function (blocks) {
    return (blocks || []).map(function (block) {
      var text = "";
      (block.spans || []).forEach(function (span) { text += span.text || ""; });
      var line = { id: block.id, text: text, kind: "paragraph", paraId: block.id };
      if (block.type === "heading") { line.kind = "heading1"; line.keepWithNext = true; }
      else if (block.type === "list") line.kind = "list";
      else if (block.type === "table") { line.kind = "table-row"; line.tableRow = true; }
      else if (block.type === "image") {
        line.kind = "image";
        line.imageAnchor = (block.attrs && block.attrs.anchor) || "inline";
      } else if (block.type === "footnote-reference") {
        line.kind = "footnote";
        line.footnote = { id: block.id, text: text };
      } else if (block.type === "page-break") { line.kind = "page-break"; line.forceBreak = true; }
      else if (block.type === "section-break") { line.kind = "section-break"; line.forceSection = true; }
      if (block.attrs) {
        if (block.attrs.keepWithNext) line.keepWithNext = true;
        if (block.attrs.keepTogether) { line.keepTogether = true; line.keepGroup = block.attrs.keepGroup || block.id; }
        if (block.attrs.height != null) line.height = block.attrs.height;
      }
      return line;
    });
  };

  NS.sampleLines = function () {
    var out = [];
    for (var i = 1; i <= 6; i += 1) {
      out.push({
        id: "p" + i, kind: "paragraph", paraId: "p" + i,
        text: NS.tr("docs-layout-sample-paragraph", "Sample paragraph { $n }", { n: i })
      });
    }
    out.splice(2, 0, {
      id: "h1", kind: "heading1", text: NS.tr("docs-layout-sample-heading", "Sample heading"),
      keepWithNext: true
    });
    var note = NS.tr("docs-layout-sample-footnote", "Sample footnote text");
    out.push({ id: "fn1", kind: "footnote", text: note, footnote: { id: "fn1", text: note } });
    return out;
  };
})(typeof window !== "undefined" ? window : globalThis);
