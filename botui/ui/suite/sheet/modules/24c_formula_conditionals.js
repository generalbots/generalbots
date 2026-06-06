"use strict";

/**
 * Module 24c: Conditional aggregate functions for SheetFormulaEngine.
 * Split out from 24a to keep that file under 450 lines. Loaded after
 * 24a. Registers SUMIF, COUNTIF, AVERAGEIF, SUMIFS, COUNTIFS.
 */

(function () {
  const F = window.SheetFormulaEngine;
  if (!F || !F.registerFunction) {
    if (typeof console !== "undefined") console.warn("SheetFormulaEngine not loaded; conditional aggregates skipped");
    return;
  }
  const flatten = F.flatten;
  const numCoerce = F.numCoerce;
  const strCoerce = F.strCoerce;

  function reg(name, arity, fn) {
    F.registerFunction(name, function (args, ctx) {
      if (arity >= 0 && args.length < arity) return { type: "error", value: "#VALUE!" };
      return fn(args, ctx);
    });
  }

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
      if (ok) { total += numCoerce(sumRange[i] || 0); count++; }
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
})();
