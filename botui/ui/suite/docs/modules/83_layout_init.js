"use strict";
/* Docs layout preview (#2424), boot wiring.
 *
 * Delegated toolbar clicks only. No server calls, no innerHTML, no native
 * dialogs. Recomputes on boot from the document-scoped projection. */
(function (window) {
  var NS = window.DocsLayout;
  if (!NS) return;

  function hydrate() {
    NS.loadState();
    if (!NS.state.lines.length) {
      var model = window.DocsModel && typeof window.DocsModel.getDoc === "function"
        ? window.DocsModel.getDoc() : null;
      NS.state.lines = model && model.blocks && model.blocks.length
        ? NS.linesFromBlocks(model.blocks) : NS.sampleLines();
    }
    NS.compute();
    NS.persist();
  }

  function boot() {
    hydrate();
    document.addEventListener("click", function (event) {
      var target = event.target && event.target.closest
        ? event.target.closest("[data-gb-layout-open]") : null;
      if (!target) return;
      event.preventDefault();
      event.stopPropagation();
      var kind = target.getAttribute("data-gb-layout-open");
      if (kind === "inspector") {
        if (NS.openInspector) NS.openInspector();
      } else if (NS.openPages) {
        NS.openPages();
      }
    });
    document.addEventListener("keydown", function (event) {
      if (event.key === "Escape") return;
      if (event.ctrlKey && event.altKey && event.key.toLowerCase() === "l") {
        event.preventDefault();
        if (NS.openPages) NS.openPages();
      }
    });
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", boot);
  else boot();
})(typeof window !== "undefined" ? window : globalThis);
