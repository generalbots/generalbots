"use strict";
/* Sheet advanced module: 11_sort — column-header sort menu wired to /api/sheet/sort */

(function () {
  let menu = null;

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

  function selection() {
    const adv = window.SheetAdvanced;
    return adv && adv.getSelection ? adv.getSelection() : null;
  }

  function populatedBounds() {
    const g = grid();
    if (!g) return null;
    let minR = Infinity, maxR = -1, minC = Infinity, maxC = -1;
    g.cells.forEach(function (_, key) {
      const parts = key.split(",");
      const r = parseInt(parts[0], 10);
      const c = parseInt(parts[1], 10);
      if (isNaN(r) || isNaN(c)) return;
      if (r < minR) minR = r;
      if (r > maxR) maxR = r;
      if (c < minC) minC = c;
      if (c > maxC) maxC = c;
    });
    if (maxR < 0) return null;
    return { start_row: minR, start_col: minC, end_row: maxR, end_col: maxC };
  }

  function sortRange(col, ascending) {
    const sel = selection();
    let range;
    if (sel && (sel.endRow > sel.startRow || sel.endCol > sel.startCol)) {
      range = { start_row: sel.startRow, start_col: sel.startCol, end_row: sel.endRow, end_col: sel.endCol };
    } else {
      range = populatedBounds();
      if (!range) return Promise.resolve(null);
    }
    if (col < range.start_col || col > range.end_col) return Promise.resolve(null);
    const payload = {
      sheet_id: currentSheetId(),
      worksheet_index: wsIndex(),
      start_row: range.start_row,
      start_col: range.start_col,
      end_row: range.end_row,
      end_col: range.end_col,
      sort_col: col,
      ascending: ascending,
    };
    return fetch("/api/sheet/sort", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    })
      .then(function (r) { return r.json(); })
      .then(function (j) {
        reloadFromServer();
        return j;
      })
      .catch(function () { return null; });
  }

  function reloadFromServer() {
    const g = grid();
    const a = api();
    if (!g || !a) return;
    a.load(currentSheetId()).then(function (sheet) {
      if (sheet) {
        window.__LOADED_SHEET = sheet;
        window.__SHEET_INITIAL_ID = sheet.id;
      }
      if (window.SheetCore && window.SheetCore.rehydrateGrid) {
        window.SheetCore.rehydrateGrid();
      } else {
        g.lastRenderedRange = null;
        g.requestRange();
      }
    });
  }

  function openMenu(col, anchor) {
    closeMenu();
    const g = grid();
    if (!g) return;
    menu = document.createElement("div");
    menu.className = "ss-sort-menu";
    menu.style.cssText =
      "position:absolute;z-index:60;background:#1e293b;border:1px solid #334155;border-radius:6px;" +
      "box-shadow:0 8px 24px rgba(0,0,0,0.4);min-width:180px;overflow:hidden;";
    const item = function (label, fn) {
      const b = document.createElement("button");
      b.type = "button";
      b.textContent = label;
      b.style.cssText =
        "display:block;width:100%;padding:8px 14px;background:none;border:none;color:#f8fafc;" +
        "text-align:left;font-size:13px;cursor:pointer;";
      b.addEventListener("mouseover", function () { b.style.background = "#334155"; });
      b.addEventListener("mouseout", function () { b.style.background = "none"; });
      b.addEventListener("click", function () { closeMenu(); fn(); });
      return b;
    };
    menu.appendChild(item("Sort A → Z", function () { sortRange(col, true); }));
    menu.appendChild(item("Sort Z → A", function () { sortRange(col, false); }));
    menu.appendChild(item("Clear filters", function () { clearFilters(); }));
    g.bodyInner.appendChild(menu);
    const rect = g.bodyInner.getBoundingClientRect();
    const anchorRect = anchor.getBoundingClientRect();
    let left = anchorRect.left - rect.left;
    let top = anchorRect.bottom - rect.top + 2;
    if (left + 190 > rect.width) left = Math.max(0, rect.width - 190);
    if (top + 120 > rect.height) top = Math.max(0, anchorRect.top - rect.top - 124);
    menu.style.left = left + "px";
    menu.style.top = top + "px";
  }

  function clearFilters() {
    return fetch("/api/sheet/filter/clear", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ sheet_id: currentSheetId(), worksheet_index: wsIndex(), col: null }),
    })
      .then(function () { reloadFromServer(); })
      .catch(function () {});
  }

  function closeMenu() {
    if (menu) {
      menu.remove();
      menu = null;
    }
  }

  function onHeaderClick(e) {
    const h = e.target.closest(".ss-sort-head");
    if (!h) return;
    e.preventDefault();
    e.stopPropagation();
    const col = parseInt(h.dataset.col, 10);
    if (isNaN(col)) return;
    openMenu(col, h);
  }

  function wire() {
    const g = grid();
    if (!g || !g.headerRow) {
      setTimeout(wire, 100);
      return;
    }
    if (g.headerRow.__sortBound) return;
    g.headerRow.__sortBound = true;
    g.headerRow.addEventListener("mousedown", onHeaderClick, true);
    document.addEventListener("mousedown", function (e) {
      if (menu && !menu.contains(e.target)) closeMenu();
    }, true);
    setTimeout(addSortHandles, 200);
  }

  function addSortHandles() {
    const g = grid();
    if (!g || !g.headerRow) return;
    g.headerRow.querySelectorAll(".ss-sort-head").forEach(function (el) { el.remove(); });
    g.headerRow.style.position = "relative";
    const HEADER_WIDTH = 48;
    const COL_WIDTH = 96;
    for (let c = 0; c < g.totalCols; c++) {
      const head = document.createElement("div");
      head.className = "ss-sort-head";
      head.dataset.col = c;
      head.style.cssText =
        "position:absolute;left:" + (HEADER_WIDTH + c * COL_WIDTH + COL_WIDTH - 20) +
        "px;top:2px;width:16px;height:16px;color:#64748b;" +
        "cursor:pointer;z-index:6;font-size:10px;line-height:16px;text-align:center;";
      head.textContent = "⤨";
      head.title = "Sort";
      g.headerRow.appendChild(head);
    }
  }

  window.SheetSort = {
    sortRange: sortRange,
    openMenu: openMenu,
    closeMenu: closeMenu,
    reload: reloadFromServer,
    refreshHandles: addSortHandles,
  };

  if (window.SheetCore) {
    window.SheetCore.sort = sortRange;
  }

  setTimeout(wire, 0);
})();