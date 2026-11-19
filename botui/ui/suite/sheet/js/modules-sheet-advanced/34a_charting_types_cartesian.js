"use strict";
/* Advanced charting — waterfall, histogram, Pareto and box plots (#2490). */

(function () {
  const registry = (window.SheetChartingTypes && window.SheetChartingTypes.renderers)
    ? window.SheetChartingTypes.renderers
    : {};
  const S = window.SheetChartingSvg;
  const P = window.SheetChartingPalette;

  function ctxScale(ctx, low, high) {
    ctx.scale = S.verticalScale(low, high, ctx.geo, 4);
    return ctx.scale;
  }

  function labelsFor(ctx, count) {
    const labels = ctx.model.labels || [];
    const result = [];
    for (let i = 0; i < count; i++) result.push(labels[i] != null ? labels[i] : String(i + 1));
    return result;
  }

  function valueAt(series, index) {
    if (!series || !series.data) return 0;
    const value = Number(series.data[index]);
    return Number.isFinite(value) ? value : 0;
  }

  function waterfall(ctx) {
    const parts = [];
    const series = ctx.model.series[0];
    const deltas = [];
    if (series) series.data.forEach(function (value) {
      if (value != null) deltas.push(Number(value) || 0);
    });
    if (!deltas.length) return parts;
    const steps = [];
    let running = 0;
    deltas.forEach(function (delta) {
      const from = running;
      running += delta;
      steps.push({ from: from, to: running, delta: delta });
    });
    const totalLabel = ctx.t("sheet-charting-total", "Total");
    const labels = labelsFor(ctx, deltas.length).concat([totalLabel]);
    const lows = steps.map(function (step) { return Math.min(step.from, step.to); }).concat([0, running]);
    const highs = steps.map(function (step) { return Math.max(step.from, step.to); }).concat([running]);
    const low = Math.min.apply(null, lows);
    const high = Math.max.apply(null, highs);
    const paddedLow = low < 0 ? low * 1.1 : 0;
    const paddedHigh = high > 0 ? high * 1.1 : 1;
    const scale = ctxScale(ctx, paddedLow, paddedHigh);
    S.axes(parts, ctx.geo, scale, labels, function (value) { return S.fmt(value); });
    const band = ctx.geo.bandWidth(labels.length);
    const geo = ctx.geo;
    steps.forEach(function (step, index) {
      const centre = geo.bandX(index, labels.length);
      const x = centre - band * 0.3;
      const yFrom = scale.y(step.from);
      const yTo = scale.y(step.to);
      const fill = step.delta >= 0 ? "#009E73" : "#D55E00";
      parts.push(S.rect(x, Math.min(yFrom, yTo), band * 0.6, Math.max(1, Math.abs(yFrom - yTo)), {
        fill: fill, stroke: P.outline(fill), rx: 1,
      }));
      parts.push(S.text(centre, Math.min(yFrom, yTo) - 3, (step.delta >= 0 ? "+" : "") + S.fmt(step.delta), {
        anchor: "middle", size: 8, fill: fill,
      }));
      if (index < steps.length - 1) {
        parts.push(S.line(centre + band * 0.3, yTo, geo.bandX(index + 1, labels.length) - band * 0.3, yTo, {
          stroke: "var(--sheet-muted, #94a3b8)", width: 0.8, dash: "3,2",
        }));
      }
    });
    const totalX = geo.bandX(steps.length, labels.length) - band * 0.3;
    const totalY = scale.y(running);
    parts.push(S.rect(totalX, Math.min(totalY, scale.y(0)), band * 0.6, Math.max(1, Math.abs(totalY - scale.y(0))), {
      fill: "#0072B2", stroke: P.outline("#0072B2"), rx: 1,
    }));
    parts.push(S.text(totalX + band * 0.3, Math.min(totalY, scale.y(0)) - 3, S.fmt(running), {
      anchor: "middle", size: 8, fill: "#0072B2",
    }));
    return parts;
  }

  function histogram(ctx) {
    const parts = [];
    const values = ctx.model.values;
    if (!values.length) return parts;
    const bins = Math.max(2, Math.min(30, Math.floor(Number(ctx.def.options && ctx.def.options.bins) || 6)));
    let low = ctx.model.min;
    let high = ctx.model.max;
    if (high === low) { low -= 1; high += 1; }
    const width = (high - low) / bins;
    const counts = new Array(bins).fill(0);
    values.forEach(function (value) {
      let index = Math.floor((value - low) / width);
      if (index < 0) index = 0;
      if (index >= bins) index = bins - 1;
      counts[index] += 1;
    });
    const scale = ctxScale(ctx, 0, Math.max.apply(null, counts) * 1.15 || 1);
    const labels = counts.map(function (_, index) {
      return S.fmt(low + index * width) + "–" + S.fmt(low + (index + 1) * width);
    });
    S.axes(parts, ctx.geo, scale, labels, function (value) { return S.fmt(value); });
    const band = ctx.geo.bandWidth(bins);
    counts.forEach(function (count, index) {
      const x = ctx.geo.bandX(index, bins) - band * 0.35;
      const y = scale.y(count);
      parts.push(S.rect(x, y, band * 0.7, ctx.geo.pad.t + ctx.geo.plotH - y, {
        fill: "#0072B2", stroke: P.outline("#0072B2"), rx: 1,
      }));
      parts.push(S.text(ctx.geo.bandX(index, bins), y - 3, String(count), { anchor: "middle", size: 8 }));
    });
    parts.push(S.text(ctx.geo.pad.l + ctx.geo.plotW, ctx.geo.pad.t + ctx.geo.plotH + 22,
      ctx.t("sheet-charting-axis-count", "Count"), { anchor: "end", size: 8 }));
    return parts;
  }

  function pareto(ctx) {
    const parts = [];
    const rows = [];
    for (let i = 0; i < ctx.model.labels.length; i++) {
      let value = 0;
      ctx.model.series.forEach(function (series) { value += valueAt(series, i); });
      rows.push({ label: ctx.model.labels[i], value: Math.max(0, value) });
    }
    rows.sort(function (a, b) { return b.value - a.value; });
    if (!rows.length) return parts;
    const total = rows.reduce(function (sum, row) { return sum + row.value; }, 0) || 1;
    const high = Math.max.apply(null, rows.map(function (row) { return row.value; })) * 1.15 || 1;
    const left = ctxScale(ctx, 0, high);
    const right = S.verticalScale(0, 100, ctx.geo, 4);
    S.axes(parts, ctx.geo, left, rows.map(function (row) { return row.label; }), function (value) { return S.fmt(value); });
    right.values().forEach(function (value) {
      parts.push(S.text(ctx.geo.pad.l + ctx.geo.plotW + 3, right.y(value), S.fmt(value) + "%", { size: 8 }));
    });
    const band = ctx.geo.bandWidth(rows.length);
    let cumulative = 0;
    const points = [];
    rows.forEach(function (row, index) {
      const x = ctx.geo.bandX(index, rows.length) - band * 0.32;
      const y = left.y(row.value);
      parts.push(S.rect(x, y, band * 0.64, ctx.geo.pad.t + ctx.geo.plotH - y, {
        fill: ctx.colors[index % ctx.colors.length], stroke: P.outline(ctx.colors[index % ctx.colors.length]), rx: 1,
      }));
      cumulative += (row.value / total) * 100;
      points.push(ctx.geo.bandX(index, rows.length) + "," + right.y(cumulative));
      parts.push(S.circle(ctx.geo.bandX(index, rows.length), right.y(cumulative), 2.5, { fill: "#D55E00" }));
    });
    parts.push(S.polyline(points, { stroke: "#D55E00", width: 2 }));
    parts.push(S.rect(ctx.geo.pad.l, ctx.geo.pad.t + ctx.geo.plotH + 20, 8, 8, { fill: "#0072B2" }));
    parts.push(S.text(ctx.geo.pad.l + 11, ctx.geo.pad.t + ctx.geo.plotH + 27,
      ctx.t("sheet-charting-value", "Value"), { size: 8 }));
    parts.push(S.rect(ctx.geo.pad.l + 60, ctx.geo.pad.t + ctx.geo.plotH + 20, 8, 8, { fill: "#D55E00" }));
    parts.push(S.text(ctx.geo.pad.l + 71, ctx.geo.pad.t + ctx.geo.plotH + 27,
      ctx.t("sheet-charting-cumulative", "Cumulative %"), { size: 8 }));
    return parts;
  }

  function boxPlot(ctx) {
    const parts = [];
    const data = window.SheetChartingData;
    const boxes = ctx.model.series.length > 1
      ? ctx.model.series
      : [{ label: ctx.model.series[0] && ctx.model.series[0].label || ctx.t("sheet-charting-all", "All"), data: ctx.model.series[0] ? ctx.model.series[0].data : [] }];
    const summaries = boxes.map(function (box) { return data.fiveNumber(box.data); });
    if (!summaries.length) return parts;
    const low = Math.min(0, Math.min.apply(null, summaries.map(function (item) { return item.min; })));
    const high = Math.max.apply(null, summaries.map(function (item) { return item.max; })) * 1.1 || 1;
    const scale = ctxScale(ctx, low, high);
    S.axes(parts, ctx.geo, scale, boxes.map(function (box) {
      return box.label || ctx.t("sheet-charting-all", "All");
    }), function (value) { return S.fmt(value); });
    const band = ctx.geo.bandWidth(boxes.length);
    boxes.forEach(function (box, index) {
      const summary = summaries[index];
      const centre = ctx.geo.bandX(index, boxes.length);
      const half = band * 0.22;
      const fill = ctx.colors[index % ctx.colors.length];
      parts.push(S.rect(centre - half, scale.y(summary.q3), half * 2, Math.max(1, scale.y(summary.q1) - scale.y(summary.q3)), {
        fill: fill, stroke: P.outline(fill), opacity: 0.75, rx: 1,
      }));
      parts.push(S.line(centre - half, scale.y(summary.median), centre + half, scale.y(summary.median), {
        stroke: P.pickText(fill), width: 1.6,
      }));
      parts.push(S.line(centre, scale.y(summary.min), centre, scale.y(summary.q3), { stroke: fill, width: 1.2 }));
      parts.push(S.line(centre - half, scale.y(summary.min), centre + half, scale.y(summary.min), { stroke: fill, width: 1.2 }));
      parts.push(S.line(centre, scale.y(summary.q1), centre, scale.y(summary.max), { stroke: fill, width: 1.2 }));
      parts.push(S.line(centre - half, scale.y(summary.max), centre + half, scale.y(summary.max), { stroke: fill, width: 1.2 }));
      summary.outliers.forEach(function (value) {
        parts.push(S.circle(centre, scale.y(value), 2, { fill: "#D55E00" }));
      });
    });
    return parts;
  }

  function register(name, renderer) {
    registry[name] = renderer;
  }

  register("waterfall", waterfall);
  register("histogram", histogram);
  register("pareto", pareto);
  register("box", boxPlot);

  window.SheetChartingTypes = {
    renderers: registry,
    register: register,
  };
})();
