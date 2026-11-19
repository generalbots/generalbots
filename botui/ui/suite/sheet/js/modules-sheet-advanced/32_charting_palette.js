"use strict";
/* Advanced charting — colour-blind-safe palettes and contrast checks (#2490). */

(function () {
  // Okabe–Ito: eight colours distinguishable under the common CVD types.
  const CVD = [
    "#0072B2", "#E69F00", "#009E73", "#D55E00",
    "#CC79A7", "#56B4E9", "#F0E442", "#000000",
  ];
  // Dark-surface high-contrast set: light marks with one warm accent.
  const CONTRAST = [
    "#56B4E9", "#F0E442", "#FFFFFF", "#E69F00",
    "#CC79A7", "#009E73", "#D55E00", "#94A3B8",
  ];

  function channel(value) {
    const normalised = Math.max(0, Math.min(1, value / 255));
    return normalised <= 0.03928 ? normalised / 12.92 : Math.pow((normalised + 0.055) / 1.055, 2.4);
  }

  function rgb(hex) {
    const raw = String(hex || "").replace("#", "");
    const full = raw.length === 3 ? raw.split("").map(function (ch) { return ch + ch; }).join("") : raw;
    if (!/^[0-9a-fA-F]{6}$/.test(full)) return [59, 130, 246];
    return [
      parseInt(full.slice(0, 2), 16),
      parseInt(full.slice(2, 4), 16),
      parseInt(full.slice(4, 6), 16),
    ];
  }

  function luminance(hex) {
    const parts = rgb(hex);
    return 0.2126 * channel(parts[0]) + 0.7152 * channel(parts[1]) + 0.0722 * channel(parts[2]);
  }

  function contrastRatio(foreground, background) {
    const first = luminance(foreground);
    const second = luminance(background);
    const light = Math.max(first, second);
    const dark = Math.min(first, second);
    return (light + 0.05) / (dark + 0.05);
  }

  function pickText(background) {
    return contrastRatio("#F8FAFC", background) >= contrastRatio("#0B1020", background)
      ? "#F8FAFC" : "#0B1020";
  }

  function palette(name) {
    return name === "contrast" ? CONTRAST : CVD;
  }

  function seriesColors(series, name) {
    const colors = palette(name);
    return series.map(function (item, index) {
      if (item && item.color) return item.color;
      return colors[index % colors.length];
    });
  }

  function tint(hex, amount) {
    const parts = rgb(hex);
    const target = luminance(hex) > 0.4 ? 15 : 245;
    const ratio = Math.max(0, Math.min(1, amount));
    const mixed = parts.map(function (value) {
      return Math.round(value + (target - value) * ratio);
    });
    return "#" + mixed.map(function (value) {
      return Math.max(0, Math.min(255, value)).toString(16).padStart(2, "0");
    }).join("");
  }

  // A mark outline is added when its fill is too close to the chart surface.
  function outline(hex, surface) {
    return contrastRatio(hex, surface || "#0F172A") >= 3 ? "none" : pickText(hex === surface ? "#0F172A" : surface || "#0F172A");
  }

  function scaleColor(value, low, high) {
    const span = high - low;
    const fraction = span > 0 ? Math.max(0, Math.min(1, (value - low) / span)) : 0.5;
    const first = rgb("#1B3A73");
    const last = rgb("#F0E442");
    const mixed = first.map(function (part, index) { return Math.round(part + (last[index] - part) * fraction); });
    return "#" + mixed.map(function (part) { return part.toString(16).padStart(2, "0"); }).join("");
  }

  window.SheetChartingPalette = {
    colors: seriesColors,
    palette: palette,
    contrastRatio: contrastRatio,
    pickText: pickText,
    outline: outline,
    tint: tint,
    scaleColor: scaleColor,
  };
})();
