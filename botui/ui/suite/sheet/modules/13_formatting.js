"use strict";

/**
 * Module 13: Conditional formatting for Sheet.
 * Provides: cell rules, color scales, data bars, icon sets, top/bottom N, formula-based.
 */

function evaluateRule(rule, value) {
  if (!rule) return false;
  const op = rule.operator || "equals";
  const target = rule.value;
  const num = (v) => (typeof v === "number" ? v : parseFloat(v));
  const str = (v) => (v == null ? "" : String(v));
  switch (op) {
    case "equals":
      return str(value) === str(target);
    case "notEquals":
      return str(value) !== str(target);
    case "contains":
      return str(value).toLowerCase().includes(str(target).toLowerCase());
    case "greater":
      return num(value) > num(target);
    case "less":
      return num(value) < num(target);
    case "greaterOrEqual":
      return num(value) >= num(target);
    case "lessOrEqual":
      return num(value) <= num(target);
    case "between":
      return num(value) >= num(target) && num(value) <= num(target2);
    case "isBlank":
      return value == null || str(value).trim() === "";
    case "isError":
      return value instanceof Error || str(value).startsWith("#");
    case "duplicate":
      return null;
    case "unique":
      return null;
    case "top10":
      return null;
    case "bottom10":
      return null;
    case "aboveAverage":
      return null;
    case "belowAverage":
      return null;
    default:
      return false;
  }
}

function buildStyleRule(rule) {
  return {
    background: rule.background || null,
    foreground: rule.foreground || null,
    bold: rule.bold || false,
    italic: rule.italic || false,
    underline: rule.underline || false,
    fontSize: rule.fontSize || null,
  };
}

function applyFormatting(ranges, rules) {
  if (!rules || rules.length === 0) return [];
  const highlights = [];
  for (const rule of rules) {
    for (const range of ranges) {
      range.cells.forEach((value, key) => {
        const [r, c] = key.split(",").map(Number);
        if (evaluateRule(rule, value)) {
          highlights.push({
            row: r,
            col: c,
            style: buildStyleRule(rule),
            ruleId: rule.id,
            priority: rule.priority || 0,
          });
        }
      });
    }
  }
  highlights.sort((a, b) => b.priority - a.priority);
  return highlights;
}

function colorScale(value, min, max, colors) {
  if (max === min || value == null) return colors ? colors[Math.floor(colors.length / 2)] : null;
  const ratio = (value - min) / (max - min);
  const idx = Math.min(colors.length - 1, Math.floor(ratio * colors.length));
  return colors[idx];
}

function dataBar(value, min, max, color, maxWidth) {
  if (max === min || value == null) return 0;
  const ratio = (value - min) / (max - min);
  return Math.max(0, Math.min(maxWidth || 100, Math.round(ratio * (maxWidth || 100))));
}

function iconSet(value, thresholds, icons) {
  if (value == null) return icons[0];
  for (let i = thresholds.length - 1; i >= 0; i--) {
    if (value >= thresholds[i]) {
      return icons[Math.min(i, icons.length - 1)];
    }
  }
  return icons[0];
}

function topN(values, n, descending) {
  const sorted = values.slice().sort((a, b) => (descending ? b - a : a - b));
  return new Set(sorted.slice(0, n));
}

function formatValue(value, format) {
  if (value == null) return "";
  switch (format) {
    case "currency":
      return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(value);
    case "percent":
      return (value * 100).toFixed(2) + "%";
    case "number":
      return new Intl.NumberFormat("pt-BR").format(value);
    case "date":
      return new Date(value).toLocaleDateString("pt-BR");
    case "time":
      return new Date(value).toLocaleTimeString("pt-BR");
    case "datetime":
      return new Date(value).toLocaleString("pt-BR");
    case "uppercase":
      return String(value).toUpperCase();
    case "lowercase":
      return String(value).toLowerCase();
    default:
      return value;
  }
}

function applyFormattingRules(range, rules) {
  if (!range || !range.cells) return [];
  const out = [];
  for (const rule of rules || []) {
    for (const [key, value] of range.cells.entries()) {
      if (evaluateRule(rule, value)) {
        out.push({
          cell: key,
          style: buildStyleRule(rule),
          formattedValue: rule.format ? formatValue(value, rule.format) : null,
        });
      }
    }
  }
  return out;
}

window.SheetFormatting = {
  evaluateRule,
  buildStyleRule,
  applyFormatting,
  colorScale,
  dataBar,
  iconSet,
  topN,
  formatValue,
  applyFormattingRules,
};
