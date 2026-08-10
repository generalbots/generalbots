"use strict";
/* Sheet advanced module: 04_undo — undo/redo stack, fill shortcuts, navigation shortcuts */

(function () {
  const MAX_UNDO = 100;
  let undoStack = [];
  let redoStack = [];
  let pendingBulk = null;

  function grid() {
    return window.SheetVirtualGrid || null;
  }

  function api() {
    return window.SheetAPI || null;
  }

  function t(key, fallback) {
    if (window.SheetI18n && window.SheetI18n.t) return window.SheetI18n.t(key);
    return fallback || key;
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

  function parseRef(ref) {
    const m = String(ref).match(/^([A-Z]+)(\d+)$/);
    if (!m) return null;
    let n = 0;
    for (let i = 0; i < m[1].length; i++) n = n * 26 + (m[1].charCodeAt(i) - 64);
    return { row: parseInt(m[2], 10) - 1, col: n - 1 };
  }

  function snapshot(keys) {
    const g = grid();
    const snap = {};
    for (const k of keys) {
      const d = g.cells.get(k);
      snap[k] = d ? { value: d.value != null ? String(d.value) : "", formula: d.formula || "" } : null;
    }
    return snap;
  }

  function persistSnapshot(snap) {
    const a = api();
    if (!a) return;
    const updates = [];
    for (const k in snap) {
      const p = k.split(",");
      const row = parseInt(p[0], 10);
      const col = parseInt(p[1], 10);
      if (isNaN(row) || isNaN(col)) continue;
      const val = snap[k] === null ? "" : (snap[k].formula || snap[k].value);
      updates.push(a.updateCell(colName(col) + (row + 1), val));
    }
    Promise.all(updates).then(function () {
      const g = grid();
      if (g) {
        g.lastRenderedRange = null;
        g.requestRange();
      }
    });
  }

  function applySnapshot(snap) {
    const g = grid();
    for (const k in snap) {
      if (snap[k] === null) {
        g.cells.set(k, { value: "" });
      } else {
        const cur = g.cells.get(k) || {};
        g.cells.set(k, { value: snap[k].value, formula: snap[k].formula || undefined });
        if (!cur.value && !cur.formula && snap[k].value) g.cells.set(k, { value: snap[k].value, formula: snap[k].formula || undefined });
      }
    }
    g.lastRenderedRange = null;
    g.requestRange();
  }

  function pushEntry(label, oldSnap, newSnap) {
    undoStack.push({ label: label, old: oldSnap, new: newSnap });
    if (undoStack.length > MAX_UNDO) undoStack.shift();
    redoStack = [];
  }

  window.SheetUndo = {
    beforeBulk: function (label, keys) {
      pendingBulk = { label: label, keys: keys, old: snapshot(keys) };
    },
    recordBulk: function () {
      if (!pendingBulk) return;
      const snap = snapshot(pendingBulk.keys);
      pushEntry(pendingBulk.label, pendingBulk.old, snap);
      pendingBulk = null;
    },
    cancelBulk: function () {
      pendingBulk = null;
    },
    suspend: function () {
      window.__gbSuspendUndo = true;
    },
    resume: function () {
      window.__gbSuspendUndo = false;
    },
    undo: function () {
      const entry = undoStack.pop();
      if (!entry) return false;
      redoStack.push(entry);
      applySnapshot(entry.old);
      persistSnapshot(entry.old);
      updateButtons();
      return true;
    },
    redo: function () {
      const entry = redoStack.pop();
      if (!entry) return false;
      undoStack.push(entry);
      applySnapshot(entry.new);
      persistSnapshot(entry.new);
      updateButtons();
      return true;
    },
    canUndo: function () {
      return undoStack.length > 0;
    },
    canRedo: function () {
      return redoStack.length > 0;
    },
  };

  function updateButtons() {
    const u = document.getElementById("undoBtn");
    const r = document.getElementById("redoBtn");
    if (u) u.style.opacity = window.SheetUndo.canUndo() ? "1" : "0.35";
    if (r) r.style.opacity = window.SheetUndo.canRedo() ? "1" : "0.35";
  }

  function wrapUpdateCell() {
    const a = api();
    if (!a || a.__undoWrapped) return;
    a.__undoWrapped = true;
    const orig = a.updateCell.bind(a);
    a.updateCell = function (ref, value) {
      const suspended = window.__gbSuspendUndo === true;
      const p = parseRef(ref);
      const key = p ? p.row + "," + p.col : null;
      const before = key && grid() ? grid().cells.get(key) : null;
      const strVal = value == null ? "" : String(value);
      if (!suspended && !strVal.startsWith("=") && key && window.SheetCore && window.SheetCore.validateEdit) {
        const check = window.SheetCore.validateEdit(p.row, p.col, strVal);
        if (!check.valid) {
          showToast(check.message || t("toast.invalid_value", "Invalid value"));
          if (grid() && key) {
            const node = grid().bodyInner.querySelector('[data-row="' + p.row + '"][data-col="' + p.col + '"]');
            if (node) {
              const cur = grid().cells.get(key);
              node.textContent = cur && cur.value != null ? cur.value : "";
              node.blur();
            }
          }
          return Promise.resolve(null);
        }
      }
      const res = orig(ref, value);
      if (!suspended && key) {
        const oldSnap = {};
        oldSnap[key] = before ? { value: before.value != null ? String(before.value) : "", formula: before.formula || "" } : null;
        const newValue = value == null ? "" : String(value);
        const newSnap = {};
        newSnap[key] = { value: newValue.startsWith("=") ? "" : newValue, formula: newValue.startsWith("=") ? newValue : (before ? before.formula || "" : "") };
        res.then(function () {
          pushEntry("Edit", oldSnap, newSnap);
          updateButtons();
        });
      }
      return res;
    };
  }

  function showToast(msg) {
    const id = "ss-validation-toast";
    let toast = document.getElementById(id);
    if (!toast) {
      toast = document.createElement("div");
      toast.id = id;
      toast.style.cssText = "position:fixed;bottom:24px;left:50%;transform:translateX(-50%);background:#dc2626;color:#fff;padding:10px 18px;border-radius:6px;font-size:13px;z-index:10000;box-shadow:0 4px 12px rgba(0,0,0,0.3);";
      document.body.appendChild(toast);
    }
    toast.textContent = msg;
    toast.style.display = "block";
    clearTimeout(toast.__timer);
    toast.__timer = setTimeout(function () { toast.style.display = "none"; }, 2600);
  }

  function hasValue(r, c) {
    const d = grid().cells.get(r + "," + c);
    return !!(d && (d.value || d.formula));
  }

  function jumpEdge(dr, dc) {
    const g = grid();
    if (!g) return;
    let r = g.selectedRow || 0;
    let c = g.selectedCol || 0;
    const startHas = hasValue(r, c);
    if (dr !== 0) {
      if (startHas) {
        while (r + dr >= 0 && r + dr < g.totalRows && hasValue(r + dr, c)) r += dr;
      } else {
        while (r + dr >= 0 && r + dr < g.totalRows && !hasValue(r + dr, c)) r += dr;
      }
    }
    if (dc !== 0) {
      if (startHas) {
        while (c + dc >= 0 && c + dc < g.totalCols && hasValue(r, c + dc)) c += dc;
      } else {
        while (c + dc >= 0 && c + dc < g.totalCols && !hasValue(r, c + dc)) c += dc;
      }
    }
    g.selectCell(r, c);
  }

  function fillDirection(dr, dc) {
    const g = grid();
    if (!g) return;
    if (g.selectedRow == null || g.selectedCol == null) return;
    const sel = window.SheetAdvanced && window.SheetAdvanced.getSelection ? window.SheetAdvanced.getSelection() : null;
    const r1 = sel ? sel.startRow : g.selectedRow;
    const c1 = sel ? sel.startCol : g.selectedCol;
    const r2 = sel ? sel.endRow : g.selectedRow;
    const c2 = sel ? sel.endCol : g.selectedCol;
    window.__gbSuspendUndo = true;
    const keys = [];
    const updates = [];
    for (let r = r1; r <= r2; r++) {
      for (let c = c1; c <= c2; c++) {
        const srcR = r - dr;
        const srcC = c - dc;
        if (srcR < 0 || srcC < 0) continue;
        const src = g.cells.get(srcR + "," + srcC);
        if (!src) continue;
        const val = src.formula || (src.value != null ? src.value : "");
        g.cells.set(r + "," + c, src.formula ? { value: "", formula: src.formula } : { value: src.value != null ? src.value : "" });
        keys.push(r + "," + c);
        updates.push(api().updateCell(colName(c) + (r + 1), val));
      }
    }
    Promise.all(updates).then(function () {
      window.__gbSuspendUndo = false;
      const s = {};
      keys.forEach(function (k) {
        const d = g.cells.get(k);
        s[k] = d ? { value: d.value != null ? String(d.value) : "", formula: d.formula || "" } : null;
      });
      pushEntry(dr !== 0 ? "Fill Down" : "Fill Right", snapshot(keys), s);
      g.lastRenderedRange = null;
      g.requestRange();
      updateButtons();
    });
  }

  function onKeyDown(e) {
    const g = grid();
    if (!g) return;
    if (g.editingCell != null) return;
    const t = e.target;
    if (t && (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.tagName === "SELECT" || t.isContentEditable)) return;
    const mod = e.ctrlKey || e.metaKey;

    if (mod && e.key.toLowerCase() === "z") {
      e.preventDefault();
      if (e.shiftKey) window.SheetUndo.redo();
      else window.SheetUndo.undo();
      return;
    }
    if (mod && e.key.toLowerCase() === "y") {
      e.preventDefault();
      window.SheetUndo.redo();
      return;
    }
    if (mod && e.key.toLowerCase() === "a") {
      e.preventDefault();
      const sel = {
        startRow: 0, startCol: 0, endRow: g.totalRows - 1, endCol: g.totalCols - 1,
      };
      if (window.SheetAdvanced && window.SheetAdvanced.setRange) {
        window.SheetAdvanced.setRange(0, 0, g.totalRows - 1, g.totalCols - 1);
      }
      g.selectCell(0, 0);
      return;
    }
    if (mod && e.key.toLowerCase() === "d") {
      e.preventDefault();
      fillDirection(1, 0);
      return;
    }
    if (mod && e.key.toLowerCase() === "r") {
      e.preventDefault();
      fillDirection(0, 1);
      return;
    }
    if (mod && e.key === "ArrowDown" && !e.shiftKey) { e.preventDefault(); jumpEdge(1, 0); return; }
    if (mod && e.key === "ArrowUp" && !e.shiftKey) { e.preventDefault(); jumpEdge(-1, 0); return; }
    if (mod && e.key === "ArrowRight" && !e.shiftKey) { e.preventDefault(); jumpEdge(0, 1); return; }
    if (mod && e.key === "ArrowLeft" && !e.shiftKey) { e.preventDefault(); jumpEdge(0, -1); return; }
    if (mod && e.key === "Home") {
      e.preventDefault();
      g.selectCell(g.selectedRow, 0);
      return;
    }
    if (mod && e.key === "End") {
      e.preventDefault();
      g.selectCell(g.selectedRow, g.totalCols - 1);
      return;
    }
    if (mod && e.key === "PageUp") {
      e.preventDefault();
      g.selectCell(Math.max(0, g.selectedRow - 40), g.selectedCol);
      return;
    }
    if (mod && e.key === "PageDown") {
      e.preventDefault();
      g.selectCell(Math.min(g.totalRows - 1, g.selectedRow + 40), g.selectedCol);
      return;
    }
  }

  function wire() {
    if (!api()) {
      setTimeout(wire, 100);
      return;
    }
    if (!document.__saUndoBound) {
      document.__saUndoBound = true;
      document.addEventListener("keydown", onKeyDown, true);
    }
    wrapUpdateCell();
    const ub = document.getElementById("undoBtn");
    const rb = document.getElementById("redoBtn");
    if (ub) {
      ub.onclick = function (e) { e.preventDefault(); window.SheetUndo.undo(); };
      ub.setAttribute("title", "Undo (Ctrl+Z)");
    }
    if (rb) {
      rb.onclick = function (e) { e.preventDefault(); window.SheetUndo.redo(); };
      rb.setAttribute("title", "Redo (Ctrl+Y)");
    }
    updateButtons();
  }

  setTimeout(wire, 0);
})();