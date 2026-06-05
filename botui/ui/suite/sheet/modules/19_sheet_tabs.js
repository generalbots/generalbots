"use strict";

/**
 * Module 19: Sheet tab management for Sheet.
 * Adds click handler on the .tab-menu-btn (▼) of each tab to show a
 * popup menu with: Rename, Duplicate, Delete, Move Left, Move Right,
 * Color. Implements renameWorksheet, deleteWorksheet, duplicateWorksheet,
 * moveWorksheet, setWorksheetColor as commands (compatible with the
 * SheetUndo command-pattern module 16).
 *
 * Public API: window.SheetTabs = {
 *   rename(i, name), remove(i), duplicate(i), move(i, dir), setColor(i, c),
 *   add(name), list(), current()
 * }.
 */

(function () {
  const DEFAULT_COLORS = ["#1a73e8", "#34a853", "#fbbc04", "#ea4335", "#9334e6", "#00acc1", "#f06292"];

  function getState() { return window.state || null; }

  function getWorksheets() {
    const s = getState();
    if (!s || !s.worksheets) return null;
    return s.worksheets;
  }

  function closeAll() {
    document.querySelectorAll(".sheet-tab-menu").forEach((el) => el.remove());
    document.removeEventListener("click", onDocClick, true);
  }

  function onDocClick(e) {
    const menu = document.querySelector(".sheet-tab-menu");
    if (!menu) return;
    if (e.target.closest(".sheet-tab-menu")) return;
    if (e.target.closest(".tab-menu-btn")) return;
    closeAll();
  }

  function showMenu(index, anchor) {
    closeAll();
    const rect = anchor.getBoundingClientRect();
    const menu = document.createElement("div");
    menu.className = "sheet-tab-menu";
    menu.style.cssText = "position:fixed;background:#fff;border:1px solid #888;border-radius:4px;padding:4px;z-index:9999;min-width:160px;box-shadow:0 2px 8px rgba(0,0,0,0.2);font-size:13px;";
    menu.style.left = rect.left + "px";
    menu.style.top = rect.bottom + "px";
    const items = [
      { label: "Rename", action: () => promptRename(index) },
      { label: "Duplicate", action: () => duplicate(index) },
      { label: "Delete", action: () => remove(index) },
      { label: "Move Left", action: () => move(index, -1) },
      { label: "Move Right", action: () => move(index, 1) },
      { label: "Color…", action: () => promptColor(index) },
    ];
    for (const it of items) {
      const btn = document.createElement("div");
      btn.className = "sheet-tab-menu-item";
      btn.textContent = it.label;
      btn.style.cssText = "padding:6px 12px;cursor:pointer;border-radius:3px;";
      btn.addEventListener("mouseenter", () => { btn.style.background = "#eef"; });
      btn.addEventListener("mouseleave", () => { btn.style.background = ""; });
      btn.addEventListener("click", () => { closeAll(); it.action(); });
      menu.appendChild(btn);
    }
    document.body.appendChild(menu);
    setTimeout(() => document.addEventListener("click", onDocClick, true), 0);
  }

  function promptRename(index) {
    const ws = getWorksheets();
    if (!ws || !ws[index]) return;
    const current = ws[index].name || ("Sheet" + (index + 1));
    const next = window.prompt("Rename sheet:", current);
    if (next != null && next !== "") rename(index, next);
  }

  function promptColor(index) {
    const ws = getWorksheets();
    if (!ws || !ws[index]) return;
    const palette = document.createElement("div");
    palette.className = "sheet-color-palette";
    palette.style.cssText = "position:fixed;background:#fff;border:1px solid #888;border-radius:4px;padding:8px;z-index:9999;display:flex;gap:6px;";
    const ws1 = getWorksheets();
    for (const c of DEFAULT_COLORS) {
      const dot = document.createElement("div");
      dot.style.cssText = "width:24px;height:24px;border-radius:50%;background:" + c + ";cursor:pointer;border:2px solid #fff;box-shadow:0 0 0 1px #888;";
      dot.addEventListener("click", () => {
        setColor(index, c);
        document.body.removeChild(palette);
      });
      palette.appendChild(dot);
    }
    const clear = document.createElement("div");
    clear.textContent = "✕";
    clear.style.cssText = "width:24px;height:24px;line-height:24px;text-align:center;cursor:pointer;border:1px solid #888;border-radius:50%;";
    clear.addEventListener("click", () => {
      setColor(index, null);
      document.body.removeChild(palette);
    });
    palette.appendChild(clear);
    const tabs = document.getElementById("worksheetTabs");
    if (tabs) {
      const r = tabs.getBoundingClientRect();
      palette.style.left = r.left + "px";
      palette.style.top = r.top - 40 + "px";
    }
    document.body.appendChild(palette);
  }

  function rename(index, name) {
    const ws = getWorksheets();
    if (!ws || !ws[index]) return false;
    const oldName = ws[index].name || ("Sheet" + (index + 1));
    ws[index].name = name;
    if (window.SheetUndo && window.SheetUndo.execute) {
      window.SheetUndo.execute({
        id: "rename-" + Date.now(),
        type: "rename",
        name: "Rename sheet",
        index, oldName, newName: name,
        do() { if (ws[this.index]) ws[this.index].name = this.newName; rerenderTabs(); },
        undo() { if (ws[this.index]) ws[this.index].name = this.oldName; rerenderTabs(); },
      });
    }
    rerenderTabs();
    return true;
  }

  function remove(index) {
    const ws = getWorksheets();
    if (!ws || ws.length <= 1) {
      window.alert("Cannot delete the only sheet.");
      return false;
    }
    if (!window.confirm("Delete sheet '" + (ws[index].name || ("Sheet" + (index + 1))) + "'?")) return false;
    const removed = ws[index];
    const s = getState();
    const oldActive = s.activeWorksheet;
    ws.splice(index, 1);
    if (s.activeWorksheet >= ws.length) s.activeWorksheet = ws.length - 1;
    if (window.SheetUndo && window.SheetUndo.execute) {
      window.SheetUndo.execute({
        id: "removeSheet-" + Date.now(),
        type: "removeSheet",
        name: "Delete sheet",
        index, removed, oldActive, newActive: s.activeWorksheet,
        do() {
          if (ws[this.index] !== this.removed) ws.splice(this.index, 0, this.removed);
          rerenderTabs();
        },
        undo() {
          ws.splice(this.index, 0, this.removed);
          s.activeWorksheet = this.oldActive;
          rerenderTabs();
        },
      });
    }
    rerenderTabs();
    return true;
  }

  function duplicate(index) {
    const ws = getWorksheets();
    if (!ws || !ws[index]) return false;
    const original = ws[index];
    const copy = JSON.parse(JSON.stringify(original));
    copy.name = (original.name || "Sheet") + " (copy)";
    ws.splice(index + 1, 0, copy);
    const s = getState();
    s.activeWorksheet = index + 1;
    if (window.SheetUndo && window.SheetUndo.execute) {
      window.SheetUndo.execute({
        id: "dupSheet-" + Date.now(),
        type: "duplicateSheet",
        name: "Duplicate sheet",
        insertedAt: index + 1, removed: copy,
        do() { if (ws[this.insertedAt] !== this.removed) ws.splice(this.insertedAt, 0, this.removed); rerenderTabs(); },
        undo() { ws.splice(this.insertedAt, 1); rerenderTabs(); },
      });
    }
    rerenderTabs();
    return true;
  }

  function move(index, direction) {
    const ws = getWorksheets();
    if (!ws) return false;
    const newIdx = index + direction;
    if (newIdx < 0 || newIdx >= ws.length) return false;
    const tmp = ws[index];
    ws[index] = ws[newIdx];
    ws[newIdx] = tmp;
    const s = getState();
    if (s.activeWorksheet === index) s.activeWorksheet = newIdx;
    else if (s.activeWorksheet === newIdx) s.activeWorksheet = index;
    if (window.SheetUndo && window.SheetUndo.execute) {
      const a = newIdx, b = index;
      window.SheetUndo.execute({
        id: "moveSheet-" + Date.now(),
        type: "moveSheet",
        name: "Move sheet",
        from: b, to: a,
        do() { const t = ws[this.from]; ws[this.from] = ws[this.to]; ws[this.to] = t; rerenderTabs(); },
        undo() { const t = ws[this.to]; ws[this.to] = ws[this.from]; ws[this.from] = t; rerenderTabs(); },
      });
    }
    rerenderTabs();
    return true;
  }

  function setColor(index, color) {
    const ws = getWorksheets();
    if (!ws || !ws[index]) return false;
    const old = ws[index].color || null;
    ws[index].color = color;
    if (window.SheetUndo && window.SheetUndo.execute) {
      window.SheetUndo.execute({
        id: "colorSheet-" + Date.now(),
        type: "setSheetColor",
        name: "Sheet color",
        index, oldColor: old, newColor: color,
        do() { if (ws[this.index]) ws[this.index].color = this.newColor; rerenderTabs(); },
        undo() { if (ws[this.index]) ws[this.index].color = this.oldColor; rerenderTabs(); },
      });
    }
    rerenderTabs();
    return true;
  }

  function add(name) {
    const ws = getWorksheets();
    if (!ws) return null;
    const s = getState();
    const idx = ws.length;
    const sheet = {
      name: name || ("Sheet" + (idx + 1)),
      data: {},
      merges: [],
      validations: [],
      colWidths: {},
      rowHeights: {},
    };
    ws.push(sheet);
    s.activeWorksheet = idx;
    if (window.SheetUndo && window.SheetUndo.execute) {
      window.SheetUndo.execute({
        id: "addSheet-" + Date.now(),
        type: "addSheet",
        name: "Add sheet",
        insertedAt: idx, sheet,
        do() { if (ws[this.insertedAt] !== this.sheet) ws.splice(this.insertedAt, 0, this.sheet); rerenderTabs(); },
        undo() { ws.splice(this.insertedAt, 1); rerenderTabs(); },
      });
    }
    rerenderTabs();
    return sheet;
  }

  function list() {
    const ws = getWorksheets();
    if (!ws) return [];
    return ws.map((w, i) => ({ index: i, name: w.name || ("Sheet" + (i + 1)), color: w.color || null }));
  }

  function current() {
    const s = getState();
    if (!s) return null;
    return s.activeWorksheet;
  }

  function rerenderTabs() {
    const ws = getWorksheets();
    const s = getState();
    if (!ws || !s) return;
    const container = document.getElementById("worksheetTabs");
    if (!container) return;
    container.innerHTML = "";
    ws.forEach((sheet, i) => {
      const tab = document.createElement("div");
      tab.className = "sheet-tab" + (i === s.activeWorksheet ? " active" : "");
      tab.setAttribute("data-index", i);
      const label = document.createElement("span");
      label.textContent = sheet.name || ("Sheet" + (i + 1));
      if (sheet.color) label.style.color = sheet.color;
      tab.appendChild(label);
      const btn = document.createElement("button");
      btn.className = "tab-menu-btn";
      btn.textContent = "▼";
      btn.addEventListener("click", (e) => { e.stopPropagation(); showMenu(i, btn); });
      tab.appendChild(btn);
      tab.addEventListener("click", (e) => {
        if (e.target.closest(".tab-menu-btn")) return;
        s.activeWorksheet = i;
        rerenderTabs();
        if (typeof window.rerender === "function") window.rerender();
        else if (typeof window.renderWorksheet === "function") window.renderWorksheet();
      });
      container.appendChild(tab);
    });
  }

  function attach() {
    document.addEventListener("click", function (e) {
      const btn = e.target.closest && e.target.closest(".tab-menu-btn");
      if (btn) {
        const tab = btn.closest(".sheet-tab");
        if (!tab) return;
        const idx = parseInt(tab.getAttribute("data-index"), 10);
        if (!isNaN(idx)) {
          e.preventDefault();
          e.stopPropagation();
          showMenu(idx, btn);
        }
      }
    });
    const addBtn = document.getElementById("addSheetBtn");
    if (addBtn) {
      addBtn.addEventListener("click", function () { add(); });
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.SheetTabs = {
    rename, remove, duplicate, move, setColor, add, list, current, showMenu, rerenderTabs,
  };
})();
