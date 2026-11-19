"use strict";
/* Sheet advanced module: 28_audit_sections — trace and evaluate panels for the
   local-only formula auditor (#2491). */

(function () {
  var S = window.SheetAuditSections || (window.SheetAuditSections = {});

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

  function graphContext(ctx) {
    var G = window.SheetAuditGraph;
    var sheetIdx = ctx.index;
    var sheets = (ctx.graph && ctx.graph.sheets) || [];
    if (!sheets[sheetIdx]) sheetIdx = 0;
    return { G: G, sheets: sheets, sheetIdx: sheetIdx, graph: ctx.graph };
  }

  function resolveRef(ctx, value) {
    var gc = graphContext(ctx);
    var parsed = gc.G.parseRef(value, gc.sheets);
    if (!parsed) return null;
    var sheetIdx = parsed.sheetIdx >= 0 ? parsed.sheetIdx : gc.sheetIdx;
    if (parsed.sheetName && parsed.sheetIdx < 0) return null;
    return {
      sheetIdx: sheetIdx,
      row: parsed.row,
      col: parsed.col,
      id: gc.G.cellId(sheetIdx, parsed.row, parsed.col),
      ref: gc.G.refOf(sheetIdx, parsed.row, parsed.col, gc.sheets),
    };
  }

  function jumpTo(target) {
    if (!target) return;
    var sheets = (window.SheetAuditUI.getGraph() || {}).sheets || [];
    if (target.sheetIdx !== window.SheetAuditUI.activeIndex() &&
        window.SheetTabs && typeof window.SheetTabs.switchWorksheet === "function") {
      window.SheetTabs.switchWorksheet(target.sheetIdx);
    }
    if (window.SheetAdvanced && typeof window.SheetAdvanced.setRange === "function") {
      window.SheetAdvanced.setRange(target.row, target.col, target.row, target.col);
    }
  }
  S.jumpTo = jumpTo;

  function drawArrows(origin, result, direction) {
    var overlay = window.SheetAuditOverlay;
    if (overlay && typeof overlay.drawArrows === "function") {
      overlay.drawArrows(origin, result.nodes, direction, window.SheetAuditUI.activeIndex());
    }
  }

  function renderTrace(ctx) {
    var root = window.SheetAuditUI.section("trace");
    if (!root) return;
    clear(root);
    var T = ctx.t;
    var gc = graphContext(ctx);

    var controls = node("div", "sheet-audit-controls");
    var refInput = node("input", "sheet-audit-input");
    refInput.type = "text";
    refInput.id = "sheet-audit-trace-ref";
    refInput.value = ctx.selectionRef || "";
    refInput.setAttribute("aria-label", T("sheet-audit-trace-cell", "Cell reference"));
    refInput.setAttribute("autocomplete", "off");

    var depth = node("select", "sheet-audit-select");
    depth.id = "sheet-audit-trace-depth";
    depth.setAttribute("aria-label", T("sheet-audit-trace-depth", "Levels"));
    for (var i = 1; i <= 8; i++) {
      var option = node("option", "", T("sheet-audit-trace-level", "Level {level}", { level: String(i) }));
      option.value = String(i);
      depth.appendChild(option);
    }
    depth.value = "5";

    var resultBox = node("div", "sheet-audit-result");
    resultBox.id = "sheet-audit-trace-result";
    resultBox.setAttribute("role", "list");

    function run(direction) {
      var target = resolveRef(ctx, refInput.value.trim());
      if (!target) {
        status(ctx, T("sheet-audit-trace-bad-ref", "Enter a reference such as A1 or Sheet2!B3."));
        return;
      }
      var result = gc.G.trace(ctx.graph, target.id, direction, parseInt(depth.value, 10));
      clear(resultBox);
      var summary = T("sheet-audit-trace-summary", "{count} cells across {levels} levels",
        { count: String(Math.max(0, result.nodes.length - 1)), levels: String(result.levels) });
      resultBox.appendChild(node("p", "sheet-audit-summary", summary));
      var crossSheet = [];
      result.nodes.forEach(function (item) {
        if (item.id === target.id) return;
        var row = node("div", "sheet-audit-trace-item");
        var label = node("span", "sheet-audit-trace-ref", item.ref || item.id);
        var detail = node("span", "sheet-audit-trace-detail",
          item.formula || (item.value == null ? "" : String(item.value)));
        row.appendChild(label);
        row.appendChild(detail);
        if (item.sheetIdx === target.sheetIdx) {
          row.appendChild(button(T("sheet-audit-trace-jump", "Jump to cell"), function () {
            var parsed = gc.G.parseId(item.id);
            jumpTo({ sheetIdx: parsed.sheetIdx, row: parsed.row, col: parsed.col });
          }, "sheet-audit-btn-small"));
        } else if (crossSheet.indexOf(item.sheetName) < 0) {
          crossSheet.push(item.sheetName);
        }
        resultBox.appendChild(row);
      });
      crossSheet.forEach(function (name) {
        resultBox.appendChild(node("p", "sheet-audit-note",
          T("sheet-audit-trace-cross-sheet", "{ref} lives on {sheet}; switch sheets to draw its arrows.",
            { ref: target.ref, sheet: name })));
      });
      if (!result.nodes.length || result.nodes.length === 1) {
        resultBox.appendChild(node("p", "sheet-audit-empty",
          T("sheet-audit-trace-empty", "No related cells found in local state.")));
      }
      if (result.truncated) {
        resultBox.appendChild(node("p", "sheet-audit-note",
          T("sheet-audit-cycles-truncated", "Scan limit reached; not every chain is listed.")));
      }
      drawArrows(target, result, direction);
      status(ctx, summary);
    }

    function selectRelated() {
      var target = resolveRef(ctx, refInput.value.trim());
      if (!target) {
        status(ctx, T("sheet-audit-trace-bad-ref", "Enter a reference such as A1 or Sheet2!B3."));
        return;
      }
      var result = gc.G.trace(ctx.graph, target.id, "precedents", parseInt(depth.value, 10));
      var sameSheet = result.nodes.filter(function (item) {
        return item.sheetIdx === target.sheetIdx;
      });
      if (!sameSheet.length) {
        status(ctx, T("sheet-audit-trace-empty", "No related cells found in local state."));
        return;
      }
      var rows = sameSheet.map(function (item) {
        var parsed = gc.G.parseId(item.id);
        return parsed;
      });
      var minRow = Math.min.apply(null, rows.map(function (p) { return p.row; }));
      var maxRow = Math.max.apply(null, rows.map(function (p) { return p.row; }));
      var minCol = Math.min.apply(null, rows.map(function (p) { return p.col; }));
      var maxCol = Math.max.apply(null, rows.map(function (p) { return p.col; }));
      if (window.SheetAdvanced && typeof window.SheetAdvanced.setRange === "function") {
        window.SheetAdvanced.setRange(minRow, minCol, maxRow, maxCol);
      }
      run("precedents");
    }

    controls.appendChild(field(T("sheet-audit-trace-cell", "Cell reference"), refInput));
    controls.appendChild(field(T("sheet-audit-trace-depth", "Levels"), depth));
    var actions = node("div", "sheet-audit-actions");
    actions.appendChild(button(T("sheet-audit-trace-precedents", "Trace precedents"), function () { run("precedents"); }));
    actions.appendChild(button(T("sheet-audit-trace-dependents", "Trace dependents"), function () { run("dependents"); }));
    actions.appendChild(button(T("sheet-audit-trace-select", "Select all precedents"), selectRelated));
    actions.appendChild(button(T("sheet-audit-trace-clear", "Clear arrows"), function () {
      if (window.SheetAuditOverlay) window.SheetAuditOverlay.clearArrows();
      status(ctx, T("sheet-audit-status-ready", "Ready."));
    }));
    controls.appendChild(actions);

    var hint = node("p", "sheet-audit-hint", T("sheet-audit-trace-current", "Current cell: {ref}",
      { ref: ctx.selectionRef || "" }));
    root.appendChild(controls);
    root.appendChild(hint);
    root.appendChild(resultBox);
  }

  function makeEvalContext(snapshot, index) {
    var G = window.SheetAuditGraph;
    var sheets = (snapshot && snapshot.worksheets) || [];
    var idx = sheets[index] ? index : 0;
    return {
      resolve: function (sheetName, row, col) {
        var target = idx;
        if (sheetName) {
          target = G.sheetIndex(sheets, sheetName);
          if (target < 0) return null;
        }
        var ws = sheets[target];
        if (!ws) return null;
        var cell = (ws.data || {})[row + "," + col];
        return cell ? { value: cell.value, formula: cell.formula } : null;
      },
    };
  }

  function highlightExpression(container, source, step) {
    clear(container);
    var before = step ? source.slice(0, step.start) : source;
    var active = step ? source.slice(step.start, step.end) : "";
    var after = step ? source.slice(step.end) : "";
    container.appendChild(node("span", "sheet-audit-code", before));
    if (active) container.appendChild(node("mark", "sheet-audit-code-active", active));
    container.appendChild(node("span", "sheet-audit-code", after));
  }

  function renderEvaluate(ctx) {
    var root = window.SheetAuditUI.section("evaluate");
    if (!root) return;
    clear(root);
    var T = ctx.t;
    var graph = ctx.graph;
    var selectedFormula = "";
    if (graph && ctx.selection) {
      var cell = graph.cells[window.SheetAuditGraph.cellId(ctx.index, ctx.selection.startRow, ctx.selection.startCol)];
      if (cell && cell.formula) selectedFormula = cell.formula;
    }
    var input = node("textarea", "sheet-audit-input sheet-audit-formula-input");
    input.id = "sheet-audit-eval-input";
    input.rows = 2;
    input.value = selectedFormula || "=1+2";
    input.setAttribute("aria-label", T("sheet-audit-eval-formula", "Formula"));

    var expression = node("div", "sheet-audit-expression");
    expression.setAttribute("aria-hidden", "true");
    var stepsList = node("ol", "sheet-audit-steps");
    stepsList.setAttribute("role", "list");
    var result = node("p", "sheet-audit-result-text");
    result.setAttribute("role", "status");
    result.setAttribute("aria-live", "polite");

    var steps = [];
    var cursor = 0;
    var source = "";

    function renderStep() {
      clear(stepsList);
      highlightExpression(expression, source, steps[cursor] || null);
      steps.forEach(function (step, index) {
        var li = node("li", "sheet-audit-step" + (index === cursor ? " is-active" : ""));
        li.appendChild(node("span", "sheet-audit-step-expr", step.expr));
        li.appendChild(node("span", "sheet-audit-step-value", step.value));
        li.tabIndex = 0;
        li.setAttribute("role", "listitem");
        li.addEventListener("click", function () {
          cursor = index;
          renderStep();
        });
        li.addEventListener("keydown", function (event) {
          if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            cursor = index;
            renderStep();
          }
        });
        stepsList.appendChild(li);
      });
      var step = steps[cursor];
      if (step) {
        result.textContent = T("sheet-audit-eval-step", "Step {index} of {total}: {expression} = {value}", {
          index: String(cursor + 1), total: String(steps.length),
          expression: step.expr, value: step.value,
        });
      }
    }

    function run() {
      var formula = String(input.value || "").trim();
      if (!formula || formula.charAt(0) !== "=") {
        status(ctx, T("sheet-audit-eval-empty", "Enter a formula starting with =."));
        return;
      }
      if (!window.SheetAuditEval || typeof window.SheetAuditEval.evaluate !== "function") return;
      source = formula.replace(/^=/, "");
      var evaluated = window.SheetAuditEval.evaluate(formula, makeEvalContext(ctx.snapshot, ctx.index));
      steps = evaluated.steps || [];
      cursor = 0;
      if (evaluated.error) {
        status(ctx, T("sheet-audit-eval-error", "Evaluation stopped: {error}", { error: evaluated.error }));
      } else if (evaluated.truncated) {
        status(ctx, T("sheet-audit-eval-truncated", "Step limit reached; showing the first {count} steps.",
          { count: String(steps.length) }));
      } else {
        status(ctx, T("sheet-audit-eval-result", "Result: {value}", { value: evaluated.value }));
      }
      renderStep();
    }

    var actions = node("div", "sheet-audit-actions");
    actions.appendChild(button(T("sheet-audit-eval-run", "Evaluate step by step"), run));
    actions.appendChild(button(T("sheet-audit-eval-prev", "Previous step"), function () {
      if (cursor > 0) { cursor--; renderStep(); }
    }));
    actions.appendChild(button(T("sheet-audit-eval-next", "Next step"), function () {
      if (cursor < steps.length - 1) { cursor++; renderStep(); }
    }));
    actions.appendChild(button(T("sheet-audit-eval-reset", "Reset"), function () {
      steps = [];
      cursor = 0;
      clear(stepsList);
      clear(expression);
      result.textContent = "";
      status(ctx, T("sheet-audit-status-ready", "Ready."));
    }));

    root.appendChild(field(T("sheet-audit-eval-formula", "Formula"), input));
    root.appendChild(actions);
    root.appendChild(node("p", "sheet-audit-hint", T("sheet-audit-eval-bounded",
      "Local bounded evaluator: common operators and functions only.")));
    root.appendChild(expression);
    root.appendChild(stepsList);
    root.appendChild(result);
  }

  S.renderTrace = renderTrace;
  S.renderEvaluate = renderEvaluate;
  S.resolveRef = resolveRef;
  S.makeEvalContext = makeEvalContext;
  S.render = function (id, ctx) {
    if (id === "trace") renderTrace(ctx);
    else if (id === "evaluate") renderEvaluate(ctx);
    else if (S["render" + id.charAt(0).toUpperCase() + id.slice(1)]) {
      S["render" + id.charAt(0).toUpperCase() + id.slice(1)](ctx);
    }
  };
})();
