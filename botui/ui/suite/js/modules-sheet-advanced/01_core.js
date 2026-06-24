"use strict";
/* SheetAdvanced module 01: bootstrap + init + sparklines + pivot + slicers */
(function (window) {
  const SPARK_KEY = "gb-sheet-sparks";

  function readObj(k) { try { return JSON.parse(localStorage.getItem(k) || "{}"); } catch (_) { return {}; } }
  function readArr(k) { try { return JSON.parse(localStorage.getItem(k) || "[]"); } catch (_) { return []; } }
  function writeObj(k, obj) { try { localStorage.setItem(k, JSON.stringify(obj)); } catch (_) {} }
  function writeArr(k, arr) { try { localStorage.setItem(k, JSON.stringify(arr)); } catch (_) {} }
  function escapeHtml(s) { return String(s).replace(/[&<>"']/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]); }

  function parseA1Ref(ref) {
    const m = ref.match(/^([A-Z]+)(\d+)$/);
    if (!m) return null;
    let col = 0;
    for (let i = 0; i < m[1].length; i++) col = col * 26 + (m[1].charCodeAt(i) - 64);
    return { row: parseInt(m[2], 10) - 1, col: col - 1 };
  }

  function parseRange(rng) {
    const m = rng.match(/^([A-Z]+\d+):([A-Z]+\d+)$/);
    if (!m) return null;
    return { start: parseA1Ref(m[1]), end: parseA1Ref(m[2]) };
  }

  const SheetAdvancedProto = {};

  function init(grid, options) {
    if (!grid) return null;
    const self = Object.create(SheetAdvancedProto);
    self.grid = grid;
    self.sheetId = (options && options.sheetId) || "current";
    self.protected = readObj("gb-sheet-protect")[self.sheetId] || false;
    self._bind();
    return self;
  }

  SheetAdvancedProto._bind = function () {
    const self = this;
    document.addEventListener("keydown", function (e) {
      if (e.ctrlKey && e.shiftKey && e.key === "L") { e.preventDefault(); self.toggleAutoFilter(); }
    });
    if (this._renderAllConditionalFormats) this._renderAllConditionalFormats();
    if (this._renderAllValidations) this._renderAllValidations();
    if (this._renderAllSparklines) this._renderAllSparklines();
    if (this._renderAllNamedRanges) this._renderAllNamedRanges();
    if (this._renderAllTables) this._renderAllTables();
    if (this._renderFreezePanes) this._renderFreezePanes();
  };

  SheetAdvancedProto.addSparkline = function (cellRef, range, type) {
    const arr = readArr(SPARK_KEY + ":" + this.sheetId);
    arr.push({ cell: cellRef, range: range, type: type || "line" });
    writeArr(SPARK_KEY + ":" + this.sheetId, arr);
    this._renderSparkline(arr[arr.length - 1]);
  };
  SheetAdvancedProto.listSparklines = function () { return readArr(SPARK_KEY + ":" + this.sheetId); };
  SheetAdvancedProto.removeSparkline = function (cellRef) {
    const arr = readArr(SPARK_KEY + ":" + this.sheetId).filter(s => s.cell !== cellRef);
    writeArr(SPARK_KEY + ":" + this.sheetId, arr);
    const el = this.grid.querySelector("[data-cell-ref='" + cellRef + "'] .gb-sparkline");
    if (el) el.remove();
  };
  SheetAdvancedProto._renderSparkline = function (sp) {
    const cell = this.grid.querySelector("[data-cell-ref='" + sp.cell + "']");
    if (!cell) return;
    const values = this._readRangeValues(sp.range);
    if (!values.length) return;
    let svg = '<svg class="gb-sparkline" width="100" height="24" viewBox="0 0 100 24" preserveAspectRatio="none" style="display:block;">';
    const min = Math.min.apply(null, values);
    const max = Math.max.apply(null, values);
    const range = max - min || 1;
    if (sp.type === "line") {
      const pts = values.map((v, i) => (i / (values.length - 1) * 100) + "," + (22 - (v - min) / range * 20)).join(" ");
      svg += '<polyline fill="none" stroke="#3b82f6" stroke-width="1.5" points="' + pts + '"/>';
    } else if (sp.type === "bar") {
      values.forEach((v, i) => {
        const h = (v - min) / range * 22 + 1;
        const x = i / values.length * 100;
        svg += '<rect x="' + x + '" y="' + (24 - h) + '" width="' + (100 / values.length - 0.5) + '" height="' + h + '" fill="#10b981"/>';
      });
    } else if (sp.type === "winloss") {
      values.forEach((v, i) => {
        const x = i / values.length * 100;
        const y = v >= 0 ? 12 : 12;
        const h = Math.abs(v) / Math.max.apply(null, values.map(Math.abs)) * 10 + 1;
        svg += '<rect x="' + x + '" y="' + (v >= 0 ? y - h : y) + '" width="' + (100 / values.length - 0.5) + '" height="' + h + '" fill="' + (v >= 0 ? "#10b981" : "#ef4444") + '"/>';
      });
    }
    svg += "</svg>";
    const existing = cell.querySelector(".gb-sparkline");
    if (existing) existing.remove();
    const div = document.createElement("div");
    div.className = "gb-sparkline";
    div.innerHTML = svg;
    cell.appendChild(div);
  };
  SheetAdvancedProto._renderAllSparklines = function () {
    const self = this;
    readArr(SPARK_KEY + ":" + this.sheetId).forEach(sp => self._renderSparkline(sp));
  };
  SheetAdvancedProto._readRangeValues = function (range) {
    const r = parseRange(range);
    if (!r) return [];
    const out = [];
    for (let row = r.start.row; row <= r.end.row; row++) {
      for (let col = r.start.col; col <= r.end.col; col++) {
        const cell = this.grid.querySelector("[data-row='" + row + "'][data-col='" + col + "']");
        if (cell) {
          const v = parseFloat(cell.textContent.replace(",", "."));
          if (!isNaN(v)) out.push(v);
        }
      }
    }
    return out;
  };

  SheetAdvancedProto.createPivot = function (config) {
    const data = parseRange(config.dataRange);
    if (!data) return null;
    const rows = [];
    for (let r = data.start.row; r <= data.end.row; r++) {
      const row = {};
      for (let c = data.start.col; c <= data.end.col; c++) {
        const cell = this.grid.querySelector("[data-row='" + r + "'][data-col='" + c + "']");
        row["col_" + c] = cell ? cell.textContent : "";
      }
      rows.push(row);
    }
    const groups = {};
    const rowKey = config.rowField;
    const valueField = config.valueField;
    const agg = config.aggregation || "sum";
    rows.forEach(r => {
      const k = r[rowKey];
      const v = parseFloat(String(r[valueField]).replace(",", ".")) || 0;
      if (!groups[k]) groups[k] = [];
      groups[k].push(v);
    });
    return Object.keys(groups).map(k => {
      const vals = groups[k];
      let agg_val = 0;
      if (agg === "sum") agg_val = vals.reduce((a, b) => a + b, 0);
      else if (agg === "avg") agg_val = vals.reduce((a, b) => a + b, 0) / vals.length;
      else if (agg === "count") agg_val = vals.length;
      else if (agg === "min") agg_val = Math.min.apply(null, vals);
      else if (agg === "max") agg_val = Math.max.apply(null, vals);
      return { key: k, value: agg_val, count: vals.length };
    });
  };
  SheetAdvancedProto.renderPivotHTML = function (config) {
    const data = this.createPivot(config);
    if (!data) return "<p>Intervalo inválido.</p>";
    return '<table class="gb-pivot" style="border-collapse:collapse;width:100%;background:#1e293b;color:#f8fafc;">' +
      '<thead><tr><th style="border:1px solid #334155;padding:6px;text-align:left;">' + escapeHtml(config.rowField) + '</th><th style="border:1px solid #334155;padding:6px;text-align:right;">' + config.aggregation + '</th><th style="border:1px solid #334155;padding:6px;text-align:right;">count</th></tr></thead>' +
      '<tbody>' + data.map(d => '<tr><td style="border:1px solid #334155;padding:6px;">' + escapeHtml(String(d.key)) + '</td><td style="border:1px solid #334155;padding:6px;text-align:right;">' + d.value.toFixed(2) + '</td><td style="border:1px solid #334155;padding:6px;text-align:right;">' + d.count + '</td></tr>').join("") + '</tbody>' +
      '</table>';
  };
  SheetAdvancedProto.addSlicer = function (column, target) {
    const map = readObj("gb-sheet-slicers");
    if (!map[this.sheetId]) map[this.sheetId] = {};
    if (!map[this.sheetId][target]) map[this.sheetId][target] = [];
    const data = this._readColumnValues(target, column);
    const unique = Array.from(new Set(data));
    map[this.sheetId][target].push({ column: column, items: unique });
    writeObj("gb-sheet-slicers", map);
  };
  SheetAdvancedProto.listSlicers = function (target) {
    const map = readObj("gb-sheet-slicers");
    return (map[this.sheetId] && map[this.sheetId][target]) || [];
  };
  SheetAdvancedProto._readColumnValues = function (range, colLetter) {
    const r = parseRange(range);
    if (!r) return [];
    const colRef = parseA1Ref(colLetter + "1");
    if (!colRef) return [];
    const out = [];
    for (let row = r.start.row; row <= r.end.row; row++) {
      const cell = this.grid.querySelector("[data-row='" + row + "'][data-col='" + colRef.col + "']");
      if (cell) out.push(cell.textContent);
    }
    return out;
  };

  window.SheetAdvancedProto = SheetAdvancedProto;
  window.SheetAdvanced_init = init;
  window.SheetAdvancedHelpers = { parseA1Ref: parseA1Ref, parseRange: parseRange, escapeHtml: escapeHtml };
})(window);
