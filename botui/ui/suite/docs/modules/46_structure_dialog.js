"use strict";
/* Docs long-document structure (#2426), Document Structure dialog shell.
 *
 * One focus-trapped dialog hosts numbering settings, caption insertion,
 * cross-reference insertion, field editing and the update-all action. Every
 * label comes from the docs-structure catalog via setAttribute (never a
 * double-quoted data-i18n expression), Esc closes, focus is restored, and a
 * polite status region announces each edit.
 */
(function (window) {
  var NS = window.DocsStructure;
  if (!NS) return;
  var UI = (NS.ui = NS.ui || {});

  UI.el = function (tag, cls) {
    var node = document.createElement(tag);
    if (cls) node.className = cls;
    return node;
  };

  UI.text = function (parent, key, fallback, tag) {
    var node = UI.el(tag || "span");
    node.setAttribute("data-i18n", key);
    node.textContent = NS.tr(key, fallback);
    parent.appendChild(node);
    return node;
  };

  UI.labeled = function (parent, key, fallback, control) {
    var wrap = UI.el("label", "gb-ds-field");
    UI.text(wrap, key, fallback);
    control.id = control.id || ("gb-ds-" + key);
    wrap.setAttribute("for", control.id);
    wrap.appendChild(control);
    parent.appendChild(wrap);
    return wrap;
  };

  UI.select = function (parent, key, fallback, options, id) {
    var control = UI.el("select");
    if (id) control.id = id;
    (options || []).forEach(function (spec) {
      var option = UI.el("option");
      option.value = spec.value;
      option.setAttribute("data-i18n", spec.key);
      option.textContent = NS.tr(spec.key, spec.fallback);
      control.appendChild(option);
    });
    UI.labeled(parent, key, fallback, control);
    return control;
  };

  UI.button = function (parent, key, fallback, cls, id) {
    var control = UI.el("button", cls || "gb-ds-btn");
    control.setAttribute("type", "button");
    control.id = id || "";
    control.setAttribute("data-i18n", key);
    control.textContent = NS.tr(key, fallback);
    parent.appendChild(control);
    return control;
  };

  UI.section = function (parent, key, fallback) {
    var part = UI.el("section", "gb-ds-section");
    UI.text(part, key, fallback, "h4");
    parent.appendChild(part);
    return part;
  };

  NS.openDialog = function () {
    var host = document.getElementById("docs-modal-container") || document.body;
    if (!host) return null;
    var restore = document.activeElement;
    var backdrop = UI.el("div", "gb-ds-backdrop");
    var dialog = UI.el("div", "gb-ds-dialog");
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");
    var titleId = NS.uid("gb-ds-title");
    dialog.setAttribute("aria-labelledby", titleId);
    var heading = UI.el("h3", "gb-ds-title");
    heading.id = titleId;
    heading.setAttribute("data-i18n", "docs-structure-title");
    heading.textContent = NS.tr("docs-structure-title", "Document structure");
    dialog.appendChild(heading);
    var note = UI.el("p", "gb-ds-note");
    note.setAttribute("data-i18n", "docs-structure-note-local");
    note.textContent = NS.tr("docs-structure-note-local",
      "Stored in this browser only; the document schema does not persist structure settings yet.");
    dialog.appendChild(note);

    var body = UI.el("div", "gb-ds-body");
    if (NS.panels) {
      ["numbering", "caption", "xref", "fields", "actions"].forEach(function (name) {
        if (typeof NS.panels[name] === "function") NS.panels[name](body);
      });
    }
    dialog.appendChild(body);

    var status = UI.el("div", "gb-ds-status");
    status.id = "docs-structure-status";
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    dialog.appendChild(status);
    var actions = UI.el("div", "gb-ds-footer");
    var closeButton = UI.button(actions, "docs-structure-close", "Close", "gb-ds-btn", "gb-ds-close");
    dialog.appendChild(actions);
    backdrop.appendChild(dialog);

    function focusables() {
      var controlSelector = "button,input,select,textarea,[tabindex]:not([tabindex='-1'])";
      var list = dialog.querySelectorAll ? dialog.querySelectorAll(controlSelector) : [];
      return Array.prototype.filter.call(list, function (node) { return !node.disabled; });
    }
    function close() {
      document.removeEventListener("keydown", onKey, true);
      if (backdrop.parentNode) backdrop.parentNode.removeChild(backdrop);
      if (restore && restore.focus) restore.focus();
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

    backdrop.addEventListener("click", function (event) {
      if (event.target === backdrop) close();
    });
    closeButton.addEventListener("click", close);
    document.addEventListener("keydown", onKey, true);
    host.appendChild(backdrop);
    var first = focusables()[0];
    if (first) first.focus();
    return close;
  };
})(window);
