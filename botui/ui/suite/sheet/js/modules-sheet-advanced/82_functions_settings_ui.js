"use strict";
/* Issue #2434 - calculation settings panel. All values are local-preview
 * settings in workbook-scoped localStorage; no workbook or server write. */
(function () {
  var S = window.SheetFunctionsSettings;
  var UI = window.SheetFunctionsUI;

  function labelled(label, control) {
    var row = UI.element("div", "ss-fn-setting-row");
    var name = UI.element("label", "ss-fn-setting-label", label);
    if (control.id) name.htmlFor = control.id;
    row.appendChild(name);
    row.appendChild(control);
    return row;
  }

  function select(id, options, value) {
    var node = UI.element("select", "ss-fn-input");
    node.id = id;
    options.forEach(function (option) {
      var item = UI.element("option", "", option.label);
      item.value = option.value;
      node.appendChild(item);
    });
    node.value = value;
    return node;
  }

  function checkbox(id, checked) {
    var node = UI.element("input", "ss-fn-checkbox");
    node.type = "checkbox";
    node.id = id;
    node.checked = Boolean(checked);
    return node;
  }

  function number(id, value, min, max, step) {
    var node = UI.element("input", "ss-fn-input");
    node.type = "number";
    node.id = id;
    node.value = String(value);
    if (min !== undefined) node.min = String(min);
    if (max !== undefined) node.max = String(max);
    if (step !== undefined) node.step = String(step);
    return node;
  }

  function render(container, ctx) {
    var current = S.state();
    var labels = {
      mode: UI.t("sheet-fn-mode", "Calculation mode"),
      auto: UI.t("sheet-fn-mode-auto", "Automatic"),
      manual: UI.t("sheet-fn-mode-manual", "Manual"),
      iterative: UI.t("sheet-fn-iterative", "Iterative calculation"),
      iterations: UI.t("sheet-fn-iterations", "Maximum iterations"),
      tolerance: UI.t("sheet-fn-tolerance", "Tolerance"),
      precision: UI.t("sheet-fn-precision", "Precision as displayed"),
      dateSystem: UI.t("sheet-fn-date-system", "Date system"),
      date1900: UI.t("sheet-fn-date-1900", "1900 date system"),
      date1904: UI.t("sheet-fn-date-1904", "1904 date system"),
      locale: UI.t("sheet-fn-locale", "Function locale"),
      decimal: UI.t("sheet-fn-decimal", "Decimal separator"),
      thousands: UI.t("sheet-fn-thousands", "Thousands separator"),
      apply: UI.t("sheet-fn-save-settings", "Apply settings"),
      saved: UI.t("sheet-fn-settings-saved", "Settings applied to this local preview."),
    };
    var panel = UI.element("div", "ss-fn-settings");
    var mode = select("ss-fn-mode", [
      { value: "auto", label: labels.auto }, { value: "manual", label: labels.manual },
    ], current.mode);
    panel.appendChild(labelled(labels.mode, mode));
    var iterative = checkbox("ss-fn-iterative", current.iterative.enabled);
    panel.appendChild(labelled(labels.iterative, iterative));
    panel.appendChild(labelled(labels.iterations, number("ss-fn-max-iterations", current.iterative.maxIterations, 1, 1000, 1)));
    var tolerance = number("ss-fn-tolerance", current.iterative.tolerance, 0, 1, 0.0000001);
    tolerance.step = "any";
    panel.appendChild(labelled(labels.tolerance, tolerance));
    panel.appendChild(labelled(labels.precision, checkbox("ss-fn-precision", current.precisionAsDisplayed)));
    panel.appendChild(labelled(labels.dateSystem, select("ss-fn-date-system", [
      { value: "1900", label: labels.date1900 }, { value: "1904", label: labels.date1904 },
    ], current.dateSystem)));
    var localeOptions = S.LOCALES.map(function (locale) {
      return { value: locale, label: locale };
    });
    var locale = select("ss-fn-locale", localeOptions, current.locale);
    panel.appendChild(labelled(labels.locale, locale));
    var decimal = UI.element("input", "ss-fn-input");
    decimal.id = "ss-fn-decimal";
    decimal.type = "text";
    decimal.maxLength = 1;
    decimal.value = current.separators.decimal;
    panel.appendChild(labelled(labels.decimal, decimal));
    var group = UI.element("input", "ss-fn-input");
    group.id = "ss-fn-group";
    group.type = "text";
    group.maxLength = 1;
    group.value = current.separators.group;
    panel.appendChild(labelled(labels.thousands, group));
    var apply = UI.element("button", "ss-fn-btn", labels.apply);
    apply.type = "button";
    apply.addEventListener("click", function () {
      var form = {
        mode: mode.value,
        iterative: iterative.checked,
        maxIterations: Number(document.getElementById("ss-fn-max-iterations").value),
        tolerance: Number(tolerance.value),
        precisionAsDisplayed: document.getElementById("ss-fn-precision").checked,
        dateSystem: document.getElementById("ss-fn-date-system").value,
        locale: locale.value,
        decimal: decimal.value,
        group: group.value,
      };
      var saved = S.applyForm(form);
      ctx.setStatus(saved ? labels.saved : UI.t("sheet-fn-settings-error", "Settings could not be saved."), !saved);
      if (saved && saved.locale !== current.locale) S.setLocale(saved.locale);
    });
    panel.appendChild(apply);
    container.appendChild(panel);
  }

  window.SheetFunctionsSettingsUI = { render: render };
})();
