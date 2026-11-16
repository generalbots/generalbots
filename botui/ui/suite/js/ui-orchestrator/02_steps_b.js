(function (window, document) {
  "use strict";
  var N = document.__gb_mod_ui_orchestrator;
  if (!N) return;
  var S = N.s;
  var reportStep = N.reportStep;
  var waitFor = N.waitFor;
  var STEP_TIMEOUT_MS = N.STEP_TIMEOUT_MS;
  var findClickable = N.findClickable;
  var moveCursorTo = N.moveCursorTo;
  var highlightFlash = N.highlightFlash;
  var pulseClick = N.pulseClick;
  var visibleText = N.visibleText;
  var findField = N.findField;
  var TYPE_DELAY_MS = N.TYPE_DELAY_MS;
  var normalize = N.normalize;
  var fieldScope = N.fieldScope;
  var waitMs = N.waitMs;
  var getElementRect = N.getElementRect;
  var openApp = N.openApp;
  var stepOpen = N.stepOpen;
  var stepClick = N.stepClick;
  var stepFill = N.stepFill;
  var syncHiddenField = N.syncHiddenField;
  var stepSelect = N.stepSelect;
  var stepSubmit = N.stepSubmit;

  function stepWait(step) {
    return waitMs(Math.min(step.ms || 500, 10000)).then(function () {
      reportStep("Waiting " + (step.ms || 500) + "ms", "info");
      return true;
    });
  }

  function parseCellRef(ref) {
    var m = /^([A-Z]+)([0-9]+)$/.exec((ref || "").toUpperCase());
    if (!m) return null;
    var col = 0;
    for (var i = 0; i < m[1].length; i++) {
      col = col * 26 + (m[1].charCodeAt(i) - 64);
    }
    return { row: parseInt(m[2], 10) - 1, col: col - 1, ref: ref.toUpperCase() };
  }

  function findCellElement(parsed) {
    var scope = fieldScope();
    var cellEl = scope.querySelector(
      '[data-row="' + parsed.row + '"][data-col="' + parsed.col + '"], ' +
      '[data-cell="' + parsed.ref + '"], [data-ref="' + parsed.ref + '"]'
    );
    if (!cellEl && scope !== document) {
      cellEl = document.querySelector(
        '[data-row="' + parsed.row + '"][data-col="' + parsed.col + '"], ' +
        '[data-cell="' + parsed.ref + '"], [data-ref="' + parsed.ref + '"]'
      );
    }
    return cellEl;
  }

  // The sheet grid is virtualized: a cell past the current scroll position
  // (or past totalCols) has no DOM node, so plans that write I4..L4 aborted
  // with "Cell 'I4' not found in app" (#2260). Grow the grid if needed and
  // scroll the target into view before retrying the lookup.
  function revealSheetCell(parsed) {
    var grid = (window.SheetDefaults && window.SheetDefaults.virtualGrid) ||
      window.SheetVirtualGrid;
    if (!grid || !grid.scrollArea) return false;
    if (parsed.col >= grid.totalCols) {
      grid.totalCols = parsed.col + 1;
      grid.__totalColsWidth = null;
    }
    if (parsed.row >= grid.totalRows) grid.totalRows = parsed.row + 1;
    var D = window.SheetDefaults || {};
    var colWidth = D.COL_WIDTH || 100;
    var rowHeight = D.ROW_HEIGHT || 28;
    try {
      grid.scrollArea.scrollLeft = Math.max(0, parsed.col * colWidth - colWidth);
      grid.scrollArea.scrollTop = Math.max(0, parsed.row * rowHeight - rowHeight);
      if (typeof grid.render === "function") grid.render();
    } catch (e) {
      return false;
    }
    return true;
  }

  function stepCell(step, retried) {
    var parsed = parseCellRef(step.cell || "");
    if (!parsed) {
      reportStep("Invalid cell reference '" + (step.cell || "?") + "'", "warn");
      return Promise.resolve(false);
    }
    var cellEl = findCellElement(parsed);
    if (!cellEl) {
      if (!retried && revealSheetCell(parsed)) {
        reportStep("Scrolling cell '" + parsed.ref + "' into view", "info");
        return waitMs(400).then(function () { return stepCell(step, true); });
      }
      reportStep("Cell '" + parsed.ref + "' not found in app", "warn");
      return Promise.resolve(false);
    }
    var rect = getElementRect(cellEl);
    return new Promise(function (resolve) {
      moveCursorTo(rect, function () {
        highlightFlash(cellEl);
        pulseClick();
        try { cellEl.dispatchEvent(new MouseEvent("mousedown", { bubbles: true })); } catch (e) {}
        try { cellEl.dispatchEvent(new MouseEvent("mouseup", { bubbles: true })); } catch (e) {}
        try { cellEl.dispatchEvent(new MouseEvent("click", { bubbles: true })); } catch (e) {}
        setTimeout(function () {
          var formula = document.getElementById("formulaInput");
          if (!formula) {
            reportStep("No formula bar for cell '" + parsed.ref + "'", "warn");
            resolve(false);
            return;
          }
          formula.focus();
          formula.value = step.value || "";
          formula.dispatchEvent(new Event("input", { bubbles: true }));
          formula.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true }));
          reportStep("Cell '" + parsed.ref + "' = " + (step.value || ""), "done");
          // #2246 / #2155 (SHEET-03) — a cell step may carry a fill colour. The
          // sheet colours come from formatSelectedCell (the same path the
          // toolbar uses), so the cell keeps the colour on reload.
          var background = step.background || step.fill_color || step.bg;
          var foreground = step.color || step.text_color;
          // Plans often express a colour as the cell VALUE ("A1 = Red") when the
          // request is 'five cells each with a different colour'. When the value
          // names a real CSS colour and no explicit fill was given, use it as
          // the cell background (the sheet sets a readable text colour itself).
          if (!background && !foreground && step.value) {
            var NAMED_FILLS = {
              amber: "#ffbf00",
              gold: "#ffd700",
              silver: "#c0c0c0",
              peach: "#ffcba4",
              mint: "#98ff98",
              cream: "#fffdd0",
              burgundy: "#800020",
              emerald: "#50c878",
              ruby: "#9b111e",
              sapphire: "#0f52ba",
            };
            var candidate = String(step.value).trim();
            var key = candidate.toLowerCase();
            if (Object.prototype.hasOwnProperty.call(NAMED_FILLS, key)) {
              background = NAMED_FILLS[key];
            } else if (candidate && window.CSS && CSS.supports && CSS.supports("color", candidate)) {
              background = candidate;
            }
          }
          // Paint style as part of the step contract: wait for the grid and the
          // format POST so the step only succeeds when the colour is actually
          // applied (avoids "automation reports done but cell.style is null").
          var paintStyle = function () {
            if (!background && !foreground) return Promise.resolve(true);
            var style = {};
            if (background) style.background = background;
            if (foreground) style.color = foreground;
            var apply = function (attempt) {
              var grid = window.SheetDefaults && window.SheetDefaults.virtualGrid;
              if (!grid || typeof grid.formatSelectedCell !== "function") {
                if (attempt < 20) {
                  return waitMs(200).then(function () { return apply(attempt + 1); });
                }
                reportStep(
                  "Cell '" + parsed.ref + "' style failed — sheet grid not ready",
                  "warn"
                );
                return Promise.resolve(false);
              }
              try {
                grid.selectedRow = parsed.row;
                grid.selectedCol = parsed.col;
                return Promise.resolve(grid.formatSelectedCell(style)).then(function () {
                  reportStep(
                    "Cell '" + parsed.ref + "' style " + JSON.stringify(style),
                    "done"
                  );
                  return true;
                }).catch(function () {
                  reportStep("Cell '" + parsed.ref + "' style failed", "warn");
                  return false;
                });
              } catch (e) {
                reportStep("Cell '" + parsed.ref + "' style failed", "warn");
                return Promise.resolve(false);
              }
            };
            return apply(0);
          };
          // Allow the value commit (keydown Enter) to flush before styling so
          // update_cell and format do not race on a brand-new cell.
          waitMs(350).then(paintStyle).then(function (styleOk) {
            setTimeout(function () {
              S.focusRing.style.display = "none";
              resolve(styleOk);
            }, 200);
          });
        }, 300);
      });
    });
  }

  var EXECUTORS = {
    open: stepOpen,
    click: stepClick,
    fill: stepFill,
    select: stepSelect,
    submit: stepSubmit,
    wait: stepWait,
    cell: stepCell,
  };

  /* ------------------------------------------------------------
     Public API
     ------------------------------------------------------------ */


  N.waitMs = waitMs;
  N.getElementRect = getElementRect;
  N.openApp = openApp;
  N.stepOpen = stepOpen;
  N.stepClick = stepClick;
  N.stepFill = stepFill;
  N.syncHiddenField = syncHiddenField;
  N.stepSelect = stepSelect;
  N.stepSubmit = stepSubmit;
  N.stepWait = stepWait;
  N.parseCellRef = parseCellRef;
  N.stepCell = stepCell;
  N.EXECUTORS = EXECUTORS;
})(window, document);
