"use strict";
/* Sheets structured data (#2436) — bounded local Goal Seek (single variable).
 * Uses the existing local formula evaluator when present, otherwise an honest
 * simple-dependency model. Never writes server schema. */

(function () {
  const S = window.SheetStructured;
  const t = S.t;
  const MAX_ITER = 80;
  const MAX_BRACKET = 50;
  const LIMIT = 1e9;
  const TOL = 1e-6;
  const NUM = "(-?(?:\\d+(?:\\.\\d*)?|\\.\\d+))";

  let activeDialog = null;
  let lastSolution = null;

  function fmt(value) {
    return String(Math.round(value * 1e6) / 1e6);
  }

  function evalNumericTerm(term) {
    const factors = term.split(/[*/]/);
    const ops = term.match(/[*/]/g) || [];
    let result = Number(factors[0]);
    if (!Number.isFinite(result)) return null;
    for (let i = 0; i < ops.length; i++) {
      const operand = Number(factors[i + 1]);
      if (!Number.isFinite(operand)) return null;
      if (ops[i] === "*") result *= operand;
      else {
        if (operand === 0) return null;
        result /= operand;
      }
    }
    return result;
  }

  // Parses a single-variable linear expression into slope * x + intercept.
  // Supports compound forms such as =B1*2+10, =2*B1-5 and =-B1/2+8.
  function linearModel(clean, variable) {
    const escaped = variable.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const varOnly = new RegExp("^" + escaped + "$");
    const varTimes = new RegExp("^" + escaped + "\\*" + NUM + "$");
    const timesVar = new RegExp("^(" + NUM + ")\\*" + escaped + "$");
    const varOver = new RegExp("^" + escaped + "/(" + NUM + ")$");
    const hasVar = new RegExp(escaped);
    const terms = clean.match(/[+-]?[^+-]+/g);
    if (!terms) return null;
    let slope = 0;
    let intercept = 0;
    for (let i = 0; i < terms.length; i++) {
      let term = terms[i];
      let sign = 1;
      if (term.charAt(0) === "+") term = term.slice(1);
      else if (term.charAt(0) === "-") { sign = -1; term = term.slice(1); }
      if (!term) return null;
      if (!hasVar.test(term)) {
        const constant = evalNumericTerm(term);
        if (constant === null) return null;
        intercept += sign * constant;
        continue;
      }
      let coefficient = null;
      let match = varOnly.exec(term);
      if (match) coefficient = 1;
      else if ((match = varTimes.exec(term))) coefficient = Number(match[1]);
      else if ((match = timesVar.exec(term))) coefficient = Number(match[1]);
      else if ((match = varOver.exec(term))) {
        if (Number(match[1]) === 0) return null;
        coefficient = 1 / Number(match[1]);
      }
      if (coefficient === null) return null;
      slope += sign * coefficient;
    }
    return { slope: slope, intercept: intercept };
  }

  function simpleModel(formula, changingRef, candidate, fallbackValue) {
    const clean = String(formula || "")
      .replace(/^\s*=/, "").replace(/\s+/g, "").replace(/\$/g, "").toUpperCase();
    const variable = String(changingRef || "").replace(/\$/g, "").toUpperCase();
    if (!clean) return S.numeric(fallbackValue);
    if (!variable) return null;
    const model = linearModel(clean, variable);
    if (!model) return null;
    return model.slope * candidate + model.intercept;
  }

  function evaluateAt(targetPos, changingPos, candidate) {
    const data = S.cellData(targetPos.row, targetPos.col);
    const formula = data && data.formula ? String(data.formula) : "";
    const evaluator = window.SheetAuditEval;
    if (evaluator && typeof evaluator.evaluate === "function" && formula) {
      const ctx = {
        resolve: function (sheetName, row, col) {
          if (row === changingPos.row && col === changingPos.col) {
            return { value: candidate, formula: "" };
          }
          const cell = S.cellData(row, col);
          return cell ? { value: cell.value, formula: cell.formula } : null;
        },
      };
      const result = evaluator.evaluate(formula, ctx);
      if (result && result.error) return { value: NaN, engine: "local" };
      return { value: S.numeric(result && result.value), engine: "local" };
    }
    const changingRef = S.refOf(changingPos.row, changingPos.col);
    const simple = simpleModel(formula, changingRef, candidate, data ? data.value : "");
    return { value: simple, engine: "simple" };
  }

  function solve(targetRef, targetValue, changingRef, onIteration) {
    const targetPos = S.parseRef(targetRef);
    const changingPos = S.parseRef(changingRef);
    const value = S.numeric(targetValue);
    if (!targetPos || !changingPos || value === null) {
      return { converged: false, reason: "invalid", iterations: [], engine: "simple" };
    }
    const start = S.numeric(S.cellValue(changingPos.row, changingPos.col));
    const origin = start === null ? 0 : start;
    const residual = function (x) {
      const evaluated = evaluateAt(targetPos, changingPos, x);
      if (evaluated.value === null || !Number.isFinite(evaluated.value)) return NaN;
      return evaluated.value - value;
    };
    const iterations = [];
    const record = function (index, x, r) {
      iterations.push({ index: index, value: x, residual: r });
      if (onIteration) onIteration(iterations[iterations.length - 1]);
    };
    let engine = "simple";
    const probe = evaluateAt(targetPos, changingPos, origin);
    engine = probe.engine;

    let lo = origin;
    let hi = origin;
    let flo = residual(lo);
    let fhi = flo;
    record(0, origin, flo);
    if (Number.isFinite(flo) && Math.abs(flo) <= TOL) {
      return { converged: true, value: origin, iterations: iterations, engine: engine, residual: flo };
    }
    let bracketed = false;
    for (let step = 1; step <= MAX_BRACKET; step++) {
      const span = Math.max(1, Math.abs(origin) * 0.1) * step;
      lo = Math.max(-LIMIT, origin - span);
      hi = Math.min(LIMIT, origin + span);
      flo = residual(lo);
      fhi = residual(hi);
      if (Number.isFinite(flo) && Math.abs(flo) <= TOL) {
        return { converged: true, value: lo, iterations: iterations, engine: engine, residual: flo };
      }
      if (Number.isFinite(fhi) && Math.abs(fhi) <= TOL) {
        return { converged: true, value: hi, iterations: iterations, engine: engine, residual: fhi };
      }
      if (Number.isFinite(flo) && Number.isFinite(fhi) && flo * fhi <= 0) {
        bracketed = true;
        break;
      }
    }
    if (!bracketed) {
      return { converged: false, reason: "nobacket", iterations: iterations, engine: engine };
    }
    let bestX = (lo + hi) / 2;
    let bestR = residual(bestX);
    for (let index = 1; index <= MAX_ITER; index++) {
      const mid = (lo + hi) / 2;
      const fmid = residual(mid);
      record(index, mid, fmid);
      if (!Number.isFinite(fmid)) break;
      if (Math.abs(fmid) <= TOL || Math.abs(hi - lo) <= TOL * Math.max(1, Math.abs(mid))) {
        bestX = mid;
        bestR = fmid;
        break;
      }
      bestX = mid;
      bestR = fmid;
      if (flo * fmid <= 0) {
        hi = mid;
        fhi = fmid;
      } else {
        lo = mid;
        flo = fmid;
      }
    }
    const converged = Number.isFinite(bestR) && Math.abs(bestR) <= TOL;
    return {
      converged: converged,
      value: bestX,
      residual: bestR,
      iterations: iterations,
      engine: engine,
      reason: converged ? "ok" : "maxiter",
    };
  }

  function fieldRow(labelKey, fallback, control) {
    const row = document.createElement("label");
    row.className = "ss-structured-field";
    const caption = document.createElement("span");
    caption.textContent = t(labelKey, fallback);
    caption.setAttribute("data-i18n", labelKey);
    control.style.width = "100%";
    row.appendChild(caption);
    row.appendChild(control);
    return row;
  }

  function button(labelKey, fallback, className) {
    const node = document.createElement("button");
    node.type = "button";
    node.className = className || "ss-structured-btn";
    node.textContent = t(labelKey, fallback);
    node.setAttribute("data-i18n", labelKey);
    return node;
  }

  function renderIterations(host, iterations) {
    host.innerHTML = "";
    const shown = iterations.slice(-12);
    const html = ["<table class=\"ss-structured-grid ss-structured-iterations\"><thead><tr>"];
    html.push("<th scope=\"col\">" + S.escapeHtml(t("sheet-structured-goalseek-iterations", "Iterations")) + "</th>");
    html.push("<th scope=\"col\">" + S.escapeHtml(t("sheet-structured-goalseek-value-col", "Value")) + "</th>");
    html.push("<th scope=\"col\">" + S.escapeHtml(t("sheet-structured-goalseek-residual", "Residual")) + "</th>");
    html.push("</tr></thead><tbody>");
    shown.forEach(function (item) {
      html.push("<tr><td>" + item.index + "</td><td>" + S.escapeHtml(fmt(item.value)) + "</td><td>" + S.escapeHtml(fmt(item.residual)) + "</td></tr>");
    });
    html.push("</tbody></table>");
    host.innerHTML = html.join("");
  }

  function open() {
    if (activeDialog) activeDialog.close();
    lastSolution = null;
    const dialog = S.createDialog("sheet-structured-goalseek", "Goal Seek");
    activeDialog = dialog;
    dialog.body.appendChild(S.localNote());

    const form = document.createElement("div");
    form.className = "ss-structured-form";
    const targetInput = document.createElement("input");
    targetInput.type = "text";
    targetInput.className = "ss-structured-input";
    const valueInput = document.createElement("input");
    valueInput.type = "number";
    valueInput.className = "ss-structured-input";
    valueInput.step = "any";
    const changingInput = document.createElement("input");
    changingInput.type = "text";
    changingInput.className = "ss-structured-input";
    const selection = window.SheetAdvanced && window.SheetAdvanced.getSelection ? window.SheetAdvanced.getSelection() : null;
    if (selection) {
      targetInput.value = S.refOf(selection.startRow, selection.startCol);
      changingInput.value = targetInput.value;
    }
    form.appendChild(fieldRow("sheet-structured-goalseek-target", "Set cell", targetInput));
    form.appendChild(fieldRow("sheet-structured-goalseek-value", "To value", valueInput));
    form.appendChild(fieldRow("sheet-structured-goalseek-changing", "By changing cell", changingInput));
    dialog.body.appendChild(form);

    const engine = document.createElement("p");
    engine.className = "ss-structured-goalseek-engine";
    dialog.body.appendChild(engine);
    const result = document.createElement("p");
    result.className = "ss-structured-goalseek-result";
    dialog.body.appendChild(result);
    const iterations = document.createElement("div");
    iterations.className = "ss-structured-table-host";
    dialog.body.appendChild(iterations);

    const run = button("sheet-structured-goalseek-run", "Solve", "ss-structured-btn ss-structured-primary");
    const apply = button("sheet-structured-apply", "Apply", "ss-structured-btn");
    apply.disabled = true;
    run.addEventListener("click", function () {
      const targetValue = S.numeric(valueInput.value);
      if (targetValue === null || !S.parseRef(targetInput.value) || !S.parseRef(changingInput.value)) {
        dialog.setStatus(t("sheet-structured-goalseek-invalid", "Enter numeric values for the target and changing cells."), true);
        valueInput.focus();
        return;
      }
      lastSolution = solve(targetInput.value.trim().toUpperCase(), targetValue, changingInput.value.trim().toUpperCase());
      engine.textContent = lastSolution.engine === "local"
        ? t("sheet-structured-goalseek-engine-local", "Engine: local formula evaluator.")
        : t("sheet-structured-goalseek-engine-simple", "Engine: simple-dependency mode (direct/linear).");
      result.textContent = "";
      renderIterations(iterations, lastSolution.iterations);
      if (lastSolution.converged) {
        result.textContent = t("sheet-structured-goalseek-result", "Solution") + ": " + fmt(lastSolution.value);
        dialog.setStatus(t("sheet-structured-goalseek-converged", "Converged in {count} iterations.", { count: String(lastSolution.iterations.length) }), false);
        apply.disabled = false;
      } else if (lastSolution.reason === "nobacket") {
        dialog.setStatus(t("sheet-structured-goalseek-nobacket", "Could not bracket the target; try another value."), true);
        apply.disabled = true;
      } else {
        dialog.setStatus(t("sheet-structured-goalseek-maxiter", "No convergence after {count} iterations.", { count: String(lastSolution.iterations.length) }), true);
        apply.disabled = true;
      }
    });
    apply.addEventListener("click", function () {
      if (!lastSolution || !lastSolution.converged) return;
      const changing = changingInput.value.trim().toUpperCase();
      S.writeValue(changing, fmt(lastSolution.value), "sheet-structured-goalseek", "Goal Seek");
      const message = t("sheet-structured-goalseek-applied", "Wrote {value} to {cell}; undo is available.", {
        value: fmt(lastSolution.value), cell: changing,
      });
      dialog.setStatus(message, false);
      S.notify(message, false);
    });
    dialog.footer.appendChild(run);
    dialog.footer.appendChild(apply);
    targetInput.focus();
  }

  window.SheetStructuredGoalSeek = { open: open, solve: solve, simpleModel: simpleModel };
})();
