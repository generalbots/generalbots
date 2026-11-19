"use strict";
/* Sheets data connectivity (#2492) - ordered, reproducible query steps.
 * window.SheetConnectSteps. Each step receives and returns a table with a
 * parallel lineage matrix so every output cell keeps its step + source field. */

(function () {
  var C = window.SheetConnectCore;
  var F = window.SheetConnectFeed;

  function columnsList(raw) {
    return String(raw == null ? "" : raw).split(",").map(function (item) { return item.trim(); })
      .filter(function (item) { return item.length > 0; }).slice(0, 30);
  }

  function indexOf(columns, name) {
    return columns.indexOf(name);
  }

  function rewrite(columns, rows, lineage) {
    return { columns: columns, rows: rows, lineage: lineage };
  }

  function selectStep(table, step) {
    var wanted = columnsList(step.config.columns);
    if (!wanted.length) return table;
    var indices = wanted.map(function (name) { return indexOf(table.columns, name); })
      .filter(function (index) { return index >= 0; });
    if (!indices.length) return table;
    return rewrite(
      indices.map(function (i) { return table.columns[i]; }),
      table.rows.map(function (row) { return indices.map(function (i) { return row[i]; }); }),
      table.lineage.map(function (line) { return indices.map(function (i) { return Object.assign({}, line[i]); }); }),
    );
  }

  function compare(a, b, op) {
    var left = String(a == null ? "" : a).toLowerCase();
    var right = String(b == null ? "" : b).toLowerCase();
    if (op === "eq") return left === right;
    if (op === "neq") return left !== right;
    if (op === "contains") return left.indexOf(right) >= 0;
    if (op === "gt") return Number(a) > Number(b) || (Number.isNaN(Number(a)) && left > right);
    if (op === "lt") return Number(a) < Number(b) || (Number.isNaN(Number(a)) && left < right);
    return false;
  }

  function filterStep(table, step, parameters) {
    var field = String(step.config.field || "").trim();
    var index = indexOf(table.columns, field);
    if (index < 0) return table;
    var op = C ? C.oneOf(step.config.op, ["eq", "neq", "contains", "gt", "lt"], "eq") : "eq";
    var value = F.applyParameters(step.config.value, parameters);
    var rows = [];
    var lineage = [];
    table.rows.forEach(function (row, position) {
      if (!compare(row[index], value, op)) return;
      rows.push(row.slice());
      lineage.push(table.lineage[position].map(function (cell) { return Object.assign({}, cell); }));
    });
    return rewrite(table.columns.slice(), rows, lineage);
  }

  function renameStep(table, step) {
    var from = String(step.config.from || "").trim();
    var to = String(step.config.to || "").trim();
    var index = indexOf(table.columns, from);
    if (index < 0 || !to) return table;
    var columns = table.columns.slice();
    columns[index] = to;
    var lineage = table.lineage.map(function (line) {
      return line.map(function (cell, position) {
        var next = Object.assign({}, cell);
        if (position === index) next.stepId = step.id;
        return next;
      });
    });
    return rewrite(columns, table.rows.map(function (row) { return row.slice(); }), lineage);
  }

  function coerce(value, kind) {
    if (kind === "number") {
      var parsed = Number(value);
      return Number.isFinite(parsed) ? parsed : value;
    }
    if (kind === "date") {
      var raw = String(value == null ? "" : value).trim();
      return /^\d{4}-\d{2}-\d{2}$/.test(raw) ? raw : value;
    }
    return String(value == null ? "" : value);
  }

  function typeStep(table, step) {
    var field = String(step.config.field || "").trim();
    var index = indexOf(table.columns, field);
    var kind = C ? C.oneOf(step.config.to, ["text", "number", "date"], "text") : "text";
    if (index < 0) return table;
    var rows = table.rows.map(function (row) {
      var copy = row.slice();
      copy[index] = coerce(copy[index], kind);
      return copy;
    });
    var lineage = table.lineage.map(function (line) {
      return line.map(function (cell, position) {
        var next = Object.assign({}, cell);
        if (position === index) next.stepId = step.id;
        return next;
      });
    });
    return rewrite(table.columns.slice(), rows, lineage);
  }

  function unpivotStep(table, step) {
    var selected = columnsList(step.config.columns);
    var indices = selected.map(function (name) { return indexOf(table.columns, name); })
      .filter(function (index) { return index >= 0; });
    if (!indices.length) return table;
    var keep = table.columns.map(function (name, position) { return { name: name, index: position }; })
      .filter(function (item) { return indices.indexOf(item.index) < 0; });
    var categoryHeader = String(step.config.categoryHeader || "series").trim();
    var valueHeader = String(step.config.valueHeader || "value").trim();
    var columns = keep.map(function (item) { return item.name; }).concat([categoryHeader, valueHeader]);
    var rows = [];
    var lineage = [];
    table.rows.forEach(function (row, position) {
      indices.forEach(function (sourceIndex) {
        var name = table.columns[sourceIndex];
        var base = keep.map(function (item) { return row[item.index]; });
        var baseLineage = keep.map(function (item) { return Object.assign({}, table.lineage[position][item.index]); });
        rows.push(base.concat([name, row[sourceIndex]]));
        lineage.push(baseLineage.concat([
          { stepId: step.id, stepType: "unpivot", sourceField: "(column)" },
          { stepId: step.id, stepType: "unpivot", sourceField: name },
        ]));
      });
    });
    return rewrite(columns, rows, lineage);
  }

  function groupStep(table, step) {
    var by = String(step.config.by || "").trim();
    var field = String(step.config.aggField || "").trim();
    var byIndex = indexOf(table.columns, by);
    var fieldIndex = indexOf(table.columns, field);
    if (byIndex < 0 || fieldIndex < 0) return table;
    var agg = C ? C.oneOf(step.config.agg, ["sum", "count"], "sum") : "sum";
    var groups = [];
    var lookup = {};
    table.rows.forEach(function (row, rowIndex) {
      var key = String(row[byIndex]);
      var bucket = lookup[key];
      if (!bucket) {
        bucket = { key: key, total: 0, count: 0, source: table.lineage[rowIndex] };
        lookup[key] = bucket;
        groups.push(bucket);
      }
      bucket.total += Number(row[fieldIndex]) || 0;
      bucket.count += 1;
    });
    var valueHeader = agg === "count" ? "count" : "total";
    var rows = groups.map(function (bucket) {
      return [bucket.key, agg === "count" ? bucket.count : Math.round(bucket.total * 100) / 100];
    });
    var lineage = groups.map(function (bucket) {
      var sourceLine = bucket.source || [];
      return [
        { stepId: step.id, stepType: "group", sourceField: by },
        Object.assign({}, sourceLine[fieldIndex] || { stepId: step.id, stepType: "group", sourceField: field },
          { stepId: step.id, stepType: "group" }),
      ];
    });
    return rewrite([by, valueHeader], rows, lineage);
  }

  function applyStep(table, step, parameters) {
    if (!table || !step || step.enabled === false) return table;
    var config = step.config || {};
    if (step.type === "select") return selectStep(table, { id: step.id, config: config });
    if (step.type === "filter") return filterStep(table, { id: step.id, config: config }, parameters);
    if (step.type === "rename") return renameStep(table, { id: step.id, config: config });
    if (step.type === "type") return typeStep(table, { id: step.id, config: config });
    if (step.type === "unpivot") return unpivotStep(table, { id: step.id, config: config });
    if (step.type === "group") return groupStep(table, { id: step.id, config: config });
    return table;
  }

  function applyAll(target, connector) {
    var table = F.simulatedFeed(connector);
    var applied = 0;
    (target.steps || []).forEach(function (step) {
      if (step.enabled === false) return;
      var next = applyStep(table, step, target.parameters);
      if (next !== table) applied += 1;
      table = next;
    });
    return { table: table, applied: applied };
  }

  function preview(target, connector, upTo) {
    var table = F.simulatedFeed(connector);
    var limit = typeof upTo === "number" ? Math.min(upTo, (target.steps || []).length) : (target.steps || []).length;
    for (var index = 0; index < limit; index += 1) {
      var step = target.steps[index];
      if (step.enabled === false) continue;
      table = applyStep(table, step, target.parameters);
    }
    return table;
  }

  window.SheetConnectSteps = {
    applyStep: applyStep,
    applyAll: applyAll,
    preview: preview,
    columnsList: columnsList,
    compare: compare,
  };
})();
