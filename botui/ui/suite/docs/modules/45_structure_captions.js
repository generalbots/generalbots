"use strict";
/* Docs long-document structure (#2426), captions and generated lists.
 *
 * A caption is a non-editable <p data-gb-caption> whose kind, target id, number
 * and title are data attributes, so it survives re-serialisation and can be a
 * cross-reference target. Figure and table captions are numbered independently.
 * The generated Table of Figures / Table of Tables is built from cross-reference
 * fields, so one "update all" pass refreshes both lists and every reference.
 */
(function (window) {
  var NS = window.DocsStructure;
  if (!NS) return;

  var KINDS = ["figure", "table"];
  var TITLE_KEYS = {
    figure: "docs-structure-toc-figures",
    table: "docs-structure-toc-tables",
  };
  var TITLE_FALLBACKS = { figure: "Table of figures", table: "Table of tables" };
  var STATUS_KEYS = {
    figure: "docs-structure-status-tof-inserted",
    table: "docs-structure-status-tot-inserted",
  };
  var STATUS_FALLBACKS = {
    figure: "Table of figures inserted",
    table: "Table of tables inserted",
  };

  function attr(node, name) {
    return node && node.getAttribute ? (node.getAttribute(name) || "") : "";
  }
  function isKind(node, kind) {
    var actual = attr(node, "data-gb-caption-kind") === "table" ? "table" : "figure";
    return actual === kind;
  }
  NS.captionKind = function (node) {
    return attr(node, "data-gb-caption-kind") === "table" ? "table" : "figure";
  };

  NS.captionLabel = function (kind, number) {
    return NS.kindLabel(kind) + " " + number;
  };

  function captionHtml(kind, title, number, id) {
    var prefix = NS.captionLabel(kind, number);
    return '<p class="gb-caption" contenteditable="false"' +
      ' data-gb-caption="1"' +
      ' data-gb-caption-kind="' + kind + '"' +
      ' data-gb-caption-id="' + NS.esc(id) + '"' +
      ' data-gb-num="' + NS.esc(number) + '"' +
      ' data-gb-caption-title="' + NS.esc(title) + '">' +
      '<span class="gb-caption-prefix">' + NS.esc(prefix) + ":</span> " +
      '<span class="gb-caption-text">' + NS.esc(title) + "</span></p>";
  }
  NS.captionHtml = captionHtml;

  function captions(root) {
    var all = root && root.querySelectorAll ? root.querySelectorAll("[data-gb-caption]") : [];
    return Array.prototype.slice.call(all);
  }
  NS.captions = captions;

  NS.nextNumber = function (kind) {
    var count = captions(NS.getArticle()).filter(function (node) { return isKind(node, kind); }).length;
    return String(count + 1);
  };

  NS.insertCaption = function (kind, title) {
    if (!NS.getArticle()) {
      NS.statusKey("docs-structure-status-no-article", "Open a document first");
      return null;
    }
    var actual = KINDS.indexOf(kind) >= 0 ? kind : "figure";
    var id = NS.uid(actual === "figure" ? "fig" : "tbl");
    var number = NS.nextNumber(actual);
    if (!NS.commitNode(captionHtml(actual, String(title || ""), number, id))) return null;
    NS.statusKey("docs-structure-status-caption-inserted", "Caption inserted");
    return id;
  };

  /* Renumber captions per kind and refresh their rendered prefix. */
  NS.renumberCaptions = function (root) {
    var counters = { figure: 0, table: 0 };
    captions(root).forEach(function (node) {
      var kind = NS.captionKind(node);
      counters[kind] = (counters[kind] || 0) + 1;
      node.setAttribute("data-gb-num", String(counters[kind]));
      var prefix = node.querySelector ? node.querySelector(".gb-caption-prefix") : null;
      if (prefix) prefix.textContent = NS.captionLabel(kind, counters[kind]) + ":";
    });
  };

  function listHtml(root, kind) {
    var list = captions(root).filter(function (node) { return isKind(node, kind); });
    if (!list.length) return "";
    var targets = NS.targetsFrom(root);
    var html = '<div class="gb-structure-list" data-gb-structure-list="' + kind + '">' +
      '<p class="gb-structure-list-title">' +
      NS.esc(NS.tr(TITLE_KEYS[kind], TITLE_FALLBACKS[kind])) + "</p>";
    list.forEach(function (node) {
      var id = attr(node, "data-gb-caption-id");
      var target = NS.findTarget(targets, id, kind);
      var line = target
        ? NS.fieldHtml(target, "full")
        : "<span>" + NS.esc(attr(node, "data-gb-caption-title")) + "</span>";
      html += '<p class="gb-structure-list-entry">' + line + "</p>";
    });
    return html + "</div>";
  }
  NS.listHtml = listHtml;

  NS.insertList = function (kind) {
    var a = NS.getArticle();
    if (!a) {
      NS.statusKey("docs-structure-status-no-article", "Open a document first");
      return false;
    }
    var actual = KINDS.indexOf(kind) >= 0 ? kind : "figure";
    NS.renumberCaptions(a);
    var html = listHtml(a, actual);
    if (!html) {
      NS.statusKey("docs-structure-status-list-empty", "Add a caption first");
      return false;
    }
    if (!NS.commitNode(html)) return false;
    NS.statusKey(STATUS_KEYS[actual], STATUS_FALLBACKS[actual]);
    return true;
  };

  /* Regenerate lists that are already in the document; called on the clone
   * during the update-all pass so no second undo step is created. */
  NS.refreshListsIn = function (root) {
    KINDS.forEach(function (kind) {
      var old = root.querySelector
        ? root.querySelector('[data-gb-structure-list="' + kind + '"]')
        : null;
      if (!old) return;
      var html = listHtml(root, kind);
      if (!html) return;
      var parent = old.parentNode;
      var next = old.nextSibling;
      if (!parent) return;
      var doc = root.ownerDocument || document;
      var node = NS.firstNode(html, doc);
      if (!node) return;
      parent.removeChild(old);
      parent.insertBefore(node, next || null);
    });
  };
})(window);
