"use strict";
/* One reusable focus-trapped dialog shell for all forms surfaces (#2427).
 * Esc closes, Tab wraps in both directions, focus returns to the opener and a
 * role=status / aria-live=polite region announces local results. */
(function (window) {
  var NS = window.DocsForms;
  if (!NS) return;
  var D = (NS.dialog = NS.dialog || {});
  var FOCUS = "button,input,select,textarea,[tabindex]:not([tabindex='-1'])";

  function el(tag, cls) {
    var node = document.createElement(tag);
    if (cls) node.className = cls;
    return node;
  }
  D.el = el;

  function i18n(node, key, fallback) {
    node.setAttribute("data-i18n", key);
    node.textContent = NS.tr(key, fallback);
    return node;
  }
  D.i18n = i18n;

  D.button = function (key, fallback, primary) {
    var button = el("button", primary ? "gb-forms-btn gb-forms-btn-primary" : "gb-forms-btn");
    button.setAttribute("type", "button");
    i18n(button, key, fallback);
    return button;
  };

  D.field = function (key, fallback, input) {
    var label = el("label", "gb-forms-field");
    i18n(label, key, fallback);
    label.appendChild(input);
    return label;
  };

  D.input = function (tag, key, fallback, value) {
    var input = el(tag, "gb-forms-input");
    if (tag === "textarea") {
      input.setAttribute("data-i18n-placeholder", key);
      input.placeholder = NS.tr(key, fallback);
    } else {
      input.setAttribute("type", "text");
      input.setAttribute("data-i18n-placeholder", key);
      input.placeholder = NS.tr(key, fallback);
    }
    if (value != null) input.value = value;
    return input;
  };

  D.shell = function (opts) {
    var options = opts || {};
    var host = document.getElementById("docs-modal-container") || document.body;
    var restore = document.activeElement;
    var backdrop = el("div", "gb-forms-backdrop");
    var dialog = el("div", "gb-forms-dialog");
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");
    var titleId = NS.uid("gb-forms-title");
    dialog.setAttribute("aria-labelledby", titleId);
    var heading = el("h3", "gb-forms-dialog-title");
    heading.id = titleId;
    i18n(heading, options.titleKey || "docs-forms-template-title", options.titleFallback || "Templates");
    dialog.appendChild(heading);
    var local = el("p", "gb-forms-local-note");
    local.textContent = NS.localNote();
    dialog.appendChild(local);
    if (options.preview) {
      var preview = el("p", "gb-forms-preview-note");
      preview.textContent = NS.previewNote();
      dialog.appendChild(preview);
    }
    var body = el("div", "gb-forms-dialog-body");
    dialog.appendChild(body);
    var status = el("div", "gb-forms-status");
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    status.id = "docs-forms-dialog-status";
    dialog.appendChild(status);
    var footer = el("div", "gb-forms-dialog-footer");
    var close = D.button("docs-forms-close", "Close");
    footer.appendChild(close);
    dialog.appendChild(footer);
    backdrop.appendChild(dialog);

    function focusables() {
      var list = dialog.querySelectorAll ? dialog.querySelectorAll(FOCUS) : [];
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
    return { body: body, status: status, close: closeDialog, root: dialog, backdrop: backdrop };
  };
})(window);
