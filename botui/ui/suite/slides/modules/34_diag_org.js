"use strict";
/* Slides diagram authoring — data-driven org chart from local rows (#2488).
 * Accepts a pasted sheet range or CSV file (name, title, reports-to). Parsing is
 * local only: no backend fetch, no CRM/Sheet binding until the compile gate. */

var SDOrg = {
  separatorOf: function (line) {
    if (line.indexOf("\t") >= 0) return "\t";
    if (line.indexOf(";") >= 0) return ";";
    if (line.indexOf(",") >= 0) return ",";
    return "|";
  },

  csvLine: function (line, separator) {
    var cells = [];
    var current = "";
    var quoted = false;
    for (var i = 0; i < line.length; i++) {
      var ch = line.charAt(i);
      if (ch === '"') {
        if (quoted && line.charAt(i + 1) === '"') { current += '"'; i++; }
        else quoted = !quoted;
      } else if (ch === separator && !quoted) {
        cells.push(current.trim());
        current = "";
      } else {
        current += ch;
      }
    }
    cells.push(current.trim());
    return cells;
  },

  splitLine: function (line) {
    return this.csvLine(line, this.separatorOf(line));
  },

  isHeader: function (cells, index) {
    if (index !== 0) return false;
    var first = String(cells[0] || "").toLowerCase();
    var third = String(cells[2] || "").toLowerCase();
    return /^(name|nome|nombre|nom|姓名|名前|이름)\b/.test(first) ||
      /reports?\s*(to|para|a|へ|에게)\b/.test(third);
  },

  parseRows: function (text) {
    var rows = [];
    var errors = [];
    var lines = String(text || "").split(/\r?\n/);
    lines.forEach(function (line, index) {
      if (!line.trim()) return;
      var cells = SDOrg.splitLine(line);
      if (SDOrg.isHeader(cells, index)) return;
      if (!cells[0]) { errors.push(index + 1); return; }
      rows.push({
        name: cells[0],
        title: cells[1] || "",
        reportsTo: cells[2] || ""
      });
    });
    return { rows: rows, errors: errors };
  },

  resolveParents: function (nodes) {
    var byName = {};
    nodes.forEach(function (node) {
      var key = String(node.name || "").toLowerCase().trim();
      if (key && !byName[key]) byName[key] = node;
    });
    nodes.forEach(function (node) {
      var boss = String(node.reportsTo || "").toLowerCase().trim();
      node.parentId = boss && byName[boss] && byName[boss].id !== node.id ? byName[boss].id : null;
    });
    nodes.forEach(function (node) {
      var seen = {};
      var cursor = node;
      while (cursor && cursor.parentId) {
        if (seen[cursor.id]) { node.parentId = null; break; }
        seen[cursor.id] = true;
        var found = null;
        nodes.forEach(function (candidate) {
          if (candidate.id === cursor.parentId) found = candidate;
        });
        cursor = found;
      }
    });
    return nodes;
  },

  build: function (text) {
    var parsed = this.parseRows(text);
    var diagram = SDPresets.create("org", Math.max(1, parsed.rows.length));
    diagram.nodes = parsed.rows.map(function (row, index) {
      var node = SDPresets.newNode(index, 0, null);
      node.text = row.name;
      node.title = row.title;
      node.name = row.name;
      node.reportsTo = row.reportsTo;
      return node;
    });
    this.resolveParents(diagram.nodes);
    SDPresets.reflow(diagram);
    return {
      diagram: diagram,
      errors: parsed.errors,
      count: parsed.rows.length
    };
  }
};
