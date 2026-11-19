"use strict";
/* #2537 - builder rendering. One delegated listener per surface so re-renders
   never leak handlers; focus is restored by data-path/data-part after each
   re-render to keep the dialog keyboard-first. */
(function (global) {
  var ns = global.GBSegments;
  if (!ns) return;

  ns.state = ns.state || { model: ns.emptyModel(), source: "header", opener: null, refresh: null, errors: [] };
  ns.onModelChange = function () {};

  function pathAttr(path) { return ns.escapeHtml(JSON.stringify(path)); }
  function parsePath(text) {
    try { return JSON.parse(text); } catch (e) { return []; }
  }

  function errorsFor(path) {
    return (ns.state.errors || []).filter(function (err) {
      return err.path.length === path.length && err.path.every(function (n, i) { return n === path[i]; });
    });
  }

  function errorMarkup(errors) {
    return errors.map(function (err) {
      return '<p class="gb-segment-error" role="alert">' + ns.escapeHtml(ns.t(err.key, err.fallback)) + '</p>';
    }).join("");
  }

  function fieldOptions(selected) {
    var html = "";
    ns.GROUPS.forEach(function (group) {
      var fields = ns.FIELDS.filter(function (f) { return f.group === group.id; });
      if (!fields.length) return;
      html += '<optgroup label="' + ns.escapeHtml(ns.t(group.key, group.label)) + '">';
      fields.forEach(function (field) {
        html += '<option value="' + field.id + '"' + (field.id === selected ? " selected" : "") + '>' +
          ns.escapeHtml(ns.fieldLabel(field.id)) + '</option>';
      });
      html += '</optgroup>';
    });
    return html;
  }

  function operatorOptions(field, selected) {
    if (!field) return '<option value="">' + ns.escapeHtml(ns.t("segments-select-operator", "Select an operator…")) + '</option>';
    return field.ops.map(function (op) {
      return '<option value="' + op + '"' + (op === selected ? " selected" : "") + '>' +
        ns.escapeHtml(ns.operatorLabel(op)) + '</option>';
    }).join("");
  }

  function valueControl(field, condition, path) {
    var common = 'data-part="value" data-path="' + pathAttr(path) + '" class="gb-segment-input gb-segment-value"';
    var value = condition.value == null ? "" : String(condition.value);
    if (!field) {
      return '<input type="text" ' + common + ' disabled data-i18n-placeholder="segments-enter-value" placeholder="' +
        ns.escapeHtml(ns.t("segments-enter-value", "Enter a value…")) + '">';
    }
    var multi = ["in", "not_in"].includes(condition.op);
    if (field.options && !multi) {
      var html = '<select ' + common + '><option value="">' +
        ns.escapeHtml(ns.t("segments-value-label", "Value")) + '</option>';
      field.options.forEach(function (option) {
        html += '<option value="' + option.value + '"' + (option.value === value ? " selected" : "") + '>' +
          ns.escapeHtml(ns.t(option.key, option.label)) + '</option>';
      });
      return html + '</select>';
    }
    var type = field.type === "number" ? "number" : (field.type === "date" && condition.op !== "within_days" ? "date" : "text");
    var placeholder = multi || field.type === "select"
      ? field.options.map(function (option) { return ns.t(option.key, option.label); }).join(", ")
      : ns.t("segments-enter-value", "Enter a value…");
    return '<input type="' + type + '" ' + common + ' value="' + ns.escapeHtml(value) +
      '" placeholder="' + ns.escapeHtml(placeholder) + '">';
  }

  function conditionMarkup(condition, path) {
    var errors = errorsFor(path);
    var field = ns.fieldById(condition.field);
    var invalid = errors.length ? " gb-segment-invalid" : "";
    return '<div class="gb-segment-row' + invalid + '" data-cond-path="' + pathAttr(path) + '">' +
      '<label class="gb-segment-cell"><span data-i18n="segments-field-label">' +
        ns.escapeHtml(ns.t("segments-field-label", "Field")) + '</span>' +
        '<select data-part="field" data-path="' + pathAttr(path) + '" class="gb-segment-input">' +
          '<option value="">' + ns.escapeHtml(ns.t("segments-select-field", "Select a field…")) + '</option>' +
          fieldOptions(condition.field) + '</select></label>' +
      '<label class="gb-segment-cell"><span data-i18n="segments-operator-label">' +
        ns.escapeHtml(ns.t("segments-operator-label", "Operator")) + '</span>' +
        '<select data-part="op" data-path="' + pathAttr(path) + '" class="gb-segment-input"' +
          (field ? "" : " disabled") + '>' + operatorOptions(field, condition.op) + '</select></label>' +
      '<label class="gb-segment-cell gb-segment-cell-value"><span data-i18n="segments-value-label">' +
        ns.escapeHtml(ns.t("segments-value-label", "Value")) + '</span>' + valueControl(field, condition, path) + '</label>' +
      '<button type="button" class="gb-segment-icon-btn gb-segment-remove" data-action="remove-condition" data-path="' +
        pathAttr(path) + '" data-i18n-aria-label="segments-remove-condition" data-i18n-title="segments-remove-condition" ' +
        'aria-label="' + ns.escapeHtml(ns.t("segments-remove-condition", "Remove condition")) + '" title="' +
        ns.escapeHtml(ns.t("segments-remove-condition", "Remove condition")) + '">×</button>' +
      (errors.length ? '<div class="gb-segment-row-errors">' + errorMarkup(errors) + '</div>' : '') +
      '</div>';
  }

  function matchMarkup(group, path, isRoot) {
    if (isRoot) return "";
    var name = "gb-segment-match-" + path.join("-");
    return '<fieldset class="gb-segment-match"><legend>' +
      ns.escapeHtml(ns.t("segments-nested-group-title", "Nested group")) + '</legend>' +
      ['all', 'any'].map(function (mode) {
        var key = mode === "all" ? "segments-match-all" : "segments-match-any";
        var fallback = mode === "all" ? "All conditions (AND)" : "Any condition (OR)";
        return '<label><input type="radio" name="' + name + '" value="' + mode + '" data-action="match" data-path="' +
          pathAttr(path) + '"' + (group.match === mode ? " checked" : "") + '> ' +
          ns.escapeHtml(ns.t(key, fallback)) + '</label>';
      }).join("") + '</fieldset>';
  }

  function groupMarkup(group, path, isRoot) {
    var errors = errorsFor(path);
    var children = group.groups.map(function (child, index) {
      var childPath = path.concat(index);
      return ns.isGroup(child) ? groupMarkup(child, childPath, false) : conditionMarkup(child, childPath);
    }).join("");
    var empty = !group.groups.length
      ? '<div class="gb-segment-empty"><strong data-i18n="segments-empty-title">' +
          ns.escapeHtml(ns.t("segments-empty-title", "No conditions yet")) + '</strong><span data-i18n="segments-empty-body">' +
          ns.escapeHtml(ns.t("segments-empty-body", "Add a condition to start building your audience.")) + '</span></div>'
      : "";
    return '<div class="gb-segment-group' + (isRoot ? " gb-segment-root" : " gb-segment-nested") + '" data-group-path="' +
      pathAttr(path) + '">' + matchMarkup(group, path, isRoot) +
      '<div class="gb-segment-children">' + empty + children + '</div>' +
      (errors.length ? '<div class="gb-segment-group-errors">' + errorMarkup(errors) + '</div>' : '') +
      '<div class="gb-segment-group-actions">' +
        '<button type="button" class="btn-secondary" data-action="add-condition" data-path="' + pathAttr(path) + '">' +
          ns.escapeHtml(ns.t("segments-add-condition", "Add condition")) + '</button>' +
        (path.length < ns.MAX_DEPTH - 1 ?
          '<button type="button" class="btn-secondary" data-action="add-group" data-path="' + pathAttr(path) + '">' +
            ns.escapeHtml(ns.t("segments-add-group", "Add nested group")) + '</button>' : '') +
        (isRoot ? "" : '<button type="button" class="btn-secondary" data-action="remove-group" data-path="' +
          pathAttr(path) + '" data-i18n-aria-label="segments-remove-group" aria-label="' +
          ns.escapeHtml(ns.t("segments-remove-group", "Remove group")) + '">' +
          ns.escapeHtml(ns.t("segments-remove-group", "Remove group")) + '</button>') +
      '</div></div>';
  }

  function captureFocus() {
    var active = document.activeElement;
    if (!active || !active.getAttribute) return null;
    var part = active.getAttribute("data-part");
    var path = active.getAttribute("data-path");
    var action = active.getAttribute("data-action");
    return part || action ? { part: part, path: path, action: action } : null;
  }

  function restoreFocus(memento) {
    if (!memento) return;
    var candidates = document.querySelectorAll("#gb-segment-builder [data-part], #gb-segment-builder [data-action]");
    for (var i = 0; i < candidates.length; i++) {
      var el = candidates[i];
      if ((memento.part && el.getAttribute("data-part") === memento.part) ||
          (memento.action && el.getAttribute("data-action") === memento.action)) {
        if (memento.path && el.getAttribute("data-path") !== memento.path) continue;
        if (typeof el.focus === "function") el.focus();
        return;
      }
    }
  }

  ns.renderBuilder = function () {
    var host = ns.byId("gb-segment-builder");
    if (!host) return;
    var memento = captureFocus();
    var root = ns.state.model;
    var matchRadios = host.querySelectorAll('input[name="gb-segment-match"]');
    matchRadios.forEach(function (radio) { radio.checked = radio.value === root.match; });
    var mount = ns.byId("gb-segment-conditions");
    if (mount) mount.innerHTML = groupMarkup(root, [], true);
    restoreFocus(memento);
  };

  ns.renderPreview = function (result) {
    var source = ns.byId("gb-segment-source");
    var count = ns.byId("gb-segment-count-value");
    var suppressed = ns.byId("gb-segment-suppressed");
    var rows = ns.byId("gb-segment-preview-rows");
    var empty = ns.byId("gb-segment-preview-empty");
    var note = ns.byId("gb-segment-sample-note");
    if (!source || !count || !suppressed || !rows || !empty) return;

    var isSample = !result || result.source !== "server";
    source.textContent = ns.previewSourceLabel(result);
    source.className = "gb-segment-source " + (isSample ? "is-sample" : "is-server");
    count.textContent = result ? String(result.matched) : "—";
    suppressed.textContent = ns.t("segments-preview-suppressed",
      "{ $matched } matched, { $suppressed } suppressed",
      { matched: result ? result.matched : 0, suppressed: result ? result.suppressed : 0 });
    ns.clear(rows);
    (result ? result.items : []).forEach(function (item) {
      var tr = document.createElement("tr");
      [item.name, item.email, ns.valueLabel("lifecycle_status", item.status)].forEach(function (text) {
        var td = document.createElement("td");
        td.textContent = text;
        tr.appendChild(td);
      });
      rows.appendChild(tr);
    });
    empty.hidden = !!(result && result.items.length);
    if (note) note.hidden = !isSample;
  };

  ns.setStatus = function (text) {
    var status = ns.byId("gb-segment-status");
    if (status) status.textContent = text || "";
  };

  ns.setErrors = function (errors) {
    ns.state.errors = errors || [];
    ns.renderBuilder();
  };

  ns.buildPreview = function () {
    var model = ns.state.model;
    ns.setStatus(ns.t("segments-preview-loading", "Calculating preview…"));
    ns.requestPreview(model, function (result) {
      ns.renderPreview(result);
      ns.setStatus("");
    });
  };
})(window);
