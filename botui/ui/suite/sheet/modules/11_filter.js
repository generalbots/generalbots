"use strict";

/**
 * Module 11: Filter engine for Sheet.
 * Provides: filter, sort, multi-criteria, regex match, date range, blank check.
 */

function buildPredicate(filter, cell) {
  if (!filter) return () => true;
  const op = filter.operator || "equals";
  const value = filter.value;
  const value2 = filter.value2;
  const num = (v) => (typeof v === "number" ? v : parseFloat(v));
  const str = (v) => (v == null ? "" : String(v));
  switch (op) {
    case "equals":
      return (c) => str(c) === str(value);
    case "notEquals":
      return (c) => str(c) !== str(value);
    case "contains":
      return (c) => str(c).toLowerCase().includes(str(value).toLowerCase());
    case "notContains":
      return (c) => !str(c).toLowerCase().includes(str(value).toLowerCase());
    case "startsWith":
      return (c) => str(c).toLowerCase().startsWith(str(value).toLowerCase());
    case "endsWith":
      return (c) => str(c).toLowerCase().endsWith(str(value).toLowerCase());
    case "greater":
      return (c) => num(c) > num(value);
    case "greaterOrEqual":
      return (c) => num(c) >= num(value);
    case "less":
      return (c) => num(c) < num(value);
    case "lessOrEqual":
      return (c) => num(c) <= num(value);
    case "between":
      return (c) => {
        const n = num(c);
        return n >= num(value) && n <= num(value2);
      };
    case "regex":
      try {
        const re = new RegExp(value, filter.regexFlags || "i");
        return (c) => re.test(str(c));
      } catch (_e) {
        return () => false;
      }
    case "isBlank":
      return (c) => c == null || str(c).trim() === "";
    case "isNotBlank":
      return (c) => c != null && str(c).trim() !== "";
    case "isTrue":
      return (c) => c === true || str(c).toLowerCase() === "true";
    case "isFalse":
      return (c) => c === false || str(c).toLowerCase() === "false";
    case "dateAfter":
      return (c) => new Date(c) > new Date(value);
    case "dateBefore":
      return (c) => new Date(c) < new Date(value);
    case "dateBetween":
      return (c) => {
        const d = new Date(c);
        return d >= new Date(value) && d <= new Date(value2);
      };
    default:
      return () => true;
  }
}

function applyFilter(rows, filters, columnIndex) {
  if (!filters || filters.length === 0) return rows.slice();
  const predicates = filters
    .filter((f) => !columnIndex || f.column === columnIndex)
    .map((f) => ({ col: f.column, fn: buildPredicate(f) }));
  return rows.filter((row) =>
    predicates.every((p) => {
      const v = row[p.col];
      return p.fn(v);
    })
  );
}

function applyMultiCriteria(rows, criteriaGroups) {
  if (!criteriaGroups || criteriaGroups.length === 0) return rows.slice();
  return rows.filter((row) =>
    criteriaGroups.every((group) => {
      if (group.operator === "OR") {
        return group.criteria.some((c) => buildPredicate(c)(row[c.column]));
      }
      return group.criteria.every((c) => buildPredicate(c)(row[c.column]));
    })
  );
}

function sortRows(rows, sortSpecs) {
  if (!sortSpecs || sortSpecs.length === 0) return rows.slice();
  return rows.slice().sort((a, b) => {
    for (const spec of sortSpecs) {
      const av = a[spec.column];
      const bv = b[spec.column];
      let cmp;
      if (typeof av === "number" && typeof bv === "number") {
        cmp = av - bv;
      } else {
        const ad = new Date(av);
        const bd = new Date(bv);
        if (!isNaN(ad.getTime()) && !isNaN(bd.getTime())) {
          cmp = ad.getTime() - bd.getTime();
        } else {
          cmp = String(av || "").localeCompare(String(bv || ""));
        }
      }
      if (cmp !== 0) return spec.descending ? -cmp : cmp;
    }
    return 0;
  });
}

function uniqueValues(rows, column) {
  const seen = new Set();
  const out = [];
  for (const row of rows) {
    const v = row[column];
    const key = v == null ? "__NULL__" : String(v);
    if (!seen.has(key)) {
      seen.add(key);
      out.push(v);
    }
  }
  return out;
}

function countBlanks(rows, column) {
  let n = 0;
  for (const row of rows) {
    const v = row[column];
    if (v == null || String(v).trim() === "") n++;
  }
  return n;
}

function quickFilter(rows, column, value) {
  if (value == null || value === "") return rows.slice();
  return rows.filter((r) => String(r[column] || "").toLowerCase().includes(String(value).toLowerCase()));
}

window.SheetFilter = {
  buildPredicate,
  applyFilter,
  applyMultiCriteria,
  sortRows,
  uniqueValues,
  countBlanks,
  quickFilter,
};
