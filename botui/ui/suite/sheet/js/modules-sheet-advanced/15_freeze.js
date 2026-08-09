"use strict";
/* Sheet advanced module: 15_freeze — freeze/unfreeze panes via /api/sheet/freeze */

(function () {
  let menu = null;

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

  function frozen() {
    const sheet = window.__LOADED_SHEET;
    if (!sheet || !sheet.worksheets || !sheet.worksheets[wsIndex()]) return { rows: 0, cols: 0 };
    const ws = sheet.worksheets[wsIndex()];
    return { rows: ws.frozen_rows || 0, cols: ws.frozen_cols || 0 };
  }

  function freeze(rows, cols) {
    return fetch("/api/sheet/freeze", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        sheet_id: currentSheetId(),
        worksheet_index: wsIndex(),
        frozen_rows: rows,
        frozen_cols: cols,
      }),
    })
      .then(function (r) { return r.json(); })
      .then(function () {
        const sheet = window.__LOADED_SHEET;
        if (sheet && sheet.worksheets && sheet.worksheets[wsIndex()]) {
          sheet.worksheets[wsIndex()].frozen_rows = rows;
          sheet.worksheets[wsIndex()].frozen_cols = cols;
        }
        return { rows: rows, cols: cols };
      })
      .catch(function () { return null; });
  }

  function freezeTopRow() {
    const f = frozen();
    return freeze(1, f.cols);
  }

  function freezeFirstColumn() {
    const f = frozen();
    return freeze(f.rows, 1);
  }

  function freezeFirstRowAndColumn() {
    return freeze(1, 1);
  }

  function unfreeze() {
    return freeze(0, 0);
  }

  function openMenu(anchor) {
    closeMenu();
    menu = document.createElement("div");
    menu.className = "ss-freeze-menu";
    menu.style.cssText =
      "position:absolute;z-index:60;background:#1e293b;border:1px solid #334155;border-radius:6px;" +
      "box-shadow:0 8px 24px rgba(0,0,0,0.4);min-width:200px;overflow:hidden;";
    const title = document.createElement("div");
    title.textContent = "Freeze";
    title.style.cssText = "padding:8px 14px;color:#94a3b8;font-size:11px;text-transform:uppercase;letter-spacing:0.5px;border-bottom:1px solid #334155;";
    menu.appendChild(title);
    const item = function (label, fn, active) {
      const b = document.createElement("button");
      b.type = "button";
      b.textContent = label;
      b.style.cssText =
        "display:block;width:100%;padding:8px 14px;background:none;border:none;color:#f8fafc;" +
        "text-align:left;font-size:13px;cursor:pointer;";
      if (active) b.textContent = "✓ " + label;
      b.addEventListener("mouseover", function () { b.style.background = "#334155"; });
      b.addEventListener("mouseout", function () { b.style.background = "none"; });
      b.addEventListener("click", function () { closeMenu(); fn(); });
      return b;
    };
    const f = frozen();
    menu.appendChild(item("Freeze Top Row", freezeTopRow, f.rows > 0));
    menu.appendChild(item("Freeze First Column", freezeFirstColumn, f.cols > 0));
    menu.appendChild(item("Freeze First Row & Column", freezeFirstRowAndColumn, f.rows > 0 && f.cols > 0));
    menu.appendChild(item("Unfreeze", unfreeze, false));
    document.body.appendChild(menu);
    const rect = anchor.getBoundingClientRect();
    let left = rect.left;
    let top = rect.bottom + 4;
    if (left + 220 > window.innerWidth) left = window.innerWidth - 230;
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
    const host = document.getElementById("sheet-app");
    if (!host) {
      setTimeout(wire, 100);
      return;
    }
    if (host.__freezeBound) return;
    host.__freezeBound = true;

    const btn = document.createElement("button");
    btn.className = "btn-icon";
    btn.id = "freezeBtn";
    btn.title = "Freeze panes";
    btn.style.cssText = "display:inline-flex;align-items:center;gap:4px;";
    btn.innerHTML =
      '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"></rect><line x1="3" y1="9" x2="21" y2="9"></line><line x1="9" y1="3" x2="9" y2="21"></line></svg>' +
      '<span style="font-size:12px;margin-left:4px;">Freeze</span>';
    btn.addEventListener("click", function (e) {
      e.preventDefault();
      openMenu(btn);
    });
    const right = host.querySelector(".toolbar-right");
    if (right) right.insertBefore(btn, right.querySelector(".toolbar-divider") ? right.querySelector(".toolbar-divider").nextSibling : right.firstChild);

    document.addEventListener("mousedown", function (e) {
      if (menu && !menu.contains(e.target) && e.target.id !== "freezeBtn") closeMenu();
    }, true);
  }

  window.SheetFreeze = {
    freeze: freeze,
    unfreeze: unfreeze,
    freezeTopRow: freezeTopRow,
    freezeFirstColumn: freezeFirstColumn,
    freezeFirstRowAndColumn: freezeFirstRowAndColumn,
    getFrozen: frozen,
  };

  if (window.SheetCore) {
    window.SheetCore.freeze = freeze;
  }

  setTimeout(wire, 0);
})();