"use strict";
/* Sheet advanced module: 18_charts — render stored charts over the grid via inline SVG */

(function () {
  const HEADER_W = 48;

  function grid() {
    if (window.SheetCore && window.SheetCore.getGrid) return window.SheetCore.getGrid();
    return window.SheetVirtualGrid || null;
  }

  function wsIndex() {
    if (window.SheetCore && window.SheetCore.wsIndex) return window.SheetCore.wsIndex();
    try {
      const v = parseInt(sessionStorage.getItem("sheet_active_index") || "0", 10);
      return isNaN(v) || v < 0 ? 0 : v;
    } catch (_) {
      return 0;
    }
  }

  function cw(idx) {
    if (window.SheetCore && window.SheetCore.colWidth) return window.SheetCore.colWidth(idx);
    return 96;
  }

  function cx(idx) {
    if (window.SheetCore && window.SheetCore.colX) return window.SheetCore.colX(idx);
    return HEADER_W + idx * 96;
  }

  function rh(idx) {
    if (window.SheetCore && window.SheetCore.rowHeight) return window.SheetCore.rowHeight(idx);
    return 24;
  }

  function chartRowTop(row) {
    let y = 0;
    for (let i = 0; i < row; i++) y += rh(i);
    return y;
  }

  function charts() {
    const sheet = window.__LOADED_SHEET;
    if (!sheet || !sheet.worksheets || !sheet.worksheets[wsIndex()]) return [];
    return sheet.worksheets[wsIndex()].charts || [];
  }

  function escapeXml(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&apos;");
  }

  function barChart(c, w, h) {
    const max = c.datasets.reduce(function (m, d) { return Math.max(m, d.data.reduce(function (x, v) { return Math.max(x, v); }, 0)); }, 1);
    const labels = c.labels && c.labels.length ? c.labels : c.datasets.map(function (_, i) { return String(i + 1); });
    const pad = { l: 30, r: 10, t: 20, b: 20 };
    const plotW = w - pad.l - pad.r;
    const plotH = h - pad.t - pad.b;
    let svg = '<svg xmlns="http://www.w3.org/2000/svg" width="' + w + '" height="' + h + '" viewBox="0 0 ' + w + ' ' + h + '">';
    svg += '<rect width="100%" height="100%" fill="#0f172a" rx="5"/>';
    svg += '<text x="' + (w / 2) + '" y="16" fill="#e2e8f0" font-size="11" text-anchor="middle" font-family="sans-serif">' + escapeXml(c.title) + '</text>';
    const series = c.datasets.length || 1;
    const barW = (plotW / labels.length) * 0.7 / series;
    for (let i = 0; i < labels.length; i++) {
      for (let s = 0; s < c.datasets.length; s++) {
        const d = c.datasets[s];
        const v = d.data[i] || 0;
        const bh = Math.max(1, (v / max) * plotH);
        const x = pad.l + i * (plotW / labels.length) + (s * barW) + (plotW / labels.length) * 0.15;
        const y = pad.t + plotH - bh;
        svg += '<rect x="' + x + '" y="' + y + '" width="' + barW + '" height="' + bh + '" fill="' + escapeXml(d.color || "#3b82f6") + '"/>';
      }
      svg += '<text x="' + (pad.l + i * (plotW / labels.length) + plotW / labels.length / 2) + '" y="' + (h - 6) + '" fill="#94a3b8" font-size="9" text-anchor="middle">' + escapeXml(String(labels[i])) + '</text>';
    }
    svg += '</svg>';
    return svg;
  }

  function lineChart(c, w, h) {
    const max = c.datasets.reduce(function (m, d) { return Math.max(m, d.data.reduce(function (x, v) { return Math.max(x, v); }, 0)); }, 1) * 1.1;
    const pad = { l: 30, r: 10, t: 20, b: 20 };
    const plotW = w - pad.l - pad.r;
    const plotH = h - pad.t - pad.b;
    let svg = '<svg xmlns="http://www.w3.org/2000/svg" width="' + w + '" height="' + h + '" viewBox="0 0 ' + w + ' ' + h + '">';
    svg += '<rect width="100%" height="100%" fill="#0f172a" rx="5"/>';
    svg += '<text x="' + (w / 2) + '" y="16" fill="#e2e8f0" font-size="11" text-anchor="middle">' + escapeXml(c.title) + '</text>';
    for (let s = 0; s < c.datasets.length; s++) {
      const d = c.datasets[s];
      let pts = "";
      for (let i = 0; i < d.data.length; i++) {
        const x = pad.l + (d.data.length === 1 ? plotW / 2 : pad.l + (i / (d.data.length - 1)) * plotW);
        const y = pad.t + plotH - (d.data[i] / max) * plotH;
        pts += x + "," + y + " ";
      }
      svg += '<polyline points="' + pts.trim() + '" fill="none" stroke="' + escapeXml(d.color || "#3b82f6") + '" stroke-width="2"/>';
    }
    svg += '</svg>';
    return svg;
  }

  function pieChart(c, w, h) {
    const total = c.datasets.reduce(function (m, d) { return m + d.data.reduce(function (x, v) { return x + v; }, 0); }, 1) || 1;
    const cx2 = w / 2;
    const cy2 = h / 2 + 6;
    const r = Math.min(w, h) / 2 - 24;
    let svg = '<svg xmlns="http://www.w3.org/2000/svg" width="' + w + '" height="' + h + '" viewBox="0 0 ' + w + ' ' + h + '">';
    svg += '<rect width="100%" height="100%" fill="#0f172a" rx="5"/>';
    svg += '<text x="' + (w / 2) + '" y="16" fill="#e2e8f0" font-size="11" text-anchor="middle">' + escapeXml(c.title) + '</text>';
    const colors = ["#3b82f6", "#ef4444", "#22c55e", "#eab308", "#a855f7", "#06b6d4", "#f97316", "#ec4899"];
    let a0 = -Math.PI / 2;
    c.datasets.forEach(function (d, s) {
      d.data.forEach(function (v, i) {
        const a1 = a0 + (v / total) * 2 * Math.PI;
        if (v <= 0) return;
        const x0 = cx2 + r * Math.cos(a0);
        const y0 = cy2 + r * Math.sin(a0);
        const x1 = cx2 + r * Math.cos(a1);
        const y1 = cy2 + r * Math.sin(a1);
        const large = a1 - a0 > Math.PI ? 1 : 0;
        svg += '<path d="M' + cx2 + ',' + cy2 + ' L' + x0 + ',' + y0 + ' A' + r + ',' + r + ' 0 ' + large + ' 1 ' + x1 + ',' + y1 + ' Z" fill="' + colors[(s + i) % colors.length] + '"/>';
        a0 = a1;
      });
    });
    svg += '</svg>';
    return svg;
  }

  function renderChart(c) {
    const g = grid();
    if (!g || !g.bodyInner) return;
    const pos = c.position || { row: 0, col: 0, width: 300, height: 180 };
    const w = pos.width || 300;
    const h = pos.height || 180;
    const layer = document.createElement("div");
    layer.className = "ss-chart-float";
    layer.style.cssText =
      "position:absolute;left:" + cx(pos.col) + "px;top:" + chartRowTop(pos.row) + "px;width:" + w + "px;height:" + h + "px;" +
      "z-index:12;overflow:hidden;border:1px solid #334155;border-radius:6px;background:#0f172a;box-shadow:0 4px 12px rgba(0,0,0,0.3);pointer-events:none;";
    layer.innerHTML = c.chart_type === "pie" ? pieChart(c, w, h) : c.chart_type === "line" ? lineChart(c, w, h) : barChart(c, w, h);
    g.bodyInner.appendChild(layer);
  }

  function renderAll() {
    const g = grid();
    if (!g || !g.bodyInner) return;
    g.bodyInner.querySelectorAll(".ss-chart-float").forEach(function (el) { el.remove(); });
    const cs = charts();
    for (let i = 0; i < cs.length; i++) renderChart(cs[i]);
  }

  function wire() {
    const g = grid();
    if (!g) {
      setTimeout(wire, 100);
      return;
    }
    renderAll();
  }

  window.SheetChartsRender = {
    renderAll: renderAll,
    renderChart: renderChart,
  };

  document.addEventListener("gb-sheet-tab", function () { setTimeout(renderAll, 60); });
  document.addEventListener("gb-sheet-selection", function () { setTimeout(renderAll, 60); });

  setTimeout(wire, 0);
})();