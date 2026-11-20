"use strict";
/* Sheets ingest (#2494) — cell provenance accessors, column mapping, bank
 * profile detection and the running-balance gate. Pure logic: no DOM, no
 * persistence side effects. */

(function () {
  var C = window.SheetIngestCore;
  var TYPES = ["text", "number", "date", "currency"];

  function colName(index) {
    var value = Math.max(0, Number(index) || 0) + 1;
    var out = "";
    while (value > 0) {
      var remainder = (value - 1) % 26;
      out = String.fromCharCode(65 + remainder) + out;
      value = Math.floor((value - 1) / 26);
    }
    return out;
  }

  function cellLabel(parse, row, col) {
    if (!parse || !parse.rows[row]) return "?";
    return colName(col) + String(row + 1);
  }

  function cellAt(parse, row, col) {
    if (!parse || !Array.isArray(parse.rows) || !parse.rows[row]) return null;
    return parse.rows[row][col] || null;
  }

  function isLow(cellValue) {
    return Boolean(cellValue) && (!Number.isFinite(Number(cellValue.confidence)) || Number(cellValue.confidence) < C.THRESHOLD);
  }

  function isUnreadable(cellValue) {
    return Boolean(cellValue) && Boolean(cellValue.unreadable);
  }

  function cellKey(fileId, row, col) {
    return String(fileId) + ":" + row + ":" + col;
  }

  function overrideFor(current, fileId, row, col) {
    var key = cellKey(fileId, row, col);
    for (var i = 0; i < current.overrides.length; i++) {
      if (current.overrides[i].key === key) return current.overrides[i];
    }
    return null;
  }

  function isConfirmed(current, fileId, row, col) {
    return Boolean(current.confirmations[cellKey(fileId, row, col)]);
  }

  function isResolved(current, fileId, row, col) {
    return isConfirmed(current, fileId, row, col) || Boolean(overrideFor(current, fileId, row, col));
  }

  function effectiveValue(current, fileId, row, col) {
    var corrected = overrideFor(current, fileId, row, col);
    if (corrected) return String(corrected.corrected);
    var cellValue = cellAt(current.parses[fileId], row, col);
    return !cellValue || isUnreadable(cellValue) ? "" : String(cellValue.value);
  }

  function formulaBlocked(current, fileId, row, col) {
    return isLow(cellAt(current.parses[fileId], row, col)) &&
      !isResolved(current, fileId, row, col);
  }

  function toNumber(value) {
    var cleaned = String(value == null ? "" : value).replace(/[^0-9.,-]/g, "").replace(/[.,](?=\d{3}(?:[.,]|$))/g, "").replace(/,(?=\d{1,2}$)/g, ".").replace(/,/g, "");
    if (!cleaned || cleaned === "-") return null;
    var parsed = Number(cleaned);
    return Number.isFinite(parsed) ? parsed : null;
  }

  function columnTypes(current, fileId) {
    var parse = current.parses[fileId];
    if (!parse) return [];
    var saved = current.mappings[fileId];
    return parse.columns.map(function (column, index) {
      return C.oneOf(saved && saved[index], TYPES, column.target);
    });
  }

  function detectProfile(parse) {
    if (!parse || !Array.isArray(parse.columns)) return { kind: "generic", columns: {} };
    var names = parse.columns.map(function (column) {
      return String(column.header || "").toLowerCase();
    });
    function find(patterns) {
      for (var i = 0; i < patterns.length; i++) {
        var index = -1;
        for (var j = 0; j < names.length; j++) {
          if (patterns[i].test(names[j])) { index = j; break; }
        }
        if (index >= 0) return index;
      }
      return -1;
    }
    var found = {
      date: find([/date/, /data/]),
      description: find([/description/, /detail/, /descri/]),
      debit: find([/debit/, /withdraw/, /saida/]),
      credit: find([/credit/, /deposit/, /entrada/]),
      balance: find([/balance/, /saldo/]),
    };
    var kind = found.date >= 0 && found.description >= 0 && found.balance >= 0 &&
      (found.debit >= 0 || found.credit >= 0) ? "bank" : "generic";
    return { kind: kind, columns: found };
  }

  function isDataRow(parse, row) {
    if (!parse || !parse.rows[row] || row === 0) return false;
    for (var i = 0; i < parse.structures.length; i++) {
      var item = parse.structures[i];
      if (item.row === row && (item.kind === "subtotal" || item.kind === "repeated-header")) return false;
    }
    return true;
  }

  function runningBalance(current, fileId) {
    var parse = current.parses[fileId];
    var profile = detectProfile(parse);
    var out = { kind: profile.kind, rows: [], mismatch: 0, blocked: 0, available: 0 };
    if (profile.kind !== "bank" || profile.columns.balance < 0) return out;
    var balanceCol = profile.columns.balance;
    var debitCol = profile.columns.debit;
    var creditCol = profile.columns.credit;
    var running = 0;
    var seeded = false;
    for (var row = 1; row < parse.rows.length; row++) {
      if (!isDataRow(parse, row)) continue;
      var cols = [balanceCol, debitCol, creditCol].filter(function (col) { return col >= 0; });
      var blocked = cols.some(function (col) { return formulaBlocked(current, fileId, row, col); });
      var stated = toNumber(effectiveValue(current, fileId, row, balanceCol));
      var debit = debitCol >= 0 ? toNumber(effectiveValue(current, fileId, row, debitCol)) : null;
      var credit = creditCol >= 0 ? toNumber(effectiveValue(current, fileId, row, creditCol)) : null;
      var statedPresent = stated !== null;
      if (statedPresent && !seeded) {
        running = stated;
        seeded = true;
      } else if (seeded) {
        running += (credit || 0) - (debit || 0);
      }
      if (!seeded) continue;
      if (statedPresent || debit !== null || credit !== null) out.available += 1;
      if (blocked) {
        out.blocked += 1;
        out.rows.push({ row: row, blocked: true, expected: null, stated: stated });
      } else if (statedPresent && Math.abs(stated - running) > 0.01) {
        out.mismatch += 1;
        out.rows.push({ row: row, blocked: false, expected: running, stated: stated });
      }
    }
    return out;
  }

  var api = {
    TYPES: TYPES,
    colName: colName,
    cellLabel: cellLabel,
    cellAt: cellAt,
    isLow: isLow,
    isUnreadable: isUnreadable,
    cellKey: cellKey,
    overrideFor: overrideFor,
    isConfirmed: isConfirmed,
    isResolved: isResolved,
    effectiveValue: effectiveValue,
    formulaBlocked: formulaBlocked,
    toNumber: toNumber,
    columnTypes: columnTypes,
    detectProfile: detectProfile,
    isDataRow: isDataRow,
    runningBalance: runningBalance,
  };
  Object.keys(api).forEach(function (key) { C[key] = api[key]; });
})();
