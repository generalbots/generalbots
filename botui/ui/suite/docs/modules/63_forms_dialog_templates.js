"use strict";
/* Template gallery and content-control inspector dialogs (#2427). */
(function (window) {
  var NS = window.DocsForms;
  if (!NS) return;
  var D = NS.dialog;
  if (!D) return;
  var UI = (NS.dialogs = NS.dialogs || {});

  UI.openTemplates = function () {
    var shell = D.shell({ titleKey: "docs-forms-template-title", titleFallback: "Template gallery" });
    var search = D.input("input", "docs-forms-search-placeholder", "Search templates\u2026");
    shell.body.appendChild(D.field("docs-forms-search-placeholder", "Search templates\u2026", search));
    var list = D.el("div", "gb-forms-template-list");
    var preview = D.el("div", "gb-forms-template-preview gb-forms-preview-note");
    var selected = null;

    function render() {
      list.textContent = "";
      var matches = NS.templates.search(search.value);
      if (!matches.length) {
        var empty = D.el("p", "gb-forms-empty");
        D.i18n(empty, "docs-forms-template-empty", "No templates match the search");
        list.appendChild(empty);
      }
      matches.forEach(function (template) {
        var card = D.el("button", "gb-forms-template-card");
        card.setAttribute("type", "button");
        card.setAttribute("data-form-template", template.id);
        card.textContent = NS.templates.name(template);
        card.addEventListener("click", function () {
          selected = template;
          preview.innerHTML = NS.templates.buildHtml(template);
          shell.status.textContent = NS.templates.name(template);
        });
        list.appendChild(card);
      });
    }

    var apply = D.button("docs-forms-template-apply", "Apply template", true);
    apply.addEventListener("click", function () {
      if (!selected) return;
      NS.templates.apply(selected);
      shell.close();
    });
    search.addEventListener("input", render);
    shell.body.appendChild(list);
    shell.body.appendChild(preview);
    shell.body.appendChild(apply);
    render();
    return shell.close;
  };

  function controlRow(descriptor, refresh) {
    var row = D.el("div", "gb-forms-control-row");
    var button = D.el("button", "gb-forms-control-pick");
    button.setAttribute("type", "button");
    button.setAttribute("data-form-control-id", descriptor.id);
    button.textContent = descriptor.label + " (" + NS.controls.typeName(descriptor.type) + ")";
    button.addEventListener("click", function () { refresh(descriptor.id); });
    row.appendChild(button);
    return row;
  }

  UI.openControls = function () {
    var shell = D.shell({ titleKey: "docs-forms-control-title", titleFallback: "Content controls" });
    var select = D.el("select", "gb-forms-input");
    NS.controls.TYPES.forEach(function (type) {
      var option = D.el("option");
      option.setAttribute("value", type.id);
      option.textContent = NS.controls.typeName(type.id);
      select.appendChild(option);
    });
    var insert = D.button("docs-forms-control-insert", "Insert control", true);
    insert.addEventListener("click", function () {
      NS.controls.insert(select.value, NS.getArticle());
      renderList();
      shell.status.textContent = NS.tr("docs-forms-control-inserted", "Control inserted into the document");
    });
    shell.body.appendChild(select);
    shell.body.appendChild(insert);

    var list = D.el("div", "gb-forms-control-list");
    var inspector = D.el("div", "gb-forms-inspector");
    shell.body.appendChild(list);
    shell.body.appendChild(inspector);

    function renderList() {
      list.textContent = "";
      NS.controls.syncFromDom(NS.getArticle());
      if (!NS.state.controls.length) {
        var empty = D.el("p", "gb-forms-empty");
        D.i18n(empty, "docs-forms-control-no-selection", "Select a control in the document first");
        list.appendChild(empty);
      }
      NS.state.controls.forEach(function (descriptor) {
        list.appendChild(controlRow(descriptor, showInspector));
      });
    }

    function showInspector(id) {
      inspector.textContent = "";
      var descriptor = NS.state.controls.filter(function (item) { return item.id === id; })[0];
      if (!descriptor) return;
      var label = D.input("input", "docs-forms-control-label", "Label", descriptor.label);
      var options = D.input("textarea", "docs-forms-control-options", "Options (one per line)", descriptor.options.join("\n"));
      var value = D.input("input", "docs-forms-control-default", "Default value", descriptor.value);
      var required = D.el("input");
      required.setAttribute("type", "checkbox");
      required.setAttribute("data-i18n-aria-label", "docs-forms-control-required");
      required.setAttribute("aria-label", NS.tr("docs-forms-control-required", "Required field"));
      required.checked = descriptor.required;
      var save = D.button("docs-forms-control-save", "Save control", true);
      save.addEventListener("click", function () {
        NS.controls.updateInDom(id, {
          label: label.value,
          options: options.value,
          value: value.value,
          required: required.checked
        }, NS.getArticle());
        renderList();
        shell.status.textContent = NS.tr("docs-forms-control-save", "Save control");
      });
      inspector.appendChild(D.field("docs-forms-control-inspector", "Field inspector", label));
      inspector.appendChild(options);
      inspector.appendChild(D.field("docs-forms-control-default", "Default value", value));
      var reqLabel = D.el("label", "gb-forms-field");
      D.i18n(reqLabel, "docs-forms-control-required", "Required field");
      reqLabel.appendChild(required);
      inspector.appendChild(reqLabel);
      inspector.appendChild(save);
    }

    renderList();
    return shell.close;
  };
})(window);
