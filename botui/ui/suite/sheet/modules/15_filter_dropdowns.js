"use strict";

/**
 * Module 15: Filter dropdowns for Sheet.
 * Renders dropdown indicators on column headers when filter mode is active.
 * Clicking a dropdown opens a popup with: select all, search box, list of
 * unique values with checkboxes, sort options, condition filters.
 *
 * Filters are stored in state.activeFilters[col] = { values: Set, sort, criteria }.
 * Filtering hides row DOM elements (or removes from virtual render set) that
 * don't match.
 */

(function () {
  function getState() {
    return window.state || null;
  }

  function getWorksheet() {
    const s = getState();
    if (!s || !s.worksheets) return null;
    return s.worksheets[s.activeWorksheet];
  }

  function isFilterActive() {
    const s = getState();
    return s && s.filterEnabled;
  }

  function collectUniqueValues(col) {
    const ws = getWorksheet();
    if (!ws || !ws.data) return [];
    const set = new Set();
    for (const key in ws.data) {
      const [r, c] = key.split(",").map(Number);
      if (c !== col) continue;
      const cell = ws.data[key];
      const v = cell && (cell.value != null ? cell.value : cell.formula);
      set.add(v == null ? "" : String(v));
    }
    return Array.from(set);
  }

  function renderFilterIndicator(col, headerEl) {
    if (!headerEl) return;
    if (headerEl.querySelector(".filter-indicator")) return;
    const indicator = document.createElement("span");
    indicator.className = "filter-indicator";
    indicator.textContent = "▼";
    indicator.style.cssText = "margin-left:4px;cursor:pointer;font-size:10px;color:#888;";
    indicator.addEventListener("click", function (e) {
      e.stopPropagation();
      openFilterDropdown(col, indicator);
    });
    headerEl.appendChild(indicator);
  }

  function openFilterDropdown(col, anchor) {
    closeFilterDropdown();
    const unique = collectUniqueValues(col);
    const popup = document.createElement("div");
    popup.className = "filter-dropdown";
    popup.style.cssText =
      "position:absolute;background:#fff;border:1px solid #888;border-radius:4px;padding:8px;z-index:9999;min-width:200px;max-height:320px;overflow:auto;box-shadow:0 2px 8px rgba(0,0,0,0.2);font-size:12px;";
    const rect = anchor.getBoundingClientRect();
    popup.style.left = rect.left + "px";
    popup.style.top = rect.bottom + "px";
    const search = document.createElement("input");
    search.type = "text";
    search.placeholder = "Search values…";
    search.style.cssText = "width:100%;padding:4px;margin-bottom:4px;box-sizing:border-box;";
    popup.appendChild(search);
    const list = document.createElement("div");
    list.style.cssText = "max-height:200px;overflow-y:auto;";
    const allChecked = new Set(unique);
    unique.forEach(function (v) {
      const row = document.createElement("label");
      row.style.cssText = "display:block;cursor:pointer;";
      const cb = document.createElement("input");
      cb.type = "checkbox";
      cb.checked = true;
      cb.value = v;
      cb.style.marginRight = "4px";
      cb.addEventListener("change", function () {
        if (cb.checked) allChecked.add(v);
        else allChecked.delete(v);
        updateStatus();
      });
      row.appendChild(cb);
      row.appendChild(document.createTextNode(v === "" ? "(Blanks)" : v));
      list.appendChild(row);
    });
    popup.appendChild(list);
    const status = document.createElement("div");
    status.style.cssText = "margin-top:4px;color:#666;font-size:11px;";
    function updateStatus() {
      status.textContent = allChecked.size + " of " + unique.length + " selected";
    }
    updateStatus();
    popup.appendChild(status);
    const btnRow = document.createElement("div");
    btnRow.style.cssText = "margin-top:8px;display:flex;gap:4px;";
    const apply = document.createElement("button");
    apply.textContent = "Apply";
    apply.style.cssText = "flex:1;padding:4px;background:#0a6;color:#fff;border:0;border-radius:3px;cursor:pointer;";
    apply.addEventListener("click", function () {
      const s = getState();
      if (!s) return;
      if (!s.activeFilters) s.activeFilters = {};
      s.activeFilters[col] = { values: allChecked };
      closeFilterDropdown();
      applyFiltersToUI();
    });
    const clear = document.createElement("button");
    clear.textContent = "Clear";
    clear.style.cssText = "flex:1;padding:4px;background:#ccc;border:0;border-radius:3px;cursor:pointer;";
    clear.addEventListener("click", function () {
      const s = getState();
      if (!s) return;
      if (s.activeFilters) delete s.activeFilters[col];
      closeFilterDropdown();
      applyFiltersToUI();
    });
    btnRow.appendChild(apply);
    btnRow.appendChild(clear);
    popup.appendChild(btnRow);
    document.body.appendChild(popup);
    search.addEventListener("input", function () {
      const q = search.value.toLowerCase();
      list.querySelectorAll("label").forEach(function (lbl) {
        lbl.style.display = lbl.textContent.toLowerCase().indexOf(q) === -1 ? "none" : "block";
      });
    });
    function onDocClick(e) {
      if (!popup.contains(e.target) && e.target !== anchor) closeFilterDropdown();
    }
    setTimeout(function () {
      document.addEventListener("click", onDocClick, true);
    }, 0);
    popup.__cleanup = function () { document.removeEventListener("click", onDocClick, true); };
  }

  function closeFilterDropdown() {
    document.querySelectorAll(".filter-dropdown").forEach(function (p) {
      if (p.__cleanup) p.__cleanup();
      p.remove();
    });
  }

  function applyFiltersToUI() {
    const ws = getWorksheet();
    if (!ws) return;
    const s = getState();
    const filters = s.activeFilters || {};
    const hasAny = Object.keys(filters).length > 0;
    const rows = document.querySelectorAll(".cell-row, [data-row]");
    rows.forEach(function (rowEl) {
      const row = parseInt(rowEl.getAttribute("data-row"), 10);
      if (isNaN(row)) return;
      let show = true;
      if (hasAny) {
        for (const colStr in filters) {
          const col = parseInt(colStr, 10);
          const allowed = filters[colStr].values;
          const key = row + "," + col;
          const cell = ws.data && ws.data[key];
          const v = cell ? (cell.value != null ? String(cell.value) : "") : "";
          if (!allowed.has(v)) { show = false; break; }
        }
      }
      rowEl.style.display = show ? "" : "none";
    });
    updateStatusBar();
  }

  function updateStatusBar() {
    const s = getState();
    const filters = s.activeFilters || {};
    const hasAny = Object.keys(filters).length > 0;
    const visible = document.querySelectorAll('.cell-row:not([style*="display: none"]), [data-row]:not([style*="display: none"])').length;
    const status = document.querySelector(".status-bar-count, #row-count");
    if (status) {
      status.textContent = hasAny
        ? visible + " rows (filtered)"
        : visible + " rows";
    }
  }

  function attachIndicators() {
    if (!isFilterActive()) return;
    const headerCells = document.querySelectorAll(".col-header, [data-col-header]");
    headerCells.forEach(function (h) {
      const col = parseInt(h.getAttribute("data-col") || h.getAttribute("data-col-header"), 10);
      if (isNaN(col)) return;
      renderFilterIndicator(col, h);
    });
  }

  function setupObserver() {
    const sheet = document.querySelector(".sheet-container, .worksheet, #cells");
    if (!sheet) return;
    const obs = new MutationObserver(function () {
      if (isFilterActive()) attachIndicators();
    });
    obs.observe(sheet, { childList: true, subtree: true });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      attachIndicators();
      setupObserver();
    });
  } else {
    attachIndicators();
    setupObserver();
  }

  let attempts = 0;
  const interval = setInterval(function () {
    if (isFilterActive()) attachIndicators();
    attempts++;
    if (attempts > 30) clearInterval(interval);
  }, 200);

  window.SheetFilterDropdowns = {
    collectUniqueValues,
    openFilterDropdown,
    closeFilterDropdown,
    applyFiltersToUI,
    attachIndicators,
  };
})();
