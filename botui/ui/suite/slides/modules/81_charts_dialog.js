"use strict";
/* Slides chart editor shell (#2486) — layer 5: accessible DOM helpers and a
 * focus-trapped modal (Esc close, focus restore, polite status region). Content
 * is built with createElement only; no innerHTML, no alert/confirm/prompt. */
(function (window) {
  var C = (window.GBSlidesCharts = window.GBSlidesCharts || {});
  var U = (C.ui = C.ui || {});
  var sequence = 0;

  U.labelled = function (node, attr, key, fallback) {
    node.setAttribute(attr, key);
    var value = C.tr(key, fallback);
    if (attr === "data-i18n") node.textContent = value;
    else if (attr === "data-i18n-placeholder") node.setAttribute("placeholder", value);
    else if (attr === "data-i18n-title") node.setAttribute("title", value);
    else if (attr === "data-i18n-aria-label") node.setAttribute("aria-label", value);
    return node;
  };

  U.el = function (tag, cls, key, fallback) {
    var node = document.createElement(tag);
    if (cls) node.className = cls;
    if (key) U.labelled(node, "data-i18n", key, fallback);
    return node;
  };

  U.button = function (key, fallback, onClick, variant) {
    var node = U.el("button", "gbch-btn" + (variant ? " gbch-btn-" + variant : ""), key, fallback);
    node.type = "button";
    if (typeof onClick === "function") node.addEventListener("click", onClick);
    return node;
  };

  U.text = function (cls, value) {
    var node = document.createElement("span");
    if (cls) node.className = cls;
    node.textContent = value === null || value === undefined ? "" : String(value);
    return node;
  };

  U.field = function (key, fallback, value, placeholderKey, placeholderFallback) {
    var label = U.el("label", "gbch-field", key, fallback);
    var input = document.createElement("input");
    input.type = "text";
    input.value = value === undefined || value === null ? "" : String(value);
    if (placeholderKey) U.labelled(input, "data-i18n-placeholder", placeholderKey, placeholderFallback);
    label.appendChild(input);
    label.control = input;
    return label;
  };

  U.checkbox = function (key, fallback, checked) {
    var label = U.el("label", "gbch-check", key, fallback);
    var input = document.createElement("input");
    input.type = "checkbox";
    input.checked = !!checked;
    label.appendChild(input);
    label.control = input;
    return label;
  };

  U.select = function (key, fallback, entries, selected) {
    var label = U.el("label", "gbch-field", key, fallback);
    var control = document.createElement("select");
    entries.forEach(function (entry) {
      var option = document.createElement("option");
      option.value = entry[0];
      option.textContent = C.tr(entry[1], entry[2]);
      if (entry[0] === selected) option.selected = true;
      control.appendChild(option);
    });
    label.appendChild(control);
    label.control = control;
    return label;
  };

  U.localNote = function () {
    return U.el("p", "gbch-local-note", "slides-charts-local-only",
      "Local only — not saved to the server");
  };

  U.deferredNote = function () {
    return U.el("p", "gbch-deferred-note", "slides-charts-deferred",
      "Native PPTX chart parts, embedded workbooks, vector PDF and PowerPoint round-trip are not available in this local preview.");
  };

  U.section = function (key, fallback) {
    var wrap = document.createElement("section");
    wrap.className = "gbch-section";
    wrap.appendChild(U.el("h3", "gbch-subhead", key, fallback));
    return wrap;
  };

  U.badge = function (key, fallback, variant) {
    return U.el("span", "gbch-badge" + (variant ? " gbch-badge-" + variant : ""), key, fallback);
  };

  function focusables(root) {
    return Array.prototype.slice.call(root.querySelectorAll(
      "button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex=\"-1\"])"
    ));
  }

  U.openDialog = function (options) {
    options = options || {};
    var previous = document.activeElement;
    var overlay = document.createElement("div");
    overlay.className = "gbch-overlay";
    var dialog = document.createElement("div");
    dialog.className = "gbch-dialog";
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");
    var titleId = "gbch-title-" + (++sequence);
    dialog.setAttribute("aria-labelledby", titleId);

    var header = document.createElement("div");
    header.className = "gbch-dialog-head";
    var title = U.el("h2", "gbch-dialog-title", options.titleKey || "slides-charts-title",
      options.titleFallback || "Chart editor (local)");
    title.id = titleId;
    header.appendChild(title);
    var close = document.createElement("button");
    close.type = "button";
    close.className = "gbch-dialog-close";
    close.textContent = "\u00d7";
    U.labelled(close, "data-i18n-aria-label", "slides-charts-close", "Close");
    close.addEventListener("click", function () { api.close(); });
    header.appendChild(close);
    dialog.appendChild(header);

    var body = document.createElement("div");
    body.className = "gbch-dialog-body";
    dialog.appendChild(body);
    var status = document.createElement("div");
    status.className = "gbch-dialog-status";
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    dialog.appendChild(status);
    var footer = document.createElement("div");
    footer.className = "gbch-dialog-foot";
    dialog.appendChild(footer);

    var closed = false;
    var api = {
      root: dialog,
      body: body,
      footer: footer,
      status: status,
      setStatus: function (key, fallback, params) {
        status.textContent = C.tr(key, fallback, params);
        return api;
      },
      addButton: function (key, fallback, onClick, variant) {
        var button = U.button(key, fallback, onClick, variant);
        footer.appendChild(button);
        return button;
      },
      close: function () {
        if (closed) return;
        closed = true;
        document.removeEventListener("keydown", onKeyDown, true);
        if (overlay.parentNode) overlay.parentNode.removeChild(overlay);
        if (previous && typeof previous.focus === "function") previous.focus();
      }
    };

    function onKeyDown(event) {
      if (closed) return;
      if (event.key === "Escape") {
        event.preventDefault();
        event.stopPropagation();
        api.close();
      } else if (event.key === "Tab") {
        var items = focusables(dialog);
        if (!items.length) { event.preventDefault(); return; }
        var first = items[0];
        var last = items[items.length - 1];
        if (event.shiftKey && document.activeElement === first) {
          event.preventDefault();
          last.focus();
        } else if (!event.shiftKey && document.activeElement === last) {
          event.preventDefault();
          first.focus();
        }
      }
    }

    overlay.appendChild(dialog);
    document.body.appendChild(overlay);
    document.addEventListener("keydown", onKeyDown, true);
    if (typeof options.onBuild === "function") options.onBuild(api);
    var initial = focusables(dialog);
    if (initial.length) initial[0].focus();
    else { dialog.setAttribute("tabindex", "-1"); dialog.focus(); }
    return api;
  };
})(typeof window !== "undefined" ? window : globalThis);
