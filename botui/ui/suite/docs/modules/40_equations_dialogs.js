"use strict";
/* Docs equation editor (#2483) — accessible dialogs.
 *
 * One focus-trapped modal primitive serves the editor, the cross-reference
 * picker and the equation list. Every visible label comes from the
 * docs-equations catalogue through DocsEquations.tr(); dynamic data-i18n
 * attributes are set with setAttribute (never double-quoted literals). */
(function (window) {
  var NS = (window.DocsEquations = window.DocsEquations || {});
  function tr(key, fallback) { return NS.tr(key, fallback); }
  function el(tag, className) {
    var node = document.createElement(tag);
    if (className) node.className = className;
    return node;
  }
  function label(node, key, fallback) {
    node.setAttribute("data-i18n", key);
    node.textContent = tr(key, fallback);
    return node;
  }
  function button(key, fallback, className) {
    var node = el("button", className || "gb-deq-btn");
    node.setAttribute("type", "button");
    label(node, key, fallback);
    return node;
  }
  function field(node, key, fallback) {
    node.setAttribute("data-i18n-placeholder", key);
    node.setAttribute("placeholder", tr(key, fallback));
    return node;
  }
  function setStatus(host, message) {
    if (host) host.textContent = message || "";
  }
  NS.setStatus = setStatus;
  var activeStatus = null;
  NS.announce = function (message) {
    if (activeStatus) setStatus(activeStatus, message);
  };

  function openDialog(config) {
    var host = document.getElementById("docs-modal-container") || document.body;
    var restore = document.activeElement;
    var backdrop = el("div", "gb-deq-backdrop");
    var dialog = el("div", "gb-deq-dialog");
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");
    var titleId = NS.uid() + "-title";
    dialog.setAttribute("aria-labelledby", titleId);
    var heading = label(el("h3", "gb-deq-title"), config.titleKey, config.titleFallback);
    heading.id = titleId;
    dialog.appendChild(heading);
    var body = el("div", "gb-deq-body");
    if (typeof config.render === "function") config.render(body);
    dialog.appendChild(body);
    var status = el("div", "gb-deq-status");
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    dialog.appendChild(status);
    var actions = el("div", "gb-deq-actions");
    if (Array.isArray(config.actions)) {
      config.actions.forEach(function (spec) {
        var node = button(spec.key, spec.fallback, "gb-deq-btn " + (spec.primary ? "gb-deq-primary" : ""));
        node.addEventListener("click", function () {
          if (typeof spec.onClick === "function") spec.onClick(body, status, close);
        });
        actions.appendChild(node);
      });
    }
    dialog.appendChild(actions);
    backdrop.appendChild(dialog);

    function close() {
      document.removeEventListener("keydown", onKey, true);
      if (activeStatus === status) activeStatus = null;
      if (backdrop.parentNode) backdrop.parentNode.removeChild(backdrop);
      if (restore && restore.focus) restore.focus();
    }
    function focusables() {
      return Array.prototype.filter.call(
        dialog.querySelectorAll("button,input,textarea,select,[tabindex]:not([tabindex='-1'])"),
        function (node) { return !node.disabled; }
      );
    }
    function onKey(event) {
      if (event.key === "Escape") { event.preventDefault(); close(); return; }
      if (event.key !== "Tab") return;
      var list = focusables();
      if (!list.length) return;
      var index = list.indexOf(document.activeElement);
      var next = event.shiftKey ? (index - 1 + list.length) % list.length : (index + 1) % list.length;
      event.preventDefault();
      list[next].focus();
    }
    backdrop.addEventListener("click", function (event) { if (event.target === backdrop) close(); });
    document.addEventListener("keydown", onKey, true);
    host.appendChild(backdrop);
    activeStatus = status;
    var first = focusables()[0];
    if (first) first.focus();
    return { close: close, body: body, status: status };
  }
  NS.openDialog = openDialog;

  function savedCaret() {
    var a = NS.getArticle && NS.getArticle();
    var sel = window.getSelection ? window.getSelection() : null;
    if (!a || !sel || !sel.rangeCount) return null;
    var range = sel.getRangeAt(0);
    var node = range.startContainer;
    while (node && node !== a) node = node.parentNode;
    return node === a ? range.cloneRange() : null;
  }
  function restoreCaret(range) {
    if (!range || !window.getSelection) return;
    try {
      var sel = window.getSelection();
      sel.removeAllRanges();
      sel.addRange(range);
    } catch (error) { /* stale range */ }
  }

  function addPalette(body, source) {
    var section = el("div", "gb-deq-section");
    section.appendChild(label(el("h4", "gb-deq-section-title"), "docs-eq-symbols", "Symbols"));
    var grid = el("div", "gb-deq-palette");
    NS.PALETTE.forEach(function (pair) {
      var item = el("button", "gb-deq-symbol");
      item.setAttribute("type", "button");
      item.setAttribute("data-i18n-aria-label", "docs-eq-insert-symbol");
      item.setAttribute("aria-label", tr("docs-eq-insert-symbol", "Insert symbol"));
      item.textContent = pair[1];
      item.addEventListener("click", function () { appendTex(source, pair[0]); });
      grid.appendChild(item);
    });
    section.appendChild(grid);
    var tpl = el("div", "gb-deq-section");
    tpl.appendChild(label(el("h4", "gb-deq-section-title"), "docs-eq-templates", "Templates"));
    var tplGrid = el("div", "gb-deq-templates");
    NS.TEMPLATES.forEach(function (item) {
      var node = button(item.key, item.fallback, "gb-deq-template");
      node.setAttribute("data-i18n-aria-label", "docs-eq-insert-template");
      node.setAttribute("aria-label", tr("docs-eq-insert-template", "Insert template"));
      node.addEventListener("click", function () { appendTex(source, item.tex); });
      tplGrid.appendChild(node);
    });
    tpl.appendChild(tplGrid);
    body.appendChild(section);
    body.appendChild(tpl);
  }
  function appendTex(source, tex) {
    var start = typeof source.selectionStart === "number" ? source.selectionStart : source.value.length;
    var end = typeof source.selectionEnd === "number" ? source.selectionEnd : start;
    source.value = source.value.slice(0, start) + tex + source.value.slice(end);
    if (source.setSelectionRange) source.setSelectionRange(start + tex.length, start + tex.length);
    source.dispatchEvent(new Event("input", { bubbles: true }));
  }

  NS.openEditor = function (existingNode) {
    var caret = existingNode ? null : savedCaret();
    var current = existingNode ? NS.read(existingNode) : null;
    var dialog = openDialog({
      titleKey: "docs-eq-editor-title", titleFallback: "Equation editor",
      render: function (body) {
        var sourceWrap = el("label", "gb-deq-field");
        sourceWrap.appendChild(label(el("span", "gb-deq-field-label"), "docs-eq-source", "LaTeX source"));
        var source = field(el("textarea", "gb-deq-source"), "docs-eq-source-placeholder", "Type LaTeX, for example \\frac{a}{b} + x^2");
        source.rows = 3;
        source.value = current ? current.src : "";
        sourceWrap.appendChild(source);
        body.appendChild(sourceWrap);

        var options = el("div", "gb-deq-options");
        var placement = el("fieldset", "gb-deq-placement");
        var legend = label(el("legend", "gb-deq-field-label"), "docs-eq-placement", "Placement");
        placement.appendChild(legend);
        ["inline", "display"].forEach(function (kind) {
          var wrap = el("label", "gb-deq-radio");
          var input = el("input");
          input.setAttribute("type", "radio");
          input.name = "gb-deq-kind";
          input.value = kind;
          input.checked = current ? current.kind === kind : kind === "inline";
          wrap.appendChild(input);
          wrap.appendChild(label(el("span"), kind === "inline" ? "docs-eq-inline" : "docs-eq-display",
            kind === "inline" ? "Inline" : "Display"));
          placement.appendChild(wrap);
        });
        options.appendChild(placement);
        var numberedWrap = el("label", "gb-deq-check");
        var numbered = el("input");
        numbered.setAttribute("type", "checkbox");
        numbered.checked = current ? current.numbered : false;
        numberedWrap.appendChild(numbered);
        numberedWrap.appendChild(label(el("span"), "docs-eq-numbered", "Auto-number this display equation"));
        options.appendChild(numberedWrap);
        body.appendChild(options);

        var descWrap = el("label", "gb-deq-field");
        descWrap.appendChild(label(el("span", "gb-deq-field-label"), "docs-eq-description", "Alternative description"));
        var desc = field(el("input", "gb-deq-desc"), "docs-eq-description-placeholder", "Describe this expression for screen readers");
        desc.setAttribute("type", "text");
        desc.value = current ? current.description : "";
        descWrap.appendChild(desc);
        body.appendChild(descWrap);

        var previewWrap = el("div", "gb-deq-field");
        previewWrap.appendChild(label(el("span", "gb-deq-field-label"), "docs-eq-preview", "Live preview"));
        var preview = el("div", "gb-deq-preview");
        preview.setAttribute("aria-live", "polite");
        preview.innerHTML = NS.latexToMathml(source.value, false, null);
        if (!source.value.trim()) preview.textContent = tr("docs-eq-preview-empty", "Preview appears here");
        previewWrap.appendChild(preview);
        body.appendChild(previewWrap);

        source.addEventListener("input", function () {
          var value = source.value.trim();
          preview.innerHTML = value ? NS.latexToMathml(value, false, null) : tr("docs-eq-preview-empty", "Preview appears here");
        });
        addPalette(body, source);
      },
      actions: [
        { key: "docs-eq-ref-picker", fallback: "Insert equation reference", onClick: function () { dialog.close(); openReferences(); } },
        { key: "docs-eq-list-title", fallback: "Equations in this document", onClick: function () { dialog.close(); openList(); } },
        { key: "docs-eq-cancel", fallback: "Cancel", onClick: function (body, status, close) { close(); } },
        {
          key: current ? "docs-eq-update" : "docs-eq-insert",
          fallback: current ? "Update equation" : "Insert equation",
          primary: true,
          onClick: function (body, status, close) {
            var source = body.querySelector(".gb-deq-source");
            var desc = body.querySelector(".gb-deq-desc");
            var choice = body.querySelector("input[name='gb-deq-kind']:checked");
            var numbered = body.querySelector(".gb-deq-check input");
            if (!source.value.trim()) {
              setStatus(status, tr("docs-eq-status-empty-source", "Enter a LaTeX expression"));
              return;
            }
            if (!NS.isBalanced(source.value)) {
              setStatus(status, tr("docs-eq-status-invalid", "The expression could not be rendered"));
              return;
            }
            restoreCaret(caret);
            var id = NS.apply({
              src: source.value,
              description: desc.value,
              kind: choice ? choice.value : "inline",
              numbered: !!(numbered && numbered.checked)
            }, existingNode);
            if (id) close();
          }
        }
      ]
    });
  };

  function openReferences() {
    var dialog = openDialog({
      titleKey: "docs-eq-ref-select", titleFallback: "Select an equation",
      render: function (body) {
        var list = el("div", "gb-deq-list");
        var items = NS.numberedRecords();
        if (!items.length) list.appendChild(label(el("p", "gb-deq-empty"), "docs-eq-ref-none", "No numbered equations yet"));
        items.forEach(function (item) {
          var row = el("div", "gb-deq-row");
          var title = el("span", "gb-deq-row-title");
          title.textContent = "(" + item.number + ") " + item.src;
          row.appendChild(title);
          var use = button("docs-eq-ref-insert", "Insert reference", "gb-deq-row-btn");
          use.addEventListener("click", function () { NS.insertReference(item.id); dialog.close(); });
          row.appendChild(use);
          list.appendChild(row);
        });
        body.appendChild(list);
      },
      actions: [{ key: "docs-eq-close", fallback: "Close", onClick: function (body, status, close) { close(); } }]
    });
  }
  NS.openReferences = openReferences;

  function openList() {
    var dialog = openDialog({
      titleKey: "docs-eq-list-title", titleFallback: "Equations in this document",
      render: function (body) {
        body.appendChild(label(el("h4", "gb-deq-section-title"), "docs-eq-search", "Find equations"));
        var search = field(el("input", "gb-deq-search"), "docs-eq-search-placeholder", "Search LaTeX source or description");
        search.setAttribute("type", "search");
        body.appendChild(search);
        var list = el("div", "gb-deq-list");
        body.appendChild(list);
        function render() {
          while (list.firstChild) list.removeChild(list.firstChild);
          var items = NS.records(search.value);
          if (!items.length) {
            list.appendChild(label(el("p", "gb-deq-empty"), search.value ? "docs-eq-no-results" : "docs-eq-list-empty",
              search.value ? "No matching equations" : "No equations in this document"));
            return;
          }
          items.forEach(function (item) {
            var row = el("div", "gb-deq-row");
            var title = el("span", "gb-deq-row-title");
            title.textContent = (item.number ? "(" + item.number + ") " : "") + item.src;
            row.appendChild(title);
            var edit = button("docs-eq-edit", "Edit", "gb-deq-row-btn");
            edit.addEventListener("click", function () { dialog.close(); NS.openEditor(item.node); });
            var ref = button("docs-eq-reference", "Reference", "gb-deq-row-btn");
            ref.addEventListener("click", function () { NS.insertReference(item.id); });
            var del = button("docs-eq-delete", "Delete equation", "gb-deq-row-btn gb-deq-danger");
            del.addEventListener("click", function () { NS.remove(item.id); render(); });
            row.appendChild(edit);
            row.appendChild(ref);
            row.appendChild(del);
            list.appendChild(row);
          });
        }
        search.addEventListener("input", render);
        render();
      },
      actions: [
        { key: "docs-eq-renumber", fallback: "Renumber", onClick: function () { NS.renumber(); NS.statusKey("docs-eq-status-renumbered", "Equations renumbered"); } },
        { key: "docs-eq-update-refs", fallback: "Update references", onClick: function () { NS.updateReferences(); NS.statusKey("docs-eq-status-ref-updated", "References updated"); } },
        { key: "docs-eq-close", fallback: "Close", onClick: function (body, status, close) { close(); } }
      ]
    });
  }
  NS.openList = openList;
})(window);
