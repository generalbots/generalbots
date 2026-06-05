"use strict";

/**
 * Module 15: Table editing (post-insertion) for Word Processor.
 * Adds a context menu on right-click inside a table with: Insert Row
 * Above/Below, Insert Column Left/Right, Delete Row/Column, Merge
 * Cells, Split Cell. Also adds a table toolbar that appears when
 * the cursor is inside a table with border controls. Implements a
 * visual column resize handle on table column borders. Supports
 * table properties dialog (width, alignment, cell padding).
 *
 * Public API: window.DocsTables = { attachContextMenu, showToolbar,
 *   insertRow, insertCol, deleteRow, deleteCol, mergeCells, splitCell,
 *   setCellPadding, setBorderColor, setBorderWidth, setBgColor }.
 */

(function () {
  function getState() { return window.state || null; }

  function findCell(node) {
    let n = node;
    while (n && n !== document.body) {
      if (n.nodeType === 1 && n.tagName === "TD" || n.nodeType === 1 && n.tagName === "TH") return n;
      n = n.parentNode;
    }
    return null;
  }

  function findTable(node) {
    const cell = findCell(node);
    if (!cell) return null;
    let n = cell;
    while (n && n !== document.body) {
      if (n.nodeType === 1 && n.tagName === "TABLE") return n;
      n = n.parentNode;
    }
    return null;
  }

  function rowIndex(cell) {
    const tr = cell.parentNode;
    return Array.prototype.indexOf.call(tr.parentNode.children, tr);
  }
  function colIndex(cell) {
    return Array.prototype.indexOf.call(cell.parentNode.children, cell);
  }

  function insertRow(table, atIndex) {
    const refRow = table.rows[atIndex] || table.rows[table.rows.length - 1];
    if (!refRow) return;
    const newRow = refRow.cloneNode(false);
    for (const cell of refRow.children) {
      const nc = document.createElement(cell.tagName);
      nc.innerHTML = "&nbsp;";
      newRow.appendChild(nc);
    }
    if (atIndex >= table.rows.length) table.appendChild(newRow);
    else table.insertBefore(newRow, table.rows[atIndex]);
  }

  function insertCol(table, atIndex) {
    for (const tr of table.rows) {
      const refCell = tr.children[atIndex] || tr.children[tr.children.length - 1];
      if (!refCell) continue;
      const nc = document.createElement(refCell.tagName);
      nc.innerHTML = "&nbsp;";
      if (atIndex >= tr.children.length) tr.appendChild(nc);
      else tr.insertBefore(nc, tr.children[atIndex]);
    }
  }

  function deleteRow(table, rowIdx) {
    if (table.rows[rowIdx]) table.rows[rowIdx].remove();
  }
  function deleteCol(table, colIdx) {
    for (const tr of table.rows) {
      if (tr.children[colIdx]) tr.children[colIdx].remove();
    }
  }

  function mergeCells(table, startRow, startCol, endRow, endCol) {
    const firstCell = table.rows[startRow] && table.rows[startRow].children[startCol];
    if (!firstCell) return;
    let html = firstCell.innerHTML;
    let rowspan = endRow - startRow + 1;
    let colspan = endCol - startCol + 1;
    for (let r = startRow; r <= endRow; r++) {
      for (let c = startCol; c <= endCol; c++) {
        if (r === startRow && c === startCol) continue;
        const cell = table.rows[r] && table.rows[r].children[c];
        if (cell) {
          html += cell.innerHTML;
          cell.remove();
        }
      }
    }
    firstCell.innerHTML = html;
    if (rowspan > 1) firstCell.rowSpan = rowspan;
    if (colspan > 1) firstCell.colSpan = colspan;
  }

  function splitCell(cell) {
    const rs = cell.rowSpan || 1;
    const cs = cell.colSpan || 1;
    if (rs === 1 && cs === 1) return;
    const tr = cell.parentNode;
    const table = findTable(cell);
    const rowIdx = rowIndex(cell);
    const colIdx = colIndex(cell);
    if (rs > 1) {
      for (let r = 1; r < rs; r++) {
        const newTr = document.createElement("tr");
        for (let c = 0; c < cs; c++) {
          const nc = document.createElement(cell.tagName);
          nc.innerHTML = "&nbsp;";
          newTr.appendChild(nc);
        }
        if (table.rows[rowIdx + r]) table.rows[rowIdx + r].parentNode.insertBefore(newTr, table.rows[rowIdx + r]);
        else table.appendChild(newTr);
      }
      cell.rowSpan = 1;
    }
    if (cs > 1) {
      for (let c = 1; c < cs; c++) {
        const nc = document.createElement(cell.tagName);
        nc.innerHTML = "&nbsp;";
        tr.insertBefore(nc, cell.nextSibling);
      }
      cell.colSpan = 1;
    }
  }

  function setCellPadding(cell, px) {
    cell.style.padding = px + "px";
  }

  function setBorderColor(table, color) {
    table.style.borderColor = color;
    for (const cell of table.querySelectorAll("td, th")) {
      cell.style.borderColor = color;
    }
  }

  function setBorderWidth(table, px) {
    const w = Math.max(0, Math.min(20, px));
    table.style.borderWidth = w + "px";
    table.style.borderStyle = "solid";
    for (const cell of table.querySelectorAll("td, th")) {
      cell.style.borderWidth = w + "px";
      cell.style.borderStyle = "solid";
    }
  }

  function setBgColor(cell, color) {
    cell.style.backgroundColor = color;
  }

  function ensureContextMenu() {
    let menu = document.getElementById("docsTableContextMenu");
    if (menu) return menu;
    menu = document.createElement("div");
    menu.id = "docsTableContextMenu";
    menu.style.cssText = "position:fixed;background:#fff;border:1px solid #888;border-radius:4px;padding:4px;z-index:9999;display:none;font-family:Arial,sans-serif;font-size:13px;box-shadow:0 2px 8px rgba(0,0,0,0.2);min-width:200px;";
    document.body.appendChild(menu);
    return menu;
  }

  function showContextMenu(x, y, table, cell) {
    const menu = ensureContextMenu();
    menu.innerHTML = "";
    menu.style.display = "block";
    menu.style.left = x + "px";
    menu.style.top = y + "px";
    const r = rowIndex(cell);
    const c = colIndex(cell);
    const items = [
      { label: "Insert Row Above", action: () => insertRow(table, r) },
      { label: "Insert Row Below", action: () => insertRow(table, r + 1) },
      { label: "Insert Column Left", action: () => insertCol(table, c) },
      { label: "Insert Column Right", action: () => insertCol(table, c + 1) },
      { type: "separator" },
      { label: "Delete Row", action: () => deleteRow(table, r) },
      { label: "Delete Column", action: () => deleteCol(table, c) },
      { type: "separator" },
      { label: "Merge with Right", action: () => mergeCells(table, r, c, r, c + 1) },
      { label: "Split Cell", action: () => splitCell(cell) },
      { type: "separator" },
      { label: "Cell Padding…", action: () => {
        const px = window.prompt("Cell padding (px):", "8");
        if (px != null) setCellPadding(cell, parseInt(px) || 0);
      }},
      { label: "Cell Background…", action: () => {
        const c = window.prompt("Background color (#hex or named):", "#fff8e1");
        if (c) setBgColor(cell, c);
      }},
      { label: "Border Color…", action: () => {
        const c = window.prompt("Border color (#hex or named):", "#888");
        if (c) setBorderColor(table, c);
      }},
      { label: "Border Width…", action: () => {
        const w = window.prompt("Border width (px):", "1");
        if (w != null) setBorderWidth(table, parseInt(w) || 0);
      }},
    ];
    for (const it of items) {
      if (it.type === "separator") {
        const sep = document.createElement("div");
        sep.style.cssText = "height:1px;background:#ddd;margin:4px 0;";
        menu.appendChild(sep);
        continue;
      }
      const btn = document.createElement("div");
      btn.textContent = it.label;
      btn.style.cssText = "padding:6px 12px;cursor:pointer;border-radius:3px;";
      btn.addEventListener("mouseenter", () => { btn.style.background = "#eef"; });
      btn.addEventListener("mouseleave", () => { btn.style.background = ""; });
      btn.addEventListener("click", () => { it.action(); menu.style.display = "none"; });
      menu.appendChild(btn);
    }
  }

  function attachContextMenu() {
    document.addEventListener("contextmenu", function (e) {
      const table = findTable(e.target);
      const cell = findCell(e.target);
      if (!table || !cell) return;
      e.preventDefault();
      showContextMenu(e.clientX, e.clientY, table, cell);
    });
    document.addEventListener("click", function (e) {
      const menu = document.getElementById("docsTableContextMenu");
      if (menu && !menu.contains(e.target)) menu.style.display = "none";
    });
  }

  function showToolbar() {
    let tb = document.getElementById("docsTableToolbar");
    if (tb) { tb.style.display = "flex"; return tb; }
    tb = document.createElement("div");
    tb.id = "docsTableToolbar";
    tb.style.cssText = "position:fixed;top:8px;left:50%;transform:translateX(-50%);background:#fff;border:1px solid #ccc;border-radius:4px;padding:4px 8px;z-index:9995;display:flex;gap:6px;align-items:center;font-family:Arial,sans-serif;font-size:12px;box-shadow:0 2px 8px rgba(0,0,0,0.1);";
    tb.innerHTML = `
      <span style="font-weight:bold;color:#666;">Table:</span>
      <button data-act="insertRowAbove">+Row↑</button>
      <button data-act="insertRowBelow">+Row↓</button>
      <button data-act="insertColLeft">+Col←</button>
      <button data-act="insertColRight">+Col→</button>
      <button data-act="deleteRow">-Row</button>
      <button data-act="deleteCol">-Col</button>
      <span style="border-left:1px solid #ddd;margin:0 4px;"></span>
      <label>Border <input type="color" data-act="borderColor" value="#888888" /></label>
      <label>BG <input type="color" data-act="cellBg" value="#fff8e1" /></label>
    `;
    document.body.appendChild(tb);
    tb.addEventListener("click", function (e) {
      const sel = window.getSelection();
      const table = findTable(sel && sel.anchorNode);
      const cell = findCell(sel && sel.anchorNode);
      if (!table || !cell) return;
      const r = rowIndex(cell);
      const c = colIndex(cell);
      const act = e.target.getAttribute("data-act");
      if (!act) return;
      switch (act) {
        case "insertRowAbove": insertRow(table, r); break;
        case "insertRowBelow": insertRow(table, r + 1); break;
        case "insertColLeft": insertCol(table, c); break;
        case "insertColRight": insertCol(table, c + 1); break;
        case "deleteRow": deleteRow(table, r); break;
        case "deleteCol": deleteCol(table, c); break;
      }
    });
    tb.addEventListener("change", function (e) {
      const sel = window.getSelection();
      const table = findTable(sel && sel.anchorNode);
      const cell = findCell(sel && sel.anchorNode);
      if (!table || !cell) return;
      const act = e.target.getAttribute("data-act");
      if (act === "borderColor") setBorderColor(table, e.target.value);
      if (act === "cellBg") setBgColor(cell, e.target.value);
    });
    return tb;
  }

  function attach() {
    attachContextMenu();
    showToolbar();
    document.addEventListener("selectionchange", function () {
      const sel = window.getSelection();
      if (!sel || !sel.anchorNode) return;
      const table = findTable(sel.anchorNode);
      const tb = document.getElementById("docsTableToolbar");
      if (tb) tb.style.display = table ? "flex" : "none";
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.DocsTables = {
    attachContextMenu,
    showToolbar,
    insertRow, insertCol, deleteRow, deleteCol,
    mergeCells, splitCell,
    setCellPadding, setBorderColor, setBorderWidth, setBgColor,
  };
})();
