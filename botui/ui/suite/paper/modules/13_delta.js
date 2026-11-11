"use strict";
/* #2172/#2175 — gb-doc-delta for paper (block model). */
(function () {
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
    });
  }
  if (window.GBDocDelta) window.GBDocDelta.register("paper", apply);
  else document.addEventListener("DOMContentLoaded", function(){ if(window.GBDocDelta) window.GBDocDelta.register("paper", apply); });
})();
