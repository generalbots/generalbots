// botui/ui/suite/sheet/modules/27_named_ranges.js
// Named ranges manager: assign a name (e.g. "revenue") to a cell range
// (e.g. "B2:B20"). Persists to localStorage keyed by document name.
//
// API:
//   window.SheetNamedRanges.add(name, range, description?)
//   window.SheetNamedRanges.remove(name)
//   window.SheetNamedRanges.update(name, range, description?)
//   window.SheetNamedRanges.get(name) -> { name, range, description } | null
//   window.SheetNamedRanges.list() -> [ {name, range, description}, ... ]
//   window.SheetNamedRanges.resolve(name) -> { start: {row,col}, end: {row,col} } | null
//   window.SheetNamedRanges.isValidName(name) -> bool
//   window.SheetNamedRanges.exportToCSV() -> string
//   window.SheetNamedRanges.importFromCSV(csv) -> { added, updated, errors }
//
// The 24_formula_engine can later call .resolve(name) to translate
// named-range identifiers into range nodes when it encounters an
// unknown identifier in a formula. For now, the manager is decoupled
// from the engine — it provides the storage and validation only.
//
// Storage key: "gb.sheet.namedRanges.{documentName}"
// Excel-compatible name rules: must start with letter or underscore,
// can contain letters/digits/underscores/periods (no spaces, no
// reserved words like TRUE/FALSE/NULL/etc).
"use strict";

