"use strict";
/* Built-in template gallery (#2427). The picker searches localized names and
 * previews the same placeholder content that apply() inserts into the local
 * editor. Templates are client-side samples only: no org library, versioning or
 * propagation is implemented. */
(function (window) {
  var NS = window.DocsForms;
  if (!NS) return;
  var T = (NS.templates = NS.templates || {});

  function block(kind, key, fallback, type) {
    return { kind: kind, key: key, fallback: fallback, type: type || "text" };
  }

  var TEMPLATES = [
    { id: "blank", nameKey: "docs-forms-template-blank", nameFallback: "Blank document", blocks: [] },
    { id: "letter", nameKey: "docs-forms-template-letter", nameFallback: "Formal letter", blocks: [
      block("heading", "docs-forms-template-ph-heading", "Heading"),
      block("field", "docs-forms-template-ph-recipient", "Recipient", "text"),
      block("paragraph", "docs-forms-template-ph-body", "Replace this text with your content."),
      block("field", "docs-forms-template-ph-signature", "Signature", "text")
    ] },
    { id: "meeting", nameKey: "docs-forms-template-meeting", nameFallback: "Meeting notes", blocks: [
      block("heading", "docs-forms-template-ph-heading", "Heading"),
      block("field", "docs-forms-template-ph-subject", "Subject", "text"),
      block("field", "docs-forms-template-ph-item", "Agenda item", "text"),
      block("paragraph", "docs-forms-template-ph-body", "Replace this text with your content.")
    ] },
    { id: "report", nameKey: "docs-forms-template-report", nameFallback: "Project report", blocks: [
      block("heading", "docs-forms-template-ph-heading", "Heading"),
      block("field", "docs-forms-template-ph-subject", "Subject", "text"),
      block("paragraph", "docs-forms-template-ph-body", "Replace this text with your content."),
      block("field", "docs-forms-template-ph-signature", "Signature", "text")
    ] },
    { id: "invoice", nameKey: "docs-forms-template-invoice", nameFallback: "Invoice", blocks: [
      block("heading", "docs-forms-template-ph-heading", "Heading"),
      block("field", "docs-forms-template-ph-recipient", "Recipient", "text"),
      block("field", "docs-forms-template-ph-subject", "Subject", "text"),
      block("paragraph", "docs-forms-template-ph-body", "Replace this text with your content.")
    ] }
  ];
  T.all = function () { return TEMPLATES.slice(); };

  T.name = function (template) { return NS.tr(template.nameKey, template.nameFallback); };

  T.search = function (query) {
    var needle = String(query == null ? "" : query).trim().toLowerCase();
    if (!needle) return T.all();
    return TEMPLATES.filter(function (template) {
      return T.name(template).toLowerCase().indexOf(needle) >= 0;
    });
  };

  /* Pure: returns the placeholder HTML inserted by apply(). */
  T.buildHtml = function (template) {
    if (!template || !template.blocks || !template.blocks.length) return "<p><br></p>";
    var out = [];
    template.blocks.forEach(function (item) {
      var text = NS.tr(item.key, item.fallback);
      if (item.kind === "field") {
        var descriptor = NS.controls.create(item.type, { label: text });
        out.push("<p>" + NS.controls.render(descriptor) + "</p>");
      } else if (item.kind === "heading") {
        out.push("<h2>" + NS.esc(text) + "</h2>");
      } else {
        out.push("<p>" + NS.esc(text) + "</p>");
      }
    });
    return out.join("");
  };

  T.apply = function (template) {
    var host = NS.getArticle();
    if (!host || !host.insertAdjacentHTML) return false;
    host.insertAdjacentHTML("beforeend", T.buildHtml(template));
    NS.state.selectedTemplate = template ? template.id : "";
    NS.controls.syncFromDom(host);
    NS.persist();
    NS.statusKey("docs-forms-template-inserted", "Template inserted into the document (local preview)");
    return true;
  };
})(window);
