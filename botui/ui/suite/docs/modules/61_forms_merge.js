"use strict";
/* Mail merge (#2427): a small CSV parser, column-to-field mapping, bounded
 * record navigation and a combined local preview. No Drive/CRM/sheet source,
 * no per-record DOCX/PDF generation and no server job are implemented. */
(function (window) {
  var NS = window.DocsForms;
  if (!NS) return;
  var M = (NS.merge = NS.merge || {});

  /* RFC-4180-ish: quoted cells, "" escapes, CRLF/LF, trailing newline. */
  M.parseCSV = function (text) {
    var source = String(text == null ? "" : text).replace(/\r\n?/g, "\n");
    var headers = [];
    var rows = [];
    var row = [];
    var cell = "";
    var quoted = false;
    var ended = false;
    for (var i = 0; i < source.length; i += 1) {
      var ch = source.charAt(i);
      if (quoted) {
        if (ch === '"') {
          if (source.charAt(i + 1) === '"') { cell += '"'; i += 1; }
          else quoted = false;
        } else cell += ch;
        continue;
      }
      if (ch === '"' && cell === "") { quoted = true; continue; }
      if (ch === ",") { row.push(cell); cell = ""; ended = false; continue; }
      if (ch === "\n") {
        row.push(cell);
        cell = "";
        if (row.some(function (part) { return part.trim() !== ""; }) || ended) rows.push(row);
        row = [];
        ended = false;
        continue;
      }
      cell += ch;
      ended = true;
    }
    if (cell !== "" || ended || row.length) {
      row.push(cell);
      if (row.some(function (part) { return part.trim() !== ""; })) rows.push(row);
    }
    if (rows.length) headers = rows.shift().map(function (part) { return part.trim(); });
    return { headers: headers, rows: rows };
  };

  M.toRecords = function (parsed) {
    var headers = (parsed && parsed.headers) || [];
    return ((parsed && parsed.rows) || []).map(function (row) {
      var record = {};
      headers.forEach(function (name, index) { record[name] = row[index] == null ? "" : row[index]; });
      return record;
    });
  };

  M.autoMap = function (headers, fields) {
    var mapping = {};
    (headers || []).forEach(function (header) {
      var needle = String(header).trim().toLowerCase();
      var match = (fields || []).filter(function (field) {
        return String(field.label || "").trim().toLowerCase() === needle;
      })[0];
      mapping[header] = match ? match.id : "";
    });
    return mapping;
  };

  M.previewRecord = function (records, index) {
    var list = records || [];
    if (!list.length) return { index: 0, record: null };
    var position = Math.max(0, Math.min(list.length - 1, index || 0));
    return { index: position, record: list[position] };
  };

  M.run = function (records, fields, mapping) {
    var list = records || [];
    var controls = fields || [];
    var map = mapping || {};
    var chunks = [];
    var missing = [];
    list.forEach(function (record, recordIndex) {
      var lines = [];
      controls.forEach(function (field) {
        var header = Object.keys(map).filter(function (name) { return map[name] === field.id; })[0];
        var value = header ? (record[header] || "") : "";
        if (field.required && String(value).trim() === "") {
          missing.push({ index: recordIndex, label: field.label || field.id });
        }
        lines.push(String(field.label || field.id) + ": " + value);
      });
      chunks.push(lines.join("\n"));
    });
    return { text: chunks.join("\n\n"), count: list.length, missing: missing };
  };

  M.load = function (text) {
    var parsed = M.parseCSV(text);
    var records = M.toRecords(parsed);
    NS.state.csvText = String(text == null ? "" : text);
    NS.state.recordIndex = 0;
    NS.state.mapping = M.autoMap(parsed.headers, NS.state.controls);
    NS.persist();
    return { parsed: parsed, records: records };
  };
})(window);
