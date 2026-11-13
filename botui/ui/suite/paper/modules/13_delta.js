"use strict";
/* #2172/#2175 — gb-doc-delta for paper (block model). */
(function () {
  // Markdown-lite → blocks. Only the constructs the plan prompt promises:
  // '# '/'## ' headings, '- ' bullets, '1. ' numbered items, blank-line
  // paragraphs, **bold**.
  function paperMarkdown(text) {
    var html = "";
    var list = null;
    function closeList() { if (list) { html += "</" + list + ">"; list = null; } }
    String(text).split(/\r?\n/).forEach(function (line) {
      var trimmed = line.trim();
      if (!trimmed) { closeList(); return; }
      var bold = function (value) {
        return value.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>");
      };
      if (/^#{1,3}\s+/.test(trimmed)) {
        closeList();
        html += "<h2>" + bold(trimmed.replace(/^#{1,3}\s+/, "")) + "</h2>";
      } else if (/^[-*]\s+/.test(trimmed)) {
        if (list !== "ul") { closeList(); html += "<ul>"; list = "ul"; }
        html += "<li>" + bold(trimmed.replace(/^[-*]\s+/, "")) + "</li>";
      } else if (/^\d+[.)]\s+/.test(trimmed)) {
        if (list !== "ol") { closeList(); html += "<ol>"; list = "ol"; }
        html += "<li>" + bold(trimmed.replace(/^\d+[.)]\s+/, "")) + "</li>";
      } else {
        closeList();
        html += "<p>" + bold(trimmed) + "</p>";
      }
    });
    closeList();
    return html;
  }

  function apply(payload) {
    // The document body is #editor-content; `[contenteditable=true]` alone
    // matched #paper-title first, so every block landed in the title.
    var root = document.getElementById("editor-content") ||
      document.querySelector(".editor-content") ||
      document.querySelector("#paper-editor, .paper-editor");
    if (!root) return;
    (payload.ops || []).forEach(function (op) {
      if (op.op === "append_block" || op.op === "update_block") {
        var el = document.createElement(op.type === "heading" ? "h2" : "p");
        if (op.id) el.setAttribute("data-delta-id", op.id);
        el.textContent = (typeof op.text === "string") ? op.text : (Array.isArray(op.runs) ? op.runs.map(function(r){return typeof r==="string"?r:(r.text||"");}).join("") : "");
        var ex = op.id && root.querySelector('[data-delta-id="'+op.id+'"]');
        if (ex) ex.replaceWith(el); else root.appendChild(el);
      }
      if (op.op === "delete_block" && op.id) { var d=root.querySelector('[data-delta-id="'+op.id+'"]'); if(d)d.remove(); }
      if (op.op === "clear") root.innerHTML = "";
      // The web plan sends the docs-style semantic fill
      // ({"op":"fill","field":"document body","value":"<markdown-lite>"}); paper
      // only understood append_block, so the planner failed with
      // "Field 'document body' not found" and the note stayed empty (#2186).
      if (op.op === "fill") {
        var field = String(op.field || "").toLowerCase();
        var target = field.indexOf("title") >= 0
          ? (document.getElementById("paper-title") || document.querySelector("#paper-title, .paper-title"))
          : root;
        if (target) {
          if (target === root) {
            target.innerHTML = paperMarkdown(op.value || "");
          } else {
            target.textContent = String(op.value || "");
          }
        }
      }
    });
  }
  if (window.GBDocDelta) window.GBDocDelta.register("paper", apply);
  else document.addEventListener("DOMContentLoaded", function(){ if(window.GBDocDelta) window.GBDocDelta.register("paper", apply); });
})();
