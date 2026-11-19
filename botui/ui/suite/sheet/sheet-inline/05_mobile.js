"use strict";
/* #2515 — Sheet touch editing contract.
   Loaded last so every existing grid/sidebar module is already registered.
   Desktop behaviour is untouched: every listener below is gated on either a
   coarse pointer or a phone-sized viewport, and all layout lives in the
   mobile-only blocks of css/04_context_menus.css. */

(function () {
  var app = document.getElementById("sheet-app");
  if (!app) return;

  var main = app.querySelector(".sheet-main");
  var toggle = document.getElementById("sheetSidebarToggle");
  var close = document.getElementById("sheetSidebarClose");
  var backdrop = document.getElementById("sheetSidebarBackdrop");

  function setDrawer(open) {
    if (!main) return;
    main.classList.toggle("sidebar-open", open);
    if (toggle) toggle.setAttribute("aria-expanded", open ? "true" : "false");
  }
  if (toggle) toggle.addEventListener("click", function () {
    setDrawer(!main.classList.contains("sidebar-open"));
  });
  if (close) close.addEventListener("click", function () { setDrawer(false); });
  if (backdrop) backdrop.addEventListener("click", function () { setDrawer(false); });
  document.addEventListener("keydown", function (e) {
    if (e.key === "Escape" && main && main.classList.contains("sidebar-open")) setDrawer(false);
  });

  var isTouch = (function () {
    try {
      return !!(window.matchMedia && (window.matchMedia("(pointer: coarse)").matches ||
        window.matchMedia("(max-width: 760px)").matches));
    } catch (e) { return false; }
  })();
  if (!isTouch) return;

  function activeGrid() {
    var D = window.SheetDefaults;
    var inst = D && D.getActiveSheetInstance && D.getActiveSheetInstance();
    return (inst && inst.grid) || window.SheetVirtualGrid || null;
  }

  // Tap to select and edit. The old grid only listened for mousedown/dblclick,
  // so a finger tap either did nothing or selected a neighbouring 24px cell.
  // A short timer lets the synthetic click land before editing begins, which
  // keeps blur-commit of the previously edited cell intact.
  var tapTimer = null;
  var longPressTimer = null;
  var downPoint = null;

  app.addEventListener("pointerdown", function (e) {
    if (e.pointerType !== "touch") return;
    var cell = e.target.closest ? e.target.closest(".vg-cell") : null;
    if (!cell) return;
    var startX = e.clientX;
    var startY = e.clientY;
    downPoint = { x: startX, y: startY, cell: cell };
    if (longPressTimer) clearTimeout(longPressTimer);
    longPressTimer = setTimeout(function () {
      longPressTimer = null;
      if (tapTimer) { clearTimeout(tapTimer); tapTimer = null; }
      cell.dispatchEvent(new MouseEvent("contextmenu", {
        bubbles: true, cancelable: true, clientX: startX, clientY: startY
      }));
      downPoint = null;
    }, 550);
  }, true);

  app.addEventListener("pointerup", function (e) {
    if (e.pointerType !== "touch" || !downPoint) return;
    if (longPressTimer) { clearTimeout(longPressTimer); longPressTimer = null; }
    var cell = downPoint.cell;
    var startX = downPoint.x;
    var startY = downPoint.y;
    downPoint = null;
    var dx = Math.abs(e.clientX - startX);
    var dy = Math.abs(e.clientY - startY);
    if (dx > 12 || dy > 12 || !cell || !cell.isConnected) return;
    if (tapTimer) clearTimeout(tapTimer);
    tapTimer = setTimeout(function () {
      tapTimer = null;
      var grid = activeGrid();
      var row = parseInt(cell.dataset.row, 10);
      var col = parseInt(cell.dataset.col, 10);
      if (!grid || isNaN(row) || isNaN(col)) return;
      if (grid.editingCell && grid.editingCell !== cell) grid.editingCell.blur();
      grid.selectCell(row, col);
      if (grid.startEditingCell) grid.startEditingCell(cell);
    }, 160);
  }, true);

  app.addEventListener("pointercancel", function () {
    if (longPressTimer) { clearTimeout(longPressTimer); longPressTimer = null; }
    if (tapTimer) { clearTimeout(tapTimer); tapTimer = null; }
    downPoint = null;
  }, true);

  // Two fingers pan/pinch the grid canvas: they must never select a cell.
  // CSS `zoom` keeps the virtual geometry, hit-testing and scrollbars in the
  // same coordinate space, unlike a transform which would desync them.
  var pointers = {};
  var pinchStart = 0;
  var zoomStart = 1;
  var content = app.querySelector(".sheet-content");

  function activePointers() {
    return Object.keys(pointers).length;
  }
  function distance() {
    var pts = Object.keys(pointers).map(function (k) { return pointers[k]; });
    if (pts.length < 2) return 0;
    return Math.sqrt(Math.pow(pts[0].x - pts[1].x, 2) + Math.pow(pts[0].y - pts[1].y, 2));
  }

  content.addEventListener("pointerdown", function (e) {
    if (e.pointerType !== "touch") return;
    pointers[e.pointerId] = { x: e.clientX, y: e.clientY };
    if (activePointers() === 2) {
      if (tapTimer) { clearTimeout(tapTimer); tapTimer = null; }
      if (longPressTimer) { clearTimeout(longPressTimer); longPressTimer = null; }
      downPoint = null;
      pinchStart = distance();
      zoomStart = parseFloat(content.style.getPropertyValue("--sheet-touch-zoom")) || 1;
    }
  }, true);

  content.addEventListener("pointermove", function (e) {
    if (e.pointerType !== "touch" || !pointers[e.pointerId]) return;
    pointers[e.pointerId] = { x: e.clientX, y: e.clientY };
    if (activePointers() < 2 || !pinchStart) return;
    e.preventDefault();
    var next = zoomStart * (distance() / pinchStart);
    content.style.setProperty("--sheet-touch-zoom", String(Math.max(1, Math.min(3, next))));
  }, { passive: false, capture: true });

  function clearPointer(e) {
    delete pointers[e.pointerId];
    if (activePointers() < 2) pinchStart = 0;
  }
  content.addEventListener("pointerup", clearPointer, true);
  content.addEventListener("pointercancel", clearPointer, true);

  // Double-tap the sheet name area resets zoom — discoverable without adding
  // a new English label or a new locale key.
  var nameInput = document.getElementById("sheetName");
  if (nameInput) nameInput.addEventListener("dblclick", function () {
    content.style.setProperty("--sheet-touch-zoom", "1");
  });
})();
