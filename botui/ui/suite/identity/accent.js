/* Identity accent engine — derives a full token set from one hex colour.
 * Plain script, no modules. Exposes window.GBIdentity. */
(function () {
  "use strict";

  /* ---- colour-space helpers ---- */

  function hexToRgb(hex) {
    hex = hex.replace("#", "");
    return [
      parseInt(hex.slice(0, 2), 16) / 255,
      parseInt(hex.slice(2, 4), 16) / 255,
      parseInt(hex.slice(4, 6), 16) / 255
    ];
  }

  function rgbToHex(r, g, b) {
    function c(v) {
      var n = Math.round(Math.max(0, Math.min(1, v)) * 255);
      return (n < 16 ? "0" : "") + n.toString(16);
    }
    return "#" + c(r) + c(g) + c(b);
  }

  function srgbToLin(v) {
    return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
  }

  function linToSrgb(v) {
    return v <= 0.0031308 ? 12.92 * v : 1.055 * Math.pow(v, 1 / 2.4) - 0.055;
  }

  function rgbToOklch(r, g, b) {
    var lr = srgbToLin(r), lg = srgbToLin(g), lb = srgbToLin(b);
    var l = 0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb;
    var m = 0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb;
    var s = 0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb;
    var l_ = Math.cbrt(l), m_ = Math.cbrt(m), s_ = Math.cbrt(s);
    var L = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
    var a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
    var bb = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;
    var C = Math.sqrt(a * a + bb * bb);
    var H = Math.atan2(bb, a) * 180 / Math.PI;
    if (H < 0) { H += 360; }
    return { L: L, C: C, H: H };
  }

  function oklchToRgb(L, C, H) {
    var hRad = H * Math.PI / 180;
    var a = C * Math.cos(hRad), b = C * Math.sin(hRad);
    var l_ = L + 0.3963377774 * a + 0.2158037573 * b;
    var m_ = L - 0.1055613458 * a - 0.0638541728 * b;
    var s_ = L - 0.0894841775 * a - 1.2914855480 * b;
    var l = l_ * l_ * l_, m = m_ * m_ * m_, s = s_ * s_ * s_;
    var r = +4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    var g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    var bb2 = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;
    return [linToSrgb(r), linToSrgb(g), linToSrgb(bb2)];
  }

  function oklchToHex(L, C, H) {
    var rgb = oklchToRgb(L, C, H);
    return rgbToHex(rgb[0], rgb[1], rgb[2]);
  }

  /* ---- WCAG 2 contrast ---- */

  function relLum(r, g, b) {
    return 0.2126 * srgbToLin(r) + 0.7152 * srgbToLin(g) + 0.0722 * srgbToLin(b);
  }

  function relLumHex(hex) {
    var c = hexToRgb(hex);
    return relLum(c[0], c[1], c[2]);
  }

  function contrastRatio(l1, l2) {
    return l1 > l2 ? (l1 + 0.05) / (l2 + 0.05) : (l2 + 0.05) / (l1 + 0.05);
  }

  function contrastPair(fg, bg) {
    var r = contrastRatio(relLumHex(fg), relLumHex(bg));
    return { ratio: Math.round(r * 100) / 100, aa: r >= 4.5, aaLarge: r >= 3 };
  }

  /* ---- derivation ---- */

  var INK = "#2C2C2C";
  var DARK_PAPER = "#1c1c28";
  var DARK_INK = "#e4e4ea";

  function isDark() {
    return document.documentElement.getAttribute("data-theme") === "dark" ||
      (document.documentElement.getAttribute("data-theme") !== "light" &&
       window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches);
  }

  function deriveDark(hex) {
    var rgb = hexToRgb(hex);
    var oklch = rgbToOklch(rgb[0], rgb[1], rgb[2]);
    var L = oklch.L, C = oklch.C, H = oklch.H;

    /* accent — boost lightness slightly for visibility on dark surfaces */
    var accentL = Math.min(L + 0.05, 0.82);
    var accentC = C * 1.05;
    var accent = oklchToHex(accentL, accentC, H);

    /* hover — lighter on dark */
    var hoverL = accentL + 0.06;
    var hover = oklchToHex(hoverL, accentC * 0.95, H - 2);

    /* light — a muted tint */
    var lightL = L * 0.55 + 0.2;
    var lightC = C * 0.5;
    var light = oklchToHex(lightL, lightC, H + 10);

    /* paper — the dark background (fixed, not accent-derived in dark) */
    var paper = DARK_PAPER;

    /* text — lighten until contrast >= 4.5:1 on dark paper */
    var paperLum = relLumHex(paper);
    var textL = L + 0.15;
    var textC = C * 0.7;
    var textH = H;
    var textHex = accent;
    for (var i = 0; i < 80; i += 1) {
      textL += 0.006;
      textC = Math.max(textC - 0.0003, 0.02);
      var candidate = oklchToHex(textL, textC, textH);
      var cr = contrastRatio(relLumHex(candidate), paperLum);
      if (cr >= 4.5) { textHex = candidate; break; }
      textHex = candidate;
    }

    var r8 = Math.round(rgb[0] * 255);
    var g8 = Math.round(rgb[1] * 255);
    var b8 = Math.round(rgb[2] * 255);
    var soft = "rgb(" + r8 + " " + g8 + " " + b8 + " / .15)";
    var glow = "rgb(" + r8 + " " + g8 + " " + b8 + " / .3)";

    /* on-accent — white or dark ink, whichever has more contrast */
    var accentRgb = hexToRgb(accent);
    var accentLum = relLum(accentRgb[0], accentRgb[1], accentRgb[2]);
    var crWhite = contrastRatio(1, accentLum);
    var crInk = contrastRatio(relLumHex(INK), accentLum);
    var onAccent = crWhite >= crInk ? "#ffffff" : INK;

    return {
      accent: accent.toUpperCase(),
      accentHover: hover.toUpperCase(),
      accentLight: light.toUpperCase(),
      accentText: textHex.toUpperCase(),
      accentSoft: soft,
      accentGlow: glow,
      onAccent: onAccent,
      paper: paper.toUpperCase()
    };
  }

  function derive(hex) {
    hex = hex.replace(/\s/g, "");
    if (hex.charAt(0) !== "#") { hex = "#" + hex; }
    /* Reference orange: return the design-system literals exactly. */
    if (hex.toUpperCase() === "#F28C28") {
      return {
        accent: "#F28C28", accentHover: "#D4751C", accentLight: "#FDB96A",
        accentText: "#B45309", accentSoft: "rgb(242 140 40 / .1)",
        accentGlow: "rgb(242 140 40 / .26)", onAccent: "#2C2C2C", paper: "#FFF8F2"
      };
    }
    var rgb = hexToRgb(hex);
    var oklch = rgbToOklch(rgb[0], rgb[1], rgb[2]);
    var L = oklch.L, C = oklch.C, H = oklch.H;

    /* hover — darker, slightly desaturated */
    var hoverL = L - 0.078;
    var hoverC = C * 0.93;
    var hover = oklchToHex(hoverL, hoverC, H - 2);

    /* light — lighter tint, reduced chroma, hue shifts warm */
    var lightL = L + 0.099;
    var lightC = C * 0.77;
    var light = oklchToHex(lightL, lightC, H + 10);

    /* paper — very high L, very low C, same hue region */
    var paperL = 0.983;
    var paperC = Math.min(C * 0.07, 0.012);
    var paperH = H + 4.5;
    var paper = oklchToHex(paperL, paperC, paperH);

    /* text — darken in OKLCH until contrast >= 4.5:1 on derived paper.
       Shift hue cooler and reduce chroma to match the reference pattern
       (e.g. #F28C28 H=59 -> text #B45309 H=49). */
    var paperLum = relLumHex(paper);
    var textL = L;
    var textC = C * 0.90;
    var textH = H - 10;
    var textHex = hex;
    for (var i = 0; i < 80; i += 1) {
      textL -= 0.006;
      textC = Math.max(textC - 0.0003, 0.02);
      var candidate = oklchToHex(textL, textC, textH);
      var cr = contrastRatio(relLumHex(candidate), paperLum);
      if (cr >= 4.5) { textHex = candidate; break; }
      textHex = candidate;
    }

    /* soft & glow — use original sRGB values */
    var r8 = Math.round(rgb[0] * 255);
    var g8 = Math.round(rgb[1] * 255);
    var b8 = Math.round(rgb[2] * 255);
    var soft = "rgb(" + r8 + " " + g8 + " " + b8 + " / .1)";
    var glow = "rgb(" + r8 + " " + g8 + " " + b8 + " / .26)";

    /* on-accent — ink or white, whichever has more contrast */
    var accentLum = relLum(rgb[0], rgb[1], rgb[2]);
    var crWhite = contrastRatio(1, accentLum);
    var crInk = contrastRatio(relLumHex(INK), accentLum);
    var onAccent = crWhite >= crInk ? "#ffffff" : INK;

    return {
      accent: hex.toUpperCase(),
      accentHover: hover.toUpperCase(),
      accentLight: light.toUpperCase(),
      accentText: textHex.toUpperCase(),
      accentSoft: soft,
      accentGlow: glow,
      onAccent: onAccent,
      paper: paper.toUpperCase()
    };
  }

  /* ---- apply to document ---- */

  /* The accent last applied, wherever it came from (the person's choice or
     the product's default), so a theme change can derive it again. */
  var applied = null;

  function apply(hex) {
    hex = normalize(hex);
    applied = hex;
    var d = mantis ? deriveMantis(hex, isDark()) : isDark() ? deriveDark(hex) : derive(hex);
    var s = document.documentElement.style;
    s.setProperty("--id-accent", d.accent);
    s.setProperty("--id-accent-hover", d.accentHover);
    s.setProperty("--id-accent-light", d.accentLight);
    s.setProperty("--id-accent-text", d.accentText);
    s.setProperty("--id-accent-soft", d.accentSoft);
    s.setProperty("--id-accent-glow", d.accentGlow);
    s.setProperty("--id-on-accent", d.onAccent);
    s.setProperty("--id-paper", d.paper);
    /* Paper band at 88% opacity for the conversation column overlay */
    var pRgb = hexToRgb(d.paper);
    var pr = Math.round(pRgb[0] * 255), pg = Math.round(pRgb[1] * 255), pb = Math.round(pRgb[2] * 255);
    s.setProperty("--id-paper-band", "rgba(" + pr + "," + pg + "," + pb + ",.88)");
    return d;
  }

  /* ---- presets ---- */

  var presets = [
    { hex: "#F28C28", en: "Orange", pt: "Laranja" },
    { hex: "#2563EB", en: "Blue", pt: "Azul" },
    { hex: "#059669", en: "Green", pt: "Verde" },
    { hex: "#0D9488", en: "Teal", pt: "Azul-petróleo" },
    { hex: "#7C3AED", en: "Purple", pt: "Roxo" },
    { hex: "#DC2626", en: "Red", pt: "Vermelho" },
    { hex: "#6B7280", en: "Graphite", pt: "Grafite" },
    { hex: "#D97706", en: "Amber", pt: "Âmbar" }
  ];

  var mantis = document.documentElement.dataset.identity === "mantis";
  var mantisColors = ["#FFA65B", "#9CBDFE", "#64D7A6", "#64D3C6", "#C0B0FE", "#FFA095", "#B6BECD", "#EBB40D"];
  var mantisPresets = presets.map(function (preset, index) {
    return Object.assign({}, preset, { hex: mantisColors[index] });
  });

  function gamutHex(lightness, chroma, hue) {
    while (chroma > 0) {
      var rgb = oklchToRgb(lightness, chroma, hue);
      if (rgb.every(function (value) { return value >= -0.001 && value <= 1.001; })) {
        return rgbToHex(rgb[0], rgb[1], rgb[2]).toUpperCase();
      }
      chroma = Math.max(0, chroma - 0.004);
    }
    return oklchToHex(lightness, 0, hue).toUpperCase();
  }

  function hueOf(hex) {
    return rgbToOklch.apply(null, hexToRgb(hex)).H;
  }

  function fromHue(hue) {
    return gamutHex(0.799, 0.15, hue);
  }

  function normalize(hex) {
    var value = String(hex || "").trim().toUpperCase();
    if (/^[\dA-F]{6}$/.test(value)) { value = "#" + value; }
    if (!/^#[\dA-F]{6}$/.test(value)) { return mantis ? mantisColors[0] : presets[0].hex; }
    if (!mantis) { return value; }
    var legacy = presets.findIndex(function (preset) { return preset.hex === value; });
    if (legacy !== -1) { return mantisColors[legacy]; }
    if (mantisColors.includes(value)) { return value; }
    // Preserve colors generated by the hue control despite hex rounding.
    var color = rgbToOklch.apply(null, hexToRgb(value));
    return Math.abs(color.L - 0.799) < 0.002 && color.C <= 0.152 ? value : fromHue(color.H);
  }

  function deriveMantis(hex, dark) {
    var color = rgbToOklch.apply(null, hexToRgb(hex));
    var paper = dark ? "#0A0A0A" : "#FAFAFA";
    var textSurface = dark ? "#111111" : paper;
    var lightness = color.L;
    var accentText = hex;
    // Same rule as the Mantis prototype (gb-shell.js inkFor): step L by 0.01
    // until 4.6:1 on the paper (#FAFAFA) or the dark surface, chroma capped
    // at 0.2.
    var textBackground = dark ? textSurface : paper;
    var guard = 0;
    while (contrastRatio(relLumHex(accentText), relLumHex(textBackground)) < 4.6 && guard++ < 90) {
      lightness += dark ? 0.01 : -0.01;
      accentText = gamutHex(lightness, Math.min(color.C, 0.2), color.H);
    }
    return {
      accent: hex,
      accentHover: gamutHex(dark ? 0.85 : 0.76, color.C, color.H),
      accentLight: gamutHex(0.9, color.C * 0.55, color.H),
      accentText: accentText,
      accentSoft: "color-mix(in srgb, " + hex + (dark ? " 16%" : " 11%") + ", transparent)",
      accentGlow: "color-mix(in srgb, " + hex + " 26%, transparent)",
      onAccent: "#171717",
      paper: paper
    };
  }

  /* ---- validate ---- */

  function validate(hex) {
    var dark = isDark();
    var d = mantis ? deriveMantis(normalize(hex), dark) : dark ? deriveDark(hex) : derive(hex);
    var ink = mantis ? (dark ? "#ededed" : "#171717") : dark ? DARK_INK : INK;
    return {
      accentOnPaper: contrastPair(d.accent, d.paper),
      accentTextOnPaper: contrastPair(d.accentText, d.paper),
      onAccentOnAccent: contrastPair(d.onAccent, d.accent),
      hoverOnPaper: contrastPair(d.accentHover, d.paper),
      inkOnPaper: contrastPair(ink, d.paper)
    };
  }

  window.GBIdentity = {
    derive: mantis ? function (hex) { return deriveMantis(normalize(hex), false); } : derive,
    deriveDark: mantis ? function (hex) { return deriveMantis(normalize(hex), true); } : deriveDark,
    isDark: isDark,
    apply: apply,
    reapply: function () { return applied ? apply(applied) : null; },
    presets: mantis ? mantisPresets : presets,
    hueOf: hueOf,
    fromHue: fromHue,
    validate: validate,
    contrastPair: contrastPair
  };
  if (mantis) {
    var stored = null;
    try { stored = localStorage.getItem("gb-accent"); } catch (e) { /* storage unavailable */ }
    apply(stored);
  }
})();
