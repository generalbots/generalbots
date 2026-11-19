"use strict";
/* Advanced charting — heatmap, treemap and sunburst (#2490). */

(function () {
  const registry = window.SheetChartingTypes.renderers;
  const S = window.SheetChartingSvg;
  const palette = window.SheetChartingPalette;

  function valueAt(series, index) {
    const value = Number(series && series.data ? series.data[index] : 0);
    return Number.isFinite(value) ? value : 0;
  }

  function heatmap(ctx) {
    const parts = [];
    const rows = ctx.model.labels.length;
    const cols = ctx.model.series.length;
    if (!rows || !cols) return parts;
    const left = 58;
    const top = 26;
    const cellW = Math.max(4, (ctx.w - left - 14) / cols);
    const cellH = Math.max(4, (ctx.h - top - 26) / rows);
    const low = ctx.model.min;
    const high = ctx.model.max;
    ctx.model.series.forEach(function (series, col) {
      parts.push(S.text(left + col * cellW + cellW / 2, top - 6,
        series.label || ctx.t("sheet-charting-value", "Value"), { anchor: "middle", size: 8 }));
      for (let row = 0; row < rows; row++) {
        const value = valueAt(series, row);
        const fill = palette.scaleColor(value, low, high);
        const x = left + col * cellW;
        const y = top + row * cellH;
        parts.push(S.rect(x, y, cellW - 1, cellH - 1, { fill: fill, stroke: palette.outline(fill, "#0F172A"), rx: 1 }));
        if (cellW > 34 && cellH > 14) {
          parts.push(S.text(x + cellW / 2, y + cellH / 2 + 3, S.fmt(value), {
            anchor: "middle", size: 8, fill: palette.pickText(fill),
          }));
        }
      }
    });
    ctx.model.labels.forEach(function (label, row) {
      parts.push(S.text(left - 5, top + row * cellH + cellH / 2 + 3, label, { anchor: "end", size: 8 }));
    });
    const legendY = ctx.h - 7;
    parts.push(S.rect(left, legendY - 7, 14, 8, { fill: palette.scaleColor(low, low, high) }));
    parts.push(S.text(left + 18, legendY, S.fmt(low), { size: 8 }));
    parts.push(S.rect(left + 64, legendY - 7, 14, 8, { fill: palette.scaleColor(high, low, high) }));
    parts.push(S.text(left + 82, legendY, S.fmt(high), { size: 8 }));
    return parts;
  }

  function treemap(ctx) {
    const parts = [];
    const weights = ctx.model.labels.map(function (label, index) {
      return ctx.model.series.reduce(function (sum, series) { return sum + Math.max(0, valueAt(series, index)); }, 0);
    });
    const total = weights.reduce(function (sum, value) { return sum + value; }, 0);
    if (!total) return parts;
    const x0 = ctx.geo.pad.l;
    const y0 = ctx.geo.pad.t;
    const width = ctx.geo.plotW;
    const height = ctx.geo.plotH;
    const maxDepth = Math.max(1, Math.min(2, Number(ctx.def.options && ctx.def.options.maxDepth) || 2));
    let x = x0;
    weights.forEach(function (weight, index) {
      const columnW = (weight / total) * width;
      const fill = ctx.colors[index % ctx.colors.length];
      const children = ctx.model.series.map(function (series) {
        return { label: series.label, value: Math.max(0, valueAt(series, index)) };
      });
      const childTotal = children.reduce(function (sum, child) { return sum + child.value; }, 0) || 1;
      if (maxDepth === 1 || children.length === 1) {
        parts.push(S.rect(x, y0, columnW - 1, height - 1, { fill: fill, stroke: palette.outline(fill), rx: 1 }));
        if (columnW > 28) parts.push(S.text(x + 4, y0 + 12, ctx.model.labels[index], { size: 8, fill: palette.pickText(fill) }));
      } else {
        let y = y0;
        children.forEach(function (child, childIndex) {
          const childH = (child.value / childTotal) * height;
          const childFill = palette.tint(fill, 0.18 + childIndex * 0.16);
          parts.push(S.rect(x, y, columnW - 1, childH - 1, { fill: childFill, stroke: palette.outline(childFill), rx: 1 }));
          if (columnW > 34 && childH > 15) {
            parts.push(S.text(x + 4, y + 11, child.label || ctx.t("sheet-charting-value", "Value"), {
              size: 8, fill: palette.pickText(childFill),
            }));
            parts.push(S.text(x + 4, y + 23, S.fmt(child.value), { size: 8, fill: palette.pickText(childFill) }));
          }
          y += childH;
        });
        if (columnW > 28) parts.push(S.text(x + 4, y0 + height - 5, ctx.model.labels[index], { size: 8, fill: palette.pickText(fill) }));
      }
      x += columnW;
    });
    return parts;
  }

  function arcPath(cx, cy, outer, inner, from, to) {
    const point = function (radius, angle) {
      return S.n(cx + radius * Math.cos(angle)) + "," + S.n(cy + radius * Math.sin(angle));
    };
    const large = to - from > Math.PI ? 1 : 0;
    return "M" + point(outer, from) + " A" + S.n(outer) + "," + S.n(outer) + " 0 " + large + " 1 " + point(outer, to) +
      " L" + point(inner, to) + " A" + S.n(inner) + "," + S.n(inner) + " 0 " + large + " 0 " + point(inner, from) + " Z";
  }

  function sunburst(ctx) {
    const parts = [];
    const totals = ctx.model.labels.map(function (label, index) {
      return ctx.model.series.reduce(function (sum, series) { return sum + Math.max(0, valueAt(series, index)); }, 0);
    });
    const total = totals.reduce(function (sum, value) { return sum + value; }, 0);
    if (!total) return parts;
    const cx = ctx.w / 2;
    const cy = ctx.h / 2 + 6;
    const outer = Math.max(20, Math.min(ctx.w, ctx.h) / 2 - 24);
    const mid = outer * 0.62;
    const inner = outer * 0.3;
    let angle = -Math.PI / 2;
    totals.forEach(function (value, index) {
      const span = (value / total) * Math.PI * 2;
      const end = angle + span;
      const fill = ctx.colors[index % ctx.colors.length];
      parts.push(S.path(arcPath(cx, cy, mid, inner, angle + 0.008, end - 0.008), {
        fill: fill, stroke: palette.outline(fill), width: 0.6,
      }));
      if (span > 0.32) {
        const centre = (angle + end) / 2;
        parts.push(S.text(cx + ((inner + mid) / 2) * Math.cos(centre), cy + ((inner + mid) / 2) * Math.sin(centre) + 3,
          ctx.model.labels[index], { anchor: "middle", size: 8, fill: palette.pickText(fill) }));
      }
      let childAngle = angle;
      ctx.model.series.forEach(function (series, childIndex) {
        const childValue = Math.max(0, valueAt(series, index));
        if (!childValue) return;
        const childSpan = (childValue / value) * span;
        const childFill = palette.tint(fill, 0.16 + childIndex * 0.17);
        parts.push(S.path(arcPath(cx, cy, outer, mid + 1, childAngle + 0.008, childAngle + childSpan - 0.008), {
          fill: childFill, stroke: palette.outline(childFill), width: 0.6,
        }));
        childAngle += childSpan;
      });
      angle = end;
    });
    return parts;
  }

  registry.heatmap = heatmap;
  registry.treemap = treemap;
  registry.sunburst = sunburst;
})();
