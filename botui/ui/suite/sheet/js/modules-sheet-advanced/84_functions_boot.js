"use strict";
/* Issue #2434 - boot and public facade for the local function layer. */
(function () {
  function wire(id, tab) {
    var button = document.getElementById(id);
    if (!button || button.dataset.functionsWired === "true") return;
    button.dataset.functionsWired = "true";
    button.addEventListener("click", function (event) {
      window.SheetFunctionsUI.open(event.currentTarget || button, tab);
    });
  }

  function boot() {
    wire("sheetFunctionsWizardBtn", "functions");
    wire("sheetFunctionsSettingsBtn", "settings");
    window.addEventListener("gb-sheet-locale", function () {
      window.SheetFunctionsUI.relocalize();
    });
    window.addEventListener("gb-sheet-functions-locale", function () {
      window.SheetFunctionsUI.relocalize();
    });
  }

  window.SheetFunctions = {
    errors: window.SheetFormulaErrors,
    parser: window.SheetFunctionsParser,
    catalog: window.SheetFunctionsCatalog,
    evaluator: window.SheetFunctionsEvaluator,
    settings: window.SheetFunctionsSettings,
    ui: window.SheetFunctionsUI,
    selfTest: window.SheetFunctionsSelfTest,
    open: function (tab) { window.SheetFunctionsUI.open(null, tab); },
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot, { once: true });
  } else {
    boot();
  }
})();
