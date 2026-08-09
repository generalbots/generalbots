"use strict";
/* Sheet advanced module: 09_validation_editor — dropdown editor for list-validated cells */

(function () {
  let current = null;

  function grid() {
    if (window.SheetCore && window.SheetCore.getGrid) return window.SheetCore.getGrid();
    return window.SheetVirtualGrid || null;
  }

  function api() {
    if (window.SheetCore && window.SheetCore.api) return window.SheetCore.api();
    return window.SheetAPI || null;
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

  function validationFor(row, col) {
    if (window.SheetCore && window.SheetCore.validationForCell) {
      return window.SheetCore.validationForCell(row, col);
    }
    return null;
  }

  function cellNode(row, col) {
    const g = grid();
    if (!g) return null;
    return g.bodyInner.querySelector('[data-row="' + row + '"][data-col="' + col + '"]');
  }

  function showDropdown(row, col, allowed) {
    const g = grid();
    const node = cellNode(row, col);
    if (!g || !node) return;
    closeDropdown();
    const sel = document.createElement("select");
    sel.style.cssText =
      "position:absolute;z-index:50;left:" + node.style.left + ";top:" + node.style.top + ";width:" +
      (parseInt(node.style.width, 10) || 96) + "px;height:" + (parseInt(node.style.height, 10) || 24) +
      "px;background:#0f172a;color:#f8fafc;border:2px solid #3b82f6;font-size:12px;outline:none;";
    allowed.forEach(function (v) {
      const opt = document.createElement("option");
      opt.value = v;
      opt.textContent = v;
      sel.appendChild(opt);
    });
    const cur = node.textContent.trim();
    if (allowed.indexOf(cur) >= 0) sel.value = cur;
    g.bodyInner.appendChild(sel);
    sel.focus();
    current = { select: sel, row: row, col: col, open: true };

    function commit() {
      if (!current || !current.open) return;
      current.open = false;
      const val = sel.value;
      const ref = colName(col) + (row + 1);
      const d = g.cells.get(row + "," + col) || {};
      g.cells.set(row + "," + col, { value: val, formula: d.formula });
      if (api()) api().updateCell(ref, val);
      if (window.SheetCore && window.SheetCore.refreshGrid) window.SheetCore.refreshGrid();
      closeDropdown();
    }

    sel.addEventListener("change", commit);
    sel.addEventListener("blur", closeDropdown);
    sel.addEventListener("keydown", function (e) {
      if (e.key === "Enter") { e.preventDefault(); commit(); }
      if (e.key === "Escape") { e.preventDefault(); closeDropdown(); }
    });
  }

  function closeDropdown() {
    if (current && current.select) {
      current.open = false;
      current.select.remove();
    }
    current = null;
  }

  function onCellMDown(e) {
    if (current) return;
    const t = e.target;
    if (!t || !t.classList || !t.classList.contains("vg-cell")) return;
    const r = parseInt(t.dataset.row, 10);
    const c = parseInt(t.dataset.col, 10);
    if (isNaN(r) || isNaN(c)) return;
    const rule = validationFor(r, c);
    if (!rule || rule.validation_type !== "list" || !rule.allowed_values || !rule.allowed_values.length) return;
    if (e.detail > 1) {
      e.preventDefault();
      showDropdown(r, c, rule.allowed_values);
    }
  }

  function wire() {
    const g = grid();
    if (!g || !g.bodyInner) {
      setTimeout(wire, 100);
      return;
    }
    if (g.bodyInner.__dvBound) return;
    g.bodyInner.__dvBound = true;
    g.bodyInner.addEventListener("mousedown", onCellMDown, true);
    document.addEventListener("mousedown", function (e) {
      if (current && !current.select.contains(e.target)) closeDropdown();
    }, true);
  }

  window.SheetValidationEditor = { show: showDropdown, close: closeDropdown };

  if (window.SheetCore) {
    window.SheetCore.validationEditorShow = showDropdown;
  }

  setTimeout(wire, 0);
})();