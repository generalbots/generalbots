"use strict";
/* SheetAdvanced module 03: charts + freeze + autofilter + protection + export/import + helpers */
(function (window) {
  const P = window.SheetAdvancedProto;
  const H = window.SheetAdvancedHelpers;
  if (!P) { console.error("Load 01_core.js first"); return; }

  const CHART_TYPES = ["line", "bar", "pie", "area", "scatter", "radar"];
  const FREEZE_KEY = "gb-sheet-freeze";
  const FILTER_KEY = "gb-sheet-filters";
  const PROTECT_KEY = "gb-sheet-protect";
  const SPARK_KEY = "gb-sheet-sparks";
  const COND_KEY = "gb-sheet-cond";
  const VAL_KEY = "gb-sheet-validation";
  const NAME_KEY = "gb-sheet-names";
  const TABLE_KEY = "gb-sheet-tables";

  function readObj(k) { try { return JSON.parse(localStorage.getItem(k) || "{}"); } catch (_) { return {}; } }
  function readArr(k) { try { return JSON.parse(localStorage.getItem(k) || "[]"); } catch (_) { return []; } }
  function writeObj(k, obj) { try { localStorage.setItem(k, JSON.stringify(obj)); } catch (_) {} }
  function writeArr(k, arr) { try { localStorage.setItem(k, JSON.stringify(arr)); } catch (_) {} }

  P.listChartTypes = function () { return CHART_TYPES.slice(); };
  P.createChart = function (type, dataRange, position) {
    if (CHART_TYPES.indexOf(type) < 0) return null;
    const values = this._readRangeValues(dataRange);
    if (!values.length) return null;
    let svg = '<svg class="gb-chart" width="320" height="200" viewBox="0 0 320 200" style="background:#1e293b;border:1px solid #334155;border-radius:4px;padding:8px;">';
    const max = Math.max.apply(null, values) || 1;
    const min = Math.min.apply(null, values);
    const range = max - min || 1;
    if (type === "line") {
      const pts = values.map((v, i) => (20 + i * 280 / (values.length - 1)) + "," + (180 - (v - min) / range * 160)).join(" ");
      svg += '<polyline fill="none" stroke="#3b82f6" stroke-width="2" points="' + pts + '"/>';
    } else if (type === "bar") {
      const w = 260 / values.length;
      values.forEach((v, i) => {
        const h = (v - min) / range * 160;
        svg += '<rect x="' + (20 + i * w) + '" y="' + (180 - h) + '" width="' + (w - 2) + '" height="' + h + '" fill="#10b981"/>';
      });
    } else if (type === "pie") {
      const total = values.reduce((a, b) => a + b, 0) || 1;
      let acc = 0;
      const cx = 160, cy = 100, r = 70;
      const colors = ["#3b82f6", "#10b981", "#f59e0b", "#ef4444", "#a855f7", "#06b6d4", "#84cc16", "#ec4899"];
      values.forEach((v, i) => {
        const start = (acc / total) * 2 * Math.PI - Math.PI / 2;
        acc += v;
        const end = (acc / total) * 2 * Math.PI - Math.PI / 2;
        const x1 = cx + r * Math.cos(start), y1 = cy + r * Math.sin(start);
        const x2 = cx + r * Math.cos(end), y2 = cy + r * Math.sin(end);
        const large = end - start > Math.PI ? 1 : 0;
        svg += '<path d="M ' + cx + ' ' + cy + ' L ' + x1 + ' ' + y1 + ' A ' + r + ' ' + r + ' 0 ' + large + ' 1 ' + x2 + ' ' + y2 + ' Z" fill="' + colors[i % colors.length] + '"/>';
      });
      svg += '<circle cx="' + cx + '" cy="' + cy + '" r="30" fill="#1e293b"/>';
    } else if (type === "area") {
      let path = "M 20 180 ";
      values.forEach((v, i) => {
        path += "L " + (20 + i * 280 / (values.length - 1)) + " " + (180 - (v - min) / range * 160) + " ";
      });
      path += "L 300 180 Z";
      svg += '<path d="' + path + '" fill="#3b82f6" fill-opacity="0.4" stroke="#3b82f6" stroke-width="1"/>';
    } else if (type === "scatter") {
      values.forEach((v, i) => {
        const x = 20 + i * 280 / (values.length - 1);
        const y = 180 - (v - min) / range * 160;
        svg += '<circle cx="' + x + '" cy="' + y + '" r="3" fill="#ec4899"/>';
      });
    } else if (type === "radar") {
      const cx = 160, cy = 100, r = 70;
      const n = values.length;
      let pts = "";
      values.forEach((v, i) => {
        const ang = (i / n) * 2 * Math.PI - Math.PI / 2;
        const radius = (v - min) / range * r;
        pts += (cx + radius * Math.cos(ang)) + "," + (cy + radius * Math.sin(ang)) + " ";
      });
      svg += '<polygon points="' + pts + '" fill="#a855f7" fill-opacity="0.4" stroke="#a855f7" stroke-width="1"/>';
      for (let i = 0; i < n; i++) {
        const ang = (i / n) * 2 * Math.PI - Math.PI / 2;
        svg += '<line x1="' + cx + '" y1="' + cy + '" x2="' + (cx + r * Math.cos(ang)) + '" y2="' + (cy + r * Math.sin(ang)) + '" stroke="#334155"/>';
      }
    }
    svg += "</svg>";
    const wrap = document.createElement("div");
    wrap.className = "gb-chart-wrap";
    wrap.style.cssText = "display:inline-block;margin:8px;";
    wrap.dataset.type = type;
    wrap.dataset.range = dataRange;
    wrap.innerHTML = svg + '<div style="text-align:center;font-size:11px;color:#94a3b8;">' + type + ': ' + dataRange + '</div>';
    if (position && this.grid.parentNode) {
      this.grid.parentNode.appendChild(wrap);
    } else {
      return wrap.outerHTML;
    }
    return wrap;
  };

  P.setFreezePanes = function (rows, cols) {
    const map = readObj(FREEZE_KEY);
    map[this.sheetId] = { rows: rows || 0, cols: cols || 0 };
    writeObj(FREEZE_KEY, map);
    this._renderFreezePanes();
  };
  P.getFreezePanes = function () { return readObj(FREEZE_KEY)[this.sheetId] || { rows: 0, cols: 0 }; };
  P._renderFreezePanes = function () {
    const f = this.getFreezePanes();
    this.grid.style.setProperty("--gb-freeze-rows", f.rows);
    this.grid.style.setProperty("--gb-freeze-cols", f.cols);
    this.grid.dataset.freezeRows = f.rows;
    this.grid.dataset.freezeCols = f.cols;
  };
  P.setAutoFilter = function (range) {
    const map = readObj(FILTER_KEY);
    map[this.sheetId] = range;
    writeObj(FILTER_KEY, map);
  };
  P.getAutoFilter = function () { return readObj(FILTER_KEY)[this.sheetId] || null; };
  P.toggleAutoFilter = function () {
    const f = this.getAutoFilter();
    if (f) { const map = readObj(FILTER_KEY); delete map[this.sheetId]; writeObj(FILTER_KEY, map); }
    else this.setAutoFilter("A1:Z1");
  };
  P.goalSeek = function (targetCell, changingCell, goal) {
    const targetEl = this.grid.querySelector("[data-cell-ref='" + targetCell + "']");
    const changeEl = this.grid.querySelector("[data-cell-ref='" + changingCell + "']");
    if (!targetEl || !changeEl) return { ok: false, msg: "Célula inválida" };
    let low = -1e9, high = 1e9;
    for (let i = 0; i < 50; i++) {
      const mid = (low + high) / 2;
      changeEl.textContent = String(mid);
      const cur = parseFloat(targetEl.textContent) || 0;
      if (Math.abs(cur - goal) < 0.001) return { ok: true, value: mid, current: cur };
      if (cur < goal) low = mid; else high = mid;
    }
    return { ok: false, msg: "Não convergiu" };
  };
  P.protectSheet = function (password) {
    const map = readObj(PROTECT_KEY);
    if (password) map[this.sheetId] = { locked: true, hash: btoa(String(password)) };
    else map[this.sheetId] = { locked: true, hash: null };
    writeObj(PROTECT_KEY, map);
    this.protected = true;
  };
  P.unprotectSheet = function (password) {
    const map = readObj(PROTECT_KEY);
    const cfg = map[this.sheetId];
    if (!cfg) return true;
    if (cfg.hash && btoa(String(password)) !== cfg.hash) return false;
    delete map[this.sheetId];
    writeObj(PROTECT_KEY, map);
    this.protected = false;
    return true;
  };
  P.isProtected = function () { return !!this.protected; };

  P.xlookup = function (lookup, arr, ret, notFound) {
    const idx = arr.indexOf(lookup);
    return idx >= 0 ? ret[idx] : (notFound || null);
  };
  P.let = function (vars, calc) { return calc(); };
  P.lambda = function (args, body) {
    return function () {
      const params = Array.prototype.slice.call(arguments);
      const scope = {};
      args.forEach((a, i) => { scope[a] = params[i]; });
      return body(scope);
    };
  };

  P.exportJSON = function () {
    return JSON.stringify({
      sparklines: readArr(SPARK_KEY + ":" + this.sheetId),
      conditionals: readObj(COND_KEY)[this.sheetId] || [],
      validations: readObj(VAL_KEY)[this.sheetId] || [],
      names: readObj(NAME_KEY)[this.sheetId] || {},
      tables: readObj(TABLE_KEY)[this.sheetId] || {},
      freeze: readObj(FREEZE_KEY)[this.sheetId] || { rows: 0, cols: 0 },
      autoFilter: readObj(FILTER_KEY)[this.sheetId] || null
    }, null, 2);
  };
  P.importJSON = function (data) {
    try {
      const obj = typeof data === "string" ? JSON.parse(data) : data;
      if (obj.sparklines) writeArr(SPARK_KEY + ":" + this.sheetId, obj.sparklines);
      if (obj.conditionals) { const m = readObj(COND_KEY); m[this.sheetId] = obj.conditionals; writeObj(COND_KEY, m); }
      if (obj.validations) { const m = readObj(VAL_KEY); m[this.sheetId] = obj.validations; writeObj(VAL_KEY, m); }
      if (obj.names) { const m = readObj(NAME_KEY); m[this.sheetId] = obj.names; writeObj(NAME_KEY, m); }
      if (obj.tables) { const m = readObj(TABLE_KEY); m[this.sheetId] = obj.tables; writeObj(TABLE_KEY, m); }
      if (obj.freeze) { const m = readObj(FREEZE_KEY); m[this.sheetId] = obj.freeze; writeObj(FREEZE_KEY, m); }
      if (obj.autoFilter) { const m = readObj(FILTER_KEY); m[this.sheetId] = obj.autoFilter; writeObj(FILTER_KEY, m); }
      this._bind();
      return true;
    } catch (_) { return false; }
  };

  window.SheetAdvanced = {
    init: window.SheetAdvanced_init,
    _proto: P,
    CHART_TYPES: CHART_TYPES,
    parseA1Ref: H.parseA1Ref,
    parseRange: H.parseRange
  };
})(window);
