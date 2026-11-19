"use strict";
/* Docs rich table UI (#2482): contextual toolbar, accessible dialogs and a
 * polite status region. Labels are translated through DocsTables.tr and also
 * tagged with data-i18n* so the suite locale switcher keeps them in sync. */
(function (window) {
  var NS = (window.DocsTables = window.DocsTables || {});
  var UI = (window.DocsTablesUI = window.DocsTablesUI || {});
  var toolbar = null;
  var statusNode = null;

  var BUTTONS = [
    ["structure", "insertRowAbove", "docs-tables-insert-row-above", "Insert row above", "M3 8h18M12 3v10"],
    ["structure", "insertRowBelow", "docs-tables-insert-row-below", "Insert row below", "M3 16h18M12 21V11"],
    ["structure", "deleteRow", "docs-tables-delete-row", "Delete row", "M3 12h18M8 6l8 12M16 6L8 18"],
    ["columns", "insertColLeft", "docs-tables-insert-col-left", "Insert column to the left", "M8 3v18M3 12h10"],
    ["columns", "insertColRight", "docs-tables-insert-col-right", "Insert column to the right", "M16 3v18M11 12h10"],
    ["columns", "deleteCol", "docs-tables-delete-col", "Delete column", "M12 3v18M6 8l12 8M18 8L6 16"],
    ["cells", "merge", "docs-tables-merge", "Merge cells", "M3 4h8v16H3zM13 4h8v16h-8zM11 12h2"],
    ["cells", "split", "docs-tables-split", "Split cells", "M12 3v18M3 12h18"],
    ["look", "header", "docs-tables-header-row", "Toggle header row", "M3 4h18v6H3zM3 12h18v8H3z"],
    ["look", "banded", "docs-tables-banded-rows", "Toggle banded rows", "M3 4h18v5H3zM3 10h18v5H3zM3 16h18v5H3z"],
    ["look", "borders", "docs-tables-borders", "Toggle borders", "M3 3h18v18H3zM3 12h18M12 3v18"],
    ["look", "shading", "docs-tables-shading", "Toggle cell shading", "M4 4h16v16H4zM4 12h16"],
    ["padding", "padInc", "docs-tables-padding-inc", "Increase cell padding", "M4 4h16v16H4zM8 8h8v8H8z"],
    ["padding", "padDec", "docs-tables-padding-dec", "Decrease cell padding", "M4 4h16v16H4zM10 10h4v4h-4z"],
    ["padding", "valignTop", "docs-tables-valign-top", "Align cell content to top", "M4 4h16v16H4zM8 8h8"],
    ["padding", "valignMiddle", "docs-tables-valign-middle", "Align cell content to middle", "M4 4h16v16H4zM8 12h8"],
    ["padding", "valignBottom", "docs-tables-valign-bottom", "Align cell content to bottom", "M4 4h16v16H4zM8 16h8"],
    ["size", "colWidth", "docs-tables-col-width", "Set column width", "M4 4h8v16H4zM14 9h6M14 15h6"],
    ["size", "autofit", "docs-tables-autofit", "Autofit columns", "M3 6h18M3 12h12M3 18h16"],
    ["size", "sortAsc", "docs-tables-sort-asc", "Sort column A to Z", "M7 4v16M4 7l3-3 3 3M14 6h6M14 12h4M14 18h2"],
    ["size", "sortDesc", "docs-tables-sort-desc", "Sort column Z to A", "M7 4v16M4 17l3 3 3-3M14 6h2M14 12h4M14 18h6"],
    ["convert", "textToTable", "docs-tables-text-to-table", "Convert text to table", "M3 5h8v14H3zM13 5h8v14h-8zM3 12h18"],
    ["convert", "tableToText", "docs-tables-table-to-text", "Convert table to text", "M3 5h18v14H3zM6 9h12M6 13h12M6 17h8"],
    ["convert", "styles", "docs-tables-styles", "Table styles", "M12 3l3 5 6 1-4 4 1 6-6-3-6 3 1-6-4-4 6-1z"],
    ["convert", "a11y", "docs-tables-a11y-check", "Check table accessibility", "M12 3l9 16H3zM12 9v5M12 17h.01"],
  ];

  function el(tag, className) {
    var node = document.createElement(tag);
    if (className) node.setAttribute("class", className);
    return node;
  }
  function icon(path) {
    var svg = el("svg");
    svg.setAttribute("width", "15");
    svg.setAttribute("height", "15");
    svg.setAttribute("viewBox", "0 0 24 24");
    svg.setAttribute("fill", "none");
    svg.setAttribute("stroke", "currentColor");
    svg.setAttribute("stroke-width", "1.8");
    svg.setAttribute("aria-hidden", "true");
    svg.innerHTML = '<path d="' + path + '"/>';
    return svg;
  }
  function label(node, key, fallback) {
    var value = NS.tr(key, fallback);
    node.setAttribute("data-i18n-title", key);
    node.setAttribute("data-i18n-aria-label", key);
    node.setAttribute("title", value);
    node.setAttribute("aria-label", value);
    return node;
  }
  function button(action, key, fallback, path, primary) {
    var node = el("button", primary ? "gb-dtbl-primary" : "");
    node.setAttribute("type", "button");
    node.setAttribute("data-docs-table-action", action);
    label(node, key, fallback);
    node.appendChild(icon(path));
    return node;
  }

  function build() {
    if (toolbar) return toolbar;
    toolbar = el("div");
    toolbar.id = "gb-docs-tbl-toolbar";
    toolbar.setAttribute("role", "toolbar");
    label(toolbar, "docs-tables-toolbar", "Table tools");
    var group = null;
    var last = "";
    BUTTONS.forEach(function (spec) {
      if (spec[0] !== last) {
        group = el("div", "gb-dtbl-group");
        toolbar.appendChild(group);
        last = spec[0];
      }
      group.appendChild(button(spec[1], spec[2], spec[3], spec[4]));
    });
    statusNode = el("div", "gb-dtbl-status");
    statusNode.setAttribute("role", "status");
    statusNode.setAttribute("aria-live", "polite");
    toolbar.appendChild(statusNode);
    (document.body || document.documentElement).appendChild(toolbar);
    toolbar.addEventListener("click", onClick);
    return toolbar;
  }

  UI.setStatus = function (message) {
    if (!statusNode) return;
    statusNode.textContent = message || "";
  };

  UI.init = function () {
    build();
    if (document.addEventListener) {
      document.addEventListener("selectionchange", function () { UI.refresh(); });
      window.addEventListener("localeChanged", function () { relabel(); });
    }
    UI.refresh();
  };

  function relabel() {
    if (!toolbar) return;
    BUTTONS.forEach(function (spec) {
      var node = toolbar.querySelector('[data-docs-table-action="' + spec[1] + '"]');
      if (node) label(node, spec[2], spec[3]);
    });
    label(toolbar, "docs-tables-toolbar", "Table tools");
  }

  UI.refresh = function (table) {
    if (!toolbar) return;
    table = table || NS.activeTable();
    if (!table) {
      toolbar.classList.remove("gb-dtbl-open");
      UI.setStatus("");
      return;
    }
    toolbar.classList.add("gb-dtbl-open");
    UI.setStatus(NS.tr("docs-tables-toolbar-hint", "Editing table — select cells or place the cursor"));
    setPressed("banded", (table.getAttribute("class") || "").indexOf("docs-tbl-banded") >= 0);
    setPressed("borders", (table.getAttribute("class") || "").indexOf("docs-tbl-noborders") < 0);
    setPressed("header", !!table.tHead);
    position(table);
  };

  function setPressed(action, pressed) {
    var node = toolbar.querySelector('[data-docs-table-action="' + action + '"]');
    if (node) node.setAttribute("aria-pressed", pressed ? "true" : "false");
  }

  function position(table) {
    if (!table.getBoundingClientRect || !toolbar.offsetWidth) return;
    try {
      var rect = table.getBoundingClientRect();
      var width = toolbar.offsetWidth || 600;
      var left = Math.max(8, Math.min(rect.left, (window.innerWidth || 1200) - width - 8));
      var top = rect.top > 60 ? rect.top - 42 : rect.bottom + 6;
      toolbar.style.left = left + "px";
      toolbar.style.top = top + "px";
    } catch (error) { /* offscreen or detached table */ }
  }

  function onClick(event) {
    var target = event.target;
    while (target && target !== toolbar && !target.getAttribute) target = target.parentNode;
    var node = target && target.getAttribute ? target.closest("[data-docs-table-action]") : null;
    if (!node || !toolbar.contains(node)) return;
    dispatch(node.getAttribute("data-docs-table-action"));
  }

  function dispatch(action) {
    var ops = NS.ops || {};
    switch (action) {
      case "insertRowAbove": ops.insertRow && ops.insertRow("above"); break;
      case "insertRowBelow": ops.insertRow && ops.insertRow("below"); break;
      case "deleteRow": ops.deleteRow && ops.deleteRow(); break;
      case "insertColLeft": ops.insertCol && ops.insertCol("left"); break;
      case "insertColRight": ops.insertCol && ops.insertCol("right"); break;
      case "deleteCol": ops.deleteCol && ops.deleteCol(); break;
      case "merge": ops.merge && ops.merge(); break;
      case "split": ops.split && ops.split(); break;
      case "header": ops.toggleHeader && ops.toggleHeader(); break;
      case "banded": ops.toggleBanded && ops.toggleBanded(); break;
      case "borders": ops.toggleBorders && ops.toggleBorders(); break;
      case "shading": ops.toggleShading && ops.toggleShading(); break;
      case "padInc": ops.cellPadding && ops.cellPadding(1); break;
      case "padDec": ops.cellPadding && ops.cellPadding(-1); break;
      case "valignTop": ops.vAlign && ops.vAlign("top"); break;
      case "valignMiddle": ops.vAlign && ops.vAlign("middle"); break;
      case "valignBottom": ops.vAlign && ops.vAlign("bottom"); break;
      case "autofit": ops.autofit && ops.autofit(); break;
      case "sortAsc": ops.sort && ops.sort(false); break;
      case "sortDesc": ops.sort && ops.sort(true); break;
      case "tableToText": ops.tableToText && ops.tableToText(); break;
      case "colWidth": UI.openWidthDialog(); break;
      case "styles": UI.openStylePicker(); break;
      case "textToTable": UI.openTextToTable(); break;
      case "a11y": if (window.DocsTablesA11y) window.DocsTablesA11y.open(); break;
      default: break;
    }
    UI.refresh();
  }
  UI.dispatch = dispatch;

  UI.toggle = function () {
    if (!toolbar) return;
    var open = toolbar.classList.toggle("gb-dtbl-open");
    if (open) UI.refresh();
  };

  /* Shared by 35b_tables_dialogs.js. */
  UI.el = el;
  UI.makeButton = button;
  UI.label = label;
  UI.shortcutHint = function () {
    return NS.tr("docs-tables-shortcut", "Press Ctrl+Alt+T to open table tools");
  };
})(window);
