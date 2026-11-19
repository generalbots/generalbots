"use strict";
/* Sheet advanced module: 25_audit_graph — local-only formula dependency model (#2491).
   Pure helpers: no DOM, no network, never mutates workbook state. The graph is
   rebuilt from the cells already held in memory; it is NOT the server graph. */

(function () {
  var MAX_RANGE_CELLS = 4000;
  var MAX_GRAPH_CELLS = 250000;
  var MAX_CYCLES = 40;
  var CYCLE_BUDGET = 200000;

  function asObject(value) {
    return value && typeof value === "object" ? value : {};
  }
  function asArray(value) {
    return Array.isArray(value) ? value : [];
  }
  function text(value) {
    return value == null ? "" : String(value);
  }
  function colIdx(letters) {
    var name = text(letters).toUpperCase().replace(/\$/g, "");
    var n = 0;
    for (var i = 0; i < name.length; i++) {
      var code = name.charCodeAt(i);
      if (code < 65 || code > 90) break;
      n = n * 26 + (code - 64);
    }
    return n - 1;
  }
  function colName(idx) {
    var n = Math.max(0, idx) + 1;
    var out = "";
    while (n > 0) {
      var rem = (n - 1) % 26;
      out = String.fromCharCode(65 + rem) + out;
      n = Math.floor((n - 1) / 26);
    }
    return out;
  }
  function cellId(sheetIdx, row, col) {
    return sheetIdx + ":" + row + "," + col;
  }
  function parseId(id) {
    var parts = text(id).split(":");
    var xy = parts[1] ? parts[1].split(",") : [];
    return {
      sheetIdx: parseInt(parts[0], 10),
      row: parseInt(xy[0], 10),
      col: parseInt(xy[1], 10),
    };
  }
  function sheetIndex(sheets, name) {
    if (!name) return -1;
    var wanted = text(name).toLowerCase();
    for (var i = 0; i < sheets.length; i++) {
      if (text(sheets[i].name).toLowerCase() === wanted) return i;
    }
    return -1;
  }
  function refOf(sheetIdx, row, col, sheets) {
    var ref = colName(col) + (row + 1);
    var ws = sheets[sheetIdx];
    var name = ws ? text(ws.name) : "";
    return name ? name + "!" + ref : ref;
  }
  function parseRef(value, sheets) {
    var raw = text(value).trim().replace(/\$/g, "");
    var match = /^(?:(?:'((?:[^']|'')+)'|([A-Za-z_][A-Za-z0-9_. -]*))!)?([A-Za-z]{1,3})([1-9][0-9]{0,6})$/.exec(raw);
    if (!match) return null;
    var name = match[1] ? match[1].replace(/''/g, "'") : (match[2] || "");
    var row = parseInt(match[4], 10) - 1;
    var col = colIdx(match[3]);
    if (row < 0 || col < 0) return null;
    var index = name ? sheetIndex(sheets, name) : 0;
    return { sheetName: name, sheetIdx: index, row: row, col: col, ref: colName(col) + (row + 1) };
  }

  // Mask quoted text so references inside string literals are not followed.
  function maskStrings(formula) {
    var out = text(formula).split("");
    var quoted = false;
    for (var i = 0; i < out.length; i++) {
      if (quoted) {
        if (out[i] === '"') {
          if (out[i + 1] === '"') { out[++i] = " "; }
          else quoted = false;
        }
        out[i] = " ";
        continue;
      }
      if (out[i] === '"') { quoted = true; out[i] = " "; }
    }
    return out.join("");
  }

  var REF_RE = /((?:'(?:[^']|'')+'|[A-Za-z_][A-Za-z0-9_. ]*)!)?(\$?[A-Za-z]{1,3}\$?[1-9][0-9]{0,6})(?::(\$?[A-Za-z]{1,3}\$?[1-9][0-9]{0,6}))?/g;

  function extractRefs(formula) {
    var src = maskStrings(formula);
    var found = [];
    var seen = {};
    REF_RE.lastIndex = 0;
    var match;
    while ((match = REF_RE.exec(src)) !== null) {
      var before = match.index > 0 ? src.charAt(match.index - 1) : "";
      var after = src.charAt(REF_RE.lastIndex);
      if (/[A-Za-z0-9_.$]/.test(before)) continue;
      if (after === "(" || /[A-Za-z0-9_.$]/.test(after)) continue;
      var prefix = match[1] || "";
      var name = prefix ? prefix.slice(0, -1).replace(/^'|'$/g, "").replace(/''/g, "'") : "";
      var first = match[2].replace(/\$/g, "");
      var cellMatch = /^([A-Za-z]+)([1-9][0-9]*)$/.exec(first);
      if (!cellMatch) continue;
      var endMatch = match[3] ? /^([A-Za-z]+)([1-9][0-9]*)$/.exec(match[3].replace(/\$/g, "")) : null;
      var item = {
        sheetName: name,
        row: parseInt(cellMatch[2], 10) - 1,
        col: colIdx(cellMatch[1]),
        endRow: endMatch ? parseInt(endMatch[2], 10) - 1 : -1,
        endCol: endMatch ? colIdx(endMatch[1]) : -1,
        text: match[0],
      };
      if (item.row < 0 || item.col < 0) continue;
      var key = name + "|" + item.row + "," + item.col + "|" + item.endRow + "," + item.endCol;
      if (seen[key]) continue;
      seen[key] = true;
      found.push(item);
    }
    return found;
  }

  function iterativeFlag(book) {
    var raw = asObject(book);
    var calculation = asObject(raw.calculation || raw.calc || raw.settings);
    var keys = ["iterate", "iterative", "iterativeCalculation", "iterative_calculation", "enableIterative"];
    for (var i = 0; i < keys.length; i++) {
      if (calculation[keys[i]] === true || raw[keys[i]] === true) return true;
    }
    return false;
  }

  function normalizeSnapshot(raw, currentIndex) {
    var book = asObject(raw);
    var sheets = asArray(book.worksheets).map(function (ws, index) {
      var source = asObject(ws && ws.data);
      var data = {};
      Object.keys(source).forEach(function (key) {
        var cell = asObject(source[key]);
        data[key] = { value: cell.value, formula: cell.formula, comment: cell.comment };
      });
      return { index: index, name: text(ws && ws.name), data: data };
    });
    if (!sheets.length) sheets.push({ index: 0, name: "", data: {} });
    var idx = Number.isInteger(currentIndex) ? Math.min(Math.max(currentIndex, 0), sheets.length - 1) : 0;
    // The visible grid holds the freshest local values: fold a read-only copy
    // over the persisted snapshot so edits made this session are audited too.
    var D = window.SheetDefaults;
    var inst = D && D.getActiveSheetInstance ? D.getActiveSheetInstance() : null;
    var grid = (inst && inst.grid) || window.SheetVirtualGrid || null;
    if (grid && grid.cells && typeof grid.cells.forEach === "function") {
      grid.cells.forEach(function (cell, key) {
        if (!/^-?\d+,-?\d+$/.test(text(key))) return;
        var copy = asObject(cell);
        sheets[idx].data[key] = { value: copy.value, formula: copy.formula, comment: copy.comment };
      });
    }
    return { worksheets: sheets, sheetIndex: idx, iterative: iterativeFlag(book) };
  }

  function buildGraph(snapshot) {
    var snap = snapshot || { worksheets: [], sheetIndex: 0, iterative: false };
    var sheets = asArray(snap.worksheets);
    var cells = {};
    var precedents = {};
    var dependents = {};
    var count = 0;
    var truncated = false;
    for (var s = 0; s < sheets.length; s++) {
      var ws = sheets[s];
      var data = asObject(ws && ws.data);
      var keys = Object.keys(data);
      for (var k = 0; k < keys.length; k++) {
        var xy = /^(-?\d+),(-?\d+)$/.exec(keys[k]);
        if (!xy) continue;
        var row = parseInt(xy[1], 10);
        var col = parseInt(xy[2], 10);
        if (row < 0 || col < 0) continue;
        var id = cellId(s, row, col);
        var source = asObject(data[keys[k]]);
        cells[id] = {
          id: id, sheetIdx: s, sheetName: text(ws.name), row: row, col: col,
          ref: refOf(s, row, col, sheets),
          value: source.value, formula: text(source.formula),
          comment: text(source.comment), missing: false,
        };
        count++;
        if (count >= MAX_GRAPH_CELLS) { truncated = true; break; }
      }
      if (truncated) break;
    }

    var links = 0;
    function link(from, to) {
      if (!precedents[to]) precedents[to] = [];
      if (!dependents[from]) dependents[from] = [];
      if (precedents[to].indexOf(from) < 0) {
        precedents[to].push(from);
        dependents[from].push(to);
        links++;
      }
    }
    for (var id in cells) {
      var cell = cells[id];
      if (!cell.formula) continue;
      var refs = extractRefs(cell.formula);
      for (var r = 0; r < refs.length; r++) {
        var item = refs[r];
        var targetSheet = item.sheetName ? sheetIndex(sheets, item.sheetName) : cell.sheetIdx;
        if (targetSheet < 0) continue;
        var maxRow = item.endRow >= 0 ? item.endRow : item.row;
        var maxCol = item.endCol >= 0 ? item.endCol : item.col;
        var span = (maxRow - item.row + 1) * (maxCol - item.col + 1);
        if (span > MAX_RANGE_CELLS) { truncated = true; span = MAX_RANGE_CELLS; }
        var emitted = 0;
        for (var rr = item.row; rr <= maxRow && emitted < span; rr++) {
          for (var cc = item.col; cc <= maxCol && emitted < span; cc++) {
            var targetId = cellId(targetSheet, rr, cc);
            if (!cells[targetId]) {
              cells[targetId] = {
                id: targetId, sheetIdx: targetSheet, sheetName: text(sheets[targetSheet] && sheets[targetSheet].name),
                row: rr, col: cc, ref: refOf(targetSheet, rr, cc, sheets),
                value: "", formula: "", comment: "", missing: true,
              };
            }
            link(targetId, id);
            emitted++;
          }
        }
      }
      if (truncated && count >= MAX_GRAPH_CELLS) break;
    }
    return {
      sheets: sheets, cells: cells, precedents: precedents, dependents: dependents,
      links: links, formulaCount: Object.keys(cells).filter(function (key) { return !!cells[key].formula; }).length,
      iterative: !!snap.iterative, truncated: truncated,
    };
  }


  window.__SheetAuditGraphInternals = {
    asObject: asObject, asArray: asArray, text: text, colIdx: colIdx, colName: colName,
    cellId: cellId, parseId: parseId, sheetIndex: sheetIndex, refOf: refOf,
    parseRef: parseRef, extractRefs: extractRefs, iterativeFlag: iterativeFlag,
    normalizeSnapshot: normalizeSnapshot, buildGraph: buildGraph,
    MAX_GRAPH_CELLS: MAX_GRAPH_CELLS, MAX_CYCLES: MAX_CYCLES, CYCLE_BUDGET: CYCLE_BUDGET,
  };

  window.SheetAuditGraph = {
    colIdx: colIdx,
    colName: colName,
    cellId: cellId,
    parseId: parseId,
    parseRef: parseRef,
    refOf: refOf,
    sheetIndex: sheetIndex,
    extractRefs: extractRefs,
    normalizeSnapshot: normalizeSnapshot,
    buildGraph: buildGraph,
    iterativeFlag: iterativeFlag,
  };
})();
