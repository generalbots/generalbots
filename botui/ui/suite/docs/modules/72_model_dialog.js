"use strict";
/* Docs structured document model (#2423), focus-trapped inspector dialog. */
(function (window) {
  var NS = window.DocsModel;
  if (!NS) return;
  var UI = (NS.dialog = NS.dialog || {});

  function el(tag, cls) {
    var node = document.createElement(tag);
    if (cls) node.className = cls;
    return node;
  }
  function i18n(node, key, fallback) {
    node.setAttribute("data-i18n", key);
    node.textContent = NS.tr(key, fallback);
    return node;
  }
  NS.el = el;
  NS.i18n = i18n;

  UI.shell = function (opts) {
    var options = opts || {};
    var host = document.getElementById("docs-modal-container") || document.body;
    var restore = document.activeElement;
    var backdrop = el("div", "gb-model-backdrop");
    var dialog = el("div", "gb-model-dialog");
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");
    var titleId = NS.uid("gb-model-title");
    dialog.setAttribute("aria-labelledby", titleId);
    var heading = i18n(el("h3", "gb-model-dialog-title"), "docs-model-title", "Structured document model");
    heading.id = titleId;
    dialog.appendChild(heading);
    var note = el("p", "gb-model-local-note");
    note.textContent = NS.localNote();
    dialog.appendChild(note);
    var body = el("div", "gb-model-dialog-body");
    dialog.appendChild(body);
    var status = el("div", "gb-model-status");
    status.id = "docs-model-status";
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    dialog.appendChild(status);
    var footer = el("div", "gb-model-dialog-footer");
    var close = i18n(el("button", "gb-model-btn"), "docs-model-close", "Close");
    close.setAttribute("type", "button");
    footer.appendChild(close);
    dialog.appendChild(footer);
    backdrop.appendChild(dialog);

    var selector = "button,input,select,textarea,[tabindex]:not([tabindex='-1'])";
    function focusables() {
      var list = dialog.querySelectorAll ? dialog.querySelectorAll(selector) : [];
      return Array.prototype.filter.call(list, function (node) { return !node.disabled; });
    }
    function closeDialog() {
      document.removeEventListener("keydown", onKey, true);
      if (backdrop.parentNode) backdrop.parentNode.removeChild(backdrop);
      if (restore && restore.focus) restore.focus();
    }
    function onKey(event) {
      var list = focusables();
      var action = NS.focusMove(list, document.activeElement, event);
      if (action.action === "close") { event.preventDefault(); closeDialog(); return; }
      if (action.action === "focus") { event.preventDefault(); list[action.index].focus(); }
    }
    close.addEventListener("click", closeDialog);
    backdrop.addEventListener("click", function (event) {
      if (event.target === backdrop) closeDialog();
    });
    document.addEventListener("keydown", onKey, true);
    host.appendChild(backdrop);
    var first = focusables()[0];
    if (first) first.focus();
    void options;
    return { body: body, status: status, close: closeDialog, root: dialog };
  };

  UI.open = function () {
    var shell = UI.shell();
    if (NS.renderInspector) NS.renderInspector(shell.body);
    return shell.close;
  };
})(typeof window !== "undefined" ? window : globalThis);
