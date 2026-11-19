"use strict";
/* Docs large .docx load path (#2533), boot wiring.
 *
 * Delegated toolbar clicks open the local-only panel. Edits to a mounted block
 * enqueue a changed-block op so the incremental save display reflects real
 * interactions. No server calls, no native dialogs, no innerHTML. */
(function (window) {
  var NS = window.DocsLarge;
  if (!NS) return;

  function boot() {
    document.addEventListener("click", function (event) {
      var target = event.target && event.target.closest
        ? event.target.closest("[data-gb-large-open]") : null;
      if (!target) return;
      event.preventDefault();
      event.stopPropagation();
      if (target.getAttribute("data-gb-large-open") === "panel" && NS.dialog && NS.dialog.open) {
        NS.dialog.open();
      }
    });

    document.addEventListener("input", function (event) {
      var node = event.target;
      if (!node || !node.classList || !node.classList.contains("gb-large-block")) return;
      var blockId = node.getAttribute("data-block-id");
      if (!blockId) return;
      NS.Save.enqueue(NS.state, blockId, (node.textContent || "").length);
      NS.persist();
      if (NS.dialog && NS.dialog.refresh) NS.dialog.refresh();
    });
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", boot);
  else boot();
})(typeof window !== "undefined" ? window : globalThis);
