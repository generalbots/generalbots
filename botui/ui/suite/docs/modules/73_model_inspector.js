"use strict";
/* Docs structured document model (#2423), inspector sections.
 *
 * Tree view of blocks, audited op log, schema version, one-way projection
 * previews, paste normalisation, selection/a11y summary and per-author undo.
 * Every string is resolved through the catalog; all node text is set with
 * textContent so nothing here writes innerHTML. */
(function (window) {
  var NS = window.DocsModel;
  if (!NS) return;
  var el = NS.el, i18n = NS.i18n, tr = NS.tr;

  function section(titleKey, titleFallback) {
    var wrap = el("section", "gb-model-section");
    wrap.appendChild(i18n(el("h4", "gb-model-section-title"), titleKey, titleFallback));
    return wrap;
  }
  function line(labelKey, labelFallback, value) {
    var row = el("div", "gb-model-row");
    row.appendChild(i18n(el("span", "gb-model-label"), labelKey, labelFallback));
    var val = el("span", "gb-model-value");
    val.textContent = String(value == null ? "" : value);
    row.appendChild(val);
    return row;
  }
  function pre(text) {
    var node = el("pre", "gb-model-pre");
    node.textContent = text;
    return node;
  }
  function button(key, fallback, onClick) {
    var node = i18n(el("button", "gb-model-btn"), key, fallback);
    node.setAttribute("type", "button");
    if (onClick) node.addEventListener("click", onClick);
    return node;
  }
  function refresh(host) {
    return function () {
      host.textContent = "";
      NS.renderInspector(host);
    };
  }

  function renderMeta(host, doc) {
    var check = NS.validate(doc);
    var wrap = section("docs-model-meta", "Document metadata");
    wrap.appendChild(line("docs-model-schema-version", "Schema version", doc.schema_version));
    wrap.appendChild(line("docs-model-block-count", "Blocks", doc.blocks.length));
    wrap.appendChild(line("docs-model-comment-count", "Comments", doc.comments.length));
    wrap.appendChild(line("docs-model-validate", "Validation",
      tr(check.ok ? "docs-model-valid" : "docs-model-invalid", check.ok ? "Schema valid" : "Schema invalid")));
    host.appendChild(wrap);
  }

  function renderAuthors(host) {
    var wrap = section("docs-model-authors", "Per-author undo");
    var authors = NS.state.authors || [];
    authors.forEach(function (author) {
      var row = el("div", "gb-model-row");
      var name = el("span", "gb-model-value");
      name.textContent = author;
      row.appendChild(name);
      row.appendChild(button("docs-model-undo-for", "Undo for { $author }", function () {
        var op = NS.undo(author);
        NS.statusKey(op ? "docs-model-status-undone" : "docs-model-status-empty",
          op ? "Undone { $author }'s last operation" : "Nothing to change", { author: author });
        refresh(host)();
      }));
      row.appendChild(button("docs-model-redo-for", "Redo for { $author }", function () {
        var op = NS.redo(author);
        NS.statusKey(op ? "docs-model-status-redone" : "docs-model-status-empty",
          op ? "Redone { $author }'s last operation" : "Nothing to change", { author: author });
        refresh(host)();
      }));
      var depth = el("span", "gb-model-label");
      depth.textContent = tr("docs-model-depth", "undo { $a } / redo { $r }",
        { a: NS.undoDepth(author), r: NS.redoDepth(author) });
      depth.setAttribute("data-i18n", "docs-model-depth");
      depth.setAttribute("data-i18n-params", JSON.stringify({ a: NS.undoDepth(author), r: NS.redoDepth(author) }));
      row.appendChild(depth);
      wrap.appendChild(row);
    });
    host.appendChild(wrap);
  }

  function renderTree(host, doc) {
    var wrap = section("docs-model-tree", "Block tree");
    doc.blocks.forEach(function (block) {
      var node = el("div", "gb-model-block");
      var head = el("span", "gb-model-block-head");
      head.textContent = block.type + " · " + block.id;
      node.appendChild(head);
      (block.spans || []).forEach(function (span) {
        var s = el("div", "gb-model-span");
        s.textContent = span.id + " [" + (span.marks || []).map(function (m) { return m.type; }).join(",") + "] " + span.text;
        node.appendChild(s);
      });
      wrap.appendChild(node);
    });
    host.appendChild(wrap);
  }

  function renderOps(host) {
    var wrap = section("docs-model-ops", "Operation log");
    var ops = NS.history();
    if (!ops.length) wrap.appendChild(i18n(el("p", "gb-model-empty"), "docs-model-no-ops", "No operations yet"));
    ops.forEach(function (op) {
      var row = el("div", "gb-model-op");
      var stamp = new Date(op.timestamp).toISOString();
      row.textContent = op.seq + " · " + op.author + " · " + op.type + " · " + stamp + " · → " + op.inverse.type;
      wrap.appendChild(row);
    });
    host.appendChild(wrap);
  }

  function renderProjections(host, doc) {
    var wrap = section("docs-model-projections", "One-way projections (preview only)");
    wrap.appendChild(i18n(el("div", "gb-model-label"), "docs-model-json", "JSON view"));
    wrap.appendChild(pre(NS.toJson(doc)));
    wrap.appendChild(i18n(el("div", "gb-model-label"), "docs-model-html", "HTML projection (read-only)"));
    wrap.appendChild(pre(NS.toHtml(doc)));
    wrap.appendChild(i18n(el("div", "gb-model-label"), "docs-model-markdown", "Markdown projection (read-only)"));
    wrap.appendChild(pre(NS.toMarkdown(doc)));
    host.appendChild(wrap);
  }

  function renderPaste(host) {
    var wrap = section("docs-model-paste", "Paste normalisation preview");
    var input = el("textarea", "gb-model-paste-input");
    input.setAttribute("data-i18n-placeholder", "docs-model-paste-placeholder");
    input.placeholder = tr("docs-model-paste-placeholder", "Paste plain or structured text…");
    var go = button("docs-model-normalise", "Normalise", function () {
      var blocks = NS.normalizePaste(input.value);
      preview.textContent = blocks.map(function (b) { return b.type + " · " + b.id + " · " + NS.plainText(b); }).join("\n");
      NS.statusKey("docs-model-status-normalised", "Paste normalised into blocks");
    });
    var preview = pre("");
    wrap.appendChild(input);
    wrap.appendChild(go);
    wrap.appendChild(preview);
    host.appendChild(wrap);
  }

  function renderA11y(host, doc) {
    var wrap = section("docs-model-a11y", "Accessibility summary");
    var summary = NS.a11ySummary(doc);
    var rows = [
      ["docs-model-a11y-blocks", "Blocks", "blocks"],
      ["docs-model-a11y-headings", "Headings", "headings"],
      ["docs-model-a11y-tables", "Tables", "tables"],
      ["docs-model-a11y-spans", "Inline spans", "spans"],
      ["docs-model-a11y-marks", "Inline marks", "marks"],
      ["docs-model-a11y-comments", "Comments", "comments"]
    ];
    rows.forEach(function (row) {
      wrap.appendChild(line(row[0], row[1], summary[row[2]]));
    });
    var sel = summary.selection || {};
    wrap.appendChild(line("docs-model-selection", "Selection (block id + offset)",
      (sel.blockId || "—") + " @ " + sel.offset));
    wrap.appendChild(i18n(el("p", "gb-model-deferred"), "docs-model-deferred",
      "Deferred: DOCX/ODT round-trip, server validation, HTML migration, co-editing and 100k-word scale."));
    wrap.appendChild(i18n(el("p", "gb-model-deferred"), "docs-model-ime-deferred",
      "Deferred: IME composition and spellcheck ranges."));
    host.appendChild(wrap);
  }

  NS.renderInspector = function (host) {
    if (!host) return;
    var doc = NS.getDoc();
    host.appendChild(button("docs-model-refresh", "Refresh", refresh(host)));
    renderMeta(host, doc);
    renderAuthors(host);
    renderTree(host, doc);
    renderOps(host);
    renderProjections(host, doc);
    renderPaste(host);
    renderA11y(host, doc);
  };
})(typeof window !== "undefined" ? window : globalThis);
