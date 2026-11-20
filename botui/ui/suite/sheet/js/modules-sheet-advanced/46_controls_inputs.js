"use strict";
/* Sheets enterprise controls (#2437) — unlocked input ranges: mark ranges,
 * validate entered values against simple local rules and highlight them for
 * reviewers. All state stays in this browser. */

(function () {
  var C = window.SheetControls;

  var RULES = [
    { key: "any", label: "sheet-ctrl-input-rule-any", fallback: "Any value" },
    { key: "number", label: "sheet-ctrl-input-rule-number", fallback: "Number" },
    { key: "email", label: "sheet-ctrl-input-rule-email", fallback: "Email address" },
    { key: "date", label: "sheet-ctrl-input-rule-date", fallback: "Date" },
    { key: "required", label: "sheet-ctrl-input-rule-required", fallback: "Required value" },
  ];

  function ruleLabel(rule) {
    var found = RULES.filter(function (item) { return item.key === rule; })[0];
    return found ? C.t(found.label, found.fallback) : String(rule);
  }

  function validRef(ref) {
    return /^\$?[A-Za-z]{1,3}\$?[0-9]+(?::\$?[A-Za-z]{1,3}\$?[0-9]+)?$/.test(String(ref || "").trim());
  }

  function validate(rule, value) {
    var text = String(value == null ? "" : value).trim();
    if (rule === "number") {
      if (text === "") return false;
      return Number.isFinite(Number(text.replace(/,/g, "")));
    }
    if (rule === "email") return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(text);
    if (rule === "date") return text !== "" && !Number.isNaN(Date.parse(text));
    if (rule === "required") return text.length > 0;
    return true;
  }

  function mark(ref, rule) {
    var clean = String(ref || "").trim().toUpperCase();
    if (!validRef(clean)) return false;
    var chosen = RULES.filter(function (item) { return item.key === rule; })[0] ? rule : "any";
    C.mutate(function (state) {
      state.inputs = state.inputs.filter(function (entry) { return entry.ref !== clean; });
      state.inputs.push({ ref: clean, rule: chosen });
    });
    highlight();
    return true;
  }

  function remove(ref) {
    C.mutate(function (state) {
      state.inputs = state.inputs.filter(function (entry) { return entry.ref !== String(ref || "").trim().toUpperCase(); });
    });
    highlight();
    return true;
  }

  function currentValue(ref) {
    var data = null;
    if (window.SheetChartingData && window.SheetChartingData.cellValue) {
      data = window.SheetChartingData.cellValue(ref);
    }
    if (data != null && String(data) !== "") return String(data);
    return "";
  }

  function apply(ref, value) {
    var clean = String(ref || "").trim().toUpperCase();
    var entry = C.state().inputs.filter(function (item) { return item.ref === clean; })[0];
    if (!entry) return { ok: false, key: "sheet-ctrl-input-needs" };
    if (!validate(entry.rule, value)) {
      return { ok: false, key: "sheet-ctrl-input-invalid", rule: ruleLabel(entry.rule) };
    }
    if (window.SheetControlsAudit && window.SheetControlsAudit.recordEdit) {
      window.SheetControlsAudit.recordEdit(clean, currentValue(clean), String(value), false);
    }
    return { ok: true, ref: clean, rule: entry.rule };
  }

  function highlight() {
    var refs = C.state().inputs.map(function (entry) { return entry.ref; });
    var cells = document.querySelectorAll("[data-ref]");
    var changed = 0;
    Array.prototype.forEach.call(cells, function (cell) {
      var ref = String(cell.getAttribute("data-ref") || "").toUpperCase();
      var match = refs.indexOf(ref) !== -1;
      if (match && (cell.className || "").indexOf("ss-ctrl-input-cell") === -1) {
        cell.className = ((cell.className || "") + " ss-ctrl-input-cell").trim();
        changed++;
      } else if (!match && (cell.className || "").indexOf("ss-ctrl-input-cell") !== -1) {
        cell.className = String(cell.className || "").replace(/\s*ss-ctrl-input-cell/g, "");
        changed++;
      }
    });
    return changed;
  }

  function render(container, ctx) {
    var doc = container.ownerDocument || document;
    function make(tag, className, text) {
      var node = doc.createElement(tag);
      if (className) node.className = className;
      if (text != null) node.textContent = text;
      return node;
    }
    var grid = make("div", "ss-ctrl-form");
    var inputs = C.state().inputs;
    var list = make("ul", "ss-ctrl-list");
    if (!inputs.length) {
      var empty = make("li", "ss-ctrl-empty", C.t("sheet-ctrl-input-empty", "No input ranges are marked yet."));
      empty.setAttribute("data-i18n", "sheet-ctrl-input-empty");
      list.appendChild(empty);
    } else {
      inputs.forEach(function (entry) {
        var item = make("li", "ss-ctrl-list-item");
        item.textContent = entry.ref + " - " + ruleLabel(entry.rule);
        item.setAttribute("data-ref", entry.ref);
        var button = make("button", "ss-ctrl-button-small", C.t("sheet-ctrl-remove", "Remove"));
        button.type = "button";
        button.setAttribute("data-i18n", "sheet-ctrl-remove");
        button.addEventListener("click", function () {
          remove(entry.ref);
          ctx.setStatus(C.t("sheet-ctrl-status-ready", "Ready."), false);
          ctx.rerender();
        });
        item.appendChild(button);
        list.appendChild(item);
      });
    }
    grid.appendChild(list);

    var addRow = make("div", "ss-ctrl-row");
    var ref = make("input", "ss-ctrl-input");
    ref.type = "text";
    ref.id = "ss-ctrl-input-ref";
    ref.placeholder = C.t("sheet-ctrl-input-ref", "Cell range");
    ref.setAttribute("data-i18n-placeholder", "sheet-ctrl-input-ref");
    var rule = make("select", "ss-ctrl-input");
    rule.id = "ss-ctrl-input-rule";
    RULES.forEach(function (item) {
      var option = make("option", "", C.t(item.label, item.fallback));
      option.value = item.key;
      option.setAttribute("data-i18n", item.label);
      rule.appendChild(option);
    });
    var add = make("button", "ss-ctrl-button", C.t("sheet-ctrl-input-add", "Mark input range"));
    add.type = "button";
    add.id = "ss-ctrl-input-add";
    add.setAttribute("data-i18n", "sheet-ctrl-input-add");
    addRow.appendChild(ref);
    addRow.appendChild(rule);
    addRow.appendChild(add);
    grid.appendChild(addRow);

    var applyRow = make("div", "ss-ctrl-row");
    var value = make("input", "ss-ctrl-input");
    value.type = "text";
    value.id = "ss-ctrl-input-value";
    value.placeholder = C.t("sheet-ctrl-input-value", "Value to validate");
    value.setAttribute("data-i18n-placeholder", "sheet-ctrl-input-value");
    var apply = make("button", "ss-ctrl-button", C.t("sheet-ctrl-input-apply", "Validate and record"));
    apply.type = "button";
    apply.id = "ss-ctrl-input-apply";
    apply.setAttribute("data-i18n", "sheet-ctrl-input-apply");
    applyRow.appendChild(value);
    applyRow.appendChild(apply);
    grid.appendChild(applyRow);
    container.appendChild(grid);

    add.addEventListener("click", function () {
      var ok = mark(ref.value, rule.value);
      ctx.setStatus(ok ? C.t("sheet-ctrl-status-ready", "Ready.") : C.t("sheet-ctrl-input-needs", "Enter a range and a value."), !ok);
      if (ok) ctx.rerender();
    });
    apply.addEventListener("click", function () {
      var result = applyValue(ref.value || (inputs[0] && inputs[0].ref), value.value);
      ctx.setStatus(result.ok
        ? C.t("sheet-ctrl-input-valid", "Value accepted and recorded locally.")
        : (result.key === "sheet-ctrl-input-invalid"
          ? C.t("sheet-ctrl-input-invalid", "Invalid {rule} value.", { rule: result.rule })
          : C.t("sheet-ctrl-input-needs", "Enter a range and a value.")), !result.ok);
    });
  }

  function applyValue(ref, value) { return apply(ref, value); }

  window.SheetControlsInputs = {
    RULES: RULES,
    ruleLabel: ruleLabel,
    validRef: validRef,
    validate: validate,
    mark: mark,
    remove: remove,
    apply: apply,
    highlight: highlight,
    render: render,
  };
})();
