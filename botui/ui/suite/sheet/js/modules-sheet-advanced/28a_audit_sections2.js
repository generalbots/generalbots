"use strict";
/* Sheet advanced module: 28a_audit_sections2 — watch, errors, cycles, formula
   text and dependency map panels for the local-only auditor (#2491). */

(function () {
  var S = window.SheetAuditSections || (window.SheetAuditSections = {});
  var MAX_EVAL_CELLS = 400;

  function node(tag, className, text) {
    return window.SheetAuditUI.element(tag, className, text);
  }
  function button(label, handler, className) {
    var b = node("button", className || "sheet-audit-btn", label);
    b.type = "button";
    b.addEventListener("click", handler);
    return b;
  }
  function field(labelText, input) {
    var wrap = node("label", "sheet-audit-field");
    wrap.appendChild(node("span", "sheet-audit-field-label", labelText));
    wrap.appendChild(input);
    return wrap;
  }
  function clear(container) {
    while (container && container.firstChild) container.removeChild(container.firstChild);
  }
  function status(ctx, message) {
    window.SheetAuditUI.setStatus(message);
  }
  function t(key, fallback, params) {
    return window.SheetAuditT(key, fallback, params);
  }

  function cellOf(ctx, sheetIdx, row, col) {
    var G = window.SheetAuditGraph;
    if (!ctx.graph) return null;
    return ctx.graph.cells[G.cellId(sheetIdx, row, col)] || null;
  }
  function localValue(ctx, cell) {
    if (!cell) return "";
    if (!cell.formula || !window.SheetAuditEval) return cell.value == null ? "" : String(cell.value);
    var evaluated = window.SheetAuditEval.evaluate(cell.formula,
      S.makeEvalContext(ctx.snapshot, ctx.index));
    return evaluated.error || evaluated.value;
  }

  function watchRows(ctx) {
    return (ctx.watch || window.SheetAuditUI.getWatch() || []).map(function (item) {
      var cell = cellOf(ctx, item.sheetIdx, item.row, item.col);
      var sheets = (ctx.graph && ctx.graph.sheets) || [];
      var ws = sheets[item.sheetIdx];
      return {
        item: item,
        ref: cell ? cell.ref : window.SheetAuditGraph.colName(item.col) + (item.row + 1),
        sheet: ws ? ws.name : t("sheet-audit-sheet-fallback", "Sheet {index}",
          { index: String(item.sheetIdx + 1) }),
        formula: cell ? cell.formula : "",
        value: localValue(ctx, cell),
      };
    });
  }

  function renderWatchList(ctx) {
    var root = window.SheetAuditUI.section("watch");
    if (!root) return;
    var list = root.querySelector(".sheet-audit-watch-list");
    if (!list) return;
    clear(list);
    var rows = watchRows(ctx);
    if (!rows.length) {
      list.appendChild(node("p", "sheet-audit-empty", t("sheet-audit-watch-empty", "No pinned cells yet.")));
      return;
    }
    rows.forEach(function (row) {
      var line = node("div", "sheet-audit-watch-item");
      line.appendChild(node("span", "sheet-audit-watch-ref", row.ref));
      line.appendChild(node("span", "sheet-audit-watch-sheet", row.sheet));
      line.appendChild(node("span", "sheet-audit-watch-formula", row.formula));
      line.appendChild(node("span", "sheet-audit-watch-value", row.value));
      line.appendChild(button(t("sheet-audit-watch-jump", "Go to pinned cell"), function () {
        S.jumpTo(row.item);
      }, "sheet-audit-btn-small"));
      line.appendChild(button(t("sheet-audit-watch-remove", "Remove watch"), function () {
        var current = window.SheetAuditUI.getWatch();
        window.SheetAuditUI.setWatch(current.filter(function (item) {
          return !(item.sheetIdx === row.item.sheetIdx && item.row === row.item.row && item.col === row.item.col);
        }));
        renderWatchList(ctx);
      }, "sheet-audit-btn-small"));
      list.appendChild(line);
    });
  }

  S.updateWatch = function () {
    var UI = window.SheetAuditUI;
    if (!UI || !UI.getWatch().length) return;
    renderWatchList({
      graph: UI.getGraph(),
      snapshot: UI.getSnapshot(),
      index: UI.activeIndex(),
      watch: UI.getWatch(),
    });
  };

  function renderWatch(ctx) {
    var root = window.SheetAuditUI.section("watch");
    if (!root) return;
    clear(root);
    var input = node("input", "sheet-audit-input");
    input.type = "text";
    input.id = "sheet-audit-watch-ref";
    input.value = ctx.selectionRef || "";
    input.setAttribute("aria-label", t("sheet-audit-watch-ref", "Cell"));

    var actions = node("div", "sheet-audit-actions");
    actions.appendChild(button(t("sheet-audit-watch-add", "Add watch"), function () {
      var target = S.resolveRef(ctx, input.value.trim());
      if (!target) {
        status(ctx, t("sheet-audit-trace-bad-ref", "Enter a reference such as A1 or Sheet2!B3."));
        return;
      }
      var current = window.SheetAuditUI.getWatch();
      var exists = current.some(function (item) {
        return item.sheetIdx === target.sheetIdx && item.row === target.row && item.col === target.col;
      });
      if (!exists) {
        current.push({ sheetIdx: target.sheetIdx, row: target.row, col: target.col });
        window.SheetAuditUI.setWatch(current);
      }
      renderWatchList(ctx);
    }));
    actions.appendChild(button(t("sheet-audit-watch-refresh", "Refresh watch"), function () {
      window.SheetAuditUI.refreshGraph();
      renderWatchList({
        graph: window.SheetAuditUI.getGraph(),
        snapshot: window.SheetAuditUI.getSnapshot(),
        index: window.SheetAuditUI.activeIndex(),
        watch: window.SheetAuditUI.getWatch(),
      });
      status(ctx, t("sheet-audit-watch-updated", "Watch list refreshed at {time}.",
        { time: new Date().toLocaleTimeString() }));
    }));

    var list = node("div", "sheet-audit-watch-list");
    list.setAttribute("role", "list");
    root.appendChild(field(t("sheet-audit-watch-ref", "Cell"), input));
    root.appendChild(actions);
    root.appendChild(node("p", "sheet-audit-hint", t("sheet-audit-watch-live",
      "Live updates follow local cell changes.")));
    root.appendChild(list);
    renderWatchList(ctx);
  }

  function evalIssues(ctx) {
    var out = {};
    var cells = (ctx.graph && ctx.graph.cells) || {};
    var keys = Object.keys(cells).filter(function (id) { return !!cells[id].formula; });
    for (var i = 0; i < keys.length && i < MAX_EVAL_CELLS; i++) {
      var cell = cells[keys[i]];
      var evaluated = window.SheetAuditEval.evaluate(cell.formula,
        S.makeEvalContext(ctx.snapshot, cell.sheetIdx));
      if (evaluated.error) out[cell.id] = evaluated.error;
    }
    return out;
  }

  function renderErrors(ctx) {
    var root = window.SheetAuditUI.section("errors");
    if (!root) return;
    clear(root);
    var list = node("div", "sheet-audit-result");
    list.setAttribute("role", "list");
    root.appendChild(button(t("sheet-audit-errors-scan", "Scan for errors"), function () {
      window.SheetAuditUI.refreshGraph();
      var fresh = {
        graph: window.SheetAuditUI.getGraph(),
        snapshot: window.SheetAuditUI.getSnapshot(),
        index: window.SheetAuditUI.activeIndex(),
        selection: ctx.selection,
      };
      var errors = fresh.graph ? window.SheetAuditGraph.inspectErrors(fresh.graph, evalIssues(fresh)) : [];
      clear(list);
      if (!errors.length) {
        list.appendChild(node("p", "sheet-audit-empty",
          t("sheet-audit-errors-empty", "No errors found in local cells.")));
        status(ctx, t("sheet-audit-errors-empty", "No errors found in local cells."));
        return;
      }
      status(ctx, t("sheet-audit-errors-summary", "{count} errors found.",
        { count: String(errors.length) }));
      var head = node("div", "sheet-audit-error-head");
      head.setAttribute("role", "presentation");
      head.appendChild(node("span", "sheet-audit-error-type",
        t("sheet-audit-errors-type", "Type")));
      head.appendChild(node("span", "sheet-audit-error-location",
        t("sheet-audit-errors-location", "Location")));
      head.appendChild(node("span", "sheet-audit-error-cause",
        t("sheet-audit-errors-cause", "Likely cause (local)")));
      head.appendChild(node("span", "sheet-audit-error-fix",
        t("sheet-audit-errors-fix", "Suggested fix")));
      list.appendChild(head);
      errors.forEach(function (error) {
        var row = node("div", "sheet-audit-error-item");
        row.appendChild(node("span", "sheet-audit-error-type", error.type));
        row.appendChild(node("span", "sheet-audit-error-location", error.ref));
        row.appendChild(node("span", "sheet-audit-error-cause",
          t(error.causeKey, error.causeFallback)));
        row.appendChild(node("span", "sheet-audit-error-fix",
          t(error.fixKey, error.fixFallback)));
        row.appendChild(button(t("sheet-audit-cycles-jump", "Jump to this cell"), function () {
          var parsed = window.SheetAuditGraph.parseId(error.id);
          S.jumpTo(parsed);
        }, "sheet-audit-btn-small"));
        list.appendChild(row);
      });
    }));
    root.appendChild(node("p", "sheet-audit-hint", t("sheet-audit-local-note",
      "Local-only analysis of in-memory cells; no server graph or evaluator parity is implied.")));
    root.appendChild(list);
  }

  function renderCycles(ctx) {
    var root = window.SheetAuditUI.section("cycles");
    if (!root) return;
    clear(root);
    var list = node("div", "sheet-audit-result");
    list.setAttribute("role", "list");
    root.appendChild(button(t("sheet-audit-cycles-scan", "Detect circular references"), function () {
      var found = ctx.graph ? window.SheetAuditGraph.findCycles(ctx.graph) : { cycles: [], truncated: false };
      clear(list);
      if (found.iterative) {
        list.appendChild(node("p", "sheet-audit-note", t("sheet-audit-cycles-iterative",
          "Iterative calculation appears enabled locally; cycles are reported but not treated as fatal.")));
      }
      if (!found.cycles.length) {
        list.appendChild(node("p", "sheet-audit-empty", t("sheet-audit-cycles-empty",
          "No circular references found in local cells.")));
        status(ctx, t("sheet-audit-cycles-empty", "No circular references found in local cells."));
        return;
      }
      status(ctx, t("sheet-audit-cycles-summary", "{count} circular reference chain(s) found.",
        { count: String(found.cycles.length) }));
      found.cycles.forEach(function (cycle) {
        var row = node("div", "sheet-audit-cycle-item");
        row.appendChild(node("p", "sheet-audit-cycle-path", t("sheet-audit-cycles-path", "Cycle: {path}",
          { path: cycle.refs.join(" \u2192 ") })));
        cycle.refs.forEach(function (ref) {
          row.appendChild(button(ref, function () {
            var target = S.resolveRef(ctx, ref);
            if (target) S.jumpTo(target);
          }, "sheet-audit-btn-small"));
        });
        list.appendChild(row);
      });
      if (found.truncated) {
        list.appendChild(node("p", "sheet-audit-note", t("sheet-audit-cycles-truncated",
          "Scan limit reached; not every chain is listed.")));
      }
    }));
    root.appendChild(list);
  }

  function selectionRange(ctx) {
    var sel = ctx.selection;
    var G = window.SheetAuditGraph;
    if (!sel) return "A1";
    var first = G.colName(sel.startCol) + (sel.startRow + 1);
    var last = G.colName(sel.endCol) + (sel.endRow + 1);
    return first === last ? first : first + ":" + last;
  }

  function renderText(ctx) {
    var root = window.SheetAuditUI.section("text");
    if (!root) return;
    clear(root);
    var input = node("input", "sheet-audit-input");
    input.type = "text";
    input.id = "sheet-audit-text-range";
    input.value = selectionRange(ctx);
    input.setAttribute("aria-label", t("sheet-audit-text-range", "Range"));
    var output = node("pre", "sheet-audit-text-output");
    output.tabIndex = 0;
    var lines = [];

    var actions = node("div", "sheet-audit-actions");
    actions.appendChild(button(t("sheet-audit-text-render", "Show formula text"), function () {
      var raw = input.value.trim().replace(/\$/g, "");
      var parts = raw.split(":");
      var first = window.SheetAuditGraph.parseRef(parts[0], (ctx.graph && ctx.graph.sheets) || []);
      var last = window.SheetAuditGraph.parseRef(parts[parts.length - 1], (ctx.graph && ctx.graph.sheets) || []) || first;
      if (!first) {
        status(ctx, t("sheet-audit-trace-bad-ref", "Enter a reference such as A1 or Sheet2!B3."));
        return;
      }
      var range = {
        startRow: Math.min(first.row, last.row), endRow: Math.max(first.row, last.row),
        startCol: Math.min(first.col, last.col), endCol: Math.max(first.col, last.col),
      };
      var result = ctx.graph ? window.SheetAuditGraph.formulaAsText(ctx.snapshot, ctx.index, range) : { rows: [] };
      lines = result.rows.map(function (row) {
        return t("sheet-audit-text-line", "{ref}: {formula} → {value}", {
          ref: row.ref, formula: row.formula, value: row.value == null ? "" : String(row.value),
        });
      });
      output.textContent = lines.join("\n");
      if (result.truncated) {
        status(ctx, t("sheet-audit-text-truncated", "Output limit reached; first {count} formulas shown.",
          { count: String(lines.length) }));
      } else if (!lines.length) {
        status(ctx, t("sheet-audit-text-empty", "No formulas in this range."));
      } else {
        status(ctx, t("sheet-audit-status-ready", "Ready."));
      }
    }));
    actions.appendChild(button(t("sheet-audit-text-copy", "Copy to clipboard"), function () {
      if (!navigator.clipboard || typeof navigator.clipboard.writeText !== "function") return;
      navigator.clipboard.writeText(lines.join("\n")).then(function () {
        status(ctx, t("sheet-audit-text-copied", "Copied to clipboard."));
      }).catch(function () {
        status(ctx, t("sheet-audit-status-ready", "Ready."));
      });
    }));
    actions.appendChild(button(t("sheet-audit-show-toggle", "Show formulas"), function () {
      var overlay = window.SheetAuditOverlay;
      if (!overlay) return;
      var enabled = overlay.toggleShowFormulas(ctx.snapshot, ctx.index);
      status(ctx, enabled ? t("sheet-audit-show-on", "Showing formulas in the local view.")
        : t("sheet-audit-show-off", "Showing values."));
    }));
    actions.appendChild(button(t("sheet-audit-fit-button", "Column fit hint"), function () {
      var overlay = window.SheetAuditOverlay;
      var hints = window.SheetAuditGraph.columnFitHint(ctx.snapshot, ctx.index,
        overlay && typeof overlay.isShowingFormulas === "function" ? overlay.isShowingFormulas() : false);
      if (!hints.length) {
        status(ctx, t("sheet-audit-fit-none", "No column-fit hint available."));
        return;
      }
      var summary = hints.map(function (hint) {
        return t("sheet-audit-fit-col", "Column {col}: about {width}px",
          { col: hint.letter, width: String(hint.width) });
      }).join("; ");
      status(ctx, t("sheet-audit-fit-hint", "Suggested widths: {summary}", { summary: summary }));
    }));

    root.appendChild(field(t("sheet-audit-text-range", "Range"), input));
    root.appendChild(actions);
    root.appendChild(node("p", "sheet-audit-hint", t("sheet-audit-print-hint",
      "Printing uses the current on-screen view; no vector export is performed.")));
    root.appendChild(output);
  }

  function renderMap(ctx) {
    var root = window.SheetAuditUI.section("map");
    if (!root) return;
    clear(root);
    var list = node("div", "sheet-audit-result");
    list.setAttribute("role", "list");
    root.appendChild(button(t("sheet-audit-map-render", "Build dependency map"), function () {
      var map = ctx.graph ? window.SheetAuditGraph.dependencyMap(ctx.graph, ctx.index)
        : { rows: [], links: 0, formulas: 0 };
      clear(list);
      if (!map.rows.length) {
        list.appendChild(node("p", "sheet-audit-empty",
          t("sheet-audit-map-empty", "No formula dependencies in this sheet.")));
        status(ctx, t("sheet-audit-map-empty", "No formula dependencies in this sheet."));
        return;
      }
      var summary = t("sheet-audit-map-summary", "{formulas} formulas, {links} dependency links.",
        { formulas: String(map.formulas), links: String(map.links) });
      list.appendChild(node("p", "sheet-audit-summary", summary));
      status(ctx, summary);
      map.rows.forEach(function (row) {
        var line = node("div", "sheet-audit-map-item");
        line.appendChild(node("span", "sheet-audit-map-ref", row.ref));
        line.appendChild(node("span", "sheet-audit-map-counts",
          t("sheet-audit-map-row", "{ref}: {precedents} precedents, {dependents} dependents", {
            ref: row.ref, precedents: String(row.precedents), dependents: String(row.dependents),
          })));
        line.appendChild(button(t("sheet-audit-cycles-jump", "Jump to this cell"), function () {
          var parsed = window.SheetAuditGraph.parseId(row.id);
          S.jumpTo(parsed);
        }, "sheet-audit-btn-small"));
        list.appendChild(line);
      });
    }));
    root.appendChild(list);
  }

  S.renderWatch = renderWatch;
  S.renderErrors = renderErrors;
  S.renderCycles = renderCycles;
  S.renderText = renderText;
  S.renderMap = renderMap;
})();
