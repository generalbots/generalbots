"use strict";
/* Docs structured document model (#2423), one-way projections + paste.
 *
 * HTML and Markdown here are READ-ONLY previews generated FROM the schema,
 * never a source of truth. The strings are rendered into <pre> nodes with
 * textContent, so this module never writes innerHTML. There is no DOCX/ODT
 * round-trip, HTML-body migration or server validation in this slice. */
(function (window) {
  var NS = window.DocsModel;
  if (!NS) return;

  function inline(span, asMarkdown) {
    var text = NS.esc(span.text);
    if (asMarkdown) return markdownInline(span);
    var types = (span.marks || []).map(function (mark) { return mark.type; });
    var out = text;
    if (types.indexOf("code") >= 0) out = "<code>" + out + "</code>";
    if (types.indexOf("bold") >= 0) out = "<strong>" + out + "</strong>";
    if (types.indexOf("italic") >= 0) out = "<em>" + out + "</em>";
    if (types.indexOf("underline") >= 0) out = "<u>" + out + "</u>";
    if (types.indexOf("strike") >= 0) out = "<s>" + out + "</s>";
    (span.marks || []).forEach(function (mark) {
      if (mark.type === "link") out = '<a href="' + NS.esc(mark.value || "#") + '">' + out + "</a>";
      if (mark.type === "field") out = '<span data-field="' + NS.esc(mark.value || "") + '">' + out + "</span>";
    });
    return out;
  }

  function markdownInline(span) {
    var out = span.text.replace(/[\\*_`]/g, "\\$&");
    var types = (span.marks || []).map(function (mark) { return mark.type; });
    if (types.indexOf("code") >= 0) out = "`" + out + "`";
    if (types.indexOf("bold") >= 0) out = "**" + out + "**";
    if (types.indexOf("italic") >= 0) out = "*" + out + "*";
    if (types.indexOf("strike") >= 0) out = "~~" + out + "~~";
    (span.marks || []).forEach(function (mark) {
      if (mark.type === "link") out = "[" + out + "](" + (mark.value || "#") + ")";
      if (mark.type === "field") out = "{" + out + "}";
    });
    return out;
  }

  function spansHtml(block) {
    return (block.spans || []).map(function (span) { return inline(span, false); }).join("");
  }
  function spansMd(block) {
    return (block.spans || []).map(function (span) { return inline(span, true); }).join("");
  }

  function toHtml(doc) {
    return (doc.blocks || []).map(function (block) {
      var content = spansHtml(block);
      switch (block.type) {
        case "heading": return "<h2>" + content + "</h2>";
        case "list": return "<li>" + content + "</li>";
        case "table": return "<table><tr><td>" + content + "</td></tr></table>";
        case "image": return '<img alt="' + NS.esc(content) + '" />';
        case "footnote-reference": return '<sup data-footnote="' + NS.esc(content) + '">' + content + "</sup>";
        case "page-break": return '<hr data-page-break="true" />';
        case "section-break": return '<hr data-section-break="true" />';
        default: return "<p>" + content + "</p>";
      }
    }).join("\n");
  }
  NS.toHtml = toHtml;

  function toMarkdown(doc) {
    return (doc.blocks || []).map(function (block) {
      var content = spansMd(block);
      switch (block.type) {
        case "heading": return "## " + content;
        case "list": return "- " + content;
        case "table": return "| " + content + " |";
        case "image": return "![" + content + "]()";
        case "footnote-reference": return "[^" + content + "]";
        case "page-break": return "---";
        case "section-break": return "***";
        default: return content;
      }
    }).join("\n");
  }
  NS.toMarkdown = toMarkdown;

  NS.toJson = function (doc) { return JSON.stringify(doc || NS.getDoc(), null, 2); };

  /* ---- paste normalisation: plain/structured text -> typed blocks -------- */
  function normalizePaste(text) {
    var lines = String(text == null ? "" : text).replace(/\r\n?/g, "\n").split("\n");
    var blocks = [], buffer = [];
    function flush() {
      if (buffer.length) {
        blocks.push(NS.createBlock("paragraph", buffer.join(" ")));
        buffer = [];
      }
    }
    lines.forEach(function (raw) {
      var line = raw.trim();
      if (!line) { flush(); return; }
      if (line.indexOf("# ") === 0) { flush(); blocks.push(NS.createBlock("heading", line.slice(2))); return; }
      if (line.indexOf("- ") === 0 || line.indexOf("* ") === 0) { flush(); blocks.push(NS.createBlock("list", line.slice(2))); return; }
      buffer.push(line);
    });
    flush();
    return blocks.length ? blocks : [NS.createBlock("paragraph", "")];
  }
  NS.normalizePaste = normalizePaste;

  /* ---- accessibility + selection summary -------------------------------- */
  function a11ySummary(doc) {
    var headings = 0, spans = 0, marks = 0, tables = 0;
    (doc.blocks || []).forEach(function (block) {
      if (block.type === "heading") headings += 1;
      if (block.type === "table") tables += 1;
      (block.spans || []).forEach(function (span) {
        spans += 1;
        marks += (span.marks || []).length;
      });
    });
    return {
      blocks: (doc.blocks || []).length,
      headings: headings,
      tables: tables,
      spans: spans,
      marks: marks,
      comments: (doc.comments || []).length,
      selection: NS.state.selection
    };
  }
  NS.a11ySummary = a11ySummary;
})(typeof window !== "undefined" ? window : globalThis);
