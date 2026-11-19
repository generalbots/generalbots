"use strict";
/* Docs enterprise review workflow (#2425), boot wiring.
 *
 * Only event delegation and initial state hydration. All review logic lives in
 * the model/UI modules above; nothing here restarts a server or persists
 * anything outside document-scoped localStorage. */
(function (window) {
  var NS = window.DocsReview;
  if (!NS) return;

  function byId(id) { return document.getElementById ? document.getElementById(id) : null; }

  function hydrate() {
    NS.loadState();
    NS.applyModeClass(NS.getArticle());
    NS.renderMarkup();
    var suggest = byId("docsReviewSuggestBtn");
    if (suggest) suggest.setAttribute("aria-pressed", NS.state.suggestionMode ? "true" : "false");
  }

  function boot() {
    hydrate();
    document.addEventListener("click", function (event) {
      var target = event.target && event.target.closest ? event.target.closest("[data-gb-review-open]") : null;
      if (!target) return;
      event.preventDefault();
      event.stopPropagation();
      var action = target.getAttribute("data-gb-review-open");
      if (action === "panel") NS.openPanel();
      else if (action === "comments") NS.openPanel();
      else if (action === "compare") NS.openCompare();
      else if (action === "dashboard") NS.openDashboard();
      else if (action === "permissions") NS.openPermissions();
    });
    document.addEventListener("click", function (event) {
      var button = event.target && event.target.closest
        ? event.target.closest("#docsReviewSuggestBtn") : null;
      if (!button) return;
      event.preventDefault();
      var on = NS.toggleSuggestionMode();
      button.setAttribute("aria-pressed", on ? "true" : "false");
    });
    document.addEventListener("keydown", function (event) {
      if (event.key === "Escape") return;
      if (event.ctrlKey && event.altKey && event.key.toLowerCase() === "r") {
        event.preventDefault();
        NS.openPanel();
      }
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})(window);
