"use strict";
/* Docs table dialogs (#2482): width, text-to-table, style picker and the
 * shared focus-trapped modal used by the accessibility panel. */
(function (window) {
  var NS = (window.DocsTables = window.DocsTables || {});
  var UI = (window.DocsTablesUI = window.DocsTablesUI || {});

  /* ---- accessible modal: focus trap, Esc, focus restore ---- */
  UI.openDialog = function (config) {
    var host = document.getElementById("docs-modal-container") || document.body;
    var restore = document.activeElement;
    var backdrop = UI.el("div", "gb-dtbl-backdrop");
    var dialog = UI.el("div", "gb-dtbl-dialog");
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");
    var titleId = NS.uid();
    dialog.setAttribute("aria-labelledby", titleId);
    var heading = UI.el("h3");
    heading.id = titleId;
    heading.setAttribute("data-i18n", config.titleKey);
    heading.textContent = NS.tr(config.titleKey, config.titleFallback);
    dialog.appendChild(heading);
    var body = UI.el("div", "gb-dtbl-body");
    if (typeof config.render === "function") config.render(body);
    dialog.appendChild(body);
    var actions = UI.el("div", "gb-dtbl-actions");
    var cancel = UI.makeButton("close", "docs-tables-cancel", "Cancel", "M6 6l12 12M18 6L6 18");
    cancel.removeAttribute("data-docs-table-action");
    actions.appendChild(cancel);
    var apply = null;
    if (config.applyKey !== null) {
      apply = UI.el("button", "gb-dtbl-primary");
      apply.setAttribute("type", "button");
      apply.setAttribute("data-i18n", config.applyKey || "docs-tables-apply");
      apply.textContent = NS.tr(config.applyKey || "docs-tables-apply", config.applyFallback || "Apply");
      actions.appendChild(apply);
    }
    dialog.appendChild(actions);
    backdrop.appendChild(dialog);

    function close() {
      document.removeEventListener("keydown", onKey, true);
      if (backdrop.parentNode) backdrop.parentNode.removeChild(backdrop);
      if (restore && restore.focus) restore.focus();
    }
    function focusables() {
      return Array.prototype.filter.call(
        dialog.querySelectorAll("button,input,textarea,select,[tabindex]:not([tabindex='-1'])"),
        function (node) { return !node.disabled; }
      );
    }
    function onKey(event) {
      if (event.key === "Escape") { event.preventDefault(); close(); return; }
      if (event.key !== "Tab") return;
      var list = focusables();
      if (!list.length) return;
      var index = list.indexOf(document.activeElement);
      var next = event.shiftKey
        ? (index - 1 + list.length) % list.length
        : (index + 1) % list.length;
      event.preventDefault();
      list[next].focus();
    }
    cancel.addEventListener("click", close);
    if (apply) apply.addEventListener("click", function () {
      if (typeof config.onApply === "function") config.onApply(body, close);
    });
    backdrop.addEventListener("click", function (event) { if (event.target === backdrop) close(); });
    document.addEventListener("keydown", onKey, true);
    host.appendChild(backdrop);
    var first = focusables()[0];
    if (first) first.focus();
    return close;
  };

  UI.field = function (kind, key, fallback, id) {
    var wrap = UI.el("label");
    var caption = UI.el("span");
    caption.setAttribute("data-i18n", key);
    caption.textContent = NS.tr(key, fallback);
    wrap.appendChild(caption);
    var input = UI.el(kind === "textarea" ? "textarea" : "input");
    input.id = id;
    if (kind !== "textarea") input.setAttribute("type", kind === "number" ? "number" : "text");
    input.setAttribute("data-i18n-placeholder", key);
    input.setAttribute("placeholder", NS.tr(key, fallback));
    wrap.appendChild(input);
    return { wrap: wrap, input: input };
  };

  UI.openWidthDialog = function () {
    UI.openDialog({
      titleKey: "docs-tables-col-width", titleFallback: "Set column width",
      render: function (body) {
        var field = UI.field("number", "docs-tables-width-label", "Width in pixels", "gb-dtbl-width");
        field.input.min = "20";
        field.input.max = "1200";
        body.appendChild(field.wrap);
      },
      onApply: function (body, close) {
        var value = parseInt(body.querySelector("#gb-dtbl-width").value, 10);
        close();
        if (NS.ops && NS.ops.setColumnWidth) NS.ops.setColumnWidth(value > 0 ? value : 120);
      },
    });
  };

  UI.openTextToTable = function () {
    UI.openDialog({
      titleKey: "docs-tables-text-to-table", titleFallback: "Convert text to table",
      render: function (body) {
        var hint = UI.el("p");
        hint.setAttribute("data-i18n", "docs-tables-text-hint");
        hint.textContent = NS.tr("docs-tables-text-hint", "One row per line; separate columns with tabs or commas");
        body.appendChild(hint);
        var text = UI.field("textarea", "docs-tables-text-label", "Table text", "gb-dtbl-text");
        body.appendChild(text.wrap);
        var sep = UI.field("text", "docs-tables-separator", "Column separator", "gb-dtbl-sep");
        sep.input.value = ",";
        body.appendChild(sep.wrap);
      },
      onApply: function (body, close) {
        var text = body.querySelector("#gb-dtbl-text").value;
        var sep = body.querySelector("#gb-dtbl-sep").value;
        close();
        if (NS.ops && NS.ops.textToTable) NS.ops.textToTable(text, sep);
      },
    });
  };

  UI.openStylePicker = function () {
    var styles = [
      ["docs-tbl-style-plain", "docs-tables-style-plain", "Plain"],
      ["docs-tbl-style-grid", "docs-tables-style-grid", "Grid"],
      ["docs-tbl-style-banded", "docs-tables-style-banded", "Banded rows"],
      ["docs-tbl-style-accent", "docs-tables-style-accent", "Accent"],
      ["docs-tbl-style-minimal", "docs-tables-style-minimal", "Minimal"],
    ];
    var closeDialog = UI.openDialog({
      titleKey: "docs-tables-styles", titleFallback: "Table styles", applyKey: null,
      render: function (body) {
        var grid = UI.el("div", "gb-dtbl-style-list");
        styles.forEach(function (spec) {
          var styleButton = UI.el("button");
          styleButton.setAttribute("type", "button");
          styleButton.setAttribute("data-i18n", spec[1]);
          styleButton.textContent = NS.tr(spec[1], spec[2]);
          styleButton.addEventListener("click", function () {
            if (NS.ops && NS.ops.applyStyle) NS.ops.applyStyle(spec[0]);
            closeDialog();
          });
          grid.appendChild(styleButton);
        });
        body.appendChild(grid);
      },
    });
  };

})(window);
