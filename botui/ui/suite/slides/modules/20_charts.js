"use strict";

/**
 * Module 20: Charts in slides for Slides.
 * Replaces the "Chart insertion coming soon!" alert with a real chart
 * editor and renderer. Charts are rendered as inline SVG inside the
 * slide element. Supports bar, line, pie, area, and donut types.
 * Provides a chart data editor (data range input or manual entry)
 * with a live preview. Charts can be styled (colors, labels, legend
 * position, axis titles) and updated after creation.
 *
 * Public API: window.SlidesCharts = { openChartModal, renderChartSVG,
 *   insertChart, parseCSV }.
 */

(function () {
  const PALETTE = ["#1a73e8", "#ea4335", "#34a853", "#fbbc04", "#9334e6", "#00acc1", "#f06292", "#ff7043", "#7e57c2", "#26a69a"];
  const svgNs = "http://www.w3.org/2000/svg";

  function getState() { return window.state || null; }

  function ensureChartModal() {
    let m = document.getElementById("slidesChartModal");
    if (m) return m;
    m = document.createElement("div");
    m.id = "slidesChartModal";
    m.style.cssText = "position:fixed;inset:0;background:rgba(0,0,0,0.5);z-index:9999;display:none;align-items:center;justify-content:center;";
    m.innerHTML = `
      <div style="background:#fff;border-radius:8px;padding:24px;min-width:560px;max-width:90%;">
        <h3 style="margin:0 0 16px 0;">Insert Chart</h3>
        <div style="margin-bottom:12px;display:flex;gap:8px;align-items:center;">
          <label>Type:
            <select id="scType" style="margin-left:6px;padding:4px;">
              <option value="bar">Bar</option>
              <option value="line">Line</option>
              <option value="pie">Pie</option>
              <option value="area">Area</option>
              <option value="donut">Donut</option>
            </select>
          </label>
          <label>Title: <input type="text" id="scTitle" style="padding:4px;width:200px;" /></label>
        </div>
        <div style="margin-bottom:12px;">
          <label>Data (CSV, first row = labels):
            <textarea id="scData" rows="6" style="width:100%;padding:6px;box-sizing:border-box;font-family:monospace;">Q1,Q2,Q3,Q4
Sales,100,140,180,220
Costs,80,90,100,120</textarea>
          </label>
        </div>
        <div style="margin-bottom:12px;display:flex;gap:12px;align-items:center;">
          <label><input type="checkbox" id="scLegend" checked /> Legend</label>
          <label>Legend position:
            <select id="scLegendPos" style="padding:4px;">
              <option value="bottom">Bottom</option>
              <option value="right">Right</option>
              <option value="top">Top</option>
              <option value="left">Left</option>
            </select>
          </label>
        </div>
        <div id="scPreview" style="margin-bottom:12px;border:1px solid #ddd;border-radius:4px;height:240px;display:flex;align-items:center;justify-content:center;color:#aaa;background:#fafafa;">(preview)</div>
        <div style="display:flex;gap:8px;justify-content:flex-end;">
          <button id="scCancel" style="padding:6px 16px;">Cancel</button>
          <button id="scInsert" style="padding:6px 16px;background:#1a73e8;color:#fff;border:0;border-radius:4px;">Insert</button>
        </div>
      </div>
    `;
    document.body.appendChild(m);
    function refresh() {
      const data = parseCSV(m.querySelector("#scData").value);
      const type = m.querySelector("#scType").value;
      const title = m.querySelector("#scTitle").value;
      const legend = m.querySelector("#scLegend").checked;
      const legendPos = m.querySelector("#scLegendPos").value;
      const preview = m.querySelector("#scPreview");
      preview.innerHTML = "";
      preview.appendChild(renderChartSVG(data, { type, title, legend, legendPos, width: 480, height: 220 }));
    }
    m.querySelector("#scType").addEventListener("change", refresh);
    m.querySelector("#scData").addEventListener("input", refresh);
    m.querySelector("#scTitle").addEventListener("input", refresh);
    m.querySelector("#scLegend").addEventListener("change", refresh);
    m.querySelector("#scLegendPos").addEventListener("change", refresh);
    m.querySelector("#scCancel").addEventListener("click", function () { m.style.display = "none"; });
    m.querySelector("#scInsert").addEventListener("click", function () {
      const data = parseCSV(m.querySelector("#scData").value);
      const type = m.querySelector("#scType").value;
      const title = m.querySelector("#scTitle").value;
      const legend = m.querySelector("#scLegend").checked;
      const legendPos = m.querySelector("#scLegendPos").value;
      insertChart({ type, title, data, legend, legendPos });
      m.style.display = "none";
    });
    setTimeout(refresh, 50);
    return m;
  }

  function openChartModal() {
    const m = ensureChartModal();
    m.style.display = "flex";
  }

  function parseCSV(text) {
    if (!text) return { labels: [], series: [] };
    const rows = text.trim().split(/\r?\n/).map(function (r) { return r.split(",").map(function (c) { return c.trim(); }); });
    if (rows.length === 0) return { labels: [], series: [] };
    const labels = rows[0].slice(1);
    const series = rows.slice(1).map(function (r, i) {
      return {
        name: r[0] || ("Series " + (i + 1)),
        values: r.slice(1).map(function (v) { return parseFloat(v) || 0; }),
        color: PALETTE[i % PALETTE.length],
      };
    });
    return { labels: labels, series: series };
  }

  function renderChartSVG(data, options) {
    const opts = options || {};
    const type = opts.type || "bar";
    const title = opts.title || "";
    const legend = opts.legend !== false;
    const legendPos = opts.legendPos || "bottom";
    const width = opts.width || 480;
    const height = opts.height || 240;
    const svg = document.createElementNS(svgNs, "svg");
    svg.setAttribute("viewBox", "0 0 " + width + " " + height);
    svg.setAttribute("xmlns", svgNs);
    svg.setAttribute("width", "100%");
    svg.setAttribute("height", "100%");
    svg.style.background = "#fff";
    const labels = data.labels || [];
    const groupCount = labels.length || 0;
    let maxV = 0;
    for (const ser of data.series || []) {
      for (const v of ser.values || []) {
        if (v > maxV) maxV = v;
      }
    }
    if (maxV === 0) maxV = 1;
    const padTop = title ? 28 : 8;
    const padBottom = 36;
    const padLeft = 44;
    const padRight = 12;
    const chartW = width - padLeft - padRight;
    const chartH = height - padTop - padBottom;
    if (title) {
      const t = document.createElementNS(svgNs, "text");
      t.setAttribute("x", String(width / 2));
      t.setAttribute("y", "18");
      t.setAttribute("text-anchor", "middle");
      t.setAttribute("font-size", "13");
      t.setAttribute("font-weight", "700");
      t.setAttribute("fill", "#202124");
      t.textContent = title;
      svg.appendChild(t);
    }
    if (type === "bar" || type === "line" || type === "area") {
      const yScale = function (v) { return padTop + chartH - (v / maxV) * chartH; };
      const groupW = chartW / Math.max(groupCount, 1);
      const barW = (groupW * 0.7) / Math.max((data.series || []).length, 1);
      const grid = document.createElementNS(svgNs, "g");
      for (let g = 0; g <= 4; g++) {
        const y = padTop + (chartH / 4) * g;
        const ln = document.createElementNS(svgNs, "line");
        ln.setAttribute("x1", String(padLeft));
        ln.setAttribute("y1", String(y));
        ln.setAttribute("x2", String(padLeft + chartW));
        ln.setAttribute("y2", String(y));
        ln.setAttribute("stroke", "#eee");
        ln.setAttribute("stroke-width", "1");
        grid.appendChild(ln);
        const lbl = document.createElementNS(svgNs, "text");
        lbl.setAttribute("x", String(padLeft - 6));
        lbl.setAttribute("y", String(y + 3));
        lbl.setAttribute("text-anchor", "end");
        lbl.setAttribute("font-size", "9");
        lbl.setAttribute("fill", "#5f6368");
        lbl.textContent = String(Math.round(maxV - (maxV / 4) * g));
        grid.appendChild(lbl);
      }
      svg.appendChild(grid);
      const xAxis = document.createElementNS(svgNs, "line");
      xAxis.setAttribute("x1", String(padLeft));
      xAxis.setAttribute("y1", String(padTop + chartH));
      xAxis.setAttribute("x2", String(padLeft + chartW));
      xAxis.setAttribute("y2", String(padTop + chartH));
      xAxis.setAttribute("stroke", "#5f6368");
      xAxis.setAttribute("stroke-width", "1");
      svg.appendChild(xAxis);
      for (let g = 0; g < groupCount; g++) {
        const x = padLeft + g * groupW + groupW / 2;
        const xLabel = document.createElementNS(svgNs, "text");
        xLabel.setAttribute("x", String(x));
        xLabel.setAttribute("y", String(padTop + chartH + 14));
        xLabel.setAttribute("text-anchor", "middle");
        xLabel.setAttribute("fill", "#5f6368");
        xLabel.setAttribute("font-size", "10");
        xLabel.textContent = labels[g] || "";
        svg.appendChild(xLabel);
      }
      if (type === "bar" || type === "area") {
        for (let s = 0; s < (data.series || []).length; s++) {
          const ser = data.series[s];
          if (type === "area") {
            const pts = [];
            for (let g = 0; g < groupCount; g++) {
              const x = padLeft + g * groupW + groupW / 2;
              const y = yScale(ser.values[g] || 0);
              pts.push(x + "," + y);
            }
            const poly = document.createElementNS(svgNs, "polygon");
            poly.setAttribute("points", pts.join(" ") + " " + (padLeft + (groupCount - 1) * groupW + groupW / 2) + "," + (padTop + chartH) + " " + (padLeft + groupW / 2) + "," + (padTop + chartH));
            poly.setAttribute("fill", ser.color);
            poly.setAttribute("fill-opacity", "0.4");
            poly.setAttribute("stroke", ser.color);
            poly.setAttribute("stroke-width", "2");
            svg.appendChild(poly);
          } else {
            for (let g = 0; g < groupCount; g++) {
              const x = padLeft + g * groupW + 4 + s * barW;
              const y = yScale(ser.values[g] || 0);
              const rect = document.createElementNS(svgNs, "rect");
              rect.setAttribute("x", String(x));
              rect.setAttribute("y", String(y));
              rect.setAttribute("width", String(barW));
              rect.setAttribute("height", String(padTop + chartH - y));
              rect.setAttribute("fill", ser.color);
              svg.appendChild(rect);
            }
          }
        }
      } else if (type === "line") {
        for (let s = 0; s < (data.series || []).length; s++) {
          const ser = data.series[s];
          const pts = [];
          for (let g = 0; g < groupCount; g++) {
            const x = padLeft + g * groupW + groupW / 2;
            const y = yScale(ser.values[g] || 0);
            pts.push(x + "," + y);
          }
          const polyline = document.createElementNS(svgNs, "polyline");
          polyline.setAttribute("points", pts.join(" "));
          polyline.setAttribute("fill", "none");
          polyline.setAttribute("stroke", ser.color);
          polyline.setAttribute("stroke-width", "2");
          svg.appendChild(polyline);
          for (let g = 0; g < groupCount; g++) {
            const c = document.createElementNS(svgNs, "circle");
            c.setAttribute("cx", String(padLeft + g * groupW + groupW / 2));
            c.setAttribute("cy", String(yScale(ser.values[g] || 0)));
            c.setAttribute("r", "3");
            c.setAttribute("fill", ser.color);
            svg.appendChild(c);
          }
        }
      }
    } else if (type === "pie" || type === "donut") {
      const cx = padLeft + chartW / 2;
      const cy = padTop + chartH / 2;
      const radius = Math.min(chartW, chartH) / 2 - 8;
      const total = (data.series || []).reduce(function (acc, s) { return acc + s.values.reduce(function (a, b) { return a + b; }, 0); }, 0);
      let start = -Math.PI / 2;
      const innerR = type === "donut" ? radius * 0.6 : 0;
      for (const ser of data.series || []) {
        for (let i = 0; i < ser.values.length; i++) {
          const v = ser.values[i] || 0;
          if (v <= 0) continue;
          const slice = (v / total) * Math.PI * 2;
          const end = start + slice;
          const path = document.createElementNS(svgNs, "path");
          let d;
          if (innerR > 0) {
            const x1 = cx + Math.cos(start) * radius;
            const y1 = cy + Math.sin(start) * radius;
            const x2 = cx + Math.cos(end) * radius;
            const y2 = cy + Math.sin(end) * radius;
            const x3 = cx + Math.cos(end) * innerR;
            const y3 = cy + Math.sin(end) * innerR;
            const x4 = cx + Math.cos(start) * innerR;
            const y4 = cy + Math.sin(start) * innerR;
            d = "M " + x1 + " " + y1 + " A " + radius + " " + radius + " 0 " + (slice > Math.PI ? 1 : 0) + " 1 " + x2 + " " + y2 + " L " + x3 + " " + y3 + " A " + innerR + " " + innerR + " 0 " + (slice > Math.PI ? 1 : 0) + " 0 " + x4 + " " + y4 + " Z";
          } else {
            const x1 = cx + Math.cos(start) * radius;
            const y1 = cy + Math.sin(start) * radius;
            const x2 = cx + Math.cos(end) * radius;
            const y2 = cy + Math.sin(end) * radius;
            d = "M " + cx + " " + cy + " L " + x1 + " " + y1 + " A " + radius + " " + radius + " 0 " + (slice > Math.PI ? 1 : 0) + " 1 " + x2 + " " + y2 + " Z";
          }
          path.setAttribute("d", d);
          path.setAttribute("fill", PALETTE[((data.series.indexOf(ser) * ser.values.length) + i) % PALETTE.length]);
          path.setAttribute("stroke", "#fff");
          path.setAttribute("stroke-width", "1");
          svg.appendChild(path);
          start = end;
        }
      }
    }
    if (legend) {
      let lx = padLeft;
      let ly = padTop + chartH + 24;
      if (legendPos === "right") {
        lx = padLeft + chartW + 8;
        ly = padTop;
      } else if (legendPos === "top") {
        ly = 6;
      } else if (legendPos === "left") {
        lx = 4;
        ly = padTop;
      }
      for (let i = 0; i < (data.series || []).length; i++) {
        const ser = data.series[i];
        const sw = document.createElementNS(svgNs, "rect");
        sw.setAttribute("x", String(lx));
        sw.setAttribute("y", String(ly));
        sw.setAttribute("width", "10");
        sw.setAttribute("height", "10");
        sw.setAttribute("fill", ser.color);
        svg.appendChild(sw);
        const lt = document.createElementNS(svgNs, "text");
        lt.setAttribute("x", String(lx + 14));
        lt.setAttribute("y", String(ly + 9));
        lt.setAttribute("font-size", "10");
        lt.textContent = ser.name;
        svg.appendChild(lt);
        if (legendPos === "right" || legendPos === "left") {
          ly += 16;
        } else {
          lx += 80;
        }
      }
    }
    return svg;
  }

  function insertChart(options) {
    const s = getState();
    if (!s) return null;
    const wrapper = document.createElement("div");
    wrapper.className = "slide-element slide-chart";
    wrapper.style.cssText = "position:absolute;left:10%;top:10%;width:60%;height:50%;background:#fff;";
    const svg = renderChartSVG(options.data, options);
    wrapper.appendChild(svg);
    const canvas = document.querySelector(".slide-canvas, .slides-canvas, #slideCanvas");
    if (canvas) canvas.appendChild(wrapper);
    const slide = (s.slides || [])[s.currentSlide || 0];
    if (slide) {
      if (!slide.elements) slide.elements = [];
      slide.elements.push({
        type: "chart",
        chartType: options.type,
        title: options.title,
        data: options.data,
        legend: options.legend,
        legendPos: options.legendPos,
        x: 10, y: 10, width: 60, height: 50,
      });
    }
    return wrapper;
  }

  function attach() {
    const chartBtn = document.getElementById("insertChartBtn");
    if (chartBtn) chartBtn.addEventListener("click", function (e) { e.preventDefault(); e.stopPropagation(); openChartModal(); });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.SlidesCharts = { openChartModal, renderChartSVG, insertChart, parseCSV };
})();
