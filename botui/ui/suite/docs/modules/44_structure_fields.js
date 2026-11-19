"use strict";
/* Docs long-document structure (#2426), general field engine.
 *
 * A field is a non-editable <span data-gb-field="xref"> carrying its target,
 * kind and display format as data attributes plus a rendered label span. Targets
 * are headings, bookmarks, figure captions and table captions; labels resolve to
 * the current number and/or text and refresh in one pass. Page-number fields
 * are intentionally absent: there is no page model, so no page value is ever
 * fabricated. Equation references (modules 38-41) use a different class and are
 * not touched here.
 */
(function (window) {
  var NS = window.DocsStructure;
  if (!NS) return;

  var KIND_KEYS = {
    heading: "docs-structure-kind-heading",
    bookmark: "docs-structure-kind-bookmark",
    figure: "docs-structure-kind-figure",
    table: "docs-structure-kind-table",
  };
  var KIND_FALLBACKS = {
    heading: "Heading", bookmark: "Bookmark", figure: "Figure", table: "Table",
  };
  var FORMATS = ["number", "text", "full"];

  function kindKey(kind) { return KIND_KEYS[kind] || KIND_KEYS.heading; }
  NS.kindKey = kindKey;
  NS.kindLabel = function (kind) { return NS.tr(kindKey(kind), KIND_FALLBACKS[kind] || KIND_FALLBACKS.heading); };
  NS.formats = function () { return FORMATS.slice(); };

  function attr(node, name) {
    return node && node.getAttribute ? (node.getAttribute(name) || "") : "";
  }
  function cleanText(node) { return ((node && node.textContent) || "").trim(); }

  function pushTarget(list, node, id, spec) {
    if (!id) return;
    list.push({
      id: id,
      kind: spec.kind,
      number: attr(node, "data-gb-num"),
      text: attr(node, "data-gb-title") || attr(node, "data-gb-caption-title") ||
        attr(node, "data-gb-bookmark") || cleanText(node),
      prefix: spec.kind === "figure" || spec.kind === "table" ? NS.kindLabel(spec.kind) : "",
      node: node,
    });
  }

  var TARGET_SPECS = [
    { selector: "h1,h2,h3,h4,h5,h6", kind: "heading" },
    { selector: "[data-gb-bookmark]", kind: "bookmark" },
    { selector: "[data-gb-caption]", kind: "figure" },
  ];

  NS.targetsFrom = function (root) {
    var out = [];
    if (!root || !root.querySelectorAll) return out;
    TARGET_SPECS.forEach(function (spec) {
      var nodes = Array.prototype.slice.call(root.querySelectorAll(spec.selector));
      nodes.forEach(function (node, index) {
        var kind = spec.kind;
        if (kind === "figure" && attr(node, "data-gb-caption-kind") === "table") kind = "table";
        var id = kind === "heading"
          ? (attr(node, "id") || attr(node, "data-gb-toc-anchor") || "sec-" + (index + 1))
          : (kind === "bookmark" ? attr(node, "id") : attr(node, "data-gb-caption-id"));
        pushTarget(out, node, id, { kind: kind });
      });
    });
    return out;
  };

  NS.targets = function () { return NS.targetsFrom(NS.getArticle()); };

  /* Pure resolver; `page` deliberately resolves to an empty string. */
  NS.label = function (target, format) {
    if (!target || format === "page") return "";
    var prefix = target.prefix || "";
    var number = target.number || "";
    var text = target.text || "";
    if (format === "text") return text;
    if (format === "number") return number ? (prefix ? prefix + " " + number : number) : text;
    if (number && text) {
      return prefix ? prefix + " " + number + ": " + text : number + " " + text;
    }
    return text || number || prefix;
  };

  function fieldHtml(target, format) {
    var fmt = FORMATS.indexOf(format) >= 0 ? format : "full";
    var text = NS.label(target, fmt);
    return '<span class="gb-field" contenteditable="false" data-gb-field="xref"' +
      ' data-gb-field-target="' + NS.esc(target.id) + '"' +
      ' data-gb-field-kind="' + NS.esc(target.kind) + '"' +
      ' data-gb-field-format="' + fmt + '">' +
      '<span class="gb-field-label">' + NS.esc(text) + "</span></span>";
  }
  NS.fieldHtml = fieldHtml;

  function findTarget(list, id, kind) {
    for (var i = 0; i < list.length; i++) {
      if (list[i].id === id && (!kind || list[i].kind === kind)) return list[i];
    }
    return null;
  }
  NS.findTarget = findTarget;

  NS.insertXref = function (id, format, kind) {
    var target = findTarget(NS.targets(), id, kind);
    if (!target) {
      NS.statusKey("docs-structure-status-no-target", "Select a reference target first");
      return false;
    }
    if (!NS.commitNode(fieldHtml(target, format))) return false;
    NS.statusKey("docs-structure-status-xref-inserted", "Cross-reference inserted");
    return true;
  };

  function fieldNodes(root) {
    var all = root && root.querySelectorAll ? root.querySelectorAll("[data-gb-field]") : [];
    return Array.prototype.slice.call(all).filter(function (node) {
      return attr(node, "data-gb-field") === "xref";
    });
  }
  NS.fieldNodes = fieldNodes;

  /* Recompute every field label against the (already renumbered) root. */
  NS.updateIn = function (root) {
    var map = NS.targetsFrom(root);
    fieldNodes(root).forEach(function (field) {
      var target = findTarget(map, attr(field, "data-gb-field-target"), attr(field, "data-gb-field-kind"));
      var labelEl = field.querySelector ? field.querySelector(".gb-field-label") : null;
      if (!labelEl) labelEl = field.firstChild;
      if (!labelEl) return;
      labelEl.textContent = target
        ? NS.label(target, attr(field, "data-gb-field-format") || "full")
        : "";
    });
  };

  NS.updateFields = function () {
    if (!NS.getArticle()) {
      NS.statusKey("docs-structure-status-no-article", "Open a document first");
      return false;
    }
    NS.commitArticle(NS.updateIn);
    NS.statusKey("docs-structure-status-updated", "All fields updated");
    return true;
  };

  NS.editField = function (field, id, format, kind) {
    if (!field) return false;
    var target = findTarget(NS.targets(), id, kind);
    if (!target) {
      NS.statusKey("docs-structure-status-no-target", "Select a reference target first");
      return false;
    }
    if (!NS.commitNode(fieldHtml(target, format), field)) return false;
    NS.statusKey("docs-structure-status-field-updated", "Field updated");
    return true;
  };
})(window);
