"use strict";
/* Issue #2434 - statistical regression/variance/distribution functions and
 * text functions. Normal CDF uses a regularised incomplete gamma (machine
 * precision); NORM.INV bisects the CDF so the inverse is equally accurate. */
(function () {
  var E = window.SheetFormulaErrors;
  var LANCZOS = [
    76.18009172947146, -86.50532032941677, 24.01409824083091,
    -1.231739572450155, 0.001208650973866179, -0.000005395239384953,
  ];

  function logGamma(x) {
    var y = x + 5.5;
    y -= (x + 0.5) * Math.log(y);
    var series = 1.000000000190015;
    for (var i = 0; i < LANCZOS.length; i += 1) series += LANCZOS[i] / (x + i + 1);
    return -y + Math.log(2.5066282746310005 * series / x);
  }

  function gammaP(a, x) {
    if (x <= 0) return 0;
    if (x < a + 1) {
      var sum = 1 / a;
      var term = sum;
      for (var n = 1; n < 400; n += 1) {
        term *= x / (a + n);
        sum += term;
        if (Math.abs(term) < Math.abs(sum) * 1e-16) break;
      }
      return sum * Math.exp(-x + a * Math.log(x) - logGamma(a));
    }
    var tiny = 1e-300;
    var b = x + 1 - a;
    var c = 1 / tiny;
    var d = 1 / b;
    var h = d;
    for (var i = 1; i < 400; i += 1) {
      var an = -i * (i - a);
      b += 2;
      d = an * d + b;
      if (Math.abs(d) < tiny) d = tiny;
      c = b + an / c;
      if (Math.abs(c) < tiny) c = tiny;
      d = 1 / d;
      var delta = d * c;
      h *= delta;
      if (Math.abs(delta - 1) < 1e-16) break;
    }
    var q = Math.exp(-x + a * Math.log(x) - logGamma(a)) * h;
    return 1 - q;
  }

  function normalCdf(z) {
    if (z === 0) return 0.5;
    var tail = gammaP(0.5, z * z / 2) / 2;
    return z > 0 ? 0.5 + tail : 0.5 - tail;
  }

  function pairs(values) {
    var failed = E.firstError(values);
    if (failed) return failed;
    var xs = E.numbers(E.flatten([values[0]]));
    var ys = E.numbers(E.flatten([values[1]]));
    if (E.isError(xs)) return xs;
    if (E.isError(ys)) return ys;
    if (xs.length !== ys.length || xs.length < 2) return E.err("#NUM!");
    return { xs: xs, ys: ys, n: xs.length };
  }

  function sums(values) {
    var p = pairs(values);
    if (E.isError(p)) return p;
    var sx = 0, sy = 0, sxx = 0, syy = 0, sxy = 0;
    for (var i = 0; i < p.n; i += 1) {
      sx += p.xs[i]; sy += p.ys[i];
      sxx += p.xs[i] * p.xs[i];
      syy += p.ys[i] * p.ys[i];
      sxy += p.xs[i] * p.ys[i];
    }
    return { n: p.n, sx: sx, sy: sy, sxx: sxx, syy: syy, sxy: sxy };
  }

  function CORREL(values) {
    var s = sums(values);
    if (E.isError(s)) return s;
    var dx = s.n * s.sxx - s.sx * s.sx;
    var dy = s.n * s.syy - s.sy * s.sy;
    if (dx <= 0 || dy <= 0) return E.err("#DIV/0!");
    return E.finite((s.n * s.sxy - s.sx * s.sy) / Math.sqrt(dx * dy));
  }

  function SLOPE(values) {
    var s = sums([values[1], values[0]]);
    if (E.isError(s)) return s;
    var denominator = s.n * s.sxx - s.sx * s.sx;
    if (denominator === 0) return E.err("#DIV/0!");
    return E.finite((s.n * s.sxy - s.sx * s.sy) / denominator);
  }

  function INTERCEPT(values) {
    var s = sums([values[1], values[0]]);
    if (E.isError(s)) return s;
    var slope = SLOPE(values);
    if (E.isError(slope)) return slope;
    return E.finite((s.sy - slope * s.sx) / s.n);
  }

  function FORECAST(values) {
    var x = E.num(values[0]);
    if (E.isError(x)) return x;
    var slope = SLOPE([values[1], values[2]]);
    if (E.isError(slope)) return slope;
    var intercept = INTERCEPT([values[1], values[2]]);
    if (E.isError(intercept)) return intercept;
    return E.finite(slope * x + intercept);
  }

  function variance(values, sample) {
    var failed = E.firstError(values);
    if (failed) return failed;
    var list = E.numbers(E.flatten([values[0]]));
    if (E.isError(list)) return list;
    var n = list.length;
    if (n < 2 || (sample && n < 2)) return E.err("#DIV/0!");
    var mean = list.reduce(function (a, b) { return a + b; }, 0) / n;
    var total = list.reduce(function (a, b) { return a + (b - mean) * (b - mean); }, 0);
    return E.finite(total / (sample ? n - 1 : n));
  }

  function STDEV_S(values) { var v = variance(values, true); return E.isError(v) ? v : E.positiveRoot(v); }
  function STDEV_P(values) { var v = variance(values, false); return E.isError(v) ? v : E.positiveRoot(v); }
  function VAR_S(values) { return variance(values, true); }
  function VAR_P(values) { return variance(values, false); }

  function MODE_SNGL(values) {
    var failed = E.firstError(values);
    if (failed) return failed;
    var list = E.numbers(E.flatten(values));
    if (E.isError(list)) return list;
    var counts = {};
    var best = null;
    var bestCount = 1;
    list.forEach(function (value) {
      var key = String(value);
      counts[key] = (counts[key] || 0) + 1;
      if (counts[key] > bestCount) { bestCount = counts[key]; best = value; }
    });
    return best === null ? E.err("#N/A") : E.finite(best);
  }

  function NORM_DIST(values) {
    var x = E.num(values[0]);
    if (E.isError(x)) return x;
    var mean = E.num(values[1]);
    if (E.isError(mean)) return mean;
    var sd = E.num(values[2]);
    if (E.isError(sd)) return sd;
    if (sd <= 0) return E.err("#NUM!");
    var cumulative = values[3] === undefined || values[3] === null ? true : Boolean(values[3]);
    var z = (x - mean) / sd;
    if (!cumulative) return E.finite(Math.exp(-z * z / 2) / (sd * Math.sqrt(2 * Math.PI)));
    return E.finite(normalCdf(z));
  }

  function NORM_INV(values) {
    var p = E.num(values[0]);
    if (E.isError(p)) return p;
    var mean = E.num(values[1]);
    if (E.isError(mean)) return mean;
    var sd = E.num(values[2]);
    if (E.isError(sd)) return sd;
    if (p <= 0 || p >= 1 || sd <= 0) return E.err("#NUM!");
    var lo = -40, hi = 40;
    for (var i = 0; i < 200; i += 1) {
      var mid = (lo + hi) / 2;
      if (normalCdf(mid) < p) lo = mid;
      else hi = mid;
    }
    return E.finite(mean + sd * (lo + hi) / 2);
  }

  function TEXTJOIN(values) {
    var failed = E.firstError(values);
    if (failed) return failed;
    var delimiter = String(values[0] === undefined || values[0] === null ? "" : values[0]);
    var ignore = Boolean(values[1]);
    var parts = [];
    E.flatten(values.slice(2)).forEach(function (value) {
      if (ignore && (value === "" || value === null || value === undefined)) return;
      parts.push(E.format(value));
    });
    return parts.join(delimiter);
  }

  function CONCAT(values) {
    var failed = E.firstError(values);
    if (failed) return failed;
    return E.flatten(values).map(function (value) {
      return value === null || value === undefined ? "" : E.format(value);
    }).join("");
  }

  function NUMBERVALUE(values) {
    var failed = E.firstError(values);
    if (failed) return failed;
    if (typeof values[0] !== "string") return E.err("#VALUE!");
    var settings = window.SheetFunctionsSettings;
    var current = settings ? settings.state() : null;
    var decimal = values[1] === undefined ? (current ? current.separators.decimal : ".") : String(values[1]).slice(0, 1);
    var group = values[2] === undefined ? (current ? current.separators.group : ",") : String(values[2]).slice(0, 1);
    if (!decimal || decimal === group) return E.err("#VALUE!");
    var cleaned = values[0].trim()
      .split(group).join("")
      .split(decimal).join(".")
      .replace(/^\+/, "");
    if (!/^-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?$/.test(cleaned)) return E.err("#VALUE!");
    return E.finite(Number(cleaned));
  }

  window.SheetFunctionsStatText = {
    CORREL: CORREL, SLOPE: SLOPE, INTERCEPT: INTERCEPT, FORECAST: FORECAST,
    STDEV_S: STDEV_S, STDEV_P: STDEV_P, VAR_S: VAR_S, VAR_P: VAR_P,
    MODE_SNGL: MODE_SNGL, NORM_DIST: NORM_DIST, NORM_INV: NORM_INV,
    TEXTJOIN: TEXTJOIN, CONCAT: CONCAT, NUMBERVALUE: NUMBERVALUE,
    gammaP: gammaP, normalCdf: normalCdf,
  };
})();
