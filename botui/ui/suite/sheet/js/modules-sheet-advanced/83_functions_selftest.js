"use strict";
/* Issue #2434 - in-app self-test panel with known vectors. Every case runs
 * through the same local evaluator used by the wizard. */
(function () {
  var E = window.SheetFormulaErrors;
  var V = window.SheetFunctionsEvaluator;
  var C = window.SheetFunctionsCatalog;
  var S = window.SheetFunctionsSettings;
  var UI = window.SheetFunctionsUI;

  var VECTORS = [
    { key: "sheet-fn-test-pmt", fallback: "PMT loan payment", formula: "PMT(0.05/12,360,100000)", expected: -536.8216230121398, kind: "number" },
    { key: "sheet-fn-test-networkdays", fallback: "NETWORKDAYS across a weekend", formula: "NETWORKDAYS(\"2026-03-02\",\"2026-03-08\")", expected: 5, kind: "number" },
    { key: "sheet-fn-test-irr", fallback: "IRR of a cashflow series", formula: "IRR([-1000,100,200,300,400,500])", expected: 0.1200576195419627, kind: "number" },
    { key: "sheet-fn-test-norm", fallback: "NORM.INV at the median", formula: "NORM.INV(0.5,0,1)", expected: 0, kind: "number" },
    { key: "sheet-fn-test-textjoin", fallback: "TEXTJOIN skips blanks", formula: "TEXTJOIN(\"-\",TRUE,\"a\",\"\",\"b\")", expected: "a-b", kind: "value" },
    { key: "sheet-fn-test-div", fallback: "Nested division propagates #DIV/0!", formula: "SUM(1,1/0)", expected: "#DIV/0!", kind: "code" },
    { key: "sheet-fn-test-iferror", fallback: "IFERROR catches division by zero", formula: "IFERROR(1/0,0)", expected: 0, kind: "number" },
    { key: "sheet-fn-test-ifna", fallback: "IFNA only catches #N/A", formula: "IFNA(1/0,9)", expected: "#DIV/0!", kind: "code" },
    { key: "sheet-fn-test-coercion", fallback: "Text never silently becomes zero", formula: "SUM(\"abc\")", expected: "#VALUE!", kind: "code" },
    { key: "sheet-fn-test-alias", fallback: "pt-BR SOMA resolves to SUM", formula: "SOMA(1,2)", expected: 3, kind: "number", locale: "pt-BR" },
    { key: "sheet-fn-test-alias-en", fallback: "SUM resolves in English", formula: "SUM(1,2)", expected: 3, kind: "number", locale: "en" },
    { key: "sheet-fn-test-help", fallback: "Every function has help and an example", kind: "help" },
    { key: "sheet-fn-test-iterative", fallback: "Iterative calculation converges", kind: "iterative" },
  ];

  function within(actual, expected) {
    return typeof actual === "number" && Math.abs(actual - expected) <= 1e-8 * Math.max(1, Math.abs(expected));
  }

  function observed(vector) {
    if (vector.kind === "help") {
      var missing = C.all().filter(function (fn) {
        return !fn.example || fn.expected === undefined || !UI.t(fn.helpKey, "");
      });
      return { pass: missing.length === 0, value: missing.length ? missing.map(function (fn) { return fn.name; }).join(", ") : "OK" };
    }
    if (vector.kind === "iterative") {
      var settings = S.normalize({
        iterative: { enabled: true, maxIterations: 100, tolerance: 1e-9 },
      });
      var result = V.evaluateCircular({ A1: "=B1+1", B1: "=A1/2" }, { settings: settings });
      var pass = result.converged && within(result.values && result.values.A1, 2);
      return { pass: pass, value: "A1=" + V.format(result.values && result.values.A1, { settings: settings }) + ", " + result.iterations + " iterations" };
    }
    var settings = S.state();
    var result = V.evaluate(vector.formula, {
      locale: vector.locale || settings.locale, settings: settings,
    });
    if (vector.kind === "code") {
      return { pass: E.code(result) === vector.expected, value: E.format(result) };
    }
    if (vector.kind === "number") {
      return { pass: within(result, vector.expected), value: V.format(result, { settings: settings }) };
    }
    return { pass: result === vector.expected, value: V.format(result, { settings: settings }) };
  }

  function render(container, ctx) {
    var labels = {
      title: UI.t("sheet-fn-tests-summary", "Test results"),
      name: UI.t("sheet-fn-test-name", "Test"),
      expected: UI.t("sheet-fn-test-expected", "Expected"),
      actual: UI.t("sheet-fn-test-actual", "Actual"),
      result: UI.t("sheet-fn-test-result", "Result"),
      pass: UI.t("sheet-fn-test-pass", "PASS"),
      fail: UI.t("sheet-fn-test-fail", "FAIL"),
      run: UI.t("sheet-fn-tests-run", "Run self-test"),
    };
    var panel = UI.element("div", "ss-fn-tests");
    var run = UI.element("button", "ss-fn-btn", labels.run);
    run.type = "button";
    var summary = UI.element("p", "ss-fn-tests-summary", labels.title);
    summary.setAttribute("role", "status");
    summary.setAttribute("aria-live", "polite");
    var table = UI.element("table", "ss-fn-tests-table");
    panel.appendChild(run);
    panel.appendChild(summary);
    panel.appendChild(table);
    container.appendChild(panel);

    function execute() {
      table.textContent = "";
      var head = UI.element("tr", "ss-fn-tests-head");
      [labels.name, labels.expected, labels.actual, labels.result].forEach(function (text) {
        head.appendChild(UI.element("th", "", text));
      });
      table.appendChild(head);
      var passed = 0;
      VECTORS.forEach(function (vector) {
        var result = observed(vector);
        if (result.pass) passed += 1;
        var row = UI.element("tr", result.pass ? "ss-fn-test-pass" : "ss-fn-test-fail");
        row.appendChild(UI.element("td", "", UI.t(vector.key, vector.fallback)));
        row.appendChild(UI.element("td", "", vector.expected === undefined ? "OK" : String(vector.expected)));
        row.appendChild(UI.element("td", "", result.value));
        row.appendChild(UI.element("td", "", result.pass ? labels.pass : labels.fail));
        table.appendChild(row);
      });
      summary.textContent = labels.title + ": " + passed + " " + labels.pass + " / " +
        (VECTORS.length - passed) + " " + labels.fail;
      ctx.setStatus(summary.textContent, passed !== VECTORS.length);
    }

    run.addEventListener("click", execute);
    execute();
  }

  window.SheetFunctionsSelfTest = { render: render, VECTORS: VECTORS, observed: observed };
})();
