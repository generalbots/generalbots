"use strict";
/* Sheet advanced module: 19_notes — cell notes via right-click context menu */

(function () {
  let menu = null;

  function grid() {
    if (window.SheetCore && window.SheetCore.getGrid) return window.SheetCore.getGrid();
    return window.SheetVirtualGrid || null;
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

  function colName(idx) {
    let n = idx + 1;
    let s = "";
    while (n > 0) {
      const r = (n - 1) % 26;
      s = String.fromCharCode(65 + r) + s;
      n = Math.floor((n - 1) / 26);
    }
    return s;
  }

  function saveNote(row, col, note) {
    return fetch("/api/sheet/note", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        sheet_id: currentSheetId(),
        worksheet_index: wsIndex(),
        row: row,
        col: col,
        note: note,
      }),
    }).then(function (r) { return r.json(); }).catch(function () { return null; });
  }

  function updateLocalCell(row, col, note) {
    const g = grid();
    if (!g) return;
    const d = g.cells.get(row + "," + col) || {};
    d.note = note;
    d.has_comment = !!note;
    g.cells.set(row + "," + col, d);
    if (window.SheetCore && window.SheetCore.conditionalRender) window.SheetCore.conditionalRender();
  }

  function addNote(row, col) {
    const d = grid() ? grid().cells.get(row + "," + col) : null;
    const existing = (d && d.note) ? String(d.note) : "";
    const note = window.prompt("Cell note for " + colName(col) + (row + 1) + ":", existing);
    if (note === null) return;
    const trimmed = note.trim();
    saveNote(row, col, trimmed).then(function () {
      updateLocalCell(row, col, trimmed);
    });
  }

  function clearNote(row, col) {
    saveNote(row, col, "").then(function () {
      updateLocalCell(row, col, "");
    });
  }

  function cellRef(row, col) {
    return colName(col) + (row + 1);
  }

  // Open threaded comments (cross-app collab API) anchored to a cell.
  function openCellComments(row, col) {
    if (!window.GBCollabComments) return;
    var ref = cellRef(row, col);
    window.GBCollabComments.open({
      resourceType: "sheet:cell",
      resourceId: currentSheetId() + ":" + wsIndex() + ":" + ref,
      title: "Comments on " + ref,
    });
  }

  function openMenu(row, col, e) {
    closeMenu();
    const g = grid();
    if (!g || !g.bodyInner) return;
    const d = g.cells.get(row + "," + col);
    const hasNote = !!(d && (d.note || d.has_comment));
    menu = document.createElement("div");
    menu.className = "ss-cell-menu";
    menu.style.cssText =
      "position:absolute;z-index:70;background:#1e293b;border:1px solid #334155;border-radius:6px;" +
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
    menu.appendChild(item(hasNote ? "Edit note…" : "Add note…", function () { addNote(row, col); }));
    if (hasNote) menu.appendChild(item("Clear note", function () { clearNote(row, col); }));
    menu.appendChild(item("Comment…", function () { openCellComments(row, col); }));
    document.body.appendChild(menu);
    let left = e.clientX;
    let top = e.clientY;
    if (left + 190 > window.innerWidth) left = window.innerWidth - 200;
    if (top + 120 > window.innerHeight) top = window.innerHeight - 130;
    menu.style.left = left + "px";
    menu.style.top = top + "px";
  }

  function closeMenu() {
    if (menu) {
      menu.remove();
      menu = null;
    }
  }

  function onContextMenu(e) {
    const t = e.target;
    if (!t || !t.classList || !t.classList.contains("vg-cell")) return;
    const r = parseInt(t.dataset.row, 10);
    const c = parseInt(t.dataset.col, 10);
    if (isNaN(r) || isNaN(c)) return;
    e.preventDefault();
    openMenu(r, c, e);
  }

  // Clicking the amber comment triangle opens the threaded comments for that
  // cell (the marker carries data-row/data-col from the conditional renderer).
  function onClick(e) {
    const t = e.target;
    if (!t || !t.classList || !t.classList.contains("ss-comment-marker")) return;
    const r = parseInt(t.dataset.row, 10);
    const c = parseInt(t.dataset.col, 10);
    if (isNaN(r) || isNaN(c)) return;
    openCellComments(r, c);
  }

  function wire() {
    const g = grid();
    if (!g || !g.bodyInner) {
      setTimeout(wire, 100);
      return;
    }
    if (g.bodyInner.__notesBound) return;
    g.bodyInner.__notesBound = true;
    g.bodyInner.addEventListener("contextmenu", onContextMenu, true);
    g.bodyInner.addEventListener("click", onClick, true);
    document.addEventListener("mousedown", function (e) {
      if (menu && !menu.contains(e.target)) closeMenu();
    }, true);
  }

  window.SheetNotes = {
    addNote: addNote,
    clearNote: clearNote,
  };

  setTimeout(wire, 0);
})();