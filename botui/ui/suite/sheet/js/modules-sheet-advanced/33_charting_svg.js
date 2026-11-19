"use strict";
/* Advanced charting — shared SVG primitives (#2490). All strings are numbers,
 * escaped translations or colours; no user-facing literal is authored here. */

(function () {
  function esc(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;").replace(/'/g, "&apos;");
  }

  function n(value) {
    const numeric = Number(value);
    return Number.isFinite(numeric) ? Math.round(numeric * 100) / 100 : 0;
  }

  function fmt(value, format) {
    const numeric = n(value);
    if (typeof window.gbFormatNumber === "function") return esc(window.gbFormatNumber(numeric, { maximumFractionDigits: 2 }));
    return esc(numeric.toLocaleString(format || "en-US", { maximumFractionDigits: 2 }));
  }

  function open(width, height, alt) {
    return '<svg xmlns="http://www.w3.org/2000/svg" width="' + n(width) + '" height="' + n(height) +
      '" viewBox="0 0 ' + n(width) + " " + n(height) + '" role="img" aria-label="' + esc(alt) + '">';
  }

  function surface(width, height) {
    return '<rect x="0" y="0" width="' + n(width) + '" height="' + n(height) +
      '" fill="var(--sheet-bg, #0f172a)" rx="5"/>';
  }

  function text(x, y, value, options) {
    const opts = options || {};
    return '<text x="' + n(x) + '" y="' + n(y) + '" fill="' + esc(opts.fill || "var(--sheet-muted, #94a3b8)") +
      '" font-size="' + n(opts.size || 9) + '" text-anchor="' + esc(opts.anchor || "start") +
      '" font-family="sans-serif"' + (opts.weight ? ' font-weight="' + esc(opts.weight) + '"' : "") +
      '>' + esc(value) + "</text>";
  }

  function line(x1, y1, x2, y2, options) {
    const opts = options || {};
    return '<line x1="' + n(x1) + '" y1="' + n(y1) + '" x2="' + n(x2) + '" y2="' + n(y2) +
      '" stroke="' + esc(opts.stroke || "var(--sheet-border, #334155)") + '" stroke-width="' + n(opts.width || 0.5) +
      '"' + (opts.dash ? ' stroke-dasharray="' + esc(opts.dash) + '"' : "") + "/>";
  }

  function rect(x, y, width, height, options) {
    const opts = options || {};
    return '<rect x="' + n(x) + '" y="' + n(y) + '" width="' + n(Math.max(0, width)) +
      '" height="' + n(Math.max(0, height)) + '" fill="' + esc(opts.fill || "#3b82f6") +
      '"' + (opts.stroke && opts.stroke !== "none" ? ' stroke="' + esc(opts.stroke) + '" stroke-width="1"' : "") +
      (opts.opacity != null ? ' opacity="' + n(opts.opacity) + '"' : "") +
      (opts.rx ? ' rx="' + n(opts.rx) + '"' : "") + "/>";
  }

  function circle(x, y, radius, options) {
    const opts = options || {};
    return '<circle cx="' + n(x) + '" cy="' + n(y) + '" r="' + n(radius) + '" fill="' + esc(opts.fill || "#3b82f6") +
      '"' + (opts.stroke ? ' stroke="' + esc(opts.stroke) + '" stroke-width="' + n(opts.width || 1) + '"' : "") + "/>";
  }

  function polyline(points, options) {
    const opts = options || {};
    return '<polyline points="' + esc(points.join(" ")) + '" fill="none" stroke="' + esc(opts.stroke || "#3b82f6") +
      '" stroke-width="' + n(opts.width || 2) + '"' + (opts.dash ? ' stroke-dasharray="' + esc(opts.dash) + '"' : "") + "/>";
  }

  function path(data, options) {
    const opts = options || {};
    return '<path d="' + esc(data) + '" fill="' + esc(opts.fill || "none") + '"' +
      (opts.stroke ? ' stroke="' + esc(opts.stroke) + '" stroke-width="' + n(opts.width || 0.6) + '"' : "") + "/>";
  }

  function frame(width, height, title, alt, parts) {
    const header = title ? [text(width / 2, 14, title, { anchor: "middle", size: 11, fill: "#E2E8F0" })] : [];
    return open(width, height, alt) + surface(width, height) + header.join("") + parts.join("") + "</svg>";
  }

  function geometry(width, height, compact) {
    const pad = compact
      ? { l: 36, r: 12, t: 24, b: 26 }
      : { l: 44, r: 14, t: 26, b: 32 };
    const plotW = Math.max(10, width - pad.l - pad.r);
    const plotH = Math.max(10, height - pad.t - pad.b);
    return {
      pad: pad,
      plotW: plotW,
      plotH: plotH,
      x: function (index, count) {
        return pad.l + (count <= 1 ? plotW / 2 : (index / (count - 1)) * plotW);
      },
      bandX: function (index, count) {
        const band = plotW / Math.max(1, count);
        return pad.l + index * band + band / 2;
      },
      bandWidth: function (count) { return plotW / Math.max(1, count); },
    };
  }

  function verticalScale(low, high, geo, ticks) {
    const span = high - low || 1;
    const count = ticks || 4;
    const y = function (value) {
      return geo.pad.t + geo.plotH - ((Number(value) - low) / span) * geo.plotH;
    };
    return { y: y, span: span, values: function () {
      const result = [];
      for (let i = 0; i <= count; i++) result.push(low + (span / count) * i);
      return result;
    } };
  }

  function axes(parts, geo, scale, labels, format) {
    scale.values().forEach(function (value) {
      const y = scale.y(value);
      parts.push(line(geo.pad.l, y, geo.pad.l + geo.plotW, y, { width: 0.5 }));
      parts.push(text(geo.pad.l - 4, y + 3, format ? format(value) : value, { anchor: "end" }));
    });
    parts.push(line(geo.pad.l, geo.pad.t, geo.pad.l, geo.pad.t + geo.plotH, { width: 0.8 }));
    labels.forEach(function (label, index) {
      const x = geo.x(index, labels.length);
      parts.push(text(x, geo.pad.t + geo.plotH + 12, label, { anchor: "middle", size: 8 }));
    });
  }

  function legend(parts, geo, width, entries) {
    let x = geo.pad.l;
    const y = heightMinus(geo);
    entries.slice(0, 4).forEach(function (entry) {
      if (x > width - 70) return;
      parts.push(rect(x, y - 7, 8, 8, { fill: entry.color }));
      parts.push(text(x + 11, y, entry.label, { size: 8 }));
      x += 16 + String(entry.label).length * 5;
    });
  }

  function heightMinus(geo) {
    return geo.pad.t + geo.plotH + 22;
  }

  function titleOf(definition, t) {
    return definition && definition.title ? definition.title : "";
  }

  window.SheetChartingSvg = {
    esc: esc,
    n: n,
    fmt: fmt,
    open: open,
    surface: surface,
    text: text,
    line: line,
    rect: rect,
    circle: circle,
    polyline: polyline,
    path: path,
    frame: frame,
    geometry: geometry,
    verticalScale: verticalScale,
    axes: axes,
    legend: legend,
    titleOf: titleOf,
  };
})();
