"use strict";
/* Docs layout preview (#2424), deterministic pagination pass.
 *
 * `paginate(input)` is pure: line boxes in, page boxes out. `repaginate(prev,
 * input)` reuses the unchanged prefix of page boxes so an edit on one page does
 * not rebuild earlier pages. This is a PREVIEW model, not the PDF renderer. */
(function (window) {
  var NS = window.DocsLayout;
  if (!NS) return;

  NS.pageSignature = function (page) {
    return JSON.stringify({
      n: page.number, s: page.sectionId, v: page.variant,
      b: (page.boxes || []).map(function (box) {
        return [box.id, box.y, box.height, box.column, box.anchor];
      }),
      /* Include line content so a text edit invalidates that page box: an
       * unchanged earlier page still keeps object identity on repaginate. */
      t: (page.lines || []).map(function (line) { return [line.id, line.text]; }),
      f: (page.footnotes || []).map(function (note) { return note.id; }),
      h: page.headerLines, ft: page.footerLines, w: page.watermark
    });
  };

  NS.projectInput = function (state) {
    var src = state || NS.state || {};
    return {
      lines: src.lines || [], setup: src.setup || NS.defaultSetup(),
      sections: src.sections || NS.defaultSections(), fields: src.fields || {},
      keep: {
        widows: (src.setup && src.setup.widows) != null ? src.setup.widows : 2,
        orphans: (src.setup && src.setup.orphans) != null ? src.setup.orphans : 2,
        tableSplit: !!(src.setup && src.setup.tableSplit)
      }
    };
  };

  NS.paginate = function (input) {
    var src = input || {};
    var lines = src.lines || [];
    var setup = src.setup || NS.defaultSetup();
    var sections = NS.buildSections(src.sections, lines);
    var fields = src.fields || {};
    var settings = src.keep || {};
    var firstGeometry = NS.resolveGeometry(setup, sections[0].geometry, 1);

    var entries = lines.map(function (line, index) {
      var metrics = NS.lineMetrics(line, firstGeometry);
      return {
        id: line.id, lineIndex: index, text: line.text || "",
        paraId: line.paraId, keepWithNext: !!line.keepWithNext,
        keepGroup: line.keepTogether && line.keepGroup ? line.keepGroup : null,
        tableRow: !!line.tableRow, forceBreak: !!line.forceBreak,
        metrics: metrics, footnote: NS.footnoteOf(line)
      };
    });

    var groupHeights = {};
    entries.forEach(function (entry) {
      if (!entry.keepGroup) return;
      groupHeights[entry.keepGroup] = (groupHeights[entry.keepGroup] || 0) + entry.metrics.height;
    });

    var boundaries = {};
    sections.forEach(function (section, index) {
      if (section.startLine > 0) boundaries[section.startLine] = index;
    });

    var pages = [];
    var pageNumber = 0;
    var sectionIndex = 0;
    var section = sections[0];
    var atSectionStart = true;
    var page = null;

    function newPage(sectionToUse, indexOfSection) {
      if (sectionToUse.restartNumbering && atSectionStart) {
        pageNumber = Math.max(1, sectionToUse.startNumber) - 1;
      }
      pageNumber += 1;
      var geometry = NS.resolveGeometry(setup, sectionToUse.geometry, pageNumber);
      page = {
        index: pages.length, number: pageNumber, sectionId: sectionToUse.id,
        sectionIndex: indexOfSection, sectionName: sectionToUse.name,
        isSectionFirst: atSectionStart, geometry: geometry,
        lines: [], footnotes: [], keepRulesFired: [], boxes: [], lineCount: 0
      };
      pages.push(page);
      atSectionStart = false;
      return page;
    }

    newPage(section, sectionIndex);
    entries.forEach(function (entry, index) {
      var boundary = boundaries[index];
      if (boundary != null && boundary !== sectionIndex) {
        sectionIndex = boundary;
        section = sections[sectionIndex];
        atSectionStart = true;
        newPage(section, sectionIndex);
      }
      if (entry.forceBreak && page.lines.length) {
        newPage(section, sectionIndex);
        page.keepRulesFired.push({ rule: "manual-page-break", id: entry.id, page: page.index });
      }
      var decision = NS.breakDecision(entry, entries[index + 1], page, groupHeights, settings);
      if (decision.break) {
        newPage(section, sectionIndex);
        if (decision.rule) {
          page.keepRulesFired.push({ rule: decision.rule, id: entry.id, page: page.index });
        }
      }
      page.lines.push(entry);
      if (entry.footnote) {
        page.footnotes.push({
          id: entry.footnote.id, text: entry.footnote.text,
          refLineId: entry.id, page: page.index
        });
      }
    });

    NS.applyWidowOrphan(pages, settings);
    pages.forEach(function (item) { NS.normalizeBoxes(item); });
    pages.forEach(function (item) {
      if (NS.resolvePageChrome) NS.resolvePageChrome(item, sections[item.sectionIndex], fields, pages.length);
      item.signature = NS.pageSignature(item);
    });

    var firings = [];
    pages.forEach(function (item) {
      firings = firings.concat(item.keepRulesFired || []);
    });
    return {
      pages: pages, totalPages: pages.length, sectionCount: sections.length,
      fields: fields, keepFirings: firings
    };
  };

  /* Recompute but reuse the stable page-box prefix. Only pages from the first
   * changed page onward are rebuilt; earlier boxes keep object identity. */
  NS.repaginate = function (previous, input) {
    var fresh = NS.paginate(input);
    var prior = previous && previous.pages ? previous.pages : [];
    var limit = Math.min(prior.length, fresh.pages.length);
    var reused = 0;
    while (reused < limit && prior[reused].signature === fresh.pages[reused].signature) {
      reused += 1;
    }
    for (var i = 0; i < reused; i += 1) fresh.pages[i] = prior[i];
    fresh.reusedCount = reused;
    fresh.changedFrom = reused;
    fresh.changedPages = fresh.pages.slice(reused).map(function (page) { return page.index; });
    return fresh;
  };
})(typeof window !== "undefined" ? window : globalThis);
