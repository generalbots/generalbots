"use strict";

/**
 * Module 24a: Built-in function registrations for SheetFormulaEngine.
 * Loaded after 24_formula_engine.js. Registers 60+ safe functions:
 * aggregations, logic, text, math, datetime, lookup, conditional
 * aggregates, and helpers. Replaces any insecure `new Function()`
 * usage with hand-written, parser-friendly implementations.
 *
 * No public API; this is a side-effect module that augments
 * window.SheetFormulaEngine.FUNCS via the exposed registerFunction.
 */

(function () {
  const F = window.SheetFormulaEngine;
  if (!F || !F.registerFunction) {
    if (typeof console !== "undefined") console.warn("SheetFormulaEngine not loaded; function registrations skipped");
    return;
  }
  const reg = F.registerFunction;
  const AGG = F.AGG;
  const flatten = F.flatten;
  const numCoerce = F.numCoerce;
  const strCoerce = F.strCoerce;
  const toBool = F.toBool;
  const isError = F.isError;
  const cmpValues = F.cmpValues;

  reg("SUM", 1, function (a) { return { type: "num", value: AGG._flattenNumeric(a[0]).reduce(function (s, x) { return s + x; }, 0) }; });
  reg("AVERAGE", 1, function (a) {
    const arr = AGG._flattenNumeric(a[0]);
    if (arr.length === 0) return { type: "error", value: "#DIV/0!" };
    return { type: "num", value: arr.reduce(function (s, x) { return s + x; }, 0) / arr.length };
  });
  reg("MIN", 1, function (a) {
    const arr = AGG._flattenNumeric(a[0]);
    if (arr.length === 0) return { type: "error", value: "#VALUE!" };
    return { type: "num", value: Math.min.apply(null, arr) };
  });
  reg("MAX", 1, function (a) {
    const arr = AGG._flattenNumeric(a[0]);
    if (arr.length === 0) return { type: "error", value: "#VALUE!" };
    return { type: "num", value: Math.max.apply(null, arr) };
  });
  reg("COUNT", 1, function (a) { return { type: "num", value: AGG._flattenNumeric(a[0]).length }; });
  reg("COUNTA", 1, function (a) { return { type: "num", value: AGG._countAll(a[0]) }; });
  reg("PRODUCT", 1, function (a) { return { type: "num", value: AGG._flattenNumeric(a[0]).reduce(function (s, x) { return s * x; }, 1) }; });
  reg("STDEV", 1, function (a) {
    const arr = AGG._flattenNumeric(a[0]);
    if (arr.length < 2) return { type: "error", value: "#DIV/0!" };
    const m = arr.reduce(function (s, x) { return s + x; }, 0) / arr.length;
    const v = arr.reduce(function (s, x) { return s + (x - m) * (x - m); }, 0) / (arr.length - 1);
    return { type: "num", value: Math.sqrt(v) };
  });
  reg("STDEVP", 1, function (a) {
    const arr = AGG._flattenNumeric(a[0]);
    if (arr.length === 0) return { type: "error", value: "#DIV/0!" };
    const m = arr.reduce(function (s, x) { return s + x; }, 0) / arr.length;
    const v = arr.reduce(function (s, x) { return s + (x - m) * (x - m); }, 0) / arr.length;
    return { type: "num", value: Math.sqrt(v) };
  });
  reg("MEDIAN", 1, function (a) {
    const arr = AGG._flattenNumeric(a[0]).sort(function (x, y) { return x - y; });
    if (arr.length === 0) return { type: "error", value: "#NUM!" };
    const m = Math.floor(arr.length / 2);
    return { type: "num", value: arr.length % 2 ? arr[m] : (arr[m - 1] + arr[m]) / 2 };
  });

  reg("IF", 2, function (a) {
    return toBool(a[0]) ? (a[1] || { type: "empty" }) : (a[2] || { type: "empty" });
  });
  reg("AND", 1, function (a) {
    for (const v of flatten(a[0])) if (!toBool(v)) return { type: "bool", value: false };
    return { type: "bool", value: true };
  });
  reg("OR", 1, function (a) {
    for (const v of flatten(a[0])) if (toBool(v)) return { type: "bool", value: true };
    return { type: "bool", value: false };
  });
  reg("NOT", 1, function (a) { return { type: "bool", value: !toBool(a[0]) }; });
  reg("IFERROR", 1, function (a) {
    if (a.length < 1) return { type: "error", value: "#VALUE!" };
    return isError(a[0]) ? (a[1] || { type: "empty" }) : a[0];
  });

  reg("ISBLANK", 1, function (a) { return { type: "bool", value: a[0] && a[0].type === "empty" }; });
  reg("ISNUMBER", 1, function (a) { return { type: "bool", value: a[0] && a[0].type === "num" }; });
  reg("ISTEXT", 1, function (a) { return { type: "bool", value: a[0] && a[0].type === "str" }; });
  reg("ISERROR", 1, function (a) { return { type: "bool", value: isError(a[0]) }; });
  reg("ISLOGICAL", 1, function (a) { return { type: "bool", value: a[0] && a[0].type === "bool" }; });
  reg("ISNONTEXT", 1, function (a) { return { type: "bool", value: !(a[0] && a[0].type === "str") }; });

  reg("CONCATENATE", 1, function (a) {
    let s = "";
    for (const v of a) s += strCoerce(v);
    return { type: "str", value: s };
  });
  reg("CONCAT", 1, function (a) {
    let s = "";
    for (const v of a) s += strCoerce(v);
    return { type: "str", value: s };
  });
  reg("LEFT", 1, function (a) {
    const s = strCoerce(a[0]);
    const n = a.length > 1 ? Math.floor(numCoerce(a[1])) : 1;
    return { type: "str", value: s.slice(0, n) };
  });
  reg("RIGHT", 1, function (a) {
    const s = strCoerce(a[0]);
    const n = a.length > 1 ? Math.floor(numCoerce(a[1])) : 1;
    return { type: "str", value: s.slice(-n) };
  });
  reg("MID", 3, function (a) {
    const s = strCoerce(a[0]);
    const start = Math.floor(numCoerce(a[1])) - 1;
    const len = Math.floor(numCoerce(a[2]));
    return { type: "str", value: start < 0 ? "" : s.slice(start, start + len) };
  });
  reg("LEN", 1, function (a) { return { type: "num", value: strCoerce(a[0]).length }; });
  reg("UPPER", 1, function (a) { return { type: "str", value: strCoerce(a[0]).toUpperCase() }; });
  reg("LOWER", 1, function (a) { return { type: "str", value: strCoerce(a[0]).toLowerCase() }; });
  reg("TRIM", 1, function (a) { return { type: "str", value: strCoerce(a[0]).replace(/\s+/g, " ").trim() }; });
  reg("PROPER", 1, function (a) {
    const s = strCoerce(a[0]);
    return { type: "str", value: s.replace(/\w\S*/g, function (w) { return w.charAt(0).toUpperCase() + w.slice(1).toLowerCase(); }) };
  });
  reg("REPLACE", 4, function (a) {
    const s = strCoerce(a[0]);
    const start = Math.floor(numCoerce(a[1])) - 1;
    const len = Math.floor(numCoerce(a[2]));
    const rep = strCoerce(a[3]);
    return { type: "str", value: s.slice(0, start) + rep + s.slice(start + len) };
  });
  reg("SUBSTITUTE", 3, function (a) {
    const s = strCoerce(a[0]);
    const find = strCoerce(a[1]);
    const rep = strCoerce(a[2]);
    if (a.length > 3) {
      const n = Math.floor(numCoerce(a[3]));
      let idx = 0, count = 0, result = s;
      while ((idx = result.indexOf(find, idx)) !== -1) {
        count++;
        if (count === n) {
          return { type: "str", value: result.slice(0, idx) + rep + result.slice(idx + find.length) };
        }
        idx += find.length;
      }
      return { type: "str", value: s };
    }
    return { type: "str", value: s.split(find).join(rep) };
  });
  reg("FIND", 2, function (a) {
    const s = strCoerce(a[0]);
    const find = strCoerce(a[1]);
    const start = a.length > 2 ? Math.floor(numCoerce(a[2])) - 1 : 0;
    const idx = s.indexOf(find, start);
    return { type: "num", value: idx === -1 ? 0 : idx + 1 };
  });
  reg("SEARCH", 2, function (a) {
    const s = strCoerce(a[0]);
    const find = strCoerce(a[1]);
    const re = new RegExp(find.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i");
    const m = s.slice(a.length > 2 ? Math.floor(numCoerce(a[2])) - 1 : 0).match(re);
    return { type: "num", value: m ? m.index + 1 : 0 };
  });
  reg("EXACT", 2, function (a) { return { type: "bool", value: strCoerce(a[0]) === strCoerce(a[1]) }; });
  reg("REPT", 2, function (a) {
    const s = strCoerce(a[0]);
    const n = Math.floor(numCoerce(a[1]));
    return { type: "str", value: s.length * n > 32767 ? "" : s.repeat(Math.max(0, n)) };
  });
  reg("TEXT", 2, function (a) {
    const v = a[0];
    const fmt = strCoerce(a[1]);
    const n = numCoerce(v);
    if (/^[$€¥£]/.test(fmt) && /0\.0+%/.test(fmt)) return { type: "str", value: "$" + n.toFixed(2) };
    if (/0\.0+%/.test(fmt)) return { type: "str", value: (n * 100).toFixed(2) + "%" };
    if (/0\.00/.test(fmt)) return { type: "str", value: n.toFixed(2) };
    if (/0\.0/.test(fmt)) return { type: "str", value: n.toFixed(1) };
    if (/0/.test(fmt)) return { type: "str", value: String(Math.round(n)) };
    return { type: "str", value: strCoerce(v) };
  });
  reg("VALUE", 1, function (a) {
    const s = strCoerce(a[0]);
    const n = parseFloat(s);
    if (Number.isNaN(n)) return { type: "error", value: "#VALUE!" };
    return { type: "num", value: n };
  });

  reg("ROUND", 2, function (a) { const n = numCoerce(a[0]); const d = numCoerce(a[1]); const m = Math.pow(10, d); return { type: "num", value: Math.round(n * m) / m }; });
  reg("ROUNDUP", 2, function (a) { const n = numCoerce(a[0]); const d = numCoerce(a[1]); const m = Math.pow(10, d); if (n >= 0) return { type: "num", value: Math.ceil(n * m) / m }; return { type: "num", value: Math.floor(n * m) / m }; });
  reg("ROUNDDOWN", 2, function (a) { const n = numCoerce(a[0]); const d = numCoerce(a[1]); const m = Math.pow(10, d); if (n >= 0) return { type: "num", value: Math.floor(n * m) / m }; return { type: "num", value: Math.ceil(n * m) / m }; });
  reg("CEILING", 2, function (a) { return { type: "num", value: Math.ceil(numCoerce(a[0]) / numCoerce(a[1])) * numCoerce(a[1]) }; });
  reg("FLOOR", 2, function (a) { return { type: "num", value: Math.floor(numCoerce(a[0]) / numCoerce(a[1])) * numCoerce(a[1]) }; });
  reg("INT", 1, function (a) { return { type: "num", value: Math.floor(numCoerce(a[0])) }; });
  reg("ABS", 1, function (a) { return { type: "num", value: Math.abs(numCoerce(a[0])) }; });
  reg("SQRT", 1, function (a) { return { type: "num", value: Math.sqrt(numCoerce(a[0])) }; });
  reg("POWER", 2, function (a) { return { type: "num", value: Math.pow(numCoerce(a[0]), numCoerce(a[1])) }; });
  reg("MOD", 2, function (a) { return { type: "num", value: numCoerce(a[0]) % numCoerce(a[1]) }; });
  reg("EXP", 1, function (a) { return { type: "num", value: Math.exp(numCoerce(a[0])) }; });
  reg("LN", 1, function (a) { return { type: "num", value: Math.log(numCoerce(a[0])) }; });
  reg("LOG", 2, function (a) { return { type: "num", value: Math.log(numCoerce(a[0])) / Math.log(numCoerce(a[1])) }; });
  reg("LOG10", 1, function (a) { return { type: "num", value: Math.log10(numCoerce(a[0])) }; });
  reg("SIGN", 1, function (a) { const n = numCoerce(a[0]); return { type: "num", value: n > 0 ? 1 : n < 0 ? -1 : 0 }; });
  reg("PI", 0, function () { return { type: "num", value: Math.PI }; });
  reg("RAND", 0, function () { return { type: "num", value: Math.random() }; });
  reg("RANDBETWEEN", 2, function (a) {
    const lo = Math.min(numCoerce(a[0]), numCoerce(a[1]));
    const hi = Math.max(numCoerce(a[0]), numCoerce(a[1]));
    return { type: "num", value: Math.floor(Math.random() * (hi - lo + 1)) + lo };
  });
  reg("SIN", 1, function (a) { return { type: "num", value: Math.sin(numCoerce(a[0])) }; });
  reg("COS", 1, function (a) { return { type: "num", value: Math.cos(numCoerce(a[0])) }; });
  reg("TAN", 1, function (a) { return { type: "num", value: Math.tan(numCoerce(a[0])) }; });
  reg("ASIN", 1, function (a) { return { type: "num", value: Math.asin(numCoerce(a[0])) }; });
  reg("ACOS", 1, function (a) { return { type: "num", value: Math.acos(numCoerce(a[0])) }; });
  reg("ATAN", 1, function (a) { return { type: "num", value: Math.atan(numCoerce(a[0])) }; });
  reg("ATAN2", 2, function (a) { return { type: "num", value: Math.atan2(numCoerce(a[0]), numCoerce(a[1])) }; });

  reg("DATE", 3, function (a) { return { type: "num", value: Date.UTC(numCoerce(a[0]), numCoerce(a[1]) - 1, numCoerce(a[2])) / 86400000 }; });
  reg("YEAR", 1, function (a) { return { type: "num", value: new Date(numCoerce(a[0]) * 86400000).getUTCFullYear() }; });
  reg("MONTH", 1, function (a) { return { type: "num", value: new Date(numCoerce(a[0]) * 86400000).getUTCMonth() + 1 }; });
  reg("DAY", 1, function (a) { return { type: "num", value: new Date(numCoerce(a[0]) * 86400000).getUTCDate() }; });
  reg("HOUR", 1, function (a) { return { type: "num", value: new Date(numCoerce(a[0]) * 86400000).getUTCHours() }; });
  reg("MINUTE", 1, function (a) { return { type: "num", value: new Date(numCoerce(a[0]) * 86400000).getUTCMinutes() }; });
  reg("SECOND", 1, function (a) { return { type: "num", value: new Date(numCoerce(a[0]) * 86400000).getUTCSeconds() }; });
  reg("NOW", 0, function () { return { type: "num", value: Date.now() / 86400000 }; });
  reg("TODAY", 0, function () { const d = new Date(); return { type: "num", value: Date.UTC(d.getFullYear(), d.getMonth(), d.getDate()) / 86400000 }; });
  reg("DATEDIF", 3, function (a) {
    const s = new Date(numCoerce(a[0]) * 86400000);
    const e = new Date(numCoerce(a[1]) * 86400000);
    const unit = strCoerce(a[2]);
    let v = 0;
    if (unit === "Y") v = e.getUTCFullYear() - s.getUTCFullYear();
    else if (unit === "M") v = (e.getUTCFullYear() - s.getUTCFullYear()) * 12 + (e.getUTCMonth() - s.getUTCMonth());
    else if (unit === "D") v = Math.floor((e - s) / 86400000);
    return { type: "num", value: v };
  });

  reg("VLOOKUP", 3, function (a) {
    const lookup = a[0];
    const range = a[1];
    const colIdx = Math.floor(numCoerce(a[2])) - 1;
    if (!range || !range.values) return { type: "error", value: "#REF!" };
    const arr = range.values;
    const cols = (range.end.col - range.start.col) + 1;
    for (let i = 0; i < arr.length; i += cols) {
      const c = cmpValues(arr[i], lookup);
      if (c.type === "num" && c.value === 0) {
        return arr[i + colIdx] || { type: "empty" };
      }
      if (strCoerce(arr[i]) === strCoerce(lookup)) return arr[i + colIdx] || { type: "empty" };
    }
    return { type: "error", value: "#N/A" };
  });
  reg("HLOOKUP", 3, function (a) {
    const lookup = a[0];
    const range = a[1];
    const rowIdx = Math.floor(numCoerce(a[2])) - 1;
    if (!range || !range.values) return { type: "error", value: "#REF!" };
    const arr = range.values;
    const cols = (range.end.col - range.start.col) + 1;
    for (let c = 0; c < cols; c++) {
      if (strCoerce(arr[c]) === strCoerce(lookup)) {
        return arr[rowIdx * cols + c] || { type: "empty" };
      }
    }
    return { type: "error", value: "#N/A" };
  });
  reg("XLOOKUP", 3, function (a) {
    const lookup = a[0];
    const lookupArr = a[1];
    const returnArr = a[2];
    if (!lookupArr || !returnArr) return { type: "error", value: "#N/A" };
    const lv = flatten(lookupArr);
    const rv = flatten(returnArr);
    for (let i = 0; i < lv.length && i < rv.length; i++) {
      if (strCoerce(lv[i]) === strCoerce(lookup)) return rv[i] || { type: "empty" };
    }
    if (a.length > 3 && !isError(a[3])) return a[3];
    return { type: "error", value: "#N/A" };
  });
  reg("INDEX", 2, function (a) {
    const range = a[0];
    if (!range || !range.values) return { type: "error", value: "#REF!" };
    const arr = range.values;
    const cols = (range.end.col - range.start.col) + 1;
    if (a.length === 2) {
      const row = Math.floor(numCoerce(a[1])) - 1;
      return arr[row * cols] || { type: "empty" };
    }
    const row = Math.floor(numCoerce(a[1])) - 1;
    const col = Math.floor(numCoerce(a[2])) - 1;
    return arr[row * cols + col] || { type: "empty" };
  });
  reg("MATCH", 2, function (a) {
    const lookup = a[0];
    const arr = flatten(a[1]);
    for (let i = 0; i < arr.length; i++) {
      if (strCoerce(arr[i]) === strCoerce(lookup)) return { type: "num", value: i + 1 };
    }
    return { type: "error", value: "#N/A" };
  });
  reg("CHOOSE", 2, function (a) {
    const idx = Math.floor(numCoerce(a[0])) - 1;
    return a[idx + 1] || { type: "error", value: "#VALUE!" };
  });
  reg("ROW", 0, function (a, ctx) { return { type: "num", value: (ctx.currentCell && ctx.currentCell.row != null) ? ctx.currentCell.row + 1 : 1 }; });
  reg("COLUMN", 0, function (a, ctx) { return { type: "num", value: (ctx.currentCell && ctx.currentCell.col != null) ? ctx.currentCell.col + 1 : 1 }; });
  reg("ROWS", 1, function (a) { return { type: "num", value: a[0] && a[0].values ? a[0].values.length / ((a[0].end.col - a[0].start.col) + 1) : 0 }; });
  reg("COLUMNS", 1, function (a) { return { type: "num", value: a[0] && a[0].values ? (a[0].end.col - a[0].start.col) + 1 : 0 }; });

  reg("SUMIF", 2, function (a) {
    const range = flatten(a[0]);
    const criteria = a[1];
    const sumRange = a.length > 2 ? flatten(a[2]) : range;
    let total = 0;
    const cv = strCoerce(criteria);
    const reMatch = cv.match(/^([<>=!]+)(.+)$/);
    const op = reMatch ? reMatch[1] : "=";
    const target = reMatch ? reMatch[2] : cv;
    for (let i = 0; i < range.length; i++) {
      const v = strCoerce(range[i]);
      let ok = false;
      if (op === "=") ok = v === target;
      else if (op === "<>") ok = v !== target;
      else {
        const n = parseFloat(v);
        const tn = parseFloat(target);
        if (!Number.isNaN(n) && !Number.isNaN(tn)) {
          if (op === "<") ok = n < tn;
          else if (op === ">") ok = n > tn;
          else if (op === "<=") ok = n <= tn;
          else if (op === ">=") ok = n >= tn;
        }
      }
      if (ok) total += numCoerce(sumRange[i] || 0);
    }
    return { type: "num", value: total };
  });
  reg("COUNTIF", 2, function (a) {
    const range = flatten(a[0]);
    const criteria = a[1];
    const cv = strCoerce(criteria);
    const reMatch = cv.match(/^([<>=!]+)(.+)$/);
    const op = reMatch ? reMatch[1] : "=";
    const target = reMatch ? reMatch[2] : cv;
    let n = 0;
    for (const v of range) {
      const sv = strCoerce(v);
      let ok = false;
      if (op === "=") ok = sv === target;
      else if (op === "<>") ok = sv !== target;
      else {
        const x = parseFloat(sv);
        const t = parseFloat(target);
        if (!Number.isNaN(x) && !Number.isNaN(t)) {
          if (op === "<") ok = x < t;
          else if (op === ">") ok = x > t;
          else if (op === "<=") ok = x <= t;
          else if (op === ">=") ok = x >= t;
        }
      }
      if (ok) n++;
    }
    return { type: "num", value: n };
  });
  reg("AVERAGEIF", 2, function (a) {
    const range = flatten(a[0]);
    const criteria = a[1];
    const sumRange = a.length > 2 ? flatten(a[2]) : range;
    let total = 0, count = 0;
    const cv = strCoerce(criteria);
    for (let i = 0; i < range.length; i++) {
      const v = strCoerce(range[i]);
      if (v === cv) { total += numCoerce(sumRange[i] || 0); count++; }
    }
    if (count === 0) return { type: "error", value: "#DIV/0!" };
    return { type: "num", value: total / count };
  });
  reg("SUMIFS", 3, function (a) {
    let total = 0;
    const len = flatten(a[0]).length;
    for (let i = 0; i < len; i++) {
      let ok = true;
      for (let p = 1; p + 1 < a.length; p += 2) {
        const v = flatten(a[p])[i];
        const cv = strCoerce(v);
        const c = strCoerce(a[p + 1]);
        if (cv !== c) { ok = false; break; }
      }
      if (ok) total += numCoerce(flatten(a[0])[i] || 0);
    }
    return { type: "num", value: total };
  });
  reg("COUNTIFS", 1, function (a) {
    let n = 0;
    const len = flatten(a[0]).length;
    for (let i = 0; i < len; i++) {
      let ok = true;
      for (let p = 1; p + 1 < a.length; p += 2) {
        const v = flatten(a[p])[i];
        const cv = strCoerce(v);
        const c = strCoerce(a[p + 1]);
        if (cv !== c) { ok = false; break; }
      }
      if (ok) n++;
    }
    return { type: "num", value: n };
  });

  reg("TRUE", 0, function () { return { type: "bool", value: true }; });
  reg("FALSE", 0, function () { return { type: "bool", value: false }; });
  reg("NA", 0, function () { return { type: "error", value: "#N/A" }; });
})();
