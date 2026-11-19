"use strict";
/* Issue #2434 - function wizard: category browser, search, signature, argument
 * form, live preview and insert-into-formula-bar. No window.prompt. */
(function () {
  var C = window.SheetFunctionsCatalog;
  var E = window.SheetFormulaErrors;
  var V = window.SheetFunctionsEvaluator;
  var S = window.SheetFunctionsSettings;
  var UI = window.SheetFunctionsUI;

  function labels() {
    return {
      all: UI.t("sheet-fn-category-all", "All"),
      financial: UI.t("sheet-fn-category-financial", "Financial"),
      date: UI.t("sheet-fn-category-date", "Date"),
      statistical: UI.t("sheet-fn-category-statistical", "Statistical"),
      text: UI.t("sheet-fn-category-text", "Text"),
      logic: UI.t("sheet-fn-category-logic", "Logic"),
      search: UI.t("sheet-fn-search", "Search functions"),
      results: UI.t("sheet-fn-results", "Functions"),
      signature: UI.t("sheet-fn-signature", "Signature"),
      help: UI.t("sheet-fn-help", "Description"),
      example: UI.t("sheet-fn-example", "Example"),
      args: UI.t("sheet-fn-args", "Arguments"),
      preview: UI.t("sheet-fn-preview", "Live preview"),
      insert: UI.t("sheet-fn-insert", "Insert into cell"),
      inserted: UI.t("sheet-fn-inserted", "Formula placed in the formula bar. Press Enter to apply it to the selected cell."),
      badgeNative: UI.t("sheet-fn-badge-native", "Native engine"),
      badgeLocal: UI.t("sheet-fn-badge-local", "Local preview"),
      noSelection: UI.t("sheet-fn-no-selection", "Select a function to see its help."),
      noResults: UI.t("sheet-fn-no-results", "No functions match the search."),
      argPlaceholder: UI.t("sheet-fn-arg-placeholder", "Number, text or range"),
    };
  }

  function field(label, className, value) {
    var node = UI.element("div", "ss-fn-field");
    if (label) node.appendChild(UI.element("span", "ss-fn-field-label", label));
    var input = UI.element("input", "ss-fn-input " + (className || ""));
    input.type = "text";
    if (value !== undefined) input.value = value;
    node.appendChild(input);
    return node;
  }

  function render(container, ctx) {
    var L = labels();
    var state = { category: "all", query: "", selected: null, args: [] };
    var layout = UI.element("div", "ss-fn-wizard");
    var controls = UI.element("div", "ss-fn-controls");
    var search = UI.element("input", "ss-fn-search");
    search.type = "search";
    search.placeholder = L.search;
    search.setAttribute("data-i18n-placeholder", "sheet-fn-search");
    search.setAttribute("aria-label", L.search);
    search.setAttribute("data-i18n-aria-label", "sheet-fn-search");
    controls.appendChild(search);
    var categoryBar = UI.element("div", "ss-fn-categories");
    controls.appendChild(categoryBar);
    var columns = UI.element("div", "ss-fn-columns");
    var list = UI.element("div", "ss-fn-list");
    list.setAttribute("role", "listbox");
    list.setAttribute("aria-label", L.results);
    var detail = UI.element("div", "ss-fn-detail");
    columns.appendChild(list);
    columns.appendChild(detail);
    layout.appendChild(controls);
    layout.appendChild(columns);
    container.appendChild(layout);

    var categoryButtons = [];
    ["all"].concat(C.categories()).forEach(function (id) {
      var button = UI.element("button", "ss-fn-category", L[id]);
      button.type = "button";
      button.dataset.category = id;
      categoryButtons.push(button);
      categoryBar.appendChild(button);
    });

    function matches(fn) {
      if (state.category !== "all" && fn.category !== state.category) return false;
      var needle = state.query.toUpperCase();
      if (!needle) return true;
      var locale = S.state().locale;
      var aliases = Object.keys(C.ALIASES[locale] || {}).filter(function (key) {
        return C.ALIASES[locale][key] === fn.name;
      });
      return fn.name.toUpperCase().indexOf(needle) >= 0 ||
        aliases.join(" ").toUpperCase().indexOf(needle) >= 0;
    }

    function badgeFor(fn) {
      var badge = UI.element("span", "ss-fn-badge ss-fn-badge-" + fn.badge,
        fn.badge === "native" ? L.badgeNative : L.badgeLocal);
      return badge;
    }

    function previewFormula() {
      var values = state.args.map(function (value) { return String(value || "").trim(); });
      while (values.length && values[values.length - 1] === "") values.pop();
      return state.selected.name + "(" + values.join(",") + ")";
    }

    function renderPreview(node) {
      if (!state.selected) return;
      var formula = previewFormula();
      var settings = S.state();
      var result = V.evaluate(formula, { locale: settings.locale, settings: settings });
      node.textContent = formula + " = " + V.format(result, { settings: settings });
    }

    function renderDetail() {
      detail.textContent = "";
      var fn = state.selected;
      if (!fn) {
        detail.appendChild(UI.element("p", "ss-fn-empty", L.noSelection));
        return;
      }
      var title = UI.element("h3", "ss-fn-fn-title", fn.name);
      title.appendChild(document.createTextNode(" "));
      title.appendChild(badgeFor(fn));
      detail.appendChild(title);
      var signature = UI.element("p", "ss-fn-signature", L.signature + ": " + fn.signature);
      detail.appendChild(signature);
      detail.appendChild(UI.element("p", "ss-fn-help-text", UI.t(fn.helpKey, "")));
      detail.appendChild(UI.element("p", "ss-fn-example", L.example + ": " + fn.example));
      detail.appendChild(UI.element("h4", "", L.args));
      var argsBox = UI.element("div", "ss-fn-args-box");
      state.args = [];
      fn.args.forEach(function (arg, index) {
        var item = field(arg, "ss-fn-arg", "");
        var input = item.querySelector("input");
        input.setAttribute("data-i18n-placeholder", "sheet-fn-arg-placeholder");
        input.addEventListener("input", function () {
          state.args[index] = input.value;
          renderPreview(previewNode);
        });
        state.args.push("");
        argsBox.appendChild(item);
      });
      detail.appendChild(argsBox);
      var previewNode = UI.element("p", "ss-fn-preview");
      previewNode.setAttribute("role", "status");
      previewNode.setAttribute("aria-live", "polite");
      detail.appendChild(UI.element("h4", "", L.preview));
      detail.appendChild(previewNode);
      renderPreview(previewNode);
      var insert = UI.element("button", "ss-fn-btn", L.insert);
      insert.type = "button";
      insert.addEventListener("click", function () {
        var bar = document.getElementById("formulaInput");
        var formula = "=" + (state.args.some(function (value) { return String(value).trim(); }) ? previewFormula() : fn.example);
        if (!bar) {
          ctx.setStatus(UI.t("sheet-fn-inserted", L.inserted), false);
          return;
        }
        bar.value = formula;
        bar.dispatchEvent(new Event("input", { bubbles: true }));
        bar.dispatchEvent(new Event("change", { bubbles: true }));
        bar.focus();
        ctx.setStatus(UI.t("sheet-fn-inserted", L.inserted), false);
      });
      detail.appendChild(insert);
    }

    function renderList() {
      list.textContent = "";
      var found = C.all().filter(matches);
      if (!found.length) list.appendChild(UI.element("p", "ss-fn-empty", L.noResults));
      found.forEach(function (fn) {
        var button = UI.element("button", "ss-fn-item", fn.name);
        button.type = "button";
        button.setAttribute("role", "option");
        button.setAttribute("aria-selected", state.selected && state.selected.name === fn.name ? "true" : "false");
        button.addEventListener("click", function () {
          state.selected = fn;
          state.args = [];
          Array.prototype.forEach.call(list.children, function (child) {
            child.setAttribute("aria-selected", child === button ? "true" : "false");
          });
          renderDetail();
        });
        list.appendChild(button);
      });
    }

    categoryButtons.forEach(function (button) {
      button.addEventListener("click", function () {
        state.category = button.dataset.category;
        Array.prototype.forEach.call(categoryButtons, function (item) {
          item.classList.toggle("is-active", item === button);
        });
        renderList();
      });
    });
    search.addEventListener("input", function () {
      state.query = search.value;
      renderList();
    });
    categoryButtons[0].classList.add("is-active");
    renderList();
    renderDetail();
  }

  window.SheetFunctionsWizard = { render: render };
})();
