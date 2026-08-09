"use strict";
/* Sheet advanced module: 08_statusbar — live SUM/AVG/COUNT + range ref status bar */

(function () {
  let bar = null;

  function grid() {
    if (window.SheetCore && window.SheetCore.getGrid) return window.SheetCore.getGrid();
    return window.SheetVirtualGrid || null;
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

  function host() {
    if (window.SheetCore && window.SheetCore.getHost) return window.SheetCore.getHost();
    return document.getElementById("sheet-content");
  }

  function ensureBar() {
    if (bar && bar.isConnected) return bar;
    const h = host();
    if (!h) return null;
    bar = document.createElement("div");
    bar.className = "ss-status-bar";
    bar.style.cssText = "display:flex;align-items:center;gap:24px;padding:4px 16px;background:#0f172a;border-top:1px solid #334155;flex-shrink:0;font-size:12px;color:#94a3b8;min-height:26px;";
    bar.innerHTML =
      '<span class="ss-status-range" style="font-family:monospace;font-weight:600;color:#f8fafc;min-width:80px;"></span>' +
      '<span class="ss-status-sum">Σ = <b></b></span>' +
      '<span class="ss-status-avg">AVG = <b></b></span>' +
      '<span class="ss-status-count">COUNT = <b></b></span>';
    h.appendChild(bar);
    return bar;
  }

  function num(v) {
    if (v == null || v === "") return null;
    const n = Number(v);
    return isNaN(n) ? null : n;
  }

  function fmt(n) {
    if (n == null) return "—";
    return Number.isInteger(n) ? String(n) : n.toFixed(2);
  }

  function update() {
    const g = grid();
    const sel = window.SheetAdvanced ? window.SheetAdvanced.getSelection() : null;
    const b = ensureBar();
    if (!g || !sel || !b) return;
    const ref = colName(sel.startCol) + (sel.startRow + 1) + ":" + colName(sel.endCol) + (sel.endRow + 1);
    let sum = 0;
    let sumN = 0;
    let count = 0;
    for (let r = sel.startRow; r <= sel.endRow; r++) {
      for (let c = sel.startCol; c <= sel.endCol; c++) {
        const d = g.cells.get(r + "," + c);
        const v = d ? (d.value != null ? d.value : (d.formula || "")) : "";
        if (v === "" || String(v).indexOf("#") === 0) continue;
        count++;
        const n = num(v);
        if (n != null) {
          sum += n;
          sumN++;
        }
      }
    }
    b.querySelector(".ss-status-range").textContent = ref;
    b.querySelector(".ss-status-sum b").textContent = fmt(sumN ? sum : "—");
    b.querySelector(".ss-status-avg b").textContent = fmt(sumN ? sum / sumN : "—");
    b.querySelector(".ss-status-count b").textContent = String(count);
  }

  function wire() {
    document.addEventListener("gb-sheet-selection", update);
    document.addEventListener("gb-sheet-tab", function () { setTimeout(update, 50); });
    window.addEventListener("resize", update);
    setTimeout(update, 200);
  }

  window.SheetStatusBar = { update: update, wire: wire };

  setTimeout(wire, 0);
})();