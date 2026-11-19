"use strict";
/* Docs long-document structure (#2426), Document Structure dialog panels.
 *
 * Numbering levels, caption insertion, cross-reference insertion, field editing
 * and the update actions. Option text is translated from the docs-structure
 * catalog; document-derived option labels are content, not authored UI strings.
 */
(function (window) {
  var NS = window.DocsStructure;
  if (!NS) return;
  var UI = NS.ui;
  if (!UI) return;
  var panels = (NS.panels = NS.panels || {});

  function option(value, key, fallback) {
    return { value: value, key: key, fallback: fallback };
  }
  function selectedOption(select) {
    var options = select.options || (select.querySelectorAll ? select.querySelectorAll("option") : []);
    var index = typeof select.selectedIndex === "number" ? select.selectedIndex : 0;
    return options[index] || null;
  }
  var KIND_OPTIONS = [
    option("figure", "docs-structure-kind-figure", "Figure"),
    option("table", "docs-structure-kind-table", "Table"),
  ];
  var SCHEME_OPTIONS = [
    option("decimal", "docs-structure-scheme-decimal", "Decimal (1, 1.1)"),
    option("legal", "docs-structure-scheme-legal", "Legal (1., 1.1.)"),
  ];
  var FORMAT_OPTIONS = [
    option("number", "docs-structure-xref-format-number", "Number only"),
    option("text", "docs-structure-xref-format-text", "Text only"),
    option("full", "docs-structure-xref-format-full", "Number and text"),
  ];

  panels.numbering = function (body) {
    var section = UI.section(body, "docs-structure-section-numbering", "Heading numbering");
    var config = NS.config();
    var schemeControls = [];
    var continueControls = [];
    for (var i = 1; i <= NS.LEVELS; i++) {
      var row = UI.el("div", "gb-ds-level");
      UI.text(row, "docs-structure-level", "Level");
      row.appendChild(document.createTextNode(" " + i));
      schemeControls.push(UI.select(row, "docs-structure-scheme", "Numbering style",
        SCHEME_OPTIONS, "gb-ds-scheme-" + i));
      var setting = config.levels[i - 1] || {};
      schemeControls[i - 1].value = setting.scheme || "decimal";
      var continuing = UI.el("input");
      continuing.type = "checkbox";
      continuing.id = "gb-ds-continue-" + i;
      continuing.checked = setting.continuePrevious === true;
      UI.labeled(row, "docs-structure-continue-previous", "Continue previous list", continuing);
      continueControls.push(continuing);
      section.appendChild(row);
    }
    var restartAll = UI.el("input");
    restartAll.type = "checkbox";
    restartAll.id = "gb-ds-restart-section";
    restartAll.checked = config.restartPerSection !== false;
    UI.labeled(section, "docs-structure-restart-section", "Restart at each higher level", restartAll);
    UI.button(section, "docs-structure-apply-numbering", "Apply numbering", "gb-ds-btn gb-ds-primary", "gb-ds-apply-numbering")
      .addEventListener("click", function () {
        var levels = [];
        for (var j = 0; j < NS.LEVELS; j++) {
          levels.push({
            level: j + 1,
            scheme: schemeControls[j].value === "legal" ? "legal" : "decimal",
            continuePrevious: continueControls[j].checked === true,
          });
        }
        NS.setConfig({ levels: levels, restartPerSection: restartAll.checked === true });
        NS.applyNumbering();
      });
    UI.button(section, "docs-structure-restart-here", "Restart numbering at this heading", "gb-ds-btn", "gb-ds-restart-here")
      .addEventListener("click", function () { NS.restartHere(); });
  };

  panels.caption = function (body) {
    var section = UI.section(body, "docs-structure-caption-insert", "Insert caption");
    var kind = UI.select(section, "docs-structure-caption-kind", "Caption type", KIND_OPTIONS, "gb-ds-caption-kind");
    var title = UI.el("input");
    title.type = "text";
    title.id = "gb-ds-caption-title";
    title.setAttribute("data-i18n-placeholder", "docs-structure-caption-title-placeholder");
    title.setAttribute("placeholder", NS.tr("docs-structure-caption-title-placeholder",
      "Describe the figure or table"));
    UI.labeled(section, "docs-structure-caption-title", "Caption text", title);
    UI.button(section, "docs-structure-insert-caption", "Insert caption", "gb-ds-btn gb-ds-primary", "gb-ds-insert-caption")
      .addEventListener("click", function () {
        NS.insertCaption(kind.value, title.value.trim());
        title.value = "";
      });
  };

  function targetOptions(root) {
    return NS.targetsFrom(root).map(function (target) {
      return {
        value: target.id,
        label: (NS.kindLabel(target.kind) + " " + (target.number || "") + " " + (target.text || "")).trim(),
        kind: target.kind,
      };
    });
  }

  function fillTargets(select, root) {
    var items = targetOptions(root);
    while (select.firstChild) select.removeChild(select.firstChild);
    if (!items.length) {
      var empty = UI.el("option");
      empty.value = "";
      empty.textContent = NS.tr("docs-structure-no-targets", "No numbered items available yet");
      select.appendChild(empty);
      return items;
    }
    items.forEach(function (item) {
      var entry = UI.el("option");
      entry.value = item.value;
      entry.textContent = item.label;
      entry.setAttribute("data-gb-kind", item.kind);
      select.appendChild(entry);
    });
    return items;
  }

  panels.xref = function (body) {
    var section = UI.section(body, "docs-structure-xref-insert", "Insert cross-reference");
    var target = UI.el("select");
    target.id = "gb-ds-xref-target";
    fillTargets(target, NS.getArticle());
    UI.labeled(section, "docs-structure-xref-target", "Reference target", target);
    var format = UI.select(section, "docs-structure-xref-format", "Reference shows", FORMAT_OPTIONS, "gb-ds-xref-format");
    format.value = "full";
    UI.text(section, "docs-structure-page-deferred",
      "Page numbers need a layout engine and are not available yet.", "p");
    UI.button(section, "docs-structure-insert-xref", "Insert reference", "gb-ds-btn gb-ds-primary", "gb-ds-insert-xref")
      .addEventListener("click", function () {
        var chosen = selectedOption(target);
        var kind = chosen ? chosen.getAttribute("data-gb-kind") : "";
        NS.insertXref(target.value, format.value, kind);
      });
  };

  panels.fields = function (body) {
    var section = UI.section(body, "docs-structure-fields", "Fields");
    var fields = UI.el("select");
    fields.id = "gb-ds-field-select";
    var list = NS.fieldNodes(NS.getArticle());
    if (!list.length) {
      var empty = UI.el("option");
      empty.value = "";
      empty.textContent = NS.tr("docs-structure-no-fields", "No fields in this document");
      fields.appendChild(empty);
    }
    list.forEach(function (field, index) {
      var entry = UI.el("option");
      entry.value = String(index);
      entry.textContent = NS.tr("docs-structure-kind-" + (field.getAttribute("data-gb-field-kind") || "heading"),
        "Field") + " " + (field.getAttribute("data-gb-field-target") || "");
      fields.appendChild(entry);
    });
    UI.labeled(section, "docs-structure-field-select", "Field to edit", fields);
    var target = UI.el("select");
    target.id = "gb-ds-field-target";
    fillTargets(target, NS.getArticle());
    UI.labeled(section, "docs-structure-field-target", "Field target", target);
    var format = UI.select(section, "docs-structure-field-format", "Field display", FORMAT_OPTIONS, "gb-ds-field-format");
    format.value = "full";
    fields.addEventListener("change", function () {
      var field = list[parseInt(fields.value, 10)];
      if (!field) return;
      target.value = field.getAttribute("data-gb-field-target") || target.value;
      format.value = field.getAttribute("data-gb-field-format") || "full";
    });
    UI.button(section, "docs-structure-apply-field", "Apply field", "gb-ds-btn", "gb-ds-apply-field")
      .addEventListener("click", function () {
        var field = list[parseInt(fields.value, 10)];
        if (!field) return;
        var chosen = selectedOption(target);
        NS.editField(field, target.value, format.value,
          chosen ? chosen.getAttribute("data-gb-kind") : field.getAttribute("data-gb-field-kind"));
      });
  };

  panels.actions = function (body) {
    var section = UI.section(body, "docs-structure-update-all", "Update all fields");
    UI.button(section, "docs-structure-update-all", "Update all fields", "gb-ds-btn gb-ds-primary", "gb-ds-update-all")
      .addEventListener("click", function () { NS.updateAll(); });
    UI.button(section, "docs-structure-insert-tof", "Insert table of figures", "gb-ds-btn", "gb-ds-insert-tof")
      .addEventListener("click", function () { NS.insertList("figure"); });
    UI.button(section, "docs-structure-insert-tot", "Insert table of tables", "gb-ds-btn", "gb-ds-insert-tot")
      .addEventListener("click", function () { NS.insertList("table"); });
  };
})(window);
