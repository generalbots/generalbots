"use strict";
/* Sheet advanced module: 29_audit_overlay — view-only trace arrows and
   show-formulas layer for the local-only formula auditor (#2491). Never writes
   to the workbook; it only swaps rendered text and adds a decorative SVG. */

(function () {
  var MAX_ARROWS = 60;
  var svg = null;
  var showing = false;
  var observer = null;
  var suppress = 0;
  var watchQueued = false;

  function grid() {
    var D = window.SheetDefaults;
    var inst = D && D.getActiveSheetInstance ? D.getActiveSheetInstance() : null;
    return (inst && inst.grid) || window.SheetVirtualGrid || null;
  }
  function bodyInner() {
    var g = grid();
    return g ? g.bodyInner : null;
  }
  function suppressed(fn) {
    suppress++;
    try { return fn(); } finally { suppress--; }
  }
  function scheduleWatch() {
    if (!window.SheetAuditSections || typeof window.SheetAuditSections.updateWatch !== "function") return;
    if (watchQueued || !(window.SheetAuditUI && window.SheetAuditUI.getWatch().length)) return;
    watchQueued = true;
    var raf = window.requestAnimationFrame || function (callback) { return setTimeout(callback, 0); };
    raf(function () {
      watchQueued = false;
      window.SheetAuditSections.updateWatch();
    });
  }

  function onGridMutation() {
    if (suppress > 0) return;
    if (showing) applyShowFormulas();
    scheduleWatch();
  }

  function ensureObserver() {
    var body = bodyInner();
    if (!body || typeof MutationObserver !== "function") return;
    if (observer) observer.disconnect();
    observer = new MutationObserver(onGridMutation);
    observer.observe(body, { childList: true, characterData: true, subtree: true });
  }

  function geometry() {
    var C = window.SheetCore || {};
    return {
      colX: typeof C.colX === "function" ? C.colX : function (c) { return 48 + c * 96; },
      colWidth: typeof C.colWidth === "function" ? C.colWidth : function () { return 96; },
      rowTop: typeof C.rowTop === "function" ? C.rowTop : function (r) { return r * 24; },
      rowHeight: typeof C.rowHeight === "function" ? C.rowHeight : function () { return 24; },
    };
  }
  function centerOf(row, col, geo) {
    return {
      x: geo.colX(col) + geo.colWidth(col) / 2,
      y: geo.rowTop(row) + geo.rowHeight(row) / 2,
    };
  }

  function svgElement(name) {
    return document.createElementNS("http://www.w3.org/2000/svg", name);
  }

  function clearArrows() {
    if (svg && svg.parentNode) {
      suppressed(function () { svg.parentNode.removeChild(svg); });
    }
    svg = null;
  }

  function drawArrows(origin, nodes, direction, sheetIdx) {
    var body = bodyInner();
    if (!body || !origin) return 0;
    clearArrows();
    var geo = geometry();
    var from = centerOf(origin.row, origin.col, geo);
    var canvas = svgElement("svg");
    canvas.setAttribute("class", "sheet-audit-arrows");
    canvas.setAttribute("aria-hidden", "true");
    canvas.setAttribute("width", "100%");
    canvas.setAttribute("height", "100%");
    var defs = svgElement("defs");
    var marker = svgElement("marker");
    marker.setAttribute("id", "sheet-audit-arrowhead");
    marker.setAttribute("markerWidth", "8");
    marker.setAttribute("markerHeight", "8");
    marker.setAttribute("refX", "7");
    marker.setAttribute("refY", "4");
    marker.setAttribute("orient", "auto");
    var head = svgElement("path");
    head.setAttribute("d", "M0,0 L8,4 L0,8 z");
    head.setAttribute("class", "sheet-audit-arrowhead");
    marker.appendChild(head);
    defs.appendChild(marker);
    canvas.appendChild(defs);

    var drawn = 0;
    var activeSheet = typeof sheetIdx === "number" ? sheetIdx : window.SheetAuditUI.activeIndex();
    (nodes || []).forEach(function (item) {
      if (drawn >= MAX_ARROWS || item.id === origin.id || item.sheetIdx !== activeSheet) return;
      var parsed = window.SheetAuditGraph.parseId(item.id);
      var to = centerOf(parsed.row, parsed.col, geo);
      var start = direction === "dependents" ? to : from;
      var end = direction === "dependents" ? from : to;
      var path = svgElement("path");
      path.setAttribute("d", "M" + start.x + "," + start.y + " L" + end.x + "," + end.y);
      path.setAttribute("class", "sheet-audit-arrow" +
        (direction === "dependents" ? " is-dependent" : " is-precedent"));
      path.setAttribute("marker-end", "url(#sheet-audit-arrowhead)");
      canvas.appendChild(path);
      drawn++;
    });
    suppressed(function () { body.appendChild(canvas); });
    svg = canvas;
    return drawn;
  }

  function applyShowFormulas() {
    var body = bodyInner();
    if (!body) return;
    var g = grid();
    suppressed(function () {
      var cells = body.querySelectorAll("[data-row][data-col]");
      Array.prototype.forEach.call(cells, function (node) {
        if (node.contentEditable === "true" || (g && g.editingCell === node)) return;
        if (!node.dataset.formula) {
          if (node.dataset.auditValue != null) {
            node.textContent = node.dataset.auditValue;
            node.classList.remove("sheet-audit-formula-view");
            delete node.dataset.auditValue;
          }
          return;
        }
        if (node.dataset.auditValue == null) node.dataset.auditValue = node.textContent;
        if (node.textContent !== node.dataset.formula) node.textContent = node.dataset.formula;
        node.classList.add("sheet-audit-formula-view");
      });
    });
  }

  function restoreShowFormulas() {
    var body = bodyInner();
    if (!body) return;
    suppressed(function () {
      var cells = body.querySelectorAll("[data-audit-value]");
      Array.prototype.forEach.call(cells, function (node) {
        node.textContent = node.dataset.auditValue;
        node.classList.remove("sheet-audit-formula-view");
        delete node.dataset.auditValue;
      });
    });
  }

  function setShowFormulas(enabled) {
    if (showing === !!enabled) return showing;
    showing = !!enabled;
    if (showing) {
      ensureObserver();
      applyShowFormulas();
    } else {
      restoreShowFormulas();
    }
    window.dispatchEvent(new CustomEvent("gb-sheet-audit-formulas", {
      detail: { showing: showing }, bubbles: true,
    }));
    return showing;
  }
  function toggleShowFormulas() {
    return setShowFormulas(!showing);
  }
  function isShowingFormulas() {
    return showing;
  }

  function chainCellChanges() {
    var g = grid();
    if (!g) return;
    if (!Object.prototype.hasOwnProperty.call(g, "__sheetAuditOriginalCellChanged")) {
      g.__sheetAuditOriginalCellChanged = typeof g.onCellValueChanged === "function"
        ? g.onCellValueChanged : null;
    }
    var original = g.__sheetAuditOriginalCellChanged;
    g.onCellValueChanged = function () {
      if (original) original.apply(this, arguments);
      scheduleWatch();
    };
    ensureObserver();
  }

  function boot() {
    chainCellChanges();
    ensureObserver();
  }

  window.SheetAuditOverlay = {
    boot: boot,
    drawArrows: drawArrows,
    clearArrows: clearArrows,
    setShowFormulas: setShowFormulas,
    toggleShowFormulas: toggleShowFormulas,
    isShowingFormulas: isShowingFormulas,
    scheduleWatch: scheduleWatch,
    chainCellChanges: chainCellChanges,
  };
})();
