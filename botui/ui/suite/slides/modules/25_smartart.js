"use strict";

/**
 * Module 25: SmartArt diagrams for Slides.
 * Provides pre-built diagram types (hierarchy, cycle, process, radial,
 * matrix, pyramid) generated as inline SVG. Each node is editable
 * for text, color, and shape; the entire SmartArt is treated as a
 * single grouped SlideElement.
 *
 * Public API: window.SlidesSmartArt = { create, render, getTypes,
 *   editNode, exportSVG, getDiagram }.
 */

(function () {
  const TYPES = [
    "hierarchy", "cycle", "process", "radial", "matrix", "pyramid", "venn", "funnel",
  ];
  const PALETTE = ["#4285f4", "#ea4335", "#fbbc04", "#34a853", "#ff6d01", "#46bdc6", "#7baaf7", "#f07b72"];

  function el(tag, attrs, text) {
    const e = document.createElementNS("http://www.w3.org/2000/svg", tag);
    if (attrs) for (const k in attrs) e.setAttribute(k, attrs[k]);
    if (text != null) e.textContent = text;
    return e;
  }

  function defaultNodes(type, count) {
    const n = count || 4;
    const out = [];
    for (let i = 0; i < n; i++) {
      out.push({ id: "n" + i, text: type === "pyramid" ? "Nível " + (i + 1) : "Item " + (i + 1), color: PALETTE[i % PALETTE.length] });
    }
    return out;
  }

  function build(type, nodes) {
    const N = nodes ? nodes.length : (type === "hierarchy" ? 5 : type === "cycle" ? 5 : type === "process" ? 4 : type === "matrix" ? 4 : type === "pyramid" ? 5 : 6);
    const items = nodes || defaultNodes(type, N);
    const svg = el("svg", { viewBox: "0 0 800 450", xmlns: "http://www.w3.org/2000/svg", width: "100%", height: "100%" });
    svg.style.background = "#ffffff";
    svg.style.overflow = "visible";
    if (type === "hierarchy") {
      const levels = items.length <= 4 ? 2 : 3;
      const perLevel = Math.ceil(items.length / levels);
      const xs = items.map((_, i) => 50 + (700 / Math.max(items.length - 1, 1)) * i);
      const yTop = 50;
      const yLevel2 = 200;
      const yLevel3 = 360;
      if (items.length > 4) {
        for (let i = 0; i < perLevel; i++) {
          const idx = i;
          if (!items[idx]) continue;
          svg.appendChild(el("rect", { x: xs[idx] - 50, y: yLevel2, width: 100, height: 50, fill: items[idx].color, rx: 6 }));
          const t = el("text", { x: xs[idx], y: yLevel2 + 30, "text-anchor": "middle", fill: "#fff", "font-size": 14, "font-weight": 600 }, items[idx].text);
          svg.appendChild(t);
        }
        for (let i = perLevel; i < items.length; i++) {
          const idx = i;
          svg.appendChild(el("rect", { x: xs[idx] - 45, y: yLevel3, width: 90, height: 40, fill: items[idx].color, rx: 5, opacity: 0.85 }));
          const t = el("text", { x: xs[idx], y: yLevel3 + 25, "text-anchor": "middle", fill: "#fff", "font-size": 12 }, items[idx].text);
          svg.appendChild(t);
        }
        svg.appendChild(el("ellipse", { cx: 400, cy: 20, rx: 110, ry: 25, fill: "#202124" }));
        svg.appendChild(el("text", { x: 400, y: 25, "text-anchor": "middle", fill: "#fff", "font-size": 16, "font-weight": 700 }, "TÓPICO"));
      } else {
        for (let i = 0; i < items.length; i++) {
          svg.appendChild(el("rect", { x: xs[i] - 60, y: yTop, width: 120, height: 70, fill: items[i].color, rx: 8 }));
          const t = el("text", { x: xs[i], y: yTop + 40, "text-anchor": "middle", fill: "#fff", "font-size": 16, "font-weight": 600 }, items[i].text);
          svg.appendChild(t);
        }
        const childY = 250;
        for (let i = 0; i < items.length; i++) {
          svg.appendChild(el("line", { x1: xs[i], y1: yTop + 70, x2: xs[i], y2: childY, stroke: "#9aa0a6", "stroke-width": 1.5 }));
        }
        const labels = ["Sub A", "Sub B", "Sub C", "Sub D", "Sub E", "Sub F"];
        let lblIdx = 0;
        for (let i = 0; i < items.length; i++) {
          for (let k = 0; k < 2; k++) {
            const xOff = (k === 0 ? -1 : 1) * 40;
            svg.appendChild(el("rect", { x: xs[i] + xOff - 35, y: childY, width: 70, height: 36, fill: "#f1f3f4", stroke: items[i].color, "stroke-width": 1.5, rx: 4 }));
            svg.appendChild(el("text", { x: xs[i] + xOff, y: childY + 22, "text-anchor": "middle", fill: "#3c4043", "font-size": 12 }, labels[lblIdx % labels.length]));
            lblIdx++;
          }
        }
      }
    } else if (type === "cycle") {
      const cx = 400, cy = 220, r = 130;
      const n = items.length;
      for (let i = 0; i < n; i++) {
        const a = (i / n) * Math.PI * 2 - Math.PI / 2;
        const x = cx + r * Math.cos(a);
        const y = cy + r * Math.sin(a);
        svg.appendChild(el("circle", { cx: x, cy: y, r: 50, fill: items[i].color }));
        const t = el("text", { x: x, y: y + 5, "text-anchor": "middle", fill: "#fff", "font-size": 13, "font-weight": 600 }, items[i].text);
        svg.appendChild(t);
      }
      for (let i = 0; i < n; i++) {
        const a1 = (i / n) * Math.PI * 2 - Math.PI / 2;
        const a2 = ((i + 1) / n) * Math.PI * 2 - Math.PI / 2;
        const mx = cx + r * Math.cos((a1 + a2) / 2) * 0.78;
        const my = cy + r * Math.sin((a1 + a2) / 2) * 0.78;
        const ax1 = cx + (r - 50) * Math.cos(a1);
        const ay1 = cy + (r - 50) * Math.sin(a1);
        const ax2 = cx + (r - 50) * Math.cos(a2);
        const ay2 = cy + (r - 50) * Math.sin(a2);
        const path = el("path", {
          d: "M" + ax1 + " " + ay1 + " Q " + mx + " " + my + " " + ax2 + " " + ay2,
          fill: "none", stroke: "#9aa0a6", "stroke-width": 1.5, "marker-end": "url(#smartart-arrow)",
        });
        svg.appendChild(path);
      }
      const defs = el("defs");
      const marker = el("marker", { id: "smartart-arrow", viewBox: "0 0 10 10", refX: 9, refY: 5, markerWidth: 6, markerHeight: 6, orient: "auto" });
      marker.appendChild(el("path", { d: "M0 0 L10 5 L0 10 z", fill: "#9aa0a6" }));
      defs.appendChild(marker);
      svg.insertBefore(defs, svg.firstChild);
    } else if (type === "process") {
      const y = 200;
      const totalW = 700;
      const stepW = totalW / items.length;
      for (let i = 0; i < items.length; i++) {
        const x = 50 + stepW * i;
        svg.appendChild(el("rect", { x: x, y: y, width: stepW - 20, height: 70, fill: items[i].color, rx: 6 }));
        const t = el("text", { x: x + (stepW - 20) / 2, y: y + 40, "text-anchor": "middle", fill: "#fff", "font-size": 15, "font-weight": 600 }, items[i].text);
        svg.appendChild(t);
        if (i < items.length - 1) {
          svg.appendChild(el("line", { x1: x + stepW - 20, y1: y + 35, x2: x + stepW, y2: y + 35, stroke: "#5f6368", "stroke-width": 2, "marker-end": "url(#smartart-arrow)" }));
        }
      }
      const defs = el("defs");
      const marker = el("marker", { id: "smartart-arrow", viewBox: "0 0 10 10", refX: 9, refY: 5, markerWidth: 6, markerHeight: 6, orient: "auto" });
      marker.appendChild(el("path", { d: "M0 0 L10 5 L0 10 z", fill: "#5f6368" }));
      defs.appendChild(marker);
      svg.insertBefore(defs, svg.firstChild);
    } else if (type === "radial") {
      const cx = 400, cy = 220;
      svg.appendChild(el("circle", { cx: cx, cy: cy, r: 50, fill: "#202124" }));
      svg.appendChild(el("text", { x: cx, y: cy + 5, "text-anchor": "middle", fill: "#fff", "font-size": 14, "font-weight": 700 }, "Central"));
      const r = 150;
      for (let i = 0; i < items.length; i++) {
        const a = (i / items.length) * Math.PI * 2 - Math.PI / 2;
        const x = cx + r * Math.cos(a);
        const y = cy + r * Math.sin(a);
        svg.appendChild(el("line", { x1: cx, y1: cy, x2: x, y2: y, stroke: items[i].color, "stroke-width": 2 }));
        svg.appendChild(el("circle", { cx: x, cy: y, r: 38, fill: items[i].color }));
        const t = el("text", { x: x, y: y + 4, "text-anchor": "middle", fill: "#fff", "font-size": 11, "font-weight": 600 }, items[i].text);
        svg.appendChild(t);
      }
    } else if (type === "matrix") {
      const cols = 2;
      const rows = Math.ceil(items.length / cols);
      const w = 320, h = 130;
      for (let i = 0; i < items.length; i++) {
        const c = i % cols;
        const r = Math.floor(i / cols);
        const x = 60 + c * (w + 40);
        const y = 50 + r * (h + 30);
        svg.appendChild(el("rect", { x: x, y: y, width: w, height: h, fill: items[i].color, rx: 8 }));
        const t = el("text", { x: x + w / 2, y: y + h / 2 + 6, "text-anchor": "middle", fill: "#fff", "font-size": 18, "font-weight": 600 }, items[i].text);
        svg.appendChild(t);
      }
    } else if (type === "pyramid") {
      const cx = 400;
      const stepH = 60;
      for (let i = items.length - 1; i >= 0; i--) {
        const w = 80 + (items.length - 1 - i) * 60;
        const y = 50 + (items.length - 1 - i) * stepH;
        const path = "M" + (cx - w) + " " + y + " L" + (cx + w) + " " + y + " L" + (cx + w - 30) + " " + (y + stepH - 6) + " L" + (cx - w + 30) + " " + (y + stepH - 6) + " z";
        svg.appendChild(el("path", { d: path, fill: items[i].color, stroke: "#fff", "stroke-width": 1.5 }));
        const t = el("text", { x: cx, y: y + stepH / 2 + 5, "text-anchor": "middle", fill: "#fff", "font-size": 13, "font-weight": 600 }, items[i].text);
        svg.appendChild(t);
      }
    } else if (type === "venn") {
      const cx = 400, cy = 220, r = 100;
      const cols = [items[0] ? items[0].color : PALETTE[0], items[1] ? items[1].color : PALETTE[1], items[2] ? items[2].color : PALETTE[2]];
      const dx = 60;
      svg.appendChild(el("circle", { cx: cx - dx, cy: cy, r: r, fill: cols[0], opacity: 0.45 }));
      svg.appendChild(el("circle", { cx: cx + dx, cy: cy, r: r, fill: cols[1], opacity: 0.45 }));
      svg.appendChild(el("circle", { cx: cx, cy: cy + 60, r: r, fill: cols[2], opacity: 0.45 }));
      for (let i = 0; i < 3; i++) {
        if (items[i]) svg.appendChild(el("text", { x: i === 0 ? cx - dx - 60 : i === 1 ? cx + dx + 60 : cx, y: i === 2 ? cy + 200 : cy - 110, "text-anchor": i === 2 ? "middle" : (i === 0 ? "end" : "start"), fill: cols[i], "font-size": 14, "font-weight": 600 }, items[i].text));
      }
    } else if (type === "funnel") {
      const stages = items.length;
      const topW = 600, botW = 100;
      for (let i = 0; i < stages; i++) {
        const t = i / stages;
        const w1 = topW + (botW - topW) * t;
        const w2 = topW + (botW - topW) * (t + 1 / stages);
        const y1 = 50 + i * 60;
        const y2 = y1 + 60;
        const x = 400;
        const path = "M" + (x - w1 / 2) + " " + y1 + " L" + (x + w1 / 2) + " " + y1 + " L" + (x + w2 / 2) + " " + y2 + " L" + (x - w2 / 2) + " " + y2 + " z";
        svg.appendChild(el("path", { d: path, fill: items[i].color, stroke: "#fff", "stroke-width": 1.5 }));
        svg.appendChild(el("text", { x: x, y: y1 + 35, "text-anchor": "middle", fill: "#fff", "font-size": 13, "font-weight": 600 }, items[i].text));
      }
    }
    return svg;
  }

  function create(type, nodes) {
    if (!TYPES.includes(type)) type = "process";
    return { type, nodes: nodes || defaultNodes(type), id: "smart-" + Date.now() };
  }

  function render(diagram, container) {
    const c = container || document.querySelector(".slide-canvas, .slides-canvas, #slideCanvas");
    if (!c) return null;
    const wrap = document.createElement("div");
    wrap.className = "slide-element smartart";
    wrap.dataset.smartId = diagram.id;
    wrap.style.cssText = "position:absolute;left:5%;top:5%;width:90%;height:90%;";
    wrap.appendChild(build(diagram.type, diagram.nodes));
    c.appendChild(wrap);
    return wrap;
  }

  function editNode(diagram, nodeId, updates) {
    const idx = diagram.nodes.findIndex((n) => n.id === nodeId);
    if (idx < 0) return null;
    for (const k in updates) diagram.nodes[idx][k] = updates[k];
    return diagram.nodes[idx];
  }

  function exportSVG(diagram) {
    const s = build(diagram.type, diagram.nodes);
    return new XMLSerializer().serializeToString(s);
  }

  function getTypes() { return TYPES.slice(); }
  function getPalette() { return PALETTE.slice(); }

  function attach() {
    const btn = document.querySelector("[data-toolbar-action='smartart']");
    if (btn) {
      btn.addEventListener("click", function (e) {
        e.preventDefault();
        const type = window.prompt("Tipo (" + TYPES.join(", ") + "):", "process") || "process";
        render(create(type));
      });
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.SlidesSmartArt = { create, render, getTypes, editNode, exportSVG, getDiagram: create, getPalette, PALETTE };
})();
