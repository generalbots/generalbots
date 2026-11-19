"use strict";
/* Slides masters (#2429) — layer 3a: accessible dialog shell and DOM helpers.
 * Focus-trapped dialog with Esc close and focus restore, a polite status
 * region, no native alert/confirm/prompt, every label resolved via gbT. */
(function (window) {
  var M = (window.GBSlidesMasters = window.GBSlidesMasters || {});
  var seq = 0;
  var t = M.tr;

  M.el = function (tag, cls, key, fallback) {
    var node = document.createElement(tag);
    if (cls) node.className = cls;
    if (key) {
      node.setAttribute("data-i18n", key);
      node.textContent = t(key, fallback);
    }
    return node;
  };
  M.labelled = function (node, attr, key, fallback) {
    node.setAttribute(attr, key);
    var value = t(key, fallback);
    if (attr === "data-i18n") node.textContent = value;
    else if (attr === "data-i18n-placeholder") node.setAttribute("placeholder", value);
    else if (attr === "data-i18n-title") node.setAttribute("title", value);
    else if (attr === "data-i18n-aria-label") node.setAttribute("aria-label", value);
    return node;
  };
  M.button = function (key, fallback, onClick, variant) {
    var node = M.el("button", "sm-btn" + (variant ? " sm-btn-" + variant : ""), key, fallback);
    node.type = "button";
    if (typeof onClick === "function") node.addEventListener("click", onClick);
    return node;
  };
  M.textField = function (key, fallback, value, placeholderKey, placeholderFallback) {
    var wrap = document.createElement("label");
    wrap.className = "sm-field";
    M.labelled(wrap, "data-i18n", key, fallback);
    var input = document.createElement("input");
    input.type = "text";
    input.value = value === undefined || value === null ? "" : String(value);
    if (placeholderKey) M.labelled(input, "data-i18n-placeholder", placeholderKey, placeholderFallback);
    wrap.appendChild(input);
    wrap.control = input;
    return wrap;
  };
  M.selectField = function (key, fallback, options, selected) {
    var wrap = document.createElement("label");
    wrap.className = "sm-field";
    M.labelled(wrap, "data-i18n", key, fallback);
    var select = document.createElement("select");
    options.forEach(function (opt) {
      var o = document.createElement("option");
      o.value = opt.value;
      o.textContent = opt.label;
      if (opt.value === selected) o.selected = true;
      select.appendChild(o);
    });
    wrap.appendChild(select);
    wrap.control = select;
    return wrap;
  };
  M.localNote = function () {
    return M.el("p", "sm-local-note", "slides-masters-local-only", "Local only — not saved to the server");
  };
  M.section = function (titleKey, titleFallback) {
    var box = document.createElement("section");
    box.className = "sm-section";
    box.appendChild(M.el("h3", "sm-section-title", titleKey, titleFallback));
    return box;
  };
  M.status = function (api, key, fallback, params) {
    api.setStatus(t(key, fallback, params));
  };

  function focusables(root) {
    return Array.prototype.slice.call(root.querySelectorAll(
      "button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex=\"-1\"])"
    ));
  }

  M.openDialog = function (options) {
    options = options || {};
    var previous = document.activeElement;
    var overlay = document.createElement("div");
    overlay.className = "sm-overlay";
    var dialog = document.createElement("div");
    dialog.className = "sm-dialog";
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");
    var titleId = "sm-title-" + (++seq);
    dialog.setAttribute("aria-labelledby", titleId);

    var head = document.createElement("div");
    head.className = "sm-dialog-head";
    var title = M.el("h2", "sm-dialog-title", "slides-masters-title", "Slide masters & brand kit");
    title.id = titleId;
    head.appendChild(title);
    var close = document.createElement("button");
    close.type = "button";
    close.className = "sm-dialog-close";
    close.textContent = "\u00d7";
    M.labelled(close, "data-i18n-aria-label", "slides-masters-close", "Close");
    head.appendChild(close);
    dialog.appendChild(head);

    var body = document.createElement("div");
    body.className = "sm-dialog-body";
    dialog.appendChild(body);
    var status = document.createElement("div");
    status.className = "sm-dialog-status";
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    status.textContent = t("slides-masters-status-ready", "Ready");
    dialog.appendChild(status);

    var closed = false;
    var api = {
      root: dialog,
      body: body,
      setStatus: function (text) { status.textContent = text; return api; },
      rebuild: function () {
        body.innerHTML = "";
        if (typeof options.onBuild === "function") options.onBuild(api);
        var first = focusables(dialog);
        if (first.length) first[0].focus();
      },
      addFooter: function () {},
      close: function () {
        if (closed) return;
        closed = true;
        document.removeEventListener("keydown", onKey, true);
        if (overlay.parentNode) overlay.parentNode.removeChild(overlay);
        if (previous && typeof previous.focus === "function") previous.focus();
      }
    };
    close.addEventListener("click", function () { api.close(); });

    function onKey(event) {
      if (closed) return;
      if (event.key === "Escape") {
        event.preventDefault(); event.stopPropagation(); api.close();
      } else if (event.key === "Tab") {
        var items = focusables(dialog);
        if (!items.length) { event.preventDefault(); return; }
        var first = items[0], last = items[items.length - 1];
        if (event.shiftKey && document.activeElement === first) {
          event.preventDefault(); last.focus();
        } else if (!event.shiftKey && document.activeElement === last) {
          event.preventDefault(); first.focus();
        }
      }
    }
    overlay.appendChild(dialog);
    document.body.appendChild(overlay);
    document.addEventListener("keydown", onKey, true);
    api.rebuild();
    return api;
  };
})(typeof window !== "undefined" ? window : globalThis);
