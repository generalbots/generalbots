"use strict";
/* Sheet advanced module: 25a_audit_reports — local-only trace / cycle / error /
   formula-text / dependency-map reports built on the 25_audit_graph model. */
(function () {
  var I = window.__SheetAuditGraphInternals;
  var asObject = I.asObject;
  var asArray = I.asArray;
  var text = I.text;
  var colName = I.colName;
  var refOf = I.refOf;
  var MAX_GRAPH_CELLS = I.MAX_GRAPH_CELLS;
  var MAX_CYCLES = I.MAX_CYCLES;
  var CYCLE_BUDGET = I.CYCLE_BUDGET;

  function trace(graph, id, direction, maxLevels) {
    var g = asObject(graph);
    var edges = direction === "dependents" ? g.dependents : g.precedents;
    var map = edges || {};
    var limit = Math.min(Math.max(parseInt(maxLevels, 10) || 5, 1), 12);
    var queue = [{ id: id, level: 0 }];
    var seen = {};
    seen[id] = true;
    var nodes = [];
    var truncated = false;
    var levels = 0;
    while (queue.length) {
      var item = queue.shift();
      var cell = (g.cells || {})[item.id];
      levels = Math.max(levels, item.level);
      nodes.push({
        id: item.id, level: item.level, ref: cell ? cell.ref : item.id,
        sheetIdx: cell ? cell.sheetIdx : -1, sheetName: cell ? cell.sheetName : "",
        formula: cell ? cell.formula : "", value: cell ? cell.value : "",
        missing: !!(cell && cell.missing),
      });
      if (item.level >= limit) continue;
      var next = map[item.id] || [];
      for (var i = 0; i < next.length; i++) {
        var neighbor = next[i];
        if (seen[neighbor]) continue;
        seen[neighbor] = true;
        queue.push({ id: neighbor, level: item.level + 1 });
        if (nodes.length + queue.length > MAX_GRAPH_CELLS) { truncated = true; break; }
      }
      if (truncated) break;
    }
    return { nodes: nodes, levels: levels, truncated: truncated };
  }

  function canonCycle(ids) {
    var best = null;
    for (var i = 0; i < ids.length; i++) {
      var rotated = ids.slice(i).concat(ids.slice(0, i)).join(">");
      if (best === null || rotated < best) best = rotated;
    }
    return best;
  }

  function findCycles(graph) {
    var g = asObject(graph);
    var cells = g.cells || {};
    var edges = g.dependents || {};
    var color = {};
    var cycles = [];
    var keys = Object.keys(cells);
    var seenCycles = {};
    var steps = 0;
    var truncated = false;
    for (var rootIndex = 0; rootIndex < keys.length && !truncated; rootIndex++) {
      var root = keys[rootIndex];
      if (color[root]) continue;
      color[root] = 1;
      var path = [root];
      var stack = [{ id: root, next: (edges[root] || []).slice(), i: 0 }];
      while (stack.length) {
        var frame = stack[stack.length - 1];
        if (frame.i >= frame.next.length) {
          color[frame.id] = 2;
          stack.pop();
          path.pop();
          continue;
        }
        var neighbor = frame.next[frame.i++];
        if (!cells[neighbor]) continue;
        if (color[neighbor] === 1) {
          var at = path.indexOf(neighbor);
          if (at >= 0 && cycles.length < MAX_CYCLES) {
            var loop = path.slice(at);
            var canonical = canonCycle(loop);
            if (!seenCycles[canonical]) {
              seenCycles[canonical] = true;
              cycles.push({
                ids: loop.slice(),
                refs: loop.map(function (id) { return cells[id].ref; }),
              });
            }
          }
          if (cycles.length >= MAX_CYCLES) { truncated = true; break; }
        } else if (!color[neighbor]) {
          color[neighbor] = 1;
          path.push(neighbor);
          stack.push({ id: neighbor, next: (edges[neighbor] || []).slice(), i: 0 });
        }
        steps++;
        if (steps > CYCLE_BUDGET) { truncated = true; break; }
      }
    }
    return { cycles: cycles, iterative: !!g.iterative, truncated: truncated };
  }

  var ERROR_INFO = {
    "#DIV/0!": ["sheet-audit-cause-div0", "A divisor evaluates to zero.", "sheet-audit-fix-div0", "Check the denominator or guard the division with IF."],
    "#REF!": ["sheet-audit-cause-ref", "A referenced cell or range is missing.", "sheet-audit-fix-ref", "Restore the referenced cell or rewrite the formula."],
    "#NAME?": ["sheet-audit-cause-name", "A function or name is not recognised in local state.", "sheet-audit-fix-name", "Check spelling; only common functions are supported locally."],
    "#VALUE!": ["sheet-audit-cause-value", "An argument has the wrong type for its operator.", "sheet-audit-fix-value", "Convert the argument to the expected number or text."],
    "#N/A": ["sheet-audit-cause-na", "A lookup found no matching value.", "sheet-audit-fix-na", "Check the lookup value and its source range."],
    "#NUM!": ["sheet-audit-cause-num", "A number is outside the supported range.", "sheet-audit-fix-num", "Review the numeric inputs and their magnitude."],
    "#NULL!": ["sheet-audit-cause-null", "An invalid range operator was used.", "sheet-audit-fix-null", "Use a single colon between two valid references."],
    "#CYCLE!": ["sheet-audit-cause-cycle", "The cell is part of a circular reference.", "sheet-audit-fix-cycle", "Break one formula in the reported cycle."],
  };

  function inspectErrors(graph, evalIssues) {
    var g = asObject(graph);
    var cells = g.cells || {};
    var extra = asObject(evalIssues);
    var out = [];
    Object.keys(cells).forEach(function (id) {
      var cell = cells[id];
      var candidates = [];
      var value = text(cell.value).trim();
      if (/^#[A-Z/0-9?]+!$/.test(value) || value === "#N/A") candidates.push(value);
      if (/#REF!/.test(cell.formula)) candidates.push("#REF!");
      if (extra[id]) candidates.push(text(extra[id]));
      var unique = {};
      candidates.forEach(function (type) {
        if (unique[type]) return;
        unique[type] = true;
        var info = ERROR_INFO[type] || ["sheet-audit-cause-other", "The cell reports an error from its cached value.", "sheet-audit-fix-other", "Inspect the formula and its precedents."];
        out.push({
          id: id, ref: cell.ref, sheetIdx: cell.sheetIdx, sheetName: cell.sheetName,
          type: type, causeKey: info[0], causeFallback: info[1],
          fixKey: info[2], fixFallback: info[3],
        });
      });
    });
    return out;
  }

  function formulaAsText(snapshot, sheetIdx, range) {
    var snap = asObject(snapshot);
    var sheets = asArray(snap.worksheets);
    var index = Number.isInteger(sheetIdx) ? sheetIdx : 0;
    var ws = sheets[index];
    if (!ws) return { rows: [], truncated: false };
    var r = asObject(range);
    var minRow = Number.isInteger(r.startRow) ? r.startRow : 0;
    var maxRow = Number.isInteger(r.endRow) ? r.endRow : minRow;
    var minCol = Number.isInteger(r.startCol) ? r.startCol : 0;
    var maxCol = Number.isInteger(r.endCol) ? r.endCol : minCol;
    var rows = [];
    var truncated = false;
    var data = asObject(ws.data);
    Object.keys(data).sort().forEach(function (key) {
      var xy = /^(-?\d+),(-?\d+)$/.exec(key);
      if (!xy) return;
      var row = parseInt(xy[1], 10);
      var col = parseInt(xy[2], 10);
      if (row < minRow || row > maxRow || col < minCol || col > maxCol) return;
      var cell = asObject(data[key]);
      if (!cell.formula) return;
      if (rows.length >= 5000) { truncated = true; return; }
      rows.push({ ref: refOf(index, row, col, sheets), formula: text(cell.formula), value: cell.value });
    });
    return { rows: rows, truncated: truncated };
  }

  function dependencyMap(graph, sheetIdx) {
    var g = asObject(graph);
    var cells = g.cells || {};
    var index = Number.isInteger(sheetIdx) ? sheetIdx : 0;
    var rows = [];
    var links = 0;
    Object.keys(cells).forEach(function (id) {
      var cell = cells[id];
      if (cell.sheetIdx !== index || !cell.formula) return;
      var pre = (g.precedents || {})[id] || [];
      var dep = (g.dependents || {})[id] || [];
      links += pre.length;
      rows.push({ id: id, ref: cell.ref, precedents: pre.length, dependents: dep.length, formula: cell.formula });
    });
    rows.sort(function (a, b) { return b.dependents - a.dependents || b.precedents - a.precedents || a.ref.localeCompare(b.ref); });
    return { rows: rows, links: links, formulas: rows.length };
  }

  function columnFitHint(snapshot, sheetIdx, showFormulas) {
    var snap = asObject(snapshot);
    var sheets = asArray(snap.worksheets);
    var ws = sheets[Number.isInteger(sheetIdx) ? sheetIdx : 0];
    if (!ws) return [];
    var data = asObject(ws.data);
    var widths = {};
    Object.keys(data).forEach(function (key) {
      var xy = /^(-?\d+),(-?\d+)$/.exec(key);
      if (!xy) return;
      var col = parseInt(xy[2], 10);
      var cell = asObject(data[key]);
      var content = showFormulas && cell.formula ? text(cell.formula) : text(cell.value);
      var len = Math.max(content.length, showFormulas ? colName(col).length + 2 : 0);
      widths[col] = Math.max(widths[col] || 0, len);
    });
    return Object.keys(widths).map(function (key) {
      var col = parseInt(key, 10);
      return { col: col, letter: colName(col), width: Math.min(260, Math.max(64, widths[col] * 7 + 18)) };
    }).sort(function (a, b) { return a.col - b.col; });
  }


  Object.assign(window.SheetAuditGraph, {
    trace: trace,
    findCycles: findCycles,
    inspectErrors: inspectErrors,
    formulaAsText: formulaAsText,
    dependencyMap: dependencyMap,
    columnFitHint: columnFitHint,
  });
})();
