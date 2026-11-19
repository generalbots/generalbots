"use strict";
/* Docs structured document model (#2423), boot wiring.
 *
 * Only delegated clicks and state hydration. No server calls, no innerHTML,
 * no native dialogs. */
(function (window) {
  var NS = window.DocsModel;
  if (!NS) return;

  function hydrate() {
    NS.loadState();
    NS.ensureDoc();
  }

  function boot() {
    hydrate();
    document.addEventListener("click", function (event) {
      var target = event.target && event.target.closest ? event.target.closest("[data-gb-model-open]") : null;
      if (!target) return;
      event.preventDefault();
      event.stopPropagation();
      if (NS.dialog && NS.dialog.open) NS.dialog.open();
    });
    document.addEventListener("keydown", function (event) {
      if (event.key === "Escape") return;
      if (event.ctrlKey && event.altKey && event.key.toLowerCase() === "m") {
        event.preventDefault();
        if (NS.dialog && NS.dialog.open) NS.dialog.open();
      }
    });
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", boot);
  else boot();
})(typeof window !== "undefined" ? window : globalThis);
