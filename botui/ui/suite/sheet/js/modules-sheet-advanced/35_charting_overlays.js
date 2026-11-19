"use strict";
/* Advanced charting — analysis overlays drawn over cartesian charts (#2490). */

(function () {
  const S = window.SheetChartingSvg;
  const data = window.SheetChartingData;

  function setting(ctx, name) {
    const analysis = ctx.def.analysis || {};
    const options = ctx.def.options || {};
    return analysis[name] != null ? analysis[name] : options[name];
  }

  function trendline(ctx, parts, map) {
    const kind = setting(ctx, "trendline") || "none";
    if (!kind || kind === "none") return;
    const series = ctx.model.series[0];
    if (!series || !series.data.length) return;
    const values = series.data;
    let predicted;
    let labelKey;
    if (kind === "exponential") {
      const fit = data.exponential(values);
      predicted = values.map(function (_, index) { return fit.predict(index); });
      labelKey = "sheet-charting-trendline-exponential";
    } else if (kind === "moving" || kind === "moving-average") {
      const period = Number(setting(ctx, "movingPeriod")) || 3;
      predicted = data.movingAverage(values, period);
      labelKey = "sheet-charting-trendline-moving";
    } else {
      const fit = data.regression(values);
      predicted = values.map(function (_, index) { return fit.predict(index); });
      labelKey = "sheet-charting-trendline-linear";
    }
    const points = [];
    predicted.forEach(function (value, index) {
      if (value != null && Number.isFinite(value)) points.push(map.x(index) + "," + map.y(value));
    });
    if (points.length < 2) return;
    parts.push(S.polyline(points, { stroke: "#CC79A7", width: 1.6, dash: "6,3" }));
    parts.push(S.text(map.lastX, map.top + 10, ctx.t(labelKey, kind), { anchor: "end", size: 8, fill: "#CC79A7" }));
  }

  function errorBars(ctx, parts, map) {
    if (!setting(ctx, "errorBars")) return;
    const series = ctx.model.series[0];
    if (!series) return;
    const summary = data.stats(series.data);
    const percent = Math.max(0, Number(setting(ctx, "errorPercent")) || 0);
    series.data.forEach(function (value, index) {
      if (value == null) return;
      const numeric = Number(value);
      const delta = percent > 0 ? Math.abs(numeric) * (percent / 100) : summary.sd;
      if (!Number.isFinite(delta)) return;
      const x = map.x(index);
      const top = map.y(numeric + delta);
      const bottom = map.y(numeric - delta);
      parts.push(S.line(x, top, x, bottom, { stroke: "#CC79A7", width: 1 }));
      parts.push(S.line(x - 3, top, x + 3, top, { stroke: "#CC79A7", width: 1 }));
      parts.push(S.line(x - 3, bottom, x + 3, bottom, { stroke: "#CC79A7", width: 1 }));
    });
  }

  function referenceLines(ctx, parts, map) {
    const entries = setting(ctx, "referenceLines") || [];
    const list = Array.isArray(entries) ? entries : [entries];
    list.forEach(function (entry) {
      const value = Number(entry && entry.value != null ? entry.value : entry);
      if (!Number.isFinite(value)) return;
      const y = map.y(value);
      parts.push(S.line(map.left, y, map.right, y, { stroke: "#D55E00", width: 1.1, dash: "4,3" }));
      const label = entry && entry.label ? entry.label : S.fmt(value);
      parts.push(S.text(map.right - 2, y - 3, label, { anchor: "end", size: 8, fill: "#D55E00" }));
    });
  }

  function targetBand(ctx, parts, map) {
    const band = setting(ctx, "targetBand");
    if (!band) return;
    const low = Number(band.low);
    const high = Number(band.high);
    if (!Number.isFinite(low) || !Number.isFinite(high)) return;
    const top = map.y(Math.max(low, high));
    const bottom = map.y(Math.min(low, high));
    parts.push(S.rect(map.left, top, map.right - map.left, Math.max(1, bottom - top), {
      fill: "#F0E442", opacity: 0.16,
    }));
    parts.push(S.text(map.left + 3, top + 9, ctx.t("sheet-charting-target-band", "Target band"), {
      size: 8, fill: "#F0E442",
    }));
  }

  function variance(ctx, parts, map) {
    if (!setting(ctx, "variance")) return;
    const series = ctx.model.series[0];
    if (!series) return;
    const band = setting(ctx, "targetBand");
    const references = setting(ctx, "referenceLines") || [];
    const first = Array.isArray(references) ? references[0] : references;
    let target = NaN;
    if (band && Number.isFinite(Number(band.low)) && Number.isFinite(Number(band.high))) {
      target = (Number(band.low) + Number(band.high)) / 2;
    } else if (first && first.value != null) {
      target = Number(first.value);
    }
    if (!Number.isFinite(target) || !target) return;
    series.data.forEach(function (value, index) {
      if (value == null) return;
      const numeric = Number(value);
      const delta = ((numeric - target) / target) * 100;
      const sign = delta >= 0 ? "+" : "";
      parts.push(S.text(map.x(index), map.y(numeric) - 6, sign + S.fmt(delta) + "%", {
        anchor: "middle", size: 8, fill: delta >= 0 ? "#009E73" : "#D55E00",
      }));
    });
  }

  function apply(ctx, parts, map) {
    if (!map || !ctx.def) return parts;
    targetBand(ctx, parts, map);
    referenceLines(ctx, parts, map);
    trendline(ctx, parts, map);
    errorBars(ctx, parts, map);
    variance(ctx, parts, map);
    return parts;
  }

  window.SheetChartingOverlays = { apply: apply };
})();
