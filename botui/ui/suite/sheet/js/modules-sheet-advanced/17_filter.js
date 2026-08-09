"use strict";
/* Sheet advanced module: 17_filter — client-side row filtering + column filter menu */

(function () {
  let menu = null;
  let hiddenRows = new Set();
  let wrapped = false;

  function grid() {
    if (window.SheetCore && window.SheetCore.getGrid) return window.SheetCore.getGrid();
    return window.SheetVirtualGrid || null;
  }

  function api() {
    if (window.SheetCore && window.SheetCore.api) return window.SheetCore.api();
    return window.SheetAPI || null;
  }

  function currentSheetId() {
    if (window.SheetCore && window.SheetCore.currentSheetId) return window.SheetCore.currentSheetId();
    return window.__SHEET_INITIAL_ID || "current";
  }

  function wsIndex() {
    if (window.SheetCore && window.SheetCore.wsIndex) return window.SheetCore.wsIndex();
    try {
      const v = parseInt(sessionStorage.getItem("sheet_active_index") || "0", 10);
      return isNaN(v) || v < 0 ? 0 : v;
    } catch (_) {
      return 0;
    }
  }

  function currentFilters() {
    const sheet = window.__LOADED_SHEET;
    if (!sheet || !sheet.worksheets || !sheet.worksheets[wsIndex()]) return {};
    return sheet.worksheets[wsIndex()].filters || {};
  }

  function distinctValues(col) {
    const g = grid();
    const seen = [];
    const map = {};
    if (!g) return [];
    g.cells.forEach(function (data, key) {
      const parts = key.split(",");
      if (parseInt(parts[1], 10) !== col) return;
      const v = data && data.value != null ? String(data.value) : "";
      if (v === "") return;
      if (!map[v]) {
        map[v] = true;
        seen.push(v);
      }
    });
    return seen.sort();
  }

  function rowValue(g, row, col) {
    const d = g.cells.get(row + "," + col);
    return d ? (d.value != null ? String(d.value) : "") : "";
  }

  function isNum(v) {
    if (v == null || v === "") return false;
    return !isNaN(Number(v));
  }

  function matchesFilter(value, f) {
    if (!f) return true;
    const n = Number(value);
    if (f.values && f.values.length) {
      if (f.values.indexOf(value) >= 0) return true;
      return false;
    }
    if (f.condition) {
      const cond = f.condition.trim();
      if (cond.indexOf("contains:") === 0) return value.toLowerCase().indexOf(cond.slice(9).toLowerCase()) >= 0;
      const m = cond.match(/^(>=|<=|<>|>|<|=)\s*(.+)$/);
      if (m) {
        const rhs = Number(m[2]);
        if (!isNaN(rhs) && !isNaN(n)) {
          switch (m[1]) {
            case ">": return n > rhs;
            case "<": return n < rhs;
            case ">=": return n >= rhs;
            case "<=": return n <= rhs;
            case "=": return n === rhs;
            case "<>": return n !== rhs;
          }
        }
        return String(value) === m[2];
      }
      return String(value).toLowerCase().indexOf(cond.toLowerCase()) >= 0;
    }
    if (f.value1 != null && f.value1 !== "") {
      const v1 = Number(f.value1);
      if (f.value2 != null && f.value2 !== "") {
        const v2 = Number(f.value2);
        if (!isNaN(v1) && !isNaN(v2) && !isNaN(n)) return n >= v1 && n <= v2;
      }
      return value === f.value1;
    }
    return true;
  }

  function computeHiddenRows() {
    const g = grid();
    const filters = currentFilters();
    const hidden = new Set();
    if (!g || !Object.keys(filters).length) {
      hiddenRows = hidden;
      return hidden;
    }
    const cols = Object.keys(filters).map(Number);
    for (let r = 0; r < g.totalRows; r++) {
      let keep = true;
      for (let ci = 0; ci < cols.length; ci++) {
        const col = cols[ci];
        const f = filters[col];
        if (!f) continue;
        const v = rowValue(g, r, col);
        if (!matchesFilter(v, f)) { keep = false; break; }
      }
      if (!keep) hidden.add(r);
    }
    hiddenRows = hidden;
    return hidden;
  }

  function hideRowsPass() {
    const g = grid();
    if (!g) return;
    computeHiddenRows();
    if (!hiddenRows.size) return;
    const visible = g.visibleRowRange();
    for (let r = visible.start; r < visible.end; r++) {
      if (!hiddenRows.has(r)) continue;
      const els = g.bodyInner.querySelectorAll('[data-row="' + r + '"]');
      for (let i = 0; i < els.length; i++) els[i].style.display = "none";
    }
  }

  function wrapRender() {
    const g = grid();
    if (!g || wrapped) return;
    wrapped = true;
    const orig = g.render.bind(g);
    g.render = function () {
      orig();
      hideRowsPass();
    };
  }

  function persistFilter(col, payload) {
    const body = {
      sheet_id: currentSheetId(),
      worksheet_index: wsIndex(),
      col: col,
      filter_type: payload.filter_type || "text",
      values: payload.values || [],
      condition: payload.condition || null,
      value1: payload.value1 || null,
      value2: payload.value2 || null,
    };
    return fetch("/api/sheet/filter", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    }).then(function (r) { return r.json(); }).catch(function () { return null; });
  }

  function clearFilter(col) {
    return fetch("/api/sheet/filter/clear", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ sheet_id: currentSheetId(), worksheet_index: wsIndex(), col: col }),
    }).then(function (r) { return r.json(); }).catch(function () { return null; });
  }

  function reloadLocal() {
    const a = api();
    if (!a) return;
    a.load(currentSheetId()).then(function (sheet) {
      if (sheet) {
        window.__LOADED_SHEET = sheet;
        window.__SHEET_INITIAL_ID = sheet.id;
      }
      if (window.SheetCore && window.SheetCore.rehydrateGrid) window.SheetCore.rehydrateGrid();
      if (window.SheetCore && window.SheetCore.refreshFrozen) window.SheetCore.refreshFrozen();
    });
  }

  function showFilterMenu(col, anchor) {
    closeMenu();
    const g = grid();
    const vals = distinctValues(col);
    const cur = currentFilters()[col];
    menu = document.createElement("div");
    menu.className = "ss-filter-menu";
    menu.style.cssText =
      "position:absolute;z-index:60;background:#1e293b;border:1px solid #334155;border-radius:6px;" +
      "box-shadow:0 8px 24px rgba(0,0,0,0.4);min-width:230px;max-height:320px;overflow-y:auto;padding:8px;";
    const title = document.createElement("div");
    title.textContent = "Filter column " + (g && g.colName ? g.colName(col) : "");
    title.style.cssText = "color:#94a3b8;font-size:11px;text-transform:uppercase;letter-spacing:0.5px;margin-bottom:8px;";
    menu.appendChild(title);

    const btnRow = document.createElement("div");
    btnRow.style.cssText = "display:flex;gap:6px;margin-bottom:8px;";
    const applyBtn = document.createElement("button");
    applyBtn.textContent = "Apply";
    applyBtn.type = "button";
    applyBtn.style.cssText = "flex:1;background:#3b82f6;color:#fff;border:none;border-radius:4px;padding:6px;font-size:12px;cursor:pointer;";
    const clearBtn = document.createElement("button");
    clearBtn.textContent = "Clear";
    clearBtn.type = "button";
    clearBtn.style.cssText = "flex:1;background:#334155;color:#f8fafc;border:none;border-radius:4px;padding:6px;font-size:12px;cursor:pointer;";
    btnRow.appendChild(applyBtn);
    btnRow.appendChild(clearBtn);
    menu.appendChild(btnRow);

    const checks = {};
    const body = document.createElement("div");
    if (cur && cur.values && cur.values.length) {
      vals.forEach(function (v) { checks[v] = cur.values.indexOf(v) >= 0; });
      vals.forEach(function (v) {
        if (checks[v] === undefined) checks[v] = true;
      });
    } else {
      vals.forEach(function (v) { checks[v] = true; });
    }
    vals.slice(0, 200).forEach(function (v) {
      const row = document.createElement("label");
      row.style.cssText = "display:flex;align-items:center;gap:8px;padding:3px 0;font-size:13px;color:#f8fafc;cursor:pointer;";
      const cb = document.createElement("input");
      cb.type = "checkbox";
      cb.checked = checks[v] !== false;
      cb.addEventListener("change", function () { checks[v] = cb.checked; });
      const span = document.createElement("span");
      span.textContent = v;
      span.style.cssText = "white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:160px;";
      row.appendChild(cb);
      row.appendChild(span);
      body.appendChild(row);
    });
    if (!vals.length) {
      body.innerHTML = '<div style="color:#94a3b8;font-size:12px;padding:8px;">No values in this column.</div>';
    }
    menu.appendChild(body);

    applyBtn.addEventListener("click", function () {
      const selected = vals.filter(function (v) { return checks[v]; });
      persistFilter(col, { filter_type: "list", values: selected }).then(function () {
        reloadLocal();
        closeMenu();
      });
    });
    clearBtn.addEventListener("click", function () {
      clearFilter(col).then(function () {
        reloadLocal();
        closeMenu();
      });
    });

    document.body.appendChild(menu);
    const rect = anchor.getBoundingClientRect();
    let left = rect.left;
    let top = rect.bottom + 4;
    if (left + 250 > window.innerWidth) left = window.innerWidth - 260;
    if (top + 320 > window.innerHeight) top = Math.max(0, rect.top - 330);
    menu.style.left = left + "px";
    menu.style.top = top + "px";
  }

  function closeMenu() {
    if (menu) {
      menu.remove();
      menu = null;
    }
  }

  function wire() {
    const g = grid();
    if (!g) {
      setTimeout(wire, 100);
      return;
    }
    wrapRender();
    if (g.headerRow && !g.headerRow.__filterBound) {
      g.headerRow.__filterBound = true;
      g.headerRow.addEventListener("mousedown", function (e) {
        const h = e.target.closest(".ss-filter-head");
        if (!h) return;
        e.preventDefault();
        e.stopPropagation();
        showFilterMenu(parseInt(h.dataset.col, 10), h);
      }, true);
    }
    document.addEventListener("mousedown", function (e) {
      if (menu && !menu.contains(e.target)) closeMenu();
    }, true);
    renderFilterHeads();
  }

  function renderFilterHeads() {
    const g = grid();
    if (!g || !g.headerRow) return;
    g.headerRow.querySelectorAll(".ss-filter-head").forEach(function (el) { el.remove(); });
    const HEADER_WIDTH = 48;
    const COL_WIDTH = 96;
    const active = currentFilters();
    for (let c = 0; c < g.totalCols; c++) {
      const head = document.createElement("div");
      head.className = "ss-filter-head";
      head.dataset.col = c;
      head.title = "Filter";
      const baseX = window.SheetCore && window.SheetCore.colX ? window.SheetCore.colX(c) : HEADER_WIDTH + c * COL_WIDTH;
      const w = window.SheetCore && window.SheetCore.colWidth ? window.SheetCore.colWidth(c) : COL_WIDTH;
      head.style.cssText =
        "position:absolute;left:" + (baseX + w - 34) + "px;top:2px;width:16px;height:16px;color:" + (active[c] ? "#3b82f6" : "#475569") +
        ";cursor:pointer;z-index:6;font-size:10px;line-height:16px;text-align:center;";
      head.textContent = "▾";
      g.headerRow.appendChild(head);
    }
  }

  window.SheetFilter = {
    matchesFilter: matchesFilter,
    computeHidden: computeHiddenRows,
    applyFilter: persistFilter,
    clearFilter: clearFilter,
    hidePass: hideRowsPass,
    renderHeads: renderFilterHeads,
  };

  document.addEventListener("gb-sheet-tab", function () { setTimeout(wire, 60); });
  setTimeout(wire, 0);
})();