"use strict";
/* Advanced charting — range binding and local statistics (#2490).
 * Plain script, loaded after 18_charts.js. Never writes back to the server:
 * it only reads the in-memory grid and produces chart-model compatible data.
 */

(function () {
  function cellValue(row, col) {
    const grid = window.SheetCore && window.SheetCore.getGrid ? window.SheetCore.getGrid() : null;
    if (grid && grid.cells) {
      const fromMap = grid.cells instanceof Map ? grid.cells.get(row + "," + col) : grid.cells[row + "," + col];
      if (fromMap && fromMap.value != null) return fromMap.value;
      if (fromMap && fromMap.formula) return fromMap.formula;
    }
    const sheet = window.__LOADED_SHEET;
    let wsIndex = 0;
    try {
      const parsed = parseInt(sessionStorage.getItem("sheet_active_index") || "0", 10);
      if (!isNaN(parsed) && parsed >= 0) wsIndex = parsed;
    } catch (_) {
      wsIndex = 0;
    }
    const ws = sheet && sheet.worksheets && sheet.worksheets[wsIndex];
    if (ws && ws.cells) {
      const stored = ws.cells instanceof Map ? ws.cells.get(row + "," + col) : ws.cells[row + "," + col];
      if (stored && stored.value != null) return stored.value;
      if (stored && stored.formula) return stored.formula;
    }
    return "";
  }

  function parseRef(ref) {
    if (typeof ref !== "string") return null;
    const cleaned = ref.replace(/\$/g, "").trim();
    const match = /^([A-Za-z]{1,3})([0-9]+)$/.exec(cleaned);
    if (!match) return null;
    let col = 0;
    const letters = match[1].toUpperCase();
    for (let i = 0; i < letters.length; i++) col = col * 26 + (letters.charCodeAt(i) - 64);
    return (parseInt(match[2], 10) < 1 || col > 16384) ? null : { col: col - 1, row: parseInt(match[2], 10) - 1 };
  }

  function parseRange(text) {
    if (typeof text !== "string") return null;
    const parts = text.trim().split("!");
    const refs = parts[parts.length - 1].split(":");
    if (refs.length !== 2) return null;
    const start = parseRef(refs[0]);
    const end = parseRef(refs[1]);
    if (!start || !end) return null;
    return {
      sheet: parts.length > 1 ? parts[0] : null,
      start: { row: Math.min(start.row, end.row), col: Math.min(start.col, end.col) },
      end: { row: Math.max(start.row, end.row), col: Math.max(start.col, end.col) },
    };
  }

  function isNumeric(value) {
    if (value === "" || value === null || typeof value === "boolean") return false;
    return Number.isFinite(Number(value));
  }

  function clamp(value, low, high) {
    let out = value;
    if (Number.isFinite(low) && out < low) out = low;
    if (Number.isFinite(high) && out > high) out = high;
    return out;
  }

  function boundsOf(options) {
    const src = options || {};
    const low = src.min === "" || src.min == null ? NaN : Number(src.min);
    const high = src.max === "" || src.max == null ? NaN : Number(src.max);
    return { low: Number.isFinite(low) ? low : NaN, high: Number.isFinite(high) ? high : NaN };
  }

  function bindRange(text, options) {
    const range = parseRange(text);
    if (!range) return { range: text || "", labels: [], series: [] };
    const bounds = boundsOf(options);
    const rowCount = range.end.row - range.start.row + 1;
    const labelColumnIsData = (function () {
      for (let r = range.start.row; r <= range.end.row; r++) {
        const value = cellValue(r, range.start.col);
        if (value !== "" && !isNumeric(value)) return false;
      }
      return true;
    })();
    const firstDataCol = labelColumnIsData ? range.start.col : range.start.col + 1;
    if (firstDataCol > range.end.col) return { range: text, labels: [], series: [] };

    const firstRowHasHeader = (function () {
      for (let c = firstDataCol; c <= range.end.col; c++) {
        const value = cellValue(range.start.row, c);
        if (value !== "" && !isNumeric(value)) return true;
      }
      return false;
    })();
    const firstDataRow = range.start.row + (firstRowHasHeader ? 1 : 0);
    const labels = [];
    const rows = [];
    for (let r = firstDataRow; r <= range.end.row; r++) {
      if (labelColumnIsData) labels.push(String(r + 1));
      else {
        const raw = cellValue(r, range.start.col);
        labels.push(raw === "" ? String(r + 1) : String(raw));
      }
      const row = [];
      for (let c = firstDataCol; c <= range.end.col; c++) {
        const raw = cellValue(r, c);
        const numeric = Number(raw);
        row.push(raw === "" || !Number.isFinite(numeric) ? null : clamp(numeric, bounds.low, bounds.high));
      }
      rows.push(row);
    }
    const series = [];
    for (let c = firstDataCol; c <= range.end.col; c++) {
      const header = firstRowHasHeader ? String(cellValue(range.start.row, c) || "") : "";
      series.push({
        label: header,
        data: rows.map(function (row) { return row[c - firstDataCol]; }),
      });
    }
    return { range: text, labels: labels.slice(0, rowCount), series: series };
  }

  function numbers(values) {
    return (values || []).map(function (value) {
      const numeric = Number(value);
      return Number.isFinite(numeric) ? numeric : null;
    });
  }

  function model(definition) {
    const def = definition || {};
    const options = def.options || {};
    let labels = [];
    let series = [];
    if (def.datasets && def.datasets.length) {
      labels = (def.labels || []).slice();
      series = def.datasets.map(function (dataset) {
        return {
          label: dataset.label || "",
          color: dataset.color || "",
          data: numbers(dataset.data).map(function (value) {
            return value == null ? null : clamp(value, boundsOf(options).low, boundsOf(options).high);
          }),
        };
      });
      if (!labels.length && series.length) {
        const count = series[0].data.length;
        for (let i = 0; i < count; i++) labels.push(String(i + 1));
      }
    } else if (def.data_range) {
      const bound = bindRange(def.data_range, options);
      labels = bound.labels;
      series = bound.series;
    }
    const values = [];
    series.forEach(function (item) {
      item.data.forEach(function (value) {
        if (value != null) values.push(value);
      });
    });
    const extent = values.length
      ? { min: Math.min.apply(null, values), max: Math.max.apply(null, values) }
      : { min: 0, max: 0 };
    return {
      labels: labels,
      series: series,
      values: values,
      min: extent.min,
      max: extent.max,
      range: def.data_range || "",
    };
  }

  function stats(values) {
    const clean = numbers(values).filter(function (value) { return value != null; });
    if (!clean.length) return { count: 0, mean: 0, sd: 0, min: 0, max: 0 };
    const mean = clean.reduce(function (sum, value) { return sum + value; }, 0) / clean.length;
    const variance = clean.reduce(function (sum, value) { return sum + Math.pow(value - mean, 2); }, 0) / clean.length;
    return {
      count: clean.length,
      mean: mean,
      sd: Math.sqrt(variance),
      min: Math.min.apply(null, clean),
      max: Math.max.apply(null, clean),
    };
  }

  function quantile(sorted, probability) {
    if (!sorted.length) return 0;
    const position = (sorted.length - 1) * Math.max(0, Math.min(1, probability));
    const lower = Math.floor(position);
    const upper = Math.ceil(position);
    if (lower === upper) return sorted[lower];
    return sorted[lower] + (sorted[upper] - sorted[lower]) * (position - lower);
  }

  function fiveNumber(values) {
    const sorted = numbers(values).filter(function (value) { return value != null; }).slice().sort(function (a, b) { return a - b; });
    if (!sorted.length) return { min: 0, q1: 0, median: 0, q3: 0, max: 0, outliers: [] };
    const q1 = quantile(sorted, 0.25);
    const q3 = quantile(sorted, 0.75);
    const iqr = q3 - q1;
    const lowFence = q1 - 1.5 * iqr;
    const highFence = q3 + 1.5 * iqr;
    return {
      min: sorted.filter(function (value) { return value >= lowFence; })[0] != null
        ? sorted.filter(function (value) { return value >= lowFence; })[0] : sorted[0],
      q1: q1,
      median: quantile(sorted, 0.5),
      q3: q3,
      max: sorted.filter(function (value) { return value <= highFence; }).slice(-1)[0] != null
        ? sorted.filter(function (value) { return value <= highFence; }).slice(-1)[0] : sorted[sorted.length - 1],
      outliers: sorted.filter(function (value) { return value < lowFence || value > highFence; }),
    };
  }

  function regression(values) {
    const points = numbers(values).map(function (value, index) { return { x: index, y: value }; })
      .filter(function (point) { return point.y != null; });
    if (!points.length) return { slope: 0, intercept: 0, predict: function () { return 0; } };
    const count = points.length;
    const meanX = points.reduce(function (sum, point) { return sum + point.x; }, 0) / count;
    const meanY = points.reduce(function (sum, point) { return sum + point.y; }, 0) / count;
    let numerator = 0;
    let denominator = 0;
    points.forEach(function (point) {
      numerator += (point.x - meanX) * (point.y - meanY);
      denominator += Math.pow(point.x - meanX, 2);
    });
    const slope = denominator ? numerator / denominator : 0;
    const intercept = meanY - slope * meanX;
    return {
      slope: slope,
      intercept: intercept,
      predict: function (x) { return intercept + slope * x; },
    };
  }

  function exponential(values) {
    const points = numbers(values).map(function (value, index) { return { x: index, y: value }; })
      .filter(function (point) { return point.y != null && point.y > 0; });
    if (!points.length) return { predict: function () { return 0; } };
    const logs = points.map(function (point) { return point.y ? Math.log(point.y) : 0; });
    const linearFit = regression(logs);
    return { predict: function (x) { return Math.exp(linearFit.predict(x)); } };
  }

  function movingAverage(values, period) {
    const clean = numbers(values);
    const width = Math.max(2, Math.min(clean.length || 2, Math.floor(Number(period) || 3)));
    return clean.map(function (_, index) {
      const from = Math.max(0, index - width + 1);
      const windowValues = clean.slice(from, index + 1).filter(function (value) { return value != null; });
      if (!windowValues.length) return null;
      return windowValues.reduce(function (sum, value) { return sum + value; }, 0) / windowValues.length;
    });
  }

  window.SheetChartingData = {
    bindRange: bindRange,
    parseRange: parseRange,
    parseRef: parseRef,
    cellValue: cellValue,
    clamp: clamp,
    model: model,
    stats: stats,
    fiveNumber: fiveNumber,
    regression: regression,
    exponential: exponential,
    movingAverage: movingAverage,
  };
})();
