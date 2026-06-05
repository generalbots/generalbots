"use strict";

/**
 * Module 12: Data validation for Sheet.
 * Provides: list, number range, text length, custom formula, date checks, error alerts.
 */

function evalFormulaSafe(formula, context) {
  if (!formula || typeof formula !== "string") return null;
  const allowed = /^[A-Z]+\d+(\s*[+\-*/^()A-Z0-9<>=!&|\s,.]*)?$/i;
  if (!allowed.test(formula.replace(/^=/, ""))) return null;
  try {
    const safe = formula
      .replace(/^=/, "")
      .replace(/\^/g, "**")
      .replace(/&&/g, "&&")
      .replace(/\s+/g, " ");
    if (context && typeof context === "object") {
      for (const k of Object.keys(context)) {
        const re = new RegExp(`\\b${k}\\b`, "g");
        safe.replace(re, String(context[k]));
      }
    }
    const fn = new Function("with (Math) { return (" + safe + "); }");
    const result = fn.call({});
    return Number.isFinite(result) ? result : null;
  } catch (_e) {
    return null;
  }
}

function validateList(value, source, allowBlank) {
  if (value == null || value === "") return allowBlank !== false;
  if (Array.isArray(source)) {
    return source.some((s) => String(s) === String(value));
  }
  if (typeof source === "string") {
    const opts = source.split(",").map((s) => s.trim());
    return opts.includes(String(value));
  }
  return true;
}

function validateNumber(value, min, max, allowBlank) {
  if (value == null || value === "") return allowBlank !== false;
  const n = Number(value);
  if (!Number.isFinite(n)) return false;
  if (min != null && n < min) return false;
  if (max != null && n > max) return false;
  return true;
}

function validateTextLength(value, min, max, allowBlank) {
  if (value == null || value === "") return allowBlank !== false;
  const len = String(value).length;
  if (min != null && len < min) return false;
  if (max != null && len > max) return false;
  return true;
}

function validateDate(value, min, max, allowBlank) {
  if (value == null || value === "") return allowBlank !== false;
  const d = new Date(value);
  if (isNaN(d.getTime())) return false;
  if (min) {
    const minD = new Date(min);
    if (d < minD) return false;
  }
  if (max) {
    const maxD = new Date(max);
    if (d > maxD) return false;
  }
  return true;
}

function validateRegex(value, pattern, allowBlank) {
  if (value == null || value === "") return allowBlank !== false;
  try {
    const re = new RegExp(pattern);
    return re.test(String(value));
  } catch (_e) {
    return false;
  }
}

function validateCustom(value, formula, context, allowBlank) {
  if (value == null || value === "") return allowBlank !== false;
  const ctx = Object.assign({}, context || {}, { VALUE: value });
  const result = evalFormulaSafe(formula, ctx);
  return result === 1 || result === true;
}

function buildValidator(rule) {
  if (!rule) return () => true;
  const allowBlank = rule.allowBlank !== false;
  switch (rule.type) {
    case "list":
      return (value) => validateList(value, rule.source, allowBlank);
    case "number":
      return (value) => validateNumber(value, rule.min, rule.max, allowBlank);
    case "textLength":
      return (value) => validateTextLength(value, rule.min, rule.max, allowBlank);
    case "date":
      return (value) => validateDate(value, rule.min, rule.max, allowBlank);
    case "regex":
      return (value) => validateRegex(value, rule.pattern, allowBlank);
    case "custom":
      return (value, ctx) => validateCustom(value, rule.formula, ctx, allowBlank);
    case "email":
      return (value) =>
        !value || allowBlank
          ? allowBlank
          : /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(String(value));
    case "url":
      return (value) => {
        if (!value) return allowBlank;
        try {
          new URL(String(value));
          return true;
        } catch (_e) {
          return false;
        }
      };
    default:
      return () => true;
  }
}

function applyValidationToRange(range, rule, context) {
  const errors = [];
  const validator = buildValidator(rule);
  range.forEach((row, r) => {
    row.forEach((value, c) => {
      if (!validator(value, context)) {
        errors.push({ row: r, col: c, value, message: rule.errorMessage || "Invalid value" });
      }
    });
  });
  return errors;
}

window.SheetValidation = {
  validateList,
  validateNumber,
  validateTextLength,
  validateDate,
  validateRegex,
  validateCustom,
  buildValidator,
  applyValidationToRange,
  evalFormulaSafe,
};
