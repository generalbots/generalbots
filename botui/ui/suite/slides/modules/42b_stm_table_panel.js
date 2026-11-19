"use strict";
/* Slides table & media authoring (#2487) — table panel synchronisation. Split
 * from the editor module to keep each source file below the project limit. */
(function () {
  var STM = window.SlidesTableMedia = window.SlidesTableMedia || {};

  STM.syncTablePanel = function () {
    var state = STM.tableState;
    var panel = state && state.panel;
    if (!panel) return;
    var options = state.options || STM.tableDefaults();
    panel.headerInput.checked = !!options.header;
    panel.bandedInput.checked = !!options.banded;
    panel.bordersInput.checked = !!options.borders;
    panel.paddingInput.value = String(options.padding === undefined ? 4 : options.padding);
    panel.alignSelect.value = options.align || "left";
    panel.styleSelect.value = options.style || "plain";
    panel.widths.innerHTML = "";
    var widths = options.widths || [];
    var model = state.model;
    for (var i = 0; i < (model ? model.cols : 0); i++) {
      var input = document.createElement("input");
      input.type = "number";
      input.min = "0";
      input.step = "10";
      input.value = String(widths[i] || "");
      STM.labelled(input, "data-i18n-aria-label", "slides-stm-table-width", "Column width");
      input.addEventListener("change", function () {
        var current = STM.tableById(state.activeId);
        if (!current) return;
        var before = { model: STM.cloneTable(state.model), options: STM.cloneTable(state.options) };
        var all = Array.prototype.slice.call(panel.widths.querySelectorAll("input"));
        var nextOptions = STM.tableWidths(state.model, state.options, all.map(function (node) { return node.value; }));
        STM.tableCommit(current, before, { model: STM.cloneTable(state.model), options: nextOptions },
          "slides-stm-table-widths-saved", "Column widths updated");
      });
      panel.widths.appendChild(input);
    }
  };
})();
