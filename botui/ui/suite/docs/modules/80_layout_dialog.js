"use strict";
/* Docs layout preview (#2424), shared focus-trapped dialog shell.
 *
 * Escape closes, Tab is trapped and focus is restored to the invoking element.
 * Content is built with createElement + textContent only: no innerHTML for
 * content and no native alert/confirm/prompt. */
(function (window) {
  var NS = window.DocsLayout;
  if (!NS) return;
  var UI = (NS.dialog = NS.dialog || {});

  UI.shell = function (opts) {
    var options = opts || {};
    var host = document.getElementById("docs-modal-container") || document.body;
    var restore = document.activeElement;
    var backdrop = NS.el("div", "gb-layout-backdrop");
    var dialog = NS.el("div", "gb-layout-dialog");
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");
    var titleId = "gb-layout-title-" + Math.random().toString(36).slice(2, 9);
    dialog.setAttribute("aria-labelledby", titleId);

    var heading = NS.i18n(
      NS.el("h3", "gb-layout-dialog-title"),
      options.titleKey || "docs-layout-title", options.titleFallback || "Pages and layout"
    );
    heading.id = titleId;
    dialog.appendChild(heading);

    var note = NS.el("p", "gb-layout-local-note");
    note.textContent = NS.localNote();
    dialog.appendChild(note);

    var body = NS.el("div", "gb-layout-dialog-body");
    dialog.appendChild(body);

    var status = NS.el("div", "gb-layout-status");
    status.id = options.statusId || "docs-layout-status";
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    dialog.appendChild(status);

    var footer = NS.el("div", "gb-layout-dialog-footer");
    var close = NS.i18n(NS.el("button", "gb-layout-btn"), "docs-layout-close", "Close");
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
})(typeof window !== "undefined" ? window : globalThis);
