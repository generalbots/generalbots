"use strict";
/* Slides chart styling (#2486) — layer 2: theme-token colours, per-series
 * overrides, colour-blind-safe patterns and a WCAG contrast check. Pure and
 * DOM-free; the renderer consumes the descriptors produced here. */
(function (window) {
  var C = (window.GBSlidesCharts = window.GBSlidesCharts || {});
  var S = (C.style = C.style || {});
  var FALLBACK = ["#3b82f6", "#8b5cf6", "#10b981", "#f59e0b", "#ef4444", "#06b6d4"];
  S.PATTERNS = ["solid", "diagonal", "dots", "grid", "cross", "horizontal"];
  S.BACKGROUND = "#ffffff";

  function tokens(theme) {
    if (!theme || !theme.flat) return {};
    return theme.flat;
  }
  S.tokens = tokens;

  S.palette = function (theme, seen) {
    var flat = tokens(theme);
    var colors = [1, 2, 3, 4, 5, 6].map(function (n) {
      return flat["color.accent" + n];
    }).filter(function (color) { return typeof color === "string" && color.length > 0; });
    if (!flat["color.accent1"] && typeof window.GBSlidesMasters !== "undefined" &&
      window.GBSlidesMasters.state && window.GBSlidesMasters.state.themes) {
      var master = window.GBSlidesMasters.state.themes[window.GBSlidesMasters.state.master.defaultThemeId];
      if (master && (seen || []).indexOf(master) < 0) colors = S.palette(master, (seen || []).concat([theme, master]));
    }
    return colors.length ? colors : FALLBACK.slice();
  };

  S.font = function (theme, role) {
    var flat = tokens(theme);
    return flat["font." + (role === "title" ? "major" : "minor")] || "Arial, Helvetica, sans-serif";
  };

  S.seriesColor = function (chart, index, theme) {
    var series = chart.series[index];
    if (series && series.color) return series.color;
    var colors = S.palette(theme);
    return colors[index % colors.length];
  };

  S.seriesPattern = function (chart, index) {
    return chart.a11y.colorBlindSafe ? S.PATTERNS[index % S.PATTERNS.length] : "solid";
  };

  S.fillDescriptor = function (chart, index, theme) {
    var color = S.seriesColor(chart, index, theme);
    var pattern = S.seriesPattern(chart, index);
    if (pattern === "solid") return { fill: color, pattern: pattern };
    return { fill: color, pattern: pattern, patternId: "gbch-pat-" + pattern };
  };

  function parseColor(value) {
    var text = String(value || "").trim();
    var hex = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(text);
    if (hex) {
      var h = hex[1];
      if (h.length === 3) h = h[0] + h[0] + h[1] + h[1] + h[2] + h[2];
      return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)];
    }
    var rgb = /rgba?\(\s*(\d+)[,\s]+(\d+)[,\s]+(\d+)/i.exec(text);
    return rgb ? [parseInt(rgb[1], 10), parseInt(rgb[2], 10), parseInt(rgb[3], 10)] : null;
  }
  S.parseColor = parseColor;

  function luminance(rgb) {
    var channels = rgb.map(function (channel) {
      var value = channel / 255;
      return value <= 0.03928 ? value / 12.92 : Math.pow((value + 0.055) / 1.055, 2.4);
    });
    return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
  }
  S.luminance = luminance;

  S.contrast = function (foreground, background) {
    var a = luminance(parseColor(foreground) || [0, 0, 0]);
    var b = luminance(parseColor(background) || [255, 255, 255]);
    return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
  };

  S.contrastWarnings = function (chart, theme, background) {
    if (!chart || !chart.a11y.contrastCheck) return [];
    var bg = background || S.BACKGROUND;
    return chart.series.map(function (series, index) {
      var color = S.seriesColor(chart, index, theme);
      var ratio = S.contrast(color, bg);
      return { seriesIndex: index, name: series.name, color: color, ratio: ratio, low: ratio < 3 };
    }).filter(function (item) { return item.low; });
  };

  S.formatNumber = function (value, format) {
    var n = C.num(value, 0);
    if (format === "0%") return Math.round(n * 100) + "%";
    if (format === "currency") return "$" + n.toFixed(2);
    if (format === "0.0") return n.toFixed(1);
    var rounded = Math.round(n);
    var text = String(Math.abs(rounded));
    var parts = [];
    while (text.length > 3) {
      parts.unshift(text.slice(-3));
      text = text.slice(0, -3);
    }
    parts.unshift(text);
    return (rounded < 0 ? "-" : "") + parts.join(",");
  };

  /* Defensive descriptor: a <pattern> definition used by materialize() when
   * the colour-blind-safe option is on. Geometry builders only reference it
   * by id; the wrapper emits the definition once per chart. */
  S.patternDefs = function (chart, theme) {
    if (!chart.a11y.colorBlindSafe) return [];
    return S.PATTERNS.map(function (name) {
      return { tag: "pattern", attrs: {
        id: "gbch-pat-" + name, width: 8, height: 8,
        patternUnits: "userSpaceOnUse"
      }, children: [{ tag: "rect", attrs: { width: 8, height: 8, fill: "none" }, children: [] },
        patternMark(name, S.seriesColor(chart, S.PATTERNS.indexOf(name), theme))] };
    });
  };

  function patternMark(name, color) {
    var stroke = { stroke: color, "stroke-width": 2 };
    if (name === "diagonal") return { tag: "path", attrs: Object.assign({ d: "M0 8 L8 0" }, stroke), children: [] };
    if (name === "horizontal") return { tag: "path", attrs: Object.assign({ d: "M0 4 H8" }, stroke), children: [] };
    if (name === "cross") return { tag: "path", attrs: Object.assign({ d: "M0 4 H8 M4 0 V8" }, stroke), children: [] };
    if (name === "grid") return { tag: "rect", attrs: Object.assign({ x: 1, y: 1, width: 6, height: 6, fill: "none" }, stroke), children: [] };
    return { tag: "circle", attrs: { cx: 4, cy: 4, r: 1.6, fill: color }, children: [] };
  }
})(typeof window !== "undefined" ? window : globalThis);
