// botui/ui/suite/sheet/modules/27_named_ranges.js
// Named ranges manager — refactored to delegate persistence to
// botserver via window.SheetAPI. Client-side name/range validation
// remains (UX, no network for typos); authoritative storage is
// server-side (PostgreSQL via botsheet-core).
//
// Source of truth: botserver/crates/botsheet/src/handlers/crud.rs
//   - handle_create_named_range
//   - handle_update_named_range
//   - handle_list_named_ranges
//   - handle_delete_named_range
//
// API:
//   window.SheetNamedRanges.add(name, range, description?) -> Promise<{ok,...}>
//   window.SheetNamedRanges.update(name, range, description?) -> Promise
//   window.SheetNamedRanges.remove(name) -> Promise
//   window.SheetNamedRanges.get(name) -> Promise<{name,range,description}|null>
//   window.SheetNamedRanges.list() -> Promise<[entry,...]>
//   window.SheetNamedRanges.isValidName(name) -> bool  (sync, UX only)
//   window.SheetNamedRanges.isValidRange(range) -> bool  (sync, UX only)
//   window.SheetNamedRanges.resolve(range) -> {start,end}|null  (sync helper)
//   window.SheetNamedRanges.renderList(container, list)  (UI render)
//   window.SheetNamedRanges.exportToCSV() -> Promise<string>
//   window.SheetNamedRanges.importFromCSV(csv) -> Promise<{added,updated,errors}>
"use strict";

(function () {
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
  const RANGE_PATTERN = /^\$?([A-Za-z]+)\$?(\d+)(?::\$?([A-Za-z]+)\$?(\d+))?$/;

  function colToNum(col) {
    let n = 0;
    for (let i = 0; i < col.length; i++) n = n * 26 + (col.charCodeAt(i) - 64);
    return n - 1;
  }

  function getSheetId() {
    const el = document.getElementById("sheetName");
    return (el && el.value) ? el.value : "default";
  }

  function getAPI() {
    return window.SheetAPI || null;
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
    return RANGE_PATTERN.test(range);
  }

  function resolve(range) {
    const m = (typeof range === "string") ? range.match(RANGE_PATTERN) : null;
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

  function list() {
    const API = getAPI();
    if (!API) return Promise.resolve([]);
    return API.listNamedRanges(getSheetId()).then(function (r) {
      if (!r.ok) return [];
      const data = r.data || {};
      return Array.isArray(data.ranges) ? data.ranges : [];
    });
  }

  function get(name) {
    return list().then(function (items) {
      for (let i = 0; i < items.length; i++) {
        if (items[i].name.toLowerCase() === String(name).toLowerCase()) return items[i];
      }
      return null;
    });
  }

  function add(name, range, description) {
    if (!isValidName(name)) {
      return Promise.resolve({ ok: false, error: "Invalid name. Must start with a letter or underscore, contain only letters/digits/underscores/periods, and not be a reserved word." });
    }
    if (!isValidRange(range)) {
      return Promise.resolve({ ok: false, error: "Invalid range. Use Excel A1 notation, e.g. A1 or A1:B10." });
    }
    const API = getAPI();
    if (!API) return Promise.resolve({ ok: false, error: "API client not loaded" });
    return API.createNamedRange(getSheetId(), name, range.toUpperCase(), description || "")
      .then(function (r) {
        if (!r.ok) {
          return { ok: false, error: (r.error && r.error.message) || "Server rejected the named range" };
        }
        if (API.cacheClear) API.cacheClear("GET /api/sheet/named-ranges");
        return { ok: true, entry: (r.data && r.data.entry) || { name: name, range: range.toUpperCase(), description: description || "" } };
      });
  }

  function update(name, range, description) {
    const API = getAPI();
    if (!API) return Promise.resolve({ ok: false, error: "API client not loaded" });
    if (range !== undefined && !isValidRange(range)) {
      return Promise.resolve({ ok: false, error: "Invalid range" });
    }
    return get(name).then(function (existing) {
      if (!existing) return { ok: false, error: "Name not found" };
      const id = existing.id;
      return API.updateNamedRange(id, range ? range.toUpperCase() : undefined, description)
        .then(function (r) {
          if (!r.ok) return { ok: false, error: (r.error && r.error.message) || "Update failed" };
          if (API.cacheClear) API.cacheClear("GET /api/sheet/named-ranges");
          return { ok: true, entry: (r.data && r.data.entry) || existing };
        });
    });
  }

  function remove(name) {
    const API = getAPI();
    if (!API) return Promise.resolve({ ok: false, error: "API client not loaded" });
    return get(name).then(function (existing) {
      if (!existing) return { ok: false, error: "Name not found" };
      return API.deleteNamedRange(existing.id).then(function (r) {
        if (!r.ok) return { ok: false, error: (r.error && r.error.message) || "Delete failed" };
        if (API.cacheClear) API.cacheClear("GET /api/sheet/named-ranges");
        return { ok: true, removed: existing };
      });
    });
  }

  function clear() {
    return list().then(function (items) {
      const API = getAPI();
      const ps = items.map(function (it) { return API.deleteNamedRange(it.id); });
      return Promise.all(ps).then(function (results) {
        if (API.cacheClear) API.cacheClear("GET /api/sheet/named-ranges");
        return { ok: true, removed: results.length };
      });
    });
  }

  function exportToCSV() {
    const API = getAPI();
    if (!API) return Promise.reject(new Error("SheetAPI not loaded; cannot export without server"));
    return API.exportNamedRangesCSV(getSheetId()).then(function (r) {
      if (!r || !r.ok) return Promise.reject(new Error((r && r.error && r.error.message) || "Export failed"));
      const d = r.data || {};
      if (typeof d === "string") return d;
      if (d.csv) return d.csv;
      return JSON.stringify(d);
    });
  }

  function importFromCSV(csv) {
    if (typeof csv !== "string") return Promise.resolve({ added: 0, updated: 0, errors: ["invalid input"] });
    const API = getAPI();
    if (!API) return Promise.reject(new Error("SheetAPI not loaded; cannot import without server"));
    return API.importNamedRangesCSV(getSheetId(), csv).then(function (r) {
      if (!r || !r.ok) {
        return Promise.reject(new Error((r && r.error && r.error.message) || "Import failed"));
      }
      const data = r.data || {};
      return {
        added: data.added || 0,
        updated: data.updated || 0,
        errors: data.errors || [],
        entries: data.entries || [],
      };
    });
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

  function renderList(container, items) {
    if (!container) return;
    const data = items || [];
    if (data.length === 0) {
      container.innerHTML =
        '<p style="text-align:center;color:#888;padding:20px;">No named ranges defined. Use the form above to add one.</p>';
      return;
    }
    let html = '<table class="named-ranges-table"><thead><tr><th>Name</th><th>Range</th><th>Description</th><th></th></tr></thead><tbody>';
    for (let i = 0; i < data.length; i++) {
      const r = data[i];
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
        remove(n).then(function () { return list(); }).then(function (fresh) {
          renderList(container, fresh);
        });
      });
    }
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
