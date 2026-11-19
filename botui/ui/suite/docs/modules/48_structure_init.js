"use strict";
/* Docs long-document structure (#2426), update-all pass and boot wiring.
 *
 * One pass renumbers headings, renumbers captions, regenerates any tables of
 * figures/tables already in the document and refreshes every cross-reference
 * field, all inside a single commitArticle() call (one undo step).
 */
(function (window) {
  var NS = window.DocsStructure;
  if (!NS) return;

  NS.refreshIn = function (root) {
    NS.renderNumbering(root);
    NS.renumberCaptions(root);
    NS.refreshListsIn(root);
    NS.updateIn(root);
  };

  NS.updateAll = function () {
    if (!NS.getArticle()) {
      NS.statusKey("docs-structure-status-no-article", "Open a document first");
      return false;
    }
    NS.commitArticle(NS.refreshIn);
    NS.statusKey("docs-structure-status-updated", "All fields updated");
    return true;
  };

  function boot() {
    document.addEventListener("click", function (event) {
      var opener = event.target && event.target.closest
        ? event.target.closest("[data-gb-structure-open]")
        : null;
      if (!opener) return;
      event.preventDefault();
      event.stopPropagation();
      NS.openDialog();
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})(window);
