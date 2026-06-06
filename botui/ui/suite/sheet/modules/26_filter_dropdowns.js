"use strict";

/**
 * Module 26: Filter dropdowns for Spreadsheet (P1+ feature).
 * Opens a checkbox-list overlay anchored to a column header. Renders
 * unique values from the column, plus A→Z / Z→A sort buttons, top-N
 * picker, blank toggle. Applies the resulting filter + sort to the
 * worksheet's in-memory state (filter.criteria, sortOrder[col]).
 *
 * On apply, the criterion and sort order are sent to the backend via
 * SheetAPI.filter / SheetAPI.sortRange so that other sessions and
 * re-opens see the same view. The local state is the source of truth
 * for instant UX; the server is the source of truth for persistence.
 *
 * Public API: window.SheetFilterDropdowns = {
 *   open, close, applyToColumn, clearColumn, getActiveFilters, ...
 * }.
 */

(function () {
  function getState() { return window.state || null; }
  function getSheet() {
    const s = getState();
    if (!s) return null;
    return (s.worksheets || [])[s.currentSheet || 0];
  }
  function uniqueValues(col) {
    const ws = getSheet();
    if (!ws || !ws.cells) return [];
    const set = new Set();
    let hasBlank = false;
    for (let r = 0; r < ws.cells.length; r++) {
      const v = ws.cells[r] && ws.cells[r][col];
      if (v == null || v === "") { hasBlank = true; continue; }
      set.add(String(v));
    }
    const out = Array.from(set).sort(function (a, b) { return a.localeCompare(b); });
    if (hasBlank) out.push("(Em branco)");
    return out;
  }

  function readColumn(col) {
    const ws = getSheet();
    if (!ws || !ws.cells) return [];
    const out = [];
    for (let r = 0; r < ws.cells.length; r++) {
      const v = ws.cells[r] && ws.cells[r][col];
      out.push({ row: r, value: v == null ? "" : v });
    }
    return out;
  }

  let _overlay = null;
  let _state = {};

  function overlay() {
    if (_overlay) return _overlay;
    _overlay = document.createElement("div");
    _overlay.id = "sheetFilterDropdown";
    _overlay.style.cssText = "position:absolute;background:#fff;border:1px solid #dadce0;border-radius:4px;box-shadow:0 4px 12px rgba(0,0,0,0.18);z-index:9997;display:none;min-width:240px;max-width:280px;max-height:340px;overflow:hidden;font-family:Inter,Arial,sans-serif;";
    document.body.appendChild(_overlay);
    document.addEventListener("click", function (e) {
      if (!_overlay.contains(e.target) && !e.target.closest("[data-filter-trigger]")) close();
    });
    return _overlay;
  }

  function close() {
    const o = overlay();
    o.style.display = "none";
    o.innerHTML = "";
  }

  function open(col, anchorX, anchorY) {
    const o = overlay();
    o.innerHTML = "";
    const vals = uniqueValues(col);
    _state[col] = _state[col] || { selected: {}, sort: null, top: null, blank: true };
    const sel = _state[col].selected;
    vals.forEach(function (v) { if (sel[v] === undefined) sel[v] = true; });

    const header = document.createElement("div");
    header.style.cssText = "padding:8px 10px;border-bottom:1px solid #e8eaed;font-weight:600;font-size:12px;color:#5f6368;text-transform:uppercase;";
    header.textContent = "Filtrar coluna " + ((window.SheetFormulaEngine || {}).indexToColName || function (c) { return c; })(col);
    o.appendChild(header);

    const search = document.createElement("input");
    search.type = "text";
    search.placeholder = "Buscar valores...";
    search.style.cssText = "margin:8px 10px;width:calc(100% - 20px);padding:6px 8px;border:1px solid #dadce0;border-radius:3px;box-sizing:border-box;font-size:13px;";
    o.appendChild(search);

    const sortBar = document.createElement("div");
    sortBar.style.cssText = "display:flex;gap:4px;padding:0 10px 6px;";
    const aBtn = button("A → Z", function () { setSort(col, "asc"); rerender(); });
    const dBtn = button("Z → A", function () { setSort(col, "desc"); rerender(); });
    const cBtn = button("Limpar", function () { _state[col] = { selected: {}, sort: null, top: null, blank: true }; vals.forEach(function (v) { _state[col].selected[v] = true; }); rerender(); applyToColumn(col); });
    sortBar.appendChild(aBtn); sortBar.appendChild(dBtn); sortBar.appendChild(cBtn);
    o.appendChild(sortBar);

    const list = document.createElement("div");
    list.style.cssText = "max-height:200px;overflow-y:auto;padding:0 10px 8px;";
    o.appendChild(list);

    function rerender() {
      list.innerHTML = "";
      const filterText = search.value.toLowerCase();
      const items = vals.filter(function (v) { return v.toLowerCase().indexOf(filterText) >= 0; });
      const selectAll = document.createElement("label");
      selectAll.style.cssText = "display:flex;align-items:center;padding:4px 0;font-size:12px;color:#5f6368;cursor:pointer;border-bottom:1px solid #e8eaed;margin-bottom:4px;";
      const allChk = document.createElement("input");
      allChk.type = "checkbox";
      allChk.checked = items.every(function (v) { return _state[col].selected[v] !== false; });
      allChk.addEventListener("change", function () {
        items.forEach(function (v) { _state[col].selected[v] = allChk.checked; });
        rerender();
      });
      selectAll.appendChild(allChk);
      const allLabel = document.createElement("span");
      allLabel.style.cssText = "margin-left:6px;";
      allLabel.textContent = "(Selecionar tudo)";
      selectAll.appendChild(allLabel);
      list.appendChild(selectAll);
      for (const v of items) {
        const label = document.createElement("label");
        label.style.cssText = "display:flex;align-items:center;padding:3px 0;font-size:13px;cursor:pointer;";
        const chk = document.createElement("input");
        chk.type = "checkbox";
        chk.checked = _state[col].selected[v] !== false;
        chk.addEventListener("change", function () { _state[col].selected[v] = chk.checked; });
        label.appendChild(chk);
        const txt = document.createElement("span");
        txt.style.cssText = "margin-left:6px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;";
        txt.textContent = v.length > 32 ? v.slice(0, 32) + "..." : v;
        label.appendChild(txt);
        list.appendChild(label);
      }
    }
    search.addEventListener("input", rerender);
    rerender();

    const applyBar = document.createElement("div");
    applyBar.style.cssText = "display:flex;gap:6px;justify-content:flex-end;padding:8px 10px;border-top:1px solid #e8eaed;background:#f8f9fa;";
    const cancelBtn = button("Cancelar", function () { close(); });
    cancelBtn.style.cssText += "background:#fff;border:1px solid #dadce0;";
    const okBtn = button("Aplicar", function () { applyToColumn(col); close(); });
    okBtn.style.cssText += "background:#1a73e8;color:#fff;border:0;";
    applyBar.appendChild(cancelBtn);
    applyBar.appendChild(okBtn);
    o.appendChild(applyBar);

    o.style.left = Math.min(anchorX, window.innerWidth - 280) + "px";
    o.style.top = Math.min(anchorY, window.innerHeight - 360) + "px";
    o.style.display = "block";
  }

  function button(label, onClick) {
    const b = document.createElement("button");
    b.textContent = label;
    b.style.cssText = "padding:4px 10px;border:1px solid #dadce0;border-radius:3px;background:#fff;font-size:12px;cursor:pointer;";
    b.addEventListener("click", onClick);
    return b;
  }

  function setSort(col, dir) {
    if (!_state[col]) _state[col] = { selected: {}, sort: null, top: null, blank: true };
    _state[col].sort = dir;
  }
  function setTop(col, n) {
    if (!_state[col]) _state[col] = { selected: {}, sort: null, top: null, blank: true };
    _state[col].top = n;
  }

  function applyToColumn(col) {
    const ws = getSheet();
    if (!ws) return false;
    const st = _state[col] || { selected: {}, sort: null, top: null, blank: true };
    if (!ws.filter) ws.filter = { hidden: [] };
    if (!ws.filter.criteria) ws.filter.criteria = {};
    const sel = st.selected;
    const criterion = function (v) {
      if (v == null || v === "") return st.blank !== false;
      const sv = String(v);
      return sel[sv] !== false;
    };
    ws.filter.criteria[col] = criterion;
    if (st.sort === "asc" || st.sort === "desc") {
      const rows = readColumn(col);
      const indices = rows.slice();
      indices.sort(function (a, b) {
        const an = parseFloat(a.value);
        const bn = parseFloat(b.value);
        if (!Number.isNaN(an) && !Number.isNaN(bn)) return st.sort === "asc" ? an - bn : bn - an;
        const av = String(a.value), bv = String(b.value);
        return st.sort === "asc" ? av.localeCompare(bv) : bv.localeCompare(av);
      });
      const sorted = indices.map(function (x) { return x.row; });
      if (!ws.sortOrder) ws.sortOrder = {};
      ws.sortOrder[col] = sorted;
    } else {
      if (ws.sortOrder) delete ws.sortOrder[col];
    }
    if (typeof window.SheetRender === "object" && window.SheetRender.repaint) {
      window.SheetRender.repaint();
    } else if (typeof window.renderGrid === "function") {
      window.renderGrid();
    }
    syncToServer(col, ws);
    return true;
  }

  function clearColumn(col) {
    if (_state[col]) delete _state[col];
    const ws = getSheet();
    if (ws && ws.filter && ws.filter.criteria) delete ws.filter.criteria[col];
    if (ws && ws.sortOrder) delete ws.sortOrder[col];
    if (typeof window.SheetRender === "object" && window.SheetRender.repaint) window.SheetRender.repaint();
    syncClearToServer(col, ws);
  }

  function getSheetId() {
    const el = document.getElementById("sheetName");
    return (el && el.value) ? el.value : null;
  }

  function syncToServer(col, ws) {
    const API = window.SheetAPI;
    const sheetId = getSheetId();
    if (!API || !sheetId || !ws) return;
    const st = _state[col] || {};
    const allowed = Object.keys(st.selected || {}).filter(function (k) { return st.selected[k] !== false; });
    const criterion = {
      column: col,
      allowed: allowed,
      blank: st.blank !== false,
      sort: st.sort || null,
    };
    API.filter(sheetId, null, criterion).catch(function () { /* offline ok; local state is source of truth for instant UX */ });
    if ((st.sort === "asc" || st.sort === "desc") && ws.sortOrder && ws.sortOrder[col]) {
      API.sortRange(sheetId, null, col, st.sort).catch(function () {});
    }
  }

  function syncClearToServer(col, ws) {
    const API = window.SheetAPI;
    const sheetId = getSheetId();
    if (!API || !sheetId) return;
    API.clearFilter(sheetId).catch(function () {});
  }

  function getActiveFilters() {
    const out = {};
    for (const col in _state) if (Object.keys(_state[col].selected).length) out[col] = _state[col];
    return out;
  }

  function attach() {
    document.addEventListener("click", function (e) {
      const trigger = e.target.closest && e.target.closest("[data-filter-trigger]");
      if (!trigger) return;
      e.preventDefault();
      e.stopPropagation();
      const col = parseInt(trigger.dataset.filterTrigger, 10);
      const rect = trigger.getBoundingClientRect();
      open(col, rect.left, rect.bottom);
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.SheetFilterDropdowns = { open, close, applyToColumn, clearColumn, getState: function () { return _state; }, getActiveFilters, uniqueValues, setSort, setTop };
})();
