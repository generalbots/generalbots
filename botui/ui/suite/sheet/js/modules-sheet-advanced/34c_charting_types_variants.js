"use strict";
/* Advanced charting — combo and stacked/clustered bar variants (#2490). */

(function () {
  const registry = window.SheetChartingTypes.renderers;
  const S = window.SheetChartingSvg;
  const P = window.SheetChartingPalette;

  function valueAt(series, index) {
    const value = Number(series && series.data ? series.data[index] : 0);
    return Number.isFinite(value) ? value : 0;
  }

  function combo(ctx) {
    const parts = [];
    const bars = ctx.model.series[0];
    const line = ctx.model.series[1] || ctx.model.series[0];
    if (!bars) return parts;
    const labels = ctx.model.labels;
    const barValues = bars.data.map(function (_, index) { return valueAt(bars, index); });
    const lineValues = line.data.map(function (_, index) { return valueAt(line, index); });
    const barHigh = Math.max.apply(null, barValues.concat([0])) * 1.15 || 1;
    const barLow = Math.min.apply(null, barValues.concat([0]));
    const lineHigh = Math.max.apply(null, lineValues.concat([0])) * 1.15 || 1;
    const lineLow = Math.min.apply(null, lineValues.concat([0]));
    const left = S.verticalScale(barLow < 0 ? barLow * 1.1 : 0, barHigh, ctx.geo, 4);
    const right = S.verticalScale(lineLow < 0 ? lineLow * 1.1 : 0, lineHigh, ctx.geo, 4);
    S.axes(parts, ctx.geo, left, labels, function (value) { return S.fmt(value); });
    right.values().forEach(function (value) {
      parts.push(S.text(ctx.geo.pad.l + ctx.geo.plotW + 3, right.y(value), S.fmt(value), { size: 8 }));
    });
    const band = ctx.geo.bandWidth(labels.length);
    barValues.forEach(function (value, index) {
      const x = ctx.geo.bandX(index, labels.length) - band * 0.3;
      const y = left.y(value);
      const baseline = left.y(0);
      parts.push(S.rect(x, Math.min(y, baseline), band * 0.6, Math.max(1, Math.abs(baseline - y)), {
        fill: ctx.colors[0], stroke: ctx.colors[0] === "none" ? "none" : P.outline(ctx.colors[0]), rx: 1,
      }));
    });
    const points = lineValues.map(function (value, index) {
      return ctx.geo.x(index, labels.length) + "," + right.y(value);
    });
    parts.push(S.polyline(points, { stroke: ctx.colors[1] || "#D55E00", width: 2 }));
    lineValues.forEach(function (value, index) {
      parts.push(S.circle(ctx.geo.x(index, labels.length), right.y(value), 2.5, { fill: ctx.colors[1] || "#D55E00" }));
    });
    parts.push(S.rect(ctx.geo.pad.l, ctx.geo.pad.t + ctx.geo.plotH + 20, 8, 8, { fill: ctx.colors[0] }));
    parts.push(S.text(ctx.geo.pad.l + 11, ctx.geo.pad.t + ctx.geo.plotH + 27,
      ctx.t("sheet-charting-value", "Value"), { size: 8 }));
    parts.push(S.rect(ctx.geo.pad.l + 60, ctx.geo.pad.t + ctx.geo.plotH + 20, 8, 8, { fill: ctx.colors[1] || "#D55E00" }));
    parts.push(S.text(ctx.geo.pad.l + 71, ctx.geo.pad.t + ctx.geo.plotH + 27,
      ctx.t("sheet-charting-secondary-axis", "Secondary axis"), { size: 8 }));
    return parts;
  }

  function barGeometry(ctx, stacked) {
    const values = [];
    ctx.model.series.forEach(function (series) {
      series.data.forEach(function (_, index) { values.push(valueAt(series, index)); });
    });
    let positive = 0;
    let negative = 0;
    if (stacked) {
      for (let index = 0; index < ctx.model.labels.length; index++) {
        let up = 0;
        let down = 0;
        ctx.model.series.forEach(function (series) {
          const value = valueAt(series, index);
          if (value >= 0) up += value; else down += value;
        });
        positive = Math.max(positive, up);
        negative = Math.min(negative, down);
      }
    } else {
      positive = Math.max.apply(null, values.concat([0]));
      negative = Math.min.apply(null, values.concat([0]));
    }
    return {
      low: negative < 0 ? negative * 1.1 : 0,
      high: (positive > 0 ? positive * 1.1 : 0) || 1,
    };
  }

  function verticalVariant(ctx, stacked) {
    const parts = [];
    const extent = barGeometry(ctx, stacked);
    const scale = S.verticalScale(extent.low, extent.high, ctx.geo, 4);
    S.axes(parts, ctx.geo, scale, ctx.model.labels, function (value) { return S.fmt(value); });
    const band = ctx.geo.bandWidth(ctx.model.labels.length);
    const baseline = scale.y(0);
    ctx.model.labels.forEach(function (label, index) {
      const centre = ctx.geo.bandX(index, ctx.model.labels.length);
      if (stacked) {
        let up = 0;
        let down = 0;
        ctx.model.series.forEach(function (series, seriesIndex) {
          const value = valueAt(series, index);
          const from = value >= 0 ? up : down;
          if (value >= 0) up += value; else down += value;
          const y = scale.y(from + value);
          const base = scale.y(from);
          parts.push(S.rect(centre - band * 0.32, Math.min(y, base), band * 0.64, Math.max(1, Math.abs(base - y)), {
            fill: ctx.colors[seriesIndex], stroke: P.outline(ctx.colors[seriesIndex]), rx: 1,
          }));
        });
      } else {
        const width = (band * 0.72) / Math.max(1, ctx.model.series.length);
        ctx.model.series.forEach(function (series, seriesIndex) {
          const value = valueAt(series, index);
          const x = centre - band * 0.36 + seriesIndex * width;
          const y = scale.y(value);
          parts.push(S.rect(x, Math.min(y, baseline), Math.max(1, width - 1), Math.max(1, Math.abs(baseline - y)), {
            fill: ctx.colors[seriesIndex], stroke: P.outline(ctx.colors[seriesIndex]), rx: 1,
          }));
        });
      }
    });
    return parts;
  }

  function horizontalVariant(ctx, stacked) {
    const parts = [];
    const extent = barGeometry(ctx, stacked);
    const geo = ctx.geo;
    const span = extent.high - extent.low || 1;
    const x = function (value) { return geo.pad.l + ((value - extent.low) / span) * geo.plotW; };
    const count = Math.max(1, ctx.model.labels.length);
    const band = geo.plotH / count;
    for (let tick = 0; tick <= 4; tick++) {
      const value = extent.low + (span / 4) * tick;
      parts.push(S.line(x(value), geo.pad.t, x(value), geo.pad.t + geo.plotH, { width: 0.5 }));
      parts.push(S.text(x(value), geo.pad.t + geo.plotH + 14, S.fmt(value), { anchor: "middle", size: 8 }));
    }
    ctx.model.labels.forEach(function (label, index) {
      const centre = geo.pad.t + index * band + band / 2;
      parts.push(S.text(geo.pad.l - 5, centre + 3, label, { anchor: "end", size: 8 }));
      if (stacked) {
        let up = 0;
        let down = 0;
        ctx.model.series.forEach(function (series, seriesIndex) {
          const value = valueAt(series, index);
          const from = value >= 0 ? up : down;
          if (value >= 0) up += value; else down += value;
          const left = Math.min(x(from), x(from + value));
          parts.push(S.rect(left, centre - band * 0.3, Math.max(1, Math.abs(x(from + value) - x(from))), band * 0.6, {
            fill: ctx.colors[seriesIndex], stroke: P.outline(ctx.colors[seriesIndex]), rx: 1,
          }));
        });
      } else {
        const height = (band * 0.72) / Math.max(1, ctx.model.series.length);
        ctx.model.series.forEach(function (series, seriesIndex) {
          const value = valueAt(series, index);
          const left = Math.min(x(0), x(value));
          parts.push(S.rect(left, centre - band * 0.36 + seriesIndex * height,
            Math.max(1, Math.abs(x(value) - x(0))), Math.max(1, height - 1), {
              fill: ctx.colors[seriesIndex], stroke: P.outline(ctx.colors[seriesIndex]), rx: 1,
            }));
        });
      }
    });
    return parts;
  }

  function stackedColumn(ctx) { return verticalVariant(ctx, true); }
  function clusteredColumn(ctx) { return verticalVariant(ctx, false); }
  function stackedBar(ctx) { return horizontalVariant(ctx, true); }
  function clusteredBar(ctx) { return horizontalVariant(ctx, false); }

  registry.combo = combo;
  registry["stacked-column"] = stackedColumn;
  registry["clustered-column"] = clusteredColumn;
  registry["stacked-bar"] = stackedBar;
  registry["clustered-bar"] = clusteredBar;
})();
