"use strict";
/* Sheets data connectivity (#2492) - parameters that feed the local query
 * definition. window.SheetConnectParams. Prompts and defaults are stored with
 * the workbook-scoped local state; no value is transmitted anywhere. */

(function () {
  var C = window.SheetConnectCore;
  var U = window.SheetConnectUi;

  function render(container, ctx) {
    var target = ctx.state();
    container.appendChild(U.title("sheet-connect-param-title", "Parameters"));
    container.appendChild(U.note("sheet-connect-param-feed",
      "Parameters feed the query definition and the scheduled refresh preview."));
    if (!target.parameters.length) {
      container.appendChild(U.note("sheet-connect-param-empty", "No parameters defined."));
      return;
    }
    var list = U.el("div", "ss-connect-param-list");
    target.parameters.forEach(function (parameter) {
      var card = U.el("div", "ss-connect-param");
      var label = U.el("span", "ss-connect-param-label", C.t(parameter.labelKey, parameter.labelFallback));
      label.setAttribute("data-i18n", parameter.labelKey);
      card.appendChild(label);
      var prompt = U.el("p", "ss-connect-param-prompt", C.t(parameter.promptKey, parameter.promptFallback));
      prompt.setAttribute("data-i18n", parameter.promptKey);
      card.appendChild(prompt);
      var row = U.el("div", "ss-connect-param-row");
      var def = U.field("sheet-connect-param-default", "Default value", "text", parameter.defaultValue);
      def.input.disabled = true;
      row.appendChild(def);
      var value = U.field("sheet-connect-param-value", "Current value", parameter.kind === "date" ? "date" : "text", parameter.value);
      value.input.addEventListener("change", function () {
        C.mutate(function (state) {
          state.parameters = state.parameters.map(function (item) {
            return item.id === parameter.id ? Object.assign({}, item, { value: value.input.value }) : item;
          });
        });
        ctx.setStatus(C.t("sheet-connect-param-saved", "Parameters saved to the local definition."));
      });
      row.appendChild(value);
      card.appendChild(row);
      list.appendChild(card);
    });
    container.appendChild(list);
    var actions = U.el("div", "ss-connect-actions");
    actions.appendChild(U.button("sheet-connect-param-reset", "Reset to defaults", function () {
      C.mutate(function (state) {
        state.parameters = state.parameters.map(function (item) {
          return Object.assign({}, item, { value: item.defaultValue });
        });
      });
      ctx.setStatus(C.t("sheet-connect-param-reset-done", "Parameters reset to their defaults."));
      ctx.rerender();
    }));
    container.appendChild(actions);
  }

  window.SheetConnectParams = { render: render };
})();
