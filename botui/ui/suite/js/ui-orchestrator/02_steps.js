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
  function waitMs(ms) {
    return new Promise(function (resolve) { setTimeout(resolve, ms); });
  }

  function getElementRect(el) {
    var r = el.getBoundingClientRect();
    return { left: r.left, top: r.top, width: r.width, height: r.height };
  }

  function openApp(appId) {
    return new Promise(function (resolve) {
      var reg = (window.APPS_REGISTRY || []);
      var app = null;
      for (var i = 0; i < reg.length; i++) {
        if (reg[i].id === appId) { app = reg[i]; break; }
      }
      if (!app) {
        var all = document.querySelectorAll(".app-item[data-app]");
        for (var j = 0; j < all.length; j++) {
          if (all[j].getAttribute("data-app") === appId) {
            all[j].click();
            resolve(true);
            return;
          }
        }
        // #1386c — apps pruned from the desktop registry (Editor, Terminal,
        // Database, …) are Vibe workbench panes, but they still open as real
        // windows via openDeepLink (it falls back to /suite/partials/<id>.html).
        // Try that before declaring the app unavailable.
        if (window.WindowManager && typeof window.WindowManager.openDeepLink === "function") {
          try {
            window.WindowManager.openDeepLink(appId, {}, { ownerId: "chat" });
            resolve(true);
            return;
          } catch (e) {}
        }
        resolve(false);
        return;
      }
      if (!window.WindowManager) { resolve(false); return; }
      window.WindowManager.open(appId, app.title || appId, "");
      if (window.WindowManager._injectBodyContent && app.hxGet) {
        fetch(app.hxGet)
          .then(function (r) { return r.ok ? r.text() : ""; })
          .then(function (html) {
            var body = document.getElementById("window-body-" + appId);
            if (body && html) {
              window.WindowManager._injectBodyContent(appId, html);
            }
            resolve(true);
          })
          .catch(function () { resolve(true); });
      } else {
        resolve(true);
      }
    });
  }

  function stepOpen(step, plan) {
    var appId = step.app || plan.app || "chat";
    // #2184 — when the plan is going to write into this app, ask for an EMPTY
    // surface (params.blank) instead of the app's default landing state (Docs
    // otherwise reopens the most recent document and the fill lands on the
    // wrong one). The app reads it from __gbAppParams__.
    var willWrite = !!(plan && Array.isArray(plan.steps) && plan.steps.some(function (s) {
      return s && (s.op === "fill" || s.op === "cell");
    }));
    if (willWrite && step.blank !== false &&
        window.WindowManager && typeof window.WindowManager.openDeepLink === "function") {
      var params = Object.assign({}, step.params || {}, { blank: true });
      window.WindowManager.openDeepLink(appId, params, { ownerId: "chat" });
      S.currentApp = appId;
      reportStep("Opened " + appId + " (blank)", "done");
      // Wait for the WRITE TARGET, not just any control: toolbar selects and
      // buttons appear with the window shell, before the app's editor (and its
      // [data-gb-field] surface) exists, so a fill could race the boot and
      // report "field not found" (#2184).
      return waitFor(function () {
        var win = document.getElementById("window-body-" + appId);
        return win && win.querySelector("[data-gb-field], article[contenteditable], textarea[contenteditable]");
      }, STEP_TIMEOUT_MS)
        .then(function (found) {
          if (found) return waitMs(300).then(function () { return true; });
          return waitFor(function () {
            var win = document.getElementById("window-body-" + appId);
            return win && win.querySelector("input, select, textarea, button");
          }, STEP_TIMEOUT_MS).then(function () { return waitMs(600).then(function () { return true; }); });
        });
    }
    return openApp(appId).then(function (ok) {
      S.currentApp = appId;
      reportStep("Opened " + appId, ok ? "done" : "warn");
      if (!ok) {
        reportStep("App '" + appId + "' not found — aborting plan", "warn");
        return false;
      }
      return waitFor(function () {
        var appWin = document.getElementById("window-" + appId);
        return appWin && appWin.querySelector("input, select, textarea, button");
      }, STEP_TIMEOUT_MS).then(function () {
        // waitMs resolves undefined; a step must resolve true or the chain
        // counts it as a failure even though the window opened.
        return waitMs(600).then(function () { return true; });
      });
    });
  }

  function stepClick(step) {
    var el = findClickable(step.label || "");
    if (!el) {
      reportStep("Click '" + (step.label || "?") + "' — element not found", "warn");
      return Promise.resolve(false);
    }
    var rect = getElementRect(el);
    return new Promise(function (resolve) {
      moveCursorTo(rect, function () {
        highlightFlash(el);
        pulseClick();
        try {
          el.click();
        } catch (e) {
          try {
            el.dispatchEvent(new MouseEvent("click", { bubbles: true }));
          } catch (e2) {}
        }
        reportStep("Clicked '" + (visibleText(el) || step.label || "") + "'", "done");
        setTimeout(function () {
          S.focusRing.style.display = "none";
          resolve(true);
        }, 500);
      });
    });
  }

  function stepFill(step) {
    var el = findField(step.field || "");
    if (!el) {
      // Single-field apps (Notes' sticky note, small panels) have no label the
      // plan can name: the whole surface is one editable area. Filling that is
      // what the user asked for, so use it instead of failing the plan with
      // "Field not found" (#2188 NOTES-1).
      var scopeEl = fieldScope();
      var editables = Array.prototype.slice.call(
        scopeEl.querySelectorAll(
          "input:not([type=hidden]):not([type=checkbox]):not([type=radio]):not([type=submit]):not([type=button]), textarea, [contenteditable=true], [contenteditable='']"
        )
      ).filter(function (candidate) {
        return !candidate.disabled && candidate.readOnly !== true &&
          (candidate.offsetParent !== null || candidate.getClientRects().length > 0);
      });
      if (editables.length === 1) {
        el = editables[0];
        reportStep(
          "Field '" + (step.field || "?") + "' not named — using the only editable field",
          "warn"
        );
      }
    }
    if (!el) {
      reportStep("Field '" + (step.field || "?") + "' not found", "warn");
      return Promise.resolve(false);
    }
    var rect = getElementRect(el);
    var value = step.value || "";
    var isEditable = el.isContentEditable === true;
    return new Promise(function (resolve) {
      moveCursorTo(rect, function () {
        try { el.focus(); } catch (e) {}
        // #2184 — document apps expose a structured fill hook: a single hint
        // step draws a whole document (headings, bullets, paragraphs) instead
        // of dumping the value as one blob of text.
        if (typeof el.__gbFill === "function") {
          try {
            el.__gbFill(value);
            reportStep("Filled '" + (step.field || "?") + "'", "done");
            setTimeout(function () {
              S.focusRing.style.display = "none";
              resolve(true);
            }, 300);
            return;
          } catch (e) {}
        }
        if (isEditable) {
          el.textContent = value;
          el.dispatchEvent(new Event("input", { bubbles: true }));
          el.dispatchEvent(new Event("change", { bubbles: true }));
          syncHiddenField(el, value);
          reportStep("Filled '" + (step.field || "?") + "' = " + value, "done");
          setTimeout(function () {
            S.focusRing.style.display = "none";
            resolve(true);
          }, 300);
          return;
        }
        var i = 0;
        el.value = "";
        function typeNext() {
          if (i >= value.length) {
            el.dispatchEvent(new Event("input", { bubbles: true }));
            el.dispatchEvent(new Event("change", { bubbles: true }));
            reportStep("Filled '" + (step.field || "?") + "' = " + value, "done");
            setTimeout(function () {
              S.focusRing.style.display = "none";
              resolve(true);
            }, 300);
            return;
          }
          el.value += value[i];
          el.dispatchEvent(new Event("input", { bubbles: true }));
          i++;
          setTimeout(typeNext, TYPE_DELAY_MS);
        }
        typeNext();
      });
    });
  }

  function syncHiddenField(el, value) {
    var container = el.closest("form, dialog, .modal-content") || document;
    var hidden = container.querySelector('input[type=hidden][name="' + (el.name || "body") + '"]');
    if (!hidden && container === document) {
      var allHidden = document.querySelectorAll('input[type=hidden]');
      for (var i = 0; i < allHidden.length; i++) {
        if (allHidden[i].name === (el.name || "body")) { hidden = allHidden[i]; break; }
      }
    }
    if (!hidden) {
      var id = el.id || "";
      if (id) {
        var byId = document.getElementById(id + "-hidden");
        if (byId && byId.type === "hidden") hidden = byId;
      }
    }
    if (hidden) hidden.value = value;
  }

  function stepSelect(step) {
    var el = findField(step.field || "");
    if (!el || el.tagName !== "SELECT") {
      // Section pickers in real apps are anchors/buttons (Settings nav,
      // tabs, chips), not <select>s. The LLM legitimately plans
      // `select "Profile"`, so fall back to clicking the match instead of
      // failing the whole plan with "Select not found" (#2219).
      var wanted = step.value || step.field || "";
      var clickable = findClickable(wanted) ||
        (step.field && step.value ? findClickable(step.field) : null);
      if (clickable) {
        var targetRect = getElementRect(clickable);
        var label = step.field || wanted;
        return new Promise(function (resolve) {
          moveCursorTo(targetRect, function () {
            try { clickable.click(); } catch (e) { /* detached */ }
            reportStep("Selected '" + label + "'", "done");
            setTimeout(function () {
              S.focusRing.style.display = "none";
              resolve(true);
            }, 400);
          });
        });
      }
      reportStep("Select '" + (step.field || "?") + "' not found", "warn");
      return Promise.resolve(false);
    }
    var rect = getElementRect(el);
    return new Promise(function (resolve) {
      moveCursorTo(rect, function () {
        var matched = false;
        for (var i = 0; i < el.options.length; i++) {
          var opt = el.options[i];
          var t = normalize(opt.textContent || opt.value);
          var target = normalize(step.value || "");
          if (t === target || t.indexOf(target) !== -1 || target.indexOf(t) !== -1) {
            el.selectedIndex = i;
            matched = true;
            break;
          }
        }
        if (!matched && step.value) {
          el.value = step.value;
          matched = el.value === step.value;
        }
        el.dispatchEvent(new Event("change", { bubbles: true }));
        reportStep(
          "Selected '" + (step.field || "?") + "' = " + (step.value || ""),
          matched ? "done" : "warn"
        );
        setTimeout(function () {
          S.focusRing.style.display = "none";
          resolve(matched);
        }, 300);
      });
    });
  }

  function stepSubmit(step) {
    var scope = fieldScope();
    var form = scope.querySelector("form");
    if (!form) {
      // Weather, calculator and other panel apps run their action from a
      // plain button instead of a <form>. Click the app-scoped action button
      // rather than falling back to the chat form in the rest of the page.
      if (scope !== document) {
        var buttons = Array.prototype.slice.call(
          scope.querySelectorAll("button, input[type=button], input[type=submit]")
        ).filter(function (button) {
          var rect = button.getBoundingClientRect();
          return rect.width > 0 && rect.height > 0 && !button.disabled;
        });
        var action = buttons.filter(function (button) {
          return /search|go|submit|apply|save|send|run|refresh/i.test(
            (button.textContent || button.value || "")
          );
        })[0] || buttons[0];
        if (action) {
          var actionRect = getElementRect(action);
          return new Promise(function (resolve) {
            moveCursorTo(actionRect, function () {
              highlightFlash(action);
              pulseClick();
              try {
                action.click();
              } catch (e) {
                action.dispatchEvent(new MouseEvent("click", { bubbles: true }));
              }
              reportStep("Clicked '" + (visibleText(action) || "action") + "'", "done");
              setTimeout(function () {
                S.focusRing.style.display = "none";
                resolve(true);
              }, 800);
            });
          });
        }
      }
      reportStep("No form found to submit", "warn");
      return Promise.resolve(false);
    }
    var rect = getElementRect(form);
    var submitBtn = null;
    var btns = form.querySelectorAll("button[type=submit], input[type=submit]");
    if (btns.length) submitBtn = btns[0];
    return new Promise(function (resolve) {
      moveCursorTo(rect, function () {
        reportStep("Submitting form…", "info");
        if (submitBtn) {
          highlightFlash(submitBtn);
          pulseClick();
          try { submitBtn.click(); } catch (e) {}
        }
        try {
          form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
        } catch (e) {}
        setTimeout(function () {
          S.focusRing.style.display = "none";
          resolve(true);
        }, 800);
      });
    });
  }

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
          // #2246 (SHEET-03) — a cell step may also carry a fill colour. The
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
          if (background || foreground) {
            var style = {};
            if (background) style.background = background;
            if (foreground) style.color = foreground;
            // The sheet app registers its grid asynchronously: retry briefly
            // instead of dropping the colour when the plan outruns the boot.
            var paint = function (attempt) {
              var grid = window.SheetDefaults && window.SheetDefaults.virtualGrid;
              if (!grid || typeof grid.formatSelectedCell !== "function") {
                if (attempt < 15) {
                  setTimeout(function () { paint(attempt + 1); }, 200);
                }
                return;
              }
              try {
                grid.selectedRow = parsed.row;
                grid.selectedCol = parsed.col;
                grid.formatSelectedCell(style);
                reportStep(
                  "Cell '" + parsed.ref + "' style " + JSON.stringify(style),
                  "done"
                );
              } catch (e) {
                reportStep("Cell '" + parsed.ref + "' style failed", "warn");
              }
            };
            paint(0);
          }
          setTimeout(function () {
            S.focusRing.style.display = "none";
            resolve(true);
          }, 500);
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
