"use strict";
/* Sheets data connectivity (#2492) - shared i18n-safe UI primitives.
 * window.SheetConnectUi. Every user-facing literal goes through t(key,
 * fallback) and carries the matching data-i18n* attribute for relocalize. */

(function () {
  var C = window.SheetConnectCore;

  function el(tag, className, value) {
    return C.element(tag, className, value);
  }

  function button(key, fallback, onClick, className) {
    var node = el("button", className || "ss-connect-btn", C.t(key, fallback));
    node.type = "button";
    node.setAttribute("data-i18n", key);
    if (typeof onClick === "function") node.addEventListener("click", onClick);
    return node;
  }

  function iconButton(key, fallback, glyph, onClick) {
    var node = button(key, fallback, onClick, "ss-connect-icon-btn");
    node.removeAttribute("data-i18n");
    node.textContent = glyph;
    node.setAttribute("data-i18n-aria-label", key);
    node.setAttribute("aria-label", C.t(key, fallback));
    return node;
  }

  function field(key, fallback, inputType, value, placeholderKey, placeholderFallback) {
    var wrap = el("label", "ss-connect-field");
    var label = el("span", "ss-connect-field-label", C.t(key, fallback));
    label.setAttribute("data-i18n", key);
    var input = el("input", "ss-connect-input");
    input.type = inputType || "text";
    input.value = value == null ? "" : String(value);
    input.dataset.i18nKey = key;
    if (placeholderKey) {
      input.placeholder = C.t(placeholderKey, placeholderFallback || "");
      input.setAttribute("data-i18n-placeholder", placeholderKey);
    }
    wrap.appendChild(label);
    wrap.appendChild(input);
    wrap.input = input;
    return wrap;
  }

  function select(key, fallback, options, value) {
    var wrap = el("label", "ss-connect-field");
    var label = el("span", "ss-connect-field-label", C.t(key, fallback));
    label.setAttribute("data-i18n", key);
    var control = el("select", "ss-connect-input");
    options.forEach(function (option) {
      var item = el("option", "", C.t(option.key, option.fallback));
      item.setAttribute("data-i18n", option.key);
      item.value = option.value;
      control.appendChild(item);
    });
    control.value = value;
    wrap.appendChild(label);
    wrap.appendChild(control);
    wrap.input = control;
    return wrap;
  }

  function checkbox(key, fallback, checked, onChange) {
    var wrap = el("label", "ss-connect-check");
    var input = el("input");
    input.type = "checkbox";
    input.checked = Boolean(checked);
    if (typeof onChange === "function") input.addEventListener("change", onChange);
    var label = el("span", "", C.t(key, fallback));
    label.setAttribute("data-i18n", key);
    wrap.appendChild(input);
    wrap.appendChild(label);
    wrap.input = input;
    return wrap;
  }

  function title(key, fallback) {
    var node = el("h3", "ss-connect-section", C.t(key, fallback));
    node.setAttribute("data-i18n", key);
    return node;
  }

  function note(key, fallback) {
    var node = el("p", "ss-connect-note", C.t(key, fallback));
    node.setAttribute("data-i18n", key);
    return node;
  }

  function table(columns, rows, onCellClick) {
    var root = el("div", "ss-connect-table-wrap");
    if (!columns || !columns.length) return root;
    var grid = el("table", "ss-connect-table");
    var head = el("thead");
    var headRow = el("tr");
    columns.forEach(function (name) { headRow.appendChild(el("th", "", name)); });
    head.appendChild(headRow);
    grid.appendChild(head);
    var body = el("tbody");
    rows.forEach(function (row, rowIndex) {
      var tr = el("tr");
      columns.forEach(function (name, colIndex) {
        var cell = el("td", "", row[colIndex]);
        if (typeof onCellClick === "function") {
          cell.tabIndex = 0;
          cell.setAttribute("role", "button");
          cell.addEventListener("click", function () { onCellClick(rowIndex, colIndex); });
          cell.addEventListener("keydown", function (event) {
            if (event.key !== "Enter" && event.key !== " ") return;
            event.preventDefault();
            onCellClick(rowIndex, colIndex);
          });
        }
        tr.appendChild(cell);
      });
      body.appendChild(tr);
    });
    grid.appendChild(body);
    root.appendChild(grid);
    return root;
  }

  function keyValue(key, fallback, value) {
    var row = el("div", "ss-connect-kv");
    var label = el("span", "ss-connect-kv-label", C.t(key, fallback));
    label.setAttribute("data-i18n", key);
    row.appendChild(label);
    row.appendChild(el("span", "ss-connect-kv-value", value));
    return row;
  }

  function clear(node) {
    if (node) node.textContent = "";
  }

  window.SheetConnectUi = {
    el: el, button: button, iconButton: iconButton, field: field, select: select,
    checkbox: checkbox, title: title, note: note, table: table,
    keyValue: keyValue, clear: clear,
  };
})();