(function () {
  const STORAGE_PREFIX = "gb.sheet.namedRanges.";
  const RESERVED = {
    TRUE: 1, FALSE: 1, NULL: 1,
    IF: 1, AND: 1, OR: 1, NOT: 1,
    SUM: 1, AVERAGE: 1, COUNT: 1, MIN: 1, MAX: 1,
    VLOOKUP: 1, HLOOKUP: 1, INDEX: 1, MATCH: 1,
    ABS: 1, ROUND: 1, INT: 1, MOD: 1,
    CONCAT: 1, CONCATENATE: 1, LEFT: 1, RIGHT: 1, MID: 1, LEN: 1,
    DATE: 1, NOW: 1, TODAY: 1, YEAR: 1, MONTH: 1, DAY: 1,
    PI: 1, SQRT: 1, POWER: 1, EXP: 1, LN: 1, LOG: 1,
    ISERROR: 1, IFERROR: 1, ISBLANK: 1, ISNUMBER: 1, ISTEXT: 1,
  };

  // Standard Excel cell-reference pattern: A1, $A$1, A1:B10, Sheet1!A1, etc.
  const RANGE_PATTERN = /^\$?([A-Za-z]+)\$?(\d+)(?::\$?([A-Za-z]+)\$?(\d+))?$/;

  function colToNum(col) {
    let n = 0;
    for (let i = 0; i < col.length; i++) {
      n = n * 26 + (col.charCodeAt(i) - 64);
    }
    return n - 1;
  }

  function numToCol(n) {
    let s = "";
    n = n + 1;
    while (n > 0) {
      const r = (n - 1) % 26;
      s = String.fromCharCode(65 + r) + s;
      n = Math.floor((n - 1) / 26);
    }
    return s;
  }

  function getDocumentName() {
    const el = document.getElementById("sheetName");
    if (el && el.value) return el.value;
    return "default";
  }

  function getStorageKey() {
    return STORAGE_PREFIX + getDocumentName();
  }

  function load() {
    try {
      const raw = localStorage.getItem(getStorageKey());
      if (!raw) return [];
      const parsed = JSON.parse(raw);
      return Array.isArray(parsed) ? parsed : [];
    } catch (e) {
      return [];
    }
  }

  function save(ranges) {
    try {
      localStorage.setItem(getStorageKey(), JSON.stringify(ranges));
      return true;
    } catch (e) {
      return false;
    }
  }

  function isValidName(name) {
    if (typeof name !== "string") return false;
    if (name.length === 0 || name.length > 255) return false;
    if (!/^[A-Za-z_][A-Za-z0-9_.]*$/.test(name)) return false;
    if (RESERVED[name.toUpperCase()]) return false;
    return true;
  }

  function isValidRange(range) {
    if (typeof range !== "string") return false;
    const m = range.match(RANGE_PATTERN);
    if (!m) return false;
    return true;
  }

  function resolve(range) {
    const m = range.match(RANGE_PATTERN);
    if (!m) return null;
    const startCol = colToNum(m[1].toUpperCase());
    const startRow = parseInt(m[2], 10) - 1;
    if (m[3] && m[4]) {
      const endCol = colToNum(m[3].toUpperCase());
      const endRow = parseInt(m[4], 10) - 1;
      return {
        start: { row: Math.min(startRow, endRow), col: Math.min(startCol, endCol) },
        end: { row: Math.max(startRow, endRow), col: Math.max(startCol, endCol) },
      };
    }
    return {
      start: { row: startRow, col: startCol },
      end: { row: startRow, col: startCol },
    };
  }

  function get(name) {
    const ranges = load();
    for (let i = 0; i < ranges.length; i++) {
      if (ranges[i].name.toLowerCase() === String(name).toLowerCase()) {
        return ranges[i];
      }
    }
    return null;
  }

  function list() {
    return load().slice();
  }

  function add(name, range, description) {
    if (!isValidName(name)) {
      return { ok: false, error: "Invalid name. Must start with a letter or underscore, contain only letters/digits/underscores/periods, and not be a reserved word." };
    }
    if (!isValidRange(range)) {
      return { ok: false, error: "Invalid range. Use Excel A1 notation, e.g. A1 or A1:B10." };
    }
    const ranges = load();
    const existing = get(name);
    const entry = {
      name: name,
      range: range.toUpperCase(),
      description: typeof description === "string" ? description : "",
      addedAt: existing ? existing.addedAt : Date.now(),
      updatedAt: Date.now(),
    };
    if (existing) {
      const idx = ranges.findIndex(function (r) {
        return r.name.toLowerCase() === name.toLowerCase();
      });
      ranges[idx] = entry;
      save(ranges);
      return { ok: true, updated: true, entry: entry };
    }
    ranges.push(entry);
    save(ranges);
    return { ok: true, added: true, entry: entry };
  }

  function update(name, range, description) {
    const ranges = load();
    const idx = ranges.findIndex(function (r) {
      return r.name.toLowerCase() === String(name).toLowerCase();
    });
    if (idx < 0) return { ok: false, error: "Name not found" };
    if (range !== undefined && !isValidRange(range)) {
      return { ok: false, error: "Invalid range" };
    }
    if (range !== undefined) ranges[idx].range = range.toUpperCase();
    if (description !== undefined) ranges[idx].description = description;
    ranges[idx].updatedAt = Date.now();
    save(ranges);
    return { ok: true, entry: ranges[idx] };
  }

  function remove(name) {
    const ranges = load();
    const idx = ranges.findIndex(function (r) {
      return r.name.toLowerCase() === String(name).toLowerCase();
    });
    if (idx < 0) return { ok: false, error: "Name not found" };
    const removed = ranges.splice(idx, 1)[0];
    save(ranges);
    return { ok: true, removed: removed };
  }

  function clear() {
    save([]);
  }

  function exportToCSV() {
    const ranges = load();
    const lines = ["name,range,description"];
    for (let i = 0; i < ranges.length; i++) {
      const r = ranges[i];
      const desc = (r.description || "").replace(/"/g, '""');
      lines.push(r.name + "," + r.range + ',"' + desc + '"');
    }
    return lines.join("\n");
  }

  function importFromCSV(csv) {
    if (typeof csv !== "string") return { added: 0, updated: 0, errors: ["invalid input"] };
    const lines = csv.split(/\r?\n/);
    let added = 0;
    let updated = 0;
    const errors = [];
    for (let i = 1; i < lines.length; i++) {
      const line = lines[i].trim();
      if (!line) continue;
      const m = line.match(/^([^,]+),([^,]+)(?:,(.*))?$/);
      if (!m) {
        errors.push("Line " + (i + 1) + ": could not parse");
        continue;
      }
      const name = m[1].trim();
      const range = m[2].trim();
      const desc = m[3] ? m[3].replace(/^"|"$/g, "").replace(/""/g, '"') : "";
      const wasExisting = !!get(name);
      const result = add(name, range, desc);
      if (result.ok) {
        if (wasExisting) updated++;
        else added++;
      } else {
        errors.push("Line " + (i + 1) + ": " + result.error);
      }
    }
    return { added: added, updated: updated, errors: errors };
  }

  // UI: render the manager into a modal/dropdown
  function renderList(container) {
    if (!container) return;
    const ranges = list();
    if (ranges.length === 0) {
      container.innerHTML =
        '<p style="text-align:center;color:#888;padding:20px;">No named ranges defined. Use the form above to add one.</p>';
      return;
    }
    let html = '<table class="named-ranges-table"><thead><tr><th>Name</th><th>Range</th><th>Description</th><th></th></tr></thead><tbody>';
    for (let i = 0; i < ranges.length; i++) {
      const r = ranges[i];
      html +=
        '<tr>' +
        '<td><code>' + escapeHtml(r.name) + '</code></td>' +
        '<td><code>' + escapeHtml(r.range) + '</code></td>' +
        '<td>' + escapeHtml(r.description || "") + '</td>' +
        '<td><button class="btn-icon-small named-range-del" data-name="' + escapeHtml(r.name) + '" title="Delete">×</button></td>' +
        '</tr>';
    }
    html += '</tbody></table>';
    container.innerHTML = html;
    const dels = container.querySelectorAll(".named-range-del");
    for (let i = 0; i < dels.length; i++) {
      dels[i].addEventListener("click", function (e) {
        const n = e.currentTarget.getAttribute("data-name");
        const result = remove(n);
        if (result.ok) renderList(container);
      });
    }
  }

  function escapeHtml(s) {
    if (s === null || s === undefined) return "";
    return String(s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  window.SheetNamedRanges = {
    add: add,
    update: update,
    remove: remove,
    get: get,
    list: list,
    clear: clear,
    resolve: resolve,
    isValidName: isValidName,
    isValidRange: isValidRange,
    exportToCSV: exportToCSV,
    importFromCSV: importFromCSV,
    renderList: renderList,
  };
})();
