"use strict";
/* Slides analytics (#2489) — layer 6, accessible DOM helpers and modal shell.
 * Focus-trapped dialog with Esc close and focus restore, a polite status
 * region and no alert/confirm/prompt. Content is never built with innerHTML. */
(function (window) {
  var A = window.GBSlidesAnalytics = window.GBSlidesAnalytics || {};
  var sequence = 0;

  A.labelled = function (node, attr, key, fallback) {
    node.setAttribute(attr, key);
    var value = A.tr(key, fallback);
    if (attr === "data-i18n") node.textContent = value;
    else if (attr === "data-i18n-placeholder") node.setAttribute("placeholder", value);
    else if (attr === "data-i18n-title") node.setAttribute("title", value);
    else if (attr === "data-i18n-aria-label") node.setAttribute("aria-label", value);
    return node;
  };

  A.el = function (tag, cls, key, fallback) {
    var node = document.createElement(tag);
    if (cls) node.className = cls;
    if (key) A.labelled(node, "data-i18n", key, fallback);
    return node;
  };

  A.button = function (key, fallback, onClick, variant) {
    var node = A.el("button", "sa-btn" + (variant ? " sa-btn-" + variant : ""), key, fallback);
    node.type = "button";
    if (typeof onClick === "function") node.addEventListener("click", onClick);
    return node;
  };

  A.text = function (cls, value) {
    var node = document.createElement("span");
    if (cls) node.className = cls;
    node.textContent = value === null || value === undefined ? "" : String(value);
    return node;
  };

  A.field = function (key, fallback, value, placeholderKey, placeholderFallback) {
    var label = A.el("label", "sa-field", key, fallback);
    var input = document.createElement("input");
    input.type = "text";
    input.value = value === undefined || value === null ? "" : String(value);
    if (placeholderKey) A.labelled(input, "data-i18n-placeholder", placeholderKey, placeholderFallback);
    label.appendChild(input);
    label.control = input;
    return label;
  };

  A.select = function (key, fallback, entries, selected) {
    var label = A.el("label", "sa-field", key, fallback);
    var control = document.createElement("select");
    entries.forEach(function (entry) {
      var option = document.createElement("option");
      option.value = entry[0];
      option.textContent = A.tr(entry[1], entry[2]);
      if (entry[0] === selected) option.selected = true;
      control.appendChild(option);
    });
    label.appendChild(control);
    label.control = control;
    return label;
  };

  A.localNote = function () {
    return A.el("p", "sa-local-note", "slides-analytics-local-only",
      "Local only — not saved to the server");
  };

  A.section = function (key, fallback) {
    var wrap = document.createElement("section");
    wrap.className = "sa-section";
    wrap.appendChild(A.el("h3", "sa-subhead", key, fallback));
    return wrap;
  };

  A.metric = function (key, fallback, value) {
    var row = document.createElement("div");
    row.className = "sa-metric";
    row.appendChild(A.el("span", "sa-metric-key", key, fallback));
    row.appendChild(A.text("sa-metric-value", value));
    return row;
  };

  A.listRow = function (cells) {
    var row = document.createElement("div");
    row.className = "sa-list-row";
    cells.forEach(function (cell) {
      row.appendChild(typeof cell === "string" ? A.text("sa-list-cell", cell) : cell);
    });
    return row;
  };

  function focusables(root) {
    return Array.prototype.slice.call(root.querySelectorAll(
      "button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex=\"-1\"])"
    ));
  }

  A.openDialog = function (options) {
    options = options || {};
    var previous = document.activeElement;
    var overlay = document.createElement("div");
    overlay.className = "sa-overlay";
    var dialog = document.createElement("div");
    dialog.className = "sa-dialog";
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");
    var titleId = "sa-title-" + (++sequence);
    dialog.setAttribute("aria-labelledby", titleId);

    var header = document.createElement("div");
    header.className = "sa-dialog-head";
    var title = A.el("h2", "sa-dialog-title", options.titleKey || "slides-analytics-open",
      options.titleFallback || "Audience and analytics");
    title.id = titleId;
    header.appendChild(title);
    var close = A.el("button", "sa-dialog-close", "slides-analytics-close", "Close");
    close.type = "button";
    close.setAttribute("data-i18n-aria-label", "slides-analytics-close");
    close.textContent = "\u00d7";
    close.addEventListener("click", function () { api.close(); });
    header.appendChild(close);
    dialog.appendChild(header);

    var body = document.createElement("div");
    body.className = "sa-dialog-body";
    dialog.appendChild(body);
    var status = document.createElement("div");
    status.className = "sa-dialog-status";
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    dialog.appendChild(status);
    var footer = document.createElement("div");
    footer.className = "sa-dialog-foot";
    dialog.appendChild(footer);

    var closed = false;
    var api = {
      root: dialog,
      body: body,
      footer: footer,
      status: status,
      setStatus: function (key, fallback, params) {
        status.textContent = A.tr(key, fallback, params);
        return api;
      },
      addButton: function (key, fallback, onClick, variant) {
        var button = A.button(key, fallback, onClick, variant);
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
