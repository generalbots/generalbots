"use strict";
/* Docs layout preview (#2424), keep rules and box normalisation.
 *
 * Every helper here is pure: it takes explicit inputs and returns plain data so
 * the keep-with-next, keep-lines-together, widow/orphan, table-row and image
 * anchor rules can be proven in an offline DOM-stub harness. */
(function (window) {
  var NS = window.DocsLayout;
  if (!NS) return;

  function groupRuns(lines, field) {
    var runs = [], current = null;
    (lines || []).forEach(function (line, index) {
      var key = line && line[field];
      if (!key) { current = null; return; }
      if (!current || current.key !== key) {
        current = { key: key, start: index, ids: [] };
        runs.push(current);
      }
      current.ids.push(line.id);
      current.end = index;
    });
    return runs;
  }
  NS.paragraphRuns = function (lines) { return groupRuns(lines, "paraId"); };
  NS.keepRuns = function (lines) { return groupRuns(lines, "keepGroup"); };

  function sumHeights(entries) {
    var total = 0;
    (entries || []).forEach(function (entry) { total += (entry && entry.metrics && entry.metrics.height) || 0; });
    return total;
  }
  NS.sumHeights = sumHeights;

  NS.capacity = function (geometry) {
    return (geometry && geometry.cols || []).reduce(function (acc, col) {
      return acc + Math.max(0, col.bottom - col.top);
    }, 0);
  };
  NS.used = function (page) { return sumHeights(page && page.lines); };
  NS.fits = function (page, height) {
    return NS.used(page) + height <= NS.capacity(page && page.geometry) + 0.5;
  };

  /* Decide whether a forced page break is required before placing `entry`. */
  NS.breakDecision = function (entry, nextEntry, page, groupHeights, settings) {
    var opts = settings || {};
    var need = (entry.metrics && entry.metrics.height) || 0;
    var rule = null;
    if (entry.keepWithNext && nextEntry) {
      need += (nextEntry.metrics && nextEntry.metrics.height) || 0;
      rule = "keep-with-next";
    }
    if (entry.keepGroup && groupHeights && groupHeights[entry.keepGroup] != null) {
      if (groupHeights[entry.keepGroup] > need) rule = "keep-lines-together";
      need = Math.max(need, groupHeights[entry.keepGroup]);
    }
    if (entry.tableRow && !opts.tableSplit) {
      rule = rule || "table-row-split";
    }
    if (!NS.fits(page, need)) return { break: true, rule: rule || "page-overflow", need: need };
    return { break: false, rule: null, need: need };
  };

  /* Move lines across a page boundary so widows/orphans are controlled.
   * Returns the keep-rule firings produced by the adjustment. */
  NS.applyWidowOrphan = function (pages, settings) {
    var opts = settings || {};
    var widows = Math.max(1, Number(opts.widows) || 2);
    var orphans = Math.max(1, Number(opts.orphans) || 2);
    var firings = [];
    for (var p = 0; p < pages.length - 1; p += 1) {
      var a = pages[p], b = pages[p + 1];
      var aLines = a.lines || [], bLines = b.lines || [];
      if (!aLines.length || !bLines.length) continue;
      var para = aLines[aLines.length - 1].paraId;
      if (!para || para !== bLines[0].paraId) continue;
      var tail = [];
      for (var t = aLines.length - 1; t >= 0 && aLines[t].paraId === para; t -= 1) {
        tail.unshift(aLines[t]);
      }
      var headCount = 0;
      for (var h = 0; h < bLines.length && bLines[h].paraId === para; h += 1) headCount += 1;
      var needWidow = Math.max(0, widows - headCount);
      var needOrphan = Math.max(0, orphans - tail.length);
      var moveCount = Math.max(needWidow, needOrphan);
      if (moveCount <= 0 || moveCount > tail.length) moveCount = Math.min(moveCount, tail.length);
      if (moveCount <= 0) continue;
      var movable = tail.slice(-moveCount);
      a.lines = aLines.slice(0, aLines.length - movable.length);
      b.lines = movable.concat(bLines);
      var fired = [];
      if (needWidow > 0) fired.push("widow-control");
      if (needOrphan > 0) fired.push("orphan-control");
      movable.forEach(function (entry) {
        (b.keepRulesFired || (b.keepRulesFired = [])).push({
          rule: fired.join("+"), id: entry.id, page: b.index
        });
      });
      firings = firings.concat(fired.map(function (name) {
        return { rule: name, id: movable[movable.length - 1].id, page: b.index };
      }));
    }
    return firings;
  };

  /* Recompute stable line boxes (column + y) from a page's ordered lines. */
  NS.normalizeBoxes = function (page) {
    var geometry = page.geometry || {};
    var cols = geometry.cols && geometry.cols.length
      ? geometry.cols : [{ index: 0, top: 0, bottom: geometry.bodyHeight || 0 }];
    var colIndex = 0, y = cols[0].top;
    var boxes = [];
    (page.lines || []).forEach(function (entry, order) {
      var metrics = entry.metrics || { height: 0, kind: "paragraph", anchor: "inline" };
      if (metrics.anchor === "float") {
        boxes.push({
          id: entry.id, kind: metrics.kind, anchor: "float", column: cols.length - 1,
          y: cols[cols.length - 1].top, height: metrics.height, lineIndex: entry.lineIndex, order: order
        });
        return;
      }
      /* Only spill to the next column once the current one has content;
       * an empty column must still receive its first line even when that
       * single line is taller than the column box. */
      if (y > cols[colIndex].top && y + metrics.height > cols[colIndex].bottom && colIndex + 1 < cols.length) {
        colIndex += 1;
        y = cols[colIndex].top;
      }
      boxes.push({
        id: entry.id, kind: metrics.kind, anchor: "inline", column: colIndex,
        y: y, height: metrics.height, lineIndex: entry.lineIndex, order: order
      });
      y += metrics.height;
    });
    page.boxes = boxes;
    page.lineCount = boxes.filter(function (box) { return box.anchor !== "float"; }).length;
    return boxes;
  };

  NS.footnoteOf = function (line) {
    return line && line.footnote ? line.footnote : null;
  };
})(typeof window !== "undefined" ? window : globalThis);
