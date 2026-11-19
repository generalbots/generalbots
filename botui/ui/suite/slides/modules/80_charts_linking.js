"use strict";
/* Slides chart Sheet-range linking (#2486) — layer 4: a faithful LOCAL model of
 * a Sheet range link. No real Sheet fetch is performed: a caller supplies a
 * value matrix explicitly (applyMatrix). State transitions (link set, source
 * changed -> STALE, refresh -> timestamp + STALE cleared) are real. */
(function (window) {
  var C = (window.GBSlidesCharts = window.GBSlidesCharts || {});
  var L = (C.link = C.link || {});
  var RANGE_RE = /^([^!]+)!([A-Z]+)(\d+):([A-Z]+)(\d+)$/i;

  L.colIndex = function (letters) {
    var text = String(letters || "").toUpperCase();
    var value = 0;
    for (var i = 0; i < text.length; i += 1) {
      value = value * 26 + (text.charCodeAt(i) - 64);
    }
    return value;
  };

  L.parseRange = function (range) {
    var match = RANGE_RE.exec(String(range || "").trim());
    if (!match) return null;
    return {
      sheet: match[1],
      startCol: L.colIndex(match[2]), startRow: parseInt(match[3], 10),
      endCol: L.colIndex(match[4]), endRow: parseInt(match[5], 10)
    };
  };

  L.validRange = function (range) {
    var parsed = L.parseRange(range);
    return !!parsed && parsed.endCol >= parsed.startCol && parsed.endRow >= parsed.startRow;
  };

  L.setLink = function (id, range, refreshOnOpen, explicit) {
    var existing = C.get(id);
    if (!existing || !L.validRange(range)) return null;
    return C.update(id, {
      link: { range: String(range).trim(), refreshOnOpen: !!refreshOnOpen, lastRefreshed: 0, stale: false }
    }, explicit);
  };

  L.clearLink = function (id, explicit) {
    var existing = C.get(id);
    if (!existing) return null;
    return C.update(id, {
      link: { range: "", refreshOnOpen: false, lastRefreshed: 0, stale: false }
    }, explicit);
  };

  L.markSourceChanged = function (id, explicit) {
    var existing = C.get(id);
    if (!existing || !existing.link.range) return null;
    return C.update(id, {
      link: Object.assign({}, existing.link, { stale: true })
    }, explicit);
  };

  L.refresh = function (id, explicit, timestamp) {
    var existing = C.get(id);
    if (!existing || !existing.link.range) return null;
    var ts = C.num(timestamp, Date.now());
    C.update(id, {
      link: Object.assign({}, existing.link, { lastRefreshed: ts, stale: false })
    }, explicit);
    return { id: id, range: existing.link.range, lastRefreshed: ts, stale: false };
  };

  L.refreshOnOpen = function (state, explicit) {
    var changed = [];
    C.list(state).forEach(function (chart) {
      if (chart.link.range && chart.link.refreshOnOpen) {
        L.refresh(chart.id, explicit);
        changed.push(chart.id);
      }
    });
    return changed;
  };

  /* Simulated fetch result: the first matrix column supplies categories, every
   * remaining column becomes a series (the first row supplies series names). */
  L.columnsFromMatrix = function (matrix, hasHeader) {
    var rows = Array.isArray(matrix) ? matrix : [];
    if (!rows.length) return { categories: [], series: [] };
    var width = rows.reduce(function (max, row) {
      return Math.max(max, Array.isArray(row) ? row.length : 0);
    }, 0);
    if (!width) return { categories: [], series: [] };
    var headerOffset = hasHeader ? 1 : 0;
    var categories = [];
    var series = [];
    for (var col = 0; col < width; col += 1) {
      var values = [];
      for (var row = headerOffset; row < rows.length; row += 1) {
        values.push((rows[row] && rows[row][col] !== undefined) ? rows[row][col] : "");
      }
      if (col === 0) {
        categories = values.map(function (value) { return String(value); });
      } else {
        series.push({
          id: "s" + col,
          name: hasHeader ? String(rows[0][col] || "") : C.tr("slides-charts-column", "Column { $n }", { n: col }),
          values: values.map(function (value) { return C.num(value, 0); }),
          axis: "primary"
        });
      }
    }
    return { categories: categories, series: series };
  };

  L.applyMatrix = function (id, matrix, hasHeader, explicit, timestamp) {
    var existing = C.get(id);
    if (!existing || !existing.link.range) return null;
    var parsed = L.parseRange(existing.link.range);
    var width = parsed.endCol - parsed.startCol + 1;
    var bounded = (matrix || []).map(function (row) {
      return (row || []).slice(0, Math.max(1, width));
    });
    var columns = L.columnsFromMatrix(bounded, hasHeader !== false);
    var existingSeries = existing.series;
    var mappedSeries = columns.series.map(function (series, index) {
      return Object.assign({}, series, {
        color: existingSeries[index] && existingSeries[index].color || "",
        axis: existingSeries[index] && existingSeries[index].axis || "primary"
      });
    });
    var ts = C.num(timestamp, Date.now());
    C.update(id, {
      categories: columns.categories,
      series: mappedSeries,
      link: Object.assign({}, existing.link, { lastRefreshed: ts, stale: false })
    }, explicit);
    return { categories: columns.categories, series: mappedSeries, lastRefreshed: ts, stale: false };
  };

  L.status = function (chart) {
    var link = chart.link || {};
    return {
      linked: !!link.range,
      range: link.range || "",
      stale: !!link.stale,
      lastRefreshed: C.num(link.lastRefreshed, 0),
      refreshOnOpen: !!link.refreshOnOpen
    };
  };
})(typeof window !== "undefined" ? window : globalThis);
